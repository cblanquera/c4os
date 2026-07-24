use std::fs;

use c4os_lib::core::configuration::{
    ConfigurationScope, ConfigurationService, ManagedCeilings, SecurityConstraints,
};
use c4os_lib::core::database::{
    ChatRecord, ConfigurationSnapshotRecord, DatabaseActor, DatabaseDescriptor, DatabaseSnapshot,
    LifecycleState, ProjectPathState, ProjectRecord, SnapshotQuery, WorkspaceRecord,
};
use c4os_lib::core::services::save_workspace_with_database;
use c4os_lib::core::workspace::{
    ArchiveLimits, C4osHomeLayout, OpenWorkspaceOutcome, WorkspaceLayout, WorkspaceLockOwner,
    WorkspaceWriterLock, WriterAccess, acquire_workspace_writer_lock, create_untitled_working_copy,
    open_workspace_archive, persist_committed_generation,
};
use tempfile::TempDir;
use uuid::Uuid;

const APP_VERSION: &str = "0.1.0";
const NOW: i64 = 1_721_312_000;

#[test]
fn same_project_round_trips_as_isolated_workspace_authority() {
    let temp = TempDir::new().expect("temporary root");
    let project = temp.path().join("external-project");
    fs::create_dir(&project).expect("external Project");
    fs::write(project.join("README.md"), b"project-owned source").expect("Project marker");

    let first = build_workspace(
        temp.path().join("first-working"),
        &project,
        "First Workspace",
        "first-route",
        "First private Chat",
    );
    let second = build_workspace(
        temp.path().join("second-working"),
        &project,
        "Second Workspace",
        "second-route",
        "Second private Chat",
    );

    let first_archive = temp.path().join("first.c4os.zip");
    let second_archive = temp.path().join("second.c4os.zip");
    let first_saved = save_workspace_with_database(
        &first.database,
        &first.writer_lock,
        &first.working_root,
        &first_archive,
        &first.manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("save first with online backup");
    let second_saved = save_workspace_with_database(
        &second.database,
        &second.writer_lock,
        &second.working_root,
        &second_archive,
        &second.manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("save second with online backup");

    assert_ne!(
        first_saved.manifest.workspace_id,
        second_saved.manifest.workspace_id
    );
    assert_eq!(
        first_saved.manifest.projects[0].last_known_path,
        second_saved.manifest.projects[0].last_known_path
    );
    drop(first.database);
    drop(second.database);

    let first_snapshot = reopen_snapshot(temp.path().join("first-home"), &first_archive, "first");
    let second_snapshot =
        reopen_snapshot(temp.path().join("second-home"), &second_archive, "second");

    assert_eq!(first_snapshot.chats[0].title, "First private Chat");
    assert_eq!(second_snapshot.chats[0].title, "Second private Chat");
    assert!(
        first_snapshot.configurations[0]
            .canonical_document
            .contains("first-route")
    );
    assert!(
        second_snapshot.configurations[0]
            .canonical_document
            .contains("second-route")
    );
    assert_ne!(
        first_snapshot
            .workspace
            .expect("first Workspace")
            .workspace_id,
        second_snapshot
            .workspace
            .expect("second Workspace")
            .workspace_id
    );

    assert_eq!(
        fs::read(project.join("README.md")).expect("Project marker remains"),
        b"project-owned source"
    );
    assert!(!project.join(".c4os").exists());
    assert_eq!(
        fs::read_dir(&project).expect("Project directory").count(),
        1,
        "C4OS must not write metadata into the external Project"
    );
}

struct BuiltWorkspace {
    working_root: std::path::PathBuf,
    manifest: c4os_lib::core::workspace::WorkspaceManifest,
    database: DatabaseActor,
    writer_lock: WorkspaceWriterLock,
}

fn build_workspace(
    working_root: std::path::PathBuf,
    project: &std::path::Path,
    workspace_name: &str,
    route: &str,
    chat_title: &str,
) -> BuiltWorkspace {
    let writer_lock = match acquire_workspace_writer_lock(
        &working_root.with_extension("lock"),
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: u64::try_from(NOW).expect("positive time") * 1_000,
            label: workspace_name.into(),
        },
    )
    .expect("Workspace writer lock")
    {
        WriterAccess::Writable(lock) => lock,
        WriterAccess::ReadOnly { .. } => panic!("fixture lock must be writable"),
    };
    let mut manifest = create_untitled_working_copy(
        &writer_lock,
        &working_root,
        project,
        "Shared Project",
        APP_VERSION,
    )
    .expect("untitled working copy");
    let workspace_id = manifest.workspace_id.to_string();
    let project_id = manifest.projects[0].project_id.to_string();
    let descriptor = DatabaseDescriptor::workspace(&working_root, &workspace_id);
    let (database, _) = DatabaseActor::start(descriptor).expect("Workspace database actor");
    database
        .create_workspace(WorkspaceRecord {
            workspace_id: workspace_id.clone(),
            display_name: workspace_name.into(),
            created_at: NOW,
            updated_at: NOW,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("Workspace record");
    database
        .add_project_and_chat(
            ProjectRecord {
                workspace_id: workspace_id.clone(),
                project_id: project_id.clone(),
                display_name: "Shared Project".into(),
                current_path: project.to_string_lossy().into_owned(),
                last_known_path: project.to_string_lossy().into_owned(),
                path_state: ProjectPathState::Found,
                position: 0,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            },
            ChatRecord {
                workspace_id: workspace_id.clone(),
                project_id,
                chat_id: Uuid::new_v4().to_string(),
                title: chat_title.into(),
                created_at: NOW,
                updated_at: NOW,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            },
        )
        .expect("atomic Project and Chat records");

    let configuration_path = WorkspaceLayout::new(&working_root).workspace_configuration();
    let mut configuration =
        ConfigurationService::new(ManagedCeilings::default(), SecurityConstraints::default());
    configuration
        .save_scope_text(
            ConfigurationScope::Workspace,
            &configuration_path,
            &format!("schema_version = 1\nmodel_route = \"{route}\"\n"),
            0,
        )
        .expect("configuration save");
    let last_known_good = configuration
        .last_known_good(ConfigurationScope::Workspace)
        .expect("last-known-good configuration");
    database
        .activate_configuration(ConfigurationSnapshotRecord {
            workspace_id: workspace_id.clone(),
            scope_kind: "workspace".into(),
            scope_id: workspace_id.clone(),
            canonical_document: last_known_good.canonical_toml.clone(),
            generation: last_known_good.activated_generation,
            activated_at: NOW,
        })
        .expect("persist last-known-good configuration");
    persist_committed_generation(&writer_lock, &working_root, &mut manifest)
        .expect("persist committed working-copy generation");

    BuiltWorkspace {
        working_root,
        manifest,
        database,
        writer_lock,
    }
}

fn reopen_snapshot(
    home_root: std::path::PathBuf,
    archive: &std::path::Path,
    label: &str,
) -> c4os_lib::core::database::WorkspaceSnapshot {
    let home = C4osHomeLayout::new(home_root);
    let opened = open_workspace_archive(
        &home,
        archive,
        APP_VERSION,
        ArchiveLimits::default(),
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: u64::try_from(NOW).expect("positive fixture time") * 1_000,
            label: label.into(),
        },
        |_, manifest, _| Ok(manifest.clone()),
    )
    .expect("open saved Workspace");
    let OpenWorkspaceOutcome::Writable(opened) = opened else {
        panic!("test owns the writer lock");
    };
    let descriptor = DatabaseDescriptor::workspace(
        &opened.working_root,
        opened.manifest.workspace_id.to_string(),
    );
    let (database, _) = DatabaseActor::start(descriptor).expect("reopen Workspace database");
    match database
        .snapshot(SnapshotQuery::new(50).expect("bounded query"))
        .expect("reconstructed snapshot")
    {
        DatabaseSnapshot::Workspace(snapshot) => snapshot,
        DatabaseSnapshot::App(_) => panic!("expected Workspace snapshot"),
    }
}
