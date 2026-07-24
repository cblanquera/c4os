use c4os_lib::browser::profile::{
    BrowserProfileRegistry, BrowserProfileRegistryError, PersistentProfileLifecycle,
    PersistentProfileScope, persistent_profile_scope,
};
use c4os_lib::core::configuration::{
    BrowserEnvironment, ConfigurationScope, ManagedCeilings, SecurityConstraints,
};
use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::core::services::{
    create_workspace_from_project, restore_app_configuration, save_app_configuration,
};
use c4os_lib::core::workspace::{C4osHomeLayout, WorkspaceLockOwner};
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

fn registry_path(temp: &TempDir) -> std::path::PathBuf {
    temp.path().join("home/browser/profiles.toml")
}

fn workspace_project(workspace: &str, project: &str) -> PersistentProfileScope {
    PersistentProfileScope::WorkspaceProject {
        workspace_id: workspace.into(),
        project_id: project.into(),
    }
}

fn chat(workspace: &str, chat: &str) -> PersistentProfileScope {
    PersistentProfileScope::Chat {
        workspace_id: workspace.into(),
        chat_id: chat.into(),
    }
}

#[test]
fn missing_registry_initializes_privately_and_resolution_is_stable_across_restart() {
    let temp = TempDir::new().expect("temporary directory");
    let path = registry_path(&temp);
    let mut registry = BrowserProfileRegistry::load(&path).expect("initialize registry");
    assert_eq!(registry.snapshot().generation, 1);
    assert!(registry.snapshot().profiles.is_empty());

    let first = registry
        .resolve(1, workspace_project("workspace-1", "project-1"))
        .expect("create profile");
    assert!(first.created);
    assert_eq!(first.registry_generation, 2);
    assert_eq!(first.profile.data_generation, 1);
    assert_eq!(first.profile.lifecycle, PersistentProfileLifecycle::Ready);
    drop(registry);

    let mut restored = BrowserProfileRegistry::load(&path).expect("restore registry");
    let second = restored
        .resolve(2, workspace_project("workspace-1", "project-1"))
        .expect("resolve stable profile");
    assert!(!second.created);
    assert_eq!(second.profile.profile_id, first.profile.profile_id);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&path)
                .expect("registry metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn persistent_scopes_are_isolated_and_none_has_no_persisted_variant() {
    let temp = TempDir::new().expect("temporary directory");
    let path = registry_path(&temp);
    let mut registry = BrowserProfileRegistry::load(&path).expect("registry");
    let app = registry
        .resolve(1, PersistentProfileScope::AppWide)
        .expect("app profile");
    let project_a = registry
        .resolve(2, workspace_project("workspace-1", "project-a"))
        .expect("project A");
    let project_b = registry
        .resolve(3, workspace_project("workspace-1", "project-b"))
        .expect("project B");
    let chat_a = registry
        .resolve(4, chat("workspace-1", "chat-a"))
        .expect("chat A");
    let chat_b = registry
        .resolve(5, chat("workspace-2", "chat-a"))
        .expect("same chat id in another Workspace");

    let ids = [
        app.profile.profile_id,
        project_a.profile.profile_id,
        project_b.profile.profile_id,
        chat_a.profile.profile_id,
        chat_b.profile.profile_id,
    ];
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        5
    );
    let contents = fs::read_to_string(path).expect("registry TOML");
    assert!(!contents.contains("none"));
    assert!(!contents.contains("ephemeral"));
}

#[test]
fn production_configuration_precedence_selects_the_exact_browser_profile_scope() {
    let temp = TempDir::new().expect("temporary directory");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project_root = temp.path().join("project");
    fs::create_dir_all(&project_root).expect("Project root");
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("application database");
    let mut app_configuration = restore_app_configuration(
        &app_database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("application configuration");
    let app_generation = app_configuration.snapshot().generation;
    save_app_configuration(
        &app_database,
        &home,
        &mut app_configuration,
        "schema_version = 1\nbrowser_environment = \"app_wide\"\n",
        app_generation,
        1_700_000_000,
    )
    .expect("save app Browser Environment");
    let app_record = app_configuration
        .last_known_good(ConfigurationScope::App)
        .cloned()
        .expect("app LKG");

    let mut workspace = create_workspace_from_project(
        &home,
        &project_root,
        "Browser Project",
        "Browser Workspace",
        env!("CARGO_PKG_VERSION"),
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: 1_700_000_000_000,
            label: "browser-configuration-precedence".into(),
        },
        1_700_000_000,
    )
    .expect("Workspace");
    let workspace_id = workspace.manifest().workspace_id;
    let project_id = workspace.manifest().projects[0].project_id;
    let chat_id = Uuid::new_v4();
    workspace
        .create_chat(project_id, chat_id, "Browser Chat", 1_700_000_001)
        .expect("Chat");

    let effective = |workspace: &c4os_lib::core::services::ActiveWorkspace| {
        workspace
            .restore_effective_configuration_snapshot(
                Some(app_record.clone()),
                Some(project_id),
                Some(chat_id),
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("effective configuration")
            .configuration
            .browser_environment
    };
    assert_eq!(effective(&workspace), BrowserEnvironment::AppWide);
    assert_eq!(
        persistent_profile_scope(
            effective(&workspace),
            &workspace_id.to_string(),
            &project_id.to_string(),
            &chat_id.to_string(),
        ),
        Some(PersistentProfileScope::AppWide)
    );

    let mut configuration = workspace
        .restore_configuration_stack(
            Some(project_id),
            Some(chat_id),
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("Workspace configuration stack");
    let generation = configuration.snapshot().generation;
    workspace
        .save_configuration_scope(
            &mut configuration,
            ConfigurationScope::Workspace,
            workspace_id,
            "schema_version = 1\nbrowser_environment = \"workspace_project\"\n",
            generation,
            1_700_000_002,
        )
        .expect("Workspace Browser Environment");
    assert_eq!(effective(&workspace), BrowserEnvironment::WorkspaceProject);

    let generation = configuration.snapshot().generation;
    workspace
        .save_configuration_scope(
            &mut configuration,
            ConfigurationScope::Project,
            project_id,
            "schema_version = 1\nbrowser_environment = \"none\"\n",
            generation,
            1_700_000_003,
        )
        .expect("Project Browser Environment");
    assert_eq!(effective(&workspace), BrowserEnvironment::None);
    assert_eq!(
        persistent_profile_scope(
            effective(&workspace),
            &workspace_id.to_string(),
            &project_id.to_string(),
            &chat_id.to_string(),
        ),
        None
    );

    let generation = configuration.snapshot().generation;
    workspace
        .save_configuration_scope(
            &mut configuration,
            ConfigurationScope::Chat,
            chat_id,
            "schema_version = 1\nbrowser_environment = \"chat\"\n",
            generation,
            1_700_000_004,
        )
        .expect("Chat Browser Environment");
    assert_eq!(effective(&workspace), BrowserEnvironment::Chat);
    assert_eq!(
        persistent_profile_scope(
            effective(&workspace),
            &workspace_id.to_string(),
            &project_id.to_string(),
            &chat_id.to_string(),
        ),
        Some(PersistentProfileScope::Chat {
            workspace_id: workspace_id.to_string(),
            chat_id: chat_id.to_string(),
        })
    );
}

#[test]
fn registry_and_profile_generations_are_compare_and_swap_boundaries() {
    let temp = TempDir::new().expect("temporary directory");
    let path = registry_path(&temp);
    let mut registry = BrowserProfileRegistry::load(path).expect("registry");
    let scope = chat("workspace-1", "chat-1");
    registry.resolve(1, scope.clone()).expect("profile");

    assert!(matches!(
        registry.resolve(1, scope.clone()),
        Err(BrowserProfileRegistryError::GenerationConflict {
            expected: 1,
            actual: 2
        })
    ));
    assert!(matches!(
        registry.mark_clear_pending(2, &scope, 2, Uuid::new_v4().to_string()),
        Err(BrowserProfileRegistryError::DataGenerationConflict {
            expected: 2,
            actual: 1
        })
    ));
}

#[test]
fn pending_clear_survives_restart_and_completes_the_exact_operation() {
    let temp = TempDir::new().expect("temporary directory");
    let path = registry_path(&temp);
    let scope = workspace_project("workspace-1", "project-1");
    let operation_id = Uuid::new_v4().to_string();
    let mut registry = BrowserProfileRegistry::load(&path).expect("registry");
    let resolved = registry.resolve(1, scope.clone()).expect("profile");
    let pending = registry
        .mark_clear_pending(2, &scope, 1, operation_id.clone())
        .expect("pending marker");
    assert_eq!(pending.profile_id, resolved.profile.profile_id);
    assert_eq!(
        pending.lifecycle,
        PersistentProfileLifecycle::ClearPending {
            operation_id: operation_id.clone(),
            target_data_generation: 2
        }
    );
    let repeated = registry
        .mark_clear_pending(3, &scope, 1, operation_id.clone())
        .expect("same pending clear is idempotent");
    assert_eq!(repeated, pending);
    assert_eq!(registry.snapshot().generation, 3);
    assert!(matches!(
        registry.mark_clear_pending(3, &scope, 1, Uuid::new_v4().to_string()),
        Err(BrowserProfileRegistryError::ClearAlreadyPending)
    ));
    drop(registry);

    let mut restored = BrowserProfileRegistry::load(&path).expect("restart");
    assert!(matches!(
        restored.complete_clear(2, &scope, &operation_id, 2),
        Err(BrowserProfileRegistryError::GenerationConflict {
            expected: 2,
            actual: 3
        })
    ));
    assert!(matches!(
        restored.complete_clear(3, &scope, &operation_id, 3),
        Err(BrowserProfileRegistryError::ClearOperationMismatch)
    ));
    assert!(matches!(
        restored.complete_clear(3, &scope, &Uuid::new_v4().to_string(), 2),
        Err(BrowserProfileRegistryError::ClearOperationMismatch)
    ));
    let completed = restored
        .complete_clear(3, &scope, &operation_id, 2)
        .expect("complete exact clear");
    assert_eq!(completed.profile_id, resolved.profile.profile_id);
    assert_eq!(completed.data_generation, 2);
    assert_eq!(completed.lifecycle, PersistentProfileLifecycle::Ready);

    let stable = restored.resolve(4, scope).expect("same stable identifier");
    assert_eq!(stable.profile.profile_id, resolved.profile.profile_id);
}

#[test]
fn external_replacement_is_detected_before_registry_mutation() {
    let temp = TempDir::new().expect("temporary directory");
    let path = registry_path(&temp);
    let mut registry = BrowserProfileRegistry::load(&path).expect("registry");
    let mut replacement = fs::read_to_string(&path).expect("canonical registry");
    replacement.push('\n');
    fs::write(&path, replacement).expect("external replacement");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private mode");
    }

    assert!(matches!(
        registry.resolve(1, PersistentProfileScope::AppWide),
        Err(BrowserProfileRegistryError::ExternalReplacement)
    ));
}

#[test]
fn hostile_toml_and_noncanonical_documents_fail_closed_without_replacement() {
    let cases = [
        "schema_version = 2\ngeneration = 1\nprofiles = []\n",
        "schema_version = 1\ngeneration = 0\nprofiles = []\n",
        "schema_version = 1\ngeneration = 1\nprofiles = []\nunknown = true\n",
        "schema_version = 1\ngeneration = 1\ngeneration = 2\nprofiles = []\n",
        "schema_version=1\ngeneration=1\nprofiles=[]\n",
    ];
    for (index, hostile) in cases.into_iter().enumerate() {
        let temp = TempDir::new().expect("temporary directory");
        let path = registry_path(&temp);
        fs::create_dir_all(path.parent().expect("parent")).expect("parent");
        fs::write(&path, hostile).expect("hostile registry");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private mode");
        }
        let before = fs::read(&path).expect("before");
        assert!(
            BrowserProfileRegistry::load(&path).is_err(),
            "hostile case {index} unexpectedly loaded"
        );
        assert_eq!(fs::read(&path).expect("after"), before);
    }
}

#[test]
fn duplicate_scope_and_profile_uuid_documents_are_rejected() {
    let profile_a = Uuid::new_v4().to_string();
    let profile_b = Uuid::new_v4().to_string();
    let duplicate_scope = format!(
        r#"schema_version = 1
generation = 3

[[profiles]]
profile_id = "{profile_a}"
data_generation = 1

[profiles.scope]
kind = "app_wide"

[profiles.lifecycle]
state = "ready"

[[profiles]]
profile_id = "{profile_b}"
data_generation = 1

[profiles.scope]
kind = "app_wide"

[profiles.lifecycle]
state = "ready"
"#
    );
    let duplicate_uuid = format!(
        r#"schema_version = 1
generation = 3

[[profiles]]
profile_id = "{profile_a}"
data_generation = 1

[profiles.scope]
kind = "app_wide"

[profiles.lifecycle]
state = "ready"

[[profiles]]
profile_id = "{profile_a}"
data_generation = 1

[profiles.scope]
kind = "chat"
workspace_id = "workspace-1"
chat_id = "chat-1"

[profiles.lifecycle]
state = "ready"
"#
    );

    for hostile in [duplicate_scope, duplicate_uuid] {
        let temp = TempDir::new().expect("temporary directory");
        let path = registry_path(&temp);
        fs::create_dir_all(path.parent().expect("parent")).expect("parent");
        fs::write(&path, hostile).expect("hostile registry");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private mode");
        }
        assert!(BrowserProfileRegistry::load(path).is_err());
    }
}

#[cfg(unix)]
#[test]
fn registry_rejects_symlinks_nonregular_files_and_public_permissions() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let temp = TempDir::new().expect("temporary directory");
    let target = temp.path().join("target.toml");
    fs::write(
        &target,
        "schema_version = 1\ngeneration = 1\nprofiles = []\n",
    )
    .expect("target");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).expect("target mode");
    let linked = registry_path(&temp);
    fs::create_dir_all(linked.parent().expect("parent")).expect("parent");
    symlink(&target, &linked).expect("registry symlink");
    assert!(matches!(
        BrowserProfileRegistry::load(&linked),
        Err(BrowserProfileRegistryError::SymbolicLink)
    ));

    fs::remove_file(&linked).expect("remove link");
    fs::create_dir(&linked).expect("nonregular registry");
    assert!(matches!(
        BrowserProfileRegistry::load(&linked),
        Err(BrowserProfileRegistryError::NonRegularFile)
    ));

    fs::remove_dir(&linked).expect("remove directory");
    fs::copy(&target, &linked).expect("copy registry");
    fs::set_permissions(&linked, fs::Permissions::from_mode(0o644)).expect("public mode");
    assert!(matches!(
        BrowserProfileRegistry::load(linked),
        Err(BrowserProfileRegistryError::UnsafePermissions)
    ));
}
