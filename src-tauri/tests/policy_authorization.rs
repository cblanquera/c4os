#[path = "../src/security/authorization.rs"]
#[allow(dead_code)]
mod authorization;
#[path = "../src/security/policy.rs"]
#[allow(dead_code)]
mod policy;

use authorization::{
    ApprovalAnswer, ApprovalPromptState, ApprovalQueue, AuthorizationError,
    AuthorizationInvalidation, AuthorizationLedger, AuthorizationState,
    CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk, GateDecision,
    LiveAuthorityState,
};
use policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ApprovalPreset, CategoryRule, CeilingRule,
    ClassificationConfidence, ConcreteException, ExceptionDuration, POLICY_GROUPS,
    PolicyConfiguration, PolicyDecision, PolicyGroup, RepositoryState, RuleMatcher, resolve_policy,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn set<T: Ord>(values: impl IntoIterator<Item = T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn facts() -> ActionFacts {
    ActionFacts {
        action_kind: "file.read".into(),
        native_tool: "read_file".into(),
        surface: ActionSurface::File,
        effects: set([ActionEffect::Read]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Agent,
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
        tool: "file.write".into(),
        arguments: json!({"content": "hello", "path": "README.md"}),
        risk: CanonicalRisk::Medium,
        requested_authority: set(["workspace-files.modify".into()]),
        canonical_target: "workspace:/project/README.md".into(),
        target_version: "sha256:old".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        run_id: "run-1".into(),
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

#[test]
fn exposes_exactly_four_presets_and_seven_groups_with_bounded_defaults() {
    let presets = [
        ApprovalPreset::AskForApproval,
        ApprovalPreset::ApproveSafeActions,
        ApprovalPreset::ApproveForMe,
        ApprovalPreset::Custom,
    ];
    assert_eq!(presets.len(), 4);
    assert_eq!(POLICY_GROUPS.len(), 7);
    assert_eq!(
        POLICY_GROUPS.iter().copied().collect::<BTreeSet<_>>().len(),
        7
    );

    let read = facts();
    let mut modify = facts();
    modify.action_kind = "file.modify".into();
    modify.effects = set([ActionEffect::Modify]);

    let decision = |facts: &ActionFacts, preset| {
        resolve_policy(
            facts,
            &PolicyConfiguration {
                preset,
                ..PolicyConfiguration::default()
            },
            10,
        )
        .decision
    };
    assert_eq!(
        decision(&read, ApprovalPreset::AskForApproval),
        PolicyDecision::Allow
    );
    assert_eq!(
        decision(&modify, ApprovalPreset::AskForApproval),
        PolicyDecision::Ask
    );
    assert_eq!(
        decision(&modify, ApprovalPreset::ApproveSafeActions),
        PolicyDecision::Allow
    );
    assert_eq!(
        decision(&modify, ApprovalPreset::ApproveForMe),
        PolicyDecision::Allow
    );
    assert_eq!(decision(&read, ApprovalPreset::Custom), PolicyDecision::Ask);
}

#[test]
fn most_restrictive_rule_wins_for_every_decision_permutation_and_multi_group_action() {
    let decisions = [
        PolicyDecision::Allow,
        PolicyDecision::Ask,
        PolicyDecision::Deny,
    ];
    let mut publish = facts();
    publish.surface = ActionSurface::Git;
    publish.action_kind = "git.remote.write".into();
    publish.native_tool = "git.push".into();
    publish.effects = set([ActionEffect::Modify, ActionEffect::Publish]);
    publish.scope = ActionScope::Remote;
    publish.canonical_target = "remote:https://example.invalid/repo.git".into();
    publish.sensitivity = ActionSensitivity::Ordinary;
    assert_eq!(
        publish.policy_groups(),
        set([PolicyGroup::VersionControl, PolicyGroup::NetworkAndSharing])
    );

    for left in decisions {
        for right in decisions {
            let configuration = PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                category_rules: vec![
                    CategoryRule {
                        id: "git".into(),
                        group: PolicyGroup::VersionControl,
                        decision: left,
                        matcher: RuleMatcher::default(),
                    },
                    CategoryRule {
                        id: "network".into(),
                        group: PolicyGroup::NetworkAndSharing,
                        decision: right,
                        matcher: RuleMatcher::default(),
                    },
                ],
                ..PolicyConfiguration::default()
            };
            assert_eq!(
                resolve_policy(&publish, &configuration, 10).decision,
                left.max(right),
                "{left:?} plus {right:?}"
            );
        }
    }
}

#[test]
fn partial_category_allow_does_not_silently_allow_uncovered_effects_or_groups() {
    let mut publish = facts();
    publish.surface = ActionSurface::Git;
    publish.action_kind = "git.inspect-and-publish".into();
    publish.native_tool = "git.composite".into();
    publish.effects = set([ActionEffect::Read, ActionEffect::Publish]);
    publish.scope = ActionScope::Remote;
    publish.canonical_target = "remote:https://example.invalid/repo.git".into();

    let only_read = CategoryRule {
        id: "allow-git-read".into(),
        group: PolicyGroup::VersionControl,
        decision: PolicyDecision::Allow,
        matcher: RuleMatcher {
            effects: set([ActionEffect::Read]),
            ..RuleMatcher::default()
        },
    };
    let config = PolicyConfiguration {
        preset: ApprovalPreset::AskForApproval,
        category_rules: vec![only_read],
        ..PolicyConfiguration::default()
    };
    assert_eq!(
        resolve_policy(&publish, &config, 10).decision,
        PolicyDecision::Ask
    );

    let exact = ConcreteException::from_action(
        "exact-composite",
        PolicyDecision::Allow,
        &publish,
        ExceptionDuration::Persistent,
    );
    let config = PolicyConfiguration {
        preset: ApprovalPreset::AskForApproval,
        exceptions: vec![exact],
        ..PolicyConfiguration::default()
    };
    assert_eq!(
        resolve_policy(&publish, &config, 10).decision,
        PolicyDecision::Allow
    );
}

#[test]
fn safety_managed_and_maximum_ceilings_cannot_be_bypassed() {
    let allow_everything = CategoryRule {
        id: "allow".into(),
        group: PolicyGroup::Credentials,
        decision: PolicyDecision::Allow,
        matcher: RuleMatcher::default(),
    };
    let mut reveal = facts();
    reveal.action_kind = "credential.reveal".into();
    reveal.surface = ActionSurface::Credential;
    reveal.effects = set([ActionEffect::Reveal]);
    reveal.sensitivity = ActionSensitivity::Credential;
    let config = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        category_rules: vec![allow_everything],
        ..PolicyConfiguration::default()
    };
    assert_eq!(
        resolve_policy(&reveal, &config, 10).decision,
        PolicyDecision::Deny
    );

    let mut execute = facts();
    execute.surface = ActionSurface::Terminal;
    execute.action_kind = "terminal.project.write".into();
    execute.effects = set([ActionEffect::Execute]);
    let ceiling_matcher = RuleMatcher {
        surface: Some(ActionSurface::Terminal),
        ..RuleMatcher::default()
    };
    let config = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        maximum_authority: vec![
            CeilingRule::new("max-terminal", PolicyDecision::Ask, ceiling_matcher.clone()).unwrap(),
        ],
        managed_requirements: vec![
            CeilingRule::new("managed-terminal", PolicyDecision::Deny, ceiling_matcher).unwrap(),
        ],
        ..PolicyConfiguration::default()
    };
    assert_eq!(
        resolve_policy(&execute, &config, 10).decision,
        PolicyDecision::Deny
    );
    assert!(CeilingRule::new("invalid", PolicyDecision::Allow, RuleMatcher::default()).is_err());
}

#[test]
fn unknown_and_exact_safety_exceptions_fail_closed() {
    let mut unknown = facts();
    unknown.confidence = ClassificationConfidence::Ambiguous;
    unknown.effects = set([ActionEffect::Unknown]);
    assert_eq!(
        resolve_policy(
            &unknown,
            &PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            10
        )
        .decision,
        PolicyDecision::Ask
    );

    let mut outside = facts();
    outside.effects = set([ActionEffect::Modify]);
    outside.scope = ActionScope::ExternalLocal;
    assert_eq!(
        resolve_policy(
            &outside,
            &PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            10
        )
        .decision,
        PolicyDecision::Ask
    );
    outside.explicit_scope_grant = true;
    assert_eq!(
        resolve_policy(
            &outside,
            &PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            10
        )
        .decision,
        PolicyDecision::Allow
    );

    let mut destructive = facts();
    destructive.scope = ActionScope::System;
    destructive.effects = set([ActionEffect::Delete]);
    destructive.reversibility = ActionReversibility::Destructive;
    assert_eq!(
        resolve_policy(
            &destructive,
            &PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            10
        )
        .decision,
        PolicyDecision::Deny
    );

    let mut publish = facts();
    publish.scope = ActionScope::Remote;
    publish.effects = set([ActionEffect::Publish]);
    publish.sensitivity = ActionSensitivity::Authenticated;
    publish.authenticated = true;
    publish.target_resolved = false;
    publish.canonical_target.clear();
    assert_eq!(
        resolve_policy(
            &publish,
            &PolicyConfiguration {
                preset: ApprovalPreset::ApproveForMe,
                ..PolicyConfiguration::default()
            },
            10
        )
        .decision,
        PolicyDecision::Ask
    );
}

#[test]
fn natural_language_chat_writes_apply_the_exact_repository_boundary() {
    let mut write = facts();
    write.action_kind = "file.modify".into();
    write.effects = set([ActionEffect::Modify]);
    write.request_origin = ActionRequestOrigin::NaturalLanguageChat;
    let automatic = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        ..PolicyConfiguration::default()
    };

    assert_eq!(
        resolve_policy(&write, &automatic, 10).decision,
        PolicyDecision::Allow
    );
    write.repository_state = RepositoryState::NotVersionControlled;
    assert_eq!(
        resolve_policy(&write, &automatic, 10).decision,
        PolicyDecision::Ask
    );
    write.repository_state = RepositoryState::VersionControlled;
    write.inside_active_project = false;
    assert_eq!(
        resolve_policy(&write, &automatic, 10).decision,
        PolicyDecision::Ask
    );

    write.inside_active_project = true;
    let explicit_ask = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        category_rules: vec![CategoryRule {
            id: "ask-writes".into(),
            group: PolicyGroup::WorkspaceFiles,
            decision: PolicyDecision::Ask,
            matcher: RuleMatcher {
                effects: set([ActionEffect::Modify]),
                ..RuleMatcher::default()
            },
        }],
        ..PolicyConfiguration::default()
    };
    assert_eq!(
        resolve_policy(&write, &explicit_ask, 10).decision,
        PolicyDecision::Ask
    );
}

#[test]
fn concrete_exception_is_exact_across_target_runtime_scope_and_duration() {
    let status = facts();
    let exception = ConcreteException::from_action(
        "one-read",
        PolicyDecision::Allow,
        &status,
        ExceptionDuration::Session {
            session_id: status.session_id.clone(),
        },
    );
    assert!(exception.matches(&status, 10));

    for mutated in [
        ActionFacts {
            canonical_target: "workspace:/project/other".into(),
            ..status.clone()
        },
        ActionFacts {
            runtime_id: "pi@1".into(),
            ..status.clone()
        },
        ActionFacts {
            environment_id: "docker:dev".into(),
            ..status.clone()
        },
        ActionFacts {
            session_id: "session-2".into(),
            ..status.clone()
        },
        ActionFacts {
            effects: set([ActionEffect::Read, ActionEffect::Capture]),
            ..status.clone()
        },
    ] {
        assert!(!exception.matches(&mutated, 10));
    }
    let expiring = ConcreteException::from_action(
        "short",
        PolicyDecision::Allow,
        &status,
        ExceptionDuration::Until { expires_at_ms: 20 },
    );
    assert!(expiring.matches(&status, 19));
    assert!(!expiring.matches(&status, 20));
}

#[test]
fn complete_r012_71_identity_corpus_is_parsed_and_fail_closed_when_unknown() {
    const EXPECTED: &[&str] = &[
        "terminal.project.read",
        "terminal.project.write",
        "terminal.external.read",
        "terminal.external.write",
        "terminal.network.read",
        "terminal.network.write",
        "terminal.unknown",
        "git.project.read",
        "git.external.read",
        "git.remote.read",
        "git.project.write",
        "git.external.write",
        "git.remote.write",
        "git.unknown",
        "file.project.read",
        "file.external.read",
        "file.project.write",
        "file.external.write",
        "file.project.delete",
        "file.external.delete",
        "file.project.watch",
        "file.external.watch",
        "file.unknown",
        "browser.load",
        "browser.post",
        "browser.navigation",
        "browser.use",
        "browser.read",
        "browser.capture",
        "browser.download",
        "browser.upload",
        "browser.local.read",
        "browser.auth.use",
        "browser.unknown",
        "network.read",
        "network.write",
        "network.listen",
        "network.connect",
        "network.unknown",
        "credential.use",
        "credential.create",
        "credential.update",
        "credential.delete",
        "credential.reveal",
        "credential.unknown",
        "process.read",
        "process.start",
        "process.control",
        "process.stop",
        "process.unknown",
        "clipboard.read",
        "clipboard.write",
        "dialog.open",
        "dialog.save",
        "notification.show",
        "system.read",
        "system.write",
        "app.open",
        "app.control",
        "config.read",
        "config.write",
        "plugin.read",
        "plugin.write",
        "extension.use",
        "mcp.read",
        "mcp.use",
        "mcp.write",
        "artifact.read",
        "artifact.write",
        "share.export",
        "unknown",
    ];
    let source = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wireframes/r012-cleanup/script.js"),
    )
    .unwrap();
    let start = source.find("//advanced policy grouping").unwrap();
    let relative_end = source[start..].find("  const options = [").unwrap();
    let keys = quoted_identity_keys(&source[start..start + relative_end]);
    assert_eq!(keys, EXPECTED);
    assert_eq!(keys.len(), 71);
    assert_eq!(keys.iter().collect::<BTreeSet<_>>().len(), 71);

    for key in keys {
        let converted = legacy_facts(&key);
        assert!(!converted.effects.is_empty(), "{key}");
        assert!(!converted.policy_groups().is_empty(), "{key}");
        if key == "unknown" || key.ends_with(".unknown") {
            assert_eq!(
                resolve_policy(
                    &converted,
                    &PolicyConfiguration {
                        preset: ApprovalPreset::ApproveForMe,
                        ..PolicyConfiguration::default()
                    },
                    10
                )
                .decision,
                PolicyDecision::Ask,
                "{key}"
            );
        }
    }
}

#[test]
fn authorization_binds_exact_action_and_burns_token_on_mutation_or_replay() {
    let original = action();
    let mut ledger = AuthorizationLedger::default();
    let token = ledger
        .issue_with_material(
            GateDecision::Allow,
            original.clone(),
            100,
            50,
            "authorization-1".into(),
            "unguessable-secret".into(),
        )
        .unwrap();
    let mut mutated = original.clone();
    mutated.arguments = json!({"content": "MUTATED", "path": "README.md"});
    assert_eq!(
        ledger.consume(&token, &mutated, live(&original), 110),
        Err(AuthorizationError::BindingMismatch(
            AuthorizationInvalidation::Arguments
        ))
    );
    assert!(matches!(
        ledger.record("authorization-1").unwrap().state,
        AuthorizationState::Invalidated {
            reason: AuthorizationInvalidation::Arguments,
            ..
        }
    ));
    assert_eq!(
        ledger.consume(&token, &original, live(&original), 111),
        Err(AuthorizationError::Invalidated)
    );

    let token = ledger
        .issue_with_material(
            GateDecision::Allow,
            original.clone(),
            200,
            50,
            "authorization-2".into(),
            "another-secret".into(),
        )
        .unwrap();
    let permit = ledger
        .consume(&token, &original, live(&original), 210)
        .unwrap();
    assert_eq!(permit.action(), &original);
    assert_eq!(
        ledger.consume(&token, permit.action(), live(permit.action()), 211),
        Err(AuthorizationError::Replay)
    );
}

#[test]
fn authorization_token_debug_is_redacted() {
    let token = authorization::AuthorizationToken::from_transport(
        "authorization-1",
        "do-not-print-this-secret",
    )
    .unwrap();
    let debug = format!("{token:?}");
    assert!(debug.contains("authorization-1"));
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("do-not-print-this-secret"));
}

#[test]
fn authorization_ledger_restores_valid_records_and_rejects_tampering_or_duplicates() {
    let original = action();
    let mut ledger = AuthorizationLedger::default();
    let token = ledger
        .issue_with_material(
            GateDecision::Allow,
            original.clone(),
            100,
            50,
            "restore-1".into(),
            "restore-secret".into(),
        )
        .unwrap();
    let records = ledger.records().cloned().collect::<Vec<_>>();
    let mut restored = AuthorizationLedger::restore(records.clone()).unwrap();
    let transported = authorization::AuthorizationToken::from_transport(
        token.authorization_id.clone(),
        token.secret_for_transport(),
    )
    .unwrap();
    assert!(
        restored
            .consume(&transported, &original, live(&original), 110)
            .is_ok()
    );
    let consumed_records = restored.records().cloned().collect::<Vec<_>>();
    let mut restarted = AuthorizationLedger::restore(consumed_records).unwrap();
    assert_eq!(
        restarted.consume(&transported, &original, live(&original), 111),
        Err(AuthorizationError::Replay)
    );

    let mut tampered = records[0].clone();
    tampered.binding_digest = "sha256:tampered".into();
    assert!(matches!(
        AuthorizationLedger::restore([tampered]),
        Err(AuthorizationError::PersistedDigestMismatch)
    ));
    assert!(matches!(
        AuthorizationLedger::restore([records[0].clone(), records[0].clone()]),
        Err(AuthorizationError::DuplicateAuthorization)
    ));
}

#[test]
fn authorization_rejects_every_required_exact_binding_substitution() {
    let original = action();
    let mutations: Vec<(CanonicalAction, AuthorizationInvalidation)> = vec![
        (
            CanonicalAction {
                canonical_target: "workspace:/project/other.txt".into(),
                ..original.clone()
            },
            AuthorizationInvalidation::Target,
        ),
        (
            CanonicalAction {
                workspace_id: "workspace-2".into(),
                ..original.clone()
            },
            AuthorizationInvalidation::Workspace,
        ),
        (
            CanonicalAction {
                session_id: "session-2".into(),
                ..original.clone()
            },
            AuthorizationInvalidation::Session,
        ),
        (
            CanonicalAction {
                runtime_id: "pi@1".into(),
                ..original.clone()
            },
            AuthorizationInvalidation::Runtime,
        ),
        (
            CanonicalAction {
                environment_id: "docker:dev".into(),
                ..original.clone()
            },
            AuthorizationInvalidation::Environment,
        ),
        (
            CanonicalAction {
                process_generation: original.process_generation + 1,
                ..original.clone()
            },
            AuthorizationInvalidation::ProcessGeneration,
        ),
        (
            CanonicalAction {
                requested_authority: set(["workspace-files.delete".into()]),
                ..original.clone()
            },
            AuthorizationInvalidation::RequestedAuthority,
        ),
    ];
    for (index, (mutation, expected)) in mutations.into_iter().enumerate() {
        let mut ledger = AuthorizationLedger::default();
        let token = ledger
            .issue_with_material(
                GateDecision::Allow,
                original.clone(),
                100,
                50,
                format!("exact-{index}"),
                format!("exact-secret-{index}"),
            )
            .unwrap();
        assert_eq!(
            ledger.consume(&token, &mutation, live(&original), 110),
            Err(AuthorizationError::BindingMismatch(expected))
        );
    }
}

#[test]
fn authorization_expires_cancels_revokes_and_invalidates_on_every_live_version() {
    let base = action();
    let cases = [
        (
            LiveAuthorityState {
                process_generation: base.process_generation + 1,
                ..live(&base)
            },
            AuthorizationInvalidation::ProcessGeneration,
        ),
        (
            LiveAuthorityState {
                configuration_version: base.configuration_version + 1,
                ..live(&base)
            },
            AuthorizationInvalidation::ConfigurationVersion,
        ),
        (
            LiveAuthorityState {
                policy_version: base.policy_version + 1,
                ..live(&base)
            },
            AuthorizationInvalidation::PolicyVersion,
        ),
        (
            LiveAuthorityState {
                revocation_epoch: base.revocation_epoch + 1,
                ..live(&base)
            },
            AuthorizationInvalidation::RevocationEpoch,
        ),
    ];
    for (index, (changed, expected)) in cases.into_iter().enumerate() {
        let mut ledger = AuthorizationLedger::default();
        let token = ledger
            .issue_with_material(
                GateDecision::Allow,
                base.clone(),
                100,
                50,
                format!("changed-{index}"),
                format!("secret-{index}"),
            )
            .unwrap();
        assert_eq!(
            ledger.consume(&token, &base, changed, 110),
            Err(AuthorizationError::BindingMismatch(expected))
        );
    }

    let mut ledger = AuthorizationLedger::default();
    let expired = ledger
        .issue_with_material(
            GateDecision::Allow,
            base.clone(),
            100,
            10,
            "expired".into(),
            "expired-secret".into(),
        )
        .unwrap();
    assert_eq!(
        ledger.consume(&expired, &base, live(&base), 110),
        Err(AuthorizationError::Expired)
    );

    let cancelled = ledger
        .issue_with_material(
            GateDecision::Allow,
            base.clone(),
            200,
            50,
            "cancelled".into(),
            "cancelled-secret".into(),
        )
        .unwrap();
    assert_eq!(ledger.cancel_run(&base.run_id, 201), 1);
    assert_eq!(
        ledger.consume(&cancelled, &base, live(&base), 202),
        Err(AuthorizationError::Cancelled)
    );

    let mut other_run = base.clone();
    other_run.run_id = "run-2".into();
    let revoked = ledger
        .issue_with_material(
            GateDecision::Allow,
            other_run.clone(),
            300,
            50,
            "revoked".into(),
            "revoked-secret".into(),
        )
        .unwrap();
    assert!(ledger.revoke_authorization("revoked", 301));
    assert_eq!(
        ledger.consume(&revoked, &other_run, live(&other_run), 302),
        Err(AuthorizationError::Revoked)
    );
    assert_eq!(
        ledger.issue(GateDecision::Ask, base, 400, 50),
        Err(AuthorizationError::DecisionDoesNotAllow)
    );
}

#[test]
fn approvals_serialize_within_run_and_are_visible_for_independent_runs() {
    let mut queue = ApprovalQueue::default();
    let first = action();
    let mut second = first.clone();
    second.action_id = "action-2".into();
    second.tool_call_id = "call-2".into();
    let mut independent = first.clone();
    independent.action_id = "action-3".into();
    independent.tool_call_id = "call-3".into();
    independent.run_id = "run-2".into();

    queue.enqueue("prompt-1", first, 100, 100).unwrap();
    queue.enqueue("prompt-2", second, 101, 100).unwrap();
    queue.enqueue("prompt-3", independent, 102, 100).unwrap();
    assert_eq!(
        queue.prompt("prompt-1").unwrap().state,
        ApprovalPromptState::Pending
    );
    assert_eq!(
        queue.prompt("prompt-2").unwrap().state,
        ApprovalPromptState::Queued
    );
    assert_eq!(
        queue.prompt("prompt-3").unwrap().state,
        ApprovalPromptState::Pending
    );
    assert_eq!(
        queue
            .visible_queue()
            .iter()
            .map(|record| record.prompt_id.as_str())
            .collect::<Vec<_>>(),
        ["prompt-1", "prompt-2", "prompt-3"]
    );

    queue
        .answer("prompt-1", ApprovalAnswer::Allow, 110)
        .unwrap();
    assert_eq!(
        queue.prompt("prompt-2").unwrap().state,
        ApprovalPromptState::Pending
    );
    queue.complete("prompt-1", 115).unwrap();
    assert!(matches!(
        queue.prompt("prompt-1").unwrap().state,
        ApprovalPromptState::Completed { .. }
    ));
    queue.answer("prompt-2", ApprovalAnswer::Deny, 120).unwrap();
    assert!(matches!(
        queue.prompt("prompt-2").unwrap().state,
        ApprovalPromptState::Denied { .. }
    ));
    assert_eq!(queue.cancel_run("run-2", 121), 1);
    assert!(matches!(
        queue.prompt("prompt-3").unwrap().state,
        ApprovalPromptState::Cancelled { .. }
    ));
}

#[test]
fn pending_approval_expires_on_ttl_or_target_version_change_and_promotes_its_run() {
    let mut queue = ApprovalQueue::default();
    let first = action();
    let mut next = first.clone();
    next.action_id = "action-next".into();
    next.tool_call_id = "call-next".into();
    queue.enqueue("stale", first.clone(), 100, 100).unwrap();
    queue.enqueue("next", next, 101, 100).unwrap();

    let mut changed = first;
    changed.target_version = "sha256:new".into();
    assert!(
        queue
            .revalidate_open_action("stale", &changed, 110)
            .unwrap()
    );
    assert!(matches!(
        queue.prompt("stale").unwrap().state,
        ApprovalPromptState::Expired { .. }
    ));
    assert_eq!(
        queue.prompt("next").unwrap().state,
        ApprovalPromptState::Pending
    );

    assert_eq!(
        queue.answer("next", ApprovalAnswer::Allow, 201),
        Err(authorization::ApprovalQueueError::Expired)
    );
    assert!(matches!(
        queue.prompt("next").unwrap().state,
        ApprovalPromptState::Expired { .. }
    ));
}

#[test]
fn approval_queue_restores_open_order_and_rejects_impossible_persisted_state() {
    let mut queue = ApprovalQueue::default();
    let first = action();
    let mut second = first.clone();
    second.action_id = "restore-action-2".into();
    second.tool_call_id = "restore-call-2".into();
    let mut independent = first.clone();
    independent.action_id = "restore-action-3".into();
    independent.tool_call_id = "restore-call-3".into();
    independent.run_id = "restore-run-2".into();
    queue.enqueue("restore-prompt-1", first, 100, 100).unwrap();
    queue.enqueue("restore-prompt-2", second, 101, 100).unwrap();
    queue
        .enqueue("restore-prompt-3", independent, 102, 100)
        .unwrap();
    queue
        .answer("restore-prompt-1", ApprovalAnswer::Allow, 110)
        .unwrap();

    let records = queue.prompts().cloned().collect::<Vec<_>>();
    let mut restored = ApprovalQueue::restore(records.clone()).unwrap();
    assert_eq!(
        restored
            .visible_queue()
            .iter()
            .map(|record| record.prompt_id.as_str())
            .collect::<Vec<_>>(),
        ["restore-prompt-2", "restore-prompt-3"]
    );
    let mut next = action();
    next.action_id = "restore-action-4".into();
    next.tool_call_id = "restore-call-4".into();
    let appended = restored
        .enqueue("restore-prompt-4", next, 120, 100)
        .unwrap();
    assert_eq!(appended.sequence, 3);

    let mut invalid = records;
    invalid
        .iter_mut()
        .find(|record| record.prompt_id == "restore-prompt-2")
        .unwrap()
        .state = ApprovalPromptState::Queued;
    assert!(matches!(
        ApprovalQueue::restore(invalid),
        Err(authorization::ApprovalQueueError::InvalidPersistedQueue)
    ));
}

#[test]
fn canonical_json_binding_is_stable_across_object_key_order() {
    let first = action();
    let mut second = first.clone();
    second.arguments = json!({"path": "README.md", "content": "hello"});
    assert_eq!(first, second);
    assert_eq!(
        first.binding_digest().unwrap(),
        second.binding_digest().unwrap()
    );
}

fn quoted_identity_keys(block: &str) -> Vec<String> {
    block
        .split('\'')
        .enumerate()
        .filter_map(|(index, value)| (index % 2 == 1).then_some(value))
        .filter(|value| {
            (value.contains('.') || *value == "unknown")
                && value
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character == '.')
        })
        .map(str::to_owned)
        .collect()
}

fn legacy_facts(key: &str) -> ActionFacts {
    let parts: Vec<_> = key.split('.').collect();
    let first = parts.first().copied().unwrap_or("unknown");
    let last = parts.last().copied().unwrap_or("unknown");
    let surface = match first {
        "terminal" => ActionSurface::Terminal,
        "git" => ActionSurface::Git,
        "file" => ActionSurface::File,
        "browser" => ActionSurface::Browser,
        "network" => ActionSurface::Network,
        "credential" => ActionSurface::Credential,
        "process" => ActionSurface::Process,
        "clipboard" | "dialog" | "notification" | "system" | "app" => ActionSurface::Desktop,
        "config" | "plugin" | "extension" | "mcp" | "artifact" | "share" => ActionSurface::C4os,
        other => ActionSurface::Unknown(other.into()),
    };
    let unknown = key == "unknown" || last == "unknown";
    let effect = if unknown {
        ActionEffect::Unknown
    } else {
        match last {
            "read" | "load" | "navigation" | "watch" => ActionEffect::Read,
            "capture" => ActionEffect::Capture,
            "write" | "update" => {
                if key == "git.remote.write" || key == "network.write" {
                    ActionEffect::Publish
                } else {
                    ActionEffect::Modify
                }
            }
            "create" | "download" | "show" => ActionEffect::Create,
            "delete" => ActionEffect::Delete,
            "start" => ActionEffect::Execute,
            "use" | "connect" | "control" | "stop" | "open" | "save" => ActionEffect::Control,
            "post" | "upload" | "export" => ActionEffect::Publish,
            "listen" => ActionEffect::Listen,
            "reveal" => ActionEffect::Reveal,
            _ => ActionEffect::Unknown,
        }
    };
    let mut scope = if parts.contains(&"external") {
        ActionScope::ExternalLocal
    } else if parts.contains(&"remote") || parts.contains(&"network") {
        ActionScope::Remote
    } else {
        ActionScope::Workspace
    };
    if matches!(surface, ActionSurface::Browser | ActionSurface::Network)
        && !parts.contains(&"local")
    {
        scope = ActionScope::Remote;
    }
    if key == "network.listen" {
        scope = ActionScope::ExternalLocal;
    } else if key == "system.write" {
        scope = ActionScope::System;
    } else if key == "share.export" {
        scope = ActionScope::Remote;
    } else if unknown {
        scope = ActionScope::Unknown;
    }
    let sensitivity = if matches!(surface, ActionSurface::Credential) {
        ActionSensitivity::Credential
    } else if parts.contains(&"auth") {
        ActionSensitivity::Authenticated
    } else {
        ActionSensitivity::Ordinary
    };
    ActionFacts {
        action_kind: key.into(),
        native_tool: key.into(),
        surface,
        effects: set([effect]),
        scope,
        initiator: ActionInitiator::Agent,
        sensitivity,
        reversibility: if matches!(last, "delete" | "stop") {
            ActionReversibility::Destructive
        } else if unknown {
            ActionReversibility::Unknown
        } else {
            ActionReversibility::Reversible
        },
        confidence: if unknown {
            ClassificationConfidence::Ambiguous
        } else {
            ClassificationConfidence::Known
        },
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: if unknown {
            RepositoryState::Unknown
        } else {
            RepositoryState::NotApplicable
        },
        inside_active_project: true,
        canonical_target: if unknown { "" } else { "workspace:/fixture" }.into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        runtime_id: "opencode@fixture".into(),
        environment_id: "local".into(),
        plugin_or_mcp_id: key.starts_with("plugin.").then(|| "plugin-1".into()),
        target_resolved: !unknown,
        authenticated: parts.contains(&"auth"),
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}
