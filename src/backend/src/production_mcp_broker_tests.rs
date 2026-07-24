use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

use super::{
    ProductionMcpCancellationRegistry, ProductionMcpDeferredFacility, RuntimeApplicationService,
    register_mcp_lifecycle_for_live_authority,
};
use crate::core::database::{DatabaseActor, DatabaseDescriptor, SnapshotQuery};
use crate::mcp::database::DatabaseMcpRepository;
use crate::mcp::service::{
    McpAuthority, McpAuthorityEffectStatus, McpAuthorityRequest, McpService,
};
use crate::mcp::transport::{
    McpCancellation, McpConnection, McpFuture, McpHandshakeSnapshot, McpRawResult,
    McpTransportFactory, ResolvedMcpLaunch,
};
use crate::mcp::{
    MCP_PROTOCOL_VERSION, McpCapabilitySnapshot, McpDefinitionSource, McpError,
    McpPendingTrustApproval, McpResourceSnapshot, McpScope, McpServerDefinitionInput,
    McpServerMutationInput, McpServerSnapshot, McpToolSnapshot, McpTransportDefinition,
    McpTrustApprovalState, McpWorkingDirectory,
};
use crate::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionEffectLease, RuntimeActionProposal, RuntimeApprovalDecision,
    RuntimeAuthorization, RuntimeEffectResult, RuntimeExecutionReceipt, RuntimeGatewayDecision,
};
use crate::runtime::broker_worker::{BrokerActionApplication, BrokerActionContext};
use crate::runtime::dispatch::DispatchIdentity;
use crate::runtime::opencode::C4OS_ACTION_PROPOSAL_TOOL;
use crate::runtime::opencode_broker::{
    ActiveBrokerContextResolver, InstalledBrokerFacilityRegistry, OpenCodeBrokerPump,
    OpenCodeBrokerPumpConfig, OpenCodeBrokerPumpError, OpenCodeBrokerPumpOutcome,
};
use crate::runtime::opencode_sdk::{BrokerDecision, OpenCodeSdkBroker};
use crate::runtime::supervisor::RuntimeKind;
use crate::security::authorization::{ApprovalAnswer, LiveAuthorityState};
use crate::security::gateway::{ActionGateway, ExecutionPermit, NormalizedActionResult};
use crate::security::policy::{ActionRequestOrigin, PolicyConfiguration};

const TIMEOUT: Duration = Duration::from_millis(50);

#[test]
fn production_mcp_cancellation_registry_targets_exact_server_and_credential_bindings() {
    let registry = ProductionMcpCancellationRegistry::default();
    let first = McpCancellation::default();
    let second = McpCancellation::default();
    registry
        .register("ticket-1".into(), "server-1".into(), first.clone())
        .expect("register first cancellation");
    registry
        .register("ticket-2".into(), "server-2".into(), second.clone())
        .expect("register second cancellation");

    assert_eq!(registry.quiesce_server("server-1"), 1);
    assert!(first.is_cancelled());
    assert!(!second.is_cancelled());

    let blocked = McpCancellation::default();
    assert!(
        registry
            .register("ticket-blocked".into(), "server-1".into(), blocked.clone(),)
            .is_err()
    );
    registry.allow_server("server-1");
    registry
        .register("ticket-allowed".into(), "server-1".into(), blocked.clone())
        .expect("registration after lifecycle re-enable");

    registry.complete("ticket-1");
    registry.complete("ticket-allowed");
    assert_eq!(registry.quiesce_server("server-1"), 0);
    assert!(registry.cancel_ticket("ticket-2"));
    assert!(second.is_cancelled());
}

#[test]
fn stale_lifecycle_authority_cannot_clear_a_policy_transition_quiesce() {
    let temporary = TempDir::new().unwrap();
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("database");
    let runtime =
        RuntimeApplicationService::restore(Arc::new(database), 10).expect("runtime restore");
    let cancellations = ProductionMcpCancellationRegistry::default();
    let stale = runtime
        .current_direct_live_authority(1, 1)
        .expect("initial authority");
    cancellations.quiesce_server("server-policy-race");

    let configuration = runtime
        .raw_coordinator()
        .expect("coordinator")
        .policy_configuration()
        .clone();
    runtime
        .replace_policy_settings(0, 1, configuration, true, 11)
        .expect("policy transition");

    assert!(
        register_mcp_lifecycle_for_live_authority(
            &runtime,
            &cancellations,
            "server-policy-race",
            "lifecycle-stale".into(),
            stale,
        )
        .is_err()
    );
    assert!(
        cancellations
            .register(
                "ticket-stale".into(),
                "server-policy-race".into(),
                McpCancellation::default(),
            )
            .is_err(),
        "a lifecycle operation authorized before the transition must not clear quiesce"
    );

    let current = runtime
        .current_direct_live_authority(1, 1)
        .expect("current authority");
    let current_lifecycle = register_mcp_lifecycle_for_live_authority(
        &runtime,
        &cancellations,
        "server-policy-race",
        "lifecycle-current".into(),
        current,
    )
    .expect("current lifecycle authority");
    drop(current_lifecycle);
    cancellations
        .register(
            "ticket-current".into(),
            "server-policy-race".into(),
            McpCancellation::default(),
        )
        .expect("registration after current-authority enable");
}

#[derive(Default)]
struct CountingAuthority {
    begin: AtomicUsize,
    complete: AtomicUsize,
    fail: AtomicUsize,
    redact: AtomicUsize,
}

impl McpAuthority for CountingAuthority {
    type EffectLease = ();

    fn resolve_launch(
        &self,
        _server: &crate::mcp::McpServerSnapshot,
    ) -> Result<ResolvedMcpLaunch, McpError> {
        Ok(ResolvedMcpLaunch::default())
    }

    fn begin_authorized_effect(
        &self,
        _request: &McpAuthorityRequest<'_>,
        _now_ms: u64,
    ) -> Result<Self::EffectLease, McpError> {
        self.begin.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn redact_result(
        &self,
        _request: &McpAuthorityRequest<'_>,
        mut untrusted: Value,
    ) -> Result<Value, McpError> {
        self.redact.fetch_add(1, Ordering::SeqCst);
        if let Some(object) = untrusted.as_object_mut() {
            object.remove("token");
        }
        Ok(untrusted)
    }

    fn complete_authorized_effect(
        &self,
        _lease: Self::EffectLease,
        _status: McpAuthorityEffectStatus,
        _redacted_result: &Value,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        self.complete.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn fail_authorized_effect(
        &self,
        _lease: Self::EffectLease,
        _status: McpAuthorityEffectStatus,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        self.fail.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn revoke_server(
        &self,
        _server: &McpServerSnapshot,
        _reason: &str,
        _now_ms: u64,
    ) -> Result<(), McpError> {
        Ok(())
    }
}

struct CountingFactory {
    transport_calls: Arc<AtomicUsize>,
    block_calls: Arc<std::sync::atomic::AtomicBool>,
}

impl McpTransportFactory for CountingFactory {
    fn connect<'a>(
        &'a self,
        _definition: &'a McpTransportDefinition,
        _launch: ResolvedMcpLaunch,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Box<dyn McpConnection>, McpError>> {
        let calls = Arc::clone(&self.transport_calls);
        let block_calls = Arc::clone(&self.block_calls);
        Box::pin(async move {
            Ok(Box::new(CountingConnection::new(calls, block_calls)) as Box<dyn McpConnection>)
        })
    }
}

struct CountingConnection {
    handshake: McpHandshakeSnapshot,
    transport_calls: Arc<AtomicUsize>,
    block_calls: Arc<std::sync::atomic::AtomicBool>,
}

impl CountingConnection {
    fn new(
        transport_calls: Arc<AtomicUsize>,
        block_calls: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            handshake: McpHandshakeSnapshot {
                protocol_version: MCP_PROTOCOL_VERSION.into(),
                server_name: "fixture".into(),
                server_version: "1.0.0".into(),
                instructions_present: false,
                capabilities: McpCapabilitySnapshot {
                    tools: true,
                    ..McpCapabilitySnapshot::default()
                },
            },
            transport_calls,
            block_calls,
        }
    }
}

impl McpConnection for CountingConnection {
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
        let input_schema = echo_schema();
        let input_schema_sha256 = sha256_value(&input_schema);
        Box::pin(async move {
            Ok(vec![McpToolSnapshot {
                name: "echo".into(),
                title: Some("Echo".into()),
                description: None,
                input_schema,
                input_schema_sha256,
                output_schema: None,
                output_schema_sha256: None,
            }])
        })
    }

    fn list_resources<'a>(
        &'a self,
        _timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpResourceSnapshot>, McpError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn call_tool<'a>(
        &'a self,
        _name: &'a str,
        _arguments: Value,
        _timeout_ms: u64,
        _max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        self.transport_calls.fetch_add(1, Ordering::SeqCst);
        let block_calls = Arc::clone(&self.block_calls);
        Box::pin(async move {
            while block_calls.load(Ordering::SeqCst) {
                if cancellation.is_cancelled() {
                    return Err(McpError::Cancelled);
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Ok(McpRawResult {
                value: json!({ "value": "ok", "token": "must-not-cross" }),
                is_error: false,
                output_bytes: 48,
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
        Box::pin(async { Err(McpError::Denied) })
    }

    fn close(self: Box<Self>, _timeout_ms: u64) -> McpFuture<'static, Result<(), McpError>> {
        Box::pin(async { Ok(()) })
    }
}

struct CountingApplication {
    gateway: ActionGateway,
    proposals: Vec<RuntimeActionProposal>,
    propose_calls: usize,
    approval_calls: usize,
    execute_calls: usize,
    begin_calls: usize,
    complete_calls: usize,
}

impl CountingApplication {
    fn new(database: Arc<DatabaseActor>) -> Self {
        Self {
            gateway: ActionGateway::new(PolicyConfiguration::default(), database),
            proposals: Vec::new(),
            propose_calls: 0,
            approval_calls: 0,
            execute_calls: 0,
            begin_calls: 0,
            complete_calls: 0,
        }
    }
}

impl BrokerActionApplication for CountingApplication {
    type Error = ();

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error> {
        self.propose_calls += 1;
        self.proposals.push(proposal.clone());
        RuntimeActionBridge::new(&mut self.gateway)
            .propose(proposal, now_ms)
            .map_err(|_| ())
    }

    fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, Self::Error> {
        self.approval_calls += 1;
        RuntimeActionBridge::new(&mut self.gateway)
            .answer_approval(prompt_id, answer, now_ms)
            .map_err(|_| ())
    }

    fn execute_runtime_action<F>(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: F,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>
    where
        F: FnOnce(ExecutionPermit) -> NormalizedActionResult,
    {
        self.execute_calls += 1;
        RuntimeActionBridge::new(&mut self.gateway)
            .execute(authorization, live, now_ms, effect)
            .map_err(|_| ())
    }

    fn begin_runtime_action_effect(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<RuntimeActionEffectLease, Self::Error> {
        self.begin_calls += 1;
        RuntimeActionBridge::new(&mut self.gateway)
            .begin_effect(authorization, live, now_ms)
            .map_err(|_| ())
    }

    fn complete_runtime_action_effect(
        &mut self,
        lease: RuntimeActionEffectLease,
        result: RuntimeEffectResult,
        _now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        self.complete_calls += 1;
        RuntimeActionBridge::new(&mut self.gateway)
            .complete_effect(lease, result)
            .map_err(|_| ())
    }

    fn complete_runtime_action_effect_retryable(
        &mut self,
        lease: &mut RuntimeActionEffectLease,
        result: RuntimeEffectResult,
        _now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        self.complete_calls += 1;
        RuntimeActionBridge::new(&mut self.gateway)
            .complete_effect_retryable(lease, result)
            .map_err(|_| ())
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.gateway.cancel_run(run_id, now_ms).map_err(|_| ())?;
        Ok(())
    }
}

struct BrokerPeer {
    worker: UnixStream,
    reader: BufReader<UnixStream>,
}

impl BrokerPeer {
    fn pair() -> (OpenCodeSdkBroker, Self) {
        let (broker, worker) = OpenCodeSdkBroker::authenticated_pair(4).expect("broker pair");
        worker
            .set_read_timeout(Some(TIMEOUT))
            .expect("response timeout");
        let reader = BufReader::new(worker.try_clone().expect("reader clone"));
        (broker, Self { worker, reader })
    }

    fn proposal(&mut self, target: &str) {
        self.proposal_with_correlation(target, "mcp-call-1");
    }

    fn proposal_with_correlation(&mut self, target: &str, correlation_id: &str) {
        self.send(json!({
            "schemaVersion": 1,
            "kind": "proposal",
            "correlationId": correlation_id,
            "tool": C4OS_ACTION_PROPOSAL_TOOL,
            "sessionId": "native-session-1",
            "messageId": "native-message-1",
            "processGeneration": 4,
            "payload": {
                "operation": "mcp.call-tool",
                "target": target,
                "arguments": { "value": "hello" },
            },
        }));
    }

    fn send(&mut self, frame: Value) {
        let mut encoded = serde_json::to_vec(&frame).expect("broker frame");
        encoded.push(b'\n');
        self.worker.write_all(&encoded).expect("write frame");
    }

    fn response(&mut self) -> Value {
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("response line");
        assert!(!line.is_empty());
        serde_json::from_str(&line).expect("response JSON")
    }

    fn assert_no_response(&mut self) {
        self.worker
            .set_read_timeout(Some(Duration::from_millis(20)))
            .expect("short response timeout");
        let mut line = String::new();
        let error = self
            .reader
            .read_line(&mut line)
            .expect_err("a second response must not exist");
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }
}

fn echo_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "value": { "type": "string" } },
        "required": ["value"],
        "additionalProperties": false,
    })
}

fn sha256_value(value: &Value) -> String {
    let mut output = String::from("sha256:");
    for byte in Sha256::digest(serde_json::to_vec(value).expect("test JSON")) {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("write digest");
    }
    output
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

fn context(mcp_turn: crate::mcp::McpTurnSnapshot) -> BrokerActionContext {
    BrokerActionContext {
        dispatch: DispatchIdentity {
            workspace_id: "workspace-1".into(),
            environment_id: "local".into(),
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            attempt_id: "run-1".into(),
            correlation_id: "run-correlation-1".into(),
            runtime_id: "opencode-1".into(),
            runtime_kind: RuntimeKind::OpenCode,
            adapter_version: "1.0.0".into(),
            native_version: "1.18.3".into(),
            process_generation: 4,
        },
        native_session_id: "native-session-1".into(),
        native_message_id: "native-message-1".into(),
        eligible_tool_ids: BTreeSet::from([C4OS_ACTION_PROPOSAL_TOOL.into()]),
        request_origin: ActionRequestOrigin::NaturalLanguageChat,
        configuration_version: 7,
        policy_version: 9,
        revocation_epoch: 2,
        mcp_turn: Some(mcp_turn),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preauthorized_mcp_tool_crosses_one_outer_gateway_and_returns_one_native_result() {
    let temporary = TempDir::new().expect("temporary database");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let authority = Arc::new(CountingAuthority::default());
    let transport_calls = Arc::new(AtomicUsize::new(0));
    let block_calls = Arc::new(AtomicBool::new(true));
    let factory = Arc::new(CountingFactory {
        transport_calls: Arc::clone(&transport_calls),
        block_calls: Arc::clone(&block_calls),
    });
    let repository = Arc::new(DatabaseMcpRepository::new(Arc::clone(&database), None));
    let mut service =
        McpService::restore(repository, Arc::clone(&authority), Arc::clone(&factory), 1)
            .expect("restore MCP service");
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
                prompt_id: "approval:fixture".into(),
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
            "approval:fixture",
            &action_binding_sha256,
            &definition_sha256,
            4,
        )
        .expect("trust definition");
    service
        .enable_server(
            &McpServerMutationInput {
                expected_generation: snapshot.generation,
                server_id: "fixture".into(),
            },
            5,
        )
        .await
        .expect("enable MCP server");
    let discovery_redactions = authority.redact.load(Ordering::SeqCst);
    let turn = service
        .turn_snapshot("workspace-1", "project-1", "session-1", 10)
        .expect("turn MCP snapshot");
    let target = turn.tools[0].target_id.clone();
    let service = Arc::new(tokio::sync::Mutex::new(service));

    let facilities = InstalledBrokerFacilityRegistry::new();
    let cancellations = ProductionMcpCancellationRegistry::default();
    facilities
        .install_deferred(Box::new(ProductionMcpDeferredFacility::new(
            Arc::clone(&service),
            cancellations.clone(),
            super::mcp::production_sampling::McpSamplingParentRegistry::default(),
        )))
        .expect("install production MCP facility");
    let (broker, mut peer) = BrokerPeer::pair();
    let mut pump = OpenCodeBrokerPump::attach_authenticated(
        broker,
        facilities,
        OpenCodeBrokerPumpConfig {
            receive_timeout: Duration::from_millis(5),
            response_timeout: TIMEOUT,
            max_pending_approvals: 4,
            max_terminal_requests: 16,
        },
    )
    .expect("broker pump");
    let trusted = context(turn);
    let mut resolver = ActiveBrokerContextResolver::new();
    resolver
        .activate(trusted, 1, 1_000)
        .expect("activate context");
    let mut application = CountingApplication::new(Arc::clone(&database));

    peer.proposal(&target);
    let pending = pump
        .pump_one(&mut application, &mut resolver, 20)
        .expect("pending outer approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("MCP action must retain outer approval");
    };
    let started = pump
        .answer_approval(
            &mut application,
            &mut resolver,
            "mcp-call-1",
            &prompt_id,
            ApprovalAnswer::Allow,
            21,
        )
        .expect("start production deferred MCP call");
    assert!(matches!(
        started,
        OpenCodeBrokerPumpOutcome::EffectRunning { ref correlation_id }
            if correlation_id == "mcp-call-1"
    ));

    let mut observed_executing = false;
    for _ in 0..100 {
        let current = service.lock().await.snapshot();
        if current.servers[0].lifecycle == crate::mcp::McpLifecycle::Executing {
            observed_executing = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    assert!(
        observed_executing,
        "production transport I/O must not retain the service mutex"
    );
    block_calls.store(false, Ordering::SeqCst);

    let mut responded = None;
    for now_ms in 22..222 {
        match pump.pump_one(&mut application, &mut resolver, now_ms) {
            Ok(outcome @ OpenCodeBrokerPumpOutcome::Responded { .. }) => {
                responded = Some(outcome);
                break;
            }
            Err(OpenCodeBrokerPumpError::Channel(_)) if !pump.is_sealed() => {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Ok(other) => panic!("unexpected pump outcome: {other:?}"),
            Err(error) => panic!("production MCP pump failed: {error:?}"),
        }
    }
    let Some(OpenCodeBrokerPumpOutcome::Responded {
        correlation_id,
        decision: BrokerDecision::Result(payload),
    }) = responded
    else {
        panic!("production MCP result did not settle");
    };
    assert_eq!(correlation_id, "mcp-call-1");
    assert_eq!(payload, json!({ "value": "ok" }));
    assert!(!pump.is_sealed());
    assert_eq!(pump.pending_count(), 0);

    let response = peer.response();
    assert_eq!(response["status"], "result");
    assert_eq!(response["payload"], json!({ "value": "ok" }));
    assert!(
        !serde_json::to_string(&response)
            .unwrap()
            .contains("must-not-cross")
    );

    block_calls.store(true, Ordering::SeqCst);
    peer.proposal_with_correlation(&target, "mcp-call-2");
    let pending = pump
        .pump_one(&mut application, &mut resolver, 40)
        .expect("second pending outer approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("second MCP action must retain outer approval");
    };
    pump.answer_approval(
        &mut application,
        &mut resolver,
        "mcp-call-2",
        &prompt_id,
        ApprovalAnswer::Allow,
        41,
    )
    .expect("start second production MCP call");
    let executing = loop {
        let current = service.lock().await.snapshot();
        if current.servers[0].lifecycle == crate::mcp::McpLifecycle::Executing {
            break current;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    };
    assert_eq!(cancellations.quiesce_server("fixture"), 1);
    let disabled = service
        .lock()
        .await
        .disable_server(
            &McpServerMutationInput {
                expected_generation: executing.generation,
                server_id: "fixture".into(),
            },
            42,
        )
        .await
        .expect("disable executing server");
    assert_eq!(
        disabled.servers[0].lifecycle,
        crate::mcp::McpLifecycle::Disabled
    );

    let mut cancelled_response = None;
    for now_ms in 43..243 {
        match pump.pump_one(&mut application, &mut resolver, now_ms) {
            Ok(outcome @ OpenCodeBrokerPumpOutcome::Responded { .. }) => {
                cancelled_response = Some(outcome);
                break;
            }
            Err(OpenCodeBrokerPumpError::Channel(_)) if !pump.is_sealed() => {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Ok(other) => panic!("unexpected cancellation outcome: {other:?}"),
            Err(error) => panic!("cancelled production MCP pump failed: {error:?}"),
        }
    }
    let Some(OpenCodeBrokerPumpOutcome::Responded {
        correlation_id,
        decision: BrokerDecision::Result(payload),
    }) = cancelled_response
    else {
        panic!("cancelled production MCP result did not settle");
    };
    assert_eq!(correlation_id, "mcp-call-2");
    assert_eq!(
        payload,
        json!({ "error": { "code": "cancelled_by_lifecycle_change" } })
    );
    assert_eq!(
        service.lock().await.snapshot().servers[0].lifecycle,
        crate::mcp::McpLifecycle::Disabled
    );
    let cancelled_wire_response = peer.response();
    assert_eq!(cancelled_wire_response["status"], "result");
    assert_eq!(cancelled_wire_response["payload"], payload);
    peer.assert_no_response();

    assert_eq!(application.propose_calls, 2);
    assert_eq!(application.approval_calls, 2);
    assert_eq!(application.execute_calls, 0);
    assert_eq!(application.begin_calls, 2);
    assert_eq!(application.complete_calls, 2);
    assert_eq!(application.proposals.len(), 2);
    assert_eq!(authority.begin.load(Ordering::SeqCst), 0);
    assert_eq!(authority.complete.load(Ordering::SeqCst), 0);
    assert_eq!(authority.fail.load(Ordering::SeqCst), 0);
    assert_eq!(
        authority.redact.load(Ordering::SeqCst),
        discovery_redactions + 1
    );
    assert_eq!(transport_calls.load(Ordering::SeqCst), 2);

    let action_id = &application.proposals[0].action.action_id;
    let records = database
        .security_records(SnapshotQuery::new(100).expect("security query"))
        .expect("security records");
    assert_eq!(
        records
            .iter()
            .filter(|record| record.action_id == *action_id
                && record.record_kind == "action-result"
                && record.state == "succeeded")
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.action_id == *action_id
                && record.record_kind == "action-intent"
                && record.state == "effect-finished")
            .count(),
        1
    );
    let events = database
        .mcp_event_page(None, SnapshotQuery::new(100).expect("MCP query"))
        .expect("MCP events")
        .events;
    let completed = events
        .iter()
        .filter(|event| {
            event.server_id == "fixture"
                && event.event_kind == "tool.completed"
                && event.target.as_deref() == Some("echo")
                && event.result == "succeeded"
        })
        .collect::<Vec<_>>();
    assert_eq!(completed.len(), 1);
}
