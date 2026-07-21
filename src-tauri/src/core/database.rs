use std::fs;
use std::io::Write as IoWrite;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write as _,
};

use crate::artifact::{ArtifactHistoryKind, ArtifactRecord, ArtifactWorkspaceUiState};

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Params, TransactionBehavior, params};
use rusqlite_migration::{M, Migrations};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DATABASE_BUSY_TIMEOUT: Duration = Duration::from_millis(1_500);
pub const DATABASE_QUEUE_CAPACITY: usize = 32;
pub const MAX_READ_RECORDS: usize = 250;
pub const MAX_SECURITY_CURRENT_RECORDS: usize = 4_096;
pub const MAX_SECURITY_BATCH_RECORDS: usize = MAX_SECURITY_CURRENT_RECORDS;
pub const MAX_CONCURRENT_AUXILIARY_CONNECTIONS: usize = 32;
pub const MAX_TEXT_FIELD_BYTES: usize = 1_048_576;
pub const MAX_SNAPSHOT_TEXT_BYTES: usize = 4_194_304;
pub const MAX_RUNTIME_DOCUMENT_BYTES: usize = 8 * 1_024 * 1_024;
pub const MAX_SESSION_DOCUMENT_BYTES: usize = 64 * 1_024 * 1_024;
pub const MAX_ARTIFACT_DOCUMENT_BYTES: usize = 16 * 1_024 * 1_024;
pub const MAX_ARTIFACT_UI_DOCUMENT_BYTES: usize = 64 * 1_024;
pub const MAX_RUNTIME_STATE_DOCUMENTS: usize = 256;
pub const MAX_SESSION_RECORDS: usize = 100_000;
pub const MAX_ARTIFACT_RECORDS: usize = 100_000;
pub const MAX_ARTIFACT_RECORDS_PER_SESSION: usize = 4_096;
pub const MAX_ARTIFACT_EVENTS: usize = 1_000_000;
pub const MAX_DIAGNOSTIC_MESSAGE_BYTES: usize = 16_384;
pub const MAX_WORKSPACE_DISPLAY_NAME_BYTES: usize = 512;
pub const MAX_PROJECT_DISPLAY_NAME_BYTES: usize = 256;

const APP_SCHEMA_VERSION: usize = 6;
const WORKSPACE_SCHEMA_VERSION: usize = 5;
static AUXILIARY_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

struct AuxiliaryConnectionPermit;

impl AuxiliaryConnectionPermit {
    fn acquire() -> DatabaseResult<Self> {
        AUXILIARY_CONNECTIONS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < MAX_CONCURRENT_AUXILIARY_CONNECTIONS).then_some(current + 1)
            })
            .map_err(|_| {
                DatabaseError::Actor(format!(
                    "auxiliary database connection limit {MAX_CONCURRENT_AUXILIARY_CONNECTIONS} reached"
                ))
            })?;
        Ok(Self)
    }
}

impl Drop for AuxiliaryConnectionPermit {
    fn drop(&mut self) {
        AUXILIARY_CONNECTIONS.fetch_sub(1, Ordering::AcqRel);
    }
}

struct BoundedConnection {
    connection: Connection,
    _permit: AuxiliaryConnectionPermit,
}

impl Deref for BoundedConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl DerefMut for BoundedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseKind {
    App,
    Workspace { workspace_id: String },
}

impl DatabaseKind {
    fn label(&self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Workspace { .. } => "workspace",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseDescriptor {
    pub kind: DatabaseKind,
    pub path: PathBuf,
    pub recovery_dir: PathBuf,
}

impl DatabaseDescriptor {
    pub fn app(c4os_home: impl AsRef<Path>) -> Self {
        let c4os_home = c4os_home.as_ref();
        Self {
            kind: DatabaseKind::App,
            path: c4os_home.join("state/app.sqlite3"),
            recovery_dir: c4os_home.join("state/recovery"),
        }
    }

    pub fn workspace(working_copy: impl AsRef<Path>, workspace_id: impl Into<String>) -> Self {
        let working_copy = working_copy.as_ref();
        let workspace_id = workspace_id.into();
        let recovery_parent = working_copy.parent().unwrap_or_else(|| Path::new("."));
        Self {
            kind: DatabaseKind::Workspace {
                workspace_id: workspace_id.clone(),
            },
            path: working_copy.join("state/workspace.sqlite3"),
            recovery_dir: recovery_parent.join("recovery").join(workspace_id),
        }
    }

    pub fn workspace_with_recovery_dir(
        working_copy: impl AsRef<Path>,
        workspace_id: impl Into<String>,
        recovery_dir: impl Into<PathBuf>,
    ) -> Self {
        let working_copy = working_copy.as_ref();
        Self {
            kind: DatabaseKind::Workspace {
                workspace_id: workspace_id.into(),
            },
            path: working_copy.join("state/workspace.sqlite3"),
            recovery_dir: recovery_dir.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("database operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("database migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("database actor stopped")]
    ActorStopped,
    #[error("database actor failed: {0}")]
    Actor(String),
    #[error("operation requires a {expected} database, found {actual}")]
    WrongKind {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("invalid database input: {0}")]
    InvalidInput(String),
    #[error("database compare-and-swap conflict: {0}")]
    Conflict(String),
    #[error("database validation failed: {0}")]
    Validation(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SchemaObject {
    object_type: String,
    name: String,
    table_name: String,
    sql: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub previous_version: usize,
    pub current_version: usize,
    pub backup_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotQuery {
    max_records: usize,
    include_inactive: bool,
}

impl SnapshotQuery {
    pub fn new(max_records: usize) -> DatabaseResult<Self> {
        if !(1..=MAX_READ_RECORDS).contains(&max_records) {
            return Err(DatabaseError::InvalidInput(format!(
                "max_records must be between 1 and {MAX_READ_RECORDS}"
            )));
        }

        Ok(Self {
            max_records,
            include_inactive: false,
        })
    }

    pub fn including_inactive(mut self) -> Self {
        self.include_inactive = true;
        self
    }

    pub fn max_records(self) -> usize {
        self.max_records
    }

    pub fn includes_inactive(self) -> bool {
        self.include_inactive
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationRecord {
    pub installation_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentWorkspaceRecord {
    pub workspace_id: String,
    pub display_name: String,
    pub archive_path: String,
    pub last_opened_at: i64,
    pub lifecycle_state: LifecycleState,
    pub inactivated_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceRecord {
    pub workspace_id: String,
    pub display_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub lifecycle_state: LifecycleState,
    pub inactivated_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRecord {
    pub workspace_id: String,
    pub project_id: String,
    pub display_name: String,
    pub current_path: String,
    pub last_known_path: String,
    pub path_state: ProjectPathState,
    pub position: i64,
    pub lifecycle_state: LifecycleState,
    pub inactivated_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatRecord {
    pub workspace_id: String,
    pub project_id: String,
    pub chat_id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub lifecycle_state: LifecycleState,
    pub inactivated_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationSnapshotRecord {
    pub workspace_id: String,
    pub scope_kind: String,
    pub scope_id: String,
    pub canonical_document: String,
    pub generation: u64,
    pub activated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppConfigurationSnapshotRecord {
    pub canonical_document: String,
    pub generation: u64,
    pub activated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticRecord {
    pub diagnostic_id: String,
    pub category: String,
    pub message: String,
    pub created_at: i64,
}

/// Durable current state for an Action Gateway security record. The canonical
/// document is the complete serialized authorization, approval, intent,
/// decision, or result record. Every save also appends an immutable event row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityJournalRecord {
    pub record_kind: String,
    pub record_id: String,
    pub run_id: String,
    pub action_id: String,
    pub state: String,
    pub canonical_document: String,
    pub recorded_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityJournalEvent {
    pub event_id: i64,
    pub record_kind: String,
    pub record_id: String,
    pub run_id: String,
    pub action_id: String,
    pub state: String,
    pub canonical_document: String,
    pub recorded_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityJournalPage {
    pub events: Vec<SecurityJournalEvent>,
    /// Pass this exclusive event identifier to the next page request.
    pub next_before_event_id: Option<i64>,
}

/// Strict provider/runtime state validated by its owning service before this
/// canonical JSON document reaches the single-writer actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStateDocumentRecord {
    pub document_kind: String,
    pub document_id: String,
    pub generation: u64,
    pub canonical_document: String,
    pub updated_at_ms: u64,
}

/// Complete strict session JSON. The session domain validates every immutable
/// child record before create or compare-and-swap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSessionDocumentRecord {
    pub workspace_id: String,
    pub session_id: String,
    pub revision: u64,
    pub canonical_document: String,
    pub updated_at_ms: u64,
}

/// Strict conversation UI state validated by the conversation application
/// service before it reaches the Workspace single-writer actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceConversationStateRecord {
    pub workspace_id: String,
    pub generation: u64,
    pub canonical_document: String,
    pub updated_at_ms: u64,
}

/// Complete strict artifact JSON. The artifact domain validates provider and
/// state schemas before this record reaches the Workspace single-writer actor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceArtifactDocumentRecord {
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub artifact_id: String,
    pub provider_kind: String,
    pub provider_version: u16,
    pub state_schema_version: u16,
    pub revision: u64,
    pub canonical_document: String,
    pub updated_at_ms: u64,
}

/// Strict scalar Artifact Workspace UI state. The artifact domain validates
/// the focused identity against a focus-capable record before persistence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceArtifactUiStateRecord {
    pub workspace_id: String,
    pub revision: u64,
    pub canonical_document: String,
    pub updated_at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleState {
    Active,
    Inactive,
}

impl LifecycleState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }

    fn parse(value: &str) -> DatabaseResult<Self> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            value => Err(DatabaseError::Validation(format!(
                "unknown lifecycle state {value}"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectPathState {
    Found,
    Missing,
    Relocated,
}

impl ProjectPathState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::Missing => "missing",
            Self::Relocated => "relocated",
        }
    }

    fn parse(value: &str) -> DatabaseResult<Self> {
        match value {
            "found" => Ok(Self::Found),
            "missing" => Ok(Self::Missing),
            "relocated" => Ok(Self::Relocated),
            value => Err(DatabaseError::Validation(format!(
                "unknown project path state {value}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSnapshot {
    pub installation: Option<InstallationRecord>,
    pub recents: Vec<RecentWorkspaceRecord>,
    pub configuration_lkg: Option<AppConfigurationSnapshotRecord>,
    pub generation: u64,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub workspace: Option<WorkspaceRecord>,
    pub projects: Vec<ProjectRecord>,
    pub chats: Vec<ChatRecord>,
    pub configurations: Vec<ConfigurationSnapshotRecord>,
    pub diagnostics: Vec<DiagnosticRecord>,
    pub generation: u64,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseSnapshot {
    App(AppSnapshot),
    Workspace(WorkspaceSnapshot),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceDatabaseInspection {
    pub schema_version: usize,
    pub snapshot: WorkspaceSnapshot,
    pub counts: WorkspaceRecordCounts,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceRecordCounts {
    pub projects: usize,
    pub chats: usize,
    pub configurations: usize,
    pub diagnostics: usize,
}

impl WorkspaceRecordCounts {
    fn snapshot_capacity(self) -> usize {
        self.projects
            .max(self.chats)
            .max(self.configurations)
            .max(self.diagnostics)
            .max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceBackupBarrier {
    pub snapshot: WorkspaceSnapshot,
    pub generation: u64,
    pub counts: WorkspaceRecordCounts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabasePragmas {
    pub foreign_keys: bool,
    pub journal_mode: String,
    pub synchronous: i64,
    pub busy_timeout_millis: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InactiveEntity {
    Workspace,
    Project { project_id: String },
    Chat { chat_id: String },
}

#[derive(Debug)]
enum WriteCommand {
    UpsertInstallation {
        record: InstallationRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    RecordRecent {
        record: RecentWorkspaceRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    ActivateAppConfiguration {
        record: AppConfigurationSnapshotRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    CreateWorkspace {
        record: WorkspaceRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    AddProject {
        record: ProjectRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    AddChat {
        record: ChatRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    AddProjectAndChat {
        project: ProjectRecord,
        chat: ChatRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    ReorderProjects {
        ordered_project_ids: Vec<String>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    UpdateProjectPath {
        project_id: String,
        current_path: String,
        last_known_path: String,
        path_state: ProjectPathState,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    RenameProject {
        project_id: String,
        display_name: String,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    ActivateConfiguration {
        record: ConfigurationSnapshotRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    RecordDiagnostic {
        record: DiagnosticRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    SaveSecurityRecords {
        records: Vec<SecurityJournalRecord>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    SaveRuntimeStateDocument {
        record: RuntimeStateDocumentRecord,
        expected_generation: Option<u64>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    SaveConversationState {
        record: WorkspaceConversationStateRecord,
        expected_generation: Option<u64>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    SaveArtifactDocument {
        record: WorkspaceArtifactDocumentRecord,
        expected_revision: Option<u64>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    SaveArtifactUiState {
        record: WorkspaceArtifactUiStateRecord,
        expected_revision: Option<u64>,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    CreateSessionDocument {
        record: WorkspaceSessionDocumentRecord,
        active_project_id: String,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    PromoteSessionDocument {
        record: WorkspaceSessionDocumentRecord,
        chat: ChatRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    CompareAndSwapSessionDocument {
        record: WorkspaceSessionDocumentRecord,
        expected_revision: u64,
        active_project_id: String,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    Inactivate {
        entity: InactiveEntity,
        inactivated_at: i64,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    CompensateWorkspaceInactivation {
        expected_generation: u64,
        prior_record: WorkspaceRecord,
        reply: mpsc::Sender<DatabaseResult<u64>>,
    },
    OnlineBackup {
        destination: PathBuf,
        reply: mpsc::Sender<DatabaseResult<()>>,
    },
    BackupWorkspaceSnapshot {
        destination: PathBuf,
        query: SnapshotQuery,
        reply: mpsc::Sender<DatabaseResult<WorkspaceBackupBarrier>>,
    },
    BackupCompleteWorkspaceSnapshot {
        destination: PathBuf,
        include_inactive: bool,
        reply: mpsc::Sender<DatabaseResult<WorkspaceBackupBarrier>>,
    },
    Shutdown,
}

pub struct DatabaseActor {
    descriptor: DatabaseDescriptor,
    sender: SyncSender<WriteCommand>,
    worker: Option<JoinHandle<()>>,
    _ownership_lock: fs::File,
}

impl DatabaseActor {
    pub fn start(descriptor: DatabaseDescriptor) -> DatabaseResult<(Self, MigrationReport)> {
        validate_descriptor(&descriptor)?;
        let ownership_lock = acquire_writer_ownership(&descriptor)?;
        let (sender, receiver) = mpsc::sync_channel(DATABASE_QUEUE_CAPACITY);
        let (startup_sender, startup_receiver) = mpsc::sync_channel(1);
        let worker_descriptor = descriptor.clone();
        let thread_name = format!("c4os-{}-database-writer", descriptor.kind.label());
        let worker = thread::Builder::new()
            .name(thread_name)
            .spawn(move || writer_loop(worker_descriptor, receiver, startup_sender))?;

        let report = startup_receiver
            .recv()
            .map_err(|_| DatabaseError::ActorStopped)??;

        Ok((
            Self {
                descriptor,
                sender,
                worker: Some(worker),
                _ownership_lock: ownership_lock,
            },
            report,
        ))
    }

    pub fn descriptor(&self) -> &DatabaseDescriptor {
        &self.descriptor
    }

    pub fn upsert_installation(&self, record: InstallationRecord) -> DatabaseResult<u64> {
        self.require_app()?;
        self.request(|reply| WriteCommand::UpsertInstallation { record, reply })
    }

    pub fn record_recent_workspace(&self, record: RecentWorkspaceRecord) -> DatabaseResult<u64> {
        self.require_app()?;
        self.request(|reply| WriteCommand::RecordRecent { record, reply })
    }

    pub fn activate_app_configuration(
        &self,
        record: AppConfigurationSnapshotRecord,
    ) -> DatabaseResult<u64> {
        self.require_app()?;
        self.request(|reply| WriteCommand::ActivateAppConfiguration { record, reply })
    }

    pub fn app_configuration_lkg(&self) -> DatabaseResult<Option<AppConfigurationSnapshotRecord>> {
        self.require_app()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_app_configuration_lkg(&connection)
    }

    pub fn create_workspace(&self, record: WorkspaceRecord) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::CreateWorkspace { record, reply })
    }

    pub fn add_project(&self, record: ProjectRecord) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::AddProject { record, reply })
    }

    pub fn add_chat(&self, record: ChatRecord) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::AddChat { record, reply })
    }

    pub fn add_project_and_chat(
        &self,
        project: ProjectRecord,
        chat: ChatRecord,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&project.workspace_id)?;
        self.require_workspace_id(&chat.workspace_id)?;
        self.request(|reply| WriteCommand::AddProjectAndChat {
            project,
            chat,
            reply,
        })
    }

    pub fn reorder_projects(&self, ordered_project_ids: Vec<String>) -> DatabaseResult<u64> {
        self.require_workspace()?;
        if ordered_project_ids.is_empty() {
            return Err(DatabaseError::InvalidInput(
                "ordered project identifiers cannot be empty".into(),
            ));
        }
        let unique_ids = ordered_project_ids.iter().collect::<HashSet<_>>();
        if unique_ids.len() != ordered_project_ids.len() {
            return Err(DatabaseError::InvalidInput(
                "project order cannot repeat a Project identifier".into(),
            ));
        }
        self.request(|reply| WriteCommand::ReorderProjects {
            ordered_project_ids,
            reply,
        })
    }

    pub fn update_project_path(
        &self,
        project_id: impl Into<String>,
        current_path: impl Into<String>,
        last_known_path: impl Into<String>,
        path_state: ProjectPathState,
    ) -> DatabaseResult<u64> {
        self.require_workspace()?;
        self.request(|reply| WriteCommand::UpdateProjectPath {
            project_id: project_id.into(),
            current_path: current_path.into(),
            last_known_path: last_known_path.into(),
            path_state,
            reply,
        })
    }

    pub fn rename_project(
        &self,
        project_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> DatabaseResult<u64> {
        self.require_workspace()?;
        self.request(|reply| WriteCommand::RenameProject {
            project_id: project_id.into(),
            display_name: display_name.into(),
            reply,
        })
    }

    /// Confirms one exact active Project inside this actor's bound Workspace.
    /// Runtime authority uses this narrow lookup instead of accepting a
    /// renderer-supplied Project identifier or a potentially truncated
    /// Workspace snapshot.
    pub fn active_project_exists(&self, project_id: &str) -> DatabaseResult<bool> {
        let workspace_id = self.require_workspace()?;
        require_nonempty("project_id", project_id)?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM projects
             WHERE workspace_id = ?1 AND project_id = ?2
               AND lifecycle_state = 'active'",
            params![workspace_id, project_id],
            |row| row.get(0),
        )?;
        match count {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DatabaseError::Validation(
                "active Project identity is not unique".into(),
            )),
        }
    }

    pub fn activate_configuration(
        &self,
        record: ConfigurationSnapshotRecord,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::ActivateConfiguration { record, reply })
    }

    pub fn record_diagnostic(&self, record: DiagnosticRecord) -> DatabaseResult<u64> {
        self.require_workspace()?;
        self.request(|reply| WriteCommand::RecordDiagnostic { record, reply })
    }

    /// Saves the latest state and appends the same canonical record to the
    /// immutable security event journal at one serialized writer barrier.
    pub fn save_security_record(&self, record: SecurityJournalRecord) -> DatabaseResult<u64> {
        self.save_security_records(vec![record])
    }

    pub fn save_security_records(
        &self,
        records: Vec<SecurityJournalRecord>,
    ) -> DatabaseResult<u64> {
        self.require_app()?;
        if records.is_empty() || records.len() > MAX_SECURITY_BATCH_RECORDS {
            return Err(DatabaseError::InvalidInput(format!(
                "security batch must contain between 1 and {MAX_SECURITY_BATCH_RECORDS} records"
            )));
        }
        self.request(|reply| WriteCommand::SaveSecurityRecords { records, reply })
    }

    pub fn security_records(
        &self,
        query: SnapshotQuery,
    ) -> DatabaseResult<Vec<SecurityJournalRecord>> {
        self.require_app()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_security_records(&connection, query)
    }

    pub fn security_events(
        &self,
        query: SnapshotQuery,
    ) -> DatabaseResult<Vec<SecurityJournalEvent>> {
        self.require_app()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        let page = read_security_event_page(&connection, None, query)?;
        if page.next_before_event_id.is_some() {
            return Err(DatabaseError::Validation(
                "security event read is truncated; use security_event_page with its cursor".into(),
            ));
        }
        Ok(page.events)
    }

    pub fn security_event_page(
        &self,
        before_event_id: Option<i64>,
        query: SnapshotQuery,
    ) -> DatabaseResult<SecurityJournalPage> {
        self.require_app()?;
        if before_event_id.is_some_and(|value| value <= 0) {
            return Err(DatabaseError::InvalidInput(
                "security event cursor must be positive".into(),
            ));
        }
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_security_event_page(&connection, before_event_id, query)
    }

    /// Returns the complete bounded set needed for fail-closed gateway restart.
    pub fn recoverable_security_records(&self) -> DatabaseResult<Vec<SecurityJournalRecord>> {
        self.require_app()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_recoverable_security_records(&connection)
    }

    pub fn save_runtime_state_document(
        &self,
        record: RuntimeStateDocumentRecord,
        expected_generation: Option<u64>,
    ) -> DatabaseResult<u64> {
        self.require_app()?;
        self.request(|reply| WriteCommand::SaveRuntimeStateDocument {
            record,
            expected_generation,
            reply,
        })
    }

    pub fn runtime_state_document(
        &self,
        document_kind: &str,
        document_id: &str,
    ) -> DatabaseResult<Option<RuntimeStateDocumentRecord>> {
        self.require_app()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_runtime_state_document(&connection, document_kind, document_id)
    }

    pub fn save_conversation_state(
        &self,
        record: WorkspaceConversationStateRecord,
        expected_generation: Option<u64>,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::SaveConversationState {
            record,
            expected_generation,
            reply,
        })
    }

    pub fn conversation_state(&self) -> DatabaseResult<Option<WorkspaceConversationStateRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_conversation_state(&connection, workspace_id)
    }

    pub fn save_artifact_document(
        &self,
        record: WorkspaceArtifactDocumentRecord,
        expected_revision: Option<u64>,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::SaveArtifactDocument {
            record,
            expected_revision,
            reply,
        })
    }

    pub fn artifact_document(
        &self,
        artifact_id: &str,
    ) -> DatabaseResult<Option<WorkspaceArtifactDocumentRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_artifact_document(&connection, workspace_id, artifact_id)
    }

    pub fn artifact_documents_for_session(
        &self,
        session_id: &str,
    ) -> DatabaseResult<Vec<WorkspaceArtifactDocumentRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_artifact_documents_for_session(&connection, workspace_id, session_id)
    }

    pub fn save_artifact_ui_state(
        &self,
        record: WorkspaceArtifactUiStateRecord,
        expected_revision: Option<u64>,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::SaveArtifactUiState {
            record,
            expected_revision,
            reply,
        })
    }

    pub fn artifact_ui_state(&self) -> DatabaseResult<Option<WorkspaceArtifactUiStateRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_artifact_ui_state(&connection, workspace_id)
    }

    pub fn create_session_document(
        &self,
        record: WorkspaceSessionDocumentRecord,
        active_project_id: String,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::CreateSessionDocument {
            record,
            active_project_id,
            reply,
        })
    }

    /// Atomically creates the first durable Chat row and its complete session
    /// document. A same-identity active Chat is updated in place so callers
    /// from older epochs that pre-created the shell remain compatible, while
    /// an inactive or cross-Project identity fails closed.
    pub fn promote_session_document(
        &self,
        record: WorkspaceSessionDocumentRecord,
        chat: ChatRecord,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.require_workspace_id(&chat.workspace_id)?;
        self.request(|reply| WriteCommand::PromoteSessionDocument {
            record,
            chat,
            reply,
        })
    }

    pub fn compare_and_swap_session_document(
        &self,
        record: WorkspaceSessionDocumentRecord,
        expected_revision: u64,
        active_project_id: String,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&record.workspace_id)?;
        self.request(|reply| WriteCommand::CompareAndSwapSessionDocument {
            record,
            expected_revision,
            active_project_id,
            reply,
        })
    }

    pub fn session_document(
        &self,
        session_id: &str,
    ) -> DatabaseResult<Option<WorkspaceSessionDocumentRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_session_document(&connection, workspace_id, session_id)
    }

    pub fn session_documents(&self) -> DatabaseResult<Vec<WorkspaceSessionDocumentRecord>> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_session_documents(&connection, workspace_id)
    }

    pub fn inactivate(&self, entity: InactiveEntity, inactivated_at: i64) -> DatabaseResult<u64> {
        self.require_workspace()?;
        self.request(|reply| WriteCommand::Inactivate {
            entity,
            inactivated_at,
            reply,
        })
    }

    pub fn compensate_workspace_inactivation(
        &self,
        expected_generation: u64,
        prior_record: WorkspaceRecord,
    ) -> DatabaseResult<u64> {
        self.require_workspace_id(&prior_record.workspace_id)?;
        self.request(|reply| WriteCommand::CompensateWorkspaceInactivation {
            expected_generation,
            prior_record,
            reply,
        })
    }

    pub fn snapshot(&self, query: SnapshotQuery) -> DatabaseResult<DatabaseSnapshot> {
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        match &self.descriptor.kind {
            DatabaseKind::App => read_app_snapshot(&connection, query).map(DatabaseSnapshot::App),
            DatabaseKind::Workspace { workspace_id } => {
                read_workspace_snapshot(&connection, workspace_id, query)
                    .map(DatabaseSnapshot::Workspace)
            }
        }
    }

    pub fn complete_workspace_snapshot(
        &self,
        include_inactive: bool,
    ) -> DatabaseResult<(WorkspaceSnapshot, WorkspaceRecordCounts)> {
        let workspace_id = self.require_workspace()?;
        let connection = open_read_connection(&self.descriptor.path)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        read_complete_workspace_snapshot(&connection, workspace_id, include_inactive)
    }

    pub fn pragma_snapshot(&self) -> DatabaseResult<DatabasePragmas> {
        let connection = open_read_connection(&self.descriptor.path)?;
        read_pragmas(&connection)
    }

    /// Captures a consistent SQLite snapshot, including committed WAL state, without transferring
    /// writable ownership away from this actor.
    pub fn online_backup_to(&self, destination: impl AsRef<Path>) -> DatabaseResult<()> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(WriteCommand::OnlineBackup {
                destination: destination.as_ref().to_path_buf(),
                reply: reply_sender,
            })
            .map_err(|_| DatabaseError::ActorStopped)?;
        reply_receiver
            .recv()
            .map_err(|_| DatabaseError::ActorStopped)?
    }

    /// Serializes a Workspace database backup and its canonical snapshot at one writer barrier.
    pub fn backup_workspace_snapshot_to(
        &self,
        destination: impl AsRef<Path>,
        query: SnapshotQuery,
    ) -> DatabaseResult<WorkspaceBackupBarrier> {
        self.require_workspace()?;
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(WriteCommand::BackupWorkspaceSnapshot {
                destination: destination.as_ref().to_path_buf(),
                query,
                reply: reply_sender,
            })
            .map_err(|_| DatabaseError::ActorStopped)?;
        reply_receiver
            .recv()
            .map_err(|_| DatabaseError::ActorStopped)?
    }

    pub fn backup_complete_workspace_snapshot_to(
        &self,
        destination: impl AsRef<Path>,
        include_inactive: bool,
    ) -> DatabaseResult<WorkspaceBackupBarrier> {
        self.require_workspace()?;
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(WriteCommand::BackupCompleteWorkspaceSnapshot {
                destination: destination.as_ref().to_path_buf(),
                include_inactive,
                reply: reply_sender,
            })
            .map_err(|_| DatabaseError::ActorStopped)?;
        reply_receiver
            .recv()
            .map_err(|_| DatabaseError::ActorStopped)?
    }

    fn request(
        &self,
        create: impl FnOnce(mpsc::Sender<DatabaseResult<u64>>) -> WriteCommand,
    ) -> DatabaseResult<u64> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(create(reply_sender))
            .map_err(|_| DatabaseError::ActorStopped)?;
        reply_receiver
            .recv()
            .map_err(|_| DatabaseError::ActorStopped)?
    }

    fn require_app(&self) -> DatabaseResult<()> {
        match self.descriptor.kind {
            DatabaseKind::App => Ok(()),
            DatabaseKind::Workspace { .. } => Err(DatabaseError::WrongKind {
                expected: "app",
                actual: "workspace",
            }),
        }
    }

    fn require_workspace(&self) -> DatabaseResult<&str> {
        match &self.descriptor.kind {
            DatabaseKind::Workspace { workspace_id } => Ok(workspace_id),
            DatabaseKind::App => Err(DatabaseError::WrongKind {
                expected: "workspace",
                actual: "app",
            }),
        }
    }

    fn require_workspace_id(&self, workspace_id: &str) -> DatabaseResult<()> {
        let expected = self.require_workspace()?;
        if workspace_id != expected {
            return Err(DatabaseError::InvalidInput(format!(
                "record Workspace {workspace_id} does not match open Workspace {expected}"
            )));
        }
        Ok(())
    }
}

impl Drop for DatabaseActor {
    fn drop(&mut self) {
        let _ = self.sender.send(WriteCommand::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn writer_loop(
    descriptor: DatabaseDescriptor,
    receiver: Receiver<WriteCommand>,
    startup: SyncSender<DatabaseResult<MigrationReport>>,
) {
    let opened = open_and_migrate(&descriptor);
    let (mut connection, report) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            let _ = startup.send(Err(error));
            return;
        }
    };
    if startup.send(Ok(report)).is_err() {
        return;
    }

    while let Ok(command) = receiver.recv() {
        match command {
            WriteCommand::Shutdown => break,
            WriteCommand::UpsertInstallation { record, reply } => {
                reply_result(reply, write_installation(&mut connection, record));
            }
            WriteCommand::RecordRecent { record, reply } => {
                reply_result(reply, write_recent(&mut connection, record));
            }
            WriteCommand::ActivateAppConfiguration { record, reply } => {
                reply_result(reply, write_app_configuration(&mut connection, record));
            }
            WriteCommand::CreateWorkspace { record, reply } => {
                reply_result(reply, write_workspace(&mut connection, record));
            }
            WriteCommand::AddProject { record, reply } => {
                reply_result(reply, write_project(&mut connection, record));
            }
            WriteCommand::AddChat { record, reply } => {
                reply_result(reply, write_chat(&mut connection, record));
            }
            WriteCommand::AddProjectAndChat {
                project,
                chat,
                reply,
            } => {
                reply_result(
                    reply,
                    write_project_and_chat(&mut connection, project, chat),
                );
            }
            WriteCommand::ReorderProjects {
                ordered_project_ids,
                reply,
            } => reply_result(
                reply,
                write_project_order(&mut connection, &descriptor.kind, ordered_project_ids),
            ),
            WriteCommand::UpdateProjectPath {
                project_id,
                current_path,
                last_known_path,
                path_state,
                reply,
            } => reply_result(
                reply,
                write_project_path(
                    &mut connection,
                    &descriptor.kind,
                    &project_id,
                    &current_path,
                    &last_known_path,
                    path_state,
                ),
            ),
            WriteCommand::RenameProject {
                project_id,
                display_name,
                reply,
            } => reply_result(
                reply,
                write_project_name(
                    &mut connection,
                    &descriptor.kind,
                    &project_id,
                    &display_name,
                ),
            ),
            WriteCommand::ActivateConfiguration { record, reply } => {
                reply_result(reply, write_configuration(&mut connection, record));
            }
            WriteCommand::RecordDiagnostic { record, reply } => reply_result(
                reply,
                write_diagnostic(&mut connection, &descriptor.kind, record),
            ),
            WriteCommand::SaveSecurityRecords { records, reply } => {
                reply_result(reply, write_security_records(&mut connection, records))
            }
            WriteCommand::SaveRuntimeStateDocument {
                record,
                expected_generation,
                reply,
            } => reply_result(
                reply,
                write_runtime_state_document(&mut connection, record, expected_generation),
            ),
            WriteCommand::SaveConversationState {
                record,
                expected_generation,
                reply,
            } => reply_result(
                reply,
                write_conversation_state(
                    &mut connection,
                    &descriptor.kind,
                    record,
                    expected_generation,
                ),
            ),
            WriteCommand::SaveArtifactDocument {
                record,
                expected_revision,
                reply,
            } => reply_result(
                reply,
                write_artifact_document(
                    &mut connection,
                    &descriptor.kind,
                    record,
                    expected_revision,
                ),
            ),
            WriteCommand::SaveArtifactUiState {
                record,
                expected_revision,
                reply,
            } => reply_result(
                reply,
                write_artifact_ui_state(
                    &mut connection,
                    &descriptor.kind,
                    record,
                    expected_revision,
                ),
            ),
            WriteCommand::CreateSessionDocument {
                record,
                active_project_id,
                reply,
            } => reply_result(
                reply,
                write_session_document(
                    &mut connection,
                    &descriptor.kind,
                    record,
                    &active_project_id,
                    None,
                ),
            ),
            WriteCommand::PromoteSessionDocument {
                record,
                chat,
                reply,
            } => reply_result(
                reply,
                write_promoted_session_document(&mut connection, &descriptor.kind, record, chat),
            ),
            WriteCommand::CompareAndSwapSessionDocument {
                record,
                expected_revision,
                active_project_id,
                reply,
            } => reply_result(
                reply,
                write_session_document(
                    &mut connection,
                    &descriptor.kind,
                    record,
                    &active_project_id,
                    Some(expected_revision),
                ),
            ),
            WriteCommand::Inactivate {
                entity,
                inactivated_at,
                reply,
            } => reply_result(
                reply,
                write_inactivation(&mut connection, &descriptor.kind, entity, inactivated_at),
            ),
            WriteCommand::CompensateWorkspaceInactivation {
                expected_generation,
                prior_record,
                reply,
            } => reply_result(
                reply,
                write_workspace_inactivation_compensation(
                    &mut connection,
                    &descriptor.kind,
                    expected_generation,
                    prior_record,
                ),
            ),
            WriteCommand::OnlineBackup { destination, reply } => {
                let _ = reply.send(perform_online_backup(
                    &connection,
                    &descriptor,
                    &destination,
                ));
            }
            WriteCommand::BackupWorkspaceSnapshot {
                destination,
                query,
                reply,
            } => {
                let _ = reply.send(backup_workspace_snapshot_at_barrier(
                    &connection,
                    &descriptor,
                    &destination,
                    query,
                ));
            }
            WriteCommand::BackupCompleteWorkspaceSnapshot {
                destination,
                include_inactive,
                reply,
            } => {
                let _ = reply.send(backup_complete_workspace_snapshot_at_barrier(
                    &connection,
                    &descriptor,
                    &destination,
                    include_inactive,
                ));
            }
        }
    }
}

fn reply_result(reply: mpsc::Sender<DatabaseResult<u64>>, result: DatabaseResult<u64>) {
    let _ = reply.send(result);
}

fn validate_descriptor(descriptor: &DatabaseDescriptor) -> DatabaseResult<()> {
    if descriptor.path.file_name().is_none() {
        return Err(DatabaseError::InvalidInput(
            "database path must name a file".into(),
        ));
    }
    if let DatabaseKind::Workspace { workspace_id } = &descriptor.kind {
        require_nonempty("workspace_id", workspace_id)?;
        validate_text_field("workspace_id", workspace_id, 256)?;
        if workspace_id == "."
            || workspace_id == ".."
            || workspace_id.contains('/')
            || workspace_id.contains('\\')
        {
            return Err(DatabaseError::InvalidInput(
                "workspace_id must be a single safe path component".into(),
            ));
        }
        let state_dir = descriptor.path.parent().ok_or_else(|| {
            DatabaseError::InvalidInput("Workspace database path has no state directory".into())
        })?;
        let working_copy = state_dir.parent().ok_or_else(|| {
            DatabaseError::InvalidInput("Workspace database path has no working-copy root".into())
        })?;
        if descriptor.recovery_dir.starts_with(working_copy) {
            return Err(DatabaseError::InvalidInput(
                "Workspace database recovery directory must be outside the portable working copy"
                    .into(),
            ));
        }
    }
    Ok(())
}

fn acquire_writer_ownership(descriptor: &DatabaseDescriptor) -> DatabaseResult<fs::File> {
    fs::create_dir_all(&descriptor.recovery_dir)?;
    let filename = descriptor
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| DatabaseError::InvalidInput("database filename is not UTF-8".into()))?;
    let lock_path = descriptor
        .recovery_dir
        .join(format!("{filename}.writer.lock"));
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;
    lock.try_lock().map_err(|error| {
        DatabaseError::Actor(format!(
            "single-writer ownership for {} is unavailable: {error}",
            descriptor.path.display()
        ))
    })?;
    Ok(lock)
}

fn reject_source_as_backup_destination(source: &Path, destination: &Path) -> DatabaseResult<()> {
    if source == destination {
        return Err(DatabaseError::InvalidInput(
            "online backup destination cannot be the authoritative database".into(),
        ));
    }
    if destination.exists()
        && source.exists()
        && fs::canonicalize(source)? == fs::canonicalize(destination)?
    {
        return Err(DatabaseError::InvalidInput(
            "online backup destination resolves to the authoritative database".into(),
        ));
    }
    Ok(())
}

fn open_and_migrate(
    descriptor: &DatabaseDescriptor,
) -> DatabaseResult<(Connection, MigrationReport)> {
    if let Some(parent) = descriptor.path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut connection = Connection::open(&descriptor.path)?;
    configure_write_connection(&connection)?;
    let migrations = migrations_for(&descriptor.kind);
    migrations.validate()?;
    let previous_version = read_user_version(&connection)?;
    let target_version = target_schema_version(&descriptor.kind);
    let backup_path = if previous_version != target_version {
        Some(create_validated_backup(
            &connection,
            descriptor,
            previous_version,
            target_version,
        )?)
    } else {
        None
    };

    if let Err(error) = migrations.to_latest(&mut connection) {
        let mut message = format!(
            "{} migration from version {previous_version} to {target_version} failed: {error}",
            descriptor.kind.label()
        );
        if let Some(backup_path) = &backup_path {
            match restore_validated_backup(&mut connection, backup_path, previous_version) {
                Ok(()) => message.push_str("; pre-migration database restored"),
                Err(restore_error) => {
                    let _ = write!(message, "; backup restoration failed: {restore_error}");
                }
            }
        }
        let _ = write_migration_diagnostic(descriptor, &message);
        return Err(DatabaseError::Migration(error));
    }

    if let Err(error) = validate_database(&connection, &descriptor.kind) {
        let mut message = format!(
            "{} post-migration validation from version {previous_version} to {target_version} failed: {error}",
            descriptor.kind.label()
        );
        if let Some(backup_path) = &backup_path {
            match restore_validated_backup(&mut connection, backup_path, previous_version) {
                Ok(()) => message.push_str("; pre-migration database restored"),
                Err(restore_error) => {
                    let _ = write!(message, "; backup restoration failed: {restore_error}");
                }
            }
        }
        let _ = write_migration_diagnostic(descriptor, &message);
        return Err(error);
    }

    Ok((
        connection,
        MigrationReport {
            previous_version,
            current_version: target_version,
            backup_path,
        },
    ))
}

fn configure_write_connection(connection: &Connection) -> DatabaseResult<()> {
    connection.busy_timeout(DATABASE_BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(DatabaseError::Validation(format!(
            "SQLite refused WAL mode and returned {journal_mode}"
        )));
    }
    connection.pragma_update(None, "synchronous", "FULL")?;
    Ok(())
}

fn configure_standalone_backup_connection(connection: &Connection) -> DatabaseResult<()> {
    connection.busy_timeout(DATABASE_BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("delete") {
        return Err(DatabaseError::Validation(format!(
            "SQLite refused standalone backup journal mode and returned {journal_mode}"
        )));
    }
    connection.pragma_update(None, "synchronous", "FULL")?;
    Ok(())
}

fn open_read_connection(path: &Path) -> DatabaseResult<BoundedConnection> {
    let permit = AuxiliaryConnectionPermit::acquire()?;
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(DATABASE_BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(BoundedConnection {
        connection,
        _permit: permit,
    })
}

fn open_backup_connection(path: &Path) -> DatabaseResult<BoundedConnection> {
    let permit = AuxiliaryConnectionPermit::acquire()?;
    let connection = Connection::open(path)?;
    Ok(BoundedConnection {
        connection,
        _permit: permit,
    })
}

fn migrations_for(kind: &DatabaseKind) -> Migrations<'static> {
    match kind {
        DatabaseKind::App => app_migrations(),
        DatabaseKind::Workspace { .. } => workspace_migrations(),
    }
}

fn app_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE installations (
                installation_id TEXT PRIMARY KEY NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE recent_workspaces (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                display_name TEXT NOT NULL,
                archive_path TEXT NOT NULL,
                last_opened_at INTEGER NOT NULL,
                lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN ('active', 'inactive')),
                inactivated_at INTEGER,
                CHECK ((lifecycle_state = 'active' AND inactivated_at IS NULL)
                    OR (lifecycle_state = 'inactive' AND inactivated_at IS NOT NULL))
            );
            CREATE TABLE durable_generation (
                singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
                generation INTEGER NOT NULL CHECK (generation >= 0)
            );
            INSERT INTO durable_generation(singleton, generation) VALUES (1, 0);",
        )
        .comment("app installation, recents, and durable generation"),
        M::up(
            "CREATE TABLE app_diagnostics (
                diagnostic_id TEXT PRIMARY KEY NOT NULL,
                category TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX recent_workspaces_active_order
                ON recent_workspaces(lifecycle_state, last_opened_at DESC, workspace_id);",
        )
        .comment("app diagnostic and bounded recent ordering"),
        M::up(
            "CREATE TABLE app_configuration_lkg (
                singleton INTEGER PRIMARY KEY NOT NULL CHECK (singleton = 1),
                canonical_document TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 0),
                activated_at INTEGER NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64)
            );",
        )
        .comment("durable app configuration last-known-good document"),
        M::up(
            "CREATE TABLE security_records (
                record_kind TEXT NOT NULL,
                record_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                action_id TEXT NOT NULL,
                state TEXT NOT NULL,
                canonical_document TEXT NOT NULL,
                record_sha256 TEXT NOT NULL CHECK (length(record_sha256) = 64),
                recorded_at_ms INTEGER NOT NULL CHECK (recorded_at_ms >= 0),
                PRIMARY KEY (record_kind, record_id)
            );
            CREATE TABLE security_events (
                event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                record_kind TEXT NOT NULL,
                record_id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                action_id TEXT NOT NULL,
                state TEXT NOT NULL,
                canonical_document TEXT NOT NULL,
                record_sha256 TEXT NOT NULL CHECK (length(record_sha256) = 64),
                recorded_at_ms INTEGER NOT NULL CHECK (recorded_at_ms >= 0)
            );
            CREATE INDEX security_records_run_action
                ON security_records(run_id, action_id, record_kind, record_id);
            CREATE INDEX security_events_run_action_time
                ON security_events(run_id, action_id, recorded_at_ms, event_id);",
        )
        .comment("app-owned Action Gateway current state and append-only transition journal"),
        M::up(
            "CREATE TABLE runtime_state_documents (
                document_kind TEXT NOT NULL CHECK (
                    document_kind IN ('provider-snapshot', 'supervisor-snapshot')
                ),
                document_id TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms >= 0),
                PRIMARY KEY (document_kind, document_id)
            );
            CREATE INDEX runtime_state_documents_updated
                ON runtime_state_documents(document_kind, updated_at_ms DESC, document_id);",
        )
        .comment("app-owned provider and supervised-runtime canonical state"),
        M::up(
            "CREATE TABLE runtime_state_documents_v2 (
                document_kind TEXT NOT NULL CHECK (
                    document_kind IN (
                        'provider-snapshot',
                        'supervisor-snapshot',
                        'runtime-control-plane'
                    )
                ),
                document_id TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms >= 0),
                PRIMARY KEY (document_kind, document_id)
            );
            INSERT INTO runtime_state_documents_v2(
                document_kind, document_id, generation, canonical_document,
                document_sha256, updated_at_ms
            )
            SELECT document_kind, document_id, generation, canonical_document,
                document_sha256, updated_at_ms
            FROM runtime_state_documents;
            DROP INDEX runtime_state_documents_updated;
            DROP TABLE runtime_state_documents;
            ALTER TABLE runtime_state_documents_v2 RENAME TO runtime_state_documents;
            CREATE INDEX runtime_state_documents_updated
                ON runtime_state_documents(document_kind, updated_at_ms DESC, document_id);",
        )
        .comment("atomic runtime supervisor and capability control-plane document"),
    ])
}

fn workspace_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "CREATE TABLE workspaces (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                display_name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN ('active', 'inactive')),
                inactivated_at INTEGER,
                CHECK ((lifecycle_state = 'active' AND inactivated_at IS NULL)
                    OR (lifecycle_state = 'inactive' AND inactivated_at IS NOT NULL))
            );
            CREATE TABLE projects (
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                current_path TEXT NOT NULL,
                last_known_path TEXT NOT NULL,
                path_state TEXT NOT NULL CHECK (path_state IN ('found', 'missing', 'relocated')),
                position INTEGER NOT NULL CHECK (position >= 0),
                lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN ('active', 'inactive')),
                inactivated_at INTEGER,
                PRIMARY KEY (workspace_id, project_id),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id),
                CHECK ((lifecycle_state = 'active' AND inactivated_at IS NULL)
                    OR (lifecycle_state = 'inactive' AND inactivated_at IS NOT NULL))
            );
            CREATE TABLE chats (
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                chat_id TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN ('active', 'inactive')),
                inactivated_at INTEGER,
                PRIMARY KEY (workspace_id, chat_id),
                FOREIGN KEY (workspace_id, project_id)
                    REFERENCES projects(workspace_id, project_id),
                CHECK ((lifecycle_state = 'active' AND inactivated_at IS NULL)
                    OR (lifecycle_state = 'inactive' AND inactivated_at IS NOT NULL))
            );
            CREATE TABLE configuration_lkg (
                workspace_id TEXT NOT NULL,
                scope_kind TEXT NOT NULL CHECK (scope_kind IN ('workspace', 'project', 'chat')),
                scope_id TEXT NOT NULL,
                canonical_document TEXT NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 0),
                activated_at INTEGER NOT NULL,
                PRIMARY KEY (workspace_id, scope_kind, scope_id),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id)
            );
            CREATE TABLE workspace_diagnostics (
                workspace_id TEXT NOT NULL,
                diagnostic_id TEXT NOT NULL,
                category TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (workspace_id, diagnostic_id),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id)
            );
            CREATE TABLE durable_generation (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                generation INTEGER NOT NULL CHECK (generation >= 0),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id)
            );",
        )
        .foreign_key_check()
        .comment("Workspace-owned records and durable generation"),
        M::up(
            "CREATE UNIQUE INDEX projects_workspace_position
                ON projects(workspace_id, position)
                WHERE lifecycle_state = 'active';
            CREATE INDEX projects_workspace_active_order
                ON projects(workspace_id, lifecycle_state, position, project_id);
            CREATE INDEX chats_workspace_project_active
                ON chats(workspace_id, project_id, lifecycle_state, updated_at DESC, chat_id);
            CREATE INDEX configuration_workspace_scope
                ON configuration_lkg(workspace_id, scope_kind, scope_id);
            CREATE INDEX diagnostics_workspace_time
                ON workspace_diagnostics(workspace_id, created_at DESC, diagnostic_id);",
        )
        .foreign_key_check()
        .comment("bounded Workspace query indexes"),
        M::up(
            "CREATE TABLE session_records (
                workspace_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                revision INTEGER NOT NULL CHECK (revision > 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms >= 0),
                PRIMARY KEY (workspace_id, session_id),
                FOREIGN KEY (workspace_id, session_id)
                    REFERENCES chats(workspace_id, chat_id)
            );
            CREATE INDEX session_records_workspace_updated
                ON session_records(workspace_id, updated_at_ms DESC, session_id);",
        )
        .foreign_key_check()
        .comment("Workspace-owned immutable-turn and run-attempt session documents"),
        M::up(
            "CREATE TABLE conversation_state (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                generation INTEGER NOT NULL CHECK (generation > 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms > 0),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id)
            );",
        )
        .foreign_key_check()
        .comment("Workspace-owned conversation selection and durable composer drafts"),
        M::up(
            "CREATE TABLE artifact_records (
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                artifact_id TEXT NOT NULL,
                provider_kind TEXT NOT NULL,
                provider_version INTEGER NOT NULL CHECK (provider_version > 0),
                state_schema_version INTEGER NOT NULL CHECK (state_schema_version > 0),
                revision INTEGER NOT NULL CHECK (revision > 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms > 0),
                PRIMARY KEY (workspace_id, artifact_id),
                FOREIGN KEY (workspace_id, project_id)
                    REFERENCES projects(workspace_id, project_id),
                FOREIGN KEY (workspace_id, session_id)
                    REFERENCES chats(workspace_id, chat_id)
            );
            CREATE TABLE artifact_events (
                event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                artifact_id TEXT NOT NULL,
                provider_kind TEXT NOT NULL,
                provider_version INTEGER NOT NULL CHECK (provider_version > 0),
                state_schema_version INTEGER NOT NULL CHECK (state_schema_version > 0),
                revision INTEGER NOT NULL CHECK (revision > 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms > 0),
                UNIQUE (workspace_id, artifact_id, revision),
                FOREIGN KEY (workspace_id, project_id)
                    REFERENCES projects(workspace_id, project_id),
                FOREIGN KEY (workspace_id, session_id)
                    REFERENCES chats(workspace_id, chat_id),
                FOREIGN KEY (workspace_id, artifact_id)
                    REFERENCES artifact_records(workspace_id, artifact_id)
            );
            CREATE TABLE artifact_workspace_state (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                revision INTEGER NOT NULL CHECK (revision > 0),
                canonical_document TEXT NOT NULL,
                document_sha256 TEXT NOT NULL CHECK (length(document_sha256) = 64),
                updated_at_ms INTEGER NOT NULL CHECK (updated_at_ms > 0),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id)
            );
            CREATE INDEX artifact_records_session_updated
                ON artifact_records(workspace_id, session_id, updated_at_ms, artifact_id);
            CREATE INDEX artifact_events_artifact_revision
                ON artifact_events(workspace_id, artifact_id, revision);",
        )
        .foreign_key_check()
        .comment("Workspace-owned versioned Response Artifact records and immutable history"),
    ])
}

fn target_schema_version(kind: &DatabaseKind) -> usize {
    match kind {
        DatabaseKind::App => APP_SCHEMA_VERSION,
        DatabaseKind::Workspace { .. } => WORKSPACE_SCHEMA_VERSION,
    }
}

fn read_user_version(connection: &Connection) -> DatabaseResult<usize> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    usize::try_from(version)
        .map_err(|_| DatabaseError::Validation(format!("negative SQLite user_version {version}")))
}

pub fn inspect_workspace_database_read_only(
    database_path: impl AsRef<Path>,
    expected_workspace_id: &str,
    query: SnapshotQuery,
) -> DatabaseResult<WorkspaceDatabaseInspection> {
    require_nonempty("expected_workspace_id", expected_workspace_id)?;
    validate_text_field("expected_workspace_id", expected_workspace_id, 256)?;
    let connection = open_read_connection(database_path.as_ref())?;
    connection.execute_batch("BEGIN DEFERRED")?;
    let schema_version =
        validate_workspace_candidate_connection(&connection, expected_workspace_id)?;
    let counts =
        read_workspace_record_counts(&connection, expected_workspace_id, query.include_inactive)?;

    Ok(WorkspaceDatabaseInspection {
        schema_version,
        snapshot: read_workspace_snapshot(&connection, expected_workspace_id, query)?,
        counts,
    })
}

pub fn inspect_complete_workspace_database_read_only(
    database_path: impl AsRef<Path>,
    expected_workspace_id: &str,
    include_inactive: bool,
) -> DatabaseResult<WorkspaceDatabaseInspection> {
    require_nonempty("expected_workspace_id", expected_workspace_id)?;
    validate_text_field("expected_workspace_id", expected_workspace_id, 256)?;
    let connection = open_read_connection(database_path.as_ref())?;
    connection.execute_batch("BEGIN DEFERRED")?;
    let schema_version =
        validate_workspace_candidate_connection(&connection, expected_workspace_id)?;
    let (snapshot, counts) =
        read_complete_workspace_snapshot(&connection, expected_workspace_id, include_inactive)?;
    Ok(WorkspaceDatabaseInspection {
        schema_version,
        snapshot,
        counts,
    })
}

fn validate_workspace_candidate_connection(
    connection: &Connection,
    expected_workspace_id: &str,
) -> DatabaseResult<usize> {
    validate_quick_check(connection)?;
    let schema_version = read_user_version(connection)?;
    if schema_version != WORKSPACE_SCHEMA_VERSION {
        return Err(DatabaseError::Validation(format!(
            "unsupported Workspace database user_version {schema_version}; expected {WORKSPACE_SCHEMA_VERSION}"
        )));
    }
    let kind = DatabaseKind::Workspace {
        workspace_id: expected_workspace_id.into(),
    };
    validate_database(connection, &kind)?;
    let workspace_count: i64 =
        connection.query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))?;
    if workspace_count != 1 {
        return Err(DatabaseError::Validation(format!(
            "Workspace database must contain exactly one identity row, found {workspace_count}"
        )));
    }
    let mut identity_budget = SnapshotTextBudget::default();
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(display_name AS BLOB)),
                    length(CAST(lifecycle_state AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(display_name AS BLOB)) +
                    length(CAST(lifecycle_state AS BLOB))), 0)
         FROM workspaces",
        [],
        "pre-promotion Workspace identity",
        &mut identity_budget,
    )?;
    let actual_workspace_id: String =
        connection.query_row("SELECT workspace_id FROM workspaces LIMIT 1", [], |row| {
            row.get(0)
        })?;
    if actual_workspace_id != expected_workspace_id {
        return Err(DatabaseError::Validation(format!(
            "Workspace database identity {actual_workspace_id} does not match expected {expected_workspace_id}"
        )));
    }
    let generation_rows: i64 = connection.query_row(
        "SELECT COUNT(*) FROM durable_generation WHERE workspace_id = ?1",
        [expected_workspace_id],
        |row| row.get(0),
    )?;
    if generation_rows != 1 {
        return Err(DatabaseError::Validation(format!(
            "Workspace {expected_workspace_id} must have exactly one durable generation row"
        )));
    }

    Ok(schema_version)
}

fn read_workspace_record_counts(
    connection: &Connection,
    workspace_id: &str,
    include_inactive: bool,
) -> DatabaseResult<WorkspaceRecordCounts> {
    if !include_inactive
        && connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM workspaces
                           WHERE workspace_id = ?1 AND lifecycle_state = 'active')",
            [workspace_id],
            |row| row.get::<_, i64>(0),
        )? != 1
    {
        return Ok(WorkspaceRecordCounts::default());
    }
    let lifecycle_filter = if include_inactive {
        ""
    } else {
        "AND lifecycle_state = 'active'"
    };
    let project_sql =
        format!("SELECT COUNT(*) FROM projects WHERE workspace_id = ?1 {lifecycle_filter}");
    let projects: i64 = connection.query_row(&project_sql, [workspace_id], |row| row.get(0))?;
    let (project_join, chat_filter) = if include_inactive {
        ("", "")
    } else {
        (
            "JOIN projects visible_project
                ON visible_project.workspace_id = chats.workspace_id
                AND visible_project.project_id = chats.project_id",
            "AND chats.lifecycle_state = 'active'
             AND visible_project.lifecycle_state = 'active'",
        )
    };
    let chat_sql = format!(
        "SELECT COUNT(*) FROM chats {project_join}
         WHERE chats.workspace_id = ?1 {chat_filter}"
    );
    let chats: i64 = connection.query_row(&chat_sql, [workspace_id], |row| row.get(0))?;
    let configurations: i64 = connection.query_row(
        "SELECT COUNT(*) FROM configuration_lkg WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    let diagnostics: i64 = connection.query_row(
        "SELECT COUNT(*) FROM workspace_diagnostics WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    Ok(WorkspaceRecordCounts {
        projects: count_to_usize(projects, "Project")?,
        chats: count_to_usize(chats, "Chat")?,
        configurations: count_to_usize(configurations, "configuration")?,
        diagnostics: count_to_usize(diagnostics, "diagnostic")?,
    })
}

fn count_to_usize(count: i64, kind: &str) -> DatabaseResult<usize> {
    usize::try_from(count)
        .map_err(|_| DatabaseError::Validation(format!("invalid {kind} record count {count}")))
}

fn read_complete_workspace_snapshot(
    connection: &Connection,
    workspace_id: &str,
    include_inactive: bool,
) -> DatabaseResult<(WorkspaceSnapshot, WorkspaceRecordCounts)> {
    let counts = read_workspace_record_counts(connection, workspace_id, include_inactive)?;
    let query = SnapshotQuery {
        max_records: counts.snapshot_capacity(),
        include_inactive,
    };
    let snapshot = read_workspace_snapshot(connection, workspace_id, query)?;
    if snapshot.truncated {
        return Err(DatabaseError::Validation(
            "complete Workspace snapshot disagreed with its counted capacity".into(),
        ));
    }
    Ok((snapshot, counts))
}

fn perform_online_backup(
    source: &Connection,
    descriptor: &DatabaseDescriptor,
    destination: &Path,
) -> DatabaseResult<()> {
    reject_source_as_backup_destination(&descriptor.path, destination)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let source_version = read_user_version(source)?;
    let mut backup_connection = open_backup_connection(destination)?;
    configure_standalone_backup_connection(&backup_connection)?;
    {
        let backup = Backup::new(source, &mut backup_connection)?;
        backup.run_to_completion(64, Duration::from_millis(2), None)?;
    }
    configure_standalone_backup_connection(&backup_connection)?;
    validate_database(&backup_connection, &descriptor.kind)?;
    let backup_version = read_user_version(&backup_connection)?;
    if backup_version != source_version {
        return Err(DatabaseError::Validation(format!(
            "online backup user_version {backup_version} did not match source {source_version}"
        )));
    }
    drop(backup_connection);
    fs::File::open(destination)?.sync_all()?;
    Ok(())
}

fn backup_workspace_snapshot_at_barrier(
    connection: &Connection,
    descriptor: &DatabaseDescriptor,
    destination: &Path,
    query: SnapshotQuery,
) -> DatabaseResult<WorkspaceBackupBarrier> {
    let expected_workspace_id = workspace_id(&descriptor.kind)?;
    perform_online_backup(connection, descriptor, destination)?;
    let snapshot = read_workspace_snapshot(connection, expected_workspace_id, query)?;
    let backup_inspection =
        inspect_workspace_database_read_only(destination, expected_workspace_id, query)?;
    if backup_inspection.snapshot != snapshot {
        return Err(DatabaseError::Validation(
            "serialized Workspace backup did not match its writer-barrier snapshot".into(),
        ));
    }
    Ok(WorkspaceBackupBarrier {
        generation: snapshot.generation,
        snapshot,
        counts: backup_inspection.counts,
    })
}

fn backup_complete_workspace_snapshot_at_barrier(
    connection: &Connection,
    descriptor: &DatabaseDescriptor,
    destination: &Path,
    include_inactive: bool,
) -> DatabaseResult<WorkspaceBackupBarrier> {
    let expected_workspace_id = workspace_id(&descriptor.kind)?;
    perform_online_backup(connection, descriptor, destination)?;
    let (snapshot, counts) =
        read_complete_workspace_snapshot(connection, expected_workspace_id, include_inactive)?;
    let backup_inspection = inspect_complete_workspace_database_read_only(
        destination,
        expected_workspace_id,
        include_inactive,
    )?;
    if backup_inspection.snapshot != snapshot || backup_inspection.counts != counts {
        return Err(DatabaseError::Validation(
            "complete serialized Workspace backup did not match its counted writer-barrier snapshot"
                .into(),
        ));
    }
    Ok(WorkspaceBackupBarrier {
        generation: snapshot.generation,
        snapshot,
        counts,
    })
}

pub fn migration_backup_path(
    descriptor: &DatabaseDescriptor,
    previous_version: usize,
    target_version: usize,
) -> DatabaseResult<PathBuf> {
    let filename = descriptor
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| DatabaseError::InvalidInput("database filename is not UTF-8".into()))?;
    Ok(descriptor.recovery_dir.join(format!(
        "{filename}.pre-migration-v{previous_version}-to-v{target_version}.sqlite3"
    )))
}

fn create_validated_backup(
    source: &Connection,
    descriptor: &DatabaseDescriptor,
    previous_version: usize,
    target_version: usize,
) -> DatabaseResult<PathBuf> {
    let backup_path = migration_backup_path(descriptor, previous_version, target_version)?;
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut destination = open_backup_connection(&backup_path)?;
    {
        let backup = Backup::new(source, &mut destination)?;
        backup.run_to_completion(64, Duration::from_millis(2), None)?;
    }
    validate_quick_check(&destination)?;
    let backed_up_version = read_user_version(&destination)?;
    if backed_up_version != previous_version {
        return Err(DatabaseError::Validation(format!(
            "pre-migration backup user_version {backed_up_version} did not match source {previous_version}"
        )));
    }
    Ok(backup_path)
}

fn restore_validated_backup(
    destination: &mut Connection,
    backup_path: &Path,
    expected_version: usize,
) -> DatabaseResult<()> {
    let source = open_read_connection(backup_path)?;
    validate_quick_check(&source)?;
    {
        let backup = Backup::new(&source, destination)?;
        backup.run_to_completion(64, Duration::from_millis(2), None)?;
    }
    configure_write_connection(destination)?;
    validate_quick_check(destination)?;
    let restored_version = read_user_version(destination)?;
    if restored_version != expected_version {
        return Err(DatabaseError::Validation(format!(
            "restored database user_version {restored_version} did not match prior version {expected_version}"
        )));
    }
    Ok(())
}

pub fn migration_diagnostic_path(descriptor: &DatabaseDescriptor) -> PathBuf {
    let filename = descriptor
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("database");
    descriptor
        .recovery_dir
        .join(format!("{filename}.migration-failure.txt"))
}

fn write_migration_diagnostic(
    descriptor: &DatabaseDescriptor,
    message: &str,
) -> DatabaseResult<()> {
    fs::create_dir_all(&descriptor.recovery_dir)?;
    let destination = migration_diagnostic_path(descriptor);
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("migration-failure.txt");
    let temporary = descriptor
        .recovery_dir
        .join(format!(".{filename}.tmp-{}", std::process::id()));
    let sanitized = sanitize_internal_diagnostic(message, MAX_DIAGNOSTIC_MESSAGE_BYTES);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    IoWrite::write_all(&mut file, sanitized.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, &destination)?;
    fs::File::open(&descriptor.recovery_dir)?.sync_all()?;
    Ok(())
}

fn validate_database(connection: &Connection, kind: &DatabaseKind) -> DatabaseResult<()> {
    validate_quick_check(connection)?;
    let foreign_key_failure: Option<String> = connection
        .query_row("PRAGMA foreign_key_check", [], |row| row.get(0))
        .optional()?;
    if let Some(table) = foreign_key_failure {
        return Err(DatabaseError::Validation(format!(
            "foreign-key validation failed in table {table}"
        )));
    }

    let required_tables: &[&str] = match kind {
        DatabaseKind::App => &[
            "installations",
            "recent_workspaces",
            "app_diagnostics",
            "app_configuration_lkg",
            "security_records",
            "security_events",
            "runtime_state_documents",
            "durable_generation",
        ],
        DatabaseKind::Workspace { .. } => &[
            "workspaces",
            "projects",
            "chats",
            "configuration_lkg",
            "workspace_diagnostics",
            "session_records",
            "conversation_state",
            "artifact_records",
            "artifact_events",
            "artifact_workspace_state",
            "durable_generation",
        ],
    };
    for table in required_tables {
        let exists: i64 = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?1)",
            [table],
            |row| row.get(0),
        )?;
        if exists != 1 {
            return Err(DatabaseError::Validation(format!(
                "required {kind} table is absent",
                kind = table
            )));
        }
    }
    validate_compiled_schema(connection, kind)?;
    if matches!(kind, DatabaseKind::App) {
        let _ = read_app_configuration_lkg(connection)?;
        validate_security_journal(connection)?;
        validate_runtime_state_documents(connection)?;
    } else if let DatabaseKind::Workspace { workspace_id } = kind {
        validate_session_documents(connection, workspace_id)?;
        let _ = read_conversation_state(connection, workspace_id)?;
        validate_artifact_documents(connection, workspace_id)?;
        let _ = read_artifact_ui_state(connection, workspace_id)?;
    }
    Ok(())
}

fn validate_runtime_state_documents(connection: &Connection) -> DatabaseResult<()> {
    let count: i64 =
        connection.query_row("SELECT COUNT(*) FROM runtime_state_documents", [], |row| {
            row.get(0)
        })?;
    if !(0..=MAX_RUNTIME_STATE_DOCUMENTS as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "runtime document count {count} exceeds {MAX_RUNTIME_STATE_DOCUMENTS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT document_kind, document_id FROM runtime_state_documents
         ORDER BY document_kind, document_id",
    )?;
    let identities = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (kind, id) in identities {
        let _ = read_runtime_state_document(connection, &kind, &id)?;
    }
    Ok(())
}

fn validate_session_documents(connection: &Connection, workspace_id: &str) -> DatabaseResult<()> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM session_records WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if !(0..=MAX_SESSION_RECORDS as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "session record count {count} exceeds {MAX_SESSION_RECORDS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT session_id FROM session_records
         WHERE workspace_id = ?1 ORDER BY session_id",
    )?;
    let ids = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        let _ = read_session_document(connection, workspace_id, &id)?;
    }
    Ok(())
}

fn validate_artifact_documents(connection: &Connection, workspace_id: &str) -> DatabaseResult<()> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM artifact_records WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if !(0..=MAX_ARTIFACT_RECORDS as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "artifact record count {count} exceeds {MAX_ARTIFACT_RECORDS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT artifact_id FROM artifact_records
         WHERE workspace_id = ?1 ORDER BY artifact_id",
    )?;
    let ids = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut current_records = BTreeMap::new();
    for artifact_id in ids {
        let record =
            read_artifact_document(connection, workspace_id, &artifact_id)?.ok_or_else(|| {
                DatabaseError::Validation("artifact record disappeared during validation".into())
            })?;
        current_records.insert(artifact_id, record);
    }

    let event_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM artifact_events WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if !(0..=MAX_ARTIFACT_EVENTS as i64).contains(&event_count) {
        return Err(DatabaseError::Validation(format!(
            "artifact event count {event_count} exceeds {MAX_ARTIFACT_EVENTS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT project_id, session_id, artifact_id, provider_kind,
                provider_version, state_schema_version, revision,
                canonical_document, document_sha256, updated_at_ms
         FROM artifact_events WHERE workspace_id = ?1
         ORDER BY artifact_id, revision",
    )?;
    let events = statement.query_map([workspace_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
        ))
    })?;
    let mut latest_events: BTreeMap<String, WorkspaceArtifactDocumentRecord> = BTreeMap::new();
    for event in events {
        let (
            project_id,
            session_id,
            artifact_id,
            provider_kind,
            provider_version,
            state_schema_version,
            revision,
            canonical_document,
            stored_digest,
            updated_at_ms,
        ) = event?;
        if stored_digest != canonical_document_digest(&canonical_document) {
            return Err(DatabaseError::Validation(format!(
                "artifact event digest mismatch for {artifact_id}"
            )));
        }
        let record = WorkspaceArtifactDocumentRecord {
            workspace_id: workspace_id.into(),
            project_id,
            session_id,
            artifact_id,
            provider_kind,
            provider_version: u16::try_from(provider_version).map_err(|_| {
                DatabaseError::Validation("artifact event provider version is invalid".into())
            })?,
            state_schema_version: u16::try_from(state_schema_version).map_err(|_| {
                DatabaseError::Validation("artifact event schema version is invalid".into())
            })?,
            revision: generation_to_u64(revision)?,
            canonical_document,
            updated_at_ms: generation_to_u64(updated_at_ms)?,
        };
        validate_artifact_document_record(&record, workspace_id)?;
        if let Some(previous) = latest_events.get(&record.artifact_id) {
            let provider_identity_unchanged = record.provider_kind == previous.provider_kind
                && record.provider_version == previous.provider_version;
            let provider_conversion = artifact_provider_conversion_allowed(
                &previous.canonical_document,
                &record.canonical_document,
            )?;
            if record.revision != previous.revision.saturating_add(1)
                || record.project_id != previous.project_id
                || record.session_id != previous.session_id
                || (!provider_identity_unchanged && !provider_conversion)
                || record.state_schema_version != previous.state_schema_version
            {
                return Err(DatabaseError::Validation(format!(
                    "artifact event chain is invalid for {}",
                    record.artifact_id
                )));
            }
        } else if record.revision != 1 {
            return Err(DatabaseError::Validation(format!(
                "artifact event chain does not begin at revision 1 for {}",
                record.artifact_id
            )));
        }
        latest_events.insert(record.artifact_id.clone(), record);
    }
    if current_records != latest_events {
        return Err(DatabaseError::Validation(
            "artifact current records do not match their latest immutable events".into(),
        ));
    }
    Ok(())
}

fn artifact_provider_conversion_allowed(
    previous_document: &str,
    next_document: &str,
) -> DatabaseResult<bool> {
    let previous = serde_json::from_str::<ArtifactRecord>(previous_document).map_err(|_| {
        DatabaseError::Validation("previous artifact conversion record is invalid".into())
    })?;
    let next = serde_json::from_str::<ArtifactRecord>(next_document).map_err(|_| {
        DatabaseError::Validation("next artifact conversion record is invalid".into())
    })?;
    let supported_pair = matches!(
        (
            previous.provider.type_id.as_str(),
            next.provider.type_id.as_str()
        ),
        ("file", "folder") | ("folder", "file")
    ) && previous.provider.schema_version == next.provider.schema_version;
    let explicit_transition = next.history.last().is_some_and(|entry| {
        entry.record_revision == next.record_revision
            && entry.kind == ArtifactHistoryKind::Converted
    });
    Ok(supported_pair && explicit_transition)
}

fn validate_security_journal(connection: &Connection) -> DatabaseResult<()> {
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM security_records", [], |row| {
        row.get(0)
    })?;
    if !(0..=MAX_SECURITY_CURRENT_RECORDS as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "security current-state count {count} exceeds {MAX_SECURITY_CURRENT_RECORDS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
         FROM security_records",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;
    for row in rows {
        let (
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            digest,
            recorded_at_ms,
        ) = row?;
        let record = SecurityJournalRecord {
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            recorded_at_ms: u64::try_from(recorded_at_ms).map_err(|_| {
                DatabaseError::Validation("negative security timestamp in security_records".into())
            })?,
        };
        validate_security_record(&record)?;
        if security_record_digest(&record) != digest {
            return Err(DatabaseError::Validation(format!(
                "security journal digest mismatch in security_records for {}/{}",
                record.record_kind, record.record_id
            )));
        }
    }
    Ok(())
}

fn validate_compiled_schema(connection: &Connection, kind: &DatabaseKind) -> DatabaseResult<()> {
    let actual = load_schema_objects(connection)?;
    let mut trusted_connection = Connection::open_in_memory()?;
    trusted_connection.pragma_update(None, "foreign_keys", "ON")?;
    migrations_for(kind).to_latest(&mut trusted_connection)?;
    let expected = load_schema_objects(&trusted_connection)?;
    if actual != expected {
        let actual_names = actual
            .iter()
            .map(|object| format!("{}:{}", object.object_type, object.name))
            .collect::<Vec<_>>()
            .join(", ");
        let expected_names = expected
            .iter()
            .map(|object| format!("{}:{}", object.object_type, object.name))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(DatabaseError::Validation(format!(
            "database schema does not match the trusted compiled schema; expected [{expected_names}], found [{actual_names}]"
        )));
    }
    Ok(())
}

fn load_schema_objects(connection: &Connection) -> DatabaseResult<Vec<SchemaObject>> {
    let mut statement = connection.prepare(
        "SELECT type, name, tbl_name, COALESCE(sql, '')
         FROM sqlite_schema
         WHERE name NOT LIKE 'sqlite_%'
         ORDER BY type, name, tbl_name",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(SchemaObject {
            object_type: row.get(0)?,
            name: row.get(1)?,
            table_name: row.get(2)?,
            sql: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn validate_quick_check(connection: &Connection) -> DatabaseResult<()> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if result != "ok" {
        return Err(DatabaseError::Validation(format!(
            "SQLite quick_check returned {result}"
        )));
    }
    Ok(())
}

fn read_pragmas(connection: &Connection) -> DatabaseResult<DatabasePragmas> {
    Ok(DatabasePragmas {
        foreign_keys: connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))?
            == 1,
        journal_mode: connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?,
        synchronous: connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?,
        busy_timeout_millis: connection.query_row("PRAGMA busy_timeout", [], |row| row.get(0))?,
    })
}

fn read_runtime_state_document(
    connection: &Connection,
    document_kind: &str,
    document_id: &str,
) -> DatabaseResult<Option<RuntimeStateDocumentRecord>> {
    validate_runtime_document_identity(document_kind, document_id)?;
    let raw = connection
        .query_row(
            "SELECT generation, canonical_document, document_sha256, updated_at_ms
             FROM runtime_state_documents
             WHERE document_kind = ?1 AND document_id = ?2",
            params![document_kind, document_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(generation, canonical_document, stored_digest, updated_at_ms)| {
            validate_text_field(
                "runtime canonical document",
                &canonical_document,
                MAX_RUNTIME_DOCUMENT_BYTES,
            )?;
            if stored_digest != canonical_document_digest(&canonical_document) {
                return Err(DatabaseError::Validation(format!(
                    "runtime document digest mismatch for {document_kind}/{document_id}"
                )));
            }
            Ok(RuntimeStateDocumentRecord {
                document_kind: document_kind.into(),
                document_id: document_id.into(),
                generation: generation_to_u64(generation)?,
                canonical_document,
                updated_at_ms: generation_to_u64(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn write_runtime_state_document(
    connection: &mut Connection,
    record: RuntimeStateDocumentRecord,
    expected_generation: Option<u64>,
) -> DatabaseResult<u64> {
    validate_runtime_document_identity(&record.document_kind, &record.document_id)?;
    validate_text_field(
        "runtime canonical document",
        &record.canonical_document,
        MAX_RUNTIME_DOCUMENT_BYTES,
    )?;
    if record.canonical_document.is_empty() || record.updated_at_ms == 0 {
        return Err(DatabaseError::InvalidInput(
            "runtime document and timestamp must be non-empty".into(),
        ));
    }
    let generation = i64::try_from(record.generation).map_err(|_| {
        DatabaseError::InvalidInput("runtime generation exceeds SQLite range".into())
    })?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("runtime timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current = transaction
        .query_row(
            "SELECT generation FROM runtime_state_documents
             WHERE document_kind = ?1 AND document_id = ?2",
            params![record.document_kind, record.document_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(generation_to_u64)
        .transpose()?;
    match (current, expected_generation) {
        (None, None) => {}
        (Some(current), Some(expected)) if current == expected => {
            if record.generation <= current {
                return Err(DatabaseError::Conflict(format!(
                    "runtime document generation {} does not advance {current}",
                    record.generation
                )));
            }
        }
        (actual, expected) => {
            return Err(DatabaseError::Conflict(format!(
                "runtime document expected generation {expected:?}, found {actual:?}"
            )));
        }
    }
    let digest = canonical_document_digest(&record.canonical_document);
    transaction.execute(
        "INSERT INTO runtime_state_documents(
            document_kind, document_id, generation, canonical_document,
            document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(document_kind, document_id) DO UPDATE SET
            generation = excluded.generation,
            canonical_document = excluded.canonical_document,
            document_sha256 = excluded.document_sha256,
            updated_at_ms = excluded.updated_at_ms",
        params![
            record.document_kind,
            record.document_id,
            generation,
            record.canonical_document,
            digest,
            updated_at_ms
        ],
    )?;
    let durable_generation = bump_app_generation(&transaction)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn read_conversation_state(
    connection: &Connection,
    workspace_id: &str,
) -> DatabaseResult<Option<WorkspaceConversationStateRecord>> {
    require_nonempty("workspace_id", workspace_id)?;
    let raw = connection
        .query_row(
            "SELECT generation, canonical_document, document_sha256, updated_at_ms
             FROM conversation_state WHERE workspace_id = ?1",
            [workspace_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(generation, canonical_document, stored_digest, updated_at_ms)| {
            validate_text_field(
                "conversation canonical document",
                &canonical_document,
                MAX_RUNTIME_DOCUMENT_BYTES,
            )?;
            if stored_digest != canonical_document_digest(&canonical_document) {
                return Err(DatabaseError::Validation(format!(
                    "conversation document digest mismatch for {workspace_id}"
                )));
            }
            Ok(WorkspaceConversationStateRecord {
                workspace_id: workspace_id.into(),
                generation: generation_to_u64(generation)?,
                canonical_document,
                updated_at_ms: generation_to_u64(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn write_conversation_state(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: WorkspaceConversationStateRecord,
    expected_generation: Option<u64>,
) -> DatabaseResult<u64> {
    let DatabaseKind::Workspace { workspace_id } = kind else {
        return Err(DatabaseError::WrongKind {
            expected: "workspace",
            actual: "app",
        });
    };
    if record.workspace_id != *workspace_id {
        return Err(DatabaseError::InvalidInput(
            "conversation state Workspace identity is mismatched".into(),
        ));
    }
    validate_text_field(
        "conversation canonical document",
        &record.canonical_document,
        MAX_RUNTIME_DOCUMENT_BYTES,
    )?;
    if record.generation == 0 || record.canonical_document.is_empty() || record.updated_at_ms == 0 {
        return Err(DatabaseError::InvalidInput(
            "conversation generation, document, and timestamp must be present".into(),
        ));
    }
    let generation = i64::try_from(record.generation).map_err(|_| {
        DatabaseError::InvalidInput("conversation generation exceeds SQLite range".into())
    })?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("conversation timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_workspace: i64 = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM workspaces
         WHERE workspace_id = ?1 AND lifecycle_state = 'active')",
        [workspace_id],
        |row| row.get(0),
    )?;
    if active_workspace != 1 {
        return Err(DatabaseError::Conflict(
            "active Workspace changed before conversation state commit".into(),
        ));
    }
    let current = transaction
        .query_row(
            "SELECT generation FROM conversation_state WHERE workspace_id = ?1",
            [workspace_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(generation_to_u64)
        .transpose()?;
    match (current, expected_generation) {
        (None, None) => {}
        (Some(current), Some(expected)) if current == expected && record.generation > current => {}
        (actual, expected) => {
            return Err(DatabaseError::Conflict(format!(
                "conversation state expected generation {expected:?}, found {actual:?}"
            )));
        }
    }
    let digest = canonical_document_digest(&record.canonical_document);
    transaction.execute(
        "INSERT INTO conversation_state(
            workspace_id, generation, canonical_document, document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(workspace_id) DO UPDATE SET
            generation = excluded.generation,
            canonical_document = excluded.canonical_document,
            document_sha256 = excluded.document_sha256,
            updated_at_ms = excluded.updated_at_ms",
        params![
            workspace_id,
            generation,
            record.canonical_document,
            digest,
            updated_at_ms
        ],
    )?;
    let durable_generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn validate_artifact_document_record(
    record: &WorkspaceArtifactDocumentRecord,
    workspace_id: &str,
) -> DatabaseResult<()> {
    if record.workspace_id != workspace_id {
        return Err(DatabaseError::InvalidInput(
            "artifact document Workspace identity is mismatched".into(),
        ));
    }
    for (label, value) in [
        ("project_id", record.project_id.as_str()),
        ("session_id", record.session_id.as_str()),
        ("artifact_id", record.artifact_id.as_str()),
    ] {
        require_nonempty(label, value)?;
        validate_text_field(label, value, 512)?;
    }
    if record.provider_version == 0
        || record.state_schema_version == 0
        || record.revision == 0
        || record.updated_at_ms == 0
    {
        return Err(DatabaseError::InvalidInput(
            "artifact provider, schema, revision, and timestamp must be valid".into(),
        ));
    }
    validate_text_field(
        "artifact canonical document",
        &record.canonical_document,
        MAX_ARTIFACT_DOCUMENT_BYTES,
    )?;
    if record.canonical_document.is_empty() {
        return Err(DatabaseError::InvalidInput(
            "artifact canonical document must be present".into(),
        ));
    }
    let artifact =
        serde_json::from_str::<ArtifactRecord>(&record.canonical_document).map_err(|_| {
            DatabaseError::Validation(
                "artifact canonical document is not a supported record".into(),
            )
        })?;
    artifact
        .validate()
        .map_err(|_| DatabaseError::Validation("artifact canonical document is invalid".into()))?;
    if artifact.workspace_id != record.workspace_id
        || artifact.project_id != record.project_id
        || artifact.session_id != record.session_id
        || artifact.artifact_id != record.artifact_id
        || artifact.provider.type_id != record.provider_kind
        || artifact.provider.schema_version != record.provider_version
        || artifact.schema_version != record.state_schema_version
        || artifact.record_revision != record.revision
        || artifact.updated_at_ms != record.updated_at_ms
    {
        return Err(DatabaseError::Validation(
            "artifact canonical document disagrees with its persistence envelope".into(),
        ));
    }
    Ok(())
}

fn read_artifact_document(
    connection: &Connection,
    workspace_id: &str,
    artifact_id: &str,
) -> DatabaseResult<Option<WorkspaceArtifactDocumentRecord>> {
    require_nonempty("artifact_id", artifact_id)?;
    validate_text_field("artifact_id", artifact_id, 512)?;
    let raw = connection
        .query_row(
            "SELECT project_id, session_id, provider_kind, provider_version,
                state_schema_version, revision, canonical_document,
                document_sha256, updated_at_ms
             FROM artifact_records
             WHERE workspace_id = ?1 AND artifact_id = ?2",
            params![workspace_id, artifact_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(
            project_id,
            session_id,
            provider_kind,
            provider_version,
            state_schema_version,
            revision,
            canonical_document,
            stored_digest,
            updated_at_ms,
        )| {
            validate_text_field(
                "artifact canonical document",
                &canonical_document,
                MAX_ARTIFACT_DOCUMENT_BYTES,
            )?;
            if stored_digest != canonical_document_digest(&canonical_document) {
                return Err(DatabaseError::Validation(format!(
                    "artifact document digest mismatch for {artifact_id}"
                )));
            }
            let record = WorkspaceArtifactDocumentRecord {
                workspace_id: workspace_id.into(),
                project_id,
                session_id,
                artifact_id: artifact_id.into(),
                provider_kind,
                provider_version: u16::try_from(provider_version).map_err(|_| {
                    DatabaseError::Validation("artifact provider version is invalid".into())
                })?,
                state_schema_version: u16::try_from(state_schema_version).map_err(|_| {
                    DatabaseError::Validation("artifact state schema version is invalid".into())
                })?,
                revision: generation_to_u64(revision)?,
                canonical_document,
                updated_at_ms: generation_to_u64(updated_at_ms)?,
            };
            validate_artifact_document_record(&record, workspace_id)?;
            Ok(record)
        },
    )
    .transpose()
}

fn read_artifact_documents_for_session(
    connection: &Connection,
    workspace_id: &str,
    session_id: &str,
) -> DatabaseResult<Vec<WorkspaceArtifactDocumentRecord>> {
    require_nonempty("session_id", session_id)?;
    validate_text_field("session_id", session_id, 512)?;
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM artifact_records
         WHERE workspace_id = ?1 AND session_id = ?2",
        params![workspace_id, session_id],
        |row| row.get(0),
    )?;
    if !(0..=MAX_ARTIFACT_RECORDS_PER_SESSION as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "session artifact record count {count} exceeds {MAX_ARTIFACT_RECORDS_PER_SESSION}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT artifact_id FROM artifact_records
         WHERE workspace_id = ?1 AND session_id = ?2
         ORDER BY updated_at_ms, artifact_id",
    )?;
    let ids = statement
        .query_map(params![workspace_id, session_id], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|artifact_id| {
            read_artifact_document(connection, workspace_id, &artifact_id)?.ok_or_else(|| {
                DatabaseError::Validation(format!(
                    "artifact document {artifact_id} disappeared during snapshot"
                ))
            })
        })
        .collect()
}

fn write_artifact_document(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: WorkspaceArtifactDocumentRecord,
    expected_revision: Option<u64>,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    validate_artifact_document_record(&record, workspace_id)?;
    let provider_version = i64::from(record.provider_version);
    let state_schema_version = i64::from(record.state_schema_version);
    let revision = i64::try_from(record.revision).map_err(|_| {
        DatabaseError::InvalidInput("artifact revision exceeds SQLite range".into())
    })?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("artifact timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_owner: i64 = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM workspaces AS workspace
            JOIN projects AS project ON project.workspace_id = workspace.workspace_id
            JOIN chats AS chat ON chat.workspace_id = workspace.workspace_id
            WHERE workspace.workspace_id = ?1
              AND project.project_id = ?2
              AND chat.chat_id = ?3
              AND chat.project_id = project.project_id
              AND workspace.lifecycle_state = 'active'
              AND project.lifecycle_state = 'active'
              AND chat.lifecycle_state = 'active'
        )",
        params![workspace_id, record.project_id, record.session_id],
        |row| row.get(0),
    )?;
    if active_owner != 1 {
        return Err(DatabaseError::Conflict(
            "active artifact Workspace, Project, or Chat owner changed".into(),
        ));
    }
    let current = transaction
        .query_row(
            "SELECT project_id, session_id, provider_kind, provider_version,
                state_schema_version, revision, canonical_document
             FROM artifact_records
             WHERE workspace_id = ?1 AND artifact_id = ?2",
            params![workspace_id, record.artifact_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .optional()?;
    let event_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM artifact_events WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if event_count < 0 || event_count as usize >= MAX_ARTIFACT_EVENTS {
        return Err(DatabaseError::Conflict(
            "artifact event capacity is exhausted".into(),
        ));
    }
    if current.is_none() {
        let record_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM artifact_records WHERE workspace_id = ?1",
            [workspace_id],
            |row| row.get(0),
        )?;
        let session_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM artifact_records
             WHERE workspace_id = ?1 AND session_id = ?2",
            params![workspace_id, record.session_id],
            |row| row.get(0),
        )?;
        if record_count < 0 || record_count as usize >= MAX_ARTIFACT_RECORDS {
            return Err(DatabaseError::Conflict(
                "artifact record capacity is exhausted".into(),
            ));
        }
        if session_count < 0 || session_count as usize >= MAX_ARTIFACT_RECORDS_PER_SESSION {
            return Err(DatabaseError::Conflict(
                "session artifact capacity is exhausted".into(),
            ));
        }
    }
    let provider_conversion = current
        .as_ref()
        .map(|(_, _, _, _, _, _, current_document)| {
            artifact_provider_conversion_allowed(current_document, &record.canonical_document)
        })
        .transpose()?
        .unwrap_or(false);
    match (&current, expected_revision) {
        (None, None) if record.revision == 1 => {}
        (
            Some((
                project_id,
                session_id,
                provider_kind,
                stored_provider_version,
                stored_schema,
                current_revision,
                _,
            )),
            Some(expected),
        ) if generation_to_u64(*current_revision)? == expected
            && record.revision == expected.saturating_add(1)
            && project_id == &record.project_id
            && session_id == &record.session_id
            && ((provider_kind == &record.provider_kind
                && *stored_provider_version == provider_version)
                || provider_conversion)
            && *stored_schema == state_schema_version => {}
        (actual, expected) => {
            let actual_revision = actual
                .as_ref()
                .map(|(_, _, _, _, _, revision, _)| generation_to_u64(*revision))
                .transpose()?;
            return Err(DatabaseError::Conflict(format!(
                "artifact document expected revision {expected:?}, found {actual_revision:?}"
            )));
        }
    }
    let digest = canonical_document_digest(&record.canonical_document);
    transaction.execute(
        "INSERT INTO artifact_records(
            workspace_id, project_id, session_id, artifact_id, provider_kind,
            provider_version, state_schema_version, revision,
            canonical_document, document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(workspace_id, artifact_id) DO UPDATE SET
            provider_kind = excluded.provider_kind,
            provider_version = excluded.provider_version,
            state_schema_version = excluded.state_schema_version,
            revision = excluded.revision,
            canonical_document = excluded.canonical_document,
            document_sha256 = excluded.document_sha256,
            updated_at_ms = excluded.updated_at_ms",
        params![
            workspace_id,
            record.project_id,
            record.session_id,
            record.artifact_id,
            record.provider_kind,
            provider_version,
            state_schema_version,
            revision,
            record.canonical_document,
            digest,
            updated_at_ms,
        ],
    )?;
    transaction.execute(
        "INSERT INTO artifact_events(
            workspace_id, project_id, session_id, artifact_id, provider_kind,
            provider_version, state_schema_version, revision,
            canonical_document, document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            workspace_id,
            record.project_id,
            record.session_id,
            record.artifact_id,
            record.provider_kind,
            provider_version,
            state_schema_version,
            revision,
            record.canonical_document,
            digest,
            updated_at_ms,
        ],
    )?;
    let durable_generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn validate_artifact_ui_state_record(
    record: &WorkspaceArtifactUiStateRecord,
    workspace_id: &str,
) -> DatabaseResult<()> {
    if record.workspace_id != workspace_id {
        return Err(DatabaseError::InvalidInput(
            "artifact UI state Workspace identity is mismatched".into(),
        ));
    }
    if record.revision == 0 || record.updated_at_ms == 0 || record.canonical_document.is_empty() {
        return Err(DatabaseError::InvalidInput(
            "artifact UI state revision, timestamp, and document must be present".into(),
        ));
    }
    validate_text_field(
        "artifact UI canonical document",
        &record.canonical_document,
        MAX_ARTIFACT_UI_DOCUMENT_BYTES,
    )?;
    let state = serde_json::from_str::<ArtifactWorkspaceUiState>(&record.canonical_document)
        .map_err(|_| {
            DatabaseError::Validation(
                "artifact UI canonical document is not a supported record".into(),
            )
        })?;
    state
        .validate()
        .map_err(|_| DatabaseError::Validation("artifact UI state is invalid".into()))?;
    if state.workspace_id != record.workspace_id
        || state.revision != record.revision
        || state.updated_at_ms != record.updated_at_ms
    {
        return Err(DatabaseError::Validation(
            "artifact UI document disagrees with its persistence envelope".into(),
        ));
    }
    Ok(())
}

fn read_artifact_ui_state(
    connection: &Connection,
    workspace_id: &str,
) -> DatabaseResult<Option<WorkspaceArtifactUiStateRecord>> {
    let raw = connection
        .query_row(
            "SELECT revision, canonical_document, document_sha256, updated_at_ms
             FROM artifact_workspace_state WHERE workspace_id = ?1",
            [workspace_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(revision, canonical_document, stored_digest, updated_at_ms)| {
            if stored_digest != canonical_document_digest(&canonical_document) {
                return Err(DatabaseError::Validation(
                    "artifact UI state digest mismatch".into(),
                ));
            }
            let record = WorkspaceArtifactUiStateRecord {
                workspace_id: workspace_id.into(),
                revision: generation_to_u64(revision)?,
                canonical_document,
                updated_at_ms: generation_to_u64(updated_at_ms)?,
            };
            validate_artifact_ui_state_record(&record, workspace_id)?;
            let state =
                serde_json::from_str::<ArtifactWorkspaceUiState>(&record.canonical_document)
                    .map_err(|_| {
                        DatabaseError::Validation("artifact UI state is invalid".into())
                    })?;
            if let Some(artifact_id) = &state.focused_artifact_id {
                let artifact_record = read_artifact_document(
                    connection,
                    workspace_id,
                    artifact_id,
                )?
                .ok_or_else(|| {
                    DatabaseError::Validation("focused artifact UI identity is unavailable".into())
                })?;
                let artifact =
                    serde_json::from_str::<ArtifactRecord>(&artifact_record.canonical_document)
                        .map_err(|_| {
                            DatabaseError::Validation("focused artifact is invalid".into())
                        })?;
                state.validate_against(&artifact).map_err(|_| {
                    DatabaseError::Validation(
                        "focused artifact UI identity is not focus-capable".into(),
                    )
                })?;
            }
            Ok(record)
        },
    )
    .transpose()
}

fn write_artifact_ui_state(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: WorkspaceArtifactUiStateRecord,
    expected_revision: Option<u64>,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    validate_artifact_ui_state_record(&record, workspace_id)?;
    let revision = i64::try_from(record.revision).map_err(|_| {
        DatabaseError::InvalidInput("artifact UI state revision exceeds SQLite range".into())
    })?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("artifact UI state timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_workspace: i64 = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM workspaces
            WHERE workspace_id = ?1 AND lifecycle_state = 'active'
        )",
        [workspace_id],
        |row| row.get(0),
    )?;
    if active_workspace != 1 {
        return Err(DatabaseError::Conflict(
            "active artifact Workspace changed".into(),
        ));
    }
    let ui_state = serde_json::from_str::<ArtifactWorkspaceUiState>(&record.canonical_document)
        .map_err(|_| DatabaseError::Validation("artifact UI state is invalid".into()))?;
    if let Some(artifact_id) = &ui_state.focused_artifact_id {
        let canonical_artifact = transaction
            .query_row(
                "SELECT canonical_document FROM artifact_records
                 WHERE workspace_id = ?1 AND artifact_id = ?2",
                params![workspace_id, artifact_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or_else(|| DatabaseError::Conflict("focused artifact is unavailable".into()))?;
        let artifact = serde_json::from_str::<ArtifactRecord>(&canonical_artifact)
            .map_err(|_| DatabaseError::Validation("focused artifact is invalid".into()))?;
        ui_state
            .validate_against(&artifact)
            .map_err(|_| DatabaseError::Conflict("artifact cannot receive focus".into()))?;
    }
    let current = transaction
        .query_row(
            "SELECT revision FROM artifact_workspace_state WHERE workspace_id = ?1",
            [workspace_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(generation_to_u64)
        .transpose()?;
    match (current, expected_revision) {
        (None, None) if record.revision == 1 => {}
        (Some(current), Some(expected))
            if current == expected && record.revision == expected.saturating_add(1) => {}
        (actual, expected) => {
            return Err(DatabaseError::Conflict(format!(
                "artifact UI state expected revision {expected:?}, found {actual:?}"
            )));
        }
    }
    let digest = canonical_document_digest(&record.canonical_document);
    transaction.execute(
        "INSERT INTO artifact_workspace_state(
            workspace_id, revision, canonical_document, document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(workspace_id) DO UPDATE SET
            revision = excluded.revision,
            canonical_document = excluded.canonical_document,
            document_sha256 = excluded.document_sha256,
            updated_at_ms = excluded.updated_at_ms",
        params![
            workspace_id,
            revision,
            record.canonical_document,
            digest,
            updated_at_ms,
        ],
    )?;
    let durable_generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn read_session_document(
    connection: &Connection,
    workspace_id: &str,
    session_id: &str,
) -> DatabaseResult<Option<WorkspaceSessionDocumentRecord>> {
    require_nonempty("session_id", session_id)?;
    let raw = connection
        .query_row(
            "SELECT revision, canonical_document, document_sha256, updated_at_ms
             FROM session_records WHERE workspace_id = ?1 AND session_id = ?2",
            params![workspace_id, session_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(revision, canonical_document, stored_digest, updated_at_ms)| {
            validate_text_field(
                "session canonical document",
                &canonical_document,
                MAX_SESSION_DOCUMENT_BYTES,
            )?;
            if stored_digest != canonical_document_digest(&canonical_document) {
                return Err(DatabaseError::Validation(format!(
                    "session document digest mismatch for {session_id}"
                )));
            }
            Ok(WorkspaceSessionDocumentRecord {
                workspace_id: workspace_id.into(),
                session_id: session_id.into(),
                revision: generation_to_u64(revision)?,
                canonical_document,
                updated_at_ms: generation_to_u64(updated_at_ms)?,
            })
        },
    )
    .transpose()
}

fn read_session_documents(
    connection: &Connection,
    workspace_id: &str,
) -> DatabaseResult<Vec<WorkspaceSessionDocumentRecord>> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM session_records WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if !(0..=MAX_SESSION_RECORDS as i64).contains(&count) {
        return Err(DatabaseError::Validation(format!(
            "session record count {count} exceeds {MAX_SESSION_RECORDS}"
        )));
    }
    let mut statement = connection.prepare(
        "SELECT session_id FROM session_records
         WHERE workspace_id = ?1 ORDER BY updated_at_ms, session_id",
    )?;
    let ids = statement
        .query_map([workspace_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|session_id| {
            read_session_document(connection, workspace_id, &session_id)?.ok_or_else(|| {
                DatabaseError::Validation(format!(
                    "session document {session_id} disappeared during snapshot"
                ))
            })
        })
        .collect()
}

fn write_promoted_session_document(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: WorkspaceSessionDocumentRecord,
    chat: ChatRecord,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    if record.workspace_id != workspace_id
        || chat.workspace_id != workspace_id
        || chat.chat_id != record.session_id
    {
        return Err(DatabaseError::InvalidInput(
            "promoted Chat and session identities do not match this Workspace".into(),
        ));
    }
    require_nonempty("session_id", &record.session_id)?;
    require_nonempty("project_id", &chat.project_id)?;
    validate_canonical_display_name("Chat title", &chat.title, MAX_PROJECT_DISPLAY_NAME_BYTES)?;
    validate_inactivation(chat.lifecycle_state, chat.inactivated_at)?;
    if chat.lifecycle_state != LifecycleState::Active
        || chat.created_at <= 0
        || chat.updated_at < chat.created_at
    {
        return Err(DatabaseError::InvalidInput(
            "promoted Chat must be active with valid timestamps".into(),
        ));
    }
    validate_text_field(
        "session canonical document",
        &record.canonical_document,
        MAX_SESSION_DOCUMENT_BYTES,
    )?;
    if record.revision != 1 || record.canonical_document.is_empty() || record.updated_at_ms == 0 {
        return Err(DatabaseError::InvalidInput(
            "first session revision, document, and timestamp must be present".into(),
        ));
    }
    let revision = i64::try_from(record.revision)
        .map_err(|_| DatabaseError::InvalidInput("session revision exceeds SQLite range".into()))?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("session timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_project_exists: i64 = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM workspaces AS workspace
            JOIN projects AS project ON project.workspace_id = workspace.workspace_id
            WHERE workspace.workspace_id = ?1
              AND project.project_id = ?2
              AND workspace.lifecycle_state = 'active'
              AND project.lifecycle_state = 'active'
        )",
        params![workspace_id, chat.project_id],
        |row| row.get(0),
    )?;
    if active_project_exists != 1 {
        return Err(DatabaseError::Conflict(
            "active Workspace and Project binding changed before Chat promotion".into(),
        ));
    }
    let existing_chat = transaction
        .query_row(
            "SELECT project_id, lifecycle_state FROM chats
             WHERE workspace_id = ?1 AND chat_id = ?2",
            params![workspace_id, record.session_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    if existing_chat
        .as_ref()
        .is_some_and(|(project_id, lifecycle)| {
            project_id != &chat.project_id || lifecycle != "active"
        })
    {
        return Err(DatabaseError::Conflict(
            "Chat identity is inactive or belongs to another Project".into(),
        ));
    }
    let existing_session: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM session_records
         WHERE workspace_id = ?1 AND session_id = ?2",
        params![workspace_id, record.session_id],
        |row| row.get(0),
    )?;
    if existing_session != 0 {
        return Err(DatabaseError::Conflict(
            "session record already exists during first promotion".into(),
        ));
    }
    insert_chat(&transaction, &chat)?;
    let digest = canonical_document_digest(&record.canonical_document);
    let written = transaction.execute(
        "INSERT INTO session_records(
            workspace_id, session_id, revision, canonical_document,
            document_sha256, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            workspace_id,
            record.session_id,
            revision,
            record.canonical_document,
            digest,
            updated_at_ms
        ],
    )?;
    if written != 1 {
        return Err(DatabaseError::Conflict(
            "session promotion lost a concurrent race".into(),
        ));
    }
    let durable_generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn write_session_document(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: WorkspaceSessionDocumentRecord,
    active_project_id: &str,
    expected_revision: Option<u64>,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    if record.workspace_id != workspace_id {
        return Err(DatabaseError::InvalidInput(
            "session document belongs to another Workspace".into(),
        ));
    }
    require_nonempty("session_id", &record.session_id)?;
    require_nonempty("active_project_id", active_project_id)?;
    validate_text_field(
        "session canonical document",
        &record.canonical_document,
        MAX_SESSION_DOCUMENT_BYTES,
    )?;
    if record.revision == 0 || record.canonical_document.is_empty() || record.updated_at_ms == 0 {
        return Err(DatabaseError::InvalidInput(
            "session revision, document, and timestamp must be present".into(),
        ));
    }
    let revision = i64::try_from(record.revision)
        .map_err(|_| DatabaseError::InvalidInput("session revision exceeds SQLite range".into()))?;
    let updated_at_ms = i64::try_from(record.updated_at_ms).map_err(|_| {
        DatabaseError::InvalidInput("session timestamp exceeds SQLite range".into())
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_binding_exists: i64 = transaction.query_row(
        "SELECT EXISTS(
            SELECT 1
            FROM workspaces AS workspace
            JOIN projects AS project
              ON project.workspace_id = workspace.workspace_id
            JOIN chats AS chat
              ON chat.workspace_id = project.workspace_id
             AND chat.project_id = project.project_id
            WHERE workspace.workspace_id = ?1
              AND project.project_id = ?2
              AND chat.chat_id = ?3
              AND workspace.lifecycle_state = 'active'
              AND project.lifecycle_state = 'active'
              AND chat.lifecycle_state = 'active'
        )",
        params![workspace_id, active_project_id, record.session_id],
        |row| row.get(0),
    )?;
    if active_binding_exists != 1 {
        return Err(DatabaseError::Conflict(
            "active Workspace, Project, and Chat binding changed before session commit".into(),
        ));
    }
    let current = transaction
        .query_row(
            "SELECT revision FROM session_records
             WHERE workspace_id = ?1 AND session_id = ?2",
            params![workspace_id, record.session_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(generation_to_u64)
        .transpose()?;
    match (current, expected_revision) {
        (None, None) if record.revision == 1 => {}
        (Some(current), Some(expected))
            if current == expected
                && expected
                    .checked_add(1)
                    .is_some_and(|next| record.revision == next) => {}
        (actual, expected) => {
            return Err(DatabaseError::Conflict(format!(
                "session expected revision {expected:?}, found {actual:?}, replacement {}",
                record.revision
            )));
        }
    }
    let digest = canonical_document_digest(&record.canonical_document);
    let written = match expected_revision {
        None => transaction.execute(
            "INSERT INTO session_records(
                workspace_id, session_id, revision, canonical_document,
                document_sha256, updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                workspace_id,
                record.session_id,
                revision,
                record.canonical_document,
                digest,
                updated_at_ms
            ],
        )?,
        Some(expected) => transaction.execute(
            "UPDATE session_records SET
                revision = ?1, canonical_document = ?2,
                document_sha256 = ?3, updated_at_ms = ?4
             WHERE workspace_id = ?5 AND session_id = ?6 AND revision = ?7",
            params![
                revision,
                record.canonical_document,
                digest,
                updated_at_ms,
                workspace_id,
                record.session_id,
                i64::try_from(expected).map_err(|_| DatabaseError::Conflict(
                    "expected session revision exceeds SQLite range".into()
                ))?
            ],
        )?,
    };
    if written != 1 {
        return Err(DatabaseError::Conflict(
            "session compare-and-swap lost a concurrent race".into(),
        ));
    }
    let durable_generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(durable_generation)
}

fn validate_runtime_document_identity(
    document_kind: &str,
    document_id: &str,
) -> DatabaseResult<()> {
    if !matches!(
        document_kind,
        "provider-snapshot" | "supervisor-snapshot" | "runtime-control-plane"
    ) {
        return Err(DatabaseError::InvalidInput(
            "runtime document kind is unsupported".into(),
        ));
    }
    require_nonempty("runtime document id", document_id)?;
    if document_id.len() > 160
        || !document_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
    {
        return Err(DatabaseError::InvalidInput(
            "runtime document id is invalid".into(),
        ));
    }
    Ok(())
}

fn write_installation(
    connection: &mut Connection,
    record: InstallationRecord,
) -> DatabaseResult<u64> {
    require_nonempty("installation_id", &record.installation_id)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO installations(installation_id, created_at, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(installation_id) DO UPDATE SET updated_at = excluded.updated_at",
        params![record.installation_id, record.created_at, record.updated_at],
    )?;
    let generation = bump_app_generation(&transaction)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_recent(connection: &mut Connection, record: RecentWorkspaceRecord) -> DatabaseResult<u64> {
    require_nonempty("workspace_id", &record.workspace_id)?;
    require_nonempty("archive_path", &record.archive_path)?;
    validate_canonical_display_name(
        "Workspace display name",
        &record.display_name,
        MAX_WORKSPACE_DISPLAY_NAME_BYTES,
    )?;
    validate_inactivation(record.lifecycle_state, record.inactivated_at)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO recent_workspaces(
            workspace_id, display_name, archive_path, last_opened_at,
            lifecycle_state, inactivated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(workspace_id) DO UPDATE SET
            display_name = excluded.display_name,
            archive_path = excluded.archive_path,
            last_opened_at = excluded.last_opened_at,
            lifecycle_state = excluded.lifecycle_state,
            inactivated_at = excluded.inactivated_at",
        params![
            record.workspace_id,
            record.display_name,
            record.archive_path,
            record.last_opened_at,
            record.lifecycle_state.as_str(),
            record.inactivated_at
        ],
    )?;
    let generation = bump_app_generation(&transaction)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_app_configuration(
    connection: &mut Connection,
    record: AppConfigurationSnapshotRecord,
) -> DatabaseResult<u64> {
    validate_text_field(
        "app configuration canonical document",
        &record.canonical_document,
        MAX_TEXT_FIELD_BYTES,
    )?;
    if record.generation == 0 {
        return Err(DatabaseError::InvalidInput(
            "app configuration generation must be greater than zero".into(),
        ));
    }
    let configuration_generation = i64::try_from(record.generation).map_err(|_| {
        DatabaseError::InvalidInput(format!(
            "app configuration generation {} exceeds SQLite's signed integer range",
            record.generation
        ))
    })?;
    let document_sha256 = canonical_document_digest(&record.canonical_document);
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current_generation = transaction
        .query_row(
            "SELECT generation FROM app_configuration_lkg WHERE singleton = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    if current_generation.is_some_and(|current| configuration_generation <= current) {
        return Err(DatabaseError::InvalidInput(format!(
            "app configuration generation {} is not newer than the active generation {}",
            record.generation,
            current_generation.unwrap_or_default()
        )));
    }
    transaction.execute(
        "INSERT INTO app_configuration_lkg(
            singleton, canonical_document, generation, activated_at, document_sha256
         ) VALUES (1, ?1, ?2, ?3, ?4)
         ON CONFLICT(singleton) DO UPDATE SET
            canonical_document = excluded.canonical_document,
            generation = excluded.generation,
            activated_at = excluded.activated_at,
            document_sha256 = excluded.document_sha256",
        params![
            record.canonical_document,
            configuration_generation,
            record.activated_at,
            document_sha256
        ],
    )?;
    let generation = bump_app_generation(&transaction)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_workspace(connection: &mut Connection, record: WorkspaceRecord) -> DatabaseResult<u64> {
    require_nonempty("workspace_id", &record.workspace_id)?;
    validate_canonical_display_name(
        "Workspace display name",
        &record.display_name,
        MAX_WORKSPACE_DISPLAY_NAME_BYTES,
    )?;
    validate_inactivation(record.lifecycle_state, record.inactivated_at)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO workspaces(
            workspace_id, display_name, created_at, updated_at,
            lifecycle_state, inactivated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(workspace_id) DO UPDATE SET
            display_name = excluded.display_name,
            updated_at = excluded.updated_at,
            lifecycle_state = excluded.lifecycle_state,
            inactivated_at = excluded.inactivated_at",
        params![
            record.workspace_id,
            record.display_name,
            record.created_at,
            record.updated_at,
            record.lifecycle_state.as_str(),
            record.inactivated_at
        ],
    )?;
    transaction.execute(
        "INSERT INTO durable_generation(workspace_id, generation) VALUES (?1, 0)
         ON CONFLICT(workspace_id) DO NOTHING",
        [&record.workspace_id],
    )?;
    let generation = bump_workspace_generation(&transaction, &record.workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn insert_project(
    transaction: &rusqlite::Transaction<'_>,
    record: &ProjectRecord,
) -> DatabaseResult<()> {
    require_nonempty("project_id", &record.project_id)?;
    validate_canonical_display_name(
        "Project display name",
        &record.display_name,
        MAX_PROJECT_DISPLAY_NAME_BYTES,
    )?;
    require_nonempty("current_path", &record.current_path)?;
    require_nonempty("last_known_path", &record.last_known_path)?;
    validate_inactivation(record.lifecycle_state, record.inactivated_at)?;
    transaction.execute(
        "INSERT INTO projects(
            workspace_id, project_id, display_name, current_path, last_known_path,
            path_state, position, lifecycle_state, inactivated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(workspace_id, project_id) DO UPDATE SET
            display_name = excluded.display_name,
            current_path = excluded.current_path,
            last_known_path = excluded.last_known_path,
            path_state = excluded.path_state,
            position = excluded.position,
            lifecycle_state = excluded.lifecycle_state,
            inactivated_at = excluded.inactivated_at",
        params![
            record.workspace_id,
            record.project_id,
            record.display_name,
            record.current_path,
            record.last_known_path,
            record.path_state.as_str(),
            record.position,
            record.lifecycle_state.as_str(),
            record.inactivated_at
        ],
    )?;
    Ok(())
}

fn write_project(connection: &mut Connection, record: ProjectRecord) -> DatabaseResult<u64> {
    let workspace_id = record.workspace_id.clone();
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    insert_project(&transaction, &record)?;
    let generation = bump_workspace_generation(&transaction, &workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn insert_chat(transaction: &rusqlite::Transaction<'_>, record: &ChatRecord) -> DatabaseResult<()> {
    require_nonempty("chat_id", &record.chat_id)?;
    require_nonempty("project_id", &record.project_id)?;
    validate_inactivation(record.lifecycle_state, record.inactivated_at)?;
    transaction.execute(
        "INSERT INTO chats(
            workspace_id, project_id, chat_id, title, created_at, updated_at,
            lifecycle_state, inactivated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(workspace_id, chat_id) DO UPDATE SET
            project_id = excluded.project_id,
            title = excluded.title,
            updated_at = excluded.updated_at,
            lifecycle_state = excluded.lifecycle_state,
            inactivated_at = excluded.inactivated_at",
        params![
            record.workspace_id,
            record.project_id,
            record.chat_id,
            record.title,
            record.created_at,
            record.updated_at,
            record.lifecycle_state.as_str(),
            record.inactivated_at
        ],
    )?;
    Ok(())
}

fn write_chat(connection: &mut Connection, record: ChatRecord) -> DatabaseResult<u64> {
    let workspace_id = record.workspace_id.clone();
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    insert_chat(&transaction, &record)?;
    let generation = bump_workspace_generation(&transaction, &workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_project_and_chat(
    connection: &mut Connection,
    project: ProjectRecord,
    chat: ChatRecord,
) -> DatabaseResult<u64> {
    let workspace_id = project.workspace_id.clone();
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    insert_project(&transaction, &project)?;
    insert_chat(&transaction, &chat)?;
    let generation = bump_workspace_generation(&transaction, &workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_project_order(
    connection: &mut Connection,
    kind: &DatabaseKind,
    ordered_project_ids: Vec<String>,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let active_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM projects WHERE workspace_id = ?1 AND lifecycle_state = 'active'",
        [workspace_id],
        |row| row.get(0),
    )?;
    if active_count != i64::try_from(ordered_project_ids.len()).unwrap_or(i64::MAX) {
        return Err(DatabaseError::InvalidInput(
            "project order must include every active Project exactly once".into(),
        ));
    }
    transaction.execute(
        "UPDATE projects SET position = position + 1000000
         WHERE workspace_id = ?1 AND lifecycle_state = 'active'",
        [workspace_id],
    )?;
    for (position, project_id) in ordered_project_ids.iter().enumerate() {
        let updated = transaction.execute(
            "UPDATE projects SET position = ?1
             WHERE workspace_id = ?2 AND project_id = ?3 AND lifecycle_state = 'active'",
            params![position as i64, workspace_id, project_id],
        )?;
        if updated != 1 {
            return Err(DatabaseError::InvalidInput(format!(
                "active Project {project_id} is absent or repeated"
            )));
        }
    }
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_project_path(
    connection: &mut Connection,
    kind: &DatabaseKind,
    project_id: &str,
    current_path: &str,
    last_known_path: &str,
    path_state: ProjectPathState,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    require_nonempty("current_path", current_path)?;
    require_nonempty("last_known_path", last_known_path)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let updated = transaction.execute(
        "UPDATE projects
         SET current_path = ?1, last_known_path = ?2, path_state = ?3
         WHERE workspace_id = ?4 AND project_id = ?5",
        params![
            current_path,
            last_known_path,
            path_state.as_str(),
            workspace_id,
            project_id
        ],
    )?;
    require_one_update(updated, "Project", project_id)?;
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_project_name(
    connection: &mut Connection,
    kind: &DatabaseKind,
    project_id: &str,
    display_name: &str,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    require_nonempty("project_id", project_id)?;
    validate_text_field(
        "Project display name",
        display_name,
        MAX_PROJECT_DISPLAY_NAME_BYTES,
    )?;
    if display_name.trim().is_empty() || display_name.contains('\0') {
        return Err(DatabaseError::InvalidInput(
            "Project display name is invalid".into(),
        ));
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let updated = transaction.execute(
        "UPDATE projects SET display_name = ?1
         WHERE workspace_id = ?2 AND project_id = ?3 AND lifecycle_state = 'active'",
        params![display_name.trim(), workspace_id, project_id],
    )?;
    require_one_update(updated, "Project", project_id)?;
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_configuration(
    connection: &mut Connection,
    record: ConfigurationSnapshotRecord,
) -> DatabaseResult<u64> {
    require_nonempty("scope_id", &record.scope_id)?;
    validate_text_field(
        "configuration canonical document",
        &record.canonical_document,
        MAX_TEXT_FIELD_BYTES,
    )?;
    if !matches!(record.scope_kind.as_str(), "workspace" | "project" | "chat") {
        return Err(DatabaseError::InvalidInput(
            "configuration scope must be workspace, project, or chat".into(),
        ));
    }
    if record.generation == 0 {
        return Err(DatabaseError::InvalidInput(
            "configuration generation must be greater than zero".into(),
        ));
    }
    let configuration_generation = i64::try_from(record.generation).map_err(|_| {
        DatabaseError::InvalidInput(format!(
            "configuration generation {} exceeds SQLite's signed integer range",
            record.generation
        ))
    })?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let scope_exists = match record.scope_kind.as_str() {
        "workspace" => {
            if record.scope_id != record.workspace_id {
                return Err(DatabaseError::InvalidInput(
                    "Workspace configuration scope_id must equal workspace_id".into(),
                ));
            }
            transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM workspaces WHERE workspace_id = ?1)",
                [&record.workspace_id],
                |row| row.get::<_, i64>(0),
            )?
        }
        "project" => transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects
                           WHERE workspace_id = ?1 AND project_id = ?2)",
            params![record.workspace_id, record.scope_id],
            |row| row.get::<_, i64>(0),
        )?,
        "chat" => transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM chats
                           WHERE workspace_id = ?1 AND chat_id = ?2)",
            params![record.workspace_id, record.scope_id],
            |row| row.get::<_, i64>(0),
        )?,
        _ => unreachable!("scope kind was validated"),
    };
    if scope_exists != 1 {
        return Err(DatabaseError::InvalidInput(format!(
            "configuration {} scope {} does not reference an existing record",
            record.scope_kind, record.scope_id
        )));
    }
    let current_generation = transaction
        .query_row(
            "SELECT generation FROM configuration_lkg
             WHERE workspace_id = ?1 AND scope_kind = ?2 AND scope_id = ?3",
            params![record.workspace_id, record.scope_kind, record.scope_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    if current_generation.is_some_and(|current| configuration_generation <= current) {
        return Err(DatabaseError::InvalidInput(format!(
            "configuration generation {} is not newer than the active generation {}",
            record.generation,
            current_generation.unwrap_or_default()
        )));
    }
    transaction.execute(
        "INSERT INTO configuration_lkg(
            workspace_id, scope_kind, scope_id, canonical_document, generation, activated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(workspace_id, scope_kind, scope_id) DO UPDATE SET
            canonical_document = excluded.canonical_document,
            generation = excluded.generation,
            activated_at = excluded.activated_at",
        params![
            record.workspace_id,
            record.scope_kind,
            record.scope_id,
            record.canonical_document,
            configuration_generation,
            record.activated_at
        ],
    )?;
    let generation = bump_workspace_generation(&transaction, &record.workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_diagnostic(
    connection: &mut Connection,
    kind: &DatabaseKind,
    record: DiagnosticRecord,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    validate_diagnostic_record(&record)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    transaction.execute(
        "INSERT INTO workspace_diagnostics(
            workspace_id, diagnostic_id, category, message, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            workspace_id,
            record.diagnostic_id,
            record.category,
            record.message,
            record.created_at
        ],
    )?;
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_security_records(
    connection: &mut Connection,
    records: Vec<SecurityJournalRecord>,
) -> DatabaseResult<u64> {
    if records.is_empty() || records.len() > MAX_SECURITY_BATCH_RECORDS {
        return Err(DatabaseError::InvalidInput(format!(
            "security batch must contain between 1 and {MAX_SECURITY_BATCH_RECORDS} records"
        )));
    }
    let mut keys = HashSet::new();
    for record in &records {
        validate_security_record(record)?;
        if !keys.insert((record.record_kind.clone(), record.record_id.clone())) {
            return Err(DatabaseError::InvalidInput(
                "security batch repeats a record identity".into(),
            ));
        }
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current_count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM security_records", [], |row| {
            row.get(0)
        })?;
    let new_count = records.iter().try_fold(0_i64, |count, record| {
        let exists: i64 = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM security_records
                           WHERE record_kind = ?1 AND record_id = ?2)",
            params![record.record_kind, record.record_id],
            |row| row.get(0),
        )?;
        Ok::<_, rusqlite::Error>(count + i64::from(exists == 0))
    })?;
    if current_count
        .checked_add(new_count)
        .is_none_or(|count| count > MAX_SECURITY_CURRENT_RECORDS as i64)
    {
        return Err(DatabaseError::InvalidInput(format!(
            "security current-state capacity {MAX_SECURITY_CURRENT_RECORDS} reached"
        )));
    }

    for record in records {
        let recorded_at_ms = i64::try_from(record.recorded_at_ms).map_err(|_| {
            DatabaseError::InvalidInput(
                "security record timestamp exceeds SQLite's signed integer range".into(),
            )
        })?;
        let previous = transaction
            .query_row(
                "SELECT run_id, action_id, state, recorded_at_ms
                 FROM security_records WHERE record_kind = ?1 AND record_id = ?2",
                params![record.record_kind, record.record_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .optional()?;
        validate_security_transition(&record, previous.as_ref())?;
        let record_sha256 = security_record_digest(&record);
        transaction.execute(
            "INSERT INTO security_records(
                record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(record_kind, record_id) DO UPDATE SET
                state = excluded.state,
                canonical_document = excluded.canonical_document,
                record_sha256 = excluded.record_sha256,
                recorded_at_ms = excluded.recorded_at_ms",
            params![
                record.record_kind,
                record.record_id,
                record.run_id,
                record.action_id,
                record.state,
                record.canonical_document,
                record_sha256,
                recorded_at_ms
            ],
        )?;
        transaction.execute(
            "INSERT INTO security_events(
                record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.record_kind,
                record.record_id,
                record.run_id,
                record.action_id,
                record.state,
                record.canonical_document,
                record_sha256,
                recorded_at_ms
            ],
        )?;
    }
    let generation = bump_app_generation(&transaction)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_inactivation(
    connection: &mut Connection,
    kind: &DatabaseKind,
    entity: InactiveEntity,
    inactivated_at: i64,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    match entity {
        InactiveEntity::Workspace => {
            let updated = transaction.execute(
                "UPDATE workspaces
                 SET lifecycle_state = 'inactive', inactivated_at = ?1, updated_at = ?1
                 WHERE workspace_id = ?2",
                params![inactivated_at, workspace_id],
            )?;
            require_one_update(updated, "Workspace", workspace_id)?;
        }
        InactiveEntity::Project { project_id } => {
            let updated = transaction.execute(
                "UPDATE projects SET lifecycle_state = 'inactive', inactivated_at = ?1
                 WHERE workspace_id = ?2 AND project_id = ?3",
                params![inactivated_at, workspace_id, project_id],
            )?;
            require_one_update(updated, "Project", &project_id)?;
            compact_active_project_positions(&transaction, workspace_id)?;
        }
        InactiveEntity::Chat { chat_id } => {
            let updated = transaction.execute(
                "UPDATE chats SET lifecycle_state = 'inactive', inactivated_at = ?1
                 WHERE workspace_id = ?2 AND chat_id = ?3",
                params![inactivated_at, workspace_id, chat_id],
            )?;
            require_one_update(updated, "Chat", &chat_id)?;
        }
    }
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn write_workspace_inactivation_compensation(
    connection: &mut Connection,
    kind: &DatabaseKind,
    expected_generation: u64,
    prior_record: WorkspaceRecord,
) -> DatabaseResult<u64> {
    let workspace_id = workspace_id(kind)?;
    if prior_record.workspace_id != workspace_id {
        return Err(DatabaseError::InvalidInput(format!(
            "record Workspace {} does not match open Workspace {workspace_id}",
            prior_record.workspace_id
        )));
    }
    validate_canonical_display_name(
        "Workspace display name",
        &prior_record.display_name,
        MAX_WORKSPACE_DISPLAY_NAME_BYTES,
    )?;
    validate_inactivation(prior_record.lifecycle_state, prior_record.inactivated_at)?;
    let expected_generation = i64::try_from(expected_generation).map_err(|_| {
        DatabaseError::InvalidInput(
            "expected Workspace generation exceeds SQLite's signed integer range".into(),
        )
    })?;
    if expected_generation == i64::MAX {
        return Err(DatabaseError::InvalidInput(
            "expected Workspace generation cannot be incremented safely".into(),
        ));
    }

    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current_generation: i64 = transaction.query_row(
        "SELECT generation FROM durable_generation WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    if current_generation != expected_generation {
        return Err(DatabaseError::Validation(format!(
            "Workspace {workspace_id} generation changed from expected {expected_generation} to {current_generation}; inactivation compensation refused"
        )));
    }

    let current_record = transaction.query_row(
        "SELECT display_name, created_at, updated_at, lifecycle_state, inactivated_at
         FROM workspaces WHERE workspace_id = ?1",
        [workspace_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        },
    )?;
    let current_lifecycle = LifecycleState::parse(&current_record.3)?;
    validate_inactivation(current_lifecycle, current_record.4)?;
    if current_lifecycle != LifecycleState::Inactive
        || current_record.0 != prior_record.display_name
        || current_record.1 != prior_record.created_at
        || current_record.4 != Some(current_record.2)
    {
        return Err(DatabaseError::Validation(format!(
            "Workspace {workspace_id} is not the exact result of inactivating the supplied prior record; compensation refused"
        )));
    }

    let updated = transaction.execute(
        "UPDATE workspaces
         SET display_name = ?1, created_at = ?2, updated_at = ?3,
             lifecycle_state = ?4, inactivated_at = ?5
         WHERE workspace_id = ?6 AND lifecycle_state = 'inactive'",
        params![
            prior_record.display_name,
            prior_record.created_at,
            prior_record.updated_at,
            prior_record.lifecycle_state.as_str(),
            prior_record.inactivated_at,
            workspace_id
        ],
    )?;
    require_one_update(updated, "Workspace", workspace_id)?;
    let generation = bump_workspace_generation(&transaction, workspace_id)?;
    transaction.commit()?;
    Ok(generation)
}

fn compact_active_project_positions(
    transaction: &rusqlite::Transaction<'_>,
    workspace_id: &str,
) -> DatabaseResult<()> {
    let ordered_project_ids = {
        let mut statement = transaction.prepare(
            "SELECT project_id FROM projects
             WHERE workspace_id = ?1 AND lifecycle_state = 'active'
             ORDER BY position, project_id",
        )?;
        statement
            .query_map([workspace_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
    if ordered_project_ids.is_empty() {
        return Ok(());
    }

    let max_position: i64 = transaction.query_row(
        "SELECT MAX(position) FROM projects
         WHERE workspace_id = ?1 AND lifecycle_state = 'active'",
        [workspace_id],
        |row| row.get(0),
    )?;
    let temporary_offset = max_position.checked_add(1).ok_or_else(|| {
        DatabaseError::Validation("active Project position cannot be compacted safely".into())
    })?;
    max_position.checked_add(temporary_offset).ok_or_else(|| {
        DatabaseError::Validation("active Project position cannot be compacted safely".into())
    })?;
    transaction.execute(
        "UPDATE projects SET position = position + ?1
         WHERE workspace_id = ?2 AND lifecycle_state = 'active'",
        params![temporary_offset, workspace_id],
    )?;
    for (position, project_id) in ordered_project_ids.iter().enumerate() {
        let updated = transaction.execute(
            "UPDATE projects SET position = ?1
             WHERE workspace_id = ?2 AND project_id = ?3 AND lifecycle_state = 'active'",
            params![position as i64, workspace_id, project_id],
        )?;
        require_one_update(updated, "Project", project_id)?;
    }
    Ok(())
}

fn bump_app_generation(transaction: &rusqlite::Transaction<'_>) -> DatabaseResult<u64> {
    transaction.execute(
        "UPDATE durable_generation SET generation = generation + 1 WHERE singleton = 1",
        [],
    )?;
    let generation: i64 = transaction.query_row(
        "SELECT generation FROM durable_generation WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    generation_to_u64(generation)
}

fn bump_workspace_generation(
    transaction: &rusqlite::Transaction<'_>,
    workspace_id: &str,
) -> DatabaseResult<u64> {
    let updated = transaction.execute(
        "UPDATE durable_generation SET generation = generation + 1 WHERE workspace_id = ?1",
        [workspace_id],
    )?;
    if updated != 1 {
        return Err(DatabaseError::Validation(format!(
            "Workspace {workspace_id} has no durable generation record"
        )));
    }
    let generation: i64 = transaction.query_row(
        "SELECT generation FROM durable_generation WHERE workspace_id = ?1",
        [workspace_id],
        |row| row.get(0),
    )?;
    generation_to_u64(generation)
}

fn generation_to_u64(generation: i64) -> DatabaseResult<u64> {
    u64::try_from(generation)
        .map_err(|_| DatabaseError::Validation(format!("invalid generation {generation}")))
}

#[derive(Default)]
struct SnapshotTextBudget {
    used: usize,
}

impl SnapshotTextBudget {
    fn account(&mut self, max_field: i64, total: i64, context: &str) -> DatabaseResult<()> {
        let max_field = usize::try_from(max_field).map_err(|_| {
            DatabaseError::Validation(format!("negative text length while reading {context}"))
        })?;
        let total = usize::try_from(total).map_err(|_| {
            DatabaseError::Validation(format!("negative text total while reading {context}"))
        })?;
        if max_field > MAX_TEXT_FIELD_BYTES {
            return Err(DatabaseError::Validation(format!(
                "{context} contains a text field larger than {MAX_TEXT_FIELD_BYTES} bytes"
            )));
        }
        self.used = self.used.checked_add(total).ok_or_else(|| {
            DatabaseError::Validation("snapshot text byte accounting overflowed".into())
        })?;
        if self.used > MAX_SNAPSHOT_TEXT_BYTES {
            return Err(DatabaseError::Validation(format!(
                "snapshot text exceeds {MAX_SNAPSHOT_TEXT_BYTES} bytes"
            )));
        }
        Ok(())
    }
}

fn account_text_query<P: Params>(
    connection: &Connection,
    sql: &str,
    params: P,
    context: &str,
    budget: &mut SnapshotTextBudget,
) -> DatabaseResult<()> {
    let (max_field, total): (i64, i64) =
        connection.query_row(sql, params, |row| Ok((row.get(0)?, row.get(1)?)))?;
    budget.account(max_field, total, context)
}

fn validate_app_snapshot_text_budget(
    connection: &Connection,
    query: SnapshotQuery,
) -> DatabaseResult<()> {
    let mut budget = SnapshotTextBudget::default();
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(length(CAST(installation_id AS BLOB))), 0),
                COALESCE(SUM(length(CAST(installation_id AS BLOB))), 0)
         FROM (SELECT installation_id FROM installations
               ORDER BY updated_at DESC, installation_id LIMIT 1)",
        [],
        "installation snapshot",
        &mut budget,
    )?;
    let lifecycle_filter = if query.include_inactive {
        ""
    } else {
        "WHERE lifecycle_state = 'active'"
    };
    let recent_sql = format!(
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(display_name AS BLOB)),
                    length(CAST(archive_path AS BLOB)),
                    length(CAST(lifecycle_state AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(display_name AS BLOB)) +
                    length(CAST(archive_path AS BLOB)) +
                    length(CAST(lifecycle_state AS BLOB))), 0)
         FROM (SELECT workspace_id, display_name, archive_path, lifecycle_state
               FROM recent_workspaces {lifecycle_filter}
               ORDER BY last_opened_at DESC, workspace_id LIMIT ?1)"
    );
    account_text_query(
        connection,
        &recent_sql,
        [limit_plus_one(query.max_records)?],
        "recent Workspace snapshot",
        &mut budget,
    )?;
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(canonical_document AS BLOB)),
                    length(CAST(document_sha256 AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(canonical_document AS BLOB)) +
                    length(CAST(document_sha256 AS BLOB))), 0)
         FROM app_configuration_lkg WHERE singleton = 1",
        [],
        "app configuration snapshot",
        &mut budget,
    )
}

fn validate_workspace_snapshot_text_budget(
    connection: &Connection,
    workspace_id: &str,
    query: SnapshotQuery,
) -> DatabaseResult<()> {
    let mut budget = SnapshotTextBudget::default();
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(display_name AS BLOB)),
                    length(CAST(lifecycle_state AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(display_name AS BLOB)) +
                    length(CAST(lifecycle_state AS BLOB))), 0)
         FROM workspaces WHERE workspace_id = ?1",
        [workspace_id],
        "Workspace identity snapshot",
        &mut budget,
    )?;
    let lifecycle_filter = if query.include_inactive {
        ""
    } else {
        "AND lifecycle_state = 'active'"
    };
    let project_sql = format!(
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(project_id AS BLOB)),
                    length(CAST(display_name AS BLOB)),
                    length(CAST(current_path AS BLOB)),
                    length(CAST(last_known_path AS BLOB)),
                    length(CAST(path_state AS BLOB)),
                    length(CAST(lifecycle_state AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(project_id AS BLOB)) +
                    length(CAST(display_name AS BLOB)) +
                    length(CAST(current_path AS BLOB)) +
                    length(CAST(last_known_path AS BLOB)) +
                    length(CAST(path_state AS BLOB)) +
                    length(CAST(lifecycle_state AS BLOB))), 0)
         FROM (SELECT workspace_id, project_id, display_name, current_path,
                      last_known_path, path_state, lifecycle_state
               FROM projects WHERE workspace_id = ?1 {lifecycle_filter}
               ORDER BY position, project_id LIMIT ?2)"
    );
    account_text_query(
        connection,
        &project_sql,
        params![workspace_id, limit_plus_one(query.max_records)?],
        "Project snapshot",
        &mut budget,
    )?;
    let (project_join, chat_filter) = if query.include_inactive {
        ("", "")
    } else {
        (
            "JOIN projects visible_project
                ON visible_project.workspace_id = chats.workspace_id
                AND visible_project.project_id = chats.project_id",
            "AND chats.lifecycle_state = 'active'
             AND visible_project.lifecycle_state = 'active'",
        )
    };
    let chat_sql = format!(
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(project_id AS BLOB)),
                    length(CAST(chat_id AS BLOB)),
                    length(CAST(title AS BLOB)),
                    length(CAST(lifecycle_state AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(project_id AS BLOB)) +
                    length(CAST(chat_id AS BLOB)) +
                    length(CAST(title AS BLOB)) +
                    length(CAST(lifecycle_state AS BLOB))), 0)
         FROM (SELECT chats.workspace_id AS workspace_id,
                      chats.project_id AS project_id,
                      chats.chat_id AS chat_id,
                      chats.title AS title,
                      chats.lifecycle_state AS lifecycle_state
               FROM chats {project_join}
               WHERE chats.workspace_id = ?1 {chat_filter}
               ORDER BY chats.updated_at DESC, chats.chat_id LIMIT ?2)"
    );
    account_text_query(
        connection,
        &chat_sql,
        params![workspace_id, limit_plus_one(query.max_records)?],
        "Chat snapshot",
        &mut budget,
    )?;
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(workspace_id AS BLOB)),
                    length(CAST(scope_kind AS BLOB)),
                    length(CAST(scope_id AS BLOB)),
                    length(CAST(canonical_document AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(workspace_id AS BLOB)) +
                    length(CAST(scope_kind AS BLOB)) +
                    length(CAST(scope_id AS BLOB)) +
                    length(CAST(canonical_document AS BLOB))), 0)
         FROM (SELECT workspace_id, scope_kind, scope_id, canonical_document
               FROM configuration_lkg WHERE workspace_id = ?1
               ORDER BY scope_kind, scope_id LIMIT ?2)",
        params![workspace_id, limit_plus_one(query.max_records)?],
        "configuration snapshot",
        &mut budget,
    )?;
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(diagnostic_id AS BLOB)),
                    length(CAST(category AS BLOB)),
                    length(CAST(message AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(diagnostic_id AS BLOB)) +
                    length(CAST(category AS BLOB)) +
                    length(CAST(message AS BLOB))), 0)
         FROM (SELECT diagnostic_id, category, message
               FROM workspace_diagnostics WHERE workspace_id = ?1
               ORDER BY created_at DESC, diagnostic_id LIMIT ?2)",
        params![workspace_id, limit_plus_one(query.max_records)?],
        "diagnostic snapshot",
        &mut budget,
    )
}

fn read_app_snapshot(connection: &Connection, query: SnapshotQuery) -> DatabaseResult<AppSnapshot> {
    validate_app_snapshot_text_budget(connection, query)?;
    let installation = connection
        .query_row(
            "SELECT installation_id, created_at, updated_at
             FROM installations ORDER BY updated_at DESC, installation_id LIMIT 1",
            [],
            |row| {
                Ok(InstallationRecord {
                    installation_id: row.get(0)?,
                    created_at: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            },
        )
        .optional()?;
    let lifecycle_filter = if query.include_inactive {
        ""
    } else {
        "WHERE lifecycle_state = 'active'"
    };
    let sql = format!(
        "SELECT workspace_id, display_name, archive_path, last_opened_at,
                lifecycle_state, inactivated_at
         FROM recent_workspaces {lifecycle_filter}
         ORDER BY last_opened_at DESC, workspace_id LIMIT ?1"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([limit_plus_one(query.max_records)?], |row| {
        let lifecycle: String = row.get(4)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            lifecycle,
            row.get::<_, Option<i64>>(5)?,
        ))
    })?;
    let mut recents = Vec::new();
    for row in rows {
        let (workspace_id, display_name, archive_path, last_opened_at, lifecycle, inactivated_at) =
            row?;
        recents.push(RecentWorkspaceRecord {
            workspace_id,
            display_name,
            archive_path,
            last_opened_at,
            lifecycle_state: LifecycleState::parse(&lifecycle)?,
            inactivated_at,
        });
        validate_canonical_display_name(
            "Workspace display name",
            &recents
                .last()
                .expect("record was just appended")
                .display_name,
            MAX_WORKSPACE_DISPLAY_NAME_BYTES,
        )?;
    }
    let truncated = truncate(&mut recents, query.max_records);
    let generation: i64 = connection.query_row(
        "SELECT generation FROM durable_generation WHERE singleton = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(AppSnapshot {
        installation,
        recents,
        configuration_lkg: read_app_configuration_lkg(connection)?,
        generation: generation_to_u64(generation)?,
        truncated,
    })
}

fn read_app_configuration_lkg(
    connection: &Connection,
) -> DatabaseResult<Option<AppConfigurationSnapshotRecord>> {
    let mut budget = SnapshotTextBudget::default();
    account_text_query(
        connection,
        "SELECT COALESCE(MAX(MAX(
                    length(CAST(canonical_document AS BLOB)),
                    length(CAST(document_sha256 AS BLOB)))), 0),
                COALESCE(SUM(
                    length(CAST(canonical_document AS BLOB)) +
                    length(CAST(document_sha256 AS BLOB))), 0)
         FROM app_configuration_lkg WHERE singleton = 1",
        [],
        "app configuration last-known-good record",
        &mut budget,
    )?;
    let raw = connection
        .query_row(
            "SELECT canonical_document, generation, activated_at, document_sha256
             FROM app_configuration_lkg WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;
    raw.map(
        |(canonical_document, generation, activated_at, stored_digest)| {
            let actual_digest = canonical_document_digest(&canonical_document);
            if stored_digest != actual_digest {
                return Err(DatabaseError::Validation(
                    "app configuration last-known-good document digest mismatch".into(),
                ));
            }
            Ok(AppConfigurationSnapshotRecord {
                canonical_document,
                generation: generation_to_u64(generation)?,
                activated_at,
            })
        },
    )
    .transpose()
}

fn canonical_document_digest(document: &str) -> String {
    let digest = Sha256::digest(document.as_bytes());
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn security_record_digest(record: &SecurityJournalRecord) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"c4os-security-record-v1\0");
    for field in [
        record.record_kind.as_bytes(),
        record.record_id.as_bytes(),
        record.run_id.as_bytes(),
        record.action_id.as_bytes(),
        record.state.as_bytes(),
        record.canonical_document.as_bytes(),
    ] {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field);
    }
    hasher.update(record.recorded_at_ms.to_le_bytes());
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn read_workspace_snapshot(
    connection: &Connection,
    workspace_id: &str,
    query: SnapshotQuery,
) -> DatabaseResult<WorkspaceSnapshot> {
    validate_workspace_snapshot_text_budget(connection, workspace_id, query)?;
    let workspace = read_workspace_record(connection, workspace_id, query.include_inactive)?;
    if workspace.is_none() && !query.include_inactive {
        return Ok(WorkspaceSnapshot {
            workspace: None,
            projects: Vec::new(),
            chats: Vec::new(),
            configurations: Vec::new(),
            diagnostics: Vec::new(),
            generation: read_workspace_generation(connection, workspace_id)?,
            truncated: false,
        });
    }
    let (projects, projects_truncated) = read_projects(connection, workspace_id, query)?;
    let (chats, chats_truncated) = read_chats(connection, workspace_id, query)?;
    let (configurations, configurations_truncated) =
        read_configurations(connection, workspace_id, query.max_records)?;
    let (diagnostics, diagnostics_truncated) =
        read_diagnostics(connection, workspace_id, query.max_records)?;
    let generation = read_workspace_generation(connection, workspace_id)?;

    Ok(WorkspaceSnapshot {
        workspace,
        projects,
        chats,
        configurations,
        diagnostics,
        generation,
        truncated: projects_truncated
            || chats_truncated
            || configurations_truncated
            || diagnostics_truncated,
    })
}

fn read_workspace_generation(connection: &Connection, workspace_id: &str) -> DatabaseResult<u64> {
    connection
        .query_row(
            "SELECT generation FROM durable_generation WHERE workspace_id = ?1",
            [workspace_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(generation_to_u64)
        .transpose()
        .map(|generation| generation.unwrap_or(0))
}

fn read_workspace_record(
    connection: &Connection,
    workspace_id: &str,
    include_inactive: bool,
) -> DatabaseResult<Option<WorkspaceRecord>> {
    let lifecycle_filter = if include_inactive {
        ""
    } else {
        "AND lifecycle_state = 'active'"
    };
    let sql = format!(
        "SELECT workspace_id, display_name, created_at, updated_at,
                lifecycle_state, inactivated_at
         FROM workspaces WHERE workspace_id = ?1 {lifecycle_filter}"
    );
    let raw = connection
        .query_row(&sql, [workspace_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<i64>>(5)?,
            ))
        })
        .optional()?;
    raw.map(
        |(workspace_id, display_name, created_at, updated_at, lifecycle, inactivated_at)| {
            let record = WorkspaceRecord {
                workspace_id,
                display_name,
                created_at,
                updated_at,
                lifecycle_state: LifecycleState::parse(&lifecycle)?,
                inactivated_at,
            };
            validate_canonical_display_name(
                "Workspace display name",
                &record.display_name,
                MAX_WORKSPACE_DISPLAY_NAME_BYTES,
            )?;
            Ok(record)
        },
    )
    .transpose()
}

fn read_projects(
    connection: &Connection,
    workspace_id: &str,
    query: SnapshotQuery,
) -> DatabaseResult<(Vec<ProjectRecord>, bool)> {
    let lifecycle_filter = if query.include_inactive {
        ""
    } else {
        "AND lifecycle_state = 'active'"
    };
    let sql = format!(
        "SELECT workspace_id, project_id, display_name, current_path, last_known_path,
                path_state, position, lifecycle_state, inactivated_at
         FROM projects WHERE workspace_id = ?1 {lifecycle_filter}
         ORDER BY position, project_id LIMIT ?2"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        params![workspace_id, limit_plus_one(query.max_records)?],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<i64>>(8)?,
            ))
        },
    )?;
    let mut records = Vec::new();
    for row in rows {
        let (
            workspace_id,
            project_id,
            display_name,
            current_path,
            last_known_path,
            path_state,
            position,
            lifecycle,
            inactivated_at,
        ) = row?;
        let record = ProjectRecord {
            workspace_id,
            project_id,
            display_name,
            current_path,
            last_known_path,
            path_state: ProjectPathState::parse(&path_state)?,
            position,
            lifecycle_state: LifecycleState::parse(&lifecycle)?,
            inactivated_at,
        };
        validate_canonical_display_name(
            "Project display name",
            &record.display_name,
            MAX_PROJECT_DISPLAY_NAME_BYTES,
        )?;
        records.push(record);
    }
    let truncated = truncate(&mut records, query.max_records);
    Ok((records, truncated))
}

fn read_chats(
    connection: &Connection,
    workspace_id: &str,
    query: SnapshotQuery,
) -> DatabaseResult<(Vec<ChatRecord>, bool)> {
    let (project_join, lifecycle_filter) = if query.include_inactive {
        ("", "")
    } else {
        (
            "JOIN projects visible_project
                ON visible_project.workspace_id = chats.workspace_id
                AND visible_project.project_id = chats.project_id",
            "AND chats.lifecycle_state = 'active'
             AND visible_project.lifecycle_state = 'active'",
        )
    };
    let sql = format!(
        "SELECT chats.workspace_id, chats.project_id, chats.chat_id, chats.title,
                chats.created_at, chats.updated_at, chats.lifecycle_state, chats.inactivated_at
         FROM chats {project_join} WHERE chats.workspace_id = ?1 {lifecycle_filter}
         ORDER BY chats.updated_at DESC, chats.chat_id LIMIT ?2"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        params![workspace_id, limit_plus_one(query.max_records)?],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<i64>>(7)?,
            ))
        },
    )?;
    let mut records = Vec::new();
    for row in rows {
        let (
            workspace_id,
            project_id,
            chat_id,
            title,
            created_at,
            updated_at,
            lifecycle,
            inactivated_at,
        ) = row?;
        records.push(ChatRecord {
            workspace_id,
            project_id,
            chat_id,
            title,
            created_at,
            updated_at,
            lifecycle_state: LifecycleState::parse(&lifecycle)?,
            inactivated_at,
        });
    }
    let truncated = truncate(&mut records, query.max_records);
    Ok((records, truncated))
}

fn read_configurations(
    connection: &Connection,
    workspace_id: &str,
    max_records: usize,
) -> DatabaseResult<(Vec<ConfigurationSnapshotRecord>, bool)> {
    let mut statement = connection.prepare(
        "SELECT workspace_id, scope_kind, scope_id, canonical_document, generation, activated_at
         FROM configuration_lkg WHERE workspace_id = ?1
         ORDER BY scope_kind, scope_id LIMIT ?2",
    )?;
    let rows = statement.query_map(params![workspace_id, limit_plus_one(max_records)?], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;
    let mut records = Vec::new();
    for row in rows {
        let (workspace_id, scope_kind, scope_id, canonical_document, generation, activated_at) =
            row?;
        records.push(ConfigurationSnapshotRecord {
            workspace_id,
            scope_kind,
            scope_id,
            canonical_document,
            generation: generation_to_u64(generation)?,
            activated_at,
        });
    }
    let truncated = truncate(&mut records, max_records);
    Ok((records, truncated))
}

fn read_diagnostics(
    connection: &Connection,
    workspace_id: &str,
    max_records: usize,
) -> DatabaseResult<(Vec<DiagnosticRecord>, bool)> {
    let mut statement = connection.prepare(
        "SELECT diagnostic_id, category, message, created_at
         FROM workspace_diagnostics WHERE workspace_id = ?1
         ORDER BY created_at DESC, diagnostic_id LIMIT ?2",
    )?;
    let rows = statement.query_map(params![workspace_id, limit_plus_one(max_records)?], |row| {
        Ok(DiagnosticRecord {
            diagnostic_id: row.get(0)?,
            category: row.get(1)?,
            message: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        let record = row?;
        validate_diagnostic_record(&record)?;
        records.push(record);
    }
    let truncated = truncate(&mut records, max_records);
    Ok((records, truncated))
}

fn read_security_records(
    connection: &Connection,
    query: SnapshotQuery,
) -> DatabaseResult<Vec<SecurityJournalRecord>> {
    let mut statement = connection.prepare(
        "SELECT record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
         FROM security_records
         ORDER BY recorded_at_ms DESC, record_kind, record_id
         LIMIT ?1",
    )?;
    let rows = statement.query_map([limit_plus_one(query.max_records)?], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;
    let mut records = Vec::new();
    let mut bytes = 0_usize;
    for row in rows {
        let (
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            digest,
            recorded_at_ms,
        ) = row?;
        let recorded_at_ms = u64::try_from(recorded_at_ms)
            .map_err(|_| DatabaseError::Validation("negative security record timestamp".into()))?;
        let record = SecurityJournalRecord {
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            recorded_at_ms,
        };
        if security_record_digest(&record) != digest {
            return Err(DatabaseError::Validation(format!(
                "security record {}/{} digest mismatch",
                record.record_kind, record.record_id
            )));
        }
        validate_security_record(&record)?;
        bytes = bytes
            .checked_add(security_record_text_bytes(&record))
            .ok_or_else(|| {
                DatabaseError::Validation("security record byte accounting overflowed".into())
            })?;
        if bytes > MAX_SNAPSHOT_TEXT_BYTES {
            return Err(DatabaseError::Validation(format!(
                "security record page exceeds {MAX_SNAPSHOT_TEXT_BYTES} bytes"
            )));
        }
        records.push(record);
    }
    if records.len() > query.max_records {
        return Err(DatabaseError::Validation(
            "security current-state read is truncated; use the recoverable-state query or a narrower filter"
                .into(),
        ));
    }
    Ok(records)
}

fn read_security_event_page(
    connection: &Connection,
    before_event_id: Option<i64>,
    query: SnapshotQuery,
) -> DatabaseResult<SecurityJournalPage> {
    let mut statement = connection.prepare(
        "SELECT event_id, record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
         FROM security_events
         WHERE (?1 IS NULL OR event_id < ?1)
         ORDER BY event_id DESC LIMIT ?2",
    )?;
    let rows = statement.query_map(
        params![before_event_id, limit_plus_one(query.max_records)?],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)?,
            ))
        },
    )?;
    let mut events = Vec::new();
    let mut bytes = 0_usize;
    let mut has_more = false;
    for row in rows {
        let (
            event_id,
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            digest,
            recorded_at_ms,
        ) = row?;
        let recorded_at_ms = u64::try_from(recorded_at_ms)
            .map_err(|_| DatabaseError::Validation("negative security event timestamp".into()))?;
        let current = SecurityJournalRecord {
            record_kind: record_kind.clone(),
            record_id: record_id.clone(),
            run_id: run_id.clone(),
            action_id: action_id.clone(),
            state: state.clone(),
            canonical_document: canonical_document.clone(),
            recorded_at_ms,
        };
        if security_record_digest(&current) != digest {
            return Err(DatabaseError::Validation(format!(
                "security event {event_id} digest mismatch"
            )));
        }
        validate_security_record(&current)?;
        let next_bytes = bytes
            .checked_add(security_record_text_bytes(&current))
            .ok_or_else(|| {
                DatabaseError::Validation("security event byte accounting overflowed".into())
            })?;
        if events.len() == query.max_records || next_bytes > MAX_SNAPSHOT_TEXT_BYTES {
            has_more = true;
            break;
        }
        bytes = next_bytes;
        events.push(SecurityJournalEvent {
            event_id,
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            recorded_at_ms,
        });
    }
    let next_before_event_id = has_more
        .then(|| events.last().map(|event| event.event_id))
        .flatten();
    Ok(SecurityJournalPage {
        events,
        next_before_event_id,
    })
}

fn read_recoverable_security_records(
    connection: &Connection,
) -> DatabaseResult<Vec<SecurityJournalRecord>> {
    let mut statement = connection.prepare(
        "SELECT record_kind, record_id, run_id, action_id, state,
                canonical_document, record_sha256, recorded_at_ms
         FROM security_records current
         WHERE (record_kind = 'authorization' AND state = 'issued')
            OR (record_kind = 'approval-prompt' AND state IN ('pending', 'queued', 'approved'))
            OR (record_kind = 'action-intent' AND state = 'effect-started'
                AND NOT EXISTS (
                    SELECT 1 FROM security_records result
                    WHERE result.record_kind = 'action-result'
                      AND result.action_id = current.action_id
                ))
         ORDER BY recorded_at_ms, record_kind, record_id
         LIMIT ?1",
    )?;
    let limit = i64::try_from(MAX_SECURITY_CURRENT_RECORDS + 1)
        .map_err(|_| DatabaseError::Validation("security recovery limit overflowed".into()))?;
    let rows = statement.query_map([limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?,
        ))
    })?;
    let mut records = Vec::new();
    let mut bytes = 0_usize;
    for row in rows {
        let (record_kind, record_id, run_id, action_id, state, canonical_document, digest, at) =
            row?;
        let record = SecurityJournalRecord {
            record_kind,
            record_id,
            run_id,
            action_id,
            state,
            canonical_document,
            recorded_at_ms: u64::try_from(at).map_err(|_| {
                DatabaseError::Validation("negative security recovery timestamp".into())
            })?,
        };
        validate_security_record(&record)?;
        if security_record_digest(&record) != digest {
            return Err(DatabaseError::Validation(format!(
                "security recovery record {}/{} digest mismatch",
                record.record_kind, record.record_id
            )));
        }
        bytes = bytes
            .checked_add(security_record_text_bytes(&record))
            .ok_or_else(|| {
                DatabaseError::Validation("security recovery byte accounting overflowed".into())
            })?;
        if bytes > MAX_SNAPSHOT_TEXT_BYTES {
            return Err(DatabaseError::Validation(format!(
                "security recovery state exceeds {MAX_SNAPSHOT_TEXT_BYTES} bytes"
            )));
        }
        records.push(record);
    }
    if records.len() > MAX_SECURITY_CURRENT_RECORDS {
        return Err(DatabaseError::Validation(
            "security recovery state exceeds its bounded capacity".into(),
        ));
    }
    Ok(records)
}

fn security_record_text_bytes(record: &SecurityJournalRecord) -> usize {
    record.record_kind.len()
        + record.record_id.len()
        + record.run_id.len()
        + record.action_id.len()
        + record.state.len()
        + record.canonical_document.len()
}

fn validate_text_field(name: &str, value: &str, max_bytes: usize) -> DatabaseResult<()> {
    if value.len() > max_bytes {
        return Err(DatabaseError::InvalidInput(format!(
            "{name} exceeds {max_bytes} bytes"
        )));
    }
    Ok(())
}

fn validate_canonical_display_name(
    name: &str,
    value: &str,
    max_bytes: usize,
) -> DatabaseResult<()> {
    if value.is_empty()
        || value != value.trim()
        || value.len() > max_bytes
        || value.chars().any(char::is_control)
    {
        return Err(DatabaseError::InvalidInput(format!(
            "{name} must be canonical, non-empty, control-free, and at most {max_bytes} bytes"
        )));
    }
    Ok(())
}

fn validate_diagnostic_record(record: &DiagnosticRecord) -> DatabaseResult<()> {
    require_nonempty("diagnostic_id", &record.diagnostic_id)?;
    require_nonempty("diagnostic category", &record.category)?;
    require_nonempty("diagnostic message", &record.message)?;
    validate_text_field("diagnostic_id", &record.diagnostic_id, 256)?;
    validate_text_field("diagnostic category", &record.category, 128)?;
    validate_text_field(
        "diagnostic message",
        &record.message,
        MAX_DIAGNOSTIC_MESSAGE_BYTES,
    )?;
    if record
        .message
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(DatabaseError::InvalidInput(
            "diagnostic message contains unsafe control characters".into(),
        ));
    }
    let lower = record.message.to_ascii_lowercase();
    const CREDENTIAL_MARKERS: &[&str] = &[
        "authorization:",
        "bearer ",
        "cookie=",
        "password=",
        "secret=",
        "token=",
    ];
    if CREDENTIAL_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(DatabaseError::InvalidInput(
            "diagnostic message contains credential-like material".into(),
        ));
    }
    Ok(())
}

fn validate_security_record(record: &SecurityJournalRecord) -> DatabaseResult<()> {
    const KINDS: &[&str] = &[
        "authorization",
        "approval-prompt",
        "action-intent",
        "action-decision",
        "action-result",
    ];
    if !KINDS.contains(&record.record_kind.as_str()) {
        return Err(DatabaseError::InvalidInput(format!(
            "unknown security record kind {}",
            record.record_kind
        )));
    }
    for (name, value) in [
        ("security record_id", &record.record_id),
        ("security run_id", &record.run_id),
        ("security action_id", &record.action_id),
        ("security state", &record.state),
    ] {
        require_nonempty(name, value)?;
        validate_text_field(name, value, 256)?;
        if value.chars().any(char::is_control) {
            return Err(DatabaseError::InvalidInput(format!(
                "{name} contains control characters"
            )));
        }
    }
    validate_text_field(
        "security canonical document",
        &record.canonical_document,
        MAX_TEXT_FIELD_BYTES,
    )?;
    let document: serde_json::Value =
        serde_json::from_str(&record.canonical_document).map_err(|_| {
            DatabaseError::InvalidInput("security canonical document is not valid JSON".into())
        })?;
    let Some(envelope) = document.as_object() else {
        return Err(DatabaseError::InvalidInput(
            "security canonical document must be a JSON object".into(),
        ));
    };
    let schema_version = envelope
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64);
    let recorded_at_ms = envelope
        .get("recordedAtMs")
        .and_then(serde_json::Value::as_u64);
    let matches_column = |field: &str, expected: &str| {
        envelope.get(field).and_then(serde_json::Value::as_str) == Some(expected)
    };
    if schema_version != Some(1)
        || recorded_at_ms != Some(record.recorded_at_ms)
        || !matches_column("recordKind", &record.record_kind)
        || !matches_column("recordId", &record.record_id)
        || !matches_column("runId", &record.run_id)
        || !matches_column("actionId", &record.action_id)
        || !matches_column("state", &record.state)
        || !envelope
            .get("payload")
            .is_some_and(serde_json::Value::is_object)
    {
        return Err(DatabaseError::InvalidInput(
            "security canonical envelope does not match its indexed columns".into(),
        ));
    }
    if security_document_has_raw_secret(
        envelope
            .get("payload")
            .expect("payload presence was checked"),
    ) {
        return Err(DatabaseError::InvalidInput(
            "security canonical document contains raw credential-like material".into(),
        ));
    }
    Ok(())
}

fn validate_security_transition(
    record: &SecurityJournalRecord,
    previous: Option<&(String, String, String, i64)>,
) -> DatabaseResult<()> {
    if let Some((run_id, action_id, state, recorded_at_ms)) = previous {
        if run_id != &record.run_id || action_id != &record.action_id {
            return Err(DatabaseError::InvalidInput(
                "security record identity cannot be rebound to another run or action".into(),
            ));
        }
        if record.recorded_at_ms < u64::try_from(*recorded_at_ms).unwrap_or(u64::MAX) {
            return Err(DatabaseError::InvalidInput(
                "security record timestamp cannot move backwards".into(),
            ));
        }
        if !allowed_security_transition(&record.record_kind, state, &record.state) {
            return Err(DatabaseError::InvalidInput(format!(
                "invalid {} security transition {state} -> {}",
                record.record_kind, record.state
            )));
        }
    } else if !allowed_initial_security_state(&record.record_kind, &record.state) {
        return Err(DatabaseError::InvalidInput(format!(
            "invalid initial {} security state {}",
            record.record_kind, record.state
        )));
    }
    Ok(())
}

fn allowed_initial_security_state(kind: &str, state: &str) -> bool {
    match kind {
        "authorization" => state == "issued",
        "approval-prompt" => matches!(state, "pending" | "queued"),
        "action-intent" => state == "proposed",
        "action-decision" => matches!(state, "allow" | "ask" | "deny"),
        "action-result" => matches!(
            state,
            "succeeded" | "failed" | "cancelled" | "denied" | "unknown-after-interruption"
        ),
        _ => false,
    }
}

fn allowed_security_transition(kind: &str, from: &str, to: &str) -> bool {
    match kind {
        "authorization" => {
            from == "issued"
                && matches!(
                    to,
                    "consumed" | "expired" | "cancelled" | "revoked" | "invalidated"
                )
        }
        "approval-prompt" => match from {
            "queued" => matches!(to, "pending" | "expired" | "cancelled"),
            "pending" => matches!(to, "approved" | "denied" | "expired" | "cancelled"),
            "approved" => matches!(to, "completed" | "cancelled"),
            _ => false,
        },
        "action-intent" => match from {
            "proposed" => matches!(to, "effect-started" | "interrupted" | "effect-finished"),
            "effect-started" => matches!(to, "effect-finished" | "interrupted"),
            _ => false,
        },
        // Decisions and results are immutable terminal facts.
        "action-decision" | "action-result" => false,
        _ => false,
    }
}

fn security_document_has_raw_secret(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => fields.iter().any(|(key, value)| {
            let normalized = key
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>();
            let sensitive_key = [
                "password",
                "passwd",
                "secret",
                "credential",
                "token",
                "cookie",
                "authorization",
                "apikey",
            ]
            .iter()
            .any(|marker| normalized.contains(marker));
            let safe_reference = normalized.ends_with("id")
                || normalized.ends_with("hash")
                || normalized.ends_with("reference")
                || normalized.ends_with("kind")
                || normalized.ends_with("state");
            (sensitive_key && !safe_reference) || security_document_has_raw_secret(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(security_document_has_raw_secret),
        serde_json::Value::String(value) => {
            let lower = value.to_ascii_lowercase();
            [
                "bearer ",
                "basic ",
                "cookie=",
                "password=",
                "passwd=",
                "secret=",
                "token=",
            ]
            .iter()
            .any(|marker| lower.contains(marker))
        }
        _ => false,
    }
}

fn sanitize_internal_diagnostic(message: &str, max_bytes: usize) -> String {
    let mut sanitized = String::with_capacity(message.len().min(max_bytes));
    for character in message.chars() {
        let character = if character.is_control() && !matches!(character, '\n' | '\r' | '\t') {
            '\u{fffd}'
        } else {
            character
        };
        if sanitized.len() + character.len_utf8() > max_bytes {
            break;
        }
        sanitized.push(character);
    }
    sanitized
}

fn workspace_id(kind: &DatabaseKind) -> DatabaseResult<&str> {
    match kind {
        DatabaseKind::Workspace { workspace_id } => Ok(workspace_id),
        DatabaseKind::App => Err(DatabaseError::WrongKind {
            expected: "workspace",
            actual: "app",
        }),
    }
}

fn validate_inactivation(
    lifecycle_state: LifecycleState,
    inactivated_at: Option<i64>,
) -> DatabaseResult<()> {
    match (lifecycle_state, inactivated_at) {
        (LifecycleState::Active, None) | (LifecycleState::Inactive, Some(_)) => Ok(()),
        (LifecycleState::Active, Some(_)) => Err(DatabaseError::InvalidInput(
            "active records cannot have an inactivation timestamp".into(),
        )),
        (LifecycleState::Inactive, None) => Err(DatabaseError::InvalidInput(
            "inactive records require an inactivation timestamp".into(),
        )),
    }
}

fn require_nonempty(name: &str, value: &str) -> DatabaseResult<()> {
    if value.trim().is_empty() {
        return Err(DatabaseError::InvalidInput(format!(
            "{name} cannot be empty"
        )));
    }
    Ok(())
}

fn require_one_update(updated: usize, kind: &str, identifier: &str) -> DatabaseResult<()> {
    if updated != 1 {
        return Err(DatabaseError::InvalidInput(format!(
            "{kind} {identifier} does not exist"
        )));
    }
    Ok(())
}

fn limit_plus_one(limit: usize) -> DatabaseResult<i64> {
    i64::try_from(limit.saturating_add(1))
        .map_err(|_| DatabaseError::InvalidInput("read limit is too large".into()))
}

fn truncate<T>(records: &mut Vec<T>, max_records: usize) -> bool {
    if records.len() > max_records {
        records.truncate(max_records);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auxiliary_connection_permits_are_strictly_bounded_and_released() {
        let permits = (0..MAX_CONCURRENT_AUXILIARY_CONNECTIONS)
            .map(|_| AuxiliaryConnectionPermit::acquire().expect("permit within bound"))
            .collect::<Vec<_>>();
        let error = match AuxiliaryConnectionPermit::acquire() {
            Ok(_) => panic!("connection permit above bound must fail closed"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("connection limit"));
        drop(permits);
        AuxiliaryConnectionPermit::acquire().expect("released permit can be reacquired");
    }
}
