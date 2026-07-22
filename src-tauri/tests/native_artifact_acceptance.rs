//! Opt-in seed for the exact Task 00008 macOS acceptance walkthrough.
//!
//! The seed creates only production Workspace/Project/Chat state. Artifacts
//! must be opened and changed by the rebuilt app through picker grants,
//! descriptor-rooted filesystem operations, and the Action Gateway.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use c4os_lib::core::configuration::{ManagedCeilings, SecurityConstraints};
use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::core::services::{
    create_workspace_from_project, restore_app_configuration, save_app_configuration,
};
use c4os_lib::core::workspace::{ArchiveLimits, C4osHomeLayout, WorkspaceLockOwner};
use serde_json::json;
use uuid::Uuid;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const ACCEPTANCE_HOME_ENV: &str = "C4OS_TASK8_ACCEPTANCE_HOME";
const PROJECT_ROOT_ENV: &str = "C4OS_TASK8_PROJECT_ROOT";
const TERMINAL_ACCEPTANCE_HOME_ENV: &str = "C4OS_TASK9_ACCEPTANCE_HOME";
const TERMINAL_PROJECT_ROOT_ENV: &str = "C4OS_TASK9_PROJECT_ROOT";
const BROWSER_ACCEPTANCE_HOME_ENV: &str = "C4OS_TASK10_ACCEPTANCE_HOME";
const BROWSER_PROJECT_ROOT_ENV: &str = "C4OS_TASK10_PROJECT_ROOT";
const BROWSER_EPHEMERAL_ACCEPTANCE_HOME_ENV: &str = "C4OS_TASK10_EPHEMERAL_ACCEPTANCE_HOME";
const BROWSER_EPHEMERAL_PROJECT_ROOT_ENV: &str = "C4OS_TASK10_EPHEMERAL_PROJECT_ROOT";

fn required_absolute_directory(name: &str) -> PathBuf {
    let value = env::var_os(name).unwrap_or_else(|| panic!("{name} is required"));
    let path = PathBuf::from(value);
    assert!(path.is_absolute(), "{name} must be absolute");
    fs::create_dir_all(&path).unwrap_or_else(|error| panic!("create {name}: {error}"));
    path.canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize {name}: {error}"))
}

fn require_private_home(path: &Path) {
    let mut permissions = fs::metadata(path)
        .expect("acceptance home metadata")
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).expect("private acceptance home mode");
    assert_eq!(
        fs::metadata(path)
            .expect("acceptance home metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700,
    );
}

#[allow(clippy::too_many_arguments)]
fn seed_native_workspace(
    home_root: &Path,
    project_root: &Path,
    project_name: &str,
    workspace_name: &str,
    chat_title: &str,
    lock_label: &str,
    archive_name: &str,
    app_configuration_text: &str,
) -> serde_json::Value {
    let home = C4osHomeLayout::new(home_root);
    let (app_database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(home.root())).expect("application database");
    let mut app_configuration = restore_app_configuration(
        &app_database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )
    .expect("application configuration");
    let now_seconds = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("current time")
            .as_secs(),
    )
    .expect("current time fits i64");
    let configuration_generation = app_configuration.snapshot().generation;
    save_app_configuration(
        &app_database,
        &home,
        &mut app_configuration,
        app_configuration_text,
        configuration_generation,
        now_seconds,
    )
    .expect("enable isolated restore");

    let mut workspace = create_workspace_from_project(
        &home,
        project_root,
        project_name,
        workspace_name,
        APP_VERSION,
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: u64::try_from(now_seconds).expect("positive time") * 1_000,
            label: lock_label.into(),
        },
        now_seconds,
    )
    .expect("create production Workspace");
    let project_id = workspace.manifest().projects[0].project_id;
    let chat_id = Uuid::new_v4();
    workspace
        .create_chat(project_id, chat_id, chat_title, now_seconds + 1)
        .expect("create active Chat");
    let archive = home_root.join(archive_name);
    workspace
        .save(
            &app_database,
            &archive,
            APP_VERSION,
            ArchiveLimits::default(),
            now_seconds + 2,
        )
        .expect("save acceptance Workspace");

    json!({
        "archive": archive,
        "chatId": chat_id,
        "home": home_root,
        "projectId": project_id,
        "projectRoot": project_root,
        "workspaceId": workspace.manifest().workspace_id,
    })
}

#[test]
#[ignore = "writes only to explicit isolated Task 00008 acceptance paths"]
fn seed_task_00008_native_workspace_without_artifacts() {
    let home_root = required_absolute_directory(ACCEPTANCE_HOME_ENV);
    let project_root = required_absolute_directory(PROJECT_ROOT_ENV);
    require_private_home(&home_root);
    fs::create_dir_all(project_root.join("docs/nested")).expect("nested Project folders");
    fs::write(
        project_root.join("docs/plan.md"),
        "# Task 00008 native file\n\nOriginal trusted-root content.\n",
    )
    .expect("File fixture");
    fs::write(
        project_root.join("docs/nested/notes.txt"),
        "Nested Folder Explorer fixture.\n",
    )
    .expect("nested File fixture");
    fs::write(
        project_root.join("README.md"),
        "Task 00008 isolated native acceptance Project.\n",
    )
    .expect("Project marker");

    println!(
        "{}",
        serde_json::to_string_pretty(&seed_native_workspace(
            &home_root,
            &project_root,
            "Artifact Acceptance Project",
            "Task 00008 Artifact Acceptance",
            "Inspect trusted artifacts",
            "task-00008-native-seed",
            "task-00008-acceptance.c4os.zip",
            "schema_version = 1\nrestore_last_workspace = true\n",
        ))
        .expect("acceptance seed report")
    );
}

#[test]
#[ignore = "writes only to explicit isolated Task 00009 acceptance paths"]
fn seed_task_00009_native_workspace_without_terminal_artifacts() {
    let home_root = required_absolute_directory(TERMINAL_ACCEPTANCE_HOME_ENV);
    let project_root = required_absolute_directory(TERMINAL_PROJECT_ROOT_ENV);
    require_private_home(&home_root);
    fs::create_dir_all(project_root.join("nested")).expect("nested Terminal fixture folder");
    fs::write(
        project_root.join("README.md"),
        "Task 00009 isolated native Terminal acceptance Project.\n",
    )
    .expect("Project marker");

    println!(
        "{}",
        serde_json::to_string_pretty(&seed_native_workspace(
            &home_root,
            &project_root,
            "Terminal Acceptance Project",
            "Task 00009 Terminal Acceptance",
            "Exercise the persistent Project shell",
            "task-00009-native-seed",
            "task-00009-acceptance.c4os.zip",
            "schema_version = 1\nrestore_last_workspace = true\n",
        ))
        .expect("acceptance seed report")
    );
}

#[test]
#[ignore = "writes only to explicit isolated Task 00010 acceptance paths"]
fn seed_task_00010_chat_browser_workspace_without_browser_artifacts() {
    let home_root = required_absolute_directory(BROWSER_ACCEPTANCE_HOME_ENV);
    let project_root = required_absolute_directory(BROWSER_PROJECT_ROOT_ENV);
    require_private_home(&home_root);
    fs::write(
        project_root.join("README.md"),
        "Task 00010 isolated native Browser acceptance Project.\n",
    )
    .expect("Project marker");

    println!(
        "{}",
        serde_json::to_string_pretty(&seed_native_workspace(
            &home_root,
            &project_root,
            "Browser Acceptance Project",
            "Task 00010 Browser Acceptance",
            "Exercise the native Browser facility",
            "task-00010-native-browser-seed",
            "task-00010-acceptance.c4os.zip",
            concat!(
                "schema_version = 1\n",
                "restore_last_workspace = true\n",
                "default_approval_preset = \"ask_for_approval\"\n",
                "browser_environment = \"chat\"\n",
            ),
        ))
        .expect("acceptance seed report")
    );
}

#[test]
#[ignore = "writes only to explicit isolated Task 00010 ephemeral acceptance paths"]
fn seed_task_00010_ephemeral_browser_workspace_without_browser_artifacts() {
    let home_root = required_absolute_directory(BROWSER_EPHEMERAL_ACCEPTANCE_HOME_ENV);
    let project_root = required_absolute_directory(BROWSER_EPHEMERAL_PROJECT_ROOT_ENV);
    require_private_home(&home_root);
    fs::write(
        project_root.join("README.md"),
        "Task 00010 isolated ephemeral native Browser acceptance Project.\n",
    )
    .expect("Project marker");

    println!(
        "{}",
        serde_json::to_string_pretty(&seed_native_workspace(
            &home_root,
            &project_root,
            "Ephemeral Browser Acceptance Project",
            "Task 00010 Ephemeral Browser Acceptance",
            "Exercise one ephemeral Browser",
            "task-00010-native-browser-ephemeral-seed",
            "task-00010-ephemeral-acceptance.c4os.zip",
            concat!(
                "schema_version = 1\n",
                "restore_last_workspace = true\n",
                "default_approval_preset = \"ask_for_approval\"\n",
                "browser_environment = \"none\"\n",
            ),
        ))
        .expect("acceptance seed report")
    );
}
