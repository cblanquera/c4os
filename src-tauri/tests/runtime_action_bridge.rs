use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::runtime::action_bridge::{
    RuntimeActionBridge, RuntimeActionProposal, RuntimeApprovalDecision, RuntimeBridgeError,
    RuntimeGatewayDecision, RuntimeIntentIdentity,
};
use c4os_lib::runtime::opencode::ActionIntent;
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
use serde_json::json;
use tempfile::TempDir;

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn native_intent() -> ActionIntent {
    ActionIntent {
        workspace_id: "workspace-1".into(),
        c4os_session_id: "session-1".into(),
        c4os_turn_id: "turn-1".into(),
        c4os_run_id: "run-1".into(),
        correlation_id: "correlation-1".into(),
        runtime_id: "opencode@1".into(),
        process_generation: 4,
        native_session_id: "native-session-1".into(),
        native_request_id: "call-1".into(),
        native_tool: "write_file".into(),
        native_arguments: json!({"path": "README.md", "content": "not persisted by bridge"}),
        resources: vec!["README.md".into()],
    }
}

fn facts() -> ActionFacts {
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
        runtime_id: "opencode@1".into(),
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

fn action() -> CanonicalAction {
    CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: "action-1".into(),
        tool_call_id: "call-1".into(),
        tool: "write_file".into(),
        arguments: json!({"contentSha256": "sha256:new", "path": "README.md"}),
        risk: CanonicalRisk::Medium,
        requested_authority: set(["workspace-files.modify".into()]),
        canonical_target: "workspace:/project/README.md".into(),
        target_version: "sha256:old".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: "run-1".into(),
        runtime_id: "opencode@1".into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: None,
        process_generation: 4,
        configuration_version: 7,
        policy_version: 9,
        revocation_epoch: 2,
    }
}

fn live() -> LiveAuthorityState {
    LiveAuthorityState {
        process_generation: 4,
        configuration_version: 7,
        policy_version: 9,
        revocation_epoch: 2,
    }
}

fn success() -> NormalizedActionResult {
    NormalizedActionResult {
        status: NormalizedActionStatus::Succeeded,
        result_code: "worker-completed".into(),
        exit_code: None,
        changed_targets: vec!["workspace:/project/README.md".into()],
        output_sha256: Some(format!("sha256:{:064x}", 1)),
        completed_at_ms: 14,
    }
}

#[test]
fn runtime_worker_is_reached_only_after_exact_gateway_approval_and_consumption() {
    let temporary = TempDir::new().unwrap();
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).unwrap();
    let mut gateway = ActionGateway::new(PolicyConfiguration::default(), Arc::new(database));
    let identity = RuntimeIntentIdentity::from_opencode(&native_intent()).unwrap();
    let proposal = RuntimeActionProposal::new(identity, facts(), action()).unwrap();
    let effects = AtomicUsize::new(0);

    let mut bridge = RuntimeActionBridge::new(&mut gateway);
    let RuntimeGatewayDecision::PendingApproval { prompt_id, .. } =
        bridge.propose(proposal, 10).unwrap()
    else {
        panic!("default policy must preserve an explicit approval barrier");
    };
    assert_eq!(effects.load(Ordering::SeqCst), 0);

    let RuntimeApprovalDecision::Authorized(authorization) = bridge
        .answer_approval(&prompt_id, ApprovalAnswer::Allow, 11)
        .unwrap()
    else {
        panic!("approved prompt must yield a sealed authorization");
    };
    assert_eq!(effects.load(Ordering::SeqCst), 0);

    let result = bridge
        .execute(*authorization, live(), 12, |_| {
            effects.fetch_add(1, Ordering::SeqCst);
            success()
        })
        .unwrap();
    assert_eq!(result.result().status, NormalizedActionStatus::Succeeded);
    assert_eq!(effects.load(Ordering::SeqCst), 1);
}

#[test]
fn runtime_proposal_owns_the_binding_field_and_rejects_caller_substitution() {
    let identity = RuntimeIntentIdentity::from_opencode(&native_intent()).unwrap();
    let mut substituted = action();
    substituted.arguments["c4osRuntimeIntentSha256"] =
        serde_json::Value::String(format!("sha256:{}", "0".repeat(64)));

    assert!(matches!(
        RuntimeActionProposal::new(identity, facts(), substituted),
        Err(RuntimeBridgeError::InvalidProposal)
    ));
}

#[test]
fn runtime_proposal_rejects_cross_run_tool_and_process_substitution() {
    let identity = RuntimeIntentIdentity::from_opencode(&native_intent()).unwrap();

    let mut wrong_action = action();
    wrong_action.run_id = "run-other".into();
    assert!(matches!(
        RuntimeActionProposal::new(identity.clone(), facts(), wrong_action),
        Err(RuntimeBridgeError::BindingMismatch)
    ));

    let mut wrong_facts = facts();
    wrong_facts.native_tool = "shell".into();
    assert!(matches!(
        RuntimeActionProposal::new(identity, wrong_facts, action()),
        Err(RuntimeBridgeError::BindingMismatch)
    ));
}

#[test]
fn denied_runtime_proposal_never_yields_an_execution_capability() {
    let temporary = TempDir::new().unwrap();
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).unwrap();
    let mut gateway = ActionGateway::new(PolicyConfiguration::default(), Arc::new(database));
    let identity = RuntimeIntentIdentity::from_opencode(&native_intent()).unwrap();
    let mut denied_facts = facts();
    denied_facts.sandbox_allows = false;
    let proposal = RuntimeActionProposal::new(identity, denied_facts, action()).unwrap();

    let mut bridge = RuntimeActionBridge::new(&mut gateway);
    assert!(matches!(
        bridge.propose(proposal, 10).unwrap(),
        RuntimeGatewayDecision::Denied { .. }
    ));
}
