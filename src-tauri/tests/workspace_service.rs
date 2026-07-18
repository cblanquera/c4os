use std::collections::BTreeSet;
use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use c4os_lib::core::configuration::{ConfigurationScope, ManagedCeilings, SecurityConstraints};
use c4os_lib::core::database::{
    DatabaseActor, DatabaseDescriptor, DatabaseSnapshot, SnapshotQuery,
};
use c4os_lib::core::services::{
    ManagedAppConfiguration, WorkspaceServiceOpen, create_workspace_from_completed_clone,
    create_workspace_from_project, load_workspace_start_state, open_workspace_with_database,
    restore_app_configuration, save_app_configuration, validate_workspace_semantics,
};
use c4os_lib::core::workspace::{
    ArchiveLimits, ArchiveViolation, C4osHomeLayout, WorkspaceError, WorkspaceLayout,
    WorkspaceLockOwner, WorkspaceSemanticValidationTarget,
};
use tempfile::TempDir;
use uuid::Uuid;

const APP_VERSION: &str = "0.1.0";
const NOW: i64 = 1_721_312_000;

fn owner(label: &str) -> WorkspaceLockOwner {
    WorkspaceLockOwner {
        process_id: std::process::id(),
        app_instance_id: Uuid::new_v4(),
        acquired_unix_ms: u64::try_from(NOW).expect("positive fixture time") * 1_000,
        label: label.into(),
    }
}

#[test]
fn production_service_round_trips_recovers_and_inactivates_without_project_writes() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    let second_project = temp.path().join("second-project");
    fs::create_dir_all(&project).expect("Project");
    fs::create_dir_all(&second_project).expect("second Project");
    fs::write(project.join("README.md"), b"external Project").expect("Project marker");

    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let mut app_configuration = restore_app_configuration(
        &app_database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("empty app configuration");
    save_app_configuration(
        &app_database,
        &home,
        &mut app_configuration,
        "schema_version = 1\nrestore_last_workspace = true\n",
        0,
        NOW,
    )
    .expect("save validated app configuration");
    let restored_app_configuration = restore_app_configuration(
        &app_database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("restart app configuration");
    assert!(
        restored_app_configuration
            .snapshot()
            .configuration
            .restore_last_workspace
    );
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Lifecycle Fixture",
        APP_VERSION,
        owner("create"),
        NOW,
    )
    .expect("create Workspace from granted folder");
    let first_project_id = active.manifest().projects[0].project_id;
    assert!(active.is_project_trusted(first_project_id));
    let mut workspace_configuration = active
        .restore_configuration_stack(
            None,
            None,
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("restart Workspace configuration");
    let workspace_configuration_generation = workspace_configuration.snapshot().generation;
    active
        .save_configuration_scope(
            &mut workspace_configuration,
            ConfigurationScope::Project,
            first_project_id,
            "schema_version = 1\nmodel_route = \"route.project\"\n",
            workspace_configuration_generation,
            NOW + 1,
        )
        .expect("save validated Project configuration");
    let restored_project_configuration = active
        .restore_configuration_stack(
            Some(first_project_id),
            None,
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("restart Project configuration");
    assert_eq!(
        restored_project_configuration
            .snapshot()
            .configuration
            .model_route
            .as_deref(),
        Some("route.project")
    );

    active
        .add_project(&second_project, "Second Project", true)
        .expect("add Project");
    let snapshot = active
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("snapshot");
    let second_project_id = Uuid::parse_str(&snapshot.projects[1].project_id).expect("Project ID");
    active
        .reorder_projects(&[second_project_id, first_project_id])
        .expect("reorder Projects");
    let chat_id = Uuid::new_v4();
    active
        .create_chat(first_project_id, chat_id, "Retained Chat", NOW + 1)
        .expect("create Chat");
    active
        .inactivate_chat(chat_id, NOW + 2)
        .expect("inactivate Chat");
    let retained = active
        .snapshot(SnapshotQuery::new(20).expect("query").including_inactive())
        .expect("retained snapshot");
    assert_eq!(retained.chats.len(), 1);
    assert!(
        active
            .snapshot(SnapshotQuery::new(20).expect("query"))
            .expect("active snapshot")
            .chats
            .is_empty()
    );

    let archive = temp.path().join("lifecycle.c4os.zip");
    let saved = active
        .save(
            &app_database,
            &archive,
            APP_VERSION,
            ArchiveLimits::default(),
            NOW + 3,
        )
        .expect("save Workspace through writer barrier");
    assert_eq!(saved.manifest.generation, retained.generation);
    assert_eq!(
        load_workspace_start_state(&app_database)
            .expect("Start")
            .recents
            .len(),
        1
    );
    drop(active);

    let opened = open_workspace_with_database(
        &app_database,
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        owner("open"),
        NOW + 3,
    )
    .expect("semantic open");
    let WorkspaceServiceOpen::Writable(mut opened) = opened else {
        panic!("test owns the writer lock");
    };
    assert_eq!(opened.manifest().projects[0].project_id, second_project_id);
    let generation_before_unsaved_change = opened
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("opened snapshot")
        .generation;
    opened
        .inactivate_project(second_project_id, NOW + 4)
        .expect("unsaved authoritative mutation");
    let newer_generation = opened
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("newer snapshot")
        .generation;
    assert!(newer_generation > generation_before_unsaved_change);
    let recovered_projection = validate_workspace_semantics(
        opened.working_root(),
        opened.manifest(),
        WorkspaceSemanticValidationTarget::ActiveRecovery,
    )
    .expect("active recovery semantics");
    assert_eq!(recovered_projection.generation, newer_generation);
    drop(opened);

    let recovered = open_workspace_with_database(
        &app_database,
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        owner("recover"),
        NOW + 4,
    )
    .expect("recover newer working copy");
    let WorkspaceServiceOpen::Writable(mut recovered) = recovered else {
        panic!("recovery owns the writer lock");
    };
    let notice = recovered.recovery_notice().expect("recovery notice");
    assert_eq!(notice.working_generation, newer_generation);
    assert!(notice.working_generation > notice.archive_generation);
    recovered
        .inactivate_workspace(&app_database, NOW + 5)
        .expect("inactivate Workspace and recent");
    assert!(
        load_workspace_start_state(&app_database)
            .expect("Start")
            .recents
            .is_empty()
    );

    let DatabaseSnapshot::App(app_snapshot) = app_database
        .snapshot(SnapshotQuery::new(20).expect("query").including_inactive())
        .expect("app snapshot")
    else {
        panic!("app snapshot");
    };
    assert_eq!(app_snapshot.recents.len(), 1, "inactive recent is retained");
    assert!(
        archive.is_file(),
        "inactivation does not delete the archive"
    );
    assert_eq!(
        fs::read(project.join("README.md")).expect("Project marker"),
        b"external Project"
    );
    assert!(!project.join(".c4os").exists());
}

#[test]
fn completed_clone_enters_the_same_safe_working_copy_path() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("clone-home"));
    let cloned = temp.path().join("completed-clone");
    fs::create_dir_all(cloned.join(".git")).expect("completed clone fixture");

    let active = create_workspace_from_completed_clone(
        &home,
        &cloned,
        "Cloned Project",
        "Clone Fixture",
        APP_VERSION,
        owner("clone"),
        NOW,
    )
    .expect("completed clone enters Workspace service");
    assert_eq!(active.manifest().projects.len(), 1);
    assert!(active.is_project_trusted(active.manifest().projects[0].project_id));
    assert!(!cloned.join(".c4os").exists());
}

#[test]
fn service_bookkeeping_failure_rolls_back_archive_save_and_active_open() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("save-home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Commit Guard Fixture",
        APP_VERSION,
        owner("save-guard"),
        NOW,
    )
    .expect("create Workspace");
    let archive = temp.path().join("commit-guard.c4os.zip");
    active
        .save(
            &app_database,
            &archive,
            APP_VERSION,
            ArchiveLimits::default(),
            NOW,
        )
        .expect("seed validated archive");
    let prior_archive = fs::read(&archive).expect("prior archive bytes");

    let project_id = active.manifest().projects[0].project_id;
    active
        .create_chat(project_id, Uuid::new_v4(), "Unsaved Chat", NOW + 1)
        .expect("advance authoritative database");
    let wrong_database_root = temp.path().join("wrong-database-kind");
    let (wrong_database, _) = DatabaseActor::start(DatabaseDescriptor::workspace(
        &wrong_database_root,
        Uuid::new_v4().to_string(),
    ))
    .expect("wrong-kind database actor");
    active
        .save(
            &wrong_database,
            &archive,
            APP_VERSION,
            ArchiveLimits::default(),
            NOW + 1,
        )
        .expect_err("app-bookkeeping failure must abort archive replacement");
    assert_eq!(
        fs::read(&archive).expect("restored prior archive"),
        prior_archive,
        "a failed recent write cannot replace the authoritative archive"
    );

    let before_inactivation = active
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("active Workspace before compensated inactivation");
    active
        .inactivate_workspace(&wrong_database, NOW + 2)
        .expect_err("app-bookkeeping failure must compensate Workspace inactivation");
    let after_inactivation = active
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("compensated Workspace remains active");
    assert!(after_inactivation.workspace.is_some());
    assert!(after_inactivation.generation > before_inactivation.generation);
    assert!(
        active.is_project_trusted(project_id),
        "compensated inactivation retains runtime-local trust"
    );
    drop(active);

    let open_home = C4osHomeLayout::new(temp.path().join("open-home"));
    let failed_open = open_workspace_with_database(
        &wrong_database,
        &open_home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        owner("open-guard"),
        NOW + 2,
    );
    assert!(
        failed_open.is_err(),
        "app-bookkeeping failure must abort active promotion"
    );
    assert!(
        !open_home.active_workspace().exists(),
        "a failed recent write cannot leave a promoted active Workspace"
    );
}

#[test]
fn production_save_preflights_hostile_local_state_before_backup_or_temp_creation() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Production Preflight Fixture",
        APP_VERSION,
        owner("production-preflight"),
        NOW,
    )
    .expect("create Workspace");
    let hostile_cache = active
        .working_root()
        .join(format!("chats/{}/cache/oversized.bin", Uuid::new_v4()));
    fs::create_dir_all(hostile_cache.parent().expect("hostile cache parent"))
        .expect("hostile cache parent");
    fs::write(&hostile_cache, vec![7_u8; 4_096]).expect("hostile cache");

    let invalid_parent = temp.path().join("archive-parent-is-a-file");
    fs::write(&invalid_parent, b"must remain a file").expect("invalid archive parent fixture");
    let error = active
        .save(
            &app_database,
            &invalid_parent.join("workspace.c4os.zip"),
            APP_VERSION,
            ArchiveLimits {
                max_entry_bytes: 1_024,
                ..ArchiveLimits::default()
            },
            NOW + 1,
        )
        .expect_err("hostile local state must fail at source preflight");
    assert!(matches!(
        error,
        WorkspaceError::Archive(ArchiveViolation::EntrySizeExceeded(_))
    ));
    assert_eq!(
        fs::read(&invalid_parent).expect("archive parent fixture unchanged"),
        b"must remain a file",
        "preflight must run before archive parent creation and SQLite backup staging"
    );
}

#[test]
fn app_configuration_restart_recovers_the_editable_file_from_sqlite_lkg() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let mut configuration = restore_app_configuration(
        &database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("initial configuration");
    save_app_configuration(
        &database,
        &home,
        &mut configuration,
        "schema_version = 1\nrestore_last_workspace = true\n",
        0,
        NOW,
    )
    .expect("persist app configuration");
    let expected = fs::read(home.app_configuration()).expect("canonical editable file");
    fs::remove_file(home.app_configuration()).expect("simulate missing editable file at restart");

    let restored = restore_app_configuration(
        &database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("recover configuration at restart");
    assert_eq!(
        fs::read(home.app_configuration()).expect("recovered file"),
        expected
    );
    assert_eq!(restored.snapshot().generation, 1);
    assert!(restored.snapshot().configuration.restore_last_workspace);
}

#[test]
fn managed_app_configuration_watcher_activates_and_persists_external_replacements() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let database = Arc::new(database);
    let coordinator = ManagedAppConfiguration::start(
        Arc::clone(&database),
        home.clone(),
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("managed app configuration");
    let replacement = home.root().join("config.external-replacement.toml");
    fs::write(
        &replacement,
        "schema_version = 1\nrestore_last_workspace = true\n",
    )
    .expect("write external replacement");
    fs::rename(&replacement, home.app_configuration()).expect("activate external replacement");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = coordinator.snapshot().expect("coordinator snapshot");
        let persisted = database
            .app_configuration_lkg()
            .expect("read persisted app configuration");
        if snapshot.configuration.restore_last_workspace
            && persisted.as_ref().is_some_and(|record| {
                record.generation == snapshot.generation
                    && record
                        .canonical_document
                        .contains("restore_last_workspace = true")
            })
        {
            break;
        }
        if Instant::now() >= deadline {
            panic!(
                "watcher did not converge file, memory, and SQLite LKG: generation={}, effective={}, persisted={persisted:?}, last_error={:?}, file={:?}",
                snapshot.generation,
                snapshot.configuration.restore_last_workspace,
                coordinator.last_error(),
                fs::read_to_string(home.app_configuration()),
            );
        }
        thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(coordinator.last_error(), None);
}

#[test]
fn semantic_validation_requires_exact_file_to_sqlite_lkg_scope_reconciliation() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Configuration Authority Fixture",
        APP_VERSION,
        owner("configuration-authority"),
        NOW,
    )
    .expect("create Workspace");
    let layout = WorkspaceLayout::new(active.working_root());
    let workspace_configuration = layout.workspace_configuration();
    let durable_workspace_document =
        fs::read(&workspace_configuration).expect("durable Workspace configuration");

    fs::write(
        &workspace_configuration,
        "schema_version = 1\ndefault_runtime = \"external-runtime\"\n",
    )
    .expect("replace Workspace configuration without LKG persistence");
    assert!(
        validate_workspace_semantics(
            active.working_root(),
            active.manifest(),
            WorkspaceSemanticValidationTarget::ActiveRecovery,
        )
        .is_err(),
        "valid but divergent file content must not bypass SQLite LKG authority"
    );
    fs::write(&workspace_configuration, &durable_workspace_document)
        .expect("restore Workspace configuration");

    let project_id = active.manifest().projects[0].project_id;
    let project_configuration = layout.project_configuration(project_id);
    fs::create_dir_all(
        project_configuration
            .parent()
            .expect("Project configuration parent"),
    )
    .expect("Project configuration parent");
    fs::write(
        &project_configuration,
        "schema_version = 1\nmodel_route = \"unowned.route\"\n",
    )
    .expect("unowned Project configuration");
    assert!(
        validate_workspace_semantics(
            active.working_root(),
            active.manifest(),
            WorkspaceSemanticValidationTarget::ActiveRecovery,
        )
        .is_err(),
        "a configuration file without an LKG scope must fail"
    );
    fs::remove_file(&project_configuration).expect("remove unowned Project configuration");

    let mut service = active
        .restore_configuration_stack(
            Some(project_id),
            None,
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("restore configuration service");
    let generation = service.snapshot().generation;
    active
        .save_configuration_scope(
            &mut service,
            ConfigurationScope::Project,
            project_id,
            "schema_version = 1\nmodel_route = \"durable.route\"\n",
            generation,
            NOW + 1,
        )
        .expect("persist Project configuration LKG");
    fs::remove_file(&project_configuration).expect("remove LKG-owned Project configuration");
    assert!(
        validate_workspace_semantics(
            active.working_root(),
            active.manifest(),
            WorkspaceSemanticValidationTarget::ActiveRecovery,
        )
        .is_err(),
        "an LKG scope without its configuration file must fail"
    );
}

#[test]
fn managed_workspace_watcher_converges_workspace_project_and_chat_scopes() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Watcher Fixture",
        APP_VERSION,
        owner("workspace-watcher"),
        NOW,
    )
    .expect("create Workspace");
    let workspace_id = active.manifest().workspace_id;
    let project_id = active.manifest().projects[0].project_id;
    let chat_id = Uuid::new_v4();
    active
        .create_chat(project_id, chat_id, "Watched Chat", NOW + 1)
        .expect("create watched Chat identity");
    let layout = WorkspaceLayout::new(active.working_root());
    let replacements = [
        (
            layout.workspace_configuration(),
            "schema_version = 1\ndefault_runtime = \"watched.runtime\"\n",
        ),
        (
            layout.project_configuration(project_id),
            "schema_version = 1\nmodel_route = \"watched.project\"\n",
        ),
        (
            layout.chat_configuration(chat_id),
            "schema_version = 1\ndefault_environment = \"watched.chat\"\n",
        ),
    ];
    for (index, (path, text)) in replacements.iter().enumerate() {
        let replacement = path
            .parent()
            .expect("configuration parent")
            .join(format!(".external-{index}.toml"));
        fs::write(&replacement, text).expect("write external replacement");
        fs::rename(replacement, path).expect("activate external replacement");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = active
            .snapshot(SnapshotQuery::new(20).expect("query"))
            .expect("Workspace snapshot");
        if snapshot.configurations.len() == 3
            && snapshot.configurations.iter().any(|record| {
                record.scope_kind == "workspace"
                    && record.scope_id == workspace_id.to_string()
                    && record.canonical_document.contains("watched.runtime")
            })
            && snapshot.configurations.iter().any(|record| {
                record.scope_kind == "project"
                    && record.scope_id == project_id.to_string()
                    && record.canonical_document.contains("watched.project")
            })
            && snapshot.configurations.iter().any(|record| {
                record.scope_kind == "chat"
                    && record.scope_id == chat_id.to_string()
                    && record.canonical_document.contains("watched.chat")
            })
        {
            let generations: BTreeSet<_> = snapshot
                .configurations
                .iter()
                .map(|record| record.generation)
                .collect();
            assert_eq!(generations.len(), 3, "scope generations must be global");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Workspace configuration watcher did not converge every scope: {:?}",
            active.configuration_last_error()
        );
        thread::sleep(Duration::from_millis(25));
    }

    // Replace all three now-existing files again while preserving their exact
    // mtimes. The polling fallback must compare bounded content rather than
    // relying on notify's whole-second timestamp projection.
    let repeated_replacements = [
        (
            layout.workspace_configuration(),
            "schema_version = 1\ndefault_runtime = \"repeated.runtime\"\n",
        ),
        (
            layout.project_configuration(project_id),
            "schema_version = 1\nmodel_route = \"repeated.project\"\n",
        ),
        (
            layout.chat_configuration(chat_id),
            "schema_version = 1\ndefault_environment = \"repeated.chat\"\n",
        ),
    ];
    for (index, (path, text)) in repeated_replacements.iter().enumerate() {
        let modified = fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .expect("active configuration mtime");
        let replacement = path
            .parent()
            .expect("configuration parent")
            .join(format!(".repeated-{index}.toml"));
        fs::write(&replacement, text).expect("write repeated replacement");
        fs::File::options()
            .write(true)
            .open(&replacement)
            .and_then(|file| file.set_times(std::fs::FileTimes::new().set_modified(modified)))
            .expect("preserve repeated replacement mtime");
        fs::rename(replacement, path).expect("activate repeated replacement");
    }
    let repeated_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = active
            .snapshot(SnapshotQuery::new(20).expect("query"))
            .expect("repeated Workspace snapshot");
        if snapshot.configurations.iter().any(|record| {
            record.scope_kind == "workspace"
                && record.canonical_document.contains("repeated.runtime")
        }) && snapshot.configurations.iter().any(|record| {
            record.scope_kind == "project" && record.canonical_document.contains("repeated.project")
        }) && snapshot.configurations.iter().any(|record| {
            record.scope_kind == "chat" && record.canonical_document.contains("repeated.chat")
        }) {
            break;
        }
        assert!(
            Instant::now() < repeated_deadline,
            "repeated same-mtime replacements did not converge: {:?}",
            active.configuration_last_error()
        );
        thread::sleep(Duration::from_millis(25));
    }
    let effective = active
        .restore_configuration_stack(
            Some(project_id),
            Some(chat_id),
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("managed effective stack");
    let snapshot = effective.snapshot();
    assert_eq!(
        snapshot.configuration.default_runtime.as_deref(),
        Some("repeated.runtime")
    );
    assert_eq!(
        snapshot.configuration.model_route.as_deref(),
        Some("repeated.project")
    );
    assert_eq!(
        snapshot.configuration.default_environment.as_deref(),
        Some("repeated.chat")
    );
    assert_eq!(active.configuration_last_error(), None);
}

#[test]
fn active_save_rejects_a_valid_file_that_diverges_from_sqlite_lkg() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("app database actor");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Save Preflight Fixture",
        APP_VERSION,
        owner("save-preflight"),
        NOW,
    )
    .expect("create Workspace");
    fs::write(
        WorkspaceLayout::new(active.working_root()).workspace_configuration(),
        "schema_version = 1\ndefault_runtime = \"not-yet-durable\"\n",
    )
    .expect("write valid divergent configuration");
    let archive = temp.path().join("must-not-exist.c4os.zip");
    active
        .save(
            &app_database,
            &archive,
            APP_VERSION,
            ArchiveLimits::default(),
            NOW + 1,
        )
        .expect_err("save must reject file-to-LKG divergence before staging");
    assert!(!archive.exists());
}

#[test]
fn committed_chat_creation_reports_success_when_watcher_refresh_degrades() {
    let temp = TempDir::new().expect("temporary root");
    let home = C4osHomeLayout::new(temp.path().join("home"));
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("Project");
    let mut active = create_workspace_from_project(
        &home,
        &project,
        "Primary Project",
        "Refresh Failure Fixture",
        APP_VERSION,
        owner("refresh-failure"),
        NOW,
    )
    .expect("create Workspace");
    let project_id = active.manifest().projects[0].project_id;
    let chat_id = Uuid::new_v4();
    let chat_parent = WorkspaceLayout::new(active.working_root())
        .chat_configuration(chat_id)
        .parent()
        .expect("Chat configuration parent")
        .to_path_buf();
    fs::write(&chat_parent, b"blocks watcher target directory")
        .expect("inject watcher target refresh failure");

    let generation = active
        .create_chat(project_id, chat_id, "Durably Created", NOW + 1)
        .expect("durable Chat commit remains successful");
    let snapshot = active
        .snapshot(SnapshotQuery::new(20).expect("query"))
        .expect("Workspace snapshot");

    assert_eq!(snapshot.generation, generation);
    assert!(
        snapshot
            .chats
            .iter()
            .any(|chat| chat.chat_id == chat_id.to_string())
    );
    assert_eq!(
        active.configuration_last_error(),
        Some("Workspace configuration watcher target refresh failed")
    );
}
