#![cfg(unix)]

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::mcp::{McpDefinitionSource, McpTransportKind, McpTurnSnapshot, McpTurnToolSnapshot};
use c4os_lib::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionEffectLease, RuntimeActionProposal, RuntimeApprovalDecision,
    RuntimeAuthorization, RuntimeEffectResult, RuntimeExecutionReceipt, RuntimeGatewayDecision,
};
use c4os_lib::runtime::broker_worker::{
    BrokerActionApplication, BrokerActionContext, BrokerDeferredStart, BrokerDeferredTicket,
};
use c4os_lib::runtime::dispatch::DispatchIdentity;
use c4os_lib::runtime::opencode::{C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL};
use c4os_lib::runtime::opencode_broker::{
    ActiveBrokerContextResolver, AuthenticatedAssistantMessageEvidence, BrokerContextRegistryError,
    FacilityRegistryError, InstalledBrokerClassification, InstalledBrokerFacility,
    InstalledBrokerFacilityRegistry, InstalledDeferredBrokerFacility, OpenCodeBrokerPump,
    OpenCodeBrokerPumpConfig, OpenCodeBrokerPumpError, OpenCodeBrokerPumpOutcome,
};
use c4os_lib::runtime::opencode_sdk::{BrokerDecision, OpenCodeSdkBroker};
use c4os_lib::runtime::supervisor::RuntimeKind;
use c4os_lib::security::authorization::{ApprovalAnswer, LiveAuthorityState};
use c4os_lib::security::gateway::{
    ActionGateway, ExecutionPermit, NormalizedActionResult, NormalizedActionStatus,
};
use c4os_lib::security::policy::{
    ActionEffect, ActionRequestOrigin, ActionReversibility, ActionScope, ActionSensitivity,
    ActionSurface, PolicyConfiguration, RepositoryState,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const TIMEOUT: Duration = Duration::from_secs(1);

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
        eligible_tool_ids: BTreeSet::from([
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

fn resolver() -> ActiveBrokerContextResolver {
    let resolver = ActiveBrokerContextResolver::new();
    resolver
        .activate(context(), 1, 1_000)
        .expect("activate broker context");
    resolver
}

fn resolver_for(context: BrokerActionContext) -> ActiveBrokerContextResolver {
    let resolver = ActiveBrokerContextResolver::new();
    resolver
        .activate(context, 1, 1_000)
        .expect("activate broker context");
    resolver
}

fn sha256_value(value: &Value) -> String {
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(serde_json::to_vec(value).expect("test JSON")) {
        use std::fmt::Write as _;
        write!(&mut digest, "{byte:02x}").expect("write digest");
    }
    digest
}

fn mcp_turn_snapshot() -> McpTurnSnapshot {
    let input_schema = json!({
        "type": "object",
        "properties": { "value": { "type": "string" } },
        "required": ["value"],
        "additionalProperties": false,
    });
    let input_schema_sha256 = sha256_value(&input_schema);
    let definition_sha256 = format!("sha256:{}", "a".repeat(64));
    let route = json!({
        "serverId": "fixture",
        "source": McpDefinitionSource::User,
        "lifecycleGeneration": 3,
        "definitionSha256": definition_sha256,
        "toolName": "echo",
        "inputSchemaSha256": input_schema_sha256,
        "outputSchemaSha256": null,
        "workspaceId": "workspace-1",
        "projectId": "project-1",
        "sessionId": "session-1",
    });
    let tool = McpTurnToolSnapshot {
        target_id: format!(
            "mcp-tool:{}",
            sha256_value(&route).trim_start_matches("sha256:")
        ),
        server_id: "fixture".into(),
        source: McpDefinitionSource::User,
        lifecycle_generation: 3,
        definition_sha256,
        transport_kind: McpTransportKind::Stdio,
        tool_name: "echo".into(),
        title: Some("Echo".into()),
        description: None,
        input_schema,
        input_schema_sha256,
        output_schema_sha256: None,
    };
    let catalog = json!({
        "serviceGeneration": 7,
        "capturedAtMs": 10,
        "workspaceId": "workspace-1",
        "projectId": "project-1",
        "sessionId": "session-1",
        "tools": [&tool],
        "truncated": false,
        "omittedToolCount": 0,
    });
    let sha256 = sha256_value(&catalog);
    let snapshot = McpTurnSnapshot {
        snapshot_id: format!("mcp-turn:{}", sha256.trim_start_matches("sha256:")),
        service_generation: 7,
        captured_at_ms: 10,
        workspace_id: "workspace-1".into(),
        project_id: "project-1".into(),
        session_id: "session-1".into(),
        tools: vec![tool],
        truncated: false,
        omitted_tool_count: 0,
        sha256,
    };
    snapshot.validate().expect("valid MCP turn snapshot");
    snapshot
}

fn mcp_context() -> BrokerActionContext {
    let mut context = context();
    context.mcp_turn = Some(mcp_turn_snapshot());
    context
}

fn assistant_evidence(message_id: &str) -> AuthenticatedAssistantMessageEvidence {
    AuthenticatedAssistantMessageEvidence {
        native_session_id: "native-session-1".into(),
        native_message_id: message_id.into(),
        role: "assistant".into(),
        parent_native_message_id: Some("native-message-1".into()),
        process_generation: 4,
    }
}

fn second_context() -> BrokerActionContext {
    let mut second = context();
    second.dispatch.turn_id = "turn-2".into();
    second.dispatch.attempt_id = "run-2".into();
    second.dispatch.correlation_id = "run-correlation-2".into();
    second.native_message_id = "native-message-2".into();
    second
}

fn publish_assistant_alias(resolver: &ActiveBrokerContextResolver, message_id: &str) {
    resolver
        .publish_authenticated_assistant_alias(&context(), assistant_evidence(message_id), 10)
        .expect("publish authenticated assistant alias");
}

fn classification(
    surface: ActionSurface,
    effect: ActionEffect,
    canonical_target: &str,
) -> InstalledBrokerClassification {
    InstalledBrokerClassification {
        surface,
        effects: BTreeSet::from([effect]),
        scope: ActionScope::Workspace,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: true,
        canonical_target: canonical_target.into(),
        normalized_arguments: Map::new(),
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}

struct TestFacility {
    version: Arc<Mutex<String>>,
    calls: Arc<AtomicUsize>,
}

impl InstalledBrokerFacility for TestFacility {
    fn current_target_version(&mut self) -> Option<String> {
        self.version.lock().ok().map(|version| version.clone())
    }

    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult {
        self.calls.fetch_add(1, Ordering::SeqCst);
        NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "installed-facility-completed".into(),
            exit_code: None,
            changed_targets: vec![permit.action().canonical_target.clone()],
            output_sha256: None,
            completed_at_ms: 30,
        }
    }
}

#[derive(Default)]
struct DeferredFacilityState {
    starts: usize,
    cancellations: usize,
    ready: bool,
}

struct TestDeferredFacility {
    state: Arc<Mutex<DeferredFacilityState>>,
}

impl InstalledDeferredBrokerFacility for TestDeferredFacility {
    fn start(&mut self, _permit: ExecutionPermit) -> BrokerDeferredStart {
        let mut state = self.state.lock().expect("deferred state");
        state.starts += 1;
        BrokerDeferredStart::Started(
            BrokerDeferredTicket::new(format!("opencode-mcp-ticket-{}", state.starts))
                .expect("valid ticket"),
        )
    }

    fn poll(&mut self, ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult> {
        let mut state = self.state.lock().expect("deferred state");
        if !state.ready || !ticket.as_str().starts_with("opencode-mcp-ticket-") {
            return None;
        }
        state.ready = false;
        Some(RuntimeEffectResult::normalized(NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "mcp-tool-succeeded".into(),
            exit_code: None,
            changed_targets: vec!["mcp-tool:fixture".into()],
            output_sha256: Some(format!("sha256:{}", "b".repeat(64))),
            completed_at_ms: 30,
        }))
    }

    fn cancel(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        if !ticket.as_str().starts_with("opencode-mcp-ticket-") {
            return false;
        }
        self.state.lock().expect("deferred state").cancellations += 1;
        true
    }

    fn abandon(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        self.cancel(ticket)
    }
}

struct TestApplication {
    _temporary: TempDir,
    gateway: ActionGateway,
    proposals: Vec<RuntimeActionProposal>,
    cancellations: usize,
}

impl TestApplication {
    fn new() -> Self {
        let temporary = TempDir::new().expect("temporary database root");
        let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(temporary.path()))
            .expect("database actor");
        Self {
            _temporary: temporary,
            gateway: ActionGateway::new(PolicyConfiguration::default(), Arc::new(database)),
            proposals: Vec::new(),
            cancellations: 0,
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
        RuntimeActionBridge::new(&mut self.gateway)
            .complete_effect_retryable(lease, result)
            .map_err(|_| ())
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.cancellations += 1;
        self.gateway
            .cancel_run(run_id, now_ms)
            .map(|_| ())
            .map_err(|_| ())
    }
}

struct BrokerFixture {
    worker: UnixStream,
    reader: BufReader<UnixStream>,
}

impl BrokerFixture {
    fn pair() -> (OpenCodeSdkBroker, Self) {
        let (broker, worker) = OpenCodeSdkBroker::authenticated_pair(4).expect("broker pair");
        worker
            .set_read_timeout(Some(TIMEOUT))
            .expect("response timeout");
        let reader = BufReader::new(worker.try_clone().expect("reader clone"));
        (broker, Self { worker, reader })
    }

    fn proposal(&mut self, correlation_id: &str, tool: &str, payload: Value) {
        self.proposal_with_message(correlation_id, tool, payload, "native-message-1");
    }

    fn proposal_with_message(
        &mut self,
        correlation_id: &str,
        tool: &str,
        payload: Value,
        message_id: &str,
    ) {
        self.send(json!({
            "schemaVersion": 1,
            "kind": "proposal",
            "correlationId": correlation_id,
            "tool": tool,
            "sessionId": "native-session-1",
            "messageId": message_id,
            "processGeneration": 4,
            "payload": payload,
        }));
    }

    fn cancel(&mut self, correlation_id: &str, tool: &str) {
        self.send(json!({
            "schemaVersion": 1,
            "kind": "cancel",
            "correlationId": correlation_id,
            "tool": tool,
            "sessionId": "native-session-1",
            "messageId": "native-message-1",
            "processGeneration": 4,
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
        assert!(!line.is_empty(), "broker response must be present");
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
            .expect_err("a second response must not be written");
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }
}

fn config() -> OpenCodeBrokerPumpConfig {
    OpenCodeBrokerPumpConfig {
        receive_timeout: TIMEOUT,
        response_timeout: TIMEOUT,
        max_pending_approvals: 4,
        max_terminal_requests: 16,
    }
}

#[test]
fn empty_registry_denies_before_the_action_gateway_or_any_effect() {
    let (broker, mut peer) = BrokerFixture::pair();
    let registry = InstalledBrokerFacilityRegistry::new();
    let mut pump = OpenCodeBrokerPump::attach_authenticated(broker, registry.clone(), config())
        .expect("broker pump");
    assert!(matches!(
        registry.install_resource(
            "workspace.summary",
            Some("active".into()),
            classification(ActionSurface::C4os, ActionEffect::Read, "c4os:summary"),
            Box::new(TestFacility {
                version: Arc::new(Mutex::new("workspace:1".into())),
                calls: Arc::new(AtomicUsize::new(0)),
            }),
        ),
        Err(FacilityRegistryError::Sealed)
    ));
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal(
        "request-empty",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
    );
    let outcome = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("closed denial");
    assert!(matches!(
        outcome,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "unsupported-broker-operation"
    ));
    assert!(application.proposals.is_empty());
    let response = peer.response();
    assert_eq!(response["status"], "denied");
    assert_eq!(response["reasonCode"], "unsupported-broker-operation");
}

#[test]
fn exact_installed_resource_executes_through_gateway_and_writes_one_result() {
    let (broker, mut peer) = BrokerFixture::pair();
    let version = Arc::new(Mutex::new("workspace-generation:12".into()));
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_resource(
            "workspace.summary",
            Some("active".into()),
            classification(
                ActionSurface::C4os,
                ActionEffect::Read,
                "c4os:workspace-summary",
            ),
            Box::new(TestFacility {
                version,
                calls: calls.clone(),
            }),
        )
        .expect("install resource");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal(
        "request-read",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
    );
    let outcome = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("resource result");
    assert!(matches!(
        outcome,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Result(_),
            ..
        }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(application.proposals.len(), 1);
    let response = peer.response();
    assert_eq!(response["status"], "result");
    assert_eq!(response["payload"]["status"], "succeeded");
    peer.assert_no_response();
}

#[test]
fn uncertain_result_write_is_explicitly_generation_fatal_and_seals_the_pump() {
    let (broker, mut peer) = BrokerFixture::pair();
    let registry = InstalledBrokerFacilityRegistry::new();
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal(
        "request-broken-response",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
    );
    drop(peer);

    let error = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect_err("an unobservable descriptor write must be fatal");
    assert!(
        matches!(
            &error,
            OpenCodeBrokerPumpError::ResponseStatusUnknown(_) | OpenCodeBrokerPumpError::Channel(_)
        ),
        "unexpected error: {error:?}"
    );
    assert!(error.is_generation_fatal(), "unexpected error: {error:?}");
    assert!(pump.is_sealed());
}

#[test]
fn explicit_user_allow_and_deny_settle_exactly_once_without_duplicate_effects() {
    for (correlation_id, answer, expected_calls) in [
        ("request-user-allow", ApprovalAnswer::Allow, 1),
        ("request-user-deny", ApprovalAnswer::Deny, 0),
    ] {
        let (broker, mut peer) = BrokerFixture::pair();
        let calls = Arc::new(AtomicUsize::new(0));
        let registry = InstalledBrokerFacilityRegistry::new();
        registry
            .install_action(
                "window.focus",
                "main",
                Map::new(),
                classification(
                    ActionSurface::Desktop,
                    ActionEffect::Control,
                    "desktop:main-window",
                ),
                Box::new(TestFacility {
                    version: Arc::new(Mutex::new("window-generation:3".into())),
                    calls: calls.clone(),
                }),
            )
            .expect("install action");
        let mut pump = OpenCodeBrokerPump::attach_authenticated(broker, registry, config())
            .expect("broker pump");
        let mut application = TestApplication::new();
        let mut resolver = resolver();

        peer.proposal(
            correlation_id,
            C4OS_ACTION_PROPOSAL_TOOL,
            json!({"operation": "window.focus", "target": "main"}),
        );
        let pending = pump
            .pump_one(&mut application, &mut resolver, 10)
            .expect("pending approval");
        let OpenCodeBrokerPumpOutcome::PendingApproval {
            correlation_id: pending_correlation_id,
            prompt_id,
        } = pending
        else {
            panic!("mutating action must pause for approval");
        };
        assert_eq!(pending_correlation_id, correlation_id);
        assert_eq!(pump.pending_count(), 1);

        let outcome = pump
            .answer_approval(
                &mut application,
                &mut resolver,
                correlation_id,
                &prompt_id,
                answer,
                20,
            )
            .expect("explicit user approval settlement");
        match answer {
            ApprovalAnswer::Allow => assert!(matches!(
                outcome,
                OpenCodeBrokerPumpOutcome::Responded {
                    decision: BrokerDecision::Result(ref payload),
                    ..
                } if payload["status"] == "succeeded"
                    && payload["resultCode"] == "installed-facility-completed"
                    && payload["changedTargetCount"] == 1
                    && payload["completedAtMs"] == 30
            )),
            ApprovalAnswer::Deny => assert!(matches!(
                outcome,
                OpenCodeBrokerPumpOutcome::Responded {
                    decision: BrokerDecision::Denied { ref reason_code },
                    ..
                } if reason_code == "user-denied"
            )),
        }
        assert_eq!(calls.load(Ordering::SeqCst), expected_calls);
        assert_eq!(pump.pending_count(), 0);

        let response = peer.response();
        match answer {
            ApprovalAnswer::Allow => {
                assert_eq!(response["status"], "result");
                assert_eq!(response["payload"]["status"], "succeeded");
                assert_eq!(
                    response["payload"]["resultCode"],
                    "installed-facility-completed"
                );
                assert_eq!(response["payload"]["changedTargetCount"], 1);
                assert_eq!(response["payload"]["completedAtMs"], 30);
            }
            ApprovalAnswer::Deny => {
                assert_eq!(response["status"], "denied");
                assert_eq!(response["reasonCode"], "user-denied");
                assert!(response.get("payload").is_none());
            }
        }
        assert!(matches!(
            pump.answer_approval(
                &mut application,
                &mut resolver,
                correlation_id,
                &prompt_id,
                answer,
                21,
            ),
            Err(OpenCodeBrokerPumpError::AlreadySettled)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), expected_calls);
        peer.assert_no_response();
    }
}

#[test]
fn approval_resume_rechecks_target_version_and_never_invokes_a_stale_facility() {
    let (broker, mut peer) = BrokerFixture::pair();
    let version = Arc::new(Mutex::new("window-generation:3".into()));
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_action(
            "window.focus",
            "main",
            Map::new(),
            classification(
                ActionSurface::Desktop,
                ActionEffect::Control,
                "desktop:main-window",
            ),
            Box::new(TestFacility {
                version: version.clone(),
                calls: calls.clone(),
            }),
        )
        .expect("install action");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal(
        "request-stale",
        C4OS_ACTION_PROPOSAL_TOOL,
        json!({"operation": "window.focus", "target": "main"}),
    );
    let pending = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("pending approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("mutating action must pause for approval");
    };
    assert_eq!(pump.pending_count(), 1);
    *version.lock().expect("version lock") = "window-generation:4".into();

    let outcome = pump
        .answer_approval(
            &mut application,
            &mut resolver,
            "request-stale",
            &prompt_id,
            ApprovalAnswer::Allow,
            20,
        )
        .expect("stale target denial");
    assert!(matches!(
        outcome,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "broker-target-version-changed"
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(pump.pending_count(), 0);
    let response = peer.response();
    assert_eq!(response["status"], "denied");
    assert_eq!(response["reasonCode"], "broker-target-version-changed");
    assert!(matches!(
        pump.answer_approval(
            &mut application,
            &mut resolver,
            "request-stale",
            &prompt_id,
            ApprovalAnswer::Allow,
            21,
        ),
        Err(OpenCodeBrokerPumpError::AlreadySettled)
    ));
    peer.assert_no_response();
}

#[test]
fn descriptor_cancellation_is_observed_without_a_second_response_or_effect() {
    let (broker, mut peer) = BrokerFixture::pair();
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_action(
            "window.focus",
            "main",
            Map::new(),
            classification(
                ActionSurface::Desktop,
                ActionEffect::Control,
                "desktop:main-window",
            ),
            Box::new(TestFacility {
                version: Arc::new(Mutex::new("window-generation:3".into())),
                calls: calls.clone(),
            }),
        )
        .expect("install action");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal(
        "request-cancel",
        C4OS_ACTION_PROPOSAL_TOOL,
        json!({"operation": "window.focus", "target": "main"}),
    );
    let pending = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("pending approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("mutating action must pause for approval");
    };
    peer.cancel("request-cancel", C4OS_ACTION_PROPOSAL_TOOL);
    let cancelled = pump
        .pump_one(&mut application, &mut resolver, 15)
        .expect("observed cancellation");
    assert!(matches!(
        cancelled,
        OpenCodeBrokerPumpOutcome::ObservedCancellation {
            decision: BrokerDecision::Cancelled,
            ..
        }
    ));
    assert_eq!(application.cancellations, 1);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let response = peer.response();
    assert_eq!(response["status"], "cancelled");
    assert!(matches!(
        pump.answer_approval(
            &mut application,
            &mut resolver,
            "request-cancel",
            &prompt_id,
            ApprovalAnswer::Allow,
            20,
        ),
        Err(OpenCodeBrokerPumpError::AlreadySettled)
    ));
    peer.assert_no_response();
}

#[test]
fn authenticated_message_substitution_is_unmapped_and_denied_before_classification() {
    let (broker, mut peer) = BrokerFixture::pair();
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_resource(
            "workspace.summary",
            Some("active".into()),
            classification(
                ActionSurface::C4os,
                ActionEffect::Read,
                "c4os:workspace-summary",
            ),
            Box::new(TestFacility {
                version: Arc::new(Mutex::new("workspace-generation:12".into())),
                calls: calls.clone(),
            }),
        )
        .expect("install resource");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal_with_message(
        "request-substituted-message",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
        "native-message-substituted",
    );
    let outcome = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("unmapped context denial");
    assert!(matches!(
        outcome,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "unmapped-broker-context"
    ));
    assert!(application.proposals.is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let response = peer.response();
    assert_eq!(response["status"], "denied");
    assert_eq!(response["reasonCode"], "unmapped-broker-context");
}

#[test]
fn uncertain_unmapped_denial_write_is_also_generation_fatal() {
    let (broker, mut peer) = BrokerFixture::pair();
    let registry = InstalledBrokerFacilityRegistry::new();
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();

    peer.proposal_with_message(
        "request-broken-unmapped-denial",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
        "native-message-substituted",
    );
    drop(peer);

    let error = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect_err("an unobservable denial write must be fatal");
    assert!(error.is_generation_fatal(), "unexpected error: {error:?}");
    assert!(
        matches!(
            &error,
            OpenCodeBrokerPumpError::ResponseStatusUnknown(_) | OpenCodeBrokerPumpError::Channel(_)
        ),
        "unexpected error: {error:?}"
    );
    assert!(pump.is_sealed());
}

#[test]
fn assistant_alias_rejects_wrong_role_id_session_parent_and_generation() {
    let resolver = resolver();
    let seed = context();

    let mut wrong_role = assistant_evidence("assistant-message-1");
    wrong_role.role = "user".into();
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, wrong_role, 10),
        Err(BrokerContextRegistryError::InvalidEvidence)
    );

    let invalid_id = assistant_evidence("");
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, invalid_id, 10),
        Err(BrokerContextRegistryError::InvalidEvidence)
    );

    let same_as_user_id = assistant_evidence("native-message-1");
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, same_as_user_id, 10),
        Err(BrokerContextRegistryError::EvidenceBindingMismatch)
    );

    let mut wrong_session = assistant_evidence("assistant-message-1");
    wrong_session.native_session_id = "native-session-2".into();
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, wrong_session, 10),
        Err(BrokerContextRegistryError::EvidenceBindingMismatch)
    );

    let mut wrong_parent = assistant_evidence("assistant-message-1");
    wrong_parent.parent_native_message_id = Some("native-message-other".into());
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, wrong_parent, 10),
        Err(BrokerContextRegistryError::EvidenceBindingMismatch)
    );

    let mut missing_parent = assistant_evidence("assistant-message-1");
    missing_parent.parent_native_message_id = None;
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, missing_parent, 10),
        Err(BrokerContextRegistryError::InvalidEvidence)
    );

    let mut wrong_generation = assistant_evidence("assistant-message-1");
    wrong_generation.process_generation = 5;
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&seed, wrong_generation, 10),
        Err(BrokerContextRegistryError::EvidenceBindingMismatch)
    );
    assert_eq!(resolver.active_count(), 1);
    assert_eq!(resolver.assistant_alias_count(), 0);
}

#[test]
fn assistant_alias_conflict_cannot_rebind_to_another_attempt_seed() {
    let resolver = resolver();
    let second = second_context();
    resolver
        .activate(second.clone(), 1, 1_000)
        .expect("activate second attempt seed");
    publish_assistant_alias(&resolver, "assistant-message-shared");

    let mut conflicting = assistant_evidence("assistant-message-shared");
    conflicting.parent_native_message_id = Some(second.native_message_id.clone());
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(&second, conflicting, 10),
        Err(BrokerContextRegistryError::Conflict)
    );
    assert_eq!(resolver.active_count(), 2);
    assert_eq!(resolver.assistant_alias_count(), 1);
}

#[test]
fn repeated_authenticated_updates_for_the_same_assistant_alias_are_idempotent() {
    let resolver = resolver();
    let seed = context();
    let evidence = assistant_evidence("assistant-message-repeated");
    resolver
        .publish_authenticated_assistant_alias(&seed, evidence.clone(), 10)
        .expect("publish initial assistant alias");
    resolver
        .publish_authenticated_assistant_alias(&seed, evidence, 11)
        .expect("repeat exact authenticated assistant alias");
    assert_eq!(resolver.active_count(), 1);
    assert_eq!(resolver.assistant_alias_count(), 1);
}

#[test]
fn one_authenticated_assistant_alias_accepts_multiple_tool_calls() {
    let (broker, mut peer) = BrokerFixture::pair();
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_resource(
            "workspace.summary",
            Some("active".into()),
            classification(
                ActionSurface::C4os,
                ActionEffect::Read,
                "c4os:workspace-summary",
            ),
            Box::new(TestFacility {
                version: Arc::new(Mutex::new("workspace-generation:12".into())),
                calls: calls.clone(),
            }),
        )
        .expect("install resource");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();
    publish_assistant_alias(&resolver, "assistant-message-tools");

    for correlation_id in ["assistant-tool-1", "assistant-tool-2"] {
        peer.proposal_with_message(
            correlation_id,
            C4OS_RESOURCE_READ_TOOL,
            json!({"resource": "workspace.summary", "selector": "active"}),
            "assistant-message-tools",
        );
        let outcome = pump
            .pump_one(&mut application, &mut resolver, 10)
            .expect("assistant tool result");
        assert!(matches!(
            outcome,
            OpenCodeBrokerPumpOutcome::Responded {
                decision: BrokerDecision::Result(_),
                ..
            }
        ));
        assert_eq!(peer.response()["status"], "result");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(application.proposals.len(), 2);
    assert_eq!(resolver.assistant_alias_count(), 1);
}

#[test]
fn expired_and_retired_assistant_aliases_fail_closed() {
    let (broker, mut peer) = BrokerFixture::pair();
    let registry = InstalledBrokerFacilityRegistry::new();
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();
    let seed = context();
    publish_assistant_alias(&resolver, "assistant-message-retired");

    peer.proposal_with_message(
        "request-expired-alias",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
        "assistant-message-retired",
    );
    let expired = pump
        .pump_one(&mut application, &mut resolver, 1_001)
        .expect("expired alias denial");
    assert!(matches!(
        expired,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "stale-runtime-identity"
    ));
    assert_eq!(peer.response()["reasonCode"], "stale-runtime-identity");

    assert!(resolver.retire(&seed).expect("retire attempt seed"));
    assert_eq!(resolver.active_count(), 0);
    assert_eq!(resolver.assistant_alias_count(), 0);
    assert_eq!(
        resolver.publish_authenticated_assistant_alias(
            &seed,
            assistant_evidence("assistant-message-after-retire"),
            10,
        ),
        Err(BrokerContextRegistryError::Unmapped)
    );

    peer.proposal_with_message(
        "request-retired-alias",
        C4OS_RESOURCE_READ_TOOL,
        json!({"resource": "workspace.summary", "selector": "active"}),
        "assistant-message-retired",
    );
    let retired = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("retired alias denial");
    assert!(matches!(
        retired,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "unmapped-broker-context"
    ));
    assert_eq!(peer.response()["reasonCode"], "unmapped-broker-context");
    assert!(application.proposals.is_empty());
}

#[test]
fn approval_resume_rejects_refreshed_live_authority_context() {
    let (broker, mut peer) = BrokerFixture::pair();
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_action(
            "window.focus",
            "main",
            Map::new(),
            classification(
                ActionSurface::Desktop,
                ActionEffect::Control,
                "desktop:main-window",
            ),
            Box::new(TestFacility {
                version: Arc::new(Mutex::new("window-generation:3".into())),
                calls: calls.clone(),
            }),
        )
        .expect("install action");
    let mut pump =
        OpenCodeBrokerPump::attach_authenticated(broker, registry, config()).expect("broker pump");
    let mut application = TestApplication::new();
    let mut resolver = resolver();
    publish_assistant_alias(&resolver, "assistant-message-approval");

    peer.proposal_with_message(
        "request-live-refresh",
        C4OS_ACTION_PROPOSAL_TOOL,
        json!({"operation": "window.focus", "target": "main"}),
        "assistant-message-approval",
    );
    let pending = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("pending approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("mutating action must pause for approval");
    };

    let mut refreshed = context();
    refreshed.configuration_version = 8;
    resolver
        .refresh(refreshed, 11, 1_000)
        .expect("refresh live versions");
    assert_eq!(resolver.assistant_alias_count(), 1);
    let outcome = pump
        .answer_approval(
            &mut application,
            &mut resolver,
            "request-live-refresh",
            &prompt_id,
            ApprovalAnswer::Allow,
            20,
        )
        .expect("stale context denial");
    assert!(matches!(
        outcome,
        OpenCodeBrokerPumpOutcome::Responded {
            decision: BrokerDecision::Denied { ref reason_code },
            ..
        } if reason_code == "stale-runtime-identity"
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(application.cancellations, 1);
    let response = peer.response();
    assert_eq!(response["status"], "denied");
    assert_eq!(response["reasonCode"], "stale-runtime-identity");
}

#[test]
fn queued_cancellation_wins_ready_deferred_completion_and_never_writes_twice() {
    let (broker, mut peer) = BrokerFixture::pair();
    let state = Arc::new(Mutex::new(DeferredFacilityState::default()));
    let registry = InstalledBrokerFacilityRegistry::new();
    registry
        .install_deferred(Box::new(TestDeferredFacility {
            state: Arc::clone(&state),
        }))
        .expect("install deferred MCP facility");
    let mut pump = OpenCodeBrokerPump::attach_authenticated(
        broker,
        registry,
        OpenCodeBrokerPumpConfig {
            receive_timeout: Duration::from_millis(20),
            ..config()
        },
    )
    .expect("broker pump");
    let trusted = mcp_context();
    let target = trusted.mcp_turn.as_ref().unwrap().tools[0]
        .target_id
        .clone();
    let mut resolver = resolver_for(trusted);
    let mut application = TestApplication::new();

    peer.proposal(
        "request-deferred-cancel",
        C4OS_ACTION_PROPOSAL_TOOL,
        json!({
            "operation": "mcp.call-tool",
            "target": target,
            "arguments": { "value": "hello" },
        }),
    );
    let pending = pump
        .pump_one(&mut application, &mut resolver, 10)
        .expect("pending MCP approval");
    let OpenCodeBrokerPumpOutcome::PendingApproval { prompt_id, .. } = pending else {
        panic!("MCP action must retain explicit approval");
    };
    let started = pump
        .answer_approval(
            &mut application,
            &mut resolver,
            "request-deferred-cancel",
            &prompt_id,
            ApprovalAnswer::Allow,
            20,
        )
        .expect("start deferred MCP effect");
    assert!(matches!(
        started,
        OpenCodeBrokerPumpOutcome::EffectRunning { ref correlation_id }
            if correlation_id == "request-deferred-cancel"
    ));
    assert_eq!(pump.pending_count(), 0);
    assert!(pump.pending_approval_descriptors().is_empty());
    assert_eq!(state.lock().expect("deferred state").starts, 1);

    state.lock().expect("deferred state").ready = true;
    peer.cancel("request-deferred-cancel", C4OS_ACTION_PROPOSAL_TOOL);
    let cancelled = pump
        .pump_one(&mut application, &mut resolver, 21)
        .expect("queued cancellation");
    assert!(matches!(
        cancelled,
        OpenCodeBrokerPumpOutcome::ObservedCancellation {
            ref correlation_id,
            decision: BrokerDecision::Cancelled,
        } if correlation_id == "request-deferred-cancel"
    ));
    assert_eq!(application.cancellations, 1);
    assert_eq!(state.lock().expect("deferred state").cancellations, 1);
    let response = peer.response();
    assert_eq!(response["status"], "cancelled");

    let settled = pump
        .pump_one(&mut application, &mut resolver, 30)
        .expect("administrative late settlement");
    assert!(matches!(
        settled,
        OpenCodeBrokerPumpOutcome::SettledAfterCancellation {
            ref correlation_id,
            decision: BrokerDecision::Result(_),
        } if correlation_id == "request-deferred-cancel"
    ));
    peer.assert_no_response();
}
