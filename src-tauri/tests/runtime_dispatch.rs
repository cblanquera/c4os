//! Deterministic coverage for the Rust-owned runtime dispatch boundary.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::runtime::action_bridge::RuntimeIntentIdentity;
use c4os_lib::runtime::adapter::{
    ADAPTER_CONTRACT_SCHEMA_VERSION, AdapterAuthority, AdapterConformanceDescriptor,
    PeerCapabilityClaims, peer_capabilities,
};
use c4os_lib::runtime::attachment_materializer::WorkspaceAttachmentMaterializer;
use c4os_lib::runtime::capability::{
    AttachmentMediaType, AttachmentRequirement, CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor,
    CapabilityEvidence, CapabilityKey, CapabilityLayer, CapabilityState, DraftRequirements,
    InstalledResourcePreflight, LimitConfidence, ModelLifecycle, NumericCapabilityEvidence,
    NumericCapabilityKey, PolicyPreflight, RouteIdentity,
};
use c4os_lib::runtime::coordinator::{
    CoordinatedFirstSubmission, CoordinatedRetry, CoordinatedTurn, RuntimeCoordinator,
};
use c4os_lib::runtime::dispatch::{
    AttachmentMaterializationPlan, AttachmentPreflightResolution, CoordinatedCancellation,
    CoordinatedFirstDispatch, CoordinatedRetryDispatch, CoordinatedTurnDispatch, DispatchError,
    DispatchEventCategory, DispatchIdentity, DispatchModelRoute, FirstDispatchOptions,
    InstalledAttachmentConverterDescriptor, PeerDispatchError, PeerDispatchEvent,
    PeerDispatchRequest, PiDispatchCredentialIssuer, PiDispatchPeer,
    ProductionOpenCodeDispatchPeer, ProductionPiDispatchPeer, RetryDispatchOptions,
    RuntimeDispatchPeer, RuntimeDispatchRegistry, RuntimePeerRegistration, TurnDispatchOptions,
    coordinate_cancellation, coordinate_first_dispatch, coordinate_polled_events,
    coordinate_recovery, coordinate_retry_dispatch, coordinate_turn_dispatch,
};
use c4os_lib::runtime::pi::{PI_NATIVE_VERSION, PiAdapter, PiSidecarManifest, PiSidecarRunner};
use c4os_lib::runtime::provider::{
    ModelRoute, PROVIDER_SCHEMA_VERSION, ProviderConnectionEvidence, ProviderDiscovery,
    ProviderEndpoint, ProviderKind, ProviderProbe, ProviderProbeFailure, ProviderProfile,
    ProviderService, RouteAvailability,
};
use c4os_lib::runtime::session::{
    AdapterBinding, AttachmentSnapshot, AttemptContextSnapshot, CapabilitySnapshot,
    ConfigurationSnapshot, ExecutionEnvironmentBinding, FirstSubmission,
    MessageReplyContextSnapshot, ModelRouteSnapshot, ResourceSnapshot, RetryRequest,
    RunAttemptStatus, SessionLifecycle, SessionRecord, SessionRepository, SessionRepositoryError,
    SessionService, SideEffectState, TurnSubmission, capability_snapshot_from_effective_descriptor,
};
use c4os_lib::runtime::supervisor::{
    HealthState, OPENCODE_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation,
    RuntimeKind, RuntimeSupervisor, sha256_file,
};
use c4os_lib::security::credentials::CredentialVault;
use c4os_lib::security::gateway::ActionGateway;
use c4os_lib::security::policy::PolicyConfiguration;
use serde_json::{Value, json};
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;

#[derive(Clone, Default)]
struct MemoryRepository {
    records: Arc<Mutex<BTreeMap<String, SessionRecord>>>,
    trace: Arc<Mutex<Vec<String>>>,
}

impl SessionRepository for MemoryRepository {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError> {
        Ok(self
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .get(session_id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError> {
        Ok(self
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .values()
            .cloned()
            .collect())
    }

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        if records.contains_key(&record.session_id) {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(record.session_id.clone(), record.clone());
        self.trace.lock().unwrap().push("durable".into());
        Ok(())
    }

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let current = records
            .get(session_id)
            .ok_or(SessionRepositoryError::Conflict)?;
        if current.revision != expected_revision {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(session_id.into(), replacement.clone());
        Ok(())
    }
}

struct FixtureProbe(ProviderDiscovery);

impl ProviderProbe for FixtureProbe {
    fn test_and_discover(
        &mut self,
        profile: &ProviderProfile,
    ) -> Result<ProviderDiscovery, ProviderProbeFailure> {
        let mut discovery = self.0.clone();
        discovery.connection_evidence = Some(ProviderConnectionEvidence::from_tested_profile(
            profile,
            discovery.checked_at_ms,
            digest('6'),
        )?);
        Ok(discovery)
    }
}

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn profile() -> ProviderProfile {
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

fn route_identity() -> RouteIdentity {
    RouteIdentity {
        provider_id: "provider-openrouter".into(),
        endpoint_id: "openrouter-chat".into(),
        provider_model_id: "anthropic/claude-sonnet".into(),
        model_revision: "2026-07-19".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: OPENCODE_NATIVE_VERSION.into(),
        session_configuration_sha256: digest('a'),
    }
}

fn evidence(layer: CapabilityLayer, key: CapabilityKey) -> CapabilityEvidence {
    CapabilityEvidence {
        state: CapabilityState::Supported,
        layer,
        source: format!("fixture.{layer:?}.{key:?}").to_ascii_lowercase(),
        checked_at_ms: NOW,
        expires_at_ms: Some(NOW + 60_000),
        constraints: vec![],
        allowed_values: vec![],
        reason: None,
    }
}

fn descriptor(layer: CapabilityLayer) -> CapabilityDescriptor {
    let features = [
        CapabilityKey::InputText,
        CapabilityKey::OutputText,
        CapabilityKey::Streaming,
        CapabilityKey::ToolCalling,
    ]
    .into_iter()
    .map(|key| (key, evidence(layer, key)))
    .collect();
    let numeric_limits = [
        (NumericCapabilityKey::InputTokens, 8_192),
        (NumericCapabilityKey::OutputTokens, 4_096),
    ]
    .into_iter()
    .map(|(key, maximum)| {
        (
            key,
            NumericCapabilityEvidence {
                evidence: CapabilityEvidence {
                    state: CapabilityState::Supported,
                    layer,
                    source: format!("fixture.{layer:?}.{key:?}").to_ascii_lowercase(),
                    checked_at_ms: NOW,
                    expires_at_ms: Some(NOW + 60_000),
                    constraints: vec![],
                    allowed_values: vec![],
                    reason: None,
                },
                maximum: Some(maximum),
                confidence: LimitConfidence::Confirmed,
            },
        )
    })
    .collect();
    CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer,
        route: route_identity(),
        lifecycle: ModelLifecycle::Active,
        features,
        numeric_limits,
        raw_evidence_sha256: digest(match layer {
            CapabilityLayer::Declared => 'b',
            CapabilityLayer::AdapterNormalized => 'c',
            CapabilityLayer::Observed => 'd',
            CapabilityLayer::Effective => 'e',
        }),
    }
}

fn layers() -> [CapabilityDescriptor; 3] {
    [
        descriptor(CapabilityLayer::Declared),
        descriptor(CapabilityLayer::AdapterNormalized),
        descriptor(CapabilityLayer::Observed),
    ]
}

fn provider_route() -> ModelRoute {
    ModelRoute {
        model_id: "claude-sonnet".into(),
        display_name: "Claude Sonnet".into(),
        recommendation_rank: 1,
        availability: RouteAvailability::Available,
        checked_at_ms: NOW,
        capabilities: descriptor(CapabilityLayer::Declared),
        provider_declaration: Some(c4os_lib::runtime::provider::ProviderModelDeclaration {
            schema_version: c4os_lib::runtime::provider::PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
            provider_model_id: "claude-sonnet".into(),
            model_revision: "2026-07-19".into(),
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_catalog_sha256: digest('9'),
            declared_at_ms: NOW,
            expires_at_ms: NOW + 60_000,
        }),
    }
}

fn draft() -> DraftRequirements {
    DraftRequirements {
        attachments: vec![],
        reasoning_mode: None,
        requires_tools: false,
        requires_json_schema: false,
        prefers_streaming: true,
        estimated_input_tokens: 100,
        requested_output_tokens: 100,
        installed_resources: InstalledResourcePreflight {
            snapshot_id: "resources-1".into(),
            snapshot_sha256: digest('8'),
            tool_ids: BTreeSet::new(),
            attachment_converters: BTreeSet::new(),
        },
        policy: PolicyPreflight {
            snapshot_id: "policy-1".into(),
            version: 1,
            tool_use_allowed: true,
            attachment_conversion_allowed: false,
        },
    }
}

fn fixture_executable(root: &Path) -> std::path::PathBuf {
    let executable = root.join("bin/opencode-dispatch-fixture");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"#!/bin/sh\nexec /bin/sleep 30\n").unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&executable, permissions).unwrap();
    executable
}

fn installation(root: &Path) -> RuntimeInstallation {
    let executable = fixture_executable(root);
    RuntimeInstallation {
        runtime_id: "opencode-primary".into(),
        workspace_id: "workspace-1".into(),
        runtime_kind: RuntimeKind::OpenCode,
        native_version: OPENCODE_NATIVE_VERSION.into(),
        adapter_version: "1.0.0".into(),
        protocol_version: RUNTIME_PROTOCOL_VERSION,
        install_root: root.to_owned(),
        asset_tree_sha256: sha256_file(&executable).unwrap(),
        executable_sha256: sha256_file(&executable).unwrap(),
        executable,
        state_namespace: root.join("state"),
        arguments: vec![],
        sanitized_environment: BTreeMap::new(),
    }
}

fn model_snapshot() -> ModelRouteSnapshot {
    ModelRouteSnapshot {
        route_id: "route-claude-sonnet".into(),
        provider_id: "provider-openrouter".into(),
        endpoint_id: "openrouter-chat".into(),
        model_id: "anthropic/claude-sonnet".into(),
        model_revision: "2026-07-19".into(),
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
                provider_model_id: "anthropic/claude-sonnet".into(),
                model_revision: "2026-07-19".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: OPENCODE_NATIVE_VERSION.into(),
                session_configuration_sha256: digest('7'),
            },
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_evidence_sha256: digest('9'),
        },
        1,
    )
    .expect("exact binding capabilities")
}

fn binding() -> c4os_lib::runtime::session::SessionBinding {
    c4os_lib::runtime::session::SessionBinding {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "opencode-primary".into(),
        runtime_kind: c4os_lib::runtime::session::RuntimeKind::OpenCode,
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
        initial_model_route: model_snapshot(),
        initial_configuration: ConfigurationSnapshot {
            snapshot_id: "configuration-1".into(),
            version: 1,
            sha256: digest('7'),
        },
        initial_resources: ResourceSnapshot {
            snapshot_id: "resources-1".into(),
            version: 1,
            sha256: digest('8'),
            resource_ids: vec!["skill-project-review".into()],
        },
        initial_capabilities: binding_capabilities(),
        bound_at_ms: NOW + 10,
    }
}

fn first_submission(process_generation: u64) -> CoordinatedFirstSubmission {
    CoordinatedFirstSubmission {
        submission: FirstSubmission {
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            attempt_id: "attempt-1".into(),
            authorization_scope_id: "authority-1".into(),
            correlation_id: "correlation-1".into(),
            process_generation,
            prompt: Some("Implement the coordinator".into()),
            attachments: vec![],
            binding: binding(),
            submitted_at_ms: NOW + 10,
        },
        provider_id: "provider-openrouter".into(),
        selected_model_id: "claude-sonnet".into(),
        capability_layers: layers(),
        draft: draft(),
        preflight_at_ms: NOW + 5,
    }
}

fn attachment() -> AttachmentSnapshot {
    AttachmentSnapshot {
        attachment_id: "attachment-1".into(),
        stable_reference: "workspace-blob:attachment-1".into(),
        display_name: "coordinator-notes.md".into(),
        media_type: "text/markdown".into(),
        byte_length: 4_096,
        content_sha256: digest('b'),
        snapshot_version: 3,
        original_reference: 1,
    }
}

const DIRECT_ATTACHMENT_CONTENT: &[u8] = b"Known direct attachment content.";

fn direct_attachment() -> AttachmentSnapshot {
    let content_sha256 = {
        use sha2::{Digest, Sha256};
        use std::fmt::Write as _;

        let mut encoded = String::from("sha256:");
        for byte in Sha256::digest(DIRECT_ATTACHMENT_CONTENT) {
            let _ = write!(encoded, "{byte:02x}");
        }
        encoded
    };
    AttachmentSnapshot {
        attachment_id: "attachment-direct-1".into(),
        stable_reference: format!("workspace-blob:{content_sha256}:v3"),
        display_name: "coordinator-notes.md".into(),
        media_type: "text/markdown".into(),
        byte_length: DIRECT_ATTACHMENT_CONTENT.len() as u64,
        content_sha256,
        snapshot_version: 3,
        original_reference: 1,
    }
}

fn attachment_draft(
    resources: &ResourceSnapshot,
    attachment: &AttachmentSnapshot,
) -> DraftRequirements {
    let mut requirements = draft();
    requirements.attachments = vec![AttachmentRequirement {
        attachment_id: attachment.attachment_id.clone(),
        media_type: AttachmentMediaType::OtherFile,
        mime_type: attachment.media_type.clone(),
        bytes: attachment.byte_length,
    }];
    requirements.installed_resources.snapshot_id = resources.snapshot_id.clone();
    requirements.installed_resources.snapshot_sha256 = resources.sha256.clone();
    requirements
        .installed_resources
        .attachment_converters
        .insert(AttachmentMediaType::OtherFile);
    requirements.policy.attachment_conversion_allowed = true;
    requirements
}

fn direct_attachment_submission(
    process_generation: u64,
    attachment_only: bool,
) -> CoordinatedFirstSubmission {
    let attachment = direct_attachment();
    let mut request = first_submission(process_generation);
    request.submission.prompt = (!attachment_only).then(|| "Review the attached notes".into());
    request.submission.attachments = vec![attachment.clone()];
    request.draft = draft();
    request.draft.attachments = vec![AttachmentRequirement {
        attachment_id: attachment.attachment_id.clone(),
        media_type: AttachmentMediaType::OtherFile,
        mime_type: attachment.media_type.clone(),
        bytes: attachment.byte_length,
    }];
    request
}

fn direct_attachment_resolution(
    root: &Path,
    request: &CoordinatedFirstSubmission,
) -> AttachmentPreflightResolution {
    let workspace = root.join("workspace-active");
    let blobs = workspace.join("blobs/sha256");
    fs::create_dir_all(&blobs).unwrap();
    let digest = request.submission.attachments[0]
        .content_sha256
        .strip_prefix("sha256:")
        .unwrap();
    fs::write(blobs.join(digest), DIRECT_ATTACHMENT_CONTENT).unwrap();
    let materializer = WorkspaceAttachmentMaterializer::bind("workspace-1", workspace).unwrap();
    AttachmentPreflightResolution::Direct(
        materializer
            .materialize(&request.submission.attachments)
            .unwrap(),
    )
}

fn direct_options(root: &Path, request: &CoordinatedFirstSubmission) -> FirstDispatchOptions {
    FirstDispatchOptions {
        attachment_resolution: direct_attachment_resolution(root, request),
        ..options()
    }
}

fn attachment_submission(
    process_generation: u64,
) -> (CoordinatedFirstSubmission, AttachmentMaterializationPlan) {
    let attachment = attachment();
    let mut request = first_submission(process_generation);
    request.submission.attachments = vec![attachment.clone()];
    request
        .submission
        .binding
        .initial_resources
        .resource_ids
        .push("converter-markdown-text".into());
    let descriptor = InstalledAttachmentConverterDescriptor {
        converter_id: "converter-markdown-text".into(),
        converter_version: "2.4.1".into(),
        snapshot_id: "converter-markdown-text-snapshot".into(),
        snapshot_version: 7,
        snapshot_sha256: digest('c'),
    };
    request.submission.binding.initial_resources.sha256 = descriptor
        .expected_resource_snapshot_sha256(&request.submission.binding.initial_resources)
        .unwrap();
    request.draft = attachment_draft(&request.submission.binding.initial_resources, &attachment);
    let authority = descriptor
        .verify(&request.submission.binding.initial_resources)
        .unwrap();
    let materialization = authority
        .materialize(
            &attachment,
            &request.submission.binding.initial_resources,
            "converted:attachment-1:v3",
            "Verified converted coordinator notes.",
        )
        .unwrap();
    (
        request,
        AttachmentMaterializationPlan::from_verified(vec![materialization]),
    )
}

fn attachment_options(plan: AttachmentMaterializationPlan) -> FirstDispatchOptions {
    FirstDispatchOptions {
        attachment_resolution: AttachmentPreflightResolution::Materialize(plan),
        ..options()
    }
}

fn attachment_retry_options(plan: AttachmentMaterializationPlan) -> RetryDispatchOptions {
    RetryDispatchOptions {
        attachment_resolution: AttachmentPreflightResolution::Materialize(plan),
        ..retry_options()
    }
}

fn dispatch_identity(process_generation: u64) -> DispatchIdentity {
    DispatchIdentity {
        workspace_id: "workspace-1".into(),
        environment_id: "local".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        correlation_id: "correlation-1".into(),
        runtime_id: "opencode-primary".into(),
        runtime_kind: RuntimeKind::OpenCode,
        adapter_version: "1.0.0".into(),
        native_version: OPENCODE_NATIVE_VERSION.into(),
        process_generation,
    }
}

fn peer_registration(process_generation: u64) -> RuntimePeerRegistration {
    RuntimePeerRegistration {
        runtime_id: "opencode-primary".into(),
        workspace_id: "workspace-1".into(),
        descriptor: AdapterConformanceDescriptor {
            schema_version: ADAPTER_CONTRACT_SCHEMA_VERSION,
            runtime_kind: RuntimeKind::OpenCode,
            adapter_version: "1.0.0".into(),
            native_version: OPENCODE_NATIVE_VERSION.into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
            process_generation,
            authority: AdapterAuthority::C4osActionGatewayOnly,
            capabilities: peer_capabilities(PeerCapabilityClaims {
                health: CapabilityState::Supported,
                session_create: CapabilityState::Supported,
                session_resume: CapabilityState::Degraded,
                model_discovery: CapabilityState::Supported,
                streaming: CapabilityState::Supported,
                action_intents: CapabilityState::Degraded,
                credential_channel: CapabilityState::Supported,
                cancellation: CapabilityState::Supported,
                restart: CapabilityState::Supported,
            }),
        },
    }
}

#[derive(Default)]
struct PeerControl {
    ready: bool,
    fail_create: bool,
    fail_dispatch: bool,
    cancel_accepted: bool,
    events: VecDeque<PeerDispatchEvent>,
    requests: Vec<PeerDispatchRequest>,
}

struct DeterministicPeer {
    registration: RuntimePeerRegistration,
    control: Arc<Mutex<PeerControl>>,
    trace: Arc<Mutex<Vec<String>>>,
}

impl RuntimeDispatchPeer for DeterministicPeer {
    fn registration(&self) -> &RuntimePeerRegistration {
        &self.registration
    }

    fn readiness(&self) -> Result<(), PeerDispatchError> {
        self.trace.lock().unwrap().push("ready".into());
        if self.control.lock().unwrap().ready {
            Ok(())
        } else {
            Err(PeerDispatchError::NotReady)
        }
    }

    fn create_session(&mut self, _request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        self.trace.lock().unwrap().push("create".into());
        if self.control.lock().unwrap().fail_create {
            Err(PeerDispatchError::SessionCreate)
        } else {
            Ok(())
        }
    }

    fn dispatch(&mut self, request: &PeerDispatchRequest) -> Result<(), PeerDispatchError> {
        self.trace.lock().unwrap().push("dispatch".into());
        let mut control = self.control.lock().unwrap();
        control.requests.push(request.clone());
        if control.fail_dispatch {
            Err(PeerDispatchError::Dispatch)
        } else {
            Ok(())
        }
    }

    fn poll_events(
        &mut self,
        _recorded_at_ms: u64,
    ) -> Result<Vec<PeerDispatchEvent>, PeerDispatchError> {
        self.trace.lock().unwrap().push("poll".into());
        Ok(self.control.lock().unwrap().events.drain(..).collect())
    }

    fn cancel(&mut self, _identity: &DispatchIdentity) -> Result<bool, PeerDispatchError> {
        self.trace.lock().unwrap().push("cancel".into());
        Ok(self.control.lock().unwrap().cancel_accepted)
    }
}

fn ready_coordinator_with_repository(
    root: &Path,
    trace: Arc<Mutex<Vec<String>>>,
) -> (RuntimeCoordinator<MemoryRepository>, u64, MemoryRepository) {
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(root)).unwrap();
    let repository = MemoryRepository {
        records: Arc::default(),
        trace,
    };
    let mut coordinator = RuntimeCoordinator::new(
        ProviderService::new(),
        RuntimeSupervisor::pinned(),
        SessionService::new(repository.clone()),
        ActionGateway::new(PolicyConfiguration::default(), Arc::new(database)),
    );
    coordinator.save_provider(profile(), 0).unwrap();
    coordinator
        .test_provider(
            "provider-openrouter",
            1,
            NOW,
            &mut FixtureProbe(ProviderDiscovery {
                checked_at_ms: NOW,
                models: vec![provider_route()],
                recommended_model_id: Some("claude-sonnet".into()),
                connection_evidence: None,
            }),
        )
        .unwrap();
    coordinator
        .register_runtime(installation(root), NOW + 1)
        .unwrap();
    let process_generation = coordinator
        .start_runtime("opencode-primary", NOW + 2)
        .unwrap()
        .value;
    coordinator
        .record_runtime_health(
            "opencode-primary",
            process_generation,
            HealthState::Healthy,
            NOW + 3,
        )
        .unwrap();
    coordinator
        .create_provisional("session-1", NOW + 4)
        .unwrap();
    (coordinator, process_generation, repository)
}

fn ready_coordinator(
    root: &Path,
    trace: Arc<Mutex<Vec<String>>>,
) -> (RuntimeCoordinator<MemoryRepository>, u64) {
    let (coordinator, process_generation, _) = ready_coordinator_with_repository(root, trace);
    (coordinator, process_generation)
}

fn registry_with_peer(
    process_generation: u64,
    control: Arc<Mutex<PeerControl>>,
    trace: Arc<Mutex<Vec<String>>>,
) -> RuntimeDispatchRegistry {
    let mut registry = RuntimeDispatchRegistry::new();
    registry
        .register(DeterministicPeer {
            registration: peer_registration(process_generation),
            control,
            trace,
        })
        .unwrap();
    registry
}

fn options() -> FirstDispatchOptions {
    FirstDispatchOptions {
        title: "Coordinator".into(),
        credential_reference: None,
        credential_lease_id: None,
        attachment_resolution: AttachmentPreflightResolution::NotRequired,
        broker_authority: None,
    }
}

struct RetryFixture<'a> {
    parent_attempt_id: &'a str,
    attempt_id: &'a str,
    correlation_id: &'a str,
    automatic: bool,
    reviewed_unknown_effect: bool,
    created_at_ms: u64,
}

fn retry_request(
    coordinator: &RuntimeCoordinator<MemoryRepository>,
    process_generation: u64,
    fixture: RetryFixture<'_>,
) -> CoordinatedRetry {
    let record = coordinator.session("session-1").unwrap();
    let context = AttemptContextSnapshot::from_binding(record.binding().unwrap());
    CoordinatedRetry {
        request: RetryRequest {
            session_id: "session-1".into(),
            parent_attempt_id: fixture.parent_attempt_id.into(),
            attempt_id: fixture.attempt_id.into(),
            authorization_scope_id: format!("authority-{}", fixture.attempt_id),
            correlation_id: fixture.correlation_id.into(),
            process_generation,
            context,
            automatic: fixture.automatic,
            reviewed_unknown_effect: fixture.reviewed_unknown_effect,
            created_at_ms: fixture.created_at_ms,
        },
        provider_id: "provider-openrouter".into(),
        selected_model_id: "claude-sonnet".into(),
        capability_layers: layers(),
        draft: draft(),
        preflight_at_ms: NOW + 5,
    }
}

fn retry_options() -> RetryDispatchOptions {
    RetryDispatchOptions {
        credential_reference: None,
        credential_lease_id: None,
        attachment_resolution: AttachmentPreflightResolution::NotRequired,
        broker_authority: None,
    }
}

fn turn_request(
    coordinator: &RuntimeCoordinator<MemoryRepository>,
    process_generation: u64,
    turn_id: &str,
    attempt_id: &str,
    correlation_id: &str,
    created_at_ms: u64,
) -> CoordinatedTurn {
    let record = coordinator.session("session-1").unwrap();
    let context = AttemptContextSnapshot::from_binding(record.binding().unwrap());
    CoordinatedTurn {
        submission: TurnSubmission {
            session_id: "session-1".into(),
            turn_id: turn_id.into(),
            attempt_id: attempt_id.into(),
            authorization_scope_id: format!("authority-{attempt_id}"),
            correlation_id: correlation_id.into(),
            process_generation,
            prompt: Some(format!("Prompt for {turn_id}")),
            attachments: Vec::new(),
            reply_context: None,
            context,
            submitted_at_ms: created_at_ms,
        },
        provider_id: "provider-openrouter".into(),
        selected_model_id: "claude-sonnet".into(),
        capability_layers: layers(),
        draft: draft(),
        preflight_at_ms: created_at_ms,
    }
}

fn turn_options() -> TurnDispatchOptions {
    TurnDispatchOptions {
        credential_reference: None,
        credential_lease_id: None,
        attachment_resolution: AttachmentPreflightResolution::NotRequired,
        broker_authority: None,
    }
}

#[test]
fn full_graph_orders_readiness_before_durability_and_preserves_action_identity() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        cancel_accepted: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));

    let dispatched = coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    assert!(matches!(
        dispatched,
        CoordinatedFirstDispatch::Accepted { .. }
    ));
    assert_eq!(
        *trace.lock().unwrap(),
        ["ready", "durable", "ready", "create", "dispatch"]
    );

    let identity = dispatch_identity(process_generation);
    let action_identity = RuntimeIntentIdentity {
        binding_sha256: digest('5'),
        workspace_id: identity.workspace_id.clone(),
        session_id: identity.session_id.clone(),
        turn_id: identity.turn_id.clone(),
        run_id: identity.attempt_id.clone(),
        correlation_id: identity.correlation_id.clone(),
        runtime_id: identity.runtime_id.clone(),
        process_generation,
        native_request_id: "native-action-1".into(),
        native_tool: "write_file".into(),
    };
    control.lock().unwrap().events.extend([
        PeerDispatchEvent {
            identity: identity.clone(),
            recorded_at_ms: NOW + 20,
            payload: "hello".into(),
            category: DispatchEventCategory::TextDelta,
        },
        PeerDispatchEvent {
            identity: identity.clone(),
            recorded_at_ms: NOW + 21,
            payload: "action-intent".into(),
            category: DispatchEventCategory::ActionIntent(Box::new(action_identity.clone())),
        },
    ]);

    let events = coordinate_polled_events(
        &mut coordinator,
        &mut registry,
        "opencode-primary",
        NOW + 21,
    )
    .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event.sequence, 1);
    assert_eq!(events[1].event.sequence, 2);
    let DispatchEventCategory::ActionIntent(observed_identity) = &events[1].event.peer.category
    else {
        panic!("second event must retain its normalized action identity");
    };
    assert_eq!(observed_identity.as_ref(), &action_identity);
    assert!(matches!(
        events[1].record.attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Streaming { .. }
    ));

    let cancelled =
        coordinate_cancellation(&mut coordinator, &mut registry, &identity, NOW + 22).unwrap();
    let CoordinatedCancellation::Cancelled { record, .. } = cancelled else {
        panic!("native cancellation acceptance must close as cancelled");
    };
    assert!(matches!(
        record.attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Cancelled { .. }
    ));
    assert!(trace.lock().unwrap().ends_with(&[
        "poll".to_owned(),
        "ready".to_owned(),
        "ready".to_owned(),
        "cancel".to_owned(),
    ]));
}

#[test]
fn caller_supplied_tool_ids_cannot_exceed_the_core_owned_resource_snapshot() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let mut request = first_submission(process_generation);
    request.draft.requires_tools = true;
    request.draft.policy.tool_use_allowed = true;
    request
        .draft
        .installed_resources
        .tool_ids
        .insert("renderer-invented-tool".into());

    assert!(matches!(
        coordinate_first_dispatch(&mut coordinator, &mut registry, request, options()),
        Err(DispatchError::InvalidResourcePreflight)
    ));
    assert!(trace.lock().unwrap().is_empty());
    assert!(control.lock().unwrap().requests.is_empty());
}

#[test]
fn approved_attachment_conversion_is_bound_to_peer_request_and_reused_on_retry() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        cancel_accepted: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let (request, plan) = attachment_submission(process_generation);

    let accepted = coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        request,
        attachment_options(plan.clone()),
    )
    .unwrap();
    assert!(matches!(
        accepted,
        CoordinatedFirstDispatch::Accepted { .. }
    ));
    let first_request = control.lock().unwrap().requests[0].clone();
    assert_eq!(first_request.attachments, plan.attachments());
    assert!(
        first_request
            .input
            .contains("[C4OS approved converter output; attachment=attachment-1")
    );
    assert!(
        first_request
            .input
            .contains("Verified converted coordinator notes.")
    );

    coordinate_cancellation(
        &mut coordinator,
        &mut registry,
        &dispatch_identity(process_generation),
        NOW + 20,
    )
    .unwrap();
    let mut retry = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-1",
            attempt_id: "attempt-2",
            correlation_id: "correlation-2",
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: NOW + 21,
        },
    );
    retry.draft = attachment_draft(&retry.request.context.resources, &attachment());
    let retried = coordinate_retry_dispatch(
        &mut coordinator,
        &mut registry,
        retry,
        attachment_retry_options(plan),
    )
    .unwrap();
    assert!(matches!(retried, CoordinatedRetryDispatch::Accepted { .. }));
    let control = control.lock().unwrap();
    assert_eq!(control.requests.len(), 2);
    assert_eq!(
        control.requests[0].attachments,
        control.requests[1].attachments
    );
    assert_eq!(control.requests[0].input, control.requests[1].input);
    assert_ne!(
        control.requests[0].identity.attempt_id,
        control.requests[1].identity.attempt_id
    );
}

#[test]
fn route_compatible_durable_attachment_dispatches_directly_without_conversion() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let request = direct_attachment_submission(process_generation, false);
    let durable_attachment = request.submission.attachments[0].clone();
    let dispatch_options = direct_options(temporary.path(), &request);

    let accepted =
        coordinate_first_dispatch(&mut coordinator, &mut registry, request, dispatch_options)
            .unwrap();

    assert!(matches!(
        accepted,
        CoordinatedFirstDispatch::Accepted { .. }
    ));
    let peer_request = &control.lock().unwrap().requests[0];
    assert_eq!(peer_request.input, "Review the attached notes");
    assert_eq!(peer_request.direct_attachments.len(), 1);
    assert_eq!(
        peer_request.direct_attachments[0].snapshot(),
        &durable_attachment
    );
    assert_eq!(
        peer_request.direct_attachments[0].content(),
        DIRECT_ATTACHMENT_CONTENT
    );
    assert!(peer_request.attachments.is_empty());
}

#[test]
fn route_compatible_attachment_only_submission_dispatches_exact_durable_metadata() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let request = direct_attachment_submission(process_generation, true);
    let durable_attachment = request.submission.attachments[0].clone();
    let dispatch_options = direct_options(temporary.path(), &request);

    coordinate_first_dispatch(&mut coordinator, &mut registry, request, dispatch_options).unwrap();

    let peer_request = &control.lock().unwrap().requests[0];
    assert!(peer_request.input.is_empty());
    assert_eq!(peer_request.direct_attachments.len(), 1);
    assert_eq!(
        peer_request.direct_attachments[0].snapshot(),
        &durable_attachment
    );
    assert_eq!(
        peer_request.direct_attachments[0].content(),
        DIRECT_ATTACHMENT_CONTENT
    );
    assert!(peer_request.attachments.is_empty());
}

#[test]
fn direct_attachment_metadata_mismatch_fails_before_durable_mutation() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let before = coordinator.session("session-1").unwrap();
    let mut request = direct_attachment_submission(process_generation, false);
    let dispatch_options = direct_options(temporary.path(), &request);
    request.draft.attachments[0].bytes += 1;

    assert!(matches!(
        coordinate_first_dispatch(&mut coordinator, &mut registry, request, dispatch_options),
        Err(DispatchError::InvalidDirectAttachment)
    ));
    assert_eq!(coordinator.session("session-1").unwrap(), before);
    assert!(control.lock().unwrap().requests.is_empty());
}

#[test]
fn attachment_materialization_rejects_tamper_and_incomplete_coverage_before_mutation() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let before = coordinator.session("session-1").unwrap();
    let (mut request, plan) = attachment_submission(process_generation);
    request.submission.attachments[0].stable_reference = "workspace-blob:tampered".into();

    assert!(matches!(
        coordinate_first_dispatch(
            &mut coordinator,
            &mut registry,
            request.clone(),
            attachment_options(plan),
        ),
        Err(DispatchError::InvalidAttachmentMaterialization)
    ));
    assert_eq!(coordinator.session("session-1").unwrap(), before);
    assert!(control.lock().unwrap().requests.is_empty());

    assert!(matches!(
        coordinate_first_dispatch(
            &mut coordinator,
            &mut registry,
            request,
            attachment_options(AttachmentMaterializationPlan::default()),
        ),
        Err(DispatchError::InvalidAttachmentMaterialization)
    ));
    assert_eq!(coordinator.session("session-1").unwrap(), before);
    assert!(control.lock().unwrap().requests.is_empty());
}

#[test]
fn attachment_remove_and_cancel_are_non_mutating_preflight_outcomes() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let before = coordinator.session("session-1").unwrap();
    let (request, _) = attachment_submission(process_generation);

    let remove = FirstDispatchOptions {
        attachment_resolution: AttachmentPreflightResolution::Remove {
            attachment_ids: vec!["attachment-1".into()],
        },
        ..options()
    };
    assert!(matches!(
        coordinate_first_dispatch(&mut coordinator, &mut registry, request.clone(), remove,),
        Err(DispatchError::AttachmentRemovalRequested)
    ));
    assert_eq!(coordinator.session("session-1").unwrap(), before);

    let cancel = FirstDispatchOptions {
        attachment_resolution: AttachmentPreflightResolution::Cancel,
        ..options()
    };
    assert!(matches!(
        coordinate_first_dispatch(&mut coordinator, &mut registry, request, cancel),
        Err(DispatchError::AttachmentDispatchCancelled)
    ));
    assert_eq!(coordinator.session("session-1").unwrap(), before);
    assert!(control.lock().unwrap().requests.is_empty());
}

#[test]
fn native_dispatch_failure_terminally_closes_the_durable_attempt() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        fail_dispatch: true,
        ..PeerControl::default()
    }));
    let mut registry = registry_with_peer(process_generation, control, Arc::clone(&trace));

    let result = coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    let CoordinatedFirstDispatch::Rejected {
        record, failure, ..
    } = result
    else {
        panic!("a native rejection must not be reported as accepted");
    };
    assert_eq!(
        failure,
        c4os_lib::runtime::dispatch::DispatchFailureCode::PeerRejected
    );
    assert!(matches!(
        record.attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Failed { .. }
    ));
    assert_eq!(record.active_attempt_id, None);
    assert_eq!(
        *trace.lock().unwrap(),
        ["ready", "durable", "ready", "create", "dispatch"]
    );
}

#[test]
fn missing_or_stale_peer_rejects_before_any_durable_session_mutation() {
    for stale in [false, true] {
        let temporary = TempDir::new().unwrap();
        let trace = Arc::new(Mutex::new(Vec::new()));
        let (mut coordinator, process_generation) =
            ready_coordinator(temporary.path(), Arc::clone(&trace));
        let mut registry = RuntimeDispatchRegistry::new();
        if stale {
            registry
                .register(DeterministicPeer {
                    registration: peer_registration(process_generation + 1),
                    control: Arc::new(Mutex::new(PeerControl {
                        ready: true,
                        ..PeerControl::default()
                    })),
                    trace: Arc::clone(&trace),
                })
                .unwrap();
        }

        let error = coordinate_first_dispatch(
            &mut coordinator,
            &mut registry,
            first_submission(process_generation),
            options(),
        )
        .unwrap_err();
        assert!(matches!(
            (stale, error),
            (false, DispatchError::PeerUnavailable) | (true, DispatchError::StalePeer)
        ));
        assert!(matches!(
            coordinator.session("session-1").unwrap().lifecycle,
            SessionLifecycle::Provisional
        ));
        assert!(trace.lock().unwrap().is_empty());
    }
}

#[test]
fn duplicate_registration_is_rejected_and_recovery_forgets_old_attempt_identity() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    let duplicate = registry.register(DeterministicPeer {
        registration: peer_registration(process_generation),
        control: Arc::new(Mutex::new(PeerControl::default())),
        trace: Arc::clone(&trace),
    });
    assert!(matches!(duplicate, Err(DispatchError::DuplicatePeer)));

    coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    let recovered = coordinate_recovery(&mut coordinator, &mut registry, NOW + 30).unwrap();
    assert_eq!(recovered.value.len(), 1);
    assert!(matches!(
        recovered.value[0].attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Interrupted { .. }
    ));

    control.lock().unwrap().events.push_back(PeerDispatchEvent {
        identity: dispatch_identity(process_generation),
        recorded_at_ms: NOW + 31,
        payload: "late".into(),
        category: DispatchEventCategory::TextDelta,
    });
    let late = coordinate_polled_events(
        &mut coordinator,
        &mut registry,
        "opencode-primary",
        NOW + 31,
    );
    assert!(matches!(late, Err(DispatchError::StalePeer)));
    assert_eq!(
        coordinator
            .session("session-1")
            .unwrap()
            .attempt("attempt-1")
            .unwrap()
            .events
            .len(),
        0
    );
}

#[test]
fn mismatched_action_identity_is_rejected_before_event_persistence() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();

    let identity = dispatch_identity(process_generation);
    control.lock().unwrap().events.push_back(PeerDispatchEvent {
        identity: identity.clone(),
        recorded_at_ms: NOW + 20,
        payload: "action-intent".into(),
        category: DispatchEventCategory::ActionIntent(Box::new(RuntimeIntentIdentity {
            binding_sha256: digest('4'),
            workspace_id: identity.workspace_id.clone(),
            session_id: identity.session_id.clone(),
            turn_id: identity.turn_id.clone(),
            run_id: "other-attempt".into(),
            correlation_id: identity.correlation_id.clone(),
            runtime_id: identity.runtime_id.clone(),
            process_generation,
            native_request_id: "native-action-2".into(),
            native_tool: "write_file".into(),
        })),
    });
    let result = coordinate_polled_events(
        &mut coordinator,
        &mut registry,
        "opencode-primary",
        NOW + 20,
    );
    assert!(matches!(result, Err(DispatchError::InvalidEvent)));
    assert!(
        coordinator
            .session("session-1")
            .unwrap()
            .attempt("attempt-1")
            .unwrap()
            .events
            .is_empty()
    );
}

#[test]
fn retry_reuses_native_session_with_fresh_identity_and_failure_terminally_closes() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        cancel_accepted: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    coordinate_cancellation(
        &mut coordinator,
        &mut registry,
        &dispatch_identity(process_generation),
        NOW + 20,
    )
    .unwrap();

    trace.lock().unwrap().clear();
    let retry = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-1",
            attempt_id: "attempt-2",
            correlation_id: "correlation-2",
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: NOW + 21,
        },
    );
    let result =
        coordinate_retry_dispatch(&mut coordinator, &mut registry, retry, retry_options()).unwrap();
    let CoordinatedRetryDispatch::Accepted { record, .. } = result else {
        panic!("safe manual retry must reach the existing native session");
    };
    let retry_attempt = record.attempt("attempt-2").unwrap();
    assert_eq!(
        retry_attempt.parent_attempt_id.as_deref(),
        Some("attempt-1")
    );
    assert_eq!(retry_attempt.correlation_id, "correlation-2");
    assert_eq!(retry_attempt.authorization_scope_id, "authority-attempt-2");
    assert_eq!(*trace.lock().unwrap(), ["ready", "ready", "dispatch"]);

    let retry_identity = DispatchIdentity {
        attempt_id: "attempt-2".into(),
        correlation_id: "correlation-2".into(),
        ..dispatch_identity(process_generation)
    };
    coordinate_cancellation(&mut coordinator, &mut registry, &retry_identity, NOW + 22).unwrap();
    control.lock().unwrap().fail_dispatch = true;
    let retry = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-2",
            attempt_id: "attempt-3",
            correlation_id: "correlation-3",
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: NOW + 23,
        },
    );
    let result =
        coordinate_retry_dispatch(&mut coordinator, &mut registry, retry, retry_options()).unwrap();
    let CoordinatedRetryDispatch::Rejected { record, .. } = result else {
        panic!("native retry rejection must not leave a dispatching attempt");
    };
    assert!(matches!(
        record.attempt("attempt-3").unwrap().status,
        RunAttemptStatus::Failed { .. }
    ));
    assert_eq!(record.active_attempt_id, None);
}

#[test]
fn follow_up_turn_reuses_native_session_and_terminally_closes_rejection() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation) =
        ready_coordinator(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        cancel_accepted: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    coordinate_cancellation(
        &mut coordinator,
        &mut registry,
        &dispatch_identity(process_generation),
        NOW + 20,
    )
    .unwrap();

    trace.lock().unwrap().clear();
    let mut turn = turn_request(
        &coordinator,
        process_generation,
        "turn-2",
        "attempt-2",
        "correlation-2",
        NOW + 21,
    );
    turn.submission.reply_context = Some(MessageReplyContextSnapshot {
        target_id: "attempt-1".into(),
        target_kind: "assistant-message".into(),
        source_sha256: digest('a'),
        source_excerpt: "Immutable earlier response".into(),
    });
    let accepted =
        coordinate_turn_dispatch(&mut coordinator, &mut registry, turn, turn_options()).unwrap();
    let CoordinatedTurnDispatch::Accepted { record, .. } = accepted else {
        panic!("follow-up turn must reach the existing native session");
    };
    assert_eq!(record.turns.len(), 2);
    assert_eq!(
        record.turn("turn-2").unwrap().prompt.as_deref(),
        Some("Prompt for turn-2")
    );
    assert_eq!(*trace.lock().unwrap(), ["ready", "ready", "dispatch"]);
    assert!(
        control
            .lock()
            .unwrap()
            .requests
            .last()
            .unwrap()
            .input
            .contains("<reply-context>\nImmutable earlier response")
    );
    assert_eq!(
        record
            .turn("turn-2")
            .unwrap()
            .reply_context
            .as_ref()
            .unwrap()
            .target_id,
        "attempt-1"
    );

    let identity = DispatchIdentity {
        turn_id: "turn-2".into(),
        attempt_id: "attempt-2".into(),
        correlation_id: "correlation-2".into(),
        ..dispatch_identity(process_generation)
    };
    coordinate_cancellation(&mut coordinator, &mut registry, &identity, NOW + 22).unwrap();
    control.lock().unwrap().fail_dispatch = true;
    let turn = turn_request(
        &coordinator,
        process_generation,
        "turn-3",
        "attempt-3",
        "correlation-3",
        NOW + 23,
    );
    let rejected =
        coordinate_turn_dispatch(&mut coordinator, &mut registry, turn, turn_options()).unwrap();
    let CoordinatedTurnDispatch::Rejected { record, .. } = rejected else {
        panic!("native turn rejection must not leave a dispatching attempt");
    };
    assert!(matches!(
        record.attempt("attempt-3").unwrap().status,
        RunAttemptStatus::Failed { .. }
    ));
    assert_eq!(record.active_attempt_id, None);
}

#[test]
fn retry_stale_peer_and_unknown_effect_rules_reject_before_new_attempt_persistence() {
    let temporary = TempDir::new().unwrap();
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (mut coordinator, process_generation, repository) =
        ready_coordinator_with_repository(temporary.path(), Arc::clone(&trace));
    let control = Arc::new(Mutex::new(PeerControl {
        ready: true,
        cancel_accepted: true,
        ..PeerControl::default()
    }));
    let mut registry =
        registry_with_peer(process_generation, Arc::clone(&control), Arc::clone(&trace));
    coordinate_first_dispatch(
        &mut coordinator,
        &mut registry,
        first_submission(process_generation),
        options(),
    )
    .unwrap();
    coordinate_cancellation(
        &mut coordinator,
        &mut registry,
        &dispatch_identity(process_generation),
        NOW + 20,
    )
    .unwrap();

    let mut stale_registry = registry_with_peer(
        process_generation + 1,
        Arc::new(Mutex::new(PeerControl {
            ready: true,
            ..PeerControl::default()
        })),
        Arc::clone(&trace),
    );
    let stale = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-1",
            attempt_id: "attempt-stale",
            correlation_id: "correlation-stale",
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: NOW + 21,
        },
    );
    assert!(matches!(
        coordinate_retry_dispatch(
            &mut coordinator,
            &mut stale_registry,
            stale,
            retry_options(),
        ),
        Err(DispatchError::StalePeer)
    ));
    assert_eq!(coordinator.session("session-1").unwrap().attempts.len(), 1);

    repository
        .records
        .lock()
        .unwrap()
        .get_mut("session-1")
        .unwrap()
        .attempts[0]
        .side_effects
        .push(SideEffectState::Unknown {
            action_id: "action-unknown".into(),
        });
    let unreviewed = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-1",
            attempt_id: "attempt-unreviewed",
            correlation_id: "correlation-unreviewed",
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: NOW + 22,
        },
    );
    assert!(matches!(
        coordinate_retry_dispatch(&mut coordinator, &mut registry, unreviewed, retry_options(),),
        Err(DispatchError::Coordinator(_))
    ));
    let automatic = retry_request(
        &coordinator,
        process_generation,
        RetryFixture {
            parent_attempt_id: "attempt-1",
            attempt_id: "attempt-automatic",
            correlation_id: "correlation-automatic",
            automatic: true,
            reviewed_unknown_effect: true,
            created_at_ms: NOW + 23,
        },
    );
    assert!(matches!(
        coordinate_retry_dispatch(&mut coordinator, &mut registry, automatic, retry_options(),),
        Err(DispatchError::Coordinator(_))
    ));
    assert_eq!(coordinator.session("session-1").unwrap().attempts.len(), 1);
}

#[derive(Default)]
struct PiCancellationControl {
    accepted: bool,
}

struct PiCancellationRunner {
    control: Arc<Mutex<PiCancellationControl>>,
}

impl PiSidecarRunner for PiCancellationRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        let request: Value =
            serde_json::from_str(request_line).map_err(|error| error.to_string())?;
        let operation = request["operation"]
            .as_str()
            .ok_or_else(|| "missing Pi operation".to_owned())?;
        let payload = match operation {
            "health" => pi_cancellation_health(),
            "version" => json!({
                "adapter": "PIAdapter",
                "adapterVersion": "0.1.0",
                "nativePackage": "@earendil-works/pi-coding-agent",
                "nativeVersion": PI_NATIVE_VERSION,
                "protocol": "c4os.pi.ndjson.v1"
            }),
            "session.create" => json!({
                "sessionId": request["sessionId"],
                "nativeSessionId": "native-pi-session-1",
                "persistence": "c4os-authoritative"
            }),
            "dispatch" => json!({ "accepted": true, "runId": request["runId"] }),
            "tool.resolve" => json!({ "resolved": true, "executedBySidecar": false }),
            "cancel" => {
                let accepted = self.control.lock().unwrap().accepted;
                json!({ "cancelled": accepted, "alreadyTerminal": false })
            }
            "shutdown" => json!({ "stopped": true }),
            _ => return Err(format!("unexpected Pi operation: {operation}")),
        };
        let mut lines = Vec::new();
        if operation == "dispatch" {
            lines.push(
                serde_json::to_string(&json!({
                    "schemaVersion": 1,
                    "kind": "event",
                    "eventId": "pi-cancellation-event-1",
                    "correlationId": request["correlationId"],
                    "processGeneration": request["processGeneration"],
                    "sequence": 1,
                    "runtime": "pi",
                    "workspaceId": request["workspaceId"],
                    "sessionId": request["sessionId"],
                    "turnId": request["turnId"],
                    "runId": request["runId"],
                    "category": "tool.action_intent",
                    "nativeType": "fixture.pi.tool",
                    "toolCallId": "pi-cancellation-tool-1",
                    "payload": {
                        "tool": "c4os_read_resource",
                        "arguments": { "target": "workspace:/README.md" },
                        "authority": "c4os-action-gateway-required"
                    }
                }))
                .unwrap(),
            );
        }
        lines.push(
            serde_json::to_string(&json!({
                "schemaVersion": 1,
                "kind": "response",
                "requestId": request["requestId"],
                "correlationId": request["correlationId"],
                "processGeneration": request["processGeneration"],
                "status": "ok",
                "payload": payload
            }))
            .unwrap(),
        );
        Ok(lines)
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn terminate(&mut self) -> Result<(), String> {
        Ok(())
    }
}

fn pi_cancellation_health() -> Value {
    let mut capabilities = BTreeMap::new();
    capabilities.insert("streaming", json!({ "state": "supported" }));
    capabilities.insert("cancellation", json!({ "state": "supported" }));
    capabilities.insert("tools", json!({ "state": "supported" }));
    capabilities.insert("nativePersistence", json!({ "state": "unsupported" }));
    capabilities.insert("nativeExtensions", json!({ "state": "unsupported" }));
    capabilities.insert("nativeTools", json!({ "state": "unsupported" }));
    capabilities.insert(
        "crashResume",
        json!({ "state": "degraded", "reason": "C4OS replay required" }),
    );
    capabilities.insert(
        "providerAuthentication",
        json!({ "state": "degraded", "reason": "fixture channel" }),
    );
    capabilities.insert("rpcTransport", json!({ "state": "unsupported" }));
    json!({
        "status": "ready",
        "runtime": "pi",
        "transport": "c4os-node-sdk-sidecar",
        "protocol": "c4os.pi.ndjson.v1",
        "processGeneration": 7,
        "sessions": 0,
        "staleEventsRejected": 0,
        "capabilities": capabilities
    })
}

#[derive(Default)]
struct PiRouteCaptureControl {
    native_model_routes: Vec<Value>,
}

struct PiRouteCaptureRunner {
    control: Arc<Mutex<PiRouteCaptureControl>>,
}

impl PiSidecarRunner for PiRouteCaptureRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        let request: Value =
            serde_json::from_str(request_line).map_err(|error| error.to_string())?;
        let operation = request["operation"]
            .as_str()
            .ok_or_else(|| "missing Pi operation".to_owned())?;
        let payload = match operation {
            "health" => pi_cancellation_health(),
            "version" => json!({
                "adapter": "PIAdapter",
                "adapterVersion": "0.1.0",
                "nativePackage": "@earendil-works/pi-coding-agent",
                "nativeVersion": PI_NATIVE_VERSION,
                "protocol": "c4os.pi.ndjson.v1"
            }),
            "session.create" => {
                self.control
                    .lock()
                    .unwrap()
                    .native_model_routes
                    .push(request["payload"]["modelRoute"].clone());
                json!({
                    "sessionId": request["sessionId"],
                    "nativeSessionId": format!("native-{}", request["sessionId"].as_str().unwrap()),
                    "persistence": "c4os-authoritative"
                })
            }
            "shutdown" => json!({ "stopped": true }),
            _ => return Err(format!("unexpected Pi operation: {operation}")),
        };
        Ok(vec![
            serde_json::to_string(&json!({
                "schemaVersion": 1,
                "kind": "response",
                "requestId": request["requestId"],
                "correlationId": request["correlationId"],
                "processGeneration": request["processGeneration"],
                "status": "ok",
                "payload": payload
            }))
            .unwrap(),
        ])
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn terminate(&mut self) -> Result<(), String> {
        Ok(())
    }
}

struct FixedPiCredentialIssuer {
    c4os_provider_id: String,
    native_provider_id: String,
}

impl PiDispatchCredentialIssuer for FixedPiCredentialIssuer {
    fn native_provider_id(&self, provider_id: &str) -> Result<String, PeerDispatchError> {
        if provider_id != self.c4os_provider_id {
            return Err(PeerDispatchError::Credential);
        }
        Ok(self.native_provider_id.clone())
    }

    fn base_url(&self, provider_id: &str) -> Result<String, PeerDispatchError> {
        if provider_id != self.c4os_provider_id {
            return Err(PeerDispatchError::Credential);
        }
        Ok(match self.native_provider_id.as_str() {
            "openrouter" => "https://openrouter.ai/api/v1",
            _ => "https://api.openai.com/v1",
        }
        .into())
    }

    fn deliver_for_dispatch(
        &mut self,
        _identity: &DispatchIdentity,
        _provider_id: &str,
    ) -> Result<(), PeerDispatchError> {
        Ok(())
    }
}

fn pi_route_capture_peer(
    c4os_provider_id: &str,
    native_provider_id: &str,
) -> (
    PiDispatchPeer<PiRouteCaptureRunner>,
    Arc<Mutex<PiRouteCaptureControl>>,
) {
    const PI_GENERATION: u64 = 7;

    let sidecar_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
    let manifest = PiSidecarManifest::load(&sidecar_root).unwrap();
    let descriptor = manifest
        .conformance_descriptor(PI_GENERATION, true)
        .unwrap();
    let control = Arc::new(Mutex::new(PiRouteCaptureControl::default()));
    let mut adapter = PiAdapter::new(
        manifest,
        PiRouteCaptureRunner {
            control: Arc::clone(&control),
        },
        PI_GENERATION,
    )
    .unwrap();
    adapter.start().unwrap();
    let mut peer = PiDispatchPeer::new(
        RuntimePeerRegistration {
            runtime_id: "pi-primary".into(),
            workspace_id: "workspace-1".into(),
            descriptor,
        },
        adapter,
    )
    .unwrap();
    peer.attach_credential_issuer(FixedPiCredentialIssuer {
        c4os_provider_id: c4os_provider_id.into(),
        native_provider_id: native_provider_id.into(),
    });
    (peer, control)
}

fn pi_route_request(
    c4os_provider_id: &str,
    canonical_model_id: &str,
    session_id: &str,
) -> PeerDispatchRequest {
    PeerDispatchRequest {
        identity: DispatchIdentity {
            workspace_id: "workspace-1".into(),
            environment_id: "local".into(),
            session_id: session_id.into(),
            turn_id: "turn-1".into(),
            attempt_id: "attempt-1".into(),
            correlation_id: "correlation-1".into(),
            runtime_id: "pi-primary".into(),
            runtime_kind: RuntimeKind::Pi,
            adapter_version: "1.0.0".into(),
            native_version: PI_NATIVE_VERSION.into(),
            process_generation: 7,
        },
        model: DispatchModelRoute {
            provider_id: c4os_provider_id.into(),
            model_id: canonical_model_id.into(),
            credential_reference: None,
            credential_lease_id: None,
        },
        title: "Pi native route".into(),
        input: "Create the exact native Pi session".into(),
        eligible_tool_ids: BTreeSet::from(["c4os_propose_action".into()]),
        broker_authority: None,
        direct_attachments: Vec::new(),
        attachments: Vec::new(),
    }
}

#[test]
fn pi_credential_route_strips_exact_openai_prefix_for_native_session_creation() {
    let (mut peer, control) = pi_route_capture_peer("openai-team-a", "openai");
    let request = pi_route_request("openai-team-a", "openai/gpt-4o-mini", "session-openai");

    peer.create_session(&request).unwrap();

    assert_eq!(
        control.lock().unwrap().native_model_routes,
        vec![json!({
            "provider": "openai",
            "modelId": "gpt-4o-mini",
            "baseUrl": "https://api.openai.com/v1"
        })]
    );
}

#[test]
fn pi_credential_route_strips_only_openrouter_prefix_and_preserves_nested_model_id() {
    let (mut peer, control) = pi_route_capture_peer("openrouter-team-a", "openrouter");
    let request = pi_route_request(
        "openrouter-team-a",
        "openrouter/anthropic/claude-sonnet-4",
        "session-openrouter",
    );

    peer.create_session(&request).unwrap();

    assert_eq!(
        control.lock().unwrap().native_model_routes,
        vec![json!({
            "provider": "openrouter",
            "modelId": "anthropic/claude-sonnet-4",
            "baseUrl": "https://openrouter.ai/api/v1"
        })]
    );
}

#[test]
fn pi_credential_route_rejects_provider_prefix_mismatch_before_native_session_creation() {
    let (mut peer, control) = pi_route_capture_peer("openai-team-a", "openai");
    let request = pi_route_request(
        "openai-team-a",
        "anthropic/claude-sonnet-4",
        "session-mismatch",
    );

    assert!(matches!(
        peer.create_session(&request),
        Err(PeerDispatchError::SessionCreate)
    ));
    assert!(control.lock().unwrap().native_model_routes.is_empty());
}

#[test]
fn pi_false_cancellation_retains_exact_peer_binding_until_true_cancellation() {
    const PI_GENERATION: u64 = 7;

    let sidecar_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
    let manifest = PiSidecarManifest::load(&sidecar_root).unwrap();
    let descriptor = manifest
        .conformance_descriptor(PI_GENERATION, true)
        .unwrap();
    let control = Arc::new(Mutex::new(PiCancellationControl::default()));
    let mut adapter = PiAdapter::new(
        manifest,
        PiCancellationRunner {
            control: Arc::clone(&control),
        },
        PI_GENERATION,
    )
    .unwrap();
    adapter.start().unwrap();

    let identity = DispatchIdentity {
        workspace_id: "workspace-1".into(),
        environment_id: "local".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        correlation_id: "correlation-1".into(),
        runtime_id: "pi-primary".into(),
        runtime_kind: RuntimeKind::Pi,
        adapter_version: "1.0.0".into(),
        native_version: PI_NATIVE_VERSION.into(),
        process_generation: PI_GENERATION,
    };
    let request = PeerDispatchRequest {
        identity: identity.clone(),
        model: DispatchModelRoute {
            provider_id: "openai".into(),
            model_id: "gpt-4o-mini".into(),
            credential_reference: None,
            credential_lease_id: None,
        },
        title: "Pi cancellation binding".into(),
        input: "Propose a brokered read".into(),
        eligible_tool_ids: BTreeSet::from(["c4os_read_resource".into()]),
        broker_authority: None,
        direct_attachments: vec![],
        attachments: vec![],
    };
    let mut registry = RuntimeDispatchRegistry::new();
    registry
        .register(
            PiDispatchPeer::new(
                RuntimePeerRegistration {
                    runtime_id: identity.runtime_id.clone(),
                    workspace_id: identity.workspace_id.clone(),
                    descriptor,
                },
                adapter,
            )
            .unwrap(),
        )
        .unwrap();
    registry.dispatch_first(&request).unwrap();

    assert!(!registry.cancel(&identity).unwrap());
    let events = registry.poll_events(&identity.runtime_id, NOW + 1).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].peer.identity, identity);
    assert!(matches!(
        &events[0].peer.category,
        DispatchEventCategory::PiActionIntent(_)
    ));
    registry
        .resolve_pi_denied(
            &events[0].peer.identity,
            "pi-cancellation-tool-1",
            "policy-denied",
        )
        .unwrap();

    control.lock().unwrap().accepted = true;
    assert!(registry.cancel(&identity).unwrap());
    assert!(matches!(
        registry.resolve_pi_denied(&identity, "pi-cancellation-tool-1", "policy-denied"),
        Err(DispatchError::Peer(PeerDispatchError::StaleIdentity))
    ));
}

#[cfg(unix)]
#[test]
fn production_dispatch_types_implement_the_peer_contract() {
    fn assert_peer<T: RuntimeDispatchPeer>() {}

    assert_peer::<ProductionOpenCodeDispatchPeer>();
    assert_peer::<ProductionPiDispatchPeer>();
}
