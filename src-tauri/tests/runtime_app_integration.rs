use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use c4os_lib::core::database::{
    ChatRecord, DatabaseActor, DatabaseDescriptor, LifecycleState, ProjectPathState, ProjectRecord,
    RuntimeStateDocumentRecord, WorkspaceRecord,
};
use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, DraftRequirements, InstalledResourcePreflight,
    LimitConfidence, ModelLifecycle, NumericCapabilityEvidence, NumericCapabilityKey,
    PolicyPreflight, PreflightOutcome, RouteIdentity,
};
use c4os_lib::runtime::capability_evidence::{
    CapabilityEvidenceError, CapabilityRouteEpoch, FeatureClaim, NumericClaim,
    ProviderDeclaredCatalogClaim, RuntimeObservationOutcome, RuntimeRouteObservation,
    opencode_adapter_evidence, opencode_observed_evidence, provider_declared_evidence,
};
use c4os_lib::runtime::dispatch::{
    AttachmentPreflightResolution, CoordinatedFirstDispatch, DispatchIdentity,
    FirstDispatchOptions, PeerDispatchError, PeerDispatchEvent, PeerDispatchRequest,
    RuntimeDispatchPeer, RuntimePeerRegistration,
};
use c4os_lib::runtime::dispatch_authority::{
    AuthoritativeConfiguration, AuthorityMintIntent, ConfigurationFieldValue,
    authoritative_configuration_sha256, authoritative_resources_sha256,
    authoritative_route_configuration,
};
use c4os_lib::runtime::opencode::{HealthSnapshot, OpenCodeCompatibilityManifest};
use c4os_lib::runtime::persistence::SqliteSessionRepository;
use c4os_lib::runtime::pi::PI_NATIVE_VERSION;
use c4os_lib::runtime::provider::{
    ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
    ProviderConnectionEvidence, ProviderDiscovery, ProviderEndpoint, ProviderKind,
    ProviderModelDeclaration, ProviderProbe, ProviderProbeFailure, ProviderProfile,
    ProviderSnapshot, RouteAvailability,
};
use c4os_lib::runtime::session::{
    AdapterBinding, CapabilitySnapshot, ConfigurationSnapshot, ExecutionEnvironmentBinding,
    FirstSubmission, ModelRouteSnapshot, ResourceSnapshot, RunAttemptStatus,
    RuntimeKind as SessionRuntimeKind, SessionBinding, SessionRepository, SessionService,
    capability_snapshot_from_effective_descriptor,
};
use c4os_lib::runtime::supervisor::{
    HealthState, OPENCODE_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation,
    RuntimeKind, sha256_file,
};
use c4os_lib::security::credentials::CredentialVault;
use c4os_lib::{
    FirstRuntimeDispatchIntent, RuntimeApplicationError, RuntimeApplicationService,
    production_broker_authoritative_resources,
};
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;

struct FixtureProbe(ProviderDiscovery);

struct CasRacingProbe {
    discovery: ProviderDiscovery,
    database: Arc<DatabaseActor>,
    durable: ProviderSnapshot,
}

struct CapturingPeer {
    registration: RuntimePeerRegistration,
    native_models: Arc<Mutex<Vec<String>>>,
}

impl RuntimeDispatchPeer for CapturingPeer {
    fn registration(&self) -> &RuntimePeerRegistration {
        &self.registration
    }

    fn readiness(&self) -> Result<(), PeerDispatchError> {
        Ok(())
    }

    fn create_session(&mut self, _request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        Ok(())
    }

    fn activate_broker_context(
        &mut self,
        request: &PeerDispatchRequest,
    ) -> Result<(), PeerDispatchError> {
        request
            .broker_authority
            .as_ref()
            .ok_or(PeerDispatchError::Dispatch)
            .map(|_| ())
    }

    fn dispatch(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        self.native_models
            .lock()
            .expect("capture native model")
            .push(request.model.model_id.clone());
        Ok(())
    }

    fn poll_events(
        &mut self,
        _recorded_at_ms: u64,
    ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError> {
        Ok(Vec::new())
    }

    fn cancel(&mut self, _identity: &DispatchIdentity) -> Result<bool, PeerDispatchError> {
        Ok(true)
    }
}

impl ProviderProbe for FixtureProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let mut discovery = self.0.clone();
        discovery.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
            profile,
            discovery.checked_at_ms,
            digest('9'),
        )?);
        Ok(discovery)
    }
}

impl ProviderProbe for CasRacingProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let result = FixtureProbe(self.discovery.clone()).test_and_discover(profile);
        advance_runtime_document_generation(
            &self.database,
            "provider-snapshot",
            "providers",
            self.durable.generation,
            &self.durable,
            Some(self.durable.generation - 1),
            self.discovery.checked_at_ms,
        );
        result
    }
}

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn app_service(root: &Path) -> RuntimeApplicationService {
    app_database_and_service(root).1
}

fn app_database_and_service(root: &Path) -> (Arc<DatabaseActor>, RuntimeApplicationService) {
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(root.join("c4os-home"))).unwrap();
    let database = Arc::new(database);
    let service = RuntimeApplicationService::restore(Arc::clone(&database), NOW).unwrap();
    (database, service)
}

fn advance_runtime_document_generation<T: serde::Serialize>(
    database: &DatabaseActor,
    document_kind: &str,
    document_id: &str,
    generation: u64,
    snapshot: &T,
    expected_generation: Option<u64>,
    updated_at_ms: u64,
) {
    database
        .save_runtime_state_document(
            RuntimeStateDocumentRecord {
                document_kind: document_kind.into(),
                document_id: document_id.into(),
                generation,
                canonical_document: serde_json::to_string(snapshot).unwrap(),
                updated_at_ms,
            },
            expected_generation,
        )
        .unwrap();
}

fn advance_runtime_control_plane_revision(
    database: &DatabaseActor,
    revision: u64,
    expected_revision: Option<u64>,
    updated_at_ms: u64,
) {
    let canonical_document = database
        .runtime_state_document("runtime-control-plane", "runtime")
        .unwrap()
        .map(|document| {
            let mut snapshot: serde_json::Value =
                serde_json::from_str(&document.canonical_document).unwrap();
            snapshot["revision"] = revision.into();
            serde_json::to_string(&snapshot).unwrap()
        })
        .unwrap_or_else(|| {
            serde_json::json!({
                "schemaVersion": 1,
                "revision": revision,
                "supervisor": {
                    "stateGeneration": 0,
                    "records": [],
                    "events": [],
                    "traceEventsDropped": 0
                },
                "capabilities": {
                    "generation": 0,
                    "activeRoutes": {},
                    "activeProcesses": {},
                    "historicalRoutes": [],
                    "historicalRoutesDropped": 0
                }
            })
            .to_string()
        });
    database
        .save_runtime_state_document(
            RuntimeStateDocumentRecord {
                document_kind: "runtime-control-plane".into(),
                document_id: "runtime".into(),
                generation: revision,
                canonical_document,
                updated_at_ms,
            },
            expected_revision,
        )
        .unwrap();
}

fn provider_profile() -> ProviderProfile {
    let vault = CredentialVault::session_only().unwrap();
    ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "provider-openrouter".into(),
        kind: ProviderKind::OpenRouter,
        display_name: "OpenRouter".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openrouter-chat".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            api_kind: "openai-compatible".into(),
        },
        credential_reference: vault.store("provider-key", b"fixture-secret").unwrap(),
        enabled: true,
    }
}

fn capability_route() -> RouteIdentity {
    let mut route = RouteIdentity {
        provider_id: "provider-openrouter".into(),
        endpoint_id: "openrouter-chat".into(),
        provider_model_id: "anthropic/claude-sonnet".into(),
        model_revision: "2026-07-19".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: OPENCODE_NATIVE_VERSION.into(),
        session_configuration_sha256: digest('0'),
    };
    route.session_configuration_sha256 =
        authoritative_configuration_sha256(&authoritative_route_configuration(&route).unwrap())
            .unwrap();
    route
}

fn feature_claim(state: CapabilityState) -> FeatureClaim {
    FeatureClaim {
        state,
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: (state != CapabilityState::Supported)
            .then(|| "producer reports this capability unavailable".into()),
    }
}

fn numeric_claim(maximum: u64) -> NumericClaim {
    NumericClaim {
        state: CapabilityState::Supported,
        maximum: Some(maximum),
        confidence: LimitConfidence::Confirmed,
        reason: None,
    }
}

fn provider_capability_claim(route: RouteIdentity) -> ProviderDeclaredCatalogClaim {
    ProviderDeclaredCatalogClaim {
        route,
        lifecycle: ModelLifecycle::Active,
        declared_at_ms: NOW,
        expires_at_ms: NOW + 60_000,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::InputImage,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::OutputText,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::Streaming,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::ToolCalling,
                feature_claim(CapabilityState::Supported),
            ),
        ]),
        numeric_limits: BTreeMap::from([
            (NumericCapabilityKey::InputTokens, numeric_claim(200_000)),
            (NumericCapabilityKey::OutputTokens, numeric_claim(8_000)),
        ]),
        raw_catalog_sha256: digest('b'),
    }
}

fn adapter_evidence(state: CapabilityState) -> CapabilityEvidence {
    CapabilityEvidence {
        state,
        layer: CapabilityLayer::AdapterNormalized,
        source: "opencode.1.18.3".into(),
        checked_at_ms: NOW,
        expires_at_ms: Some(NOW + 60_000),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: (state != CapabilityState::Supported)
            .then(|| "OpenCode inventory reports this capability unavailable".into()),
    }
}

fn provider_model_route(route: RouteIdentity) -> ModelRoute {
    let supported = || adapter_evidence(CapabilityState::Supported);
    ModelRoute {
        model_id: "claude-sonnet".into(),
        display_name: "Claude Sonnet".into(),
        recommendation_rank: 0,
        availability: RouteAvailability::Available,
        checked_at_ms: NOW,
        capabilities: CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::AdapterNormalized,
            route,
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::from([
                (CapabilityKey::InputText, supported()),
                (
                    CapabilityKey::InputImage,
                    adapter_evidence(CapabilityState::Unsupported),
                ),
                (CapabilityKey::OutputText, supported()),
                (CapabilityKey::Streaming, supported()),
                (CapabilityKey::ToolCalling, supported()),
            ]),
            numeric_limits: BTreeMap::from([
                (
                    NumericCapabilityKey::InputTokens,
                    NumericCapabilityEvidence {
                        evidence: supported(),
                        maximum: Some(128_000),
                        confidence: LimitConfidence::Confirmed,
                    },
                ),
                (
                    NumericCapabilityKey::OutputTokens,
                    NumericCapabilityEvidence {
                        evidence: supported(),
                        maximum: Some(4_000),
                        confidence: LimitConfidence::Confirmed,
                    },
                ),
            ]),
            raw_evidence_sha256: digest('c'),
        },
        provider_declaration: Some(ProviderModelDeclaration {
            schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: "claude-sonnet".into(),
            model_revision: "2026-07-19".into(),
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_catalog_sha256: digest('b'),
            declared_at_ms: NOW,
            expires_at_ms: NOW + 60_000,
        }),
    }
}

fn runtime_observation(route: RouteIdentity, process_generation: u64) -> RuntimeRouteObservation {
    RuntimeRouteObservation {
        runtime_id: "opencode-primary".into(),
        route,
        process_generation,
        health_checked_at_ms: NOW + 1,
        observed_at_ms: NOW + 2,
        expires_at_ms: NOW + 30_000,
        outcome: RuntimeObservationOutcome::Available,
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([
            (
                CapabilityKey::InputText,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::InputImage,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::OutputText,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::Streaming,
                feature_claim(CapabilityState::Supported),
            ),
            (
                CapabilityKey::ToolCalling,
                feature_claim(CapabilityState::Supported),
            ),
        ]),
        numeric_limits: BTreeMap::from([
            (NumericCapabilityKey::InputTokens, numeric_claim(100_000)),
            (NumericCapabilityKey::OutputTokens, numeric_claim(2_000)),
        ]),
        raw_observation_sha256: digest('d'),
    }
}

fn capability_draft() -> DraftRequirements {
    DraftRequirements {
        attachments: Vec::new(),
        reasoning_mode: None,
        requires_tools: false,
        requires_json_schema: false,
        prefers_streaming: true,
        estimated_input_tokens: 100,
        requested_output_tokens: 100,
        installed_resources: InstalledResourcePreflight {
            snapshot_id: "resources-1".into(),
            snapshot_sha256: digest('e'),
            tool_ids: BTreeSet::new(),
            attachment_converters: BTreeSet::new(),
        },
        policy: PolicyPreflight {
            snapshot_id: "policy-1".into(),
            version: 1,
            tool_use_allowed: false,
            attachment_conversion_allowed: false,
        },
    }
}

fn runtime_installation(root: &Path) -> RuntimeInstallation {
    RuntimeInstallation {
        runtime_id: "opencode-primary".into(),
        workspace_id: "workspace-1".into(),
        runtime_kind: RuntimeKind::OpenCode,
        native_version: OPENCODE_NATIVE_VERSION.into(),
        adapter_version: "1.0.0".into(),
        protocol_version: RUNTIME_PROTOCOL_VERSION,
        install_root: root.to_path_buf(),
        asset_tree_sha256: digest('a'),
        executable: root.join("bin/opencode"),
        executable_sha256: digest('e'),
        state_namespace: root.join("state"),
        arguments: vec![],
        sanitized_environment: BTreeMap::new(),
    }
}

fn runnable_runtime_installation(root: &Path, started_marker: &Path) -> RuntimeInstallation {
    let executable = root.join("bin/opencode");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        b"#!/bin/sh\n/usr/bin/touch \"$1\"\n/bin/sleep 30\n",
    )
    .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let mut installation = runtime_installation(root);
    installation.executable_sha256 = sha256_file(&executable).unwrap();
    installation.arguments = vec![started_marker.to_string_lossy().into_owned()];
    installation
}

fn pi_runtime_installation(root: &Path) -> RuntimeInstallation {
    let mut installation = runtime_installation(root);
    installation.runtime_id = "pi-primary".into();
    installation.runtime_kind = RuntimeKind::Pi;
    installation.native_version = PI_NATIVE_VERSION.into();
    installation
}

fn binding() -> SessionBinding {
    SessionBinding {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "opencode-primary".into(),
        runtime_kind: SessionRuntimeKind::OpenCode,
        adapter: AdapterBinding {
            adapter_id: "adapter-opencode".into(),
            adapter_version: "1.0.0".into(),
            native_version: OPENCODE_NATIVE_VERSION.into(),
        },
        environment: ExecutionEnvironmentBinding {
            environment_id: "local".into(),
            environment_kind: "local".into(),
            host_alias: None,
        },
        initial_model_route: ModelRouteSnapshot {
            route_id: "route-claude".into(),
            provider_id: "provider-openrouter".into(),
            endpoint_id: "openrouter-chat".into(),
            model_id: "claude-sonnet".into(),
            model_revision: "2026-07-01".into(),
        },
        initial_configuration: ConfigurationSnapshot {
            snapshot_id: "configuration-1".into(),
            version: 1,
            sha256: digest('a'),
        },
        initial_resources: ResourceSnapshot {
            snapshot_id: "resources-1".into(),
            version: 1,
            sha256: digest('b'),
            resource_ids: vec![],
        },
        initial_capabilities: binding_capabilities(),
        bound_at_ms: NOW + 1,
    }
}

fn binding_capabilities() -> CapabilitySnapshot {
    capability_snapshot_from_effective_descriptor(
        CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::Effective,
            route: RouteIdentity {
                provider_id: "provider-openrouter".into(),
                endpoint_id: "openrouter-chat".into(),
                provider_model_id: "claude-sonnet".into(),
                model_revision: "2026-07-01".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: OPENCODE_NATIVE_VERSION.into(),
                session_configuration_sha256: digest('a'),
            },
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_evidence_sha256: digest('c'),
        },
        1,
    )
    .expect("exact binding capabilities")
}

fn seed_workspace(database: &DatabaseActor) {
    database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-1".into(),
            display_name: "C4OS".into(),
            created_at: 1,
            updated_at: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_project(ProjectRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            display_name: "Project One".into(),
            current_path: "/tmp/project-1".into(),
            last_known_path: "/tmp/project-1".into(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_chat(ChatRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            chat_id: "session-1".into(),
            title: "New Chat".into(),
            created_at: 1,
            updated_at: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
}

#[test]
fn app_runtime_is_fail_closed_until_a_real_workspace_database_is_bound() {
    let temporary = TempDir::new().unwrap();
    let service = app_service(temporary.path());

    let unavailable = service.create_provisional(0, "session-1", NOW + 1);
    assert!(unavailable.is_err());
    assert_eq!(service.snapshot(NOW + 1).unwrap().generation, 0);
    assert_eq!(service.bound_workspace_id().unwrap(), None);

    let descriptor =
        DatabaseDescriptor::workspace(temporary.path().join("workspace-active"), "workspace-1");
    let (workspace_database, _) = DatabaseActor::start(descriptor).unwrap();
    let workspace_database = Arc::new(workspace_database);
    service
        .bind_workspace(Arc::clone(&workspace_database))
        .unwrap();
    assert_eq!(
        service.bound_workspace_id().unwrap().as_deref(),
        Some("workspace-1")
    );

    let created = service.create_provisional(1, "session-1", NOW + 2).unwrap();
    assert_eq!(created.coordinator_generation, 2);
}

#[test]
fn workspace_binding_and_complete_runtime_set_publish_as_one_epoch() {
    let temporary = TempDir::new().unwrap();
    let descriptor =
        DatabaseDescriptor::workspace(temporary.path().join("workspace-active"), "workspace-1");
    let (workspace_database, _) = DatabaseActor::start(descriptor).unwrap();
    let workspace_database = Arc::new(workspace_database);
    seed_workspace(&workspace_database);
    let (database, service) = app_database_and_service(temporary.path());
    let before = service.snapshot(NOW).unwrap();

    let opencode = runtime_installation(&temporary.path().join("runtime-opencode"));
    let mut mismatched_pi = pi_runtime_installation(&temporary.path().join("runtime-pi"));
    mismatched_pi.workspace_id = "workspace-2".into();
    assert!(matches!(
        service.bind_workspace_runtime_installations(
            Arc::clone(&workspace_database),
            vec![opencode.clone(), mismatched_pi],
            NOW,
        ),
        Err(RuntimeApplicationError::Coordinator(_))
    ));
    assert_eq!(service.snapshot(NOW).unwrap(), before);
    assert!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .is_none()
    );

    let pi = pi_runtime_installation(&temporary.path().join("runtime-pi-valid"));
    let published = service
        .bind_workspace_runtime_installations(workspace_database, vec![opencode, pi], NOW + 1)
        .unwrap();
    assert_eq!(published.coordinator_generation, 1);
    assert_eq!(published.value.len(), 2);
    let snapshot = service.snapshot(NOW + 1).unwrap();
    assert_eq!(snapshot.runtimes.state_generation, 1);
    assert_eq!(snapshot.runtimes.records.len(), 2);
    assert!(snapshot.runtimes.records.iter().all(|record| {
        record.installation.workspace_id == "workspace-1"
            && record.lifecycle == c4os_lib::runtime::supervisor::RuntimeLifecycle::Stopped
    }));
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        1
    );
}

#[test]
fn workspace_runtime_binding_cas_failure_keeps_the_workspace_unbound_and_quarantines_mutation() {
    let temporary = TempDir::new().unwrap();
    let descriptor =
        DatabaseDescriptor::workspace(temporary.path().join("workspace-active"), "workspace-1");
    let (workspace_database, _) = DatabaseActor::start(descriptor).unwrap();
    let workspace_database = Arc::new(workspace_database);
    seed_workspace(&workspace_database);
    let (database, service) = app_database_and_service(temporary.path());
    let before = service.snapshot(NOW).unwrap();

    advance_runtime_control_plane_revision(&database, 1, None, NOW + 1);

    assert!(matches!(
        service.bind_workspace_runtime_installations(
            Arc::clone(&workspace_database),
            vec![
                runtime_installation(&temporary.path().join("runtime-opencode")),
                pi_runtime_installation(&temporary.path().join("runtime-pi")),
            ],
            NOW + 2,
        ),
        Err(RuntimeApplicationError::Persistence(_))
    ));
    assert_eq!(service.bound_workspace_id().unwrap(), None);
    assert_eq!(service.snapshot(NOW + 2).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        1
    );

    assert!(matches!(
        service.bind_workspace_runtime_installations(
            workspace_database,
            vec![
                runtime_installation(&temporary.path().join("runtime-opencode-retry")),
                pi_runtime_installation(&temporary.path().join("runtime-pi-retry")),
            ],
            NOW + 3,
        ),
        Err(RuntimeApplicationError::Unavailable)
    ));
    assert_eq!(service.bound_workspace_id().unwrap(), None);
    assert_eq!(service.snapshot(NOW + 3).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        1
    );
}

#[test]
fn coordinator_recovery_becomes_durable_only_through_the_bound_workspace_database() {
    let temporary = TempDir::new().unwrap();
    let descriptor =
        DatabaseDescriptor::workspace(temporary.path().join("workspace-active"), "workspace-1");
    let (workspace_database, _) = DatabaseActor::start(descriptor).unwrap();
    let workspace_database = Arc::new(workspace_database);
    seed_workspace(&workspace_database);
    let repository = SqliteSessionRepository::new(Arc::clone(&workspace_database)).unwrap();
    let seed = SessionService::new(repository.clone());
    seed.create_provisional("session-1", NOW).unwrap();
    seed.submit_first(FirstSubmission {
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        authorization_scope_id: "authorization-scope-1".into(),
        correlation_id: "correlation-1".into(),
        process_generation: 1,
        prompt: Some("Recover this run".into()),
        attachments: vec![],
        binding: binding(),
        submitted_at_ms: NOW + 1,
    })
    .unwrap();

    let service = app_service(temporary.path());
    assert!(service.recover_interrupted(0, NOW + 2).is_err());
    service.bind_workspace(workspace_database).unwrap();
    let recovered = service.recover_interrupted(1, NOW + 3).unwrap();
    assert_eq!(
        recovered.coordinator_generation, 3,
        "recovery terminalization and run-authority invalidation are distinct core mutations"
    );
    assert_eq!(recovered.value.len(), 1);
    assert!(matches!(
        recovered.value[0].attempts[0].status,
        RunAttemptStatus::Interrupted { .. }
    ));

    let durable = repository.load("session-1").unwrap().unwrap();
    assert!(matches!(
        durable.attempts[0].status,
        RunAttemptStatus::Interrupted { .. }
    ));
}

#[test]
fn coordinator_generation_advances_for_provider_and_supervisor_domains() {
    let temporary = TempDir::new().unwrap();
    let service = app_service(temporary.path());
    let initial = service.snapshot(NOW).unwrap();
    assert_eq!(initial.generation, 0);
    assert_eq!(initial.providers.generation, 0);
    assert_eq!(initial.runtimes.state_generation, 0);

    let provider = service
        .save_provider(0, provider_profile(), 0, NOW + 1)
        .unwrap();
    assert_eq!(provider.coordinator_generation, 1);
    let after_provider = service.snapshot(NOW + 1).unwrap();
    assert_eq!(after_provider.generation, 1);
    assert_eq!(after_provider.providers.generation, 1);
    assert_eq!(after_provider.runtimes.state_generation, 0);

    let runtime = service
        .register_runtime(
            1,
            runtime_installation(&temporary.path().join("runtime")),
            NOW + 2,
        )
        .unwrap();
    assert_eq!(runtime.coordinator_generation, 2);
    let after_runtime = service.snapshot(NOW + 2).unwrap();
    assert_eq!(after_runtime.generation, 2);
    assert_eq!(after_runtime.providers.generation, 1);
    assert_eq!(after_runtime.runtimes.state_generation, 1);
}

#[test]
fn provider_cas_failure_does_not_publish_the_rejected_profile_mutation() {
    let temporary = TempDir::new().unwrap();
    let (database, service) = app_database_and_service(temporary.path());
    service
        .save_provider(0, provider_profile(), 0, NOW + 1)
        .unwrap();
    let before = service.snapshot(NOW + 1).unwrap();

    let mut durable = before.providers.clone();
    durable.generation = 2;
    advance_runtime_document_generation(
        &database,
        "provider-snapshot",
        "providers",
        durable.generation,
        &durable,
        Some(before.providers.generation),
        NOW + 2,
    );

    let mut rejected = provider_profile();
    rejected.display_name = "Must not publish".into();
    assert!(matches!(
        service.save_provider(
            before.generation,
            rejected,
            before.providers.generation,
            NOW + 3,
        ),
        Err(RuntimeApplicationError::Persistence(_))
    ));
    assert_eq!(service.snapshot(NOW + 3).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("provider-snapshot", "providers")
            .unwrap()
            .unwrap()
            .generation,
        2
    );
}

#[test]
fn provider_cas_race_quarantines_the_unpublished_in_memory_mutation() {
    let temporary = TempDir::new().unwrap();
    let (database, service) = app_database_and_service(temporary.path());
    service
        .save_provider(0, provider_profile(), 0, NOW + 1)
        .unwrap();
    let before = service.snapshot(NOW + 1).unwrap();
    let mut durable = before.providers.clone();
    durable.generation = 2;
    let mut model = provider_model_route(capability_route());
    model.checked_at_ms = NOW + 2;
    let declaration = model.provider_declaration.as_mut().unwrap();
    declaration.declared_at_ms = NOW + 2;
    declaration.expires_at_ms = NOW + 60_002;
    let mut probe = CasRacingProbe {
        discovery: ProviderDiscovery {
            checked_at_ms: NOW + 2,
            models: vec![model],
            recommended_model_id: Some("claude-sonnet".into()),
            connection_evidence: None,
        },
        database: Arc::clone(&database),
        durable,
    };

    let raced = service.test_provider(
        before.generation,
        "provider-openrouter",
        before.providers.generation,
        NOW + 2,
        &mut probe,
    );
    assert!(
        matches!(raced, Err(RuntimeApplicationError::Persistence(_))),
        "provider CAS race returned {raced:?}"
    );
    assert_eq!(service.snapshot(NOW + 2).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("provider-snapshot", "providers")
            .unwrap()
            .unwrap()
            .generation,
        2
    );
    assert!(matches!(
        service.save_provider(
            before.generation,
            provider_profile(),
            before.providers.generation,
            NOW + 3,
        ),
        Err(RuntimeApplicationError::Unavailable)
    ));
}

#[test]
fn runtime_registration_cas_failure_does_not_publish_the_registration() {
    let temporary = TempDir::new().unwrap();
    let (database, service) = app_database_and_service(temporary.path());
    let before = service.snapshot(NOW).unwrap();

    advance_runtime_control_plane_revision(&database, 1, None, NOW + 1);

    assert!(matches!(
        service.register_runtime(
            before.generation,
            runtime_installation(&temporary.path().join("runtime")),
            NOW + 2,
        ),
        Err(RuntimeApplicationError::Persistence(_))
    ));
    assert_eq!(service.snapshot(NOW + 2).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        1
    );
}

#[test]
fn runtime_start_cas_failure_never_spawns_or_publishes_a_process() {
    let temporary = TempDir::new().unwrap();
    let (database, service) = app_database_and_service(temporary.path());
    let runtime_root = temporary.path().join("runtime");
    let started_marker = temporary.path().join("worker-started");
    service
        .register_runtime(
            0,
            runnable_runtime_installation(&runtime_root, &started_marker),
            NOW + 1,
        )
        .unwrap();
    let before = service.snapshot(NOW + 1).unwrap();

    advance_runtime_control_plane_revision(&database, 2, Some(1), NOW + 2);

    assert!(matches!(
        service.start_runtime(before.generation, "opencode-primary", NOW + 3),
        Err(RuntimeApplicationError::Persistence(_))
    ));
    for _ in 0..20 {
        if started_marker.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !started_marker.exists(),
        "a rejected start must not launch the runtime worker"
    );
    assert_eq!(service.snapshot(NOW + 3).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        2
    );
}

#[test]
fn runtime_health_cas_failure_does_not_publish_the_health_transition() {
    let temporary = TempDir::new().unwrap();
    let (database, service) = app_database_and_service(temporary.path());
    let runtime_root = temporary.path().join("runtime");
    let started_marker = temporary.path().join("health-worker-started");
    service
        .register_runtime(
            0,
            runnable_runtime_installation(&runtime_root, &started_marker),
            NOW + 1,
        )
        .unwrap();
    let started = service
        .start_runtime(1, "opencode-primary", NOW + 2)
        .unwrap();
    let before = service.snapshot(NOW + 2).unwrap();

    advance_runtime_control_plane_revision(&database, 3, Some(2), NOW + 3);

    assert!(matches!(
        service.record_runtime_health(
            before.generation,
            "opencode-primary",
            started.value,
            HealthState::Healthy,
            NOW + 4,
        ),
        Err(RuntimeApplicationError::Persistence(_))
    ));
    assert_eq!(service.snapshot(NOW + 4).unwrap(), before);
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        3
    );
}

#[test]
fn production_managed_runtime_lifecycle_is_durable_across_reserve_attach_and_shutdown() {
    let temporary = TempDir::new().unwrap();
    let service = app_service(temporary.path());
    let runtime_root = temporary.path().join("runtime-production");
    let started_marker = temporary.path().join("must-not-be-started-by-reservation");
    service
        .register_runtime(
            0,
            runnable_runtime_installation(&runtime_root, &started_marker),
            NOW + 1,
        )
        .unwrap();

    let reserved = service
        .reserve_managed_runtime_start(1, "opencode-primary", NOW + 2)
        .unwrap();
    let after_reserve = service.snapshot(NOW + 2).unwrap();
    let reserved_record = after_reserve
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == "opencode-primary")
        .unwrap();
    assert_eq!(reserved.value, 1);
    assert_eq!(reserved_record.process_generation, reserved.value);
    assert_eq!(reserved_record.process_id, None);
    assert_eq!(
        reserved_record.lifecycle,
        c4os_lib::runtime::supervisor::RuntimeLifecycle::Starting
    );
    assert!(!started_marker.exists());

    let attached = service
        .attach_managed_runtime_process(
            after_reserve.generation,
            "opencode-primary",
            reserved.value,
            42_424,
            HealthState::Healthy,
            NOW + 3,
        )
        .unwrap();
    let after_attach = service.snapshot(NOW + 3).unwrap();
    let attached_record = after_attach
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == "opencode-primary")
        .unwrap();
    assert_eq!(attached.coordinator_generation, after_attach.generation);
    assert_eq!(attached_record.process_id, Some(42_424));
    assert_eq!(
        attached_record.lifecycle,
        c4os_lib::runtime::supervisor::RuntimeLifecycle::Ready
    );

    let stopped = service
        .finish_managed_runtime_shutdown(
            after_attach.generation,
            "opencode-primary",
            reserved.value,
            NOW + 4,
        )
        .unwrap();
    let after_shutdown = service.snapshot(NOW + 4).unwrap();
    let stopped_record = after_shutdown
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == "opencode-primary")
        .unwrap();
    assert_eq!(stopped.coordinator_generation, after_shutdown.generation);
    assert_eq!(stopped_record.process_id, None);
    assert_eq!(
        stopped_record.lifecycle,
        c4os_lib::runtime::supervisor::RuntimeLifecycle::Stopped
    );
}

#[test]
fn application_service_preflight_uses_only_registry_owned_typed_evidence() {
    let temporary = TempDir::new().unwrap();
    let service = app_service(temporary.path());
    let route = capability_route();
    let model_route = provider_model_route(route.clone());

    service
        .save_provider(0, provider_profile(), 0, NOW)
        .unwrap();
    service
        .test_provider(
            1,
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(ProviderDiscovery {
                checked_at_ms: NOW,
                models: vec![model_route.clone()],
                recommended_model_id: Some("claude-sonnet".into()),
                connection_evidence: None,
            }),
        )
        .unwrap();

    let runtime_root = temporary.path().join("runtime-preflight");
    let started_marker = temporary.path().join("preflight-runtime-must-not-spawn");
    service
        .register_runtime(
            2,
            runnable_runtime_installation(&runtime_root, &started_marker),
            NOW + 1,
        )
        .unwrap();
    let reserved = service
        .reserve_managed_runtime_start(3, "opencode-primary", NOW + 2)
        .unwrap();
    let native_models = Arc::new(Mutex::new(Vec::new()));
    let conformance = OpenCodeCompatibilityManifest::pinned(digest('f'))
        .unwrap()
        .conformance_descriptor(reserved.value)
        .unwrap();
    service
        .register_dispatch_peer(CapturingPeer {
            registration: RuntimePeerRegistration {
                runtime_id: "opencode-primary".into(),
                workspace_id: "workspace-1".into(),
                descriptor: conformance.clone(),
            },
            native_models,
        })
        .unwrap();

    let declared = provider_declared_evidence(&provider_capability_claim(route.clone())).unwrap();
    let adapter = opencode_adapter_evidence(&model_route).unwrap();
    let health = HealthSnapshot {
        healthy: true,
        native_version: OPENCODE_NATIVE_VERSION.into(),
        process_generation: reserved.value,
        checked_at_ms: NOW + 1,
    };
    let mut substituted_route = route.clone();
    substituted_route.session_configuration_sha256 = digest('8');
    let substituted = opencode_observed_evidence(
        &conformance,
        &health,
        &runtime_observation(substituted_route, reserved.value),
    )
    .unwrap();
    assert!(matches!(
        CapabilityRouteEpoch::new(
            "opencode-primary",
            reserved.value,
            declared.clone(),
            adapter.clone(),
            substituted,
        ),
        Err(CapabilityEvidenceError::ArtifactMismatch)
    ));
    assert_eq!(service.capability_evidence_generation().unwrap(), 0);
    assert!(matches!(
        service.model_preflight(
            "provider-openrouter",
            "claude-sonnet",
            &capability_draft(),
            NOW + 3,
        ),
        Err(RuntimeApplicationError::CapabilityEvidence(
            CapabilityEvidenceError::RouteNotFound
        ))
    ));

    let observed = opencode_observed_evidence(
        &conformance,
        &health,
        &runtime_observation(route, reserved.value),
    )
    .unwrap();
    let coordinator_generation = service.snapshot(NOW + 3).unwrap().generation;
    let (attached, capability_generation, authority_generation) = service
        .attach_managed_runtime_process_with_capabilities(
            coordinator_generation,
            0,
            "opencode-primary",
            reserved.value,
            42_424,
            HealthState::Healthy,
            vec![
                CapabilityRouteEpoch::new(
                    "opencode-primary",
                    reserved.value,
                    declared,
                    adapter,
                    observed,
                )
                .unwrap(),
            ],
            NOW + 3,
        )
        .unwrap();
    assert_eq!(attached.coordinator_generation, coordinator_generation + 1);
    assert_eq!(capability_generation, 1);
    assert_eq!(authority_generation, 1);
    assert_eq!(service.capability_evidence_generation().unwrap(), 1);
    assert_eq!(
        service.snapshot(NOW + 3).unwrap().generation,
        coordinator_generation + 1,
        "the complete capability epoch must invalidate application authority exactly once"
    );

    let resolved = service
        .model_preflight(
            "provider-openrouter",
            "claude-sonnet",
            &capability_draft(),
            NOW + 3,
        )
        .unwrap();
    assert!(matches!(resolved.outcome, PreflightOutcome::Ready { .. }));
    assert_eq!(
        resolved
            .effective_capabilities
            .feature_state(CapabilityKey::InputImage),
        CapabilityState::Unsupported,
        "adapter-normalized denial must restrict declared and observed support"
    );
    assert_eq!(
        resolved
            .effective_capabilities
            .numeric_maximum(NumericCapabilityKey::InputTokens),
        Some(100_000),
        "effective limits must use the most restrictive producer evidence"
    );
    assert_eq!(
        resolved
            .effective_capabilities
            .numeric_maximum(NumericCapabilityKey::OutputTokens),
        Some(2_000)
    );
}

#[test]
fn application_dispatch_derives_active_project_scope_from_workspace_process_authority() {
    let temporary = TempDir::new().unwrap();
    let descriptor =
        DatabaseDescriptor::workspace(temporary.path().join("workspace-active"), "workspace-1");
    let (workspace_database, _) = DatabaseActor::start(descriptor).unwrap();
    let workspace_database = Arc::new(workspace_database);
    seed_workspace(&workspace_database);
    assert!(
        workspace_database
            .active_project_exists("project-1")
            .unwrap()
    );
    assert!(
        !workspace_database
            .active_project_exists("project-missing")
            .unwrap()
    );

    let service = app_service(temporary.path());
    service.bind_workspace(workspace_database).unwrap();

    let configuration = AuthoritativeConfiguration {
        version: 1,
        fields: BTreeMap::from([
            (
                "provider-id".into(),
                ConfigurationFieldValue::Text("provider-openrouter".into()),
            ),
            (
                "endpoint-id".into(),
                ConfigurationFieldValue::Text("openrouter-chat".into()),
            ),
            (
                "model-id".into(),
                ConfigurationFieldValue::Text("anthropic/claude-sonnet".into()),
            ),
            (
                "model-revision".into(),
                ConfigurationFieldValue::Text("2026-07-19".into()),
            ),
            (
                "adapter-kind".into(),
                ConfigurationFieldValue::Text("opencode".into()),
            ),
            (
                "adapter-version".into(),
                ConfigurationFieldValue::Text("1.0.0".into()),
            ),
            (
                "runtime-kind".into(),
                ConfigurationFieldValue::Text("opencode".into()),
            ),
            (
                "native-runtime-version".into(),
                ConfigurationFieldValue::Text(OPENCODE_NATIVE_VERSION.into()),
            ),
        ]),
    };
    let configuration_sha256 = authoritative_configuration_sha256(&configuration).unwrap();
    let route = RouteIdentity {
        provider_id: "provider-openrouter".into(),
        endpoint_id: "openrouter-chat".into(),
        provider_model_id: "anthropic/claude-sonnet".into(),
        model_revision: "2026-07-19".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: OPENCODE_NATIVE_VERSION.into(),
        session_configuration_sha256: configuration_sha256,
    };
    let model_route = provider_model_route(route.clone());

    service
        .save_provider(1, provider_profile(), 0, NOW)
        .unwrap();
    service
        .test_provider(
            2,
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(ProviderDiscovery {
                checked_at_ms: NOW,
                models: vec![model_route.clone()],
                recommended_model_id: Some("claude-sonnet".into()),
                connection_evidence: None,
            }),
        )
        .unwrap();
    let runtime_root = temporary.path().join("runtime");
    fs::create_dir_all(runtime_root.join("bin")).unwrap();
    let fixture_executable = runtime_root.join("bin/opencode");
    fs::write(&fixture_executable, b"#!/bin/sh\nsleep 30\n").unwrap();
    fs::set_permissions(&fixture_executable, fs::Permissions::from_mode(0o700)).unwrap();
    let mut installation = runtime_installation(&runtime_root);
    installation.executable_sha256 = sha256_file(&fixture_executable).unwrap();
    service.register_runtime(3, installation, NOW + 3).unwrap();
    let reserved = service
        .reserve_managed_runtime_start(4, "opencode-primary", NOW + 4)
        .unwrap();

    let native_models = Arc::new(Mutex::new(Vec::new()));
    let observed_manifest = OpenCodeCompatibilityManifest::pinned(digest('f'))
        .unwrap()
        .conformance_descriptor(reserved.value)
        .unwrap();
    service
        .register_dispatch_peer(CapturingPeer {
            registration: RuntimePeerRegistration {
                runtime_id: "opencode-primary".into(),
                workspace_id: "workspace-1".into(),
                descriptor: observed_manifest.clone(),
            },
            native_models: Arc::clone(&native_models),
        })
        .unwrap();
    let declared = provider_declared_evidence(&provider_capability_claim(route.clone())).unwrap();
    let adapter = opencode_adapter_evidence(&model_route).unwrap();
    let observed = opencode_observed_evidence(
        &observed_manifest,
        &HealthSnapshot {
            healthy: true,
            native_version: OPENCODE_NATIVE_VERSION.into(),
            process_generation: reserved.value,
            checked_at_ms: NOW + 1,
        },
        &runtime_observation(route.clone(), reserved.value),
    )
    .unwrap();
    service
        .attach_managed_runtime_process_with_capabilities(
            5,
            0,
            "opencode-primary",
            reserved.value,
            42_424,
            HealthState::Healthy,
            vec![
                CapabilityRouteEpoch::new(
                    "opencode-primary",
                    reserved.value,
                    declared,
                    adapter,
                    observed,
                )
                .unwrap(),
            ],
            NOW + 5,
        )
        .unwrap();

    let resources = production_broker_authoritative_resources();
    let resources_sha256 = authoritative_resources_sha256(&resources).unwrap();
    let resource_ids = resources.records.keys().cloned().collect();
    service.create_provisional(6, "session-1", NOW + 7).unwrap();

    let dispatch = service
        .dispatch_first(
            7,
            FirstRuntimeDispatchIntent {
                authority: AuthorityMintIntent {
                    workspace_id: "workspace-1".into(),
                    project_id: Some("project-1".into()),
                    runtime_id: "opencode-primary".into(),
                    expected_authority_generation: 1,
                    expected_capability_generation: 1,
                },
                provider_id: "provider-openrouter".into(),
                model_id: "claude-sonnet".into(),
                session_id: "session-1".into(),
                turn_id: "turn-1".into(),
                attempt_id: "attempt-1".into(),
                authorization_scope_id: "authority-1".into(),
                correlation_id: "correlation-1".into(),
                prompt: Some("Use the native provider route".into()),
                attachments: Vec::new(),
                draft: DraftRequirements {
                    attachments: Vec::new(),
                    reasoning_mode: None,
                    requires_tools: false,
                    requires_json_schema: false,
                    prefers_streaming: true,
                    estimated_input_tokens: 100,
                    requested_output_tokens: 100,
                    installed_resources: InstalledResourcePreflight {
                        snapshot_id: format!(
                            "resources:{}",
                            resources_sha256
                                .strip_prefix("sha256:")
                                .expect("resource digest prefix")
                        ),
                        snapshot_sha256: resources_sha256,
                        tool_ids: resource_ids,
                        attachment_converters: BTreeSet::new(),
                    },
                    policy: PolicyPreflight {
                        snapshot_id: "policy-1".into(),
                        version: 1,
                        tool_use_allowed: false,
                        attachment_conversion_allowed: false,
                    },
                },
                submitted_at_ms: NOW + 8,
                preflight_at_ms: NOW + 8,
            },
            FirstDispatchOptions {
                title: "Native route".into(),
                credential_reference: None,
                credential_lease_id: None,
                attachment_resolution: AttachmentPreflightResolution::NotRequired,
                broker_authority: None,
            },
        )
        .unwrap();

    let CoordinatedFirstDispatch::Accepted { record, .. } = dispatch else {
        panic!("minted dispatch must be accepted")
    };
    assert_eq!(
        record
            .binding()
            .expect("durable binding")
            .initial_model_route
            .model_id,
        "anthropic/claude-sonnet"
    );
    assert_eq!(
        *native_models.lock().expect("native model capture"),
        vec!["anthropic/claude-sonnet"]
    );
}
