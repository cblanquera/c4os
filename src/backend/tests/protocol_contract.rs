#[allow(dead_code)]
#[path = "../src/protocol.rs"]
mod protocol;

use c4os_lib::extension::{
    ExtensionLifecycle, ExtensionPackageKind, ExtensionServiceSnapshot, ExtensionSourceKind,
    ExtensionTrustState, MarketplaceSnapshot, PluginAppSnapshot, PluginHookSnapshot,
    PluginMcpServerSnapshot, PluginSettingSnapshot, PluginSnapshot, SkillQualifiedIdentity,
    SkillSnapshot,
    service::{
        ExtensionHookReviewInput, ExtensionKeyRevocationInput, ExtensionPackageInput,
        ExtensionPublisherLinkInput, ExtensionRevocationInput, ExtensionSkillAvailabilityInput,
        ExtensionSkillInput, MarketplaceSourceInput, SkillInstructionsSnapshot,
    },
};
use c4os_lib::mcp::{
    McpCapabilitySnapshot, McpDefinitionSource, McpEnvironmentBinding, McpEnvironmentSource,
    McpHeaderBinding, McpHeaderSource, McpInvocationSnapshot, McpInvocationStatus, McpLifecycle,
    McpPendingTrustApproval, McpResourceReadInput, McpResourceSnapshot, McpScope,
    McpSecretReference, McpServerDefinitionInput, McpServerMutationInput, McpServerRevocationInput,
    McpServerSnapshot, McpServiceSnapshot, McpToolCallInput, McpToolSnapshot,
    McpTransportDefinition, McpTransportKind, McpTrustApprovalAnswer, McpTrustApprovalInput,
    McpTrustApprovalState, McpTrustRequestInput, McpTrustRequestStatus, McpTrustResponse,
    McpTrustState, McpWorkingDirectory,
};
use c4os_lib::update::{
    DiagnosticCategory, DiagnosticSeverity, DiagnosticsExportInput, DiagnosticsExportSnapshot,
    DiagnosticsSnapshot, LocalUpdateStageInput, UpdateActionInput, UpdateCandidateSnapshot,
    UpdateChannel, UpdateComponentSnapshot, UpdateCoordinatorSnapshot, UpdateDiagnosticRecord,
    UpdateLifecycleState, UpdateOperationState, UpdatePendingOperation, UpdateRecoveryAction,
    UpdateRecoveryInput, UpdateRecoveryNotice, UpdateRevocationInput,
};
use protocol::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use ts_rs::{Config, TS};

fn request(generation: u64, command: Command) -> RequestEnvelope {
    RequestEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: RequestId::new("request-1").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        expected_generation: StateGeneration(generation),
        request: CommandRequest {
            command_id: CommandId::new("command-1").unwrap(),
            command,
        },
    }
}

fn submit_turn() -> Command {
    Command::SubmitTurn {
        session_id: SessionId::new("session-1").unwrap(),
        text: Some("hello".to_owned()),
        attachment_ids: Vec::new(),
    }
}

fn run_scope(process_generation: u64) -> RunScope {
    RunScope {
        workspace_id: WorkspaceId::new("workspace-1").unwrap(),
        session_id: SessionId::new("session-1").unwrap(),
        turn_id: TurnId::new("turn-1").unwrap(),
        attempt_id: AttemptId::new("attempt-1").unwrap(),
        runtime_id: RuntimeId::new("pi@0.80.10").unwrap(),
        environment_id: EnvironmentId::new("local").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        process_generation: ProcessGeneration::new(process_generation).unwrap(),
    }
}

fn file_artifact_workspace(state: ArtifactFileStateSnapshot) -> ArtifactWorkspaceSnapshot {
    let digest = format!("sha256:{}", "a".repeat(64));
    ArtifactWorkspaceSnapshot {
        protocol_version: PROTOCOL_VERSION,
        generation: StateGeneration(1),
        authority: "rust-core".into(),
        workspace_id: Some(WorkspaceId::new("workspace-1").unwrap()),
        active_project_id: Some(ProjectId::new("project-1").unwrap()),
        active_session_id: Some(SessionId::new("session-1").unwrap()),
        focused_artifact_id: None,
        artifacts: vec![ArtifactSnapshot {
            artifact_id: ArtifactId::new("artifact-1").unwrap(),
            project_id: ProjectId::new("project-1").unwrap(),
            session_id: SessionId::new("session-1").unwrap(),
            provider_type: "file".into(),
            provider_version: 1,
            state_schema_version: 1,
            record_revision: 2,
            title: "notes.txt".into(),
            focus_supported: true,
            status: ArtifactShellStatusSnapshot {
                kind: "ready".into(),
                message: None,
            },
            pending_approval_id: None,
            source_label: "Direct operation".into(),
            resource_version: ArtifactResourceVersionSnapshot {
                sequence: 1,
                sha256: digest,
                observed_at_ms: 10,
            },
            history: Vec::new(),
            provider_state: ArtifactProviderStateSnapshot::File(ArtifactFileSnapshot {
                breadcrumbs: vec![ArtifactBreadcrumbSnapshot {
                    id: "notes.txt".into(),
                    label: "notes.txt".into(),
                    is_current: true,
                }],
                language_label: Some("Plain text".into()),
                state,
                version_label: Some("Version 1".into()),
            }),
        }],
    }
}

fn terminal_artifact_workspace(phase: &str) -> ArtifactWorkspaceSnapshot {
    let digest = format!("sha256:{}", "b".repeat(64));
    let running = matches!(phase, "running" | "stdinReady" | "stopping");
    let prompt_ready = phase == "completed";
    let approval_waiting = phase == "approvalWaiting";
    ArtifactWorkspaceSnapshot {
        protocol_version: PROTOCOL_VERSION,
        generation: StateGeneration(2),
        authority: "rust-core".into(),
        workspace_id: Some(WorkspaceId::new("workspace-1").unwrap()),
        active_project_id: Some(ProjectId::new("project-1").unwrap()),
        active_session_id: Some(SessionId::new("session-1").unwrap()),
        focused_artifact_id: None,
        artifacts: vec![ArtifactSnapshot {
            artifact_id: ArtifactId::new("artifact-terminal-1").unwrap(),
            project_id: ProjectId::new("project-1").unwrap(),
            session_id: SessionId::new("session-1").unwrap(),
            provider_type: "terminal".into(),
            provider_version: 1,
            state_schema_version: 1,
            record_revision: 2,
            title: "$ printf hello".into(),
            focus_supported: true,
            status: ArtifactShellStatusSnapshot {
                kind: "ready".into(),
                message: approval_waiting.then(|| "Approval required".into()),
            },
            pending_approval_id: approval_waiting.then(|| "prompt-terminal-1".into()),
            source_label: "Direct operation".into(),
            resource_version: ArtifactResourceVersionSnapshot {
                sequence: 2,
                sha256: digest.clone(),
                observed_at_ms: 20,
            },
            history: Vec::new(),
            provider_state: ArtifactProviderStateSnapshot::Terminal(ArtifactTerminalSnapshot {
                terminal_session_id: "terminal-session-1".into(),
                command_id: "command-1".into(),
                command_sequence: 1,
                command: "printf hello".into(),
                working_directory_display: "/project".into(),
                shell_path: "/bin/zsh".into(),
                environment_id: "local-project".into(),
                environment_generation: 1,
                process_generation: 1,
                shell_process_id: (running || prompt_ready).then_some(42),
                foreground_process_group_id: running.then_some(43),
                columns: 100,
                rows: 30,
                output_base64: "aGVsbG8=".into(),
                output_text: "hello".into(),
                output_sequence: 2,
                retained_bytes: 5,
                dropped_bytes: 0,
                phase: phase.into(),
                exit_code: prompt_ready.then_some(0),
                status_message: None,
                stdin_ready: phase == "stdinReady",
                stop_available: matches!(phase, "running" | "stdinReady"),
                prompt_ready,
                shell_replaced: false,
            }),
        }],
    }
}

#[test]
fn rejects_unknown_protocol_versions_before_command_handling() {
    let mut envelope = request(7, submit_turn());
    envelope.protocol_version = PROTOCOL_VERSION + 1;

    let error = envelope.validate(StateGeneration(7)).unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::UnknownProtocolVersion);
}

#[test]
fn rejects_stale_and_future_renderer_generations() {
    let stale = request(6, submit_turn())
        .validate(StateGeneration(7))
        .unwrap_err();
    let future = request(8, submit_turn())
        .validate(StateGeneration(7))
        .unwrap_err();

    assert_eq!(stale.code, ProtocolErrorCode::StaleGeneration);
    assert_eq!(future.code, ProtocolErrorCode::FutureGeneration);
}

#[test]
fn snapshot_identity_validation_does_not_invent_a_response_generation() {
    let request = SnapshotRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: RequestId::new("request-snapshot").unwrap(),
        correlation_id: CorrelationId::new("correlation-snapshot").unwrap(),
        expected_generation: StateGeneration(99),
    };

    validate_snapshot_request(&request).unwrap();
    let error = snapshot_envelope(request, StateGeneration(0), ()).unwrap_err();
    assert_eq!(error.code, ProtocolErrorCode::FutureGeneration);
}

#[test]
fn rejects_unbounded_or_empty_turn_payloads() {
    let empty = request(
        1,
        Command::SubmitTurn {
            session_id: SessionId::new("session-1").unwrap(),
            text: Some("  ".to_owned()),
            attachment_ids: Vec::new(),
        },
    )
    .validate(StateGeneration(1))
    .unwrap_err();
    let oversized = request(
        1,
        Command::SubmitTurn {
            session_id: SessionId::new("session-1").unwrap(),
            text: Some("x".repeat(MAX_TEXT_BYTES + 1)),
            attachment_ids: Vec::new(),
        },
    )
    .validate(StateGeneration(1))
    .unwrap_err();

    assert_eq!(empty.code, ProtocolErrorCode::InvalidPayload);
    assert_eq!(oversized.code, ProtocolErrorCode::PayloadTooLarge);
}

#[test]
fn rejects_invalid_ids_even_when_deserialization_bypasses_constructor() {
    let json = r#"{
        "protocolVersion": 1,
        "requestId": "request with spaces",
        "correlationId": "correlation-1",
        "expectedGeneration": 2,
        "request": {
            "commandId": "command-1",
            "command": {
                "type": "focusArtifact",
                "payload": {"artifactId": "artifact-1"}
            }
        }
    }"#;
    let envelope: RequestEnvelope = serde_json::from_str(json).unwrap();

    let error = envelope.validate(StateGeneration(2)).unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::InvalidIdentifier);
}

#[test]
fn rejects_stale_worker_process_events() {
    let event = EventEnvelope {
        protocol_version: PROTOCOL_VERSION,
        event_id: RequestId::new("event-1").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        generation: StateGeneration(9),
        run_scope: Some(run_scope(2)),
        event: CoreEvent::RunChanged {
            sequence: 4,
            phase: RunPhase::Running,
        },
    };

    let error = event
        .validate(StateGeneration(9), Some(ProcessGeneration::new(3).unwrap()))
        .unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::StaleGeneration);
}

#[test]
fn rejects_run_events_without_complete_scope() {
    let event = EventEnvelope {
        protocol_version: PROTOCOL_VERSION,
        event_id: RequestId::new("event-1").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        generation: StateGeneration(9),
        run_scope: None,
        event: CoreEvent::RunChanged {
            sequence: 4,
            phase: RunPhase::Running,
        },
    };

    let error = event.validate(StateGeneration(9), None).unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::InvalidPayload);
}

#[test]
fn rejects_mismatched_event_correlation() {
    let mut scope = run_scope(3);
    scope.correlation_id = CorrelationId::new("correlation-2").unwrap();
    let event = EventEnvelope {
        protocol_version: PROTOCOL_VERSION,
        event_id: RequestId::new("event-1").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        generation: StateGeneration(9),
        run_scope: Some(scope),
        event: CoreEvent::RunChanged {
            sequence: 4,
            phase: RunPhase::Running,
        },
    };

    let error = event
        .validate(StateGeneration(9), Some(ProcessGeneration::new(3).unwrap()))
        .unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::CorrelationMismatch);
}

#[test]
fn accepts_a_current_bounded_request_and_matching_response() {
    let request = request(11, submit_turn());
    request.validate(StateGeneration(11)).unwrap();

    let response = ResponseEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id.clone(),
        correlation_id: request.correlation_id.clone(),
        generation: StateGeneration(12),
        result: ResponseResult::Ok(ResponsePayload::Acknowledged),
    };

    response
        .validate_for(&request, StateGeneration(11))
        .unwrap();
}

#[test]
fn foundation_snapshot_command_validates_before_replying() {
    let snapshot = foundation_snapshot(SnapshotRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: RequestId::new("request-1").unwrap(),
        correlation_id: CorrelationId::new("correlation-1").unwrap(),
        expected_generation: StateGeneration(0),
    })
    .unwrap();

    assert_eq!(snapshot.protocol_version, PROTOCOL_VERSION);
    assert_eq!(snapshot.generation, StateGeneration(0));
    assert_eq!(snapshot.payload.authority, "rust-core");

    let error = foundation_snapshot(SnapshotRequest {
        protocol_version: PROTOCOL_VERSION + 1,
        request_id: RequestId::new("request-2").unwrap(),
        correlation_id: CorrelationId::new("correlation-2").unwrap(),
        expected_generation: StateGeneration(0),
    })
    .unwrap_err();

    assert_eq!(error.code, ProtocolErrorCode::UnknownProtocolVersion);
}

#[test]
fn workspace_start_snapshot_is_bounded_and_allows_snapshot_catch_up() {
    let request = SnapshotRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: RequestId::new("request-start").unwrap(),
        correlation_id: CorrelationId::new("correlation-start").unwrap(),
        expected_generation: StateGeneration(2),
    };
    let payload = WorkspaceStartSnapshot {
        protocol_version: PROTOCOL_VERSION,
        generation: StateGeneration(4),
        authority: "rust-core".into(),
        active_recovery_notice: None,
        recents: vec![WorkspaceRecentSnapshot {
            workspace_id: WorkspaceId::new("workspace-1").unwrap(),
            display_name: "C4OS Workspace".into(),
            last_opened_at: 1_721_312_000,
            is_missing: false,
        }],
    };

    let response = workspace_start_snapshot(request, payload).unwrap();
    assert_eq!(response.generation, StateGeneration(4));
    assert_eq!(response.payload.recents.len(), 1);

    let oversized = WorkspaceStartSnapshot {
        protocol_version: PROTOCOL_VERSION,
        generation: StateGeneration(4),
        authority: "rust-core".into(),
        active_recovery_notice: None,
        recents: (0..=MAX_RECENT_WORKSPACES)
            .map(|index| WorkspaceRecentSnapshot {
                workspace_id: WorkspaceId::new(format!("workspace-{index}")).unwrap(),
                display_name: format!("Workspace {index}"),
                last_opened_at: 1_721_312_000,
                is_missing: false,
            })
            .collect(),
    };
    let error = workspace_start_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("request-oversized").unwrap(),
            correlation_id: CorrelationId::new("correlation-oversized").unwrap(),
            expected_generation: StateGeneration(4),
        },
        oversized,
    )
    .unwrap_err();
    assert_eq!(error.code, ProtocolErrorCode::InvalidPayload);
}

#[test]
fn redaction_markers_never_carry_original_values() {
    let value = SafeDetailValue::Redacted(RedactionMarker::new(
        "provider.api_key",
        RedactionReason::Credential,
    ));
    let serialized = serde_json::to_string(&value).unwrap();

    assert!(serialized.contains("provider.api_key"));
    assert!(serialized.contains("credential"));
    assert!(!serialized.contains("secret"));
}

#[test]
fn wire_json_uses_camel_case_fields_and_discriminants() {
    let serialized = serde_json::to_value(request(5, submit_turn())).unwrap();

    assert_eq!(serialized["protocolVersion"], PROTOCOL_VERSION);
    assert_eq!(serialized["expectedGeneration"], 5);
    assert_eq!(serialized["request"]["commandId"], "command-1");
    assert_eq!(serialized["request"]["command"]["type"], "submitTurn");
    assert_eq!(
        serialized["request"]["command"]["payload"]["sessionId"],
        "session-1"
    );
    assert!(serialized.get("protocol_version").is_none());
}

#[test]
fn legacy_extension_hook_snapshot_defaults_the_additive_argument_contract() {
    let hook: PluginHookSnapshot = serde_json::from_value(serde_json::json!({
        "hookId": "before-turn",
        "name": "hooks/before-turn.mjs",
        "event": "before-turn",
        "reviewDigest": format!("sha256:{}", "a".repeat(64)),
        "reviewed": true,
        "status": "ready",
        "grants": ["context.annotation"],
        "lastResult": null
    }))
    .expect("legacy hook snapshot");
    assert!(hook.arguments.is_empty());
    assert_eq!(
        serde_json::to_value(hook).unwrap()["arguments"],
        serde_json::json!([])
    );
}

#[test]
fn artifact_file_states_are_tagged_and_bounded_at_the_protocol_boundary() {
    let snapshot = file_artifact_workspace(ArtifactFileStateSnapshot::Dirty {
        content: "one".into(),
        draft: "two".into(),
    });
    let envelope = artifact_workspace_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("artifact-request-1").unwrap(),
            correlation_id: CorrelationId::new("artifact-correlation-1").unwrap(),
            expected_generation: StateGeneration(0),
        },
        snapshot,
    )
    .unwrap();
    let serialized = serde_json::to_value(envelope).unwrap();
    assert_eq!(
        serialized["payload"]["artifacts"][0]["providerState"]["value"]["state"]["phase"],
        "dirty"
    );

    let oversized = file_artifact_workspace(ArtifactFileStateSnapshot::Dirty {
        content: "one".into(),
        draft: "x".repeat(MAX_TEXT_BYTES + 1),
    });
    let error = artifact_workspace_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("artifact-request-2").unwrap(),
            correlation_id: CorrelationId::new("artifact-correlation-2").unwrap(),
            expected_generation: StateGeneration(0),
        },
        oversized,
    )
    .unwrap_err();
    assert_eq!(error.code, ProtocolErrorCode::PayloadTooLarge);
}

#[test]
fn artifact_unknown_and_approval_states_fail_closed() {
    let mut unknown = file_artifact_workspace(ArtifactFileStateSnapshot::Read {
        content: "one".into(),
    });
    unknown.artifacts[0].provider_state = ArtifactProviderStateSnapshot::Unknown;
    unknown.artifacts[0].focus_supported = false;
    unknown.artifacts[0].status = ArtifactShellStatusSnapshot {
        kind: "degraded".into(),
        message: Some("Unsupported".into()),
    };
    let error = artifact_workspace_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("artifact-request-unknown").unwrap(),
            correlation_id: CorrelationId::new("artifact-correlation-unknown").unwrap(),
            expected_generation: StateGeneration(0),
        },
        unknown,
    )
    .unwrap_err();
    assert_eq!(error.code, ProtocolErrorCode::InvalidPayload);

    let mut approval = file_artifact_workspace(ArtifactFileStateSnapshot::Approval {
        content: "one".into(),
        proposed_content: "two".into(),
        proposal_diff: Some("-one\n+two".into()),
        approval_summary: "Approve exact write".into(),
    });
    approval.artifacts[0].pending_approval_id = None;
    let error = artifact_workspace_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("artifact-request-approval").unwrap(),
            correlation_id: CorrelationId::new("artifact-correlation-approval").unwrap(),
            expected_generation: StateGeneration(0),
        },
        approval,
    )
    .unwrap_err();
    assert_eq!(error.code, ProtocolErrorCode::InvalidPayload);
}

#[test]
fn artifact_conflict_resolution_requires_exact_nonzero_revision() {
    let invalid = ArtifactFileConflictInput {
        artifact_id: ArtifactId::new("artifact-1").unwrap(),
        base_record_revision: 0,
        resolution: ArtifactFileConflictResolution::KeepDraft,
    };
    assert_eq!(
        invalid.validate().unwrap_err().code,
        ProtocolErrorCode::InvalidGeneration
    );
}

#[test]
fn terminal_snapshot_requires_consistent_process_and_approval_state() {
    for phase in [
        "approvalWaiting",
        "running",
        "stdinReady",
        "stopping",
        "completed",
    ] {
        artifact_workspace_snapshot(
            SnapshotRequest {
                protocol_version: PROTOCOL_VERSION,
                request_id: RequestId::new(format!("request-{phase}")).unwrap(),
                correlation_id: CorrelationId::new(format!("correlation-{phase}")).unwrap(),
                expected_generation: StateGeneration(1),
            },
            terminal_artifact_workspace(phase),
        )
        .unwrap();
    }

    let mut pending_live_control = terminal_artifact_workspace("stdinReady");
    pending_live_control.artifacts[0].pending_approval_id = Some("prompt-terminal-control".into());
    pending_live_control.artifacts[0].status.message =
        Some("Approval required for Terminal input".into());
    let ArtifactProviderStateSnapshot::Terminal(terminal) =
        &mut pending_live_control.artifacts[0].provider_state
    else {
        unreachable!();
    };
    terminal.stop_available = false;
    artifact_workspace_snapshot(
        SnapshotRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: RequestId::new("request-pending-live-terminal").unwrap(),
            correlation_id: CorrelationId::new("correlation-pending-live-terminal").unwrap(),
            expected_generation: StateGeneration(1),
        },
        pending_live_control,
    )
    .unwrap();

    let mut invalid = terminal_artifact_workspace("running");
    let ArtifactProviderStateSnapshot::Terminal(terminal) =
        &mut invalid.artifacts[0].provider_state
    else {
        unreachable!();
    };
    terminal.shell_process_id = None;
    assert_eq!(
        artifact_workspace_snapshot(
            SnapshotRequest {
                protocol_version: PROTOCOL_VERSION,
                request_id: RequestId::new("request-invalid-terminal").unwrap(),
                correlation_id: CorrelationId::new("correlation-invalid-terminal").unwrap(),
                expected_generation: StateGeneration(1),
            },
            invalid,
        )
        .unwrap_err()
        .code,
        ProtocolErrorCode::InvalidPayload
    );
}

#[test]
fn terminal_operation_inputs_bind_process_generation_and_reject_controls() {
    let invalid_command = ArtifactTerminalRunInput {
        command: "echo\u{1b}[Dhidden".into(),
        columns: 100,
        rows: 30,
    };
    assert_eq!(
        invalid_command.validate().unwrap_err().code,
        ProtocolErrorCode::InvalidPayload
    );

    let stale_stdin = ArtifactTerminalStdinInput {
        artifact_id: ArtifactId::new("artifact-terminal-1").unwrap(),
        base_record_revision: 2,
        process_generation: 0,
        text: "answer".into(),
    };
    assert_eq!(
        stale_stdin.validate().unwrap_err().code,
        ProtocolErrorCode::InvalidGeneration
    );
}

#[test]
fn mcp_definition_rejects_renderer_authored_trust() {
    let forged = serde_json::json!({
        "expectedGeneration": 1,
        "serverId": "forged.trust",
        "displayName": "Forged trust",
        "scope": { "kind": "application" },
        "transport": {
            "kind": "stdio",
            "command": "/usr/bin/false",
            "arguments": [],
            "environment": [],
            "workingDirectory": { "kind": "c4osHome" },
            "executableSha256": format!("sha256:{}", "0".repeat(64)),
        },
        "trusted": true,
        "timeoutMs": 30_000,
        "maxOutputBytes": 1_048_576,
    });

    assert!(serde_json::from_value::<McpServerDefinitionInput>(forged).is_err());
}

#[test]
fn export_bindings() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/generated");
    let config = Config::default()
        .with_out_dir(&output)
        .with_large_int("number");

    export_protocol_types(&config);
    let first = read_generated_tree(&output);
    export_protocol_types(&config);
    let second = read_generated_tree(&output);

    assert!(!first.is_empty(), "protocol export produced no bindings");
    assert_eq!(first, second, "protocol bindings are not deterministic");
}

fn export_protocol_types(config: &Config) {
    SnapshotRequest::export_all(config).unwrap();
    RequestEnvelope::export_all(config).unwrap();
    ResponseEnvelope::export_all(config).unwrap();
    EventEnvelope::export_all(config).unwrap();
    ProtocolEnvelope::<FoundationSnapshot>::export_all(config).unwrap();
    ProtocolEnvelope::<WorkspaceStartSnapshot>::export_all(config).unwrap();
    ProtocolEnvelope::<ConversationSnapshot>::export_all(config).unwrap();
    ProtocolEnvelope::<ConversationAttachmentPreviewSnapshot>::export_all(config).unwrap();
    ConversationAttachmentPreviewInput::export_all(config).unwrap();
    ConversationSubmitInput::export_all(config).unwrap();
    ConversationDraftInput::export_all(config).unwrap();
    ConversationRetryInput::export_all(config).unwrap();
    ConversationBranchInput::export_all(config).unwrap();
    ConversationBranchApprovalInput::export_all(config).unwrap();
    ArtifactWorkspaceSnapshot::export_all(config).unwrap();
    ArtifactOpenInput::export_all(config).unwrap();
    ArtifactMutationInput::export_all(config).unwrap();
    ArtifactReplyInput::export_all(config).unwrap();
    ArtifactContextExpandInput::export_all(config).unwrap();
    ArtifactFileDraftInput::export_all(config).unwrap();
    ArtifactFileConflictInput::export_all(config).unwrap();
    ArtifactFolderNavigateInput::export_all(config).unwrap();
    ArtifactFolderSelectInput::export_all(config).unwrap();
    ArtifactApprovalInput::export_all(config).unwrap();
    ArtifactTerminalRunInput::export_all(config).unwrap();
    ArtifactTerminalStdinInput::export_all(config).unwrap();
    ArtifactTerminalResizeInput::export_all(config).unwrap();
    ArtifactTerminalOutputAckInput::export_all(config).unwrap();
    ArtifactTerminalOperationInput::export_all(config).unwrap();
    ArtifactBrowserOpenInput::export_all(config).unwrap();
    ArtifactBrowserNavigateInput::export_all(config).unwrap();
    ArtifactBrowserViewportInput::export_all(config).unwrap();
    ArtifactBrowserIdentityInput::export_all(config).unwrap();
    ExtensionServiceSnapshot::export_all(config).unwrap();
    MarketplaceSnapshot::export_all(config).unwrap();
    PluginSnapshot::export_all(config).unwrap();
    PluginHookSnapshot::export_all(config).unwrap();
    PluginSettingSnapshot::export_all(config).unwrap();
    PluginAppSnapshot::export_all(config).unwrap();
    PluginMcpServerSnapshot::export_all(config).unwrap();
    SkillSnapshot::export_all(config).unwrap();
    SkillQualifiedIdentity::export_all(config).unwrap();
    ExtensionLifecycle::export_all(config).unwrap();
    ExtensionTrustState::export_all(config).unwrap();
    ExtensionSourceKind::export_all(config).unwrap();
    ExtensionPackageKind::export_all(config).unwrap();
    MarketplaceSourceInput::export_all(config).unwrap();
    ExtensionPackageInput::export_all(config).unwrap();
    ExtensionSkillInput::export_all(config).unwrap();
    ExtensionSkillAvailabilityInput::export_all(config).unwrap();
    ExtensionHookReviewInput::export_all(config).unwrap();
    ExtensionRevocationInput::export_all(config).unwrap();
    ExtensionKeyRevocationInput::export_all(config).unwrap();
    ExtensionPublisherLinkInput::export_all(config).unwrap();
    SkillInstructionsSnapshot::export_all(config).unwrap();
    McpServiceSnapshot::export_all(config).unwrap();
    McpServerSnapshot::export_all(config).unwrap();
    McpServerDefinitionInput::export_all(config).unwrap();
    McpTrustRequestInput::export_all(config).unwrap();
    McpTrustApprovalInput::export_all(config).unwrap();
    McpTrustApprovalAnswer::export_all(config).unwrap();
    McpTrustApprovalState::export_all(config).unwrap();
    McpPendingTrustApproval::export_all(config).unwrap();
    McpTrustRequestStatus::export_all(config).unwrap();
    McpTrustResponse::export_all(config).unwrap();
    McpServerMutationInput::export_all(config).unwrap();
    McpServerRevocationInput::export_all(config).unwrap();
    McpToolCallInput::export_all(config).unwrap();
    McpResourceReadInput::export_all(config).unwrap();
    McpInvocationSnapshot::export_all(config).unwrap();
    McpCapabilitySnapshot::export_all(config).unwrap();
    McpToolSnapshot::export_all(config).unwrap();
    McpResourceSnapshot::export_all(config).unwrap();
    McpTransportDefinition::export_all(config).unwrap();
    McpTransportKind::export_all(config).unwrap();
    McpDefinitionSource::export_all(config).unwrap();
    McpScope::export_all(config).unwrap();
    McpWorkingDirectory::export_all(config).unwrap();
    McpSecretReference::export_all(config).unwrap();
    McpEnvironmentBinding::export_all(config).unwrap();
    McpEnvironmentSource::export_all(config).unwrap();
    McpHeaderBinding::export_all(config).unwrap();
    McpHeaderSource::export_all(config).unwrap();
    McpTrustState::export_all(config).unwrap();
    McpLifecycle::export_all(config).unwrap();
    McpInvocationStatus::export_all(config).unwrap();
    UpdateCoordinatorSnapshot::export_all(config).unwrap();
    UpdateComponentSnapshot::export_all(config).unwrap();
    UpdateCandidateSnapshot::export_all(config).unwrap();
    UpdatePendingOperation::export_all(config).unwrap();
    UpdateRecoveryNotice::export_all(config).unwrap();
    UpdateDiagnosticRecord::export_all(config).unwrap();
    DiagnosticsSnapshot::export_all(config).unwrap();
    DiagnosticsExportSnapshot::export_all(config).unwrap();
    LocalUpdateStageInput::export_all(config).unwrap();
    UpdateActionInput::export_all(config).unwrap();
    UpdateRevocationInput::export_all(config).unwrap();
    UpdateRecoveryInput::export_all(config).unwrap();
    DiagnosticsExportInput::export_all(config).unwrap();
    UpdateChannel::export_all(config).unwrap();
    UpdateLifecycleState::export_all(config).unwrap();
    UpdateOperationState::export_all(config).unwrap();
    UpdateRecoveryAction::export_all(config).unwrap();
    DiagnosticSeverity::export_all(config).unwrap();
    DiagnosticCategory::export_all(config).unwrap();
}

fn read_generated_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.extension().is_some_and(|extension| extension == "ts") {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}
