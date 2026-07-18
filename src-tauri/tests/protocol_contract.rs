#[allow(dead_code)]
#[path = "../src/protocol.rs"]
mod protocol;

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
fn export_bindings() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/generated");
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
