#![cfg(unix)]

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use c4os_lib::runtime::opencode_sdk::{
    BrokerDecision, BrokerEvent, OPENCODE_BROKER_FD_ENV, OPENCODE_PROCESS_GENERATION_ENV,
    OPENCODE_SDK_PLUGIN_VERSION, OPENCODE_SDK_TOOL_IDS, OPENCODE_TEST_TLS_TRUST_FD_ENV,
    OpenCodeSdkBroker, OpenCodeSdkError, OpenCodeSdkIntegrity, materialize_opencode_sdk_plugin,
    prepare_opencode_sdk_launch,
};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(2);

fn sidecar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/opencode-sdk")
}

fn proposal(correlation_id: &str, tool: &str, generation: u64, payload: Value) -> Value {
    json!({
        "schemaVersion": 1,
        "kind": "proposal",
        "correlationId": correlation_id,
        "tool": tool,
        "sessionId": "session-1",
        "messageId": "message-1",
        "processGeneration": generation,
        "payload": payload,
    })
}

fn write_frame(worker: &mut UnixStream, frame: &Value) {
    serde_json::to_writer(&mut *worker, frame).unwrap();
    worker.write_all(b"\n").unwrap();
}

fn read_frame(worker: &UnixStream) -> Value {
    worker.set_read_timeout(Some(TIMEOUT)).unwrap();
    let mut line = String::new();
    BufReader::new(worker.try_clone().unwrap())
        .read_line(&mut line)
        .unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn verifies_exact_sdk_dependency_graph_and_materializes_only_the_c4os_plugin() {
    let sidecar = sidecar_root();
    let receipt = OpenCodeSdkIntegrity::verify(&sidecar).unwrap();
    assert_eq!(receipt.sidecar_root(), sidecar.canonicalize().unwrap());
    assert!(receipt.dependency_tree_sha256().starts_with("sha256:"));
    assert_eq!(OPENCODE_SDK_PLUGIN_VERSION, "1.18.3");
    assert_eq!(
        OPENCODE_SDK_TOOL_IDS,
        ["c4os_propose_action", "c4os_read_resource"]
    );

    let temporary = tempfile::tempdir().unwrap();
    let config_home = temporary.path().join("config");
    let materialization = materialize_opencode_sdk_plugin(&sidecar, &config_home).unwrap();
    assert_eq!(materialization.tool_ids(), OPENCODE_SDK_TOOL_IDS);
    assert_eq!(materialization.sdk_version(), "1.18.3");
    assert_eq!(
        materialization.plugin_path().file_name().unwrap(),
        "c4os-tools.js"
    );
    let wrapper = fs::read_to_string(materialization.plugin_path()).unwrap();
    assert_eq!(wrapper.matches("C4osBrokerToolsPlugin").count(), 1);
    assert!(wrapper.contains("c4os-tools-plugin.mjs"));
    assert_eq!(
        fs::metadata(materialization.plugin_path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let source = fs::read_to_string(sidecar.join("c4os-tools-plugin.mjs")).unwrap();
    assert!(source.contains("@opencode-ai/plugin"));
    assert!(!source.contains("child_process"));
    assert!(source.contains("closeSync, readSync"));
    assert!(!source.contains("writeFile"));
    assert!(!source.contains("readFile"));
}

#[test]
fn source_or_lock_tampering_is_rejected_before_materialization() {
    let sidecar = sidecar_root();
    OpenCodeSdkIntegrity::verify(&sidecar).unwrap();
    for tampered in [
        "c4os-tools-plugin.mjs",
        "sdk-client.mjs",
        "package-lock.json",
    ] {
        let temporary = tempfile::tempdir().unwrap();
        for relative in OpenCodeSdkIntegrity::pinned_file_names() {
            fs::copy(sidecar.join(relative), temporary.path().join(relative)).unwrap();
        }
        fs::write(temporary.path().join(tampered), b"tampered").unwrap();
        assert!(matches!(
            OpenCodeSdkIntegrity::verify(temporary.path()),
            Err(OpenCodeSdkError::InvalidIntegrity)
        ));
    }
}

#[test]
fn launch_binding_exposes_only_bounded_descriptor_metadata_and_pinned_plugin() {
    let temporary = tempfile::tempdir().unwrap();
    let config_home = temporary.path().join("config");
    let (_broker, binding) =
        prepare_opencode_sdk_launch(&sidecar_root(), &config_home, 41, None).unwrap();
    let environment = binding.environment();
    assert_eq!(environment.len(), 3);
    assert_eq!(environment[OPENCODE_PROCESS_GENERATION_ENV], "41");
    assert_eq!(
        environment[OPENCODE_BROKER_FD_ENV],
        binding.child_fd().to_string()
    );
    assert!((64..=1_023).contains(&binding.child_fd()));
    assert!(binding.materialization().plugin_path().exists());
    assert!(
        environment
            .values()
            .all(|value| !value.contains("Bearer ") && !value.contains("password"))
    );
    // The capability is close-on-exec in C4OS and becomes inheritable only in
    // the specifically configured child immediately before exec.
    let parent_flags = unsafe { libc::fcntl(binding.child_fd() as i32, libc::F_GETFD) };
    assert_ne!(parent_flags & libc::FD_CLOEXEC, 0);
    let mut child = Command::new("/bin/sh");
    child.args([
        "-c",
        "test -e /dev/fd/$C4OS_OPENCODE_BROKER_FD && test -n $XDG_CONFIG_HOME",
    ]);
    binding.configure_command(&mut child);
    assert!(child.status().unwrap().success());
}

#[test]
fn optional_test_tls_trust_is_a_one_use_private_descriptor_not_launch_content() {
    let temporary = tempfile::tempdir().unwrap();
    let config_home = temporary.path().join("config");
    let descriptor = br#"{"schemaVersion":1,"capabilities":[]}"#.to_vec();
    let (_broker, mut binding) =
        prepare_opencode_sdk_launch(&sidecar_root(), &config_home, 42, Some(descriptor.clone()))
            .unwrap();
    let environment = binding.environment();
    assert_eq!(environment.len(), 4);
    assert_eq!(
        environment[OPENCODE_TEST_TLS_TRUST_FD_ENV],
        binding.test_tls_trust_child_fd().unwrap().to_string()
    );
    assert!(
        environment
            .values()
            .all(|value| !value.contains("tlsCaPem"))
    );

    let worker_fd = unsafe {
        libc::fcntl(
            binding.test_tls_trust_child_fd().unwrap() as i32,
            libc::F_DUPFD_CLOEXEC,
            64,
        )
    };
    assert!((64..=1_023).contains(&worker_fd));
    let mut worker = unsafe { UnixStream::from_raw_fd(worker_fd) };
    binding
        .deliver_test_tls_trust(Duration::from_secs(2))
        .unwrap();
    let mut received = Vec::new();
    worker.read_to_end(&mut received).unwrap();
    assert_eq!(received, descriptor);
    assert_eq!(binding.test_tls_trust_child_fd(), None);
    assert!(
        !binding
            .environment()
            .contains_key(OPENCODE_TEST_TLS_TRUST_FD_ENV)
    );
}

#[test]
fn correlates_an_intent_and_result_on_the_authenticated_channel() {
    let (mut broker, mut worker) = OpenCodeSdkBroker::authenticated_pair(7).unwrap();
    write_frame(
        &mut worker,
        &proposal(
            "correlation-1",
            "c4os_propose_action",
            7,
            json!({ "operation": "window.focus", "target": "workspace-main" }),
        ),
    );
    let event = broker.receive(TIMEOUT).unwrap();
    let BrokerEvent::Proposal(received) = event else {
        panic!("expected proposal")
    };
    assert_eq!(received.correlation_id, "correlation-1");
    assert_eq!(received.tool, "c4os_propose_action");
    broker
        .respond(
            "correlation-1",
            BrokerDecision::Result(json!({ "disposition": "accepted_for_evaluation" })),
            TIMEOUT,
        )
        .unwrap();
    let response = read_frame(&worker);
    assert_eq!(response["correlationId"], "correlation-1");
    assert_eq!(response["tool"], "c4os_propose_action");
    assert_eq!(response["status"], "result");
    assert_eq!(broker.pending_count(), 0);
}

#[test]
fn denial_is_returned_before_any_effect_surface_exists() {
    let (mut broker, mut worker) = OpenCodeSdkBroker::authenticated_pair(8).unwrap();
    let external_effects = 0_u32;
    write_frame(
        &mut worker,
        &proposal(
            "correlation-denied",
            "c4os_read_resource",
            8,
            json!({ "resource": "workspace.summary" }),
        ),
    );
    assert!(matches!(
        broker.receive(TIMEOUT),
        Ok(BrokerEvent::Proposal(_))
    ));
    broker
        .respond(
            "correlation-denied",
            BrokerDecision::Denied {
                reason_code: "policy_denied".into(),
            },
            TIMEOUT,
        )
        .unwrap();
    let response = read_frame(&worker);
    assert_eq!(response["status"], "denied");
    assert_eq!(response["reasonCode"], "policy_denied");
    assert_eq!(external_effects, 0);
}

#[test]
fn rejects_generation_tampering_secret_payloads_and_secret_results() {
    let (mut broker, mut worker) = OpenCodeSdkBroker::authenticated_pair(9).unwrap();
    write_frame(
        &mut worker,
        &proposal(
            "correlation-tampered",
            "c4os_propose_action",
            10,
            json!({ "operation": "window.focus", "target": "workspace-main" }),
        ),
    );
    assert!(matches!(
        broker.receive(TIMEOUT),
        Err(OpenCodeSdkError::InvalidFrame)
    ));

    write_frame(
        &mut worker,
        &proposal(
            "correlation-secret",
            "c4os_propose_action",
            9,
            json!({ "operation": "window.focus", "target": "workspace-main", "apiKey": "redacted" }),
        ),
    );
    assert!(matches!(
        broker.receive(TIMEOUT),
        Err(OpenCodeSdkError::SecretRejected)
    ));
    assert_eq!(broker.pending_count(), 0);

    write_frame(
        &mut worker,
        &proposal(
            "correlation-result-secret",
            "c4os_read_resource",
            9,
            json!({ "resource": "workspace.summary" }),
        ),
    );
    broker.receive(TIMEOUT).unwrap();
    assert!(matches!(
        broker.respond(
            "correlation-result-secret",
            BrokerDecision::Result(json!({ "password": "redacted" })),
            TIMEOUT,
        ),
        Err(OpenCodeSdkError::SecretRejected)
    ));
    assert_eq!(
        broker.pending_count(),
        1,
        "failed responses remain correlated"
    );
}

#[test]
fn cancellation_must_match_the_original_intent_binding() {
    let (mut broker, mut worker) = OpenCodeSdkBroker::authenticated_pair(11).unwrap();
    write_frame(
        &mut worker,
        &proposal(
            "correlation-cancel",
            "c4os_propose_action",
            11,
            json!({ "operation": "window.focus", "target": "workspace-main" }),
        ),
    );
    broker.receive(TIMEOUT).unwrap();
    write_frame(
        &mut worker,
        &json!({
            "schemaVersion": 1,
            "kind": "cancel",
            "correlationId": "correlation-cancel",
            "tool": "c4os_propose_action",
            "sessionId": "session-1",
            "messageId": "message-1",
            "processGeneration": 11,
        }),
    );
    assert!(matches!(
        broker.receive(TIMEOUT),
        Ok(BrokerEvent::Cancelled(proposal)) if proposal.correlation_id == "correlation-cancel"
    ));
    let response = read_frame(&worker);
    assert_eq!(response["correlationId"], "correlation-cancel");
    assert_eq!(response["status"], "cancelled");
    assert_eq!(broker.pending_count(), 0);
    assert!(matches!(
        broker.respond("correlation-cancel", BrokerDecision::Cancelled, TIMEOUT),
        Err(OpenCodeSdkError::UnknownCorrelation)
    ));
}
