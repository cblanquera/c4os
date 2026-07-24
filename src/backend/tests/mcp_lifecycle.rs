#![allow(deprecated)]

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
};

use c4os_lib::mcp::{
    MCP_PROTOCOL_VERSION, MCP_STATE_SCHEMA_VERSION, McpAuditEvent, McpCapabilitySnapshot,
    McpDefinitionSource, McpEnvironmentBinding, McpEnvironmentSource, McpError,
    McpInvocationStatus, McpLifecycle, McpPendingTrustApproval, McpScope, McpSecretReference,
    McpServerDefinitionInput, McpServerMutationInput, McpServerSnapshot, McpServiceSnapshot,
    McpToolCallInput, McpToolSnapshot, McpTransportDefinition, McpTrustApprovalState,
    McpTrustState, McpWorkingDirectory,
    service::{
        McpAuthority, McpAuthorityEffectStatus, McpAuthorityRequest, McpRepository, McpService,
    },
    transport::{
        McpCancellation, McpConnection, McpFuture, McpHandshakeSnapshot, McpRawResult,
        McpSamplingBroker, McpSamplingContext, McpTransportFactory, ResolvedMcpLaunch,
        broker_sampling_request,
    },
};
use c4os_lib::{
    core::database::{DatabaseActor, DatabaseDescriptor},
    mcp::database::DatabaseMcpRepository,
};
use rmcp::model::{CreateMessageRequestParams, CreateMessageResult, SamplingMessage};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

#[derive(Default)]
struct MemoryRepository {
    state: Mutex<Option<McpServiceSnapshot>>,
    events: Mutex<Vec<McpAuditEvent>>,
    fail_next_terminal_compare_and_swap: AtomicBool,
}

impl McpRepository for MemoryRepository {
    fn load(&self) -> Result<Option<McpServiceSnapshot>, McpError> {
        Ok(self.state.lock().expect("state lock").clone())
    }

    fn compare_and_swap(
        &self,
        expected_generation: Option<u64>,
        replacement: &McpServiceSnapshot,
        event: &McpAuditEvent,
    ) -> Result<(), McpError> {
        if matches!(
            event.event_kind.as_str(),
            "tool.completed" | "resource.completed"
        ) && self
            .fail_next_terminal_compare_and_swap
            .swap(false, Ordering::SeqCst)
        {
            return Err(McpError::Persistence(
                "injected terminal commit failure".into(),
            ));
        }
        let mut state = self.state.lock().expect("state lock");
        if state.as_ref().map(|value| value.generation) != expected_generation {
            return Err(McpError::Conflict);
        }
        *state = Some(replacement.clone());
        self.events.lock().expect("events lock").push(event.clone());
        Ok(())
    }
}

impl MemoryRepository {
    fn fail_next_terminal_compare_and_swap(&self) {
        self.fail_next_terminal_compare_and_swap
            .store(true, Ordering::SeqCst);
    }
}

#[derive(Default)]
struct FakeAuthority {
    deny: AtomicBool,
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl McpAuthority for FakeAuthority {
    type EffectLease = u64;

    fn resolve_launch(&self, _server: &McpServerSnapshot) -> Result<ResolvedMcpLaunch, McpError> {
        Ok(ResolvedMcpLaunch::default())
    }

    fn begin_authorized_effect(
        &self,
        _request: &McpAuthorityRequest<'_>,
        _now_ms: u64,
    ) -> Result<Self::EffectLease, McpError> {
        if self.deny.load(Ordering::SeqCst) {
            self.order.lock().expect("order lock").push("begin-denied");
            return Err(McpError::Denied);
        }
        self.order.lock().expect("order lock").push("begin");
        Ok(7)
    }

    fn redact_result(
        &self,
        _request: &McpAuthorityRequest<'_>,
        mut untrusted: Value,
    ) -> Result<Value, McpError> {
        self.order.lock().expect("order lock").push("redact");
        if let Some(value) = untrusted.as_object_mut() {
            value.remove("secret");
            value.remove("structuredContent");
        }
        Ok(untrusted)
    }

    fn complete_authorized_effect(
        &self,
        lease: Self::EffectLease,
        status: McpAuthorityEffectStatus,
        redacted_result: &Value,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        assert_eq!(lease, 7);
        assert_eq!(status, McpAuthorityEffectStatus::Succeeded);
        assert_eq!(redacted_result, &json!({ "value": "ok" }));
        self.order.lock().expect("order lock").push("complete");
        Ok(())
    }

    fn fail_authorized_effect(
        &self,
        lease: Self::EffectLease,
        _status: McpAuthorityEffectStatus,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        assert_eq!(lease, 7);
        self.order.lock().expect("order lock").push("fail");
        Ok(())
    }

    fn revoke_server(
        &self,
        _server: &McpServerSnapshot,
        _reason: &str,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        self.order.lock().expect("order lock").push("revoke");
        Ok(())
    }
}

#[derive(Default)]
struct FakeFactory {
    connects: AtomicUsize,
    fail_connect: AtomicBool,
    calls: Arc<AtomicUsize>,
    invalid_output: Arc<AtomicBool>,
    order: Arc<Mutex<Vec<&'static str>>>,
    discovery_started: Arc<AtomicBool>,
    block_discovery: Arc<AtomicBool>,
}

impl McpTransportFactory for FakeFactory {
    fn connect<'a>(
        &'a self,
        _definition: &'a McpTransportDefinition,
        _launch: ResolvedMcpLaunch,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Box<dyn McpConnection>, McpError>> {
        self.connects.fetch_add(1, Ordering::SeqCst);
        if self.fail_connect.load(Ordering::SeqCst) {
            return Box::pin(async { Err(McpError::Transport("fixture-connect".into())) });
        }
        let calls = Arc::clone(&self.calls);
        let invalid_output = Arc::clone(&self.invalid_output);
        let order = Arc::clone(&self.order);
        let discovery_started = Arc::clone(&self.discovery_started);
        let block_discovery = Arc::clone(&self.block_discovery);
        Box::pin(async move {
            Ok(Box::new(FakeConnection::new(
                calls,
                invalid_output,
                order,
                discovery_started,
                block_discovery,
            )) as Box<dyn McpConnection>)
        })
    }
}

struct FakeConnection {
    handshake: McpHandshakeSnapshot,
    calls: Arc<AtomicUsize>,
    invalid_output: Arc<AtomicBool>,
    order: Arc<Mutex<Vec<&'static str>>>,
    discovery_started: Arc<AtomicBool>,
    block_discovery: Arc<AtomicBool>,
}

impl FakeConnection {
    fn new(
        calls: Arc<AtomicUsize>,
        invalid_output: Arc<AtomicBool>,
        order: Arc<Mutex<Vec<&'static str>>>,
        discovery_started: Arc<AtomicBool>,
        block_discovery: Arc<AtomicBool>,
    ) -> Self {
        Self {
            handshake: McpHandshakeSnapshot {
                protocol_version: MCP_PROTOCOL_VERSION.into(),
                server_name: "fixture".into(),
                server_version: "1.0.0".into(),
                instructions_present: false,
                capabilities: McpCapabilitySnapshot {
                    tools: true,
                    resources: true,
                    ..McpCapabilitySnapshot::default()
                },
            },
            calls,
            invalid_output,
            order,
            discovery_started,
            block_discovery,
        }
    }
}

impl McpConnection for FakeConnection {
    fn handshake(&self) -> &McpHandshakeSnapshot {
        &self.handshake
    }

    fn notification_epoch(&self) -> u64 {
        0
    }

    fn list_tools<'a>(
        &'a self,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpToolSnapshot>, McpError>> {
        let discovery_started = Arc::clone(&self.discovery_started);
        let block_discovery = Arc::clone(&self.block_discovery);
        let input_schema = echo_input_schema();
        let input_schema_sha256 = sha256_value(&input_schema);
        let output_schema = json!({
            "type": "object",
            "properties": { "answer": { "type": "string" } },
            "required": ["answer"],
            "additionalProperties": false,
        });
        let output_schema_sha256 = sha256_value(&output_schema);
        Box::pin(async move {
            discovery_started.store(true, Ordering::SeqCst);
            while block_discovery.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            Ok(vec![McpToolSnapshot {
                name: "echo".into(),
                title: Some("Echo".into()),
                description: None,
                input_schema,
                input_schema_sha256,
                output_schema: Some(output_schema),
                output_schema_sha256: Some(output_schema_sha256),
            }])
        })
    }

    fn list_resources<'a>(
        &'a self,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<c4os_lib::mcp::McpResourceSnapshot>, McpError>> {
        Box::pin(async {
            Ok(vec![c4os_lib::mcp::McpResourceSnapshot {
                uri: "fixture://readme".into(),
                name: "readme".into(),
                title: None,
                description: None,
                mime_type: Some("text/plain".into()),
                size: Some(2),
            }])
        })
    }

    fn call_tool<'a>(
        &'a self,
        _name: &'a str,
        _arguments: Value,
        _timeout_ms: u64,
        _max_output_bytes: u64,
        _cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.order.lock().expect("order lock").push("call");
        let invalid_output = self.invalid_output.load(Ordering::SeqCst);
        Box::pin(async move {
            Ok(McpRawResult {
                value: json!({
                    "value": "ok",
                    "secret": "must-not-persist",
                    "structuredContent": {
                        "answer": if invalid_output { json!(7) } else { json!("ok") },
                    },
                }),
                is_error: false,
                output_bytes: 43,
            })
        })
    }

    fn read_resource<'a>(
        &'a self,
        _uri: &'a str,
        _timeout_ms: u64,
        _max_output_bytes: u64,
        _cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        Box::pin(async {
            Ok(McpRawResult {
                value: json!({ "contents": ["ok"] }),
                is_error: false,
                output_bytes: 20,
            })
        })
    }

    fn close(self: Box<Self>, _timeout_ms: u64) -> McpFuture<'static, Result<(), McpError>> {
        Box::pin(async { Ok(()) })
    }
}

fn definition(expected_generation: u64) -> McpServerDefinitionInput {
    McpServerDefinitionInput {
        expected_generation,
        server_id: "fixture".into(),
        display_name: "Fixture MCP".into(),
        scope: McpScope::Application,
        transport: McpTransportDefinition::Stdio {
            command: "/bin/echo".into(),
            arguments: Vec::new(),
            environment: Vec::new(),
            working_directory: McpWorkingDirectory::C4osHome,
            executable_sha256: Some(format!("sha256:{}", "0".repeat(64))),
        },
        timeout_ms: 1_000,
        max_output_bytes: 64 * 1_024,
    }
}

fn credential_definition(
    expected_generation: u64,
    credential_reference: &str,
) -> McpServerDefinitionInput {
    let mut input = definition(expected_generation);
    input.transport = McpTransportDefinition::Stdio {
        command: "/bin/echo".into(),
        arguments: Vec::new(),
        environment: vec![McpEnvironmentBinding {
            name: "FIXTURE_TOKEN".into(),
            source: McpEnvironmentSource::Secret {
                reference: McpSecretReference::Vault {
                    credential_reference: credential_reference.into(),
                },
            },
        }],
        working_directory: McpWorkingDirectory::C4osHome,
        executable_sha256: Some(format!("sha256:{}", "0".repeat(64))),
    };
    input
}

fn echo_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "value": { "type": "string" },
        },
        "required": ["value"],
        "additionalProperties": false,
    })
}

fn sha256_value(value: &Value) -> String {
    let encoded = serde_json::to_vec(value).expect("canonical test JSON");
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(encoded) {
        use std::fmt::Write as _;
        write!(&mut digest, "{byte:02x}").expect("write digest");
    }
    digest
}

fn tool_call(snapshot: &McpServiceSnapshot) -> McpToolCallInput {
    McpToolCallInput {
        expected_generation: snapshot.generation,
        server_id: "fixture".into(),
        lifecycle_generation: snapshot.servers[0].lifecycle_generation,
        tool_name: "echo".into(),
        arguments: json!({ "value": "hi" }),
        run_id: "run-1".into(),
        workspace_id: "workspace-1".into(),
        project_id: "project-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lifecycle_cancellation_interrupts_inflight_server_test_discovery() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let discovery_started = Arc::new(AtomicBool::new(false));
    let block_discovery = Arc::new(AtomicBool::new(true));
    let factory = Arc::new(FakeFactory {
        discovery_started: Arc::clone(&discovery_started),
        block_discovery,
        ..FakeFactory::default()
    });
    let mut service = McpService::restore(repository, authority, factory, 1).expect("restore");
    let snapshot = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "1".repeat(64));
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:cancellable-test".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record trust approval");
    let snapshot = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:cancellable-test",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    let cancellation = McpCancellation::default();
    let cancel_when_discovery_starts = {
        let cancellation = cancellation.clone();
        tokio::spawn(async move {
            while !discovery_started.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
            cancellation.cancel();
        })
    };

    let result = service
        .test_server_cancellable(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            5,
            cancellation,
        )
        .await;
    cancel_when_discovery_starts.await.expect("canceller");
    assert!(matches!(result, Err(McpError::Cancelled)));
    assert_eq!(
        service.snapshot().servers[0].lifecycle,
        McpLifecycle::Failed
    );
}

#[tokio::test]
async fn authority_lease_strictly_wraps_transport_and_denial_precedes_effect() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory {
        order: Arc::clone(&authority.order),
        ..FakeFactory::default()
    });
    let mut service = McpService::restore(
        Arc::clone(&repository),
        Arc::clone(&authority),
        Arc::clone(&factory),
        1,
    )
    .expect("restore");
    let snapshot = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:test".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: format!("sha256:{}", "1".repeat(64)),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record exact trust approval");
    let snapshot = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:test",
            &format!("sha256:{}", "1".repeat(64)),
            &definition_sha256,
            4,
        )
        .expect("trust exact definition");
    let snapshot = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable");
    assert_eq!(snapshot.schema_version, MCP_STATE_SCHEMA_VERSION);
    assert_eq!(snapshot.servers[0].lifecycle, McpLifecycle::Ready);

    authority.order.lock().expect("order lock").clear();
    let invocation = service
        .call_tool(&tool_call(&snapshot), McpCancellation::default(), 4)
        .await
        .expect("call");
    assert_eq!(invocation.status, McpInvocationStatus::Succeeded);
    assert_eq!(invocation.redacted_content, json!({ "value": "ok" }));
    assert_eq!(
        *authority.order.lock().expect("order lock"),
        vec!["begin", "call", "redact", "complete"]
    );

    authority.order.lock().expect("order lock").clear();
    let before = factory.calls.load(Ordering::SeqCst);
    let mut invalid = tool_call(&service.snapshot());
    invalid.arguments = json!({ "value": 7, "unexpected": true });
    let denied = service
        .call_tool(&invalid, McpCancellation::default(), 5)
        .await;
    assert!(matches!(denied, Err(McpError::Denied)));
    assert_eq!(factory.calls.load(Ordering::SeqCst), before);
    assert!(authority.order.lock().expect("order lock").is_empty());

    authority.deny.store(true, Ordering::SeqCst);
    authority.order.lock().expect("order lock").clear();
    let before = factory.calls.load(Ordering::SeqCst);
    let denied_snapshot = service.snapshot();
    let denied = service
        .call_tool(&tool_call(&denied_snapshot), McpCancellation::default(), 6)
        .await;
    assert!(matches!(denied, Err(McpError::Denied)));
    assert_eq!(factory.calls.load(Ordering::SeqCst), before);
    assert_eq!(
        *authority.order.lock().expect("order lock"),
        vec!["begin-denied"]
    );

    authority.deny.store(false, Ordering::SeqCst);
    factory.invalid_output.store(true, Ordering::SeqCst);
    authority.order.lock().expect("order lock").clear();
    let invalid_output = service
        .call_tool(
            &tool_call(&service.snapshot()),
            McpCancellation::default(),
            7,
        )
        .await
        .expect("schema mismatch is a normalized invocation failure");
    assert_eq!(invalid_output.status, McpInvocationStatus::Failed);
    assert_eq!(
        invalid_output.redacted_content,
        json!({ "error": { "code": "transport" } })
    );
    assert_eq!(
        service.snapshot().servers[0].lifecycle,
        McpLifecycle::Failed
    );
    assert_eq!(
        *authority.order.lock().expect("order lock"),
        vec!["begin", "call", "fail"]
    );
}

#[tokio::test]
async fn terminal_commit_failure_closes_worker_and_publishes_recoverable_projection() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory {
        order: Arc::clone(&authority.order),
        ..FakeFactory::default()
    });
    let mut service = McpService::restore(
        Arc::clone(&repository),
        Arc::clone(&authority),
        Arc::clone(&factory),
        1,
    )
    .expect("restore");
    let saved = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let binding_sha256 = format!("sha256:{}", "7".repeat(64));
    let pending = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: saved.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:terminal-commit".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: binding_sha256.clone(),
                action_configuration_version: saved.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record approval");
    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: pending.generation,
                server_id: "fixture".into(),
            },
            "approval:terminal-commit",
            &binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    let ready = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: trusted.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable server");

    // mark_executing commits first; inject the fault only after transport and
    // authority finalization have begun so the terminal CAS is deterministic.
    repository.fail_next_terminal_compare_and_swap();
    let result = service
        .call_tool(&tool_call(&ready), McpCancellation::default(), 6)
        .await;

    assert!(matches!(result, Err(McpError::Persistence(_))));
    let snapshot = service.snapshot();
    assert_eq!(snapshot.active_workers, 0);
    assert_eq!(snapshot.servers[0].lifecycle, McpLifecycle::Failed);
    assert_eq!(snapshot.servers[0].active_requests, 0);
    assert_eq!(
        snapshot.servers[0].last_failure_code.as_deref(),
        Some("persistence_commit_failed")
    );
    assert_eq!(factory.calls.load(Ordering::SeqCst), 1);
    assert_eq!(factory.connects.load(Ordering::SeqCst), 1);
    let direct_enable = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            7,
        )
        .await;
    assert!(matches!(direct_enable, Err(McpError::InvalidState)));
    assert_eq!(factory.connects.load(Ordering::SeqCst), 1);
    assert_eq!(factory.calls.load(Ordering::SeqCst), 1);

    let recovered = service
        .recover_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            8,
        )
        .await
        .expect("explicit recovery restarts the worker");
    assert_eq!(recovered.servers[0].lifecycle, McpLifecycle::Ready);
    assert_eq!(recovered.active_workers, 1);
    assert_eq!(factory.connects.load(Ordering::SeqCst), 2);
    assert_eq!(
        factory.calls.load(Ordering::SeqCst),
        1,
        "the transport call completed before the CAS fault and must never replay"
    );
}

#[tokio::test]
async fn durable_executing_state_normalizes_on_database_restart_without_replay() {
    let temporary = TempDir::new().expect("temporary app home");
    let descriptor = DatabaseDescriptor::app(temporary.path());
    let (database, _) = DatabaseActor::start(descriptor.clone()).expect("app database");
    let database = Arc::new(database);
    let repository = Arc::new(DatabaseMcpRepository::new(Arc::clone(&database), None));
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory {
        order: Arc::clone(&authority.order),
        ..FakeFactory::default()
    });
    let mut service = McpService::restore(
        Arc::clone(&repository),
        Arc::clone(&authority),
        Arc::clone(&factory),
        1,
    )
    .expect("restore database service");
    let saved = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let binding_sha256 = format!("sha256:{}", "8".repeat(64));
    let pending = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: saved.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:restart-executing".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: binding_sha256.clone(),
                action_configuration_version: saved.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record approval");
    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: pending.generation,
                server_id: "fixture".into(),
            },
            "approval:restart-executing",
            &binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    let ready = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: trusted.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable server");

    let mut executing = ready.clone();
    executing.generation += 1;
    executing.last_event_id += 1;
    executing.active_workers = 1;
    executing.servers[0].lifecycle = McpLifecycle::Executing;
    executing.servers[0].lifecycle_generation += 1;
    executing.servers[0].active_requests = 1;
    let executing_event = McpAuditEvent {
        schema_version: MCP_STATE_SCHEMA_VERSION,
        event_id: executing.last_event_id,
        generation: executing.generation,
        lifecycle_generation: executing.servers[0].lifecycle_generation,
        operation_id: "operation:restart-executing".into(),
        server_id: "fixture".into(),
        event_kind: "tool.executing".into(),
        target: Some("echo".into()),
        result: "started".into(),
        detail: None,
        occurred_at_ms: 6,
    };
    repository
        .compare_and_swap(Some(ready.generation), &executing, &executing_event)
        .expect("persist crash boundary");
    drop(service);
    drop(repository);
    drop(database);

    let (database, _) = DatabaseActor::start(descriptor).expect("restart app database");
    let repository = Arc::new(DatabaseMcpRepository::new(Arc::new(database), None));
    let restart_factory = Arc::new(FakeFactory::default());
    let mut restored = McpService::restore(
        Arc::clone(&repository),
        authority,
        Arc::clone(&restart_factory),
        7,
    )
    .expect("normalize executing state after restart");
    let recovered = restored.snapshot();
    assert_eq!(recovered.active_workers, 0);
    assert_eq!(recovered.servers[0].active_requests, 0);
    assert_eq!(recovered.servers[0].lifecycle, McpLifecycle::Failed);
    assert_eq!(
        recovered.servers[0].last_failure_code.as_deref(),
        Some("restart_recovery_required")
    );
    assert_eq!(restart_factory.connects.load(Ordering::SeqCst), 0);
    assert_eq!(restart_factory.calls.load(Ordering::SeqCst), 0);
    assert!(
        repository
            .load()
            .expect("load normalized database state")
            .is_some_and(|snapshot| snapshot == recovered)
    );

    let direct_enable = restored
        .enable_server(
            &McpServerMutationInput {
                expected_generation: recovered.generation,
                server_id: "fixture".into(),
            },
            8,
        )
        .await;
    assert!(matches!(direct_enable, Err(McpError::InvalidState)));
    let ready = restored
        .recover_server(
            &McpServerMutationInput {
                expected_generation: recovered.generation,
                server_id: "fixture".into(),
            },
            9,
        )
        .await
        .expect("explicit recovery reconnects");
    assert_eq!(ready.servers[0].lifecycle, McpLifecycle::Ready);
    assert_eq!(restart_factory.connects.load(Ordering::SeqCst), 1);
    assert_eq!(restart_factory.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn credential_invalidation_revokes_authority_stops_worker_and_records_failure() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory::default());
    let credential_reference = format!("credential:{}", "a".repeat(32));
    let mut service =
        McpService::restore(Arc::clone(&repository), Arc::clone(&authority), factory, 1)
            .expect("restore");
    let snapshot = service
        .upsert_server(
            credential_definition(service.snapshot().generation, &credential_reference),
            McpDefinitionSource::User,
            2,
        )
        .expect("save credential definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "1".repeat(64));
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:credential".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record approval");
    let snapshot = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:credential",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    let ready = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable credentialed server");
    assert_eq!(ready.active_workers, 1);
    let lifecycle_generation = ready.servers[0].lifecycle_generation;
    authority.order.lock().expect("order lock").clear();

    let invalidated = service
        .invalidate_credential_reference(&credential_reference, "credential_removed", 6)
        .await
        .expect("invalidate credential reference");

    assert_eq!(invalidated.active_workers, 0);
    assert_eq!(invalidated.servers[0].lifecycle, McpLifecycle::Failed);
    assert_eq!(
        invalidated.servers[0].lifecycle_generation,
        lifecycle_generation + 1
    );
    assert_eq!(
        invalidated.servers[0].last_failure_code.as_deref(),
        Some("credential_removed")
    );
    assert_eq!(*authority.order.lock().expect("order lock"), vec!["revoke"]);
    let events = repository.events.lock().expect("events lock");
    assert_eq!(
        events.last().map(|event| event.event_kind.as_str()),
        Some("credential.invalidated")
    );
    drop(events);

    let direct_enable = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: invalidated.generation,
                server_id: "fixture".into(),
            },
            7,
        )
        .await;
    assert!(matches!(direct_enable, Err(McpError::InvalidState)));
    let recovered = service
        .recover_server(
            &McpServerMutationInput {
                expected_generation: invalidated.generation,
                server_id: "fixture".into(),
            },
            8,
        )
        .await
        .expect("explicit recovery");
    assert_eq!(recovered.servers[0].lifecycle, McpLifecycle::Ready);
}

#[tokio::test]
async fn recovery_enforces_failure_backoff_before_reconnecting() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory::default());
    let mut service =
        McpService::restore(Arc::clone(&repository), authority, Arc::clone(&factory), 1)
            .expect("restore");
    let snapshot = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "3".repeat(64));
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:recovery".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record approval");
    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:recovery",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    factory.fail_connect.store(true, Ordering::SeqCst);
    let failed = service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: trusted.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await;
    assert!(matches!(failed, Err(McpError::Transport(_))));
    let failed = service.snapshot();
    assert_eq!(failed.servers[0].lifecycle, McpLifecycle::Failed);
    assert_eq!(failed.servers[0].next_restart_at_ms, Some(1_005));
    let connects = factory.connects.load(Ordering::SeqCst);

    let too_early = service
        .recover_server(
            &McpServerMutationInput {
                expected_generation: failed.generation,
                server_id: "fixture".into(),
            },
            1_004,
        )
        .await;
    assert!(matches!(too_early, Err(McpError::InvalidState)));
    assert_eq!(factory.connects.load(Ordering::SeqCst), connects);

    factory.fail_connect.store(false, Ordering::SeqCst);
    let recovered = service
        .recover_server(
            &McpServerMutationInput {
                expected_generation: failed.generation,
                server_id: "fixture".into(),
            },
            1_005,
        )
        .await
        .expect("recover after backoff");
    assert_eq!(recovered.servers[0].lifecycle, McpLifecycle::Ready);
    assert_eq!(recovered.servers[0].next_restart_at_ms, None);
    assert!(
        repository
            .events
            .lock()
            .expect("events lock")
            .iter()
            .any(|event| event.event_kind == "lifecycle.restarting")
    );
}

#[test]
fn trust_is_bound_to_one_definition_and_updates_invalidate_it() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory::default());
    let mut service = McpService::restore(repository, authority, factory, 1).expect("restore");
    let snapshot = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "1".repeat(64));
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:binding".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record exact trust approval");

    let stale = service.trust_server(
        &McpServerMutationInput {
            expected_generation: snapshot.generation,
            server_id: "fixture".into(),
        },
        "approval:binding",
        &action_binding_sha256,
        &format!("sha256:{}", "f".repeat(64)),
        4,
    );
    assert!(matches!(stale, Err(McpError::Conflict)));

    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:binding",
            &action_binding_sha256,
            &definition_sha256,
            5,
        )
        .expect("trust exact definition");
    assert_eq!(trusted.servers[0].trust, McpTrustState::Trusted);
    assert_eq!(
        trusted.servers[0].trusted_definition_sha256.as_deref(),
        Some(definition_sha256.as_str())
    );

    let mut replacement = definition(trusted.generation);
    replacement.display_name = "Changed fixture".into();
    let updated = service
        .upsert_server(replacement, McpDefinitionSource::User, 6)
        .expect("update definition");
    assert_eq!(updated.servers[0].trust, McpTrustState::Pending);
    assert_eq!(updated.servers[0].trusted_definition_sha256, None);
    assert_eq!(updated.servers[0].pending_trust_approval, None);
}

#[test]
fn restart_downgrades_legacy_trust_without_an_exact_definition_binding() {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let factory = Arc::new(FakeFactory::default());
    let mut service = McpService::restore(
        Arc::clone(&repository),
        Arc::clone(&authority),
        Arc::clone(&factory),
        1,
    )
    .expect("restore");
    let snapshot = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "2".repeat(64));
    let snapshot = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:legacy".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: snapshot.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record trust approval");
    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            "approval:legacy",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust exact definition");
    drop(service);

    let mut legacy = trusted;
    legacy.servers[0].trusted_definition_sha256 = None;
    *repository.state.lock().expect("state lock") = Some(legacy);
    let recovered = McpService::restore(repository, authority, factory, 5)
        .expect("fail-closed legacy recovery")
        .snapshot();
    assert_eq!(recovered.servers[0].trust, McpTrustState::Pending);
    assert_eq!(recovered.servers[0].lifecycle, McpLifecycle::Disabled);
    assert_eq!(recovered.servers[0].trusted_definition_sha256, None);
    assert_eq!(
        recovered.servers[0].last_failure_code.as_deref(),
        Some("trust_binding_required")
    );
}

struct FakeSamplingBroker {
    mode: AtomicU8,
    calls: AtomicUsize,
}

impl McpSamplingBroker for FakeSamplingBroker {
    fn sample<'a>(
        &'a self,
        _context: &'a McpSamplingContext,
        _request: CreateMessageRequestParams,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<CreateMessageResult, McpError>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            match self.mode.load(Ordering::SeqCst) {
                1 => Err(McpError::Denied),
                2 => {
                    cancellation.cancelled().await;
                    Err(McpError::Cancelled)
                }
                _ => Ok(CreateMessageResult::new(
                    SamplingMessage::assistant_text("redacted response"),
                    "fixture-model".into(),
                )
                .with_stop_reason(CreateMessageResult::STOP_REASON_END_TURN)),
            }
        })
    }
}

#[tokio::test]
async fn sampling_is_brokered_bounded_denied_and_cancellable() {
    let broker = Arc::new(FakeSamplingBroker {
        mode: AtomicU8::new(0),
        calls: AtomicUsize::new(0),
    });
    let request = CreateMessageRequestParams::new(vec![SamplingMessage::user_text("hello")], 64);
    let result = broker_sampling_request(
        broker.as_ref(),
        &sampling_context(),
        request.clone(),
        McpCancellation::default(),
    )
    .await
    .expect("brokered sampling");
    assert_eq!(result.model, "fixture-model");

    broker.mode.store(1, Ordering::SeqCst);
    let denied = broker_sampling_request(
        broker.as_ref(),
        &sampling_context(),
        request.clone(),
        McpCancellation::default(),
    )
    .await;
    assert!(matches!(denied, Err(McpError::Denied)));

    broker.mode.store(2, Ordering::SeqCst);
    let cancellation = McpCancellation::default();
    let cancellation_for_task = cancellation.clone();
    let broker_for_task = Arc::clone(&broker);
    let task = tokio::spawn(async move {
        broker_sampling_request(
            broker_for_task.as_ref(),
            &sampling_context(),
            request,
            cancellation_for_task,
        )
        .await
    });
    tokio::task::yield_now().await;
    cancellation.cancel();
    assert!(matches!(
        task.await.expect("join"),
        Err(McpError::Cancelled)
    ));
    assert_eq!(broker.calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn sampling_rejects_unbounded_requests_before_broker_dispatch() {
    let broker = FakeSamplingBroker {
        mode: AtomicU8::new(0),
        calls: AtomicUsize::new(0),
    };
    let result = broker_sampling_request(
        &broker,
        &sampling_context(),
        CreateMessageRequestParams::new(vec![SamplingMessage::user_text("x")], 1_000_001),
        McpCancellation::default(),
    )
    .await;
    assert!(matches!(result, Err(McpError::BoundExceeded)));
    assert_eq!(broker.calls.load(Ordering::SeqCst), 0);
}

fn sampling_context() -> McpSamplingContext {
    McpSamplingContext {
        server_id: "fixture".into(),
        authority_id: format!("mcp:{}", "a".repeat(64)),
        lifecycle_generation: 1,
        definition_sha256: format!("sha256:{}", "b".repeat(64)),
        timeout_ms: 5_000,
        max_output_bytes: 64 * 1_024,
    }
}

#[derive(Default)]
struct DiscoveryScenarioState {
    notification_epoch: AtomicU64,
    tool_lists: AtomicUsize,
    resource_lists: AtomicUsize,
    notify_during_first_tool_list: AtomicBool,
    catalog_revision: AtomicUsize,
}

struct DiscoveryScenarioFactory {
    capabilities: McpCapabilitySnapshot,
    state: Arc<DiscoveryScenarioState>,
}

impl McpTransportFactory for DiscoveryScenarioFactory {
    fn connect<'a>(
        &'a self,
        _definition: &'a McpTransportDefinition,
        _launch: ResolvedMcpLaunch,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Box<dyn McpConnection>, McpError>> {
        let capabilities = self.capabilities.clone();
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            Ok(Box::new(DiscoveryScenarioConnection {
                handshake: McpHandshakeSnapshot {
                    protocol_version: MCP_PROTOCOL_VERSION.into(),
                    server_name: "discovery-fixture".into(),
                    server_version: "1.0.0".into(),
                    instructions_present: false,
                    capabilities,
                },
                state,
            }) as Box<dyn McpConnection>)
        })
    }
}

struct DiscoveryScenarioConnection {
    handshake: McpHandshakeSnapshot,
    state: Arc<DiscoveryScenarioState>,
}

impl McpConnection for DiscoveryScenarioConnection {
    fn handshake(&self) -> &McpHandshakeSnapshot {
        &self.handshake
    }

    fn notification_epoch(&self) -> u64 {
        self.state.notification_epoch.load(Ordering::SeqCst)
    }

    fn list_tools<'a>(
        &'a self,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpToolSnapshot>, McpError>> {
        let revision = self.state.catalog_revision.load(Ordering::SeqCst);
        self.state.tool_lists.fetch_add(1, Ordering::SeqCst);
        if self
            .state
            .notify_during_first_tool_list
            .swap(false, Ordering::SeqCst)
        {
            self.state.catalog_revision.store(1, Ordering::SeqCst);
            self.state.notification_epoch.fetch_add(1, Ordering::SeqCst);
        }
        Box::pin(async move {
            let input_schema = echo_input_schema();
            Ok(vec![McpToolSnapshot {
                name: if revision == 0 {
                    "echo".into()
                } else {
                    "echo-updated".into()
                },
                title: None,
                description: None,
                input_schema_sha256: sha256_value(&input_schema),
                input_schema,
                output_schema: None,
                output_schema_sha256: None,
            }])
        })
    }

    fn list_resources<'a>(
        &'a self,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<c4os_lib::mcp::McpResourceSnapshot>, McpError>> {
        let revision = self.state.catalog_revision.load(Ordering::SeqCst);
        self.state.resource_lists.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Ok(vec![c4os_lib::mcp::McpResourceSnapshot {
                uri: format!("fixture://resource-{revision}"),
                name: format!("resource-{revision}"),
                title: None,
                description: None,
                mime_type: Some("text/plain".into()),
                size: Some(1),
            }])
        })
    }

    fn call_tool<'a>(
        &'a self,
        _name: &'a str,
        _arguments: Value,
        _timeout_ms: u64,
        _max_output_bytes: u64,
        _cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        Box::pin(async { Err(McpError::Denied) })
    }

    fn read_resource<'a>(
        &'a self,
        _uri: &'a str,
        _timeout_ms: u64,
        _max_output_bytes: u64,
        _cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        Box::pin(async { Err(McpError::Denied) })
    }

    fn close(self: Box<Self>, _timeout_ms: u64) -> McpFuture<'static, Result<(), McpError>> {
        Box::pin(async { Ok(()) })
    }
}

async fn ready_discovery_service(
    factory: Arc<DiscoveryScenarioFactory>,
) -> McpService<MemoryRepository, FakeAuthority, DiscoveryScenarioFactory> {
    let repository = Arc::new(MemoryRepository::default());
    let authority = Arc::new(FakeAuthority::default());
    let mut service = McpService::restore(repository, authority, factory, 1).expect("restore");
    let saved = service
        .upsert_server(
            definition(service.snapshot().generation),
            McpDefinitionSource::User,
            2,
        )
        .expect("save definition");
    let definition_sha256 = service
        .definition_sha256("fixture")
        .expect("definition digest");
    let action_binding_sha256 = format!("sha256:{}", "4".repeat(64));
    let approval = service
        .record_trust_approval(
            &McpServerMutationInput {
                expected_generation: saved.generation,
                server_id: "fixture".into(),
            },
            McpPendingTrustApproval {
                prompt_id: "approval:discovery".into(),
                definition_sha256: definition_sha256.clone(),
                action_binding_sha256: action_binding_sha256.clone(),
                action_configuration_version: saved.generation,
                requested_at_ms: 3,
                expires_at_ms: 300,
                state: McpTrustApprovalState::Pending,
            },
            3,
        )
        .expect("record trust approval");
    let trusted = service
        .trust_server(
            &McpServerMutationInput {
                expected_generation: approval.generation,
                server_id: "fixture".into(),
            },
            "approval:discovery",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: trusted.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable server");
    service
}

#[tokio::test]
async fn discovery_calls_only_capabilities_negotiated_by_the_server() {
    for (tools, resources, expected_tool_lists, expected_resource_lists) in [
        (true, false, 1, 0),
        (false, true, 0, 1),
        (false, false, 0, 0),
    ] {
        let state = Arc::new(DiscoveryScenarioState::default());
        let factory = Arc::new(DiscoveryScenarioFactory {
            capabilities: McpCapabilitySnapshot {
                tools,
                resources,
                ..McpCapabilitySnapshot::default()
            },
            state: Arc::clone(&state),
        });
        let service = ready_discovery_service(factory).await;
        assert_eq!(state.tool_lists.load(Ordering::SeqCst), expected_tool_lists);
        assert_eq!(
            state.resource_lists.load(Ordering::SeqCst),
            expected_resource_lists
        );
        assert_eq!(service.snapshot().servers[0].tools.is_empty(), !tools);
        assert_eq!(
            service.snapshot().servers[0].resources.is_empty(),
            !resources
        );
    }
}

#[tokio::test]
async fn racing_list_change_refreshes_only_the_next_immutable_turn_snapshot() {
    let state = Arc::new(DiscoveryScenarioState::default());
    state
        .notify_during_first_tool_list
        .store(true, Ordering::SeqCst);
    let factory = Arc::new(DiscoveryScenarioFactory {
        capabilities: McpCapabilitySnapshot {
            tools: true,
            resources: true,
            ..McpCapabilitySnapshot::default()
        },
        state: Arc::clone(&state),
    });
    let mut service = ready_discovery_service(factory).await;

    let current_turn = service
        .turn_snapshot("workspace-1", "project-1", "session-1", 6)
        .expect("current immutable turn");
    assert_eq!(current_turn.tools[0].tool_name, "echo");
    assert_eq!(state.tool_lists.load(Ordering::SeqCst), 1);
    assert_eq!(state.resource_lists.load(Ordering::SeqCst), 1);

    let next_turn = service
        .prepare_turn_snapshot("workspace-1", "project-1", "session-1", 7)
        .await
        .expect("next turn refresh");
    assert_eq!(next_turn.tools[0].tool_name, "echo-updated");
    assert_ne!(next_turn.snapshot_id, current_turn.snapshot_id);
    assert_eq!(current_turn.tools[0].tool_name, "echo");
    assert_eq!(state.tool_lists.load(Ordering::SeqCst), 2);
    assert_eq!(state.resource_lists.load(Ordering::SeqCst), 2);

    let unchanged_following_turn = service
        .prepare_turn_snapshot("workspace-1", "project-1", "session-1", 8)
        .await
        .expect("unchanged following turn");
    assert_eq!(unchanged_following_turn.tools[0].tool_name, "echo-updated");
    assert_eq!(state.tool_lists.load(Ordering::SeqCst), 2);
    assert_eq!(state.resource_lists.load(Ordering::SeqCst), 2);
}
