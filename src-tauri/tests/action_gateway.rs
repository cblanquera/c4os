use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor, SnapshotQuery};
use c4os_lib::security::authorization::{
    ApprovalAnswer, ApprovalPromptState, AuthorizationError, CANONICAL_ACTION_SCHEMA_VERSION,
    CanonicalAction, CanonicalRisk, LiveAuthorityState,
};
use c4os_lib::security::gateway::{
    ActionGateway, ActionGatewayError, ApprovalResponse, GatewayProposal, NormalizedActionResult,
    NormalizedActionStatus,
};
use c4os_lib::security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ApprovalPreset, ClassificationConfidence,
    PolicyConfiguration, RepositoryState,
};
use serde_json::json;
use tempfile::TempDir;

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn app_database(temp: &TempDir) -> (DatabaseDescriptor, Arc<DatabaseActor>) {
    let descriptor = DatabaseDescriptor::app(temp.path());
    let (database, report) = DatabaseActor::start(descriptor.clone()).expect("app database");
    assert_eq!(report.current_version, 4);
    (descriptor, Arc::new(database))
}

fn facts(effect: ActionEffect) -> ActionFacts {
    ActionFacts {
        action_kind: if effect == ActionEffect::Read {
            "file.read".into()
        } else {
            "file.write".into()
        },
        native_tool: if effect == ActionEffect::Read {
            "read_file".into()
        } else {
            "write_file".into()
        },
        surface: ActionSurface::File,
        effects: set([effect]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Agent,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
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

fn action(effect: ActionEffect, action_id: &str, run_id: &str) -> CanonicalAction {
    CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.into(),
        tool_call_id: format!("call-{action_id}"),
        tool: if effect == ActionEffect::Read {
            "read_file".into()
        } else {
            "write_file".into()
        },
        arguments: if effect == ActionEffect::Read {
            json!({"path": "README.md"})
        } else {
            json!({"contentSha256": "sha256:new", "path": "README.md"})
        },
        risk: if effect == ActionEffect::Read {
            CanonicalRisk::Low
        } else {
            CanonicalRisk::Medium
        },
        requested_authority: set([if effect == ActionEffect::Read {
            "workspace-files.read".into()
        } else {
            "workspace-files.modify".into()
        }]),
        canonical_target: "workspace:/project/README.md".into(),
        target_version: "sha256:old".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: run_id.into(),
        runtime_id: "opencode@1".into(),
        environment_id: "local".into(),
        process_generation: 4,
        configuration_version: 7,
        policy_version: 9,
        revocation_epoch: 2,
    }
}

fn live(action: &CanonicalAction) -> LiveAuthorityState {
    LiveAuthorityState {
        process_generation: action.process_generation,
        configuration_version: action.configuration_version,
        policy_version: action.policy_version,
        revocation_epoch: action.revocation_epoch,
    }
}

fn success(at: u64) -> NormalizedActionResult {
    NormalizedActionResult {
        status: NormalizedActionStatus::Succeeded,
        result_code: "ok".into(),
        exit_code: Some(0),
        changed_targets: Vec::new(),
        output_sha256: Some(format!("sha256:{:064x}", 7)),
        completed_at_ms: at,
    }
}

#[test]
fn allow_is_persisted_before_one_effect_and_replay_is_denied() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveSafeActions,
        ..PolicyConfiguration::default()
    };
    let mut gateway = ActionGateway::new(policy, Arc::clone(&database));
    let action = action(ActionEffect::Read, "action-1", "run-1");
    let token = match gateway
        .propose(&facts(ActionEffect::Read), action.clone(), 10)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };

    let effects = AtomicUsize::new(0);
    let result = gateway
        .execute(&token, &action, live(&action), None, 11, |_| {
            effects.fetch_add(1, Ordering::SeqCst);
            success(12)
        })
        .expect("authorized effect");
    assert_eq!(result.status, NormalizedActionStatus::Succeeded);
    assert_eq!(effects.load(Ordering::SeqCst), 1);

    let replay = gateway
        .execute(&token, &action, live(&action), None, 13, |_| {
            effects.fetch_add(1, Ordering::SeqCst);
            success(14)
        })
        .expect_err("single-use token must not replay");
    assert!(matches!(
        replay,
        ActionGatewayError::Authorization(AuthorizationError::Replay)
    ));
    assert_eq!(effects.load(Ordering::SeqCst), 1);

    let records = database
        .security_records(SnapshotQuery::new(20).expect("query"))
        .expect("durable current states");
    assert!(
        records
            .iter()
            .any(|record| { record.record_kind == "authorization" && record.state == "consumed" })
    );
    assert!(
        records
            .iter()
            .any(|record| record.record_kind == "action-result" && record.state == "succeeded")
    );
    assert!(
        database
            .security_events(SnapshotQuery::new(20).expect("query"))
            .expect("append-only transitions")
            .len()
            >= 6
    );
}

#[test]
fn argument_mutation_burns_and_persists_authorization_before_any_effect() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        ..PolicyConfiguration::default()
    };
    let mut gateway = ActionGateway::new(policy, Arc::clone(&database));
    let action = action(ActionEffect::Modify, "action-2", "run-2");
    let token = match gateway
        .propose(&facts(ActionEffect::Modify), action.clone(), 20)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    let mut mutated = action.clone();
    mutated.arguments = json!({"contentSha256": "sha256:mutated", "path": "README.md"});
    let effects = AtomicUsize::new(0);
    let error = gateway
        .execute(&token, &mutated, live(&action), None, 21, |_| {
            effects.fetch_add(1, Ordering::SeqCst);
            success(22)
        })
        .expect_err("mutated arguments must fail exact binding");
    assert!(matches!(
        error,
        ActionGatewayError::Authorization(AuthorizationError::BindingMismatch(_))
    ));
    assert_eq!(effects.load(Ordering::SeqCst), 0);
    let records = database
        .security_records(SnapshotQuery::new(20).expect("query"))
        .expect("durable states");
    assert!(
        records.iter().any(|record| {
            record.record_kind == "authorization" && record.state == "invalidated"
        })
    );
}

#[test]
fn approvals_serialize_per_run_and_independent_runs_remain_visible() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let mut gateway = ActionGateway::new(PolicyConfiguration::default(), database);

    let first = action(ActionEffect::Modify, "action-a", "run-a");
    let second = action(ActionEffect::Modify, "action-b", "run-a");
    let independent = action(ActionEffect::Modify, "action-c", "run-b");
    let first_prompt = match gateway
        .propose(&facts(ActionEffect::Modify), first.clone(), 30)
        .expect("first proposal")
    {
        GatewayProposal::PendingApproval { prompt, .. } => prompt,
        other => panic!("expected prompt, found {other:?}"),
    };
    let second_prompt = match gateway
        .propose(&facts(ActionEffect::Modify), second, 31)
        .expect("second proposal")
    {
        GatewayProposal::PendingApproval { prompt, .. } => prompt,
        other => panic!("expected prompt, found {other:?}"),
    };
    let independent_prompt = match gateway
        .propose(&facts(ActionEffect::Modify), independent, 32)
        .expect("independent proposal")
    {
        GatewayProposal::PendingApproval { prompt, .. } => prompt,
        other => panic!("expected prompt, found {other:?}"),
    };
    assert_eq!(first_prompt.state, ApprovalPromptState::Pending);
    assert_eq!(second_prompt.state, ApprovalPromptState::Queued);
    assert_eq!(independent_prompt.state, ApprovalPromptState::Pending);
    assert_eq!(gateway.approval_queue().visible_queue().len(), 3);

    let token = match gateway
        .answer_approval(&first_prompt.prompt_id, ApprovalAnswer::Allow, 33)
        .expect("approve first")
    {
        ApprovalResponse::Authorized { token, .. } => token,
        other => panic!("expected approved token, found {other:?}"),
    };
    assert_eq!(
        gateway
            .approval_queue()
            .prompt(&second_prompt.prompt_id)
            .expect("promoted prompt")
            .state,
        ApprovalPromptState::Pending
    );
    let effects = AtomicUsize::new(0);
    assert!(matches!(
        gateway.execute(&token, &first, live(&first), None, 34, |_| {
            effects.fetch_add(1, Ordering::SeqCst);
            success(35)
        }),
        Err(ActionGatewayError::BindingMismatch)
    ));
    assert_eq!(effects.load(Ordering::SeqCst), 0);
    gateway
        .execute(
            &token,
            &first,
            live(&first),
            Some(&first_prompt.prompt_id),
            35,
            |_| success(36),
        )
        .expect("approved effect");
    assert_eq!(
        gateway
            .approval_queue()
            .prompt(&first_prompt.prompt_id)
            .expect("completed prompt")
            .state,
        ApprovalPromptState::Completed {
            completed_at_ms: 36
        }
    );
}

#[test]
fn raw_secret_arguments_fail_before_policy_or_authorization() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let mut gateway = ActionGateway::new(
        PolicyConfiguration {
            preset: ApprovalPreset::ApproveForMe,
            ..PolicyConfiguration::default()
        },
        Arc::clone(&database),
    );
    let hostile_arguments = [
        json!({"password": "raw-secret", "path": "README.md"}),
        json!({"value": "-----BEGIN PRIVATE KEY-----\nraw\n-----END PRIVATE KEY-----"}),
        json!({"payload": "postgres://user:pass@host/database"}),
        json!({"header": "X-Custom-Auth", "value": "raw-secret"}),
        json!({"data": "Bearer raw-secret"}),
    ];
    for (index, arguments) in hostile_arguments.into_iter().enumerate() {
        let mut unsafe_action = action(
            ActionEffect::Modify,
            &format!("action-secret-{index}"),
            "run-secret",
        );
        unsafe_action.arguments = arguments;
        let error = gateway
            .propose(
                &facts(ActionEffect::Modify),
                unsafe_action,
                40 + index as u64,
            )
            .expect_err("inline credential must fail before persistence and authorization");
        assert!(matches!(
            error,
            ActionGatewayError::InlineCredentialMaterial
        ));
    }
    assert!(
        database
            .security_records(SnapshotQuery::new(20).expect("query"))
            .expect("no records")
            .is_empty()
    );
}

#[test]
fn credential_bearing_identifiers_fail_before_the_redacted_journal() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let mut gateway = ActionGateway::new(
        PolicyConfiguration {
            preset: ApprovalPreset::ApproveForMe,
            ..PolicyConfiguration::default()
        },
        Arc::clone(&database),
    );
    let dsn = "postgres://user:pass@host/db";
    let mut unsafe_action = action(ActionEffect::Read, "action-hostile-id", "run-safe");
    unsafe_action.runtime_id = dsn.into();
    let mut unsafe_facts = facts(ActionEffect::Read);
    unsafe_facts.runtime_id = dsn.into();
    assert!(matches!(
        gateway.propose(&unsafe_facts, unsafe_action, 45),
        Err(ActionGatewayError::BindingMismatch)
    ));
    assert!(
        database
            .security_records(SnapshotQuery::new(20).unwrap())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn audit_persists_only_argument_and_target_commitments() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let mut gateway = ActionGateway::new(
        PolicyConfiguration {
            preset: ApprovalPreset::ApproveForMe,
            ..PolicyConfiguration::default()
        },
        Arc::clone(&database),
    );
    let mut proposed = action(ActionEffect::Modify, "action-redacted", "run-redacted");
    proposed.arguments = json!({"value": "ordinary but private user content"});
    let _ = gateway
        .propose(&facts(ActionEffect::Modify), proposed, 50)
        .expect("safe proposal");
    let records = database
        .security_records(SnapshotQuery::new(20).expect("query"))
        .expect("audit records");
    let audit = records
        .iter()
        .map(|record| record.canonical_document.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!audit.contains("ordinary but private user content"));
    assert!(!audit.contains("workspace:/project/README.md"));
    assert!(audit.contains("argumentsSha256"));
    assert!(audit.contains("canonicalTargetSha256"));
}

#[test]
fn issued_token_restores_by_exact_digests_and_interrupted_effect_becomes_unknown() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        ..PolicyConfiguration::default()
    };
    let mut gateway = ActionGateway::new(policy.clone(), Arc::clone(&database));
    let action = action(ActionEffect::Read, "action-restart", "run-restart");
    let token = match gateway
        .propose(&facts(ActionEffect::Read), action.clone(), 60)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    drop(gateway);

    let mut restored =
        ActionGateway::restore(policy, Arc::clone(&database), 61).expect("restore issued verifier");
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = restored.execute(&token, &action, live(&action), None, 62, |_| {
            panic!("simulated executor interruption after the durable effect-start barrier")
        });
    }));
    assert!(panic.is_err());
    drop(restored);

    let _recovered =
        ActionGateway::restore(PolicyConfiguration::default(), Arc::clone(&database), 63)
            .expect("recover interrupted effect");
    let records = database
        .security_records(SnapshotQuery::new(20).expect("query"))
        .expect("recovered states");
    assert!(
        records.iter().any(|record| {
            record.record_kind == "action-intent" && record.state == "interrupted"
        })
    );
    assert!(records.iter().any(|record| {
        record.record_kind == "action-result" && record.state == "unknown-after-interruption"
    }));
}

#[test]
fn restored_authorizations_obey_run_cancellation_expiry_and_policy_invalidation() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        ..PolicyConfiguration::default()
    };

    let cancel_action = action(ActionEffect::Read, "action-cancel-restored", "run-cancel");
    let mut initial = ActionGateway::new(policy.clone(), Arc::clone(&database));
    let cancel_token = match initial
        .propose(&facts(ActionEffect::Read), cancel_action.clone(), 100)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    drop(initial);
    let mut restored = ActionGateway::restore(policy.clone(), Arc::clone(&database), 101)
        .expect("restore issued verifier");
    assert_eq!(restored.cancel_run("run-cancel", 102).unwrap(), 1);
    assert!(matches!(
        restored.execute(
            &cancel_token,
            &cancel_action,
            live(&cancel_action),
            None,
            103,
            |_| success(104)
        ),
        Err(ActionGatewayError::Authorization(
            AuthorizationError::Cancelled
        ))
    ));

    let expire_action = action(ActionEffect::Read, "action-expire-restored", "run-expire");
    let initial = ActionGateway::new(policy.clone(), Arc::clone(&database));
    let expire_token = match initial
        .with_ttls(5, 50)
        .unwrap()
        .propose(&facts(ActionEffect::Read), expire_action.clone(), 110)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    let mut restored = ActionGateway::restore(policy.clone(), Arc::clone(&database), 111)
        .expect("restore issued verifier");
    assert_eq!(restored.expire_due(115).unwrap(), 1);
    assert!(matches!(
        restored.execute(
            &expire_token,
            &expire_action,
            live(&expire_action),
            None,
            116,
            |_| success(117)
        ),
        Err(ActionGatewayError::Authorization(
            AuthorizationError::Expired
        ))
    ));

    let policy_action = action(ActionEffect::Read, "action-policy-restored", "run-policy");
    let mut initial = ActionGateway::new(policy.clone(), Arc::clone(&database));
    let policy_token = match initial
        .propose(&facts(ActionEffect::Read), policy_action.clone(), 120)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    let mut restored = ActionGateway::restore(policy.clone(), Arc::clone(&database), 121)
        .expect("restore issued verifier");
    let mut changed_live = live(&policy_action);
    changed_live.policy_version += 1;
    assert_eq!(
        restored
            .replace_policy(
                policy,
                changed_live,
                &policy_action.runtime_id,
                &policy_action.environment_id,
                122,
            )
            .unwrap(),
        1
    );
    assert!(matches!(
        restored.execute(
            &policy_token,
            &policy_action,
            live(&policy_action),
            None,
            123,
            |_| success(124)
        ),
        Err(ActionGatewayError::Authorization(
            AuthorizationError::Invalidated
        ))
    ));
}

#[test]
fn executor_result_digest_is_validated_before_any_result_is_journaled() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, database) = app_database(&temp);
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveSafeActions,
        ..PolicyConfiguration::default()
    };
    let mut gateway = ActionGateway::new(policy, Arc::clone(&database));
    let action = action(
        ActionEffect::Read,
        "action-invalid-result",
        "run-invalid-result",
    );
    let token = match gateway
        .propose(&facts(ActionEffect::Read), action.clone(), 130)
        .expect("proposal")
    {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected authorization, found {other:?}"),
    };
    let secret = "-----BEGIN PRIVATE KEY-----neutral-field";
    let error = gateway
        .execute(&token, &action, live(&action), None, 131, |_| {
            let mut result = success(132);
            result.output_sha256 = Some(secret.into());
            result
        })
        .expect_err("raw output cannot masquerade as a digest");
    assert!(matches!(error, ActionGatewayError::InvalidNormalizedResult));
    let audit = database
        .security_records(SnapshotQuery::new(30).unwrap())
        .unwrap()
        .into_iter()
        .map(|record| record.canonical_document)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!audit.contains(secret));
    assert!(!audit.contains("action-result-v1"));
}
