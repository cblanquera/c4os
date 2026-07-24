use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionProposal, RuntimeApprovalDecision, RuntimeExecutionReceipt,
    RuntimeGatewayDecision, RuntimeIntentIdentity,
};
use c4os_lib::runtime::pi::{
    PI_MAX_IMAGE_BYTES, PI_MAX_LINE_BYTES, PI_NATIVE_VERSION, PiAdapter, PiAdapterError,
    PiAdapterState, PiDispatchAttachment, PiLaunchSpec, PiModelRoute, PiSamplingMessage,
    PiSamplingPoll, PiSamplingRequest, PiSidecarManifest, PiSidecarRunner, PiToolDecision,
    c4os_tool_names,
};
use c4os_lib::security::authorization::{
    ApprovalAnswer, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk,
    LiveAuthorityState,
};
use c4os_lib::security::gateway::{ActionGateway, NormalizedActionResult, NormalizedActionStatus};
use c4os_lib::security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence, PolicyConfiguration,
    RepositoryState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

const GENERATION: u64 = 7;

fn content_digest(content: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(content) {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn gateway_receipt_for_pi(
    intent: &c4os_lib::runtime::pi::PiEventEnvelope,
    runtime_id: &str,
    status: NormalizedActionStatus,
) -> RuntimeExecutionReceipt {
    let mut identity = RuntimeIntentIdentity::from_pi(intent).unwrap();
    identity.runtime_id = runtime_id.into();
    let facts = ActionFacts {
        action_kind: "file.read".into(),
        native_tool: "c4os_read_resource".into(),
        surface: ActionSurface::File,
        effects: set([ActionEffect::Read]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Runtime,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: RepositoryState::VersionControlled,
        inside_active_project: true,
        canonical_target: "workspace:/README.md".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        runtime_id: runtime_id.into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: "pi-action-1".into(),
        tool_call_id: "tool-call-1".into(),
        tool: "c4os_read_resource".into(),
        arguments: json!({ "target": "workspace:/README.md" }),
        risk: CanonicalRisk::Low,
        requested_authority: set(["workspace-files.read".into()]),
        canonical_target: "workspace:/README.md".into(),
        target_version: "sha256:fixture".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: "run-1".into(),
        runtime_id: runtime_id.into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: None,
        process_generation: GENERATION,
        configuration_version: 1,
        policy_version: 1,
        revocation_epoch: 1,
    };
    let proposal = RuntimeActionProposal::new(identity, facts, action).unwrap();
    let temporary = TempDir::new().unwrap();
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).unwrap();
    let mut gateway = ActionGateway::new(PolicyConfiguration::default(), Arc::new(database));
    let mut bridge = RuntimeActionBridge::new(&mut gateway);
    let authorization = match bridge.propose(proposal, 10).unwrap() {
        RuntimeGatewayDecision::Authorized(authorization) => authorization,
        RuntimeGatewayDecision::PendingApproval { prompt_id, .. } => {
            let RuntimeApprovalDecision::Authorized(authorization) = bridge
                .answer_approval(&prompt_id, ApprovalAnswer::Allow, 11)
                .unwrap()
            else {
                panic!("approval must authorize the exact Pi proposal")
            };
            authorization
        }
        RuntimeGatewayDecision::Denied { .. } => panic!("fixture proposal must be eligible"),
    };
    bridge
        .execute(
            *authorization,
            LiveAuthorityState {
                process_generation: GENERATION,
                configuration_version: 1,
                policy_version: 1,
                revocation_epoch: 1,
            },
            12,
            |_| NormalizedActionResult {
                status,
                result_code: "resource-read".into(),
                exit_code: None,
                changed_targets: vec![],
                output_sha256: Some(format!("sha256:{}", "a".repeat(64))),
                completed_at_ms: 12,
            },
        )
        .unwrap()
}

#[test]
fn manifest_and_package_validate_exact_pi_sdk_sidecar_boundary() {
    let manifest = load_manifest();
    assert_eq!(manifest.native_version, "0.80.10");
    assert_eq!(manifest.transport, "c4os-node-sdk-sidecar");
    assert_eq!(manifest.persistence, "c4os-authoritative-in-memory-worker");
    assert_eq!(manifest.tools, "c4os-brokered-only");
    assert_eq!(manifest.extensions, "disabled");
    assert_eq!(manifest.max_line_bytes, PI_MAX_LINE_BYTES);
    assert_eq!(c4os_tool_names().len(), 2);
}

#[test]
fn manifest_rejects_version_authority_and_entrypoint_changes() {
    let mut manifest = load_manifest();
    manifest.native_version = "0.80.11".into();
    assert!(matches!(
        manifest.validate(),
        Err(PiAdapterError::InvalidManifest(_))
    ));
    manifest.native_version = PI_NATIVE_VERSION.into();
    manifest.tools = "native".into();
    assert!(manifest.validate().is_err());
    manifest.tools = "c4os-brokered-only".into();
    manifest.entrypoint = "../main.mjs".into();
    assert!(manifest.validate().is_err());
}

#[test]
fn launch_spec_uses_only_generation_and_dedicated_credential_descriptor() {
    let manifest = load_manifest();
    let launch = PiLaunchSpec::new(
        PathBuf::from("/opt/c4os/node"),
        &sidecar_root(),
        &manifest,
        GENERATION,
        Some(3),
    )
    .unwrap();
    assert!(launch.environment.is_empty());
    assert_eq!(launch.executable, Path::new("/opt/c4os/node"));
    assert_eq!(
        launch.arguments.first().map(String::as_str),
        Some("--use-system-ca")
    );
    assert!(launch.arguments.iter().any(|arg| arg == "--generation=7"));
    assert!(
        launch
            .arguments
            .iter()
            .any(|arg| arg == "--credential-fd=3")
    );
    assert!(!launch.arguments.iter().any(|arg| {
        let lower = arg.to_ascii_lowercase();
        lower.contains("api_key=") || lower.contains("password=") || lower.contains("token=")
    }));
    assert!(
        PiLaunchSpec::new(
            PathBuf::from("node"),
            &sidecar_root(),
            &manifest,
            GENERATION,
            None
        )
        .is_err()
    );
}

#[test]
fn start_validates_health_version_and_explicit_capability_states() {
    let mut adapter = adapter(FakeRunner::default());
    let (health, version) = adapter.start().unwrap();
    assert_eq!(adapter.state(), &PiAdapterState::Degraded);
    assert_eq!(version.native_version, PI_NATIVE_VERSION);
    assert_eq!(health.capabilities["streaming"].state, "supported");
    assert_eq!(
        health.capabilities["nativePersistence"].state,
        "unsupported"
    );
    assert_eq!(health.capabilities["nativeExtensions"].state, "unsupported");
    assert_eq!(health.capabilities["nativeTools"].state, "unsupported");
    assert_eq!(
        health.capabilities["providerAuthentication"].state,
        "degraded"
    );
}

#[test]
fn start_rejects_any_pi_claim_to_native_tool_extension_or_persistence_authority() {
    let runner = FakeRunner {
        unsafe_health: true,
        ..FakeRunner::default()
    };
    let error = adapter(runner).start().unwrap_err();
    assert!(matches!(error, PiAdapterError::Protocol(_)));
}

#[test]
fn session_create_serializes_the_exact_eligible_tool_set_without_a_core_credential_reference() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    let request = adapter
        .runner()
        .requests()
        .iter()
        .find(|request| request["operation"] == "session.create")
        .unwrap();
    assert_eq!(request["payload"]["modelRoute"]["provider"], "openai");
    assert_eq!(request["payload"]["modelRoute"]["modelId"], "gpt-4o-mini");
    assert_eq!(
        request["payload"]["eligibleTools"],
        json!(["c4os_propose_action"])
    );
    assert!(
        request["payload"]["modelRoute"]
            .get("credentialReference")
            .is_none()
    );
}

#[test]
fn model_preflight_returns_exact_fail_closed_availability_without_creating_a_session() {
    let route = PiModelRoute {
        provider: "openai".into(),
        model_id: "gpt-4o-mini".into(),
        base_url: "https://api.openai.com/v1".into(),
    };
    let mut available = started_adapter(FakeRunner::default());
    assert_eq!(available.preflight_model(&route), Ok(true));
    let request = available
        .runner()
        .requests()
        .iter()
        .find(|request| request["operation"] == "model.preflight")
        .unwrap();
    assert_eq!(
        request["payload"],
        json!({ "provider": "openai", "modelId": "gpt-4o-mini" })
    );
    assert!(request.get("workspaceId").is_none());
    assert!(request.get("sessionId").is_none());
    assert_eq!(
        available
            .runner()
            .requests()
            .iter()
            .filter(|request| request["operation"] == "session.create")
            .count(),
        0
    );

    let mut unavailable = started_adapter(FakeRunner {
        missing_model: true,
        ..FakeRunner::default()
    });
    assert_eq!(unavailable.preflight_model(&route), Ok(false));

    let mut mismatched = started_adapter(FakeRunner {
        mismatched_model_preflight: true,
        ..FakeRunner::default()
    });
    assert!(matches!(
        mismatched.preflight_model(&route),
        Err(PiAdapterError::Protocol(_))
    ));
}

#[test]
fn session_dispatch_and_normalized_events_preserve_c4os_correlation() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "hello",
        )
        .unwrap();
    let events = adapter.take_events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.workspace_id, "workspace-1");
    assert_eq!(event.session_id, "session-1");
    assert_eq!(event.turn_id, "turn-1");
    assert_eq!(event.run_id, "run-1");
    assert_eq!(event.correlation_id, "correlation-run-1");
    assert_eq!(event.category, "content.delta");
    let request = adapter
        .runner()
        .requests()
        .iter()
        .find(|request| request["operation"] == "dispatch")
        .unwrap();
    assert!(request["payload"].get("credentialLeaseId").is_none());
    assert!(request["payload"].get("credentialReference").is_none());
}

#[test]
fn attachment_only_dispatch_serializes_exact_durable_metadata() {
    let content = b"known-png-content";
    let content_sha256 = content_digest(content);
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch_with_attachments(
            "workspace-1",
            "session-1",
            "turn-attachment",
            "run-attachment",
            "correlation-attachment",
            "",
            &[PiDispatchAttachment {
                attachment_id: "attachment-1".into(),
                stable_reference: "workspace-blob:attachment-1".into(),
                display_name: "concept.png".into(),
                media_type: "image/png".into(),
                byte_length: content.len() as u64,
                content_sha256: content_sha256.clone(),
                snapshot_version: 3,
                content: content.to_vec(),
            }],
        )
        .unwrap();

    let dispatch = adapter
        .runner()
        .requests()
        .iter()
        .find(|request| request["operation"] == "dispatch")
        .unwrap();
    assert_eq!(dispatch["payload"]["input"], "");
    assert_eq!(
        dispatch["payload"]["attachments"],
        json!([{
            "attachmentId": "attachment-1",
            "stableReference": "workspace-blob:attachment-1",
            "displayName": "concept.png",
            "mediaType": "image/png",
            "byteLength": content.len(),
            "contentSha256": content_sha256,
            "snapshotVersion": 3,
            "contentBase64": "a25vd24tcG5nLWNvbnRlbnQ="
        }])
    );
}

#[test]
fn pdf_and_tampered_image_content_are_rejected_before_sidecar_transport() {
    let content = b"known-png-content";
    let content_sha256 = content_digest(content);
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    let requests_before = adapter.runner().requests().len();
    let base = PiDispatchAttachment {
        attachment_id: "attachment-1".into(),
        stable_reference: "workspace-blob:attachment-1".into(),
        display_name: "concept.png".into(),
        media_type: "image/png".into(),
        byte_length: content.len() as u64,
        content_sha256,
        snapshot_version: 3,
        content: content.to_vec(),
    };
    let mut pdf = base.clone();
    pdf.display_name = "brief.pdf".into();
    pdf.media_type = "application/pdf".into();
    let mut tampered = base;
    tampered.content[0] ^= 1;

    for attachment in [pdf, tampered] {
        assert!(matches!(
            adapter.dispatch_with_attachments(
                "workspace-1",
                "session-1",
                "turn-invalid",
                "run-invalid",
                "correlation-invalid",
                "",
                &[attachment],
            ),
            Err(PiAdapterError::Protocol(_))
        ));
    }
    assert_eq!(adapter.runner().requests().len(), requests_before);
}

#[test]
fn decoded_image_wire_boundary_accepts_128_kib_and_rejects_one_byte_over_before_transport() {
    let content = vec![0xa5; PI_MAX_IMAGE_BYTES as usize];
    let content_sha256 = content_digest(&content);
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch_with_attachments(
            "workspace-1",
            "session-1",
            "turn-boundary",
            "run-boundary",
            "correlation-boundary",
            "",
            &[PiDispatchAttachment {
                attachment_id: "attachment-boundary".into(),
                stable_reference: "workspace-blob:attachment-boundary".into(),
                display_name: "boundary.png".into(),
                media_type: "image/png".into(),
                byte_length: content.len() as u64,
                content_sha256: content_sha256.clone(),
                snapshot_version: 1,
                content: content.clone(),
            }],
        )
        .unwrap();
    let dispatch = adapter
        .runner()
        .requests()
        .iter()
        .find(|request| request["runId"] == "run-boundary")
        .unwrap();
    let encoded = dispatch["payload"]["attachments"][0]["contentBase64"]
        .as_str()
        .unwrap();
    assert_eq!(encoded.len(), content.len().div_ceil(3) * 4);
    assert!(encoded.len() > 64 * 1024);
    assert!(serde_json::to_vec(dispatch).unwrap().len() < PI_MAX_LINE_BYTES);

    let oversized = vec![0xa5; PI_MAX_IMAGE_BYTES as usize + 1];
    let mut rejected = started_adapter(FakeRunner::default());
    create_session(&mut rejected);
    let requests_before = rejected.runner().requests().len();
    assert!(matches!(
        rejected.dispatch_with_attachments(
            "workspace-1",
            "session-1",
            "turn-over",
            "run-over",
            "correlation-over",
            "",
            &[PiDispatchAttachment {
                attachment_id: "attachment-over".into(),
                stable_reference: "workspace-blob:attachment-over".into(),
                display_name: "over.png".into(),
                media_type: "image/png".into(),
                byte_length: oversized.len() as u64,
                content_sha256: content_digest(&oversized),
                snapshot_version: 1,
                content: oversized,
            }],
        ),
        Err(PiAdapterError::Protocol(_))
    ));
    assert_eq!(rejected.runner().requests().len(), requests_before);
}

#[test]
fn ordinary_dispatch_strings_remain_bounded_to_64_kib_before_transport() {
    let mut accepted = started_adapter(FakeRunner::default());
    create_session(&mut accepted);
    accepted
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-input-boundary",
            "run-input-boundary",
            "correlation-input-boundary",
            &"x".repeat(64 * 1024),
        )
        .unwrap();

    let mut rejected = started_adapter(FakeRunner::default());
    create_session(&mut rejected);
    let requests_before = rejected.runner().requests().len();
    assert!(matches!(
        rejected.dispatch(
            "workspace-1",
            "session-1",
            "turn-input-over",
            "run-input-over",
            "correlation-input-over",
            &"x".repeat(64 * 1024 + 1),
        ),
        Err(PiAdapterError::Protocol(_))
    ));
    assert_eq!(rejected.runner().requests().len(), requests_before);
}

#[test]
fn action_intent_accepts_only_c4os_tools_and_completed_resolution_requires_gateway_receipt() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "tool",
        )
        .unwrap();
    let intent = adapter.take_events().pop().unwrap();
    assert_eq!(intent.category, "tool.action_intent");
    assert_eq!(intent.tool_call_id.as_deref(), Some("tool-call-1"));
    let receipt = gateway_receipt_for_pi(&intent, "pi-primary", NormalizedActionStatus::Succeeded);
    adapter
        .resolve_completed_tool(
            "pi-primary",
            "session-1",
            "run-1",
            "correlation-run-1",
            "tool-call-1",
            &receipt,
        )
        .unwrap();
    let requests = adapter.runner().requests();
    let resolution = requests
        .iter()
        .find(|request| request["operation"] == "tool.resolve")
        .unwrap();
    assert_eq!(resolution["payload"]["decision"], "completed");
    assert!(resolution["payload"].get("authorization").is_none());
    assert!(resolution["payload"].get("token").is_none());
}

#[test]
fn failed_gateway_effect_cannot_be_reported_as_a_completed_pi_tool() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "tool",
        )
        .unwrap();
    let intent = adapter.take_events().pop().unwrap();
    let receipt = gateway_receipt_for_pi(&intent, "pi-primary", NormalizedActionStatus::Failed);
    let before = adapter.runner().requests().len();

    assert!(
        adapter
            .resolve_completed_tool(
                "pi-primary",
                "session-1",
                "run-1",
                "correlation-run-1",
                "tool-call-1",
                &receipt,
            )
            .is_err()
    );
    assert_eq!(adapter.runner().requests().len(), before);
}

#[test]
fn denied_tool_resolution_is_non_authorizing_and_contains_no_capability() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "tool",
        )
        .unwrap();
    adapter
        .resolve_tool(
            "session-1",
            "run-1",
            "correlation-run-1",
            "tool-call-1",
            PiToolDecision::Denied {
                reason: "policy denied".into(),
            },
        )
        .unwrap();
    let request = adapter
        .runner()
        .requests()
        .iter()
        .find(|request| request["operation"] == "tool.resolve")
        .unwrap();
    assert_eq!(request["payload"]["decision"], "denied");
    assert!(request["payload"].get("authorization").is_none());
    assert!(request["payload"].get("token").is_none());
}

#[test]
fn stale_generation_run_or_sequence_events_are_rejected_without_state_mutation() {
    let runner = FakeRunner::default();
    let mut adapter = started_adapter(runner);
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "quiet",
        )
        .unwrap();
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION - 1,
        1,
        "run-1",
        "correlation-run-1",
        "content.delta",
        json!({ "delta": "stale generation" }),
    ));
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION,
        2,
        "other-run",
        "correlation-run-1",
        "content.delta",
        json!({ "delta": "stale run" }),
    ));
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION,
        3,
        "run-1",
        "correlation-run-1",
        "content.delta",
        json!({ "delta": "accepted" }),
    ));
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION,
        3,
        "run-1",
        "correlation-run-1",
        "content.delta",
        json!({ "delta": "duplicate sequence" }),
    ));
    let accepted = adapter.poll_events().unwrap();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].payload["delta"], "accepted");
    assert_eq!(adapter.stale_events_rejected(), 3);
}

#[test]
fn response_identity_duplicate_unknown_field_and_bounds_fail_closed() {
    for hostile in [
        HostileResponse::WrongCorrelation,
        HostileResponse::Duplicate,
        HostileResponse::UnknownField,
        HostileResponse::Oversized,
    ] {
        let runner = FakeRunner {
            hostile: Some(hostile),
            ..FakeRunner::default()
        };
        let error = adapter(runner).start().unwrap_err();
        assert!(matches!(error, PiAdapterError::Protocol(_)));
    }
}

#[test]
fn incoming_secret_fields_and_credential_shaped_diagnostics_fail_closed() {
    for hostile in [HostileResponse::SecretField, HostileResponse::SecretValue] {
        let runner = FakeRunner {
            hostile: Some(hostile),
            ..FakeRunner::default()
        };
        let error = adapter(runner).start().unwrap_err();
        assert!(matches!(error, PiAdapterError::Protocol(_)));
    }
}

#[test]
fn cancellation_is_idempotent_and_late_events_are_rejected() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "quiet",
        )
        .unwrap();
    assert!(
        !adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-other",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );
    assert!(
        adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-1",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );
    assert!(
        !adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-1",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION,
        9,
        "run-1",
        "correlation-run-1",
        "content.delta",
        json!({ "delta": "late" }),
    ));
    assert!(adapter.poll_events().unwrap().is_empty());
    assert_eq!(adapter.stale_events_rejected(), 1);
}

#[test]
fn rejected_cancellation_retains_the_exact_active_run_binding() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "quiet",
        )
        .unwrap();
    adapter.runner_mut().cancel_returns_false = true;
    assert!(
        !adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-1",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );

    adapter.runner_mut().cancel_returns_false = false;
    assert!(
        adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-1",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );
    assert!(
        !adapter
            .cancel(
                "workspace-1",
                "session-1",
                "turn-1",
                "run-1",
                "correlation-run-1",
            )
            .unwrap()
    );
}

#[test]
fn terminal_event_releases_session_for_a_new_correlated_run() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-1",
            "run-1",
            "correlation-run-1",
            "quiet",
        )
        .unwrap();
    adapter.runner_mut().poll_lines.push_back(event_line(
        GENERATION,
        1,
        "run-1",
        "correlation-run-1",
        "lifecycle.settled",
        json!({}),
    ));
    assert_eq!(adapter.poll_events().unwrap().len(), 1);
    adapter
        .dispatch(
            "workspace-1",
            "session-1",
            "turn-2",
            "run-2",
            "correlation-run-2",
            "quiet",
        )
        .unwrap();
}

#[test]
fn sampling_requires_the_exact_workspace_turn_run_and_correlation_binding() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter
        .start_sampling_with_credential_operation(
            "workspace-1",
            "session-1",
            "sampling-turn-1",
            "sampling-run-1",
            "sampling-correlation-1",
            &PiSamplingRequest {
                messages: vec![PiSamplingMessage {
                    role: "user".into(),
                    text: "Question".into(),
                }],
                system_prompt: None,
                max_tokens: 8,
                temperature: None,
            },
            Some(("pi-primary", "provider-openai")),
        )
        .unwrap();

    assert!(matches!(
        adapter.poll_sampling(
            "workspace-2",
            "session-1",
            "sampling-turn-1",
            "sampling-run-1",
            "sampling-correlation-1",
        ),
        Err(PiAdapterError::State(_))
    ));
    assert!(matches!(
        adapter.cancel_sampling(
            "workspace-1",
            "session-1",
            "sampling-turn-2",
            "sampling-run-1",
            "sampling-correlation-1",
        ),
        Err(PiAdapterError::State(_))
    ));
    assert_eq!(
        adapter
            .poll_sampling(
                "workspace-1",
                "session-1",
                "sampling-turn-1",
                "sampling-run-1",
                "sampling-correlation-1",
            )
            .unwrap(),
        PiSamplingPoll::Completed(c4os_lib::runtime::pi::PiSamplingResult {
            text: "sampled answer".into(),
            model: "gpt-4o-mini".into(),
            stop_reason: "endTurn".into(),
        })
    );
    adapter.close_session("workspace-1", "session-1").unwrap();
}

#[test]
fn ambiguous_sampling_start_ack_keeps_the_native_run_bound_until_shutdown() {
    let mut adapter = started_adapter(FakeRunner {
        mismatched_sampling_start: true,
        ..FakeRunner::default()
    });
    create_session(&mut adapter);
    assert!(matches!(
        adapter.start_sampling_with_credential_operation(
            "workspace-1",
            "session-1",
            "sampling-turn-1",
            "sampling-run-1",
            "sampling-correlation-1",
            &PiSamplingRequest {
                messages: vec![PiSamplingMessage {
                    role: "user".into(),
                    text: "Question".into(),
                }],
                system_prompt: None,
                max_tokens: 8,
                temperature: None,
            },
            Some(("pi-primary", "provider-openai")),
        ),
        Err(PiAdapterError::Protocol(_))
    ));
    assert!(matches!(
        adapter.close_session("workspace-1", "session-1"),
        Err(PiAdapterError::State(_))
    ));
    adapter.shutdown().unwrap();
    assert_eq!(adapter.runner().terminations, 1);
}

#[test]
fn shutdown_is_bounded_idempotent_and_terminates_supervised_runner() {
    let mut adapter = started_adapter(FakeRunner::default());
    create_session(&mut adapter);
    adapter.shutdown().unwrap();
    adapter.shutdown().unwrap();
    assert_eq!(adapter.state(), &PiAdapterState::Stopped);
    assert_eq!(adapter.runner().terminations, 1);
    assert!(matches!(
        adapter.create_session(
            "workspace-1",
            "new-session",
            PiModelRoute {
                provider: "openai".into(),
                model_id: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com/v1".into(),
            },
            &BTreeSet::new(),
        ),
        Err(PiAdapterError::State(_))
    ));
}

#[test]
fn failed_graceful_shutdown_still_terminates_the_supervised_runner() {
    let mut adapter = started_adapter(FakeRunner {
        fail_shutdown_exchange: true,
        ..FakeRunner::default()
    });
    create_session(&mut adapter);

    adapter.shutdown().unwrap();

    assert_eq!(adapter.state(), &PiAdapterState::Stopped);
    assert_eq!(adapter.runner().terminations, 1);
}

fn adapter(runner: FakeRunner) -> TestAdapter {
    PiAdapter::new(load_manifest(), runner, GENERATION).unwrap()
}

fn started_adapter(runner: FakeRunner) -> TestAdapter {
    let mut adapter = adapter(runner);
    adapter.start().unwrap();
    adapter
}

type TestAdapter = PiAdapter<FakeRunner>;

fn create_session(adapter: &mut TestAdapter) {
    adapter
        .create_session(
            "workspace-1",
            "session-1",
            PiModelRoute {
                provider: "openai".into(),
                model_id: "gpt-4o-mini".into(),
                base_url: "https://api.openai.com/v1".into(),
            },
            &BTreeSet::from(["c4os_propose_action".into()]),
        )
        .unwrap();
}

fn load_manifest() -> PiSidecarManifest {
    PiSidecarManifest::load(&sidecar_root()).unwrap()
}

fn sidecar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sidecars/pi")
}

#[derive(Debug, Clone, Copy)]
enum HostileResponse {
    WrongCorrelation,
    Duplicate,
    UnknownField,
    Oversized,
    SecretField,
    SecretValue,
}

#[derive(Default)]
struct FakeRunner {
    requests: Vec<Value>,
    poll_lines: VecDeque<String>,
    hostile: Option<HostileResponse>,
    unsafe_health: bool,
    next_sequence: u64,
    terminations: usize,
    fail_shutdown_exchange: bool,
    cancel_returns_false: bool,
    missing_model: bool,
    mismatched_model_preflight: bool,
    mismatched_sampling_start: bool,
}

impl FakeRunner {
    fn requests(&self) -> &[Value] {
        &self.requests
    }
}

impl PiSidecarRunner for FakeRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        assert!(request_line.ends_with('\n'));
        assert!(!request_line[..request_line.len() - 1].contains('\n'));
        let request: Value =
            serde_json::from_str(request_line).map_err(|error| error.to_string())?;
        self.requests.push(request.clone());
        let operation = request["operation"].as_str().unwrap();
        if operation == "shutdown" && self.fail_shutdown_exchange {
            return Err("graceful shutdown transport failed".into());
        }
        let payload = match operation {
            "health" => health_payload(self.unsafe_health),
            "version" => json!({
                "adapter": "PIAdapter",
                "adapterVersion": "0.1.0",
                "nativePackage": "@earendil-works/pi-coding-agent",
                "nativeVersion": "0.80.10",
                "protocol": "c4os.pi.ndjson.v1"
            }),
            "model.preflight" => json!({
                "available": !self.missing_model,
                "provider": if self.mismatched_model_preflight { "anthropic" } else {
                    request["payload"]["provider"].as_str().unwrap()
                },
                "modelId": request["payload"]["modelId"],
                "nativeVersion": "0.80.10"
            }),
            "session.create" => json!({
                "sessionId": request["sessionId"],
                "nativeSessionId": "native-session-1",
                "persistence": "c4os-authoritative"
            }),
            "session.close" => json!({ "closed": true }),
            "dispatch" => json!({ "accepted": true, "runId": request["runId"] }),
            "sampling.start" => json!({
                "accepted": true,
                "runId": if self.mismatched_sampling_start {
                    json!("wrong-run")
                } else {
                    request["runId"].clone()
                }
            }),
            "sampling.poll" => json!({
                "state": "completed",
                "result": {
                    "text": "sampled answer",
                    "model": "gpt-4o-mini",
                    "stopReason": "endTurn"
                }
            }),
            "sampling.cancel" => json!({
                "cancelled": true,
                "alreadyTerminal": false
            }),
            "tool.resolve" => json!({ "resolved": true, "executedBySidecar": false }),
            "cancel" => json!({
                "cancelled": !self.cancel_returns_false,
                "alreadyTerminal": self.cancel_returns_false
            }),
            "shutdown" => json!({ "stopped": true }),
            _ => json!({}),
        };
        let mut response = json!({
            "schemaVersion": 1,
            "kind": "response",
            "requestId": request["requestId"],
            "correlationId": request["correlationId"],
            "processGeneration": GENERATION,
            "status": "ok",
            "payload": payload
        });
        let mut lines = Vec::new();
        if operation == "dispatch" {
            let input = request["payload"]["input"].as_str().unwrap();
            if input == "hello" {
                self.next_sequence += 1;
                lines.push(event_from_request(
                    &request,
                    self.next_sequence,
                    "content.delta",
                    json!({ "delta": "hello" }),
                ));
            } else if input == "tool" {
                self.next_sequence += 1;
                let mut event: Value = serde_json::from_str(&event_from_request(
                    &request,
                    self.next_sequence,
                    "tool.action_intent",
                    json!({
                        "tool": "c4os_read_resource",
                        "arguments": { "target": "workspace:/README.md" },
                        "authority": "c4os-action-gateway-required"
                    }),
                ))
                .unwrap();
                event["toolCallId"] = Value::String("tool-call-1".into());
                lines.push(serde_json::to_string(&event).unwrap());
            }
        }
        if let Some(hostile) = self.hostile.take() {
            match hostile {
                HostileResponse::WrongCorrelation => {
                    response["correlationId"] = Value::String("wrong-correlation".into());
                }
                HostileResponse::Duplicate => {
                    lines.push(serde_json::to_string(&response).unwrap());
                }
                HostileResponse::UnknownField => {
                    response["ambientAuthority"] = Value::Bool(true);
                }
                HostileResponse::Oversized => {
                    response["payload"]["content"] = Value::String("x".repeat(PI_MAX_LINE_BYTES));
                }
                HostileResponse::SecretField => {
                    response["payload"]["apiKey"] = Value::String("raw".into());
                }
                HostileResponse::SecretValue => {
                    response["payload"]["message"] =
                        Value::String("Bearer raw-credential-material".into());
                }
            }
        }
        lines.push(serde_json::to_string(&response).unwrap());
        Ok(lines)
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        Ok(self.poll_lines.drain(..).collect())
    }

    fn terminate(&mut self) -> Result<(), String> {
        self.terminations += 1;
        Ok(())
    }
}

fn health_payload(unsafe_health: bool) -> Value {
    let unsupported = if unsafe_health {
        "supported"
    } else {
        "unsupported"
    };
    let mut capabilities = BTreeMap::new();
    capabilities.insert("streaming", json!({ "state": "supported" }));
    capabilities.insert("cancellation", json!({ "state": "supported" }));
    capabilities.insert("tools", json!({ "state": "supported" }));
    capabilities.insert("nativePersistence", json!({ "state": unsupported }));
    capabilities.insert("nativeExtensions", json!({ "state": unsupported }));
    capabilities.insert("nativeTools", json!({ "state": unsupported }));
    capabilities.insert(
        "crashResume",
        json!({ "state": "degraded", "reason": "C4OS replay required" }),
    );
    capabilities.insert(
        "providerAuthentication",
        json!({ "state": "degraded", "reason": "test channel absent" }),
    );
    capabilities.insert("rpcTransport", json!({ "state": "unsupported" }));
    json!({
        "status": "ready",
        "runtime": "pi",
        "transport": "c4os-node-sdk-sidecar",
        "protocol": "c4os.pi.ndjson.v1",
        "processGeneration": GENERATION,
        "sessions": 0,
        "staleEventsRejected": 0,
        "capabilities": capabilities
    })
}

fn event_from_request(request: &Value, sequence: u64, category: &str, payload: Value) -> String {
    serde_json::to_string(&json!({
        "schemaVersion": 1,
        "kind": "event",
        "eventId": format!("pi-event-{sequence}"),
        "correlationId": request["correlationId"],
        "processGeneration": GENERATION,
        "sequence": sequence,
        "runtime": "pi",
        "workspaceId": request["workspaceId"],
        "sessionId": request["sessionId"],
        "turnId": request["turnId"],
        "runId": request["runId"],
        "category": category,
        "nativeType": "fake.native.event",
        "payload": payload
    }))
    .unwrap()
}

fn event_line(
    process_generation: u64,
    sequence: u64,
    run_id: &str,
    correlation_id: &str,
    category: &str,
    payload: Value,
) -> String {
    serde_json::to_string(&json!({
        "schemaVersion": 1,
        "kind": "event",
        "eventId": format!("pi-event-{sequence}"),
        "correlationId": correlation_id,
        "processGeneration": process_generation,
        "sequence": sequence,
        "runtime": "pi",
        "workspaceId": "workspace-1",
        "sessionId": "session-1",
        "turnId": "turn-1",
        "runId": run_id,
        "category": category,
        "nativeType": "fake.native.event",
        "payload": payload
    }))
    .unwrap()
}

trait AdapterTestAccess {
    fn runner(&self) -> &FakeRunner;
    fn runner_mut(&mut self) -> &mut FakeRunner;
}

impl AdapterTestAccess for TestAdapter {
    fn runner(&self) -> &FakeRunner {
        // Test-only layout access is intentionally provided by production getters below.
        self.runner_ref()
    }

    fn runner_mut(&mut self) -> &mut FakeRunner {
        self.runner_mut_ref()
    }
}
