use std::fs;

use c4os_lib::artifact::{
    ARTIFACT_SCHEMA_VERSION, ArtifactFocusCapability, ArtifactHistoryEntry, ArtifactHistoryKind,
    ArtifactLifecycle, ArtifactProviderDescriptor, ArtifactRecord, ArtifactSource, ArtifactState,
    ArtifactWorkspaceUiState, FileArtifactState, FileLiveVersion, FileResourceReference,
    FolderArtifactState, FolderBreadcrumb, FolderListingVersion, FolderResourceReference,
    UnknownArtifactState,
};
use c4os_lib::core::database::{
    self, AppConfigurationSnapshotRecord, DatabaseActor, DatabaseDescriptor, DatabaseSnapshot,
    InactiveEntity, InstallationRecord, LifecycleState, ProjectPathState, ProjectRecord,
    RecentWorkspaceRecord, SnapshotQuery, WorkspaceArtifactDocumentRecord,
    WorkspaceArtifactUiStateRecord, WorkspaceConversationStateRecord, WorkspaceRecord,
};
use c4os_lib::runtime::opencode_native::sha256_bytes;
use rusqlite::Connection;
use tempfile::TempDir;

const NOW: i64 = 1_721_260_800;

fn workspace_record(workspace_id: &str, display_name: &str) -> WorkspaceRecord {
    WorkspaceRecord {
        workspace_id: workspace_id.into(),
        display_name: display_name.into(),
        created_at: NOW,
        updated_at: NOW,
        lifecycle_state: LifecycleState::Active,
        inactivated_at: None,
    }
}

fn project_record(workspace_id: &str, project_id: &str, position: i64) -> ProjectRecord {
    ProjectRecord {
        workspace_id: workspace_id.into(),
        project_id: project_id.into(),
        display_name: format!("Project {project_id}"),
        current_path: format!("/projects/{project_id}"),
        last_known_path: format!("/projects/{project_id}"),
        path_state: ProjectPathState::Found,
        position,
        lifecycle_state: LifecycleState::Active,
        inactivated_at: None,
    }
}

fn chat_record(workspace_id: &str, project_id: &str, chat_id: &str) -> database::ChatRecord {
    database::ChatRecord {
        workspace_id: workspace_id.into(),
        project_id: project_id.into(),
        chat_id: chat_id.into(),
        title: format!("Chat {chat_id}"),
        created_at: NOW,
        updated_at: NOW,
        lifecycle_state: LifecycleState::Active,
        inactivated_at: None,
    }
}

fn file_artifact_document(
    artifact_id: &str,
    content: &str,
    record_revision: u64,
    updated_at_ms: u64,
) -> WorkspaceArtifactDocumentRecord {
    let provider = ArtifactProviderDescriptor::file();
    let artifact = ArtifactRecord {
        schema_version: ARTIFACT_SCHEMA_VERSION,
        artifact_id: artifact_id.into(),
        workspace_id: "workspace-1".into(),
        project_id: "project-1".into(),
        session_id: "chat-1".into(),
        provider: provider.clone(),
        source: ArtifactSource::DirectOperation {
            operation_id: "operation-1".into(),
        },
        record_revision,
        lifecycle: ArtifactLifecycle::Ready,
        state: ArtifactState::File(Box::new(
            FileArtifactState::new(
                FileResourceReference::new("notes.txt", "notes.txt").expect("file resource"),
                "notes.txt",
                "text/plain",
                content,
                FileLiveVersion::new(
                    record_revision,
                    sha256_bytes(content.as_bytes()),
                    content.len() as u64,
                    updated_at_ms,
                )
                .expect("live version"),
            )
            .expect("file state"),
        )),
        history: Vec::new(),
        created_at_ms: NOW as u64,
        updated_at_ms,
    };
    artifact.validate().expect("artifact record");
    WorkspaceArtifactDocumentRecord {
        workspace_id: artifact.workspace_id.clone(),
        project_id: artifact.project_id.clone(),
        session_id: artifact.session_id.clone(),
        artifact_id: artifact.artifact_id.clone(),
        provider_kind: artifact.provider.type_id.clone(),
        provider_version: artifact.provider.schema_version,
        state_schema_version: artifact.schema_version,
        revision: artifact.record_revision,
        canonical_document: serde_json::to_string(&artifact).expect("serialize artifact"),
        updated_at_ms: artifact.updated_at_ms,
    }
}

fn converted_folder_artifact_document(
    previous: &WorkspaceArtifactDocumentRecord,
    updated_at_ms: u64,
) -> WorkspaceArtifactDocumentRecord {
    let mut artifact = serde_json::from_str::<ArtifactRecord>(&previous.canonical_document)
        .expect("previous File artifact");
    let entries = Vec::new();
    artifact.provider = ArtifactProviderDescriptor::folder();
    artifact.state = ArtifactState::Folder(Box::new(
        FolderArtifactState::new(
            FolderResourceReference::new("", "Project").expect("folder resource"),
            vec![FolderBreadcrumb {
                label: "Project".into(),
                project_relative_path: String::new(),
            }],
            entries.clone(),
            FolderListingVersion::from_entries(2, &entries, updated_at_ms).expect("folder version"),
        )
        .expect("folder state"),
    ));
    artifact.record_revision = previous.revision + 1;
    artifact.updated_at_ms = updated_at_ms;
    let state_sha256 =
        sha256_bytes(&serde_json::to_vec(&artifact.state).expect("serialize converted state"));
    artifact
        .append_history(ArtifactHistoryEntry {
            record_revision: artifact.record_revision,
            resource_version: artifact.resource_version(),
            state_sha256,
            kind: ArtifactHistoryKind::Converted,
            recorded_at_ms: updated_at_ms,
        })
        .expect("explicit conversion history");
    artifact.validate().expect("converted artifact");
    WorkspaceArtifactDocumentRecord {
        workspace_id: artifact.workspace_id.clone(),
        project_id: artifact.project_id.clone(),
        session_id: artifact.session_id.clone(),
        artifact_id: artifact.artifact_id.clone(),
        provider_kind: artifact.provider.type_id.clone(),
        provider_version: artifact.provider.schema_version,
        state_schema_version: artifact.schema_version,
        revision: artifact.record_revision,
        canonical_document: serde_json::to_string(&artifact).expect("serialize converted artifact"),
        updated_at_ms,
    }
}

fn unsupported_provider_artifact_document(
    previous: &WorkspaceArtifactDocumentRecord,
    updated_at_ms: u64,
) -> WorkspaceArtifactDocumentRecord {
    let mut artifact = serde_json::from_str::<ArtifactRecord>(&previous.canonical_document)
        .expect("previous File artifact");
    let resource_version = artifact.resource_version();
    artifact.provider = ArtifactProviderDescriptor {
        schema_version: 2,
        type_id: "future-provider".into(),
        label: "Future".into(),
        accessible_name: "Future artifact".into(),
        focus: ArtifactFocusCapability::InlineOnly,
    };
    artifact.lifecycle = ArtifactLifecycle::UnknownVersion {
        provider_schema_version: 2,
    };
    artifact.state = ArtifactState::Unknown(UnknownArtifactState {
        provider_schema_version: 2,
        state_sha256: sha256_bytes(b"future-state"),
        resource_version,
    });
    artifact.record_revision = previous.revision + 1;
    artifact.updated_at_ms = updated_at_ms;
    artifact.validate().expect("unsupported provider artifact");
    WorkspaceArtifactDocumentRecord {
        workspace_id: artifact.workspace_id.clone(),
        project_id: artifact.project_id.clone(),
        session_id: artifact.session_id.clone(),
        artifact_id: artifact.artifact_id.clone(),
        provider_kind: artifact.provider.type_id.clone(),
        provider_version: artifact.provider.schema_version,
        state_schema_version: artifact.schema_version,
        revision: artifact.record_revision,
        canonical_document: serde_json::to_string(&artifact)
            .expect("serialize unsupported provider artifact"),
        updated_at_ms,
    }
}

fn workspace_snapshot(
    actor: &DatabaseActor,
    include_inactive: bool,
) -> database::WorkspaceSnapshot {
    let mut query = SnapshotQuery::new(100).expect("valid query");
    if include_inactive {
        query = query.including_inactive();
    }
    match actor.snapshot(query).expect("snapshot") {
        DatabaseSnapshot::Workspace(snapshot) => snapshot,
        DatabaseSnapshot::App(_) => panic!("expected Workspace snapshot"),
    }
}

#[test]
fn conversation_state_is_workspace_owned_cas_and_survives_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    {
        let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
        assert_eq!(report.current_version, 5);
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("seed Workspace");
        actor
            .save_conversation_state(
                WorkspaceConversationStateRecord {
                    workspace_id: "workspace-1".into(),
                    generation: 1,
                    canonical_document: r#"{"schemaVersion":1,"activeProjectId":null,"activeSessionId":null,"drafts":{}}"#.into(),
                    updated_at_ms: NOW as u64,
                },
                None,
            )
            .expect("first state");
        let stale = actor.save_conversation_state(
            WorkspaceConversationStateRecord {
                workspace_id: "workspace-1".into(),
                generation: 2,
                canonical_document: "{}".into(),
                updated_at_ms: NOW as u64 + 1,
            },
            None,
        );
        assert!(
            stale
                .expect_err("stale create must lose")
                .to_string()
                .contains("expected")
        );
        actor
            .save_conversation_state(
                WorkspaceConversationStateRecord {
                    workspace_id: "workspace-1".into(),
                    generation: 2,
                    canonical_document: r#"{"schemaVersion":1,"activeProjectId":null,"activeSessionId":null,"drafts":{"chat-1":{"prompt":"keep me","attachments":[],"providerId":null,"modelId":null,"reasoningMode":null,"mode":"chat","replyTargetId":null}}}"#.into(),
                    updated_at_ms: NOW as u64 + 2,
                },
                Some(1),
            )
            .expect("replacement state");
    }
    let (actor, report) = DatabaseActor::start(descriptor).expect("restart Workspace database");
    assert_eq!(report.previous_version, 5);
    let restored = actor
        .conversation_state()
        .expect("read state")
        .expect("state exists");
    assert_eq!(restored.generation, 2);
    assert!(restored.canonical_document.contains("keep me"));
}

#[test]
fn artifact_documents_are_versioned_workspace_owned_and_survive_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    let revision_one = file_artifact_document("artifact-1", "one", 1, NOW as u64);
    let revision_two = file_artifact_document("artifact-1", "two", 2, NOW as u64 + 1);
    {
        let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
        assert_eq!(report.current_version, 5);
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("seed Workspace");
        actor
            .add_project(project_record("workspace-1", "project-1", 0))
            .expect("seed Project");
        actor
            .add_chat(chat_record("workspace-1", "project-1", "chat-1"))
            .expect("seed Chat");
        actor
            .save_artifact_document(revision_one.clone(), None)
            .expect("create artifact");
        let stale = actor
            .save_artifact_document(revision_two.clone(), None)
            .expect_err("an update without the exact base revision must fail");
        assert!(stale.to_string().contains("expected revision"));
        actor
            .save_artifact_document(revision_two.clone(), Some(1))
            .expect("replace artifact at exact revision");
        let ui_state = ArtifactWorkspaceUiState {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            workspace_id: "workspace-1".into(),
            revision: 1,
            focused_artifact_id: Some("artifact-1".into()),
            updated_at_ms: NOW as u64 + 2,
        };
        actor
            .save_artifact_ui_state(
                WorkspaceArtifactUiStateRecord {
                    workspace_id: "workspace-1".into(),
                    revision: 1,
                    canonical_document: serde_json::to_string(&ui_state)
                        .expect("serialize artifact UI state"),
                    updated_at_ms: NOW as u64 + 2,
                },
                None,
            )
            .expect("persist artifact focus");
        let stale_focus = actor.save_artifact_ui_state(
            WorkspaceArtifactUiStateRecord {
                workspace_id: "workspace-1".into(),
                revision: 2,
                canonical_document: serde_json::to_string(&ArtifactWorkspaceUiState {
                    schema_version: ARTIFACT_SCHEMA_VERSION,
                    workspace_id: "workspace-1".into(),
                    revision: 2,
                    focused_artifact_id: None,
                    updated_at_ms: NOW as u64 + 3,
                })
                .expect("serialize stale focus"),
                updated_at_ms: NOW as u64 + 3,
            },
            None,
        );
        assert!(
            stale_focus
                .expect_err("focus replacement needs exact revision")
                .to_string()
                .contains("expected revision")
        );
        assert_eq!(
            actor
                .artifact_documents_for_session("chat-1")
                .expect("session artifacts"),
            vec![revision_two.clone()]
        );
    }

    let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("restart Workspace");
    assert_eq!(report.previous_version, 5);
    assert_eq!(
        actor
            .artifact_document("artifact-1")
            .expect("read artifact"),
        Some(revision_two)
    );
    assert!(
        actor
            .artifact_ui_state()
            .expect("read artifact UI state")
            .expect("artifact UI state exists")
            .canonical_document
            .contains("artifact-1")
    );
    drop(actor);
    let connection = Connection::open(&descriptor.path).expect("inspect immutable history");
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM artifact_events", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("artifact event count"),
        2
    );
}

#[test]
fn artifact_provider_identity_changes_only_through_explicit_file_folder_conversion() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    let (actor, _) = DatabaseActor::start(descriptor).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("seed Workspace");
    actor
        .add_project(project_record("workspace-1", "project-1", 0))
        .expect("seed Project");
    actor
        .add_chat(chat_record("workspace-1", "project-1", "chat-1"))
        .expect("seed Chat");
    let file = file_artifact_document("artifact-1", "one", 1, NOW as u64);
    actor
        .save_artifact_document(file.clone(), None)
        .expect("create File artifact");

    let unsupported = unsupported_provider_artifact_document(&file, NOW as u64 + 1);
    assert!(
        actor
            .save_artifact_document(unsupported, Some(1))
            .expect_err("arbitrary provider mutation must fail")
            .to_string()
            .contains("expected revision")
    );

    let folder = converted_folder_artifact_document(&file, NOW as u64 + 2);
    actor
        .save_artifact_document(folder.clone(), Some(1))
        .expect("explicit File-to-Folder conversion");
    assert_eq!(
        actor
            .artifact_document("artifact-1")
            .expect("read converted artifact")
            .expect("converted artifact exists")
            .provider_kind,
        "folder"
    );
    assert!(folder.canonical_document.contains("converted"));
}

#[test]
fn artifact_session_capacity_is_checked_before_current_or_event_state_changes() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("seed Workspace");
    actor
        .add_project(project_record("workspace-1", "project-1", 0))
        .expect("seed Project");
    actor
        .add_chat(chat_record("workspace-1", "project-1", "chat-1"))
        .expect("seed Chat");
    let connection = Connection::open(&descriptor.path).expect("open capacity fixture");
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             WITH RECURSIVE counter(value) AS (
                SELECT 1 UNION ALL SELECT value + 1 FROM counter WHERE value < 4096
             )
             INSERT INTO artifact_records(
                workspace_id, project_id, session_id, artifact_id, provider_kind,
                provider_version, state_schema_version, revision, canonical_document,
                document_sha256, updated_at_ms
             )
             SELECT 'workspace-1', 'project-1', 'chat-1', 'fixture-' || value,
                    'file', 1, 1, 1, '{}',
                    '0000000000000000000000000000000000000000000000000000000000000000',
                    1721260800 + value
             FROM counter;
             INSERT INTO artifact_events(
                workspace_id, project_id, session_id, artifact_id, provider_kind,
                provider_version, state_schema_version, revision, canonical_document,
                document_sha256, updated_at_ms
             )
             SELECT workspace_id, project_id, session_id, artifact_id, provider_kind,
                    provider_version, state_schema_version, revision, canonical_document,
                    document_sha256, updated_at_ms
             FROM artifact_records;",
        )
        .expect("fill exact session capacity");
    drop(connection);

    let error = actor
        .save_artifact_document(
            file_artifact_document("artifact-overflow", "one", 1, NOW as u64),
            None,
        )
        .expect_err("capacity must fail before insertion");
    assert!(error.to_string().contains("session artifact capacity"));
    let connection = Connection::open(&descriptor.path).expect("inspect capacity state");
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM artifact_records", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("record count"),
        4096
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM artifact_events", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("event count"),
        4096
    );
}

#[test]
fn workspace_v4_upgrades_to_artifact_schema_without_changing_existing_records() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    {
        let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("seed Workspace");
        actor
            .add_project(project_record("workspace-1", "project-1", 0))
            .expect("seed Project");
        actor
            .add_chat(chat_record("workspace-1", "project-1", "chat-1"))
            .expect("seed Chat");
    }
    let connection = Connection::open(&descriptor.path).expect("prepare v4 fixture");
    connection
        .execute_batch(
            "DROP TABLE artifact_workspace_state;
             DROP TABLE artifact_events;
             DROP TABLE artifact_records;
             PRAGMA user_version = 4;",
        )
        .expect("downgrade fixture to real pre-artifact schema");
    drop(connection);

    let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("upgrade v4 Workspace");
    assert_eq!(report.previous_version, 4);
    assert_eq!(report.current_version, 5);
    let snapshot = workspace_snapshot(&actor, false);
    assert_eq!(snapshot.projects[0].project_id, "project-1");
    assert_eq!(snapshot.chats[0].chat_id, "chat-1");
    drop(actor);
    let connection = Connection::open(&descriptor.path).expect("inspect upgraded schema");
    for table in [
        "artifact_records",
        "artifact_events",
        "artifact_workspace_state",
    ] {
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get::<_, i64>(0),
                )
                .expect("artifact table"),
            1
        );
    }
}

#[test]
fn corrupted_artifact_ui_state_fails_closed_on_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    {
        let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("seed Workspace");
        actor
            .save_artifact_ui_state(
                WorkspaceArtifactUiStateRecord {
                    workspace_id: "workspace-1".into(),
                    revision: 1,
                    canonical_document: serde_json::to_string(&ArtifactWorkspaceUiState {
                        schema_version: ARTIFACT_SCHEMA_VERSION,
                        workspace_id: "workspace-1".into(),
                        revision: 1,
                        focused_artifact_id: None,
                        updated_at_ms: NOW as u64,
                    })
                    .expect("serialize UI state"),
                    updated_at_ms: NOW as u64,
                },
                None,
            )
            .expect("persist UI state");
    }
    let connection = Connection::open(&descriptor.path).expect("corrupt UI state");
    connection
        .execute(
            "UPDATE artifact_workspace_state SET canonical_document = '{}'",
            [],
        )
        .expect("corrupt UI document without digest");
    drop(connection);
    let error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("corrupt Artifact UI state must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("artifact UI state digest mismatch")
    );
    assert!(database::migration_diagnostic_path(&descriptor).exists());
}

#[test]
fn corrupted_artifact_document_fails_closed_on_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temp.path().join("active"),
        "workspace-1",
        temp.path().join("recovery"),
    );
    {
        let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("seed Workspace");
        actor
            .add_project(project_record("workspace-1", "project-1", 0))
            .expect("seed Project");
        actor
            .add_chat(chat_record("workspace-1", "project-1", "chat-1"))
            .expect("seed Chat");
        actor
            .save_artifact_document(
                file_artifact_document("artifact-1", "one", 1, NOW as u64),
                None,
            )
            .expect("create artifact");
    }
    let connection = Connection::open(&descriptor.path).expect("open database for corruption");
    connection
        .execute(
            "UPDATE artifact_records SET canonical_document = '{\"corrupt\":true}'",
            [],
        )
        .expect("corrupt artifact without its digest");
    drop(connection);

    let error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("corrupt artifact must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("artifact document digest mismatch")
    );
    assert!(database::migration_diagnostic_path(&descriptor).exists());
}

#[test]
fn app_database_round_trips_across_actor_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    let first_generation;
    {
        let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("open app database");
        assert_eq!(report.previous_version, 0);
        assert_eq!(report.current_version, 8);
        assert!(
            report
                .backup_path
                .as_ref()
                .is_some_and(|path| path.exists())
        );
        first_generation = actor
            .upsert_installation(InstallationRecord {
                installation_id: "installation-1".into(),
                created_at: NOW,
                updated_at: NOW,
            })
            .expect("write installation");
        actor
            .record_recent_workspace(RecentWorkspaceRecord {
                workspace_id: "workspace-1".into(),
                display_name: "C4OS".into(),
                archive_path: "/archives/c4os.zip".into(),
                last_opened_at: NOW,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .expect("write recent");
    }

    let (actor, report) = DatabaseActor::start(descriptor).expect("reopen app database");
    assert_eq!(report.previous_version, 8);
    assert!(report.backup_path.is_none());
    let snapshot = match actor
        .snapshot(SnapshotQuery::new(3).expect("valid bounds"))
        .expect("snapshot")
    {
        DatabaseSnapshot::App(snapshot) => snapshot,
        DatabaseSnapshot::Workspace(_) => panic!("expected app snapshot"),
    };
    assert_eq!(
        snapshot.installation.expect("installation").installation_id,
        "installation-1"
    );
    assert_eq!(snapshot.recents.len(), 1);
    assert!(snapshot.generation > first_generation);
}

#[test]
fn app_configuration_lkg_round_trips_across_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    let expected = AppConfigurationSnapshotRecord {
        canonical_document: "theme = \"system\"\nrestore_last_workspace = true\n".into(),
        generation: 17,
        activated_at: NOW,
    };
    {
        let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("app database");
        actor
            .activate_app_configuration(expected.clone())
            .expect("activate app configuration LKG");
        let stale_error = actor
            .activate_app_configuration(expected.clone())
            .expect_err("app configuration generation must be monotonic");
        assert!(stale_error.to_string().contains("not newer"));
        assert_eq!(
            actor
                .app_configuration_lkg()
                .expect("read app configuration"),
            Some(expected.clone())
        );
    }

    let (actor, report) = DatabaseActor::start(descriptor).expect("restart app database");
    assert_eq!(report.previous_version, 8);
    assert_eq!(
        actor
            .app_configuration_lkg()
            .expect("read restarted app configuration"),
        Some(expected.clone())
    );
    let snapshot = match actor
        .snapshot(SnapshotQuery::new(3).expect("query"))
        .expect("app snapshot")
    {
        DatabaseSnapshot::App(snapshot) => snapshot,
        DatabaseSnapshot::Workspace(_) => panic!("expected app snapshot"),
    };
    assert_eq!(snapshot.configuration_lkg, Some(expected));
}

#[test]
fn corrupted_app_configuration_lkg_fails_closed_on_restart() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    {
        let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("app database");
        actor
            .activate_app_configuration(AppConfigurationSnapshotRecord {
                canonical_document: "theme = \"system\"\n".into(),
                generation: 4,
                activated_at: NOW,
            })
            .expect("activate app configuration LKG");
    }
    let connection = Connection::open(&descriptor.path).expect("open database for corruption");
    connection
        .execute(
            "UPDATE app_configuration_lkg SET canonical_document = 'theme = \"corrupt\"'",
            [],
        )
        .expect("corrupt canonical document without its digest");
    drop(connection);

    let error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("corrupt app configuration must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("digest mismatch"));
    assert!(database::migration_diagnostic_path(&descriptor).exists());
    assert!(
        fs::read_dir(&descriptor.recovery_dir)
            .expect("recovery directory")
            .all(|entry| !entry
                .expect("recovery entry")
                .file_name()
                .to_string_lossy()
                .contains(".tmp-"))
    );
}

#[test]
fn missing_app_diagnostics_schema_fails_closed() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    {
        let (_actor, _) = DatabaseActor::start(descriptor.clone()).expect("app database");
    }
    let connection = Connection::open(&descriptor.path).expect("open app database");
    connection
        .execute("DROP TABLE app_diagnostics", [])
        .expect("remove required diagnostic table");
    drop(connection);
    let error = match DatabaseActor::start(descriptor) {
        Ok(_) => panic!("missing app diagnostics schema must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("app_diagnostics"));
}

#[test]
fn migration_creates_validated_online_backup_and_failure_preserves_source() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    fs::create_dir_all(descriptor.path.parent().expect("state parent")).expect("create state");
    let connection = Connection::open(&descriptor.path).expect("seed source");
    connection
        .execute_batch(
            "CREATE TABLE retained_before_failed_migration(value TEXT NOT NULL);
             INSERT INTO retained_before_failed_migration(value) VALUES ('retained');
             PRAGMA user_version = 99;",
        )
        .expect("seed future schema");
    drop(connection);

    let error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("future schema must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("migration"));
    let backup_path =
        database::migration_backup_path(&descriptor, 99, 8).expect("deterministic backup path");
    assert!(backup_path.exists());
    let backup = Connection::open(backup_path).expect("open backup");
    assert_eq!(
        backup
            .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .expect("backup quick check"),
        "ok"
    );
    let source = Connection::open(&descriptor.path).expect("reopen source");
    assert_eq!(
        source
            .query_row(
                "SELECT value FROM retained_before_failed_migration",
                [],
                |row| row.get::<_, String>(0)
            )
            .expect("retained row"),
        "retained"
    );
    assert_eq!(
        source
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .expect("version"),
        99
    );
    assert!(database::migration_diagnostic_path(&descriptor).exists());
}

#[test]
fn workspace_migration_recovery_and_writer_lock_stay_outside_portable_root() {
    let temp = TempDir::new().expect("temp directory");
    let working_copy = temp.path().join("workspace/active");
    let descriptor = DatabaseDescriptor::workspace(&working_copy, "workspace-1");
    assert_eq!(
        descriptor.recovery_dir,
        temp.path().join("workspace/recovery/workspace-1")
    );
    fs::create_dir_all(descriptor.path.parent().expect("state parent")).expect("create state");
    let connection = Connection::open(&descriptor.path).expect("seed Workspace source");
    connection
        .execute_batch(
            "CREATE TABLE retained_workspace_state(value TEXT NOT NULL);
             INSERT INTO retained_workspace_state(value) VALUES ('retained');
             PRAGMA user_version = 99;",
        )
        .expect("seed future Workspace schema");
    drop(connection);

    let error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("future Workspace schema must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("migration"));
    let backup_path =
        database::migration_backup_path(&descriptor, 99, 5).expect("Workspace backup path");
    let diagnostic_path = database::migration_diagnostic_path(&descriptor);
    let writer_lock_path = descriptor
        .recovery_dir
        .join("workspace.sqlite3.writer.lock");
    for artifact in [&backup_path, &diagnostic_path, &writer_lock_path] {
        assert!(artifact.exists(), "missing recovery artifact: {artifact:?}");
        assert!(
            !artifact.starts_with(&working_copy),
            "recovery artifact became a portable archive candidate: {artifact:?}"
        );
    }
    assert!(!working_copy.join("recovery").exists());
    assert!(
        !working_copy
            .join("state/workspace.sqlite3.writer.lock")
            .exists()
    );

    let source = Connection::open(&descriptor.path).expect("reopen Workspace source");
    assert_eq!(
        source
            .query_row("SELECT value FROM retained_workspace_state", [], |row| {
                row.get::<_, String>(0)
            })
            .expect("retained Workspace row"),
        "retained"
    );
}

#[test]
fn workspace_descriptor_rejects_recovery_nested_in_portable_root() {
    let temp = TempDir::new().expect("temp directory");
    let working_copy = temp.path().join("working");
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        &working_copy,
        "workspace-1",
        working_copy.join("recovery/workspace-1"),
    );
    let error = match DatabaseActor::start(descriptor) {
        Ok(_) => panic!("nested Workspace recovery must fail closed"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("outside the portable working copy")
    );
    assert!(!working_copy.exists());
}

#[test]
fn same_project_is_isolated_by_workspace_database() {
    let temp = TempDir::new().expect("temp directory");
    let left_descriptor = DatabaseDescriptor::workspace(temp.path().join("left"), "workspace-left");
    let right_descriptor =
        DatabaseDescriptor::workspace(temp.path().join("right"), "workspace-right");
    let (left, _) = DatabaseActor::start(left_descriptor).expect("left database");
    let (right, _) = DatabaseActor::start(right_descriptor).expect("right database");
    left.create_workspace(workspace_record("workspace-left", "Left"))
        .expect("left workspace");
    right
        .create_workspace(workspace_record("workspace-right", "Right"))
        .expect("right workspace");
    left.add_project_and_chat(
        project_record("workspace-left", "shared-project", 0),
        chat_record("workspace-left", "shared-project", "left-chat"),
    )
    .expect("left state");
    right
        .add_project_and_chat(
            project_record("workspace-right", "shared-project", 0),
            chat_record("workspace-right", "shared-project", "right-chat"),
        )
        .expect("right state");

    let left_snapshot = workspace_snapshot(&left, false);
    let right_snapshot = workspace_snapshot(&right, false);
    assert_eq!(left_snapshot.projects[0].project_id, "shared-project");
    assert_eq!(right_snapshot.projects[0].project_id, "shared-project");
    assert_eq!(left_snapshot.chats[0].chat_id, "left-chat");
    assert_eq!(right_snapshot.chats[0].chat_id, "right-chat");
}

#[test]
fn pre_promotion_inspector_is_read_only_bounded_and_fails_closed() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let candidate_path = temp.path().join("inspection/candidate.sqlite3");
    let foreign_key_candidate = temp.path().join("inspection/foreign-key.sqlite3");
    let (actor, _) = DatabaseActor::start(descriptor).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-1", 0),
            chat_record("workspace-1", "project-1", "chat-1"),
        )
        .expect("first project and chat");
    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-2", 1),
            chat_record("workspace-1", "project-2", "chat-2"),
        )
        .expect("second project and chat");
    actor
        .online_backup_to(&candidate_path)
        .expect("candidate snapshot");
    actor
        .online_backup_to(&foreign_key_candidate)
        .expect("foreign-key candidate snapshot");

    let bytes_before = fs::read(&candidate_path).expect("candidate bytes");
    let inspection = database::inspect_workspace_database_read_only(
        &candidate_path,
        "workspace-1",
        SnapshotQuery::new(1)
            .expect("bounded query")
            .including_inactive(),
    )
    .expect("semantic inspection");
    assert_eq!(inspection.schema_version, 5);
    assert_eq!(inspection.snapshot.projects.len(), 1);
    assert_eq!(inspection.snapshot.chats.len(), 1);
    assert!(inspection.snapshot.truncated);
    assert_eq!(
        fs::read(&candidate_path).expect("candidate bytes after inspection"),
        bytes_before
    );
    assert!(!temp.path().join("inspection/recovery").exists());

    let identity_error = database::inspect_workspace_database_read_only(
        &candidate_path,
        "different-workspace",
        SnapshotQuery::new(2).expect("query"),
    )
    .expect_err("mismatched Workspace identity must fail closed");
    assert!(
        identity_error
            .to_string()
            .contains("does not match expected")
    );
    assert_eq!(
        fs::read(&candidate_path).expect("candidate after identity rejection"),
        bytes_before
    );

    let connection = Connection::open(&candidate_path).expect("open candidate");
    connection
        .pragma_update(None, "user_version", 99)
        .expect("set unsupported version");
    drop(connection);
    let unsupported_bytes = fs::read(&candidate_path).expect("unsupported candidate bytes");
    let version_error = database::inspect_workspace_database_read_only(
        &candidate_path,
        "workspace-1",
        SnapshotQuery::new(2).expect("query"),
    )
    .expect_err("unsupported schema must fail closed");
    assert!(
        version_error
            .to_string()
            .contains("unsupported Workspace database user_version")
    );
    assert_eq!(
        fs::read(&candidate_path).expect("candidate after version rejection"),
        unsupported_bytes
    );

    let connection = Connection::open(&foreign_key_candidate).expect("open FK candidate");
    connection
        .pragma_update(None, "foreign_keys", "OFF")
        .expect("disable FK for corruption fixture");
    connection
        .execute(
            "INSERT INTO chats(
                workspace_id, project_id, chat_id, title, created_at, updated_at,
                lifecycle_state, inactivated_at
             ) VALUES ('workspace-1', 'absent-project', 'orphan-chat', 'Orphan', ?1, ?1,
                'active', NULL)",
            [NOW],
        )
        .expect("insert orphan Chat fixture");
    drop(connection);
    let foreign_key_error = database::inspect_workspace_database_read_only(
        &foreign_key_candidate,
        "workspace-1",
        SnapshotQuery::new(3).expect("query"),
    )
    .expect_err("foreign-key corruption must fail closed");
    assert!(
        foreign_key_error
            .to_string()
            .contains("foreign-key validation failed")
    );
}

#[test]
fn pre_promotion_rejects_untrusted_schema_objects_and_constraint_drift() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let trigger_candidate = temp.path().join("candidate/trigger.sqlite3");
    let shape_candidate = temp.path().join("candidate/shape.sqlite3");
    let (actor, _) = DatabaseActor::start(descriptor).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    actor
        .online_backup_to(&trigger_candidate)
        .expect("trigger candidate");
    actor
        .online_backup_to(&shape_candidate)
        .expect("shape candidate");

    let connection = Connection::open(&trigger_candidate).expect("open trigger candidate");
    connection
        .execute_batch(
            "CREATE TRIGGER untrusted_project_trigger
             AFTER INSERT ON projects
             BEGIN
               UPDATE workspaces SET display_name = display_name;
             END;",
        )
        .expect("inject untrusted trigger");
    drop(connection);
    let trigger_error = database::inspect_workspace_database_read_only(
        &trigger_candidate,
        "workspace-1",
        SnapshotQuery::new(10).expect("query"),
    )
    .expect_err("unexpected trigger must fail closed");
    assert!(
        trigger_error
            .to_string()
            .contains("trusted compiled schema")
    );

    let connection = Connection::open(&shape_candidate).expect("open shape candidate");
    let trusted_sql: String = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = 'workspaces'",
            [],
            |row| row.get(0),
        )
        .expect("trusted table SQL");
    let drifted_sql = trusted_sql.replace("display_name TEXT NOT NULL", "display_name TEXT");
    assert_ne!(drifted_sql, trusted_sql);
    connection
        .pragma_update(None, "writable_schema", "ON")
        .expect("enable hostile schema fixture");
    connection
        .execute(
            "UPDATE sqlite_schema SET sql = ?1
             WHERE type = 'table' AND name = 'workspaces'",
            [drifted_sql],
        )
        .expect("inject constraint drift");
    let schema_version: i64 = connection
        .query_row("PRAGMA schema_version", [], |row| row.get(0))
        .expect("schema version");
    connection
        .pragma_update(None, "schema_version", schema_version + 1)
        .expect("invalidate schema cache");
    drop(connection);
    let shape_error = database::inspect_workspace_database_read_only(
        &shape_candidate,
        "workspace-1",
        SnapshotQuery::new(10).expect("query"),
    )
    .expect_err("constraint drift must fail closed");
    assert!(shape_error.to_string().contains("trusted compiled schema"));
}

#[test]
fn workspace_display_names_are_canonical_at_ingress_and_projection() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor.clone()).expect("Workspace database");
    for invalid_name in [
        " Workspace".to_owned(),
        "Workspace\nInjected".to_owned(),
        "x".repeat(database::MAX_WORKSPACE_DISPLAY_NAME_BYTES + 1),
    ] {
        let mut record = workspace_record("workspace-1", &invalid_name);
        record.display_name = invalid_name;
        assert!(actor.create_workspace(record).is_err());
    }
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("canonical Workspace name");
    drop(actor);

    let connection = Connection::open(&descriptor.path).expect("open imported database");
    connection
        .execute(
            "UPDATE workspaces SET display_name = ?1 WHERE workspace_id = 'workspace-1'",
            ["x".repeat(database::MAX_WORKSPACE_DISPLAY_NAME_BYTES + 1)],
        )
        .expect("inject oversized imported name");
    drop(connection);
    let error = database::inspect_workspace_database_read_only(
        &descriptor.path,
        "workspace-1",
        SnapshotQuery::new(10).expect("query"),
    )
    .expect_err("unsafe imported Workspace name must not project");
    assert!(error.to_string().contains("Workspace display name"));

    let app_descriptor = DatabaseDescriptor::app(temp.path().join("app"));
    let (app, _) = DatabaseActor::start(app_descriptor).expect("app database");
    let error = app
        .record_recent_workspace(RecentWorkspaceRecord {
            workspace_id: "workspace-1".into(),
            display_name: "Workspace\u{0}".into(),
            archive_path: "/archives/workspace.zip".into(),
            last_opened_at: NOW,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect_err("unsafe recent projection must fail closed");
    assert!(error.to_string().contains("Workspace display name"));
}

#[test]
fn complete_counted_snapshot_supports_more_than_one_bounded_page() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let backup_path = temp.path().join("complete/workspace.sqlite3");
    let (actor, _) = DatabaseActor::start(descriptor).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    for position in 0..=database::MAX_READ_RECORDS {
        actor
            .add_project(project_record(
                "workspace-1",
                &format!("project-{position:03}"),
                position as i64,
            ))
            .expect("project");
    }

    let bounded = workspace_snapshot(&actor, false);
    assert_eq!(bounded.projects.len(), 100);
    assert!(bounded.truncated);
    let (complete, counts) = actor
        .complete_workspace_snapshot(false)
        .expect("complete counted snapshot");
    assert_eq!(counts.projects, database::MAX_READ_RECORDS + 1);
    assert_eq!(complete.projects.len(), database::MAX_READ_RECORDS + 1);
    assert!(!complete.truncated);
    let barrier = actor
        .backup_complete_workspace_snapshot_to(&backup_path, false)
        .expect("complete counted backup barrier");
    assert_eq!(barrier.counts, counts);
    assert_eq!(barrier.snapshot, complete);
    let inspection =
        database::inspect_complete_workspace_database_read_only(&backup_path, "workspace-1", false)
            .expect("complete read-only inspection");
    assert_eq!(inspection.counts, counts);
    assert_eq!(inspection.snapshot, complete);
}

#[test]
fn snapshot_text_field_and_aggregate_byte_limits_fail_closed() {
    let per_field = TempDir::new().expect("per-field temp directory");
    let per_field_descriptor =
        DatabaseDescriptor::workspace(per_field.path().join("working"), "workspace-1");
    {
        let (actor, _) =
            DatabaseActor::start(per_field_descriptor.clone()).expect("Workspace database");
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("workspace");
        actor
            .add_project(project_record("workspace-1", "project-1", 0))
            .expect("project");
    }
    let connection = Connection::open(&per_field_descriptor.path).expect("open Workspace DB");
    connection
        .execute(
            "UPDATE projects SET display_name = ?1
             WHERE workspace_id = 'workspace-1' AND project_id = 'project-1'",
            ["x".repeat(database::MAX_TEXT_FIELD_BYTES + 1)],
        )
        .expect("inject oversized field");
    drop(connection);
    let field_error = database::inspect_workspace_database_read_only(
        &per_field_descriptor.path,
        "workspace-1",
        SnapshotQuery::new(10).expect("query"),
    )
    .expect_err("oversized snapshot field must fail closed");
    assert!(field_error.to_string().contains("text field larger"));

    let aggregate = TempDir::new().expect("aggregate temp directory");
    let aggregate_descriptor =
        DatabaseDescriptor::workspace(aggregate.path().join("working"), "workspace-1");
    {
        let (actor, _) =
            DatabaseActor::start(aggregate_descriptor.clone()).expect("Workspace database");
        actor
            .create_workspace(workspace_record("workspace-1", "Workspace"))
            .expect("workspace");
    }
    let mut connection = Connection::open(&aggregate_descriptor.path).expect("open Workspace DB");
    let transaction = connection.transaction().expect("diagnostic transaction");
    let large_message = "x".repeat(900_000);
    for index in 0..5 {
        transaction
            .execute(
                "INSERT INTO workspace_diagnostics(
                    workspace_id, diagnostic_id, category, message, created_at
                 ) VALUES ('workspace-1', ?1, 'fixture', ?2, ?3)",
                rusqlite::params![format!("diagnostic-{index}"), large_message, NOW + index],
            )
            .expect("inject aggregate diagnostic fixture");
    }
    transaction.commit().expect("commit aggregate fixture");
    drop(connection);
    let aggregate_error = database::inspect_workspace_database_read_only(
        &aggregate_descriptor.path,
        "workspace-1",
        SnapshotQuery::new(10).expect("query"),
    )
    .expect_err("aggregate snapshot bytes must fail closed");
    assert!(
        aggregate_error
            .to_string()
            .contains("snapshot text exceeds")
    );
}

#[test]
fn serialized_backup_returns_the_same_canonical_writer_barrier_snapshot() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let backup_path = temp.path().join("saved/workspace.sqlite3");
    let (actor, _) = DatabaseActor::start(descriptor).expect("Workspace database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-1", 0),
            chat_record("workspace-1", "project-1", "chat-1"),
        )
        .expect("initial project and chat");

    let query = SnapshotQuery::new(10).expect("query").including_inactive();
    let barrier = actor
        .backup_workspace_snapshot_to(&backup_path, query)
        .expect("serialized backup barrier");
    let backup_inspection =
        database::inspect_workspace_database_read_only(&backup_path, "workspace-1", query)
            .expect("inspect serialized backup");
    assert_eq!(backup_inspection.snapshot, barrier.snapshot);
    assert_eq!(barrier.generation, barrier.snapshot.generation);

    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-2", 1),
            chat_record("workspace-1", "project-2", "chat-2"),
        )
        .expect("post-barrier state change");
    let source = workspace_snapshot(&actor, true);
    let retained_backup =
        database::inspect_workspace_database_read_only(&backup_path, "workspace-1", query)
            .expect("reinspect backup");
    assert_eq!(retained_backup.snapshot, barrier.snapshot);
    assert!(source.generation > barrier.snapshot.generation);
    assert_eq!(source.projects.len(), 2);
    assert_eq!(barrier.snapshot.projects.len(), 1);
}

#[test]
fn failed_multi_record_transition_rolls_back_every_record_and_generation() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    let initial_generation = actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");

    let error = actor
        .add_project_and_chat(
            project_record("workspace-1", "project-1", 0),
            chat_record("workspace-1", "missing-project", "chat-1"),
        )
        .expect_err("foreign key must fail transaction");
    assert!(error.to_string().contains("FOREIGN KEY"));
    let snapshot = workspace_snapshot(&actor, true);
    assert!(snapshot.projects.is_empty());
    assert!(snapshot.chats.is_empty());
    assert_eq!(snapshot.generation, initial_generation);
}

#[test]
fn inactivation_retains_records_and_only_changes_active_snapshot() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-1", 0),
            chat_record("workspace-1", "project-1", "chat-1"),
        )
        .expect("project and chat");
    actor
        .inactivate(
            InactiveEntity::Chat {
                chat_id: "chat-1".into(),
            },
            NOW + 1,
        )
        .expect("inactivate chat");
    actor
        .inactivate(
            InactiveEntity::Project {
                project_id: "project-1".into(),
            },
            NOW + 2,
        )
        .expect("inactivate project");

    let active = workspace_snapshot(&actor, false);
    assert!(active.projects.is_empty());
    assert!(active.chats.is_empty());
    let retained = workspace_snapshot(&actor, true);
    assert_eq!(retained.projects.len(), 1);
    assert_eq!(retained.chats.len(), 1);
    assert_eq!(
        retained.projects[0].lifecycle_state,
        LifecycleState::Inactive
    );
    assert_eq!(retained.projects[0].inactivated_at, Some(NOW + 2));
    assert_eq!(retained.chats[0].lifecycle_state, LifecycleState::Inactive);
    assert_eq!(retained.chats[0].inactivated_at, Some(NOW + 1));

    actor
        .inactivate(InactiveEntity::Workspace, NOW + 3)
        .expect("inactivate workspace");
    let inactive_workspace_surface = workspace_snapshot(&actor, false);
    assert!(inactive_workspace_surface.workspace.is_none());
    assert!(inactive_workspace_surface.projects.is_empty());
    assert!(inactive_workspace_surface.chats.is_empty());
    let retained_workspace = workspace_snapshot(&actor, true);
    let workspace = retained_workspace
        .workspace
        .expect("retained Workspace row");
    assert_eq!(workspace.lifecycle_state, LifecycleState::Inactive);
    assert_eq!(workspace.inactivated_at, Some(NOW + 3));
}

#[test]
fn workspace_inactivation_compensation_restores_the_exact_prior_record() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    let prior_record = WorkspaceRecord {
        workspace_id: "workspace-1".into(),
        display_name: "Before Inactivation".into(),
        created_at: NOW - 100,
        updated_at: NOW - 5,
        lifecycle_state: LifecycleState::Active,
        inactivated_at: None,
    };
    actor
        .create_workspace(prior_record.clone())
        .expect("workspace");
    actor
        .add_project(project_record("workspace-1", "retained-project", 0))
        .expect("retained Project");

    let inactivated_generation = actor
        .inactivate(InactiveEntity::Workspace, NOW + 1)
        .expect("inactivate Workspace");
    let compensated_generation = actor
        .compensate_workspace_inactivation(inactivated_generation, prior_record.clone())
        .expect("restore exact prior Workspace record");
    assert_eq!(compensated_generation, inactivated_generation + 1);

    let restored = workspace_snapshot(&actor, false);
    assert_eq!(restored.workspace, Some(prior_record));
    assert_eq!(restored.generation, compensated_generation);
    assert_eq!(restored.projects.len(), 1);
    assert_eq!(restored.projects[0].project_id, "retained-project");
}

#[test]
fn stale_workspace_inactivation_compensation_fails_without_partial_restore() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    let prior_record = workspace_record("workspace-1", "Before Inactivation");
    actor
        .create_workspace(prior_record.clone())
        .expect("workspace");
    let inactivated_generation = actor
        .inactivate(InactiveEntity::Workspace, NOW + 1)
        .expect("inactivate Workspace");
    let intervening_generation = actor
        .record_diagnostic(database::DiagnosticRecord {
            diagnostic_id: "intervening-write".into(),
            category: "database".into(),
            message: "intervening durable mutation".into(),
            created_at: NOW + 2,
        })
        .expect("intervening write");

    let error = actor
        .compensate_workspace_inactivation(inactivated_generation, prior_record)
        .expect_err("stale compensation must fail closed");
    assert!(error.to_string().contains("generation changed"));

    let retained = workspace_snapshot(&actor, true);
    assert_eq!(retained.generation, intervening_generation);
    assert_eq!(retained.diagnostics.len(), 1);
    let workspace = retained.workspace.expect("inactive Workspace remains");
    assert_eq!(workspace.display_name, "Before Inactivation");
    assert_eq!(workspace.lifecycle_state, LifecycleState::Inactive);
    assert_eq!(workspace.updated_at, NOW + 1);
    assert_eq!(workspace.inactivated_at, Some(NOW + 1));
}

#[test]
fn project_inactivation_atomically_compacts_reordered_active_positions() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let backup_path = temp.path().join("recovery-check/workspace.sqlite3");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    for (position, project_id) in ["one", "two", "three"].iter().enumerate() {
        actor
            .add_project(project_record("workspace-1", project_id, position as i64))
            .expect("project");
    }
    actor
        .reorder_projects(vec!["three".into(), "one".into(), "two".into()])
        .expect("reorder Projects");
    let generation_before = workspace_snapshot(&actor, false).generation;
    let generation_after = actor
        .inactivate(
            InactiveEntity::Project {
                project_id: "three".into(),
            },
            NOW + 1,
        )
        .expect("inactivate first ordered Project");
    assert_eq!(generation_after, generation_before + 1);

    let active = workspace_snapshot(&actor, false);
    assert_eq!(
        active
            .projects
            .iter()
            .map(|project| (project.project_id.as_str(), project.position))
            .collect::<Vec<_>>(),
        vec![("one", 0), ("two", 1)]
    );
    let retained = workspace_snapshot(&actor, true);
    assert_eq!(retained.projects.len(), 3);
    assert_eq!(
        retained
            .projects
            .iter()
            .find(|project| project.project_id == "three")
            .expect("inactive Project retained")
            .lifecycle_state,
        LifecycleState::Inactive
    );

    let query = SnapshotQuery::new(10).expect("query");
    let barrier = actor
        .backup_workspace_snapshot_to(&backup_path, query)
        .expect("canonical backup barrier");
    assert_eq!(
        barrier
            .snapshot
            .projects
            .iter()
            .map(|project| project.position)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let inspection =
        database::inspect_workspace_database_read_only(&backup_path, "workspace-1", query)
            .expect("inspect canonical active ordering");
    assert_eq!(inspection.snapshot, barrier.snapshot);
}

#[test]
fn writer_pragmas_are_durable_and_reads_are_bounded() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    for index in 0..5 {
        actor
            .record_recent_workspace(RecentWorkspaceRecord {
                workspace_id: format!("workspace-{index}"),
                display_name: format!("Workspace {index}"),
                archive_path: format!("/archives/{index}.zip"),
                last_opened_at: NOW + index,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .expect("recent");
    }
    let pragmas = actor.pragma_snapshot().expect("pragma snapshot");
    assert!(pragmas.foreign_keys);
    assert_eq!(pragmas.journal_mode.to_ascii_lowercase(), "wal");
    assert_eq!(pragmas.synchronous, 2);
    assert_eq!(pragmas.busy_timeout_millis, 1_500);

    let snapshot = match actor
        .snapshot(SnapshotQuery::new(2).expect("bounded query"))
        .expect("snapshot")
    {
        DatabaseSnapshot::App(snapshot) => snapshot,
        DatabaseSnapshot::Workspace(_) => panic!("expected app snapshot"),
    };
    assert_eq!(snapshot.recents.len(), 2);
    assert!(snapshot.truncated);
    assert_eq!(snapshot.recents[0].workspace_id, "workspace-4");
    assert!(SnapshotQuery::new(database::MAX_READ_RECORDS + 1).is_err());
}

#[test]
fn database_path_rejects_a_second_writer_actor() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    let (owner, _) = DatabaseActor::start(descriptor.clone()).expect("first writer owns database");
    let second_error = match DatabaseActor::start(descriptor.clone()) {
        Ok(_) => panic!("second writer actor must fail closed"),
        Err(error) => error,
    };
    assert!(second_error.to_string().contains("single-writer ownership"));
    drop(owner);
    DatabaseActor::start(descriptor).expect("ownership lock released on actor drop");
}

#[test]
fn projects_keep_order_and_missing_or_relocated_path_state() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    actor
        .add_project(project_record("workspace-1", "one", 0))
        .expect("project one");
    actor
        .add_project(project_record("workspace-1", "two", 1))
        .expect("project two");
    actor
        .reorder_projects(vec!["two".into(), "one".into()])
        .expect("reorder");
    actor
        .update_project_path(
            "two",
            "/projects/two-new",
            "/projects/two",
            ProjectPathState::Relocated,
        )
        .expect("relocate");
    actor
        .update_project_path(
            "one",
            "/projects/one",
            "/projects/one",
            ProjectPathState::Missing,
        )
        .expect("mark missing");

    let snapshot = workspace_snapshot(&actor, false);
    assert_eq!(snapshot.projects[0].project_id, "two");
    assert_eq!(snapshot.projects[0].path_state, ProjectPathState::Relocated);
    assert_eq!(snapshot.projects[0].last_known_path, "/projects/two");
    assert_eq!(snapshot.projects[1].project_id, "one");
    assert_eq!(snapshot.projects[1].path_state, ProjectPathState::Missing);
}

#[test]
fn configuration_and_diagnostics_round_trip_without_exposing_sql() {
    let temp = TempDir::new().expect("temp directory");
    let descriptor = DatabaseDescriptor::workspace(temp.path().join("working"), "workspace-1");
    let (actor, _) = DatabaseActor::start(descriptor).expect("database");
    actor
        .create_workspace(workspace_record("workspace-1", "Workspace"))
        .expect("workspace");
    let wrong_scope = actor
        .activate_configuration(database::ConfigurationSnapshotRecord {
            workspace_id: "workspace-1".into(),
            scope_kind: "workspace".into(),
            scope_id: "different-workspace".into(),
            canonical_document: "theme = \"system\"\n".into(),
            generation: 1,
            activated_at: NOW,
        })
        .expect_err("Workspace scope identity must match");
    assert!(wrong_scope.to_string().contains("must equal workspace_id"));
    let missing_reference = actor
        .activate_configuration(database::ConfigurationSnapshotRecord {
            workspace_id: "workspace-1".into(),
            scope_kind: "project".into(),
            scope_id: "absent-project".into(),
            canonical_document: "theme = \"system\"\n".into(),
            generation: 1,
            activated_at: NOW,
        })
        .expect_err("Project scope must reference a record");
    assert!(
        missing_reference
            .to_string()
            .contains("does not reference an existing record")
    );
    actor
        .activate_configuration(database::ConfigurationSnapshotRecord {
            workspace_id: "workspace-1".into(),
            scope_kind: "workspace".into(),
            scope_id: "workspace-1".into(),
            canonical_document: "theme = \"system\"\n".into(),
            generation: 7,
            activated_at: NOW,
        })
        .expect("configuration");
    let stale_configuration = actor
        .activate_configuration(database::ConfigurationSnapshotRecord {
            workspace_id: "workspace-1".into(),
            scope_kind: "workspace".into(),
            scope_id: "workspace-1".into(),
            canonical_document: "theme = \"dark\"\n".into(),
            generation: 7,
            activated_at: NOW + 1,
        })
        .expect_err("configuration generations must be monotonic");
    assert!(stale_configuration.to_string().contains("not newer"));
    actor
        .add_project_and_chat(
            project_record("workspace-1", "project-1", 0),
            chat_record("workspace-1", "project-1", "chat-1"),
        )
        .expect("configuration reference records");
    for (scope_kind, scope_id) in [("project", "project-1"), ("chat", "chat-1")] {
        actor
            .activate_configuration(database::ConfigurationSnapshotRecord {
                workspace_id: "workspace-1".into(),
                scope_kind: scope_kind.into(),
                scope_id: scope_id.into(),
                canonical_document: "theme = \"system\"\n".into(),
                generation: 1,
                activated_at: NOW,
            })
            .expect("referenced configuration");
    }
    let credential_diagnostic = actor
        .record_diagnostic(database::DiagnosticRecord {
            diagnostic_id: "unsafe-credential".into(),
            category: "configuration".into(),
            message: "token=do-not-store".into(),
            created_at: NOW,
        })
        .expect_err("credential-like diagnostics must fail closed");
    assert!(
        credential_diagnostic
            .to_string()
            .contains("credential-like")
    );
    let oversized_diagnostic = actor
        .record_diagnostic(database::DiagnosticRecord {
            diagnostic_id: "oversized".into(),
            category: "configuration".into(),
            message: "x".repeat(database::MAX_DIAGNOSTIC_MESSAGE_BYTES + 1),
            created_at: NOW,
        })
        .expect_err("oversized diagnostics must fail closed");
    assert!(oversized_diagnostic.to_string().contains("exceeds"));
    actor
        .record_diagnostic(database::DiagnosticRecord {
            diagnostic_id: "diagnostic-1".into(),
            category: "configuration".into(),
            message: "last-known-good activated".into(),
            created_at: NOW,
        })
        .expect("diagnostic");

    let snapshot = workspace_snapshot(&actor, false);
    assert_eq!(snapshot.configurations.len(), 3);
    assert_eq!(
        snapshot
            .configurations
            .iter()
            .find(|record| record.scope_kind == "workspace")
            .expect("Workspace configuration")
            .generation,
        7
    );
    assert_eq!(snapshot.diagnostics.len(), 1);
}

#[test]
fn online_backup_captures_committed_wal_state_and_leaves_source_authoritative() {
    let temp = TempDir::new().expect("temp directory");
    let source_descriptor = DatabaseDescriptor::app(temp.path().join("source-home"));
    let backup_descriptor = DatabaseDescriptor::app(temp.path().join("backup-home"));
    let (source, _) = DatabaseActor::start(source_descriptor.clone()).expect("source database");
    source
        .record_recent_workspace(RecentWorkspaceRecord {
            workspace_id: "captured".into(),
            display_name: "Captured".into(),
            archive_path: "/archives/captured.zip".into(),
            last_opened_at: NOW,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("committed WAL record");

    source
        .online_backup_to(&backup_descriptor.path)
        .expect("online backup");
    source
        .record_recent_workspace(RecentWorkspaceRecord {
            workspace_id: "source-only".into(),
            display_name: "Source only".into(),
            archive_path: "/archives/source-only.zip".into(),
            last_opened_at: NOW + 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .expect("post-backup source write");

    let (backup, report) =
        DatabaseActor::start(backup_descriptor).expect("open captured backup as database");
    assert_eq!(report.previous_version, report.current_version);
    assert!(report.backup_path.is_none());
    let backup_snapshot = match backup
        .snapshot(SnapshotQuery::new(10).expect("query"))
        .expect("backup snapshot")
    {
        DatabaseSnapshot::App(snapshot) => snapshot,
        DatabaseSnapshot::Workspace(_) => panic!("expected app snapshot"),
    };
    let source_snapshot = match source
        .snapshot(SnapshotQuery::new(10).expect("query"))
        .expect("source snapshot")
    {
        DatabaseSnapshot::App(snapshot) => snapshot,
        DatabaseSnapshot::Workspace(_) => panic!("expected app snapshot"),
    };
    assert_eq!(backup_snapshot.recents.len(), 1);
    assert_eq!(backup_snapshot.recents[0].workspace_id, "captured");
    assert_eq!(source_snapshot.recents.len(), 2);
    assert!(source_snapshot.generation > backup_snapshot.generation);
    assert!(source.online_backup_to(&source_descriptor.path).is_err());
}
