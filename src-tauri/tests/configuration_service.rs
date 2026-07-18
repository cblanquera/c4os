use c4os_lib::core::configuration::{
    ApprovalPreset, ConfigurationDiagnosticCode, ConfigurationError, ConfigurationEventDebouncer,
    ConfigurationScope, ConfigurationService, ConfigurationUpdate, ConfigurationWatcherNotice,
    ConfigurationWatcherPlan, MAX_CONFIGURATION_BYTES, ManagedCeilings, SecurityConstraints,
    StableReadPolicy, WatchedConfiguration, validate_scope_document,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;

fn service() -> ConfigurationService {
    ConfigurationService::new(ManagedCeilings::default(), SecurityConstraints::default())
}

fn reconcile(
    service: &mut ConfigurationService,
    scope: ConfigurationScope,
    text: &str,
) -> ConfigurationUpdate {
    service.reconcile_external_text(
        scope,
        format!("/configuration/{}.toml", scope.as_str()),
        text,
    )
}

#[test]
fn strict_schema_rejects_unknown_keys_secrets_and_unsupported_versions() {
    let mut configuration = service();

    let unknown = reconcile(
        &mut configuration,
        ConfigurationScope::App,
        "schema_version = 1\nunknown_key = true\n",
    );
    assert!(matches!(
        unknown,
        ConfigurationUpdate::Rejected {
            diagnostic,
            ..
        } if diagnostic.code == ConfigurationDiagnosticCode::InvalidToml
    ));

    let secret = reconcile(
        &mut configuration,
        ConfigurationScope::App,
        "schema_version = 1\n[provider]\napi_key = \"must-not-escape\"\n",
    );
    assert!(matches!(
        secret,
        ConfigurationUpdate::Rejected {
            diagnostic,
            ..
        } if diagnostic.code == ConfigurationDiagnosticCode::ForbiddenContent
            && diagnostic.key.as_deref() == Some("provider.api_key")
            && !diagnostic.message.contains("must-not-escape")
    ));

    let version = reconcile(
        &mut configuration,
        ConfigurationScope::App,
        "schema_version = 2\n",
    );
    assert!(matches!(
        version,
        ConfigurationUpdate::Rejected {
            diagnostic,
            ..
        } if diagnostic.code == ConfigurationDiagnosticCode::UnsupportedSchema
    ));
    assert_eq!(configuration.snapshot().generation, 0);
}

#[test]
fn reference_fields_reject_whitespace_controls_and_disguised_credentials() {
    let hostile_values = [
        ("model_route", "provider/sk-live-1234567890"),
        ("default_runtime", "runtime/token/opaque-value"),
        ("default_environment", "prod credential"),
        ("model_route", "eyJheader.payload.signature"),
    ];
    for (key, value) in hostile_values {
        let mut configuration = service();
        let text = format!("schema_version = 1\n{key} = {value:?}\n");
        let result = reconcile(&mut configuration, ConfigurationScope::App, &text);
        assert!(matches!(
            result,
            ConfigurationUpdate::Rejected {
                diagnostic,
                ..
            } if diagnostic.code == ConfigurationDiagnosticCode::InvalidValue
                && diagnostic.key.as_deref() == Some(key)
                && !diagnostic.message.contains(value)
        ));
    }
}

#[test]
fn scope_subsets_reject_app_owned_keys_outside_app() {
    for scope in [
        ConfigurationScope::Workspace,
        ConfigurationScope::Project,
        ConfigurationScope::Chat,
    ] {
        let mut configuration = service();
        let result = reconcile(
            &mut configuration,
            scope,
            "schema_version = 1\nrestore_last_workspace = true\n",
        );
        assert!(matches!(
            result,
            ConfigurationUpdate::Rejected {
                diagnostic,
                ..
            } if diagnostic.code == ConfigurationDiagnosticCode::InvalidScopeKey
                && diagnostic.key.as_deref() == Some("restore_last_workspace")
        ));
    }

    let mut configuration = service();
    assert!(matches!(
        reconcile(
            &mut configuration,
            ConfigurationScope::App,
            "schema_version = 1\nrestore_last_workspace = true\n",
        ),
        ConfigurationUpdate::Activated { .. }
    ));
}

#[test]
fn every_scope_enforces_its_distinct_allowed_key_matrix() {
    let fields = [
        ("restore_last_workspace", "restore_last_workspace = true"),
        ("model_route", "model_route = \"route\""),
        ("default_runtime", "default_runtime = \"opencode\""),
        ("default_environment", "default_environment = \"local\""),
        (
            "default_approval_preset",
            "default_approval_preset = \"custom\"",
        ),
        (
            "inherit_shell_environment",
            "inherit_shell_environment = true",
        ),
        (
            "shell_environment_allowlist",
            "shell_environment_allowlist = [\"PATH\"]",
        ),
        (
            "browser_environment",
            "browser_environment = \"workspace_project\"",
        ),
        (
            "requested_limits",
            "requested_limits = { parallel_actions = 1 }",
        ),
        (
            "approval_overrides",
            "approval_overrides = { filesystem = \"custom\" }",
        ),
        ("updates", "updates = { application = \"manual\" }"),
        ("diagnostics", "diagnostics = { retention_days = 1 }"),
    ];
    for scope in ConfigurationScope::PRECEDENCE {
        for (key, body) in fields {
            let text = format!("schema_version = 1\n{body}\n");
            let result = validate_scope_document(scope, "/config.toml", &text);
            assert_eq!(
                result.is_ok(),
                scope.allowed_keys().contains(&key),
                "unexpected {scope:?} result for {key}: {result:?}",
            );
        }
    }

    assert_ne!(
        ConfigurationScope::Workspace.allowed_keys(),
        ConfigurationScope::Project.allowed_keys()
    );
    assert_ne!(
        ConfigurationScope::Project.allowed_keys(),
        ConfigurationScope::Chat.allowed_keys()
    );
}

#[test]
fn precedence_replaces_scalars_and_lists_while_keyed_tables_merge() {
    let mut configuration = service();
    reconcile(
        &mut configuration,
        ConfigurationScope::App,
        r#"schema_version = 1
model_route = "app-route"
shell_environment_allowlist = ["APP_ONLY", "SHARED"]

[approval_overrides]
filesystem = "approve_safe_actions"
"#,
    );
    reconcile(
        &mut configuration,
        ConfigurationScope::Workspace,
        r#"schema_version = 1
model_route = "workspace-route"
shell_environment_allowlist = ["WORKSPACE_ONLY"]

[approval_overrides]
network = "custom"
"#,
    );
    reconcile(
        &mut configuration,
        ConfigurationScope::Project,
        r#"schema_version = 1
model_route = "project-route"
shell_environment_allowlist = ["PROJECT_ONLY"]

[approval_overrides]
process = "approve_for_me"
"#,
    );
    reconcile(
        &mut configuration,
        ConfigurationScope::Chat,
        r#"schema_version = 1
model_route = "chat-route"

[approval_overrides]
filesystem = "ask_for_approval"
"#,
    );

    let snapshot = configuration.snapshot();
    let effective = snapshot.configuration.as_ref();
    assert_eq!(effective.model_route.as_deref(), Some("chat-route"));
    assert_eq!(effective.shell_environment_allowlist, ["PROJECT_ONLY"]);
    assert_eq!(
        effective.approval_overrides,
        BTreeMap::from([
            ("filesystem".to_owned(), ApprovalPreset::AskForApproval),
            ("network".to_owned(), ApprovalPreset::Custom),
            ("process".to_owned(), ApprovalPreset::ApproveForMe),
        ])
    );
    assert_eq!(snapshot.generation, 4);
}

#[test]
fn managed_ceilings_and_security_constraints_are_applied_last() {
    let mut configuration = ConfigurationService::new(
        ManagedCeilings {
            parallel_actions: 4,
            output_bytes: 1_024,
        },
        SecurityConstraints {
            forced_approval_preset: Some(ApprovalPreset::AskForApproval),
            shell_environment_allowed: true,
            permitted_shell_environment_names: Some(BTreeSet::from(["SAFE".to_owned()])),
            forced_approval_overrides: BTreeMap::from([(
                "network".to_owned(),
                ApprovalPreset::Custom,
            )]),
        },
    );
    reconcile(
        &mut configuration,
        ConfigurationScope::Project,
        r#"schema_version = 1
default_approval_preset = "approve_for_me"
inherit_shell_environment = true
shell_environment_allowlist = ["SAFE", "NOT_MANAGED"]

[requested_limits]
parallel_actions = 32
output_bytes = 999999

[approval_overrides]
network = "approve_for_me"
"#,
    );

    let snapshot = configuration.snapshot();
    let effective = snapshot.configuration.as_ref();
    assert_eq!(effective.parallel_actions, Some(4));
    assert_eq!(effective.output_bytes, Some(1_024));
    assert_eq!(
        effective.default_approval_preset,
        ApprovalPreset::AskForApproval
    );
    assert_eq!(
        effective.approval_overrides["network"],
        ApprovalPreset::Custom
    );
    assert_eq!(effective.shell_environment_allowlist, ["SAFE"]);
}

#[test]
fn stale_ui_save_reports_changed_keys_without_touching_the_file() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("config.toml");
    fs::write(&path, "unchanged").expect("seed target");
    let mut configuration = service();
    reconcile(
        &mut configuration,
        ConfigurationScope::App,
        "schema_version = 1\nmodel_route = \"app\"\n",
    );
    let base_generation = configuration.snapshot().generation;
    reconcile(
        &mut configuration,
        ConfigurationScope::Workspace,
        "schema_version = 1\nmodel_route = \"workspace\"\n",
    );

    let error = configuration
        .save_scope_text(
            ConfigurationScope::App,
            &path,
            "schema_version = 1\nmodel_route = \"new-app\"\n",
            base_generation,
        )
        .expect_err("stale save must fail");
    let ConfigurationError::StaleGeneration(conflict) = error else {
        panic!("unexpected error: {error}");
    };
    assert_eq!(conflict.base_generation, base_generation);
    assert_eq!(conflict.active_generation, base_generation + 1);
    assert!(conflict.changed_keys.contains("workspace.model_route"));
    assert_eq!(fs::read_to_string(path).expect("target"), "unchanged");
}

#[test]
fn invalid_external_edit_retains_last_known_good_and_safe_diagnostic() {
    let mut configuration = service();
    reconcile(
        &mut configuration,
        ConfigurationScope::Project,
        "schema_version = 1\nmodel_route = \"known-good\"\n",
    );
    let before = configuration.snapshot();
    let canonical_before = configuration
        .last_known_good(ConfigurationScope::Project)
        .expect("last known good")
        .canonical_toml
        .clone();

    let rejected = reconcile(
        &mut configuration,
        ConfigurationScope::Project,
        "schema_version = 1\npassword = \"do-not-log-this\"\n",
    );
    assert!(matches!(rejected, ConfigurationUpdate::Rejected { .. }));
    let after = configuration.snapshot();
    assert_eq!(after.generation, before.generation);
    assert_eq!(after.configuration, before.configuration);
    assert_eq!(
        configuration
            .last_known_good(ConfigurationScope::Project)
            .expect("last known good")
            .canonical_toml,
        canonical_before
    );
    assert!(
        configuration
            .diagnostics()
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("do-not-log-this"))
    );
}

#[test]
fn ui_save_is_canonical_atomic_and_deduplicates_its_watcher_event() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("config.toml");
    let mut configuration = service().with_stable_read_policy(StableReadPolicy {
        max_attempts: 2,
        retry_delay: Duration::ZERO,
    });

    let saved = configuration
        .save_scope_text(
            ConfigurationScope::App,
            &path,
            "schema_version=1\nmodel_route=\"route\"\nrestore_last_workspace=true\n",
            0,
        )
        .expect("save");
    assert!(matches!(saved, ConfigurationUpdate::Activated { .. }));
    let canonical = configuration
        .last_known_good(ConfigurationScope::App)
        .expect("last known good")
        .canonical_toml
        .clone();
    assert_eq!(fs::read_to_string(&path).expect("saved file"), canonical);
    assert!(canonical.starts_with("schema_version = 1\n"));
    assert!(
        fs::read_dir(directory.path())
            .expect("directory")
            .all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
    );

    let generation = configuration.snapshot().generation;
    assert!(matches!(
        configuration.reload_scope(ConfigurationScope::App, &path),
        ConfigurationUpdate::DeduplicatedSelfWrite { .. }
    ));
    assert_eq!(configuration.snapshot().generation, generation);
}

#[test]
fn ui_save_revalidates_the_stable_disk_base_before_atomic_replace() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("config.toml");
    fs::write(&path, "schema_version = 1\nmodel_route = \"known-base\"\n")
        .expect("seed configuration");
    let mut configuration = service().with_stable_read_policy(StableReadPolicy {
        max_attempts: 2,
        retry_delay: Duration::ZERO,
    });
    assert!(matches!(
        configuration.reload_scope(ConfigurationScope::Project, &path),
        ConfigurationUpdate::Activated { .. }
    ));
    let base_generation = configuration.snapshot().generation;

    fs::write(
        &path,
        "schema_version = 1\nmodel_route = \"external-replacement\"\n",
    )
    .expect("external replacement");
    let error = configuration
        .save_scope_text(
            ConfigurationScope::Project,
            &path,
            "schema_version = 1\nmodel_route = \"ui-draft\"\n",
            base_generation,
        )
        .expect_err("external replacement must win the race");
    let ConfigurationError::ExternalReplacement(conflict) = error else {
        panic!("unexpected error: {error}");
    };
    assert_eq!(conflict.base_generation, base_generation);
    assert_eq!(conflict.active_generation, base_generation);
    assert!(conflict.changed_keys.contains("project.model_route"));
    assert_eq!(configuration.snapshot().generation, base_generation);
    assert!(
        fs::read_to_string(&path)
            .expect("external file retained")
            .contains("external-replacement")
    );
    assert!(
        fs::read_dir(directory.path())
            .expect("directory")
            .all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
    );
}

#[test]
fn stable_reload_rejects_oversized_files_before_activation() {
    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("config.toml");
    fs::write(&path, vec![b'x'; MAX_CONFIGURATION_BYTES + 1]).expect("oversized file");
    let mut configuration = service().with_stable_read_policy(StableReadPolicy {
        max_attempts: 2,
        retry_delay: Duration::ZERO,
    });
    let result = configuration.reload_scope(ConfigurationScope::Workspace, &path);
    assert!(matches!(
        result,
        ConfigurationUpdate::Rejected {
            diagnostic,
            ..
        } if diagnostic.code == ConfigurationDiagnosticCode::SizeLimitExceeded
    ));
    assert_eq!(configuration.snapshot().generation, 0);
}

#[test]
fn watcher_plan_targets_parent_directories_for_atomic_replacements() {
    let directory = tempdir().expect("temporary directory");
    let workspace_parent = directory.path().join("workspace");
    let chat_parent = directory.path().join("chat");
    fs::create_dir_all(&workspace_parent).expect("workspace parent");
    fs::create_dir_all(&chat_parent).expect("chat parent");
    let workspace_path = workspace_parent.join("config.toml");
    let chat_path = chat_parent.join("config.toml");

    let plan = ConfigurationWatcherPlan::new([
        WatchedConfiguration {
            scope: ConfigurationScope::Workspace,
            path: workspace_path.clone(),
        },
        WatchedConfiguration {
            scope: ConfigurationScope::Chat,
            path: chat_path.clone(),
        },
    ])
    .expect("watcher plan");

    assert_eq!(
        plan.parent_directories,
        BTreeSet::from([workspace_parent, chat_parent])
    );
    assert!(!plan.parent_directories.contains(&workspace_path));
    assert_eq!(plan.targets.len(), 2);
}

#[test]
fn watcher_debouncer_coalesces_bursts_and_deduplicates_targets() {
    let (sender, receiver) = mpsc::channel();
    let debouncer = ConfigurationEventDebouncer::start(Duration::from_millis(25), move |notice| {
        sender.send(notice).expect("notice receiver")
    });
    let workspace = WatchedConfiguration {
        scope: ConfigurationScope::Workspace,
        path: "/workspace/config.toml".into(),
    };
    let chat = WatchedConfiguration {
        scope: ConfigurationScope::Chat,
        path: "/workspace/chats/chat-1/config.toml".into(),
    };

    assert!(debouncer.push_changed([workspace.clone(), workspace.clone()]));
    assert!(debouncer.push_changed([chat.clone(), workspace.clone()]));

    let notice = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("one coalesced notice");
    let ConfigurationWatcherNotice::Changed(targets) = notice else {
        panic!("unexpected watcher notice");
    };
    assert_eq!(targets, vec![workspace, chat]);
    assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
}

#[test]
fn archive_validation_is_bounded_strict_and_does_not_activate_state() {
    let validated = validate_scope_document(
        ConfigurationScope::Workspace,
        "extracted/config/workspace.toml",
        r#"schema_version = 1
model_route = "workspace-route"
browser_environment = "workspace_project"
"#,
    )
    .expect("valid extracted scope");
    assert_eq!(validated.scope, ConfigurationScope::Workspace);
    assert!(validated.canonical_toml.starts_with("schema_version = 1\n"));
    assert_eq!(
        validated.document().model_route.as_deref(),
        Some("workspace-route")
    );

    let invalid_scope = validate_scope_document(
        ConfigurationScope::Project,
        "extracted/projects/project-1/config.toml",
        "schema_version = 1\nrestore_last_workspace = true\n",
    )
    .expect_err("app-owned key must fail Project archive validation");
    assert_eq!(
        invalid_scope.code,
        ConfigurationDiagnosticCode::InvalidScopeKey
    );

    let secret = validate_scope_document(
        ConfigurationScope::Workspace,
        "extracted/config/workspace.toml",
        "schema_version = 1\nprovider_secret = \"archive-secret\"\n",
    )
    .expect_err("secret-bearing archive configuration must fail");
    assert_eq!(secret.code, ConfigurationDiagnosticCode::ForbiddenContent);
    assert!(!secret.message.contains("archive-secret"));

    let oversized = format!(
        "schema_version = 1\n#{}",
        "x".repeat(MAX_CONFIGURATION_BYTES)
    );
    let diagnostic = validate_scope_document(
        ConfigurationScope::Chat,
        "extracted/chats/chat-1/config.toml",
        &oversized,
    )
    .expect_err("oversized scope must fail before parsing");
    assert_eq!(
        diagnostic.code,
        ConfigurationDiagnosticCode::SizeLimitExceeded
    );
    assert!(!diagnostic.message.contains(&"x".repeat(32)));
}

#[test]
fn restart_restore_revalidates_lkg_and_preserves_generation() {
    let mut before_restart = service();
    reconcile(
        &mut before_restart,
        ConfigurationScope::App,
        "schema_version = 1\nmodel_route = \"app-route\"\n",
    );
    reconcile(
        &mut before_restart,
        ConfigurationScope::Workspace,
        "schema_version = 1\nmodel_route = \"workspace-route\"\n",
    );
    let prior_snapshot = before_restart.snapshot();
    let records = before_restart.last_known_good_records();

    let mut restored = ConfigurationService::from_last_known_good(
        records.clone(),
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("restore canonical records");
    assert_eq!(restored.snapshot().generation, prior_snapshot.generation);
    assert_eq!(
        restored.snapshot().configuration,
        prior_snapshot.configuration
    );

    reconcile(
        &mut restored,
        ConfigurationScope::Project,
        "schema_version = 1\nmodel_route = \"project-route\"\n",
    );
    assert_eq!(
        restored.snapshot().generation,
        prior_snapshot.generation + 1
    );

    let mut corrupt = records;
    corrupt[0]
        .canonical_toml
        .push_str("provider_secret = \"must-not-leak\"\n");
    let diagnostic = match ConfigurationService::from_last_known_good(
        corrupt,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    ) {
        Ok(_) => panic!("corrupt persisted LKG must fail the complete restore"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.code,
        ConfigurationDiagnosticCode::ForbiddenContent
    );
    assert!(!diagnostic.message.contains("must-not-leak"));
}

#[test]
fn generation_overflow_fails_without_wrapping_or_writing() {
    let mut seed = service();
    reconcile(
        &mut seed,
        ConfigurationScope::App,
        "schema_version = 1\nmodel_route = \"seed\"\n",
    );
    let mut records = seed.last_known_good_records();
    records[0].activated_generation = u64::MAX;
    let mut restored = ConfigurationService::from_last_known_good(
        records,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("maximum generation restores deterministically");

    let result = reconcile(
        &mut restored,
        ConfigurationScope::Workspace,
        "schema_version = 1\nmodel_route = \"would-wrap\"\n",
    );
    assert!(matches!(
        result,
        ConfigurationUpdate::Rejected {
            diagnostic,
            ..
        } if diagnostic.code == ConfigurationDiagnosticCode::GenerationOverflow
    ));
    assert_eq!(restored.snapshot().generation, u64::MAX);

    let directory = tempdir().expect("temporary directory");
    let path = directory.path().join("config.toml");
    let error = restored
        .save_scope_text(
            ConfigurationScope::Workspace,
            &path,
            "schema_version = 1\nmodel_route = \"still-would-wrap\"\n",
            u64::MAX,
        )
        .expect_err("save cannot wrap generation");
    assert!(matches!(error, ConfigurationError::GenerationOverflow));
    assert!(!path.exists());
}
