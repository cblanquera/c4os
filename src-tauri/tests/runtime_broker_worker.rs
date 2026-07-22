#![cfg(unix)]

use std::collections::BTreeSet;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionEffectLease, RuntimeActionProposal, RuntimeApprovalDecision,
    RuntimeAuthorization, RuntimeEffectResult, RuntimeExecutionReceipt, RuntimeGatewayDecision,
    RuntimeIntentIdentity,
};
use c4os_lib::runtime::broker_worker::{
    AuthenticatedBrokerEvent, BrokerActionApplication, BrokerActionClassifier, BrokerActionContext,
    BrokerActionWorker, BrokerApprovalAnswer, BrokerClassificationError, BrokerDeferredStart,
    BrokerDeferredTicket, BrokerEffectExecutor, BrokerWorkerError, BrokerWorkerOutcome,
    ResolvedBrokerAction, RuntimeBrokerApprovalAnswer, RuntimeBrokerWorkerOutcome,
};
use c4os_lib::runtime::dispatch::DispatchIdentity;
use c4os_lib::runtime::opencode::{C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL};
use c4os_lib::runtime::opencode_sdk::{BrokerDecision, OpenCodeSdkBroker, OpenCodeSdkError};
use c4os_lib::runtime::supervisor::RuntimeKind;
use c4os_lib::security::authorization::{
    ApprovalAnswer, CanonicalAction, CanonicalRisk, LiveAuthorityState,
};
use c4os_lib::security::gateway::{
    ActionGateway, ExecutionPermit, NormalizedActionResult, NormalizedActionStatus,
};
use c4os_lib::security::policy::{
    ActionEffect, ActionRequestOrigin, ActionReversibility, ActionScope, ActionSensitivity,
    ActionSurface, ClassificationConfidence, PolicyConfiguration, RepositoryState,
};
use serde_json::{Map, Value, json};
use tempfile::TempDir;

const TIMEOUT: Duration = Duration::from_secs(1);

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn context() -> BrokerActionContext {
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
        eligible_tool_ids: set([
            C4OS_ACTION_PROPOSAL_TOOL.into(),
            C4OS_RESOURCE_READ_TOOL.into(),
        ]),
        request_origin: ActionRequestOrigin::NaturalLanguageChat,
        configuration_version: 7,
        policy_version: 9,
        revocation_epoch: 2,
        mcp_turn: None,
    }
}

fn pi_context() -> BrokerActionContext {
    let mut context = context();
    context.dispatch.runtime_id = "pi-1".into();
    context.dispatch.runtime_kind = RuntimeKind::Pi;
    context.dispatch.native_version = "0.80.10".into();
    context.native_session_id = "session-1".into();
    context.native_message_id = "pi-tool-1".into();
    context
}

fn pi_intent() -> RuntimeIntentIdentity {
    RuntimeIntentIdentity {
        binding_sha256: format!("sha256:{}", "7".repeat(64)),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        run_id: "run-1".into(),
        correlation_id: "run-correlation-1".into(),
        runtime_id: "pi-1".into(),
        process_generation: 4,
        native_request_id: "pi-tool-1".into(),
        native_tool: C4OS_RESOURCE_READ_TOOL.into(),
    }
}

#[derive(Clone, Copy)]
enum ClassifierMode {
    Normal,
    SandboxDenied,
    Ambiguous,
}

struct FixtureClassifier {
    mode: ClassifierMode,
}

impl FixtureClassifier {
    fn action(&self) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if matches!(self.mode, ClassifierMode::Ambiguous) {
            return Err(BrokerClassificationError::Ambiguous);
        }
        Ok(ResolvedBrokerAction {
            surface: ActionSurface::Desktop,
            effects: set([ActionEffect::Control]),
            scope: ActionScope::Workspace,
            sensitivity: ActionSensitivity::Ordinary,
            reversibility: ActionReversibility::Reversible,
            repository_state: RepositoryState::NotApplicable,
            inside_active_project: false,
            canonical_target: "desktop:main-window".into(),
            target_version: "window-generation:3".into(),
            normalized_arguments: Map::from_iter([(
                "windowId".into(),
                Value::String("main-window".into()),
            )]),
            trusted_root: true,
            explicit_scope_grant: false,
            sandbox_allows: !matches!(self.mode, ClassifierMode::SandboxDenied),
            declaration_exceeded: false,
            confidence: ClassificationConfidence::Known,
            risk: CanonicalRisk::Medium,
            plugin_or_mcp_id: None,
        })
    }

    fn mcp_action(&self) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if matches!(self.mode, ClassifierMode::Ambiguous) {
            return Err(BrokerClassificationError::Ambiguous);
        }
        Ok(ResolvedBrokerAction {
            surface: ActionSurface::Process,
            effects: set([ActionEffect::Execute]),
            scope: ActionScope::ExternalLocal,
            sensitivity: ActionSensitivity::Ordinary,
            reversibility: ActionReversibility::Destructive,
            repository_state: RepositoryState::NotApplicable,
            inside_active_project: false,
            canonical_target: "mcp-tool:fixture-echo".into(),
            target_version: format!("sha256:{}", "8".repeat(64)),
            normalized_arguments: Map::from_iter([
                ("serverId".into(), Value::String("fixture".into())),
                ("toolName".into(), Value::String("echo".into())),
            ]),
            trusted_root: false,
            explicit_scope_grant: false,
            sandbox_allows: !matches!(self.mode, ClassifierMode::SandboxDenied),
            declaration_exceeded: false,
            confidence: ClassificationConfidence::Known,
            risk: CanonicalRisk::High,
            plugin_or_mcp_id: Some(format!("mcp:{}", "9".repeat(64))),
        })
    }

    fn resource(&self) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if matches!(self.mode, ClassifierMode::Ambiguous) {
            return Err(BrokerClassificationError::Ambiguous);
        }
        Ok(ResolvedBrokerAction {
            surface: ActionSurface::C4os,
            effects: set([ActionEffect::Read]),
            scope: ActionScope::Workspace,
            sensitivity: ActionSensitivity::Ordinary,
            reversibility: ActionReversibility::Reversible,
            repository_state: RepositoryState::NotApplicable,
            inside_active_project: true,
            canonical_target: "c4os:workspace-summary".into(),
            target_version: "workspace-generation:12".into(),
            normalized_arguments: Map::new(),
            trusted_root: true,
            explicit_scope_grant: false,
            sandbox_allows: !matches!(self.mode, ClassifierMode::SandboxDenied),
            declaration_exceeded: false,
            confidence: ClassificationConfidence::Known,
            risk: CanonicalRisk::Low,
            plugin_or_mcp_id: None,
        })
    }
}

impl BrokerActionClassifier for FixtureClassifier {
    fn resolve_resource(
        &mut self,
        _context: &BrokerActionContext,
        resource: &str,
        selector: Option<&str>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if resource != "workspace.summary" || selector.is_some_and(|value| value != "active") {
            return Err(BrokerClassificationError::Unsupported);
        }
        self.resource()
    }

    fn resolve_action(
        &mut self,
        _context: &BrokerActionContext,
        operation: &str,
        target: &str,
        arguments: &Map<String, Value>,
    ) -> Result<ResolvedBrokerAction, BrokerClassificationError> {
        if operation == "mcp.call-tool"
            && target == "mcp-tool:fixture-echo"
            && arguments.get("value").and_then(Value::as_str) == Some("hello")
            && arguments.len() == 1
        {
            return self.mcp_action();
        }
        if operation == "window.focus" && target == "main" && arguments.is_empty() {
            return self.action();
        }
        Err(BrokerClassificationError::Unsupported)
    }
}

struct TestApplication {
    _temporary: TempDir,
    gateway: ActionGateway,
    proposed: Vec<RuntimeActionProposal>,
    cancel_calls: usize,
    completed_statuses: Vec<NormalizedActionStatus>,
    fail_retryable_once: bool,
}

impl TestApplication {
    fn new() -> Self {
        let temporary = TempDir::new().expect("temporary database root");
        let (database, _) =
            DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("database");
        Self {
            _temporary: temporary,
            gateway: ActionGateway::new(PolicyConfiguration::default(), Arc::new(database)),
            proposed: Vec::new(),
            cancel_calls: 0,
            completed_statuses: Vec::new(),
            fail_retryable_once: false,
        }
    }
}

impl BrokerActionApplication for TestApplication {
    type Error = ();

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error> {
        self.proposed.push(proposal.clone());
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
        self.completed_statuses.push(result.normalized.status);
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
        if self.fail_retryable_once {
            self.fail_retryable_once = false;
            return Err(());
        }
        self.completed_statuses.push(result.normalized.status);
        RuntimeActionBridge::new(&mut self.gateway)
            .complete_effect_retryable(lease, result)
            .map_err(|_| ())
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.cancel_calls += 1;
        self.gateway.cancel_run(run_id, now_ms).map_err(|_| ())?;
        Ok(())
    }
}

#[derive(Default)]
struct TestExecutor {
    calls: usize,
    actions: Vec<CanonicalAction>,
}

#[derive(Default)]
struct DeferredTestExecutor {
    starts: usize,
    cancellations: usize,
    abandonments: usize,
    ready: bool,
}

impl BrokerEffectExecutor for DeferredTestExecutor {
    fn execute(&mut self, _permit: ExecutionPermit) -> NormalizedActionResult {
        panic!("MCP effects must use the deferred facility path")
    }

    fn start_deferred(&mut self, _permit: ExecutionPermit) -> BrokerDeferredStart {
        self.starts += 1;
        BrokerDeferredStart::Started(
            BrokerDeferredTicket::new(format!("fixture-ticket-{}", self.starts))
                .expect("valid deferred ticket"),
        )
    }

    fn poll_deferred(&mut self, ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult> {
        if !self.ready || !ticket.as_str().starts_with("fixture-ticket-") {
            return None;
        }
        self.ready = false;
        Some(RuntimeEffectResult::normalized(NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "mcp-tool-succeeded".into(),
            exit_code: None,
            changed_targets: vec!["mcp-tool:fixture-echo".into()],
            output_sha256: Some(format!("sha256:{}", "a".repeat(64))),
            completed_at_ms: 30,
        }))
    }

    fn cancel_deferred(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        if !ticket.as_str().starts_with("fixture-ticket-") {
            return false;
        }
        self.cancellations += 1;
        true
    }

    fn abandon_deferred(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        if !ticket.as_str().starts_with("fixture-ticket-") {
            return false;
        }
        self.abandonments += 1;
        true
    }
}

impl BrokerEffectExecutor for TestExecutor {
    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult {
        self.calls += 1;
        self.actions.push(permit.action().clone());
        NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "worker-completed".into(),
            exit_code: None,
            changed_targets: vec!["workspace:/private/customer-record.txt".into()],
            output_sha256: Some(format!("sha256:{:064x}", 1)),
            completed_at_ms: 30,
        }
    }
}

struct AuthenticatedBrokerFixture {
    broker: OpenCodeSdkBroker,
    worker: UnixStream,
}

impl AuthenticatedBrokerFixture {
    fn new() -> Self {
        let (broker, worker) = OpenCodeSdkBroker::authenticated_pair(4).expect("broker pair");
        Self { broker, worker }
    }

    fn proposal(
        &mut self,
        correlation_id: &str,
        tool: &str,
        payload: Value,
    ) -> AuthenticatedBrokerEvent {
        self.proposal_with_session(correlation_id, tool, payload, "native-session-1")
    }

    fn proposal_with_session(
        &mut self,
        correlation_id: &str,
        tool: &str,
        payload: Value,
        native_session_id: &str,
    ) -> AuthenticatedBrokerEvent {
        self.write(json!({
            "schemaVersion": 1,
            "kind": "proposal",
            "correlationId": correlation_id,
            "tool": tool,
            "sessionId": native_session_id,
            "messageId": "native-message-1",
            "processGeneration": 4,
            "payload": payload,
        }));
        AuthenticatedBrokerEvent::receive(&mut self.broker, TIMEOUT)
            .expect("authenticated proposal")
    }

    fn cancellation(&mut self, correlation_id: &str, tool: &str) -> AuthenticatedBrokerEvent {
        self.write(json!({
            "schemaVersion": 1,
            "kind": "cancel",
            "correlationId": correlation_id,
            "tool": tool,
            "sessionId": "native-session-1",
            "messageId": "native-message-1",
            "processGeneration": 4,
        }));
        AuthenticatedBrokerEvent::receive(&mut self.broker, TIMEOUT)
            .expect("authenticated cancellation")
    }

    fn settle(&mut self, outcome: &BrokerWorkerOutcome) {
        if let BrokerWorkerOutcome::Respond {
            correlation_id,
            decision,
        } = outcome
        {
            self.broker
                .respond(correlation_id, decision.clone(), TIMEOUT)
                .expect("broker response");
        }
    }

    fn write(&mut self, frame: Value) {
        let mut encoded = serde_json::to_vec(&frame).expect("broker frame");
        encoded.push(b'\n');
        self.worker.write_all(&encoded).expect("write broker frame");
    }
}

fn action_payload() -> Value {
    json!({"operation": "window.focus", "target": "main"})
}

fn read_payload() -> Value {
    json!({"resource": "workspace.summary", "selector": "active"})
}

fn mcp_payload() -> Value {
    json!({
        "operation": "mcp.call-tool",
        "target": "mcp-tool:fixture-echo",
        "arguments": { "value": "hello" },
    })
}

fn pi_mcp_context(native_request_id: &str) -> BrokerActionContext {
    let mut context = pi_context();
    context.native_message_id = native_request_id.into();
    context
}

fn pi_mcp_intent(native_request_id: &str, digest_seed: char) -> RuntimeIntentIdentity {
    let mut intent = pi_intent();
    intent.binding_sha256 = format!("sha256:{}", digest_seed.to_string().repeat(64));
    intent.native_request_id = native_request_id.into();
    intent.native_tool = C4OS_ACTION_PROPOSAL_TOOL.into();
    intent
}

fn reason(outcome: &BrokerWorkerOutcome) -> Option<&str> {
    match outcome {
        BrokerWorkerOutcome::Respond {
            decision: BrokerDecision::Denied { reason_code },
            ..
        }
        | BrokerWorkerOutcome::ObservedCancellation {
            decision: BrokerDecision::Denied { reason_code },
            ..
        } => Some(reason_code),
        _ => None,
    }
}

#[test]
fn policy_denial_happens_before_any_worker_effect() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::SandboxDenied,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();

    let event = broker.proposal(
        "request-denied",
        C4OS_ACTION_PROPOSAL_TOOL,
        action_payload(),
    );
    let outcome = worker
        .accept_authenticated_event(&mut application, &mut executor, event, context(), 10)
        .expect("policy decision");

    assert_eq!(reason(&outcome), Some("policy-denied"));
    assert_eq!(executor.calls, 0);
    assert_eq!(application.proposed.len(), 1);
    broker.settle(&outcome);
}

#[test]
fn trusted_resource_read_is_classified_and_executes_only_through_the_gateway() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();

    let event = broker.proposal("request-read", C4OS_RESOURCE_READ_TOOL, read_payload());
    let outcome = worker
        .accept_authenticated_event(&mut application, &mut executor, event, context(), 10)
        .expect("trusted read receipt");

    assert!(matches!(
        outcome,
        BrokerWorkerOutcome::Respond {
            decision: BrokerDecision::Result(_),
            ..
        }
    ));
    assert_eq!(executor.calls, 1);
    assert_eq!(application.proposed.len(), 1);
    assert_eq!(application.proposed[0].facts.action_kind, "resource.read");
    assert_eq!(application.proposed[0].action.tool, C4OS_RESOURCE_READ_TOOL);
    assert_eq!(
        application.proposed[0].action.canonical_target,
        "c4os:workspace-summary"
    );
    assert!(
        application.proposed[0]
            .action
            .requested_authority
            .contains("extensions-c4os.read")
    );
    broker.settle(&outcome);
}

#[test]
fn authenticated_peer_cannot_invoke_a_broker_tool_omitted_from_the_core_context() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();
    let mut trusted = context();
    trusted.eligible_tool_ids = set([C4OS_ACTION_PROPOSAL_TOOL.into()]);

    let event = broker.proposal(
        "request-omitted-tool",
        C4OS_RESOURCE_READ_TOOL,
        read_payload(),
    );
    let outcome = worker
        .accept_authenticated_event(&mut application, &mut executor, event, trusted, 10)
        .expect("omitted broker tool denial");

    assert_eq!(reason(&outcome), Some("unsupported-broker-tool"));
    assert_eq!(executor.calls, 0);
    assert!(application.proposed.is_empty());
    broker.settle(&outcome);
}

#[test]
fn pi_runtime_intent_returns_the_sealed_gateway_receipt_instead_of_a_transport_summary() {
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();

    let outcome = worker
        .accept_runtime_intent(
            &mut application,
            &mut executor,
            pi_intent(),
            read_payload(),
            pi_context(),
            10,
        )
        .expect("Pi gateway receipt");

    let RuntimeBrokerWorkerOutcome::Executed {
        native_request_id,
        receipt,
    } = outcome
    else {
        panic!("trusted Pi read must execute through the sealed gateway");
    };
    assert_eq!(native_request_id, "pi-tool-1");
    assert_eq!(receipt.result().status, NormalizedActionStatus::Succeeded);
    assert_eq!(executor.calls, 1);
    assert_eq!(application.proposed.len(), 1);
    assert_eq!(application.proposed[0].intent, pi_intent());
}

#[test]
fn approval_resume_executes_the_exact_bound_action_and_redacts_the_result() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();
    let trusted = context();

    let event = broker.proposal(
        "request-approved",
        C4OS_ACTION_PROPOSAL_TOOL,
        action_payload(),
    );
    let pending = worker
        .accept_authenticated_event(&mut application, &mut executor, event, trusted.clone(), 10)
        .expect("pending approval");
    let BrokerWorkerOutcome::PendingApproval { prompt_id, .. } = &pending else {
        panic!("default policy must pause at approval");
    };
    assert_eq!(executor.calls, 0);

    let outcome = worker
        .answer_approval(
            &mut application,
            &mut executor,
            BrokerApprovalAnswer {
                correlation_id: "request-approved",
                prompt_id,
                answer: ApprovalAnswer::Allow,
                current_context: &trusted,
                now_ms: 20,
            },
        )
        .expect("approved effect receipt");
    assert_eq!(executor.calls, 1);
    assert_eq!(application.proposed.len(), 1);

    let proposal = &application.proposed[0];
    assert_eq!(proposal.intent.workspace_id, "workspace-1");
    assert_eq!(proposal.intent.session_id, "session-1");
    assert_eq!(proposal.intent.turn_id, "turn-1");
    assert_eq!(proposal.intent.run_id, "run-1");
    assert_eq!(proposal.intent.correlation_id, "run-correlation-1");
    assert_eq!(proposal.intent.runtime_id, "opencode-1");
    assert_eq!(proposal.intent.process_generation, 4);
    assert_eq!(proposal.intent.native_request_id, "request-approved");
    assert_eq!(proposal.facts.environment_id, "local");
    assert_eq!(proposal.action.environment_id, "local");
    assert_eq!(proposal.action.tool_call_id, "request-approved");
    assert_eq!(proposal.action.canonical_target, "desktop:main-window");
    assert!(
        proposal
            .action
            .requested_authority
            .contains("browser-desktop.control")
    );
    assert_eq!(
        proposal.action.arguments["broker"]["operation"],
        "window.focus"
    );
    assert!(
        proposal.action.arguments["c4osRuntimeIntentSha256"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert_eq!(executor.actions, vec![proposal.action.clone()]);

    let BrokerWorkerOutcome::Respond {
        decision: BrokerDecision::Result(result),
        ..
    } = &outcome
    else {
        panic!("a core execution receipt must produce the only success result");
    };
    let encoded = serde_json::to_string(result).expect("result JSON");
    assert_eq!(result["changedTargetCount"], 1);
    assert!(!encoded.contains("customer-record"));
    assert!(!encoded.contains("authorization"));
    assert!(!encoded.contains("c4osRuntimeIntentSha256"));
    broker.settle(&outcome);
}

#[test]
fn cancellation_burns_the_pending_approval_and_replay_cannot_execute() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();
    let trusted = context();

    let event = broker.proposal(
        "request-cancel",
        C4OS_ACTION_PROPOSAL_TOOL,
        action_payload(),
    );
    let pending = worker
        .accept_authenticated_event(&mut application, &mut executor, event, trusted.clone(), 10)
        .expect("pending approval");
    let BrokerWorkerOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("approval must be pending");
    };

    let cancellation = broker.cancellation("request-cancel", C4OS_ACTION_PROPOSAL_TOOL);
    let cancelled = worker
        .accept_authenticated_event(
            &mut application,
            &mut executor,
            cancellation,
            trusted.clone(),
            15,
        )
        .expect("cancelled request");
    assert!(matches!(
        cancelled,
        BrokerWorkerOutcome::ObservedCancellation {
            decision: BrokerDecision::Cancelled,
            ..
        }
    ));
    assert_eq!(application.cancel_calls, 1);
    assert_eq!(executor.calls, 0);
    assert_eq!(worker.pending_count(), 0);

    let replay = worker
        .answer_approval(
            &mut application,
            &mut executor,
            BrokerApprovalAnswer {
                correlation_id: "request-cancel",
                prompt_id: &prompt_id,
                answer: ApprovalAnswer::Allow,
                current_context: &trusted,
                now_ms: 20,
            },
        )
        .expect("replay denial");
    assert_eq!(reason(&replay), Some("broker-request-replayed"));
    assert_eq!(executor.calls, 0);
}

#[test]
fn stale_dispatch_identity_cancels_instead_of_resuming_effect() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();
    let trusted = context();

    let event = broker.proposal("request-stale", C4OS_ACTION_PROPOSAL_TOOL, action_payload());
    let pending = worker
        .accept_authenticated_event(&mut application, &mut executor, event, trusted.clone(), 10)
        .expect("pending approval");
    let BrokerWorkerOutcome::PendingApproval { prompt_id, .. } = &pending else {
        panic!("approval must be pending");
    };

    let mut stale = trusted;
    stale.dispatch.process_generation = 5;
    let outcome = worker
        .answer_approval(
            &mut application,
            &mut executor,
            BrokerApprovalAnswer {
                correlation_id: "request-stale",
                prompt_id,
                answer: ApprovalAnswer::Allow,
                current_context: &stale,
                now_ms: 20,
            },
        )
        .expect("stale denial");
    assert_eq!(reason(&outcome), Some("stale-runtime-identity"));
    assert_eq!(application.cancel_calls, 1);
    assert_eq!(executor.calls, 0);
    broker.settle(&outcome);
}

#[test]
fn pending_approval_and_terminal_tracking_are_bounded_and_fail_closed() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut worker = BrokerActionWorker::with_capacity(
        FixtureClassifier {
            mode: ClassifierMode::Normal,
        },
        1,
        4,
    )
    .expect("bounded worker");
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();

    let first = broker.proposal("request-one", C4OS_ACTION_PROPOSAL_TOOL, action_payload());
    let first = worker
        .accept_authenticated_event(&mut application, &mut executor, first, context(), 10)
        .expect("first pending approval");
    assert!(matches!(first, BrokerWorkerOutcome::PendingApproval { .. }));

    let second = broker.proposal("request-two", C4OS_ACTION_PROPOSAL_TOOL, action_payload());
    let second = worker
        .accept_authenticated_event(&mut application, &mut executor, second, context(), 11)
        .expect("capacity denial");
    assert_eq!(reason(&second), Some("pending-approval-capacity"));
    assert_eq!(worker.pending_count(), 1);
    assert_eq!(application.proposed.len(), 1);
    assert_eq!(executor.calls, 0);
    broker.settle(&second);
}

#[test]
fn unsupported_ambiguous_and_mismatched_requests_never_reach_the_gateway() {
    let mut broker = AuthenticatedBrokerFixture::new();
    let mut application = TestApplication::new();
    let mut executor = TestExecutor::default();

    let mut normal = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Normal,
    });
    let unsupported = broker.proposal(
        "request-unsupported",
        C4OS_ACTION_PROPOSAL_TOOL,
        json!({"operation": "shell.execute", "target": "main"}),
    );
    let unsupported = normal
        .accept_authenticated_event(&mut application, &mut executor, unsupported, context(), 10)
        .expect("unsupported denial");
    assert_eq!(reason(&unsupported), Some("unsupported-broker-operation"));
    broker.settle(&unsupported);

    let invalid = broker.proposal(
        "request-invalid",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "unexpected": true}),
    );
    let invalid = normal
        .accept_authenticated_event(&mut application, &mut executor, invalid, context(), 11)
        .expect("payload denial");
    assert_eq!(reason(&invalid), Some("invalid-broker-payload"));
    broker.settle(&invalid);

    let wrong_session = broker.proposal_with_session(
        "request-session",
        C4OS_RESOURCE_READ_TOOL,
        read_payload(),
        "native-session-other",
    );
    let wrong_session = normal
        .accept_authenticated_event(
            &mut application,
            &mut executor,
            wrong_session,
            context(),
            12,
        )
        .expect("session denial");
    assert_eq!(reason(&wrong_session), Some("native-session-mismatch"));
    broker.settle(&wrong_session);

    let mut ambiguous = BrokerActionWorker::new(FixtureClassifier {
        mode: ClassifierMode::Ambiguous,
    });
    let ambiguous_event = broker.proposal(
        "request-ambiguous",
        C4OS_ACTION_PROPOSAL_TOOL,
        action_payload(),
    );
    let ambiguous_outcome = ambiguous
        .accept_authenticated_event(
            &mut application,
            &mut executor,
            ambiguous_event,
            context(),
            13,
        )
        .expect("ambiguous denial");
    assert_eq!(
        reason(&ambiguous_outcome),
        Some("ambiguous-broker-operation")
    );
    broker.settle(&ambiguous_outcome);

    assert_eq!(application.proposed.len(), 0);
    assert_eq!(executor.calls, 0);
}

#[test]
fn authenticated_transport_rejects_process_substitution_before_worker_state() {
    let (mut broker, mut worker) = OpenCodeSdkBroker::authenticated_pair(4).expect("broker pair");
    let mut frame = serde_json::to_vec(&json!({
        "schemaVersion": 1,
        "kind": "proposal",
        "correlationId": "request-generation",
        "tool": C4OS_RESOURCE_READ_TOOL,
        "sessionId": "native-session-1",
        "messageId": "native-message-1",
        "processGeneration": 5,
        "payload": read_payload(),
    }))
    .expect("frame");
    frame.push(b'\n');
    worker.write_all(&frame).expect("write frame");

    assert!(matches!(
        AuthenticatedBrokerEvent::receive(&mut broker, TIMEOUT),
        Err(OpenCodeSdkError::InvalidFrame)
    ));
}

#[test]
fn deferred_pi_effect_capacity_and_late_cancellation_settlement_are_fail_closed() {
    let mut worker = BrokerActionWorker::with_capacity(
        FixtureClassifier {
            mode: ClassifierMode::Normal,
        },
        1,
        4,
    )
    .expect("bounded deferred worker");
    let mut application = TestApplication::new();
    let mut executor = DeferredTestExecutor::default();
    let first_context = pi_mcp_context("pi-mcp-1");

    let first = worker
        .accept_runtime_intent(
            &mut application,
            &mut executor,
            pi_mcp_intent("pi-mcp-1", 'a'),
            mcp_payload(),
            first_context.clone(),
            10,
        )
        .expect("first MCP proposal");
    let started = match first {
        RuntimeBrokerWorkerOutcome::PendingApproval { prompt_id, .. } => worker
            .answer_runtime_intent_approval(
                &mut application,
                &mut executor,
                RuntimeBrokerApprovalAnswer {
                    native_request_id: "pi-mcp-1",
                    prompt_id: &prompt_id,
                    answer: ApprovalAnswer::Allow,
                    current_context: &first_context,
                    now_ms: 20,
                },
            )
            .expect("start approved deferred MCP effect"),
        already_started @ RuntimeBrokerWorkerOutcome::EffectRunning { .. } => already_started,
        RuntimeBrokerWorkerOutcome::Denied { reason_code, .. } => {
            panic!("MCP execution was denied before deferred start: {reason_code}")
        }
        _ => panic!("MCP execution must be authorized or await explicit approval"),
    };
    assert!(matches!(
        started,
        RuntimeBrokerWorkerOutcome::EffectRunning { ref native_request_id }
            if native_request_id == "pi-mcp-1"
    ));
    assert_eq!(executor.starts, 1);
    assert_eq!(worker.pending_count(), 1);

    let second = worker
        .accept_runtime_intent(
            &mut application,
            &mut executor,
            pi_mcp_intent("pi-mcp-2", 'b'),
            mcp_payload(),
            pi_mcp_context("pi-mcp-2"),
            21,
        )
        .expect("capacity denial");
    assert!(matches!(
        second,
        RuntimeBrokerWorkerOutcome::Denied {
            ref native_request_id,
            ref reason_code,
        } if native_request_id == "pi-mcp-2" && reason_code == "pending-approval-capacity"
    ));
    assert_eq!(application.proposed.len(), 1);
    assert_eq!(executor.starts, 1, "capacity denial must precede job start");

    application
        .cancel_runtime_run("run-1", 22)
        .expect("terminalize exact run");
    assert_eq!(
        worker.cancel_runtime_deferred_for_run(&mut executor, "run-1"),
        1
    );
    assert_eq!(executor.cancellations, 1);
    executor.ready = true;
    let settled = worker
        .poll_runtime_deferred(&mut application, &mut executor, 30)
        .expect("late deferred settlement")
        .expect("ready late result");
    let RuntimeBrokerWorkerOutcome::SettledAfterCancellation {
        native_request_id,
        receipt,
    } = settled
    else {
        panic!("a terminal Pi run must receive administrative settlement only");
    };
    assert_eq!(native_request_id, "pi-mcp-1");
    assert_eq!(receipt.result().status, NormalizedActionStatus::Succeeded);
    assert_eq!(worker.pending_count(), 0);
}

#[test]
fn runtime_shutdown_drain_retries_unknown_settlement_without_losing_the_effect_lease() {
    let mut worker = BrokerActionWorker::with_capacity(
        FixtureClassifier {
            mode: ClassifierMode::Normal,
        },
        2,
        4,
    )
    .expect("bounded deferred worker");
    let mut application = TestApplication::new();
    let mut executor = DeferredTestExecutor::default();
    let trusted_context = pi_mcp_context("pi-mcp-drain");
    let proposal = worker
        .accept_runtime_intent(
            &mut application,
            &mut executor,
            pi_mcp_intent("pi-mcp-drain", 'd'),
            mcp_payload(),
            trusted_context.clone(),
            10,
        )
        .expect("MCP proposal");
    let started = match proposal {
        RuntimeBrokerWorkerOutcome::PendingApproval { prompt_id, .. } => worker
            .answer_runtime_intent_approval(
                &mut application,
                &mut executor,
                RuntimeBrokerApprovalAnswer {
                    native_request_id: "pi-mcp-drain",
                    prompt_id: &prompt_id,
                    answer: ApprovalAnswer::Allow,
                    current_context: &trusted_context,
                    now_ms: 20,
                },
            )
            .expect("start approved effect"),
        already_started @ RuntimeBrokerWorkerOutcome::EffectRunning { .. } => already_started,
        _ => panic!("unexpected MCP proposal outcome"),
    };
    assert!(matches!(
        started,
        RuntimeBrokerWorkerOutcome::EffectRunning { .. }
    ));
    assert_eq!(worker.pending_count(), 1);

    application.fail_retryable_once = true;
    assert!(matches!(
        worker.drain_deferred_unknown(&mut application, &mut executor, 29),
        Err(BrokerWorkerError::EffectStatusUnknown)
    ));
    assert_eq!(worker.pending_count(), 1);
    assert_eq!(executor.cancellations, 1);
    assert_eq!(executor.abandonments, 0);
    assert!(application.completed_statuses.is_empty());

    assert_eq!(
        worker
            .drain_deferred_unknown(&mut application, &mut executor, 30)
            .expect("drain deferred effect"),
        1
    );
    assert_eq!(executor.cancellations, 2);
    assert_eq!(executor.abandonments, 1);
    assert_eq!(worker.pending_count(), 0);
    assert_eq!(
        application.completed_statuses,
        vec![NormalizedActionStatus::UnknownAfterInterruption]
    );
}
