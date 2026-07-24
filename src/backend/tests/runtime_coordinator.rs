use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor, SnapshotQuery};
use c4os_lib::runtime::action_bridge::{
    RuntimeActionProposal, RuntimeApprovalDecision, RuntimeEffectResult, RuntimeGatewayDecision,
    RuntimeIntentIdentity,
};
use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, DraftRequirements, InstalledResourcePreflight,
    LimitConfidence, ModelLifecycle, NumericCapabilityEvidence, NumericCapabilityKey,
    PolicyPreflight, PreflightOutcome, RouteIdentity,
};
use c4os_lib::runtime::coordinator::{
    CoordinatedFirstSubmission, CoordinatedRetry, CoordinatorError, RuntimeCoordinator,
};
use c4os_lib::runtime::provider::{
    ModelRoute, PROVIDER_SCHEMA_VERSION, ProviderAuthentication, ProviderConnectionEvidence,
    ProviderDiscovery, ProviderEndpoint, ProviderKind, ProviderProbe, ProviderProbeFailure,
    ProviderProfile, ProviderService, RouteAvailability,
};
use c4os_lib::runtime::session::{
    AdapterBinding, AttemptContextSnapshot, AttemptIdentity, CapabilitySnapshot,
    ConfigurationSnapshot, ExecutionEnvironmentBinding, FirstSubmission, ModelRouteSnapshot,
    ResourceSnapshot, RetryRequest, RunAttemptStatus, RunEventKind, RunEventRecord,
    SessionLifecycle, SessionRecord, SessionRepository, SessionRepositoryError, SessionService,
    SideEffectState, TerminalAttemptOutcome, capability_snapshot_from_effective_descriptor,
};
use c4os_lib::runtime::supervisor::{
    HealthState, OPENCODE_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation,
    RuntimeKind, RuntimeSupervisor, sha256_file,
};
use c4os_lib::security::authorization::{
    ApprovalAnswer, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk,
    LiveAuthorityState,
};
use c4os_lib::security::credentials::CredentialVault;
use c4os_lib::security::gateway::{ActionGateway, NormalizedActionResult, NormalizedActionStatus};
use c4os_lib::security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence, PolicyConfiguration,
    RepositoryState,
};
use serde_json::json;
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;

#[derive(Clone, Default)]
struct MemoryRepository(Arc<Mutex<BTreeMap<String, SessionRecord>>>);

impl SessionRepository for MemoryRepository {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .get(session_id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .values()
            .cloned()
            .collect())
    }

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .0
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        if records.contains_key(&record.session_id) {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(record.session_id.clone(), record.clone());
        Ok(())
    }

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .0
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let current = records
            .get(session_id)
            .ok_or(SessionRepositoryError::Conflict)?;
        if current.revision != expected_revision {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(session_id.to_owned(), replacement.clone());
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

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
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
        authentication: ProviderAuthentication::Bearer,
        credential_reference: Some(vault.store("provider-key", b"fixture-secret").unwrap()),
        headers: BTreeMap::new(),
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
    let mut features = BTreeMap::new();
    for key in [
        CapabilityKey::InputText,
        CapabilityKey::OutputText,
        CapabilityKey::Streaming,
        CapabilityKey::ToolCalling,
    ] {
        features.insert(key, evidence(layer, key));
    }
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
            snapshot_sha256: digest('f'),
            tool_ids: set(["c4os-propose-action".into()]),
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
    let executable = root.join("bin/opencode-fixture");
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

fn first_submission(process_generation: u64) -> FirstSubmission {
    FirstSubmission {
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        authorization_scope_id: "authority-1".into(),
        correlation_id: "correlation-1".into(),
        process_generation,
        prompt: Some("Implement the coordinator".into()),
        attachments: vec![],
        skill_context: vec![],
        mcp_turn: None,
        binding: binding(),
        submitted_at_ms: NOW + 10,
    }
}

fn identity(attempt: &str, correlation: &str, generation: u64) -> AttemptIdentity {
    AttemptIdentity {
        attempt_id: attempt.into(),
        correlation_id: correlation.into(),
        process_generation: generation,
    }
}

fn retry_request(record: &SessionRecord, process_generation: u64) -> RetryRequest {
    let binding = record.binding().unwrap();
    RetryRequest {
        session_id: "session-1".into(),
        parent_attempt_id: "attempt-1".into(),
        attempt_id: "attempt-2".into(),
        authorization_scope_id: "authority-2".into(),
        correlation_id: "correlation-2".into(),
        process_generation,
        context: AttemptContextSnapshot::from_binding(binding),
        automatic: false,
        reviewed_unknown_effect: false,
        created_at_ms: NOW + 15,
    }
}

fn action_facts() -> ActionFacts {
    ActionFacts {
        action_kind: "file.write".into(),
        native_tool: "write_file".into(),
        surface: ActionSurface::File,
        effects: set([ActionEffect::Modify]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Runtime,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: RepositoryState::VersionControlled,
        inside_active_project: true,
        canonical_target: "workspace:/project/README.md".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        runtime_id: "opencode-primary".into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}

fn action(process_generation: u64) -> CanonicalAction {
    CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: "action-1".into(),
        tool_call_id: "call-1".into(),
        tool: "write_file".into(),
        arguments: json!({"path": "README.md", "contentSha256": "sha256:new"}),
        risk: CanonicalRisk::Medium,
        requested_authority: set(["workspace-files.modify".into()]),
        canonical_target: "workspace:/project/README.md".into(),
        target_version: "sha256:old".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: "attempt-2".into(),
        runtime_id: "opencode-primary".into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: None,
        process_generation,
        configuration_version: 1,
        policy_version: 1,
        revocation_epoch: 1,
    }
}

fn live(process_generation: u64) -> LiveAuthorityState {
    LiveAuthorityState {
        process_generation,
        configuration_version: 1,
        policy_version: 1,
        revocation_epoch: 1,
    }
}

fn coordinator(root: &Path) -> RuntimeCoordinator<MemoryRepository> {
    coordinator_with_database(root).0
}

fn coordinator_with_database(
    root: &Path,
) -> (RuntimeCoordinator<MemoryRepository>, Arc<DatabaseActor>) {
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(root)).unwrap();
    let database = Arc::new(database);
    (
        RuntimeCoordinator::new(
            ProviderService::new(),
            RuntimeSupervisor::pinned(),
            SessionService::new(MemoryRepository::default()),
            ActionGateway::new(PolicyConfiguration::default(), Arc::clone(&database)),
        ),
        database,
    )
}

#[test]
fn full_runtime_call_graph_is_coordinated_with_one_monotonic_generation() {
    let temporary = TempDir::new().unwrap();
    let mut coordinator = coordinator(temporary.path());
    let mut generations = vec![];

    generations.push(
        coordinator
            .save_provider(profile(), 0)
            .unwrap()
            .coordinator_generation,
    );
    let tested = coordinator
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
    generations.push(tested.coordinator_generation);
    generations.push(
        coordinator
            .register_runtime(installation(temporary.path()), NOW + 1)
            .unwrap()
            .coordinator_generation,
    );
    let started = coordinator
        .start_runtime("opencode-primary", NOW + 2)
        .unwrap();
    let process_generation = started.value;
    generations.push(started.coordinator_generation);
    generations.push(
        coordinator
            .record_runtime_health(
                "opencode-primary",
                process_generation,
                HealthState::Healthy,
                NOW + 3,
            )
            .unwrap()
            .coordinator_generation,
    );
    generations.push(
        coordinator
            .create_provisional("session-1", NOW + 4)
            .unwrap()
            .coordinator_generation,
    );

    let preflight = coordinator
        .model_preflight(
            "provider-openrouter",
            "claude-sonnet",
            &layers(),
            &draft(),
            NOW + 5,
        )
        .unwrap();
    assert!(matches!(preflight.outcome, PreflightOutcome::Ready { .. }));
    assert_eq!(
        coordinator.snapshot(NOW + 5).generation,
        *generations.last().unwrap()
    );

    let submitted = coordinator
        .submit_first(CoordinatedFirstSubmission {
            submission: first_submission(process_generation),
            provider_id: "provider-openrouter".into(),
            selected_model_id: "claude-sonnet".into(),
            capability_layers: layers(),
            draft: draft(),
            preflight_at_ms: NOW + 5,
        })
        .unwrap();
    generations.push(submitted.coordinator_generation);
    assert!(matches!(
        submitted.value.lifecycle,
        SessionLifecycle::Bound { .. }
    ));

    let first_identity = identity("attempt-1", "correlation-1", process_generation);
    generations.push(
        coordinator
            .append_normalized_event(
                "session-1",
                &first_identity,
                RunEventRecord {
                    sequence: 1,
                    session_id: "session-1".into(),
                    turn_id: "turn-1".into(),
                    attempt_id: "attempt-1".into(),
                    runtime_id: "opencode-primary".into(),
                    environment_id: "local".into(),
                    correlation_id: "correlation-1".into(),
                    process_generation,
                    kind: RunEventKind::TextDelta,
                    payload: "hello".into(),
                    recorded_at_ms: NOW + 11,
                },
            )
            .unwrap()
            .coordinator_generation,
    );
    generations.push(
        coordinator
            .request_cancellation("session-1", &first_identity, NOW + 12)
            .unwrap()
            .coordinator_generation,
    );
    let cancelled = coordinator
        .finish_attempt(
            "session-1",
            &first_identity,
            TerminalAttemptOutcome::Cancelled,
            NOW + 13,
        )
        .unwrap();
    generations.push(cancelled.coordinator_generation);

    let retried = coordinator
        .retry(CoordinatedRetry {
            request: retry_request(&cancelled.value, process_generation),
            provider_id: "provider-openrouter".into(),
            selected_model_id: "claude-sonnet".into(),
            capability_layers: layers(),
            draft: draft(),
            preflight_at_ms: NOW + 14,
        })
        .unwrap();
    generations.push(retried.coordinator_generation);

    let intent = RuntimeIntentIdentity {
        binding_sha256: digest('a'),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        run_id: "attempt-2".into(),
        correlation_id: "correlation-2".into(),
        runtime_id: "opencode-primary".into(),
        process_generation,
        native_request_id: "call-1".into(),
        native_tool: "write_file".into(),
    };
    let proposal =
        RuntimeActionProposal::new(intent, action_facts(), action(process_generation)).unwrap();
    let proposed = coordinator
        .propose_runtime_action(proposal, NOW + 16)
        .unwrap();
    generations.push(proposed.coordinator_generation);
    let RuntimeGatewayDecision::PendingApproval { prompt_id, .. } = proposed.value else {
        panic!("default policy must keep the explicit approval boundary");
    };
    let approved = coordinator
        .answer_runtime_approval(&prompt_id, ApprovalAnswer::Allow, NOW + 17)
        .unwrap();
    generations.push(approved.coordinator_generation);
    let RuntimeApprovalDecision::Authorized(authorization) = approved.value else {
        panic!("allowed approval must produce a sealed authorization");
    };
    let executed = coordinator
        .execute_runtime_action(*authorization, live(process_generation), NOW + 18, |_| {
            NormalizedActionResult {
                status: NormalizedActionStatus::Succeeded,
                result_code: "worker-completed".into(),
                exit_code: None,
                changed_targets: vec!["workspace:/project/README.md".into()],
                output_sha256: Some(digest('1')),
                completed_at_ms: NOW + 19,
            }
        })
        .unwrap();
    generations.push(executed.coordinator_generation);
    assert_eq!(
        executed.value.result().status,
        NormalizedActionStatus::Succeeded
    );

    let cancelled_effects = AtomicUsize::new(0);
    let mut cancelled_action = action(process_generation);
    cancelled_action.action_id = "action-cancelled".into();
    cancelled_action.tool_call_id = "call-cancelled".into();
    let cancelled_intent = RuntimeIntentIdentity {
        binding_sha256: digest('b'),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        run_id: "attempt-2".into(),
        correlation_id: "correlation-2".into(),
        runtime_id: "opencode-primary".into(),
        process_generation,
        native_request_id: "call-cancelled".into(),
        native_tool: "write_file".into(),
    };
    let cancelled_proposal =
        RuntimeActionProposal::new(cancelled_intent, action_facts(), cancelled_action).unwrap();
    let pending = coordinator
        .propose_runtime_action(cancelled_proposal, NOW + 20)
        .unwrap();
    generations.push(pending.coordinator_generation);
    let RuntimeGatewayDecision::PendingApproval { prompt_id, .. } = pending.value else {
        panic!("second runtime action must retain the approval boundary");
    };
    let approved = coordinator
        .answer_runtime_approval(&prompt_id, ApprovalAnswer::Allow, NOW + 21)
        .unwrap();
    generations.push(approved.coordinator_generation);
    let RuntimeApprovalDecision::Authorized(cancelled_authorization) = approved.value else {
        panic!("approved action must yield a sealed authorization");
    };
    let cancelled = coordinator
        .cancel_runtime_actions("attempt-2", NOW + 22)
        .unwrap();
    generations.push(cancelled.coordinator_generation);
    let generation_before_rejected_execution = coordinator.snapshot(NOW + 22).generation;
    assert!(
        coordinator
            .execute_runtime_action(
                *cancelled_authorization,
                live(process_generation),
                NOW + 23,
                |_| {
                    cancelled_effects.fetch_add(1, Ordering::SeqCst);
                    NormalizedActionResult {
                        status: NormalizedActionStatus::Succeeded,
                        result_code: "must-not-run".into(),
                        exit_code: None,
                        changed_targets: vec!["workspace:/project/README.md".into()],
                        output_sha256: Some(digest('2')),
                        completed_at_ms: NOW + 24,
                    }
                },
            )
            .is_err(),
        "run cancellation must burn the sealed authorization"
    );
    assert_eq!(cancelled_effects.load(Ordering::SeqCst), 0);
    assert_eq!(
        coordinator.snapshot(NOW + 23).generation,
        generation_before_rejected_execution,
        "a rejected post-cancel execution must not mutate coordinator state"
    );

    let recovered = coordinator.recover_interrupted(NOW + 25).unwrap();
    generations.push(recovered.coordinator_generation);
    assert_eq!(recovered.value.len(), 1);
    assert!(matches!(
        recovered.value[0].attempt("attempt-2").unwrap().status,
        RunAttemptStatus::Interrupted { .. }
    ));
    assert_eq!(
        coordinator.snapshot(NOW + 25).generation,
        *generations.last().unwrap()
    );
    assert!(generations.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn late_exact_effect_completion_records_gateway_result_but_preserves_terminal_unknown() {
    let temporary = TempDir::new().unwrap();
    let (mut coordinator, database) = coordinator_with_database(temporary.path());
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
        .register_runtime(installation(temporary.path()), NOW + 1)
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
    coordinator
        .submit_first(CoordinatedFirstSubmission {
            submission: first_submission(process_generation),
            provider_id: "provider-openrouter".into(),
            selected_model_id: "claude-sonnet".into(),
            capability_layers: layers(),
            draft: draft(),
            preflight_at_ms: NOW + 5,
        })
        .unwrap();

    let action_id = "action-late-completion";
    let native_request_id = "call-late-completion";
    let mut late_action = action(process_generation);
    late_action.action_id = action_id.into();
    late_action.tool_call_id = native_request_id.into();
    late_action.run_id = "attempt-1".into();
    let intent = RuntimeIntentIdentity {
        binding_sha256: digest('c'),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        run_id: "attempt-1".into(),
        correlation_id: "correlation-1".into(),
        runtime_id: "opencode-primary".into(),
        process_generation,
        native_request_id: native_request_id.into(),
        native_tool: "write_file".into(),
    };
    let proposed = coordinator
        .propose_runtime_action(
            RuntimeActionProposal::new(intent, action_facts(), late_action).unwrap(),
            NOW + 16,
        )
        .unwrap();
    let RuntimeGatewayDecision::PendingApproval { prompt_id, .. } = proposed.value else {
        panic!("late effect fixture must retain explicit approval");
    };
    let approved = coordinator
        .answer_runtime_approval(&prompt_id, ApprovalAnswer::Allow, NOW + 17)
        .unwrap();
    let RuntimeApprovalDecision::Authorized(authorization) = approved.value else {
        panic!("approval must yield exact authorization");
    };
    let lease = coordinator
        .begin_runtime_action_effect(*authorization, live(process_generation), NOW + 18)
        .unwrap()
        .value;
    let started = coordinator.session("session-1").unwrap();
    assert_eq!(
        started.attempt("attempt-1").unwrap().side_effects,
        vec![SideEffectState::Started {
            action_id: action_id.into(),
            idempotent: false,
        }]
    );

    let terminal = coordinator
        .finish_attempt(
            "session-1",
            &identity("attempt-1", "correlation-1", process_generation),
            TerminalAttemptOutcome::Interrupted {
                reason_code: "worker-channel-closed".into(),
            },
            NOW + 19,
        )
        .unwrap()
        .value;
    assert!(matches!(
        terminal.attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Interrupted { .. }
    ));
    assert_eq!(terminal.active_attempt_id, None);
    assert_eq!(
        terminal.attempt("attempt-1").unwrap().side_effects,
        vec![SideEffectState::Unknown {
            action_id: action_id.into(),
        }]
    );

    let receipt = coordinator
        .complete_runtime_action_effect(
            lease,
            RuntimeEffectResult::normalized(NormalizedActionResult {
                status: NormalizedActionStatus::Succeeded,
                result_code: "late-worker-completed".into(),
                exit_code: None,
                changed_targets: vec!["workspace:/project/README.md".into()],
                output_sha256: Some(digest('3')),
                completed_at_ms: NOW + 20,
            }),
            NOW + 20,
        )
        .unwrap()
        .value;
    assert_eq!(receipt.result().status, NormalizedActionStatus::Succeeded);
    assert_eq!(receipt.result().result_code, "late-worker-completed");

    let recovered = coordinator.session("session-1").unwrap();
    assert!(matches!(
        recovered.attempt("attempt-1").unwrap().status,
        RunAttemptStatus::Interrupted { .. }
    ));
    assert_eq!(
        recovered.attempt("attempt-1").unwrap().side_effects,
        vec![SideEffectState::Unknown {
            action_id: action_id.into(),
        }]
    );

    let records = database
        .security_records(SnapshotQuery::new(50).unwrap())
        .unwrap();
    let result_records = records
        .iter()
        .filter(|record| record.action_id == action_id && record.record_kind == "action-result")
        .collect::<Vec<_>>();
    assert_eq!(result_records.len(), 1);
    assert_eq!(result_records[0].state, "succeeded");
    assert_eq!(result_records[0].recorded_at_ms, NOW + 20);
    let intent_records = records
        .iter()
        .filter(|record| record.action_id == action_id && record.record_kind == "action-intent")
        .collect::<Vec<_>>();
    assert_eq!(intent_records.len(), 1);
    assert_eq!(intent_records[0].state, "effect-finished");
}

#[test]
fn blocked_or_stale_authority_cannot_dispatch_and_does_not_advance_generation() {
    let temporary = TempDir::new().unwrap();
    let mut coordinator = coordinator(temporary.path());
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
        .register_runtime(installation(temporary.path()), NOW + 1)
        .unwrap();
    let started = coordinator
        .start_runtime("opencode-primary", NOW + 2)
        .unwrap();
    coordinator
        .record_runtime_health(
            "opencode-primary",
            started.value,
            HealthState::Healthy,
            NOW + 3,
        )
        .unwrap();
    coordinator
        .create_provisional("session-1", NOW + 4)
        .unwrap();
    let before = coordinator.snapshot(NOW + 5).generation;

    let mut blocked_draft = draft();
    blocked_draft.requires_tools = true;
    blocked_draft.policy.tool_use_allowed = false;
    let resolved = coordinator
        .model_preflight(
            "provider-openrouter",
            "claude-sonnet",
            &layers(),
            &blocked_draft,
            NOW + 5,
        )
        .unwrap();
    assert!(matches!(resolved.outcome, PreflightOutcome::Blocked { .. }));
    assert_eq!(coordinator.snapshot(NOW + 5).generation, before);

    assert!(matches!(
        coordinator.submit_first(CoordinatedFirstSubmission {
            submission: first_submission(started.value + 1),
            provider_id: "provider-openrouter".into(),
            selected_model_id: "claude-sonnet".into(),
            capability_layers: layers(),
            draft: draft(),
            preflight_at_ms: NOW + 5,
        }),
        Err(CoordinatorError::RuntimeNotReady)
    ));
    assert_eq!(coordinator.snapshot(NOW + 5).generation, before);
}
