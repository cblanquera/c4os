//! Durable adapters from strict runtime/session domains to the existing
//! single-writer SQLite actor.

use std::sync::{Arc, RwLock, RwLockWriteGuard};

use thiserror::Error;

use crate::core::database::{
    ChatRecord, DatabaseActor, DatabaseError, DatabaseKind, LifecycleState,
    RuntimeStateDocumentRecord, WorkspaceSessionDocumentRecord,
};
use crate::runtime::capability_evidence::{
    CapabilityEvidenceError, CapabilityEvidenceRegistry, CapabilityEvidenceSnapshot,
};
use crate::runtime::provider::{ProviderError, ProviderService, ProviderSnapshot};
use crate::runtime::session::{
    SessionError, SessionRecord, SessionRepository, SessionRepositoryError,
};
use crate::runtime::supervisor::{
    CompatibilityRequirement, RuntimeSupervisor, SupervisorError, SupervisorSnapshot,
};

const PROVIDER_DOCUMENT_KIND: &str = "provider-snapshot";
const PROVIDER_DOCUMENT_ID: &str = "providers";
const SUPERVISOR_DOCUMENT_KIND: &str = "supervisor-snapshot";
const SUPERVISOR_DOCUMENT_ID: &str = "runtimes";
const CONTROL_PLANE_DOCUMENT_KIND: &str = "runtime-control-plane";
const CONTROL_PLANE_DOCUMENT_ID: &str = "runtime";
const CONTROL_PLANE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeControlPlaneSnapshot {
    schema_version: u16,
    revision: u64,
    supervisor: SupervisorSnapshot,
    capabilities: CapabilityEvidenceSnapshot,
}

pub struct RestoredRuntimeControlPlane {
    pub supervisor: RuntimeSupervisor,
    pub capabilities: CapabilityEvidenceRegistry,
    pub revision: u64,
}

#[derive(Clone)]
pub struct SqliteSessionRepository {
    database: Arc<DatabaseActor>,
    workspace_id: String,
}

/// App-lifetime session repository handle that remains fail-closed until the
/// active Workspace database actor is explicitly bound. This lets the runtime
/// coordinator exist at application startup without moving Chat authority into
/// the app database or inventing an in-memory durable fallback.
#[derive(Clone, Default)]
pub struct DeferredSessionRepository {
    bound: Arc<RwLock<Option<SqliteSessionRepository>>>,
}

pub(crate) struct PreparedWorkspaceBinding<'a> {
    bound: RwLockWriteGuard<'a, Option<SqliteSessionRepository>>,
    replacement: Option<SqliteSessionRepository>,
}

impl PreparedWorkspaceBinding<'_> {
    pub(crate) fn workspace_id(&self) -> &str {
        &self
            .replacement
            .as_ref()
            .expect("prepared binding retains its replacement")
            .workspace_id
    }

    pub(crate) fn commit(mut self) {
        *self.bound = self.replacement.take();
    }
}

impl DeferredSessionRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_workspace(
        &self,
        database: Arc<DatabaseActor>,
    ) -> Result<(), RuntimePersistenceError> {
        self.prepare_workspace_binding(database)?.commit();
        Ok(())
    }

    pub(crate) fn prepare_workspace_binding(
        &self,
        database: Arc<DatabaseActor>,
    ) -> Result<PreparedWorkspaceBinding<'_>, RuntimePersistenceError> {
        let replacement = SqliteSessionRepository::new(database)
            .map_err(|_| RuntimePersistenceError::WrongDatabase)?;
        let bound = self
            .bound
            .write()
            .map_err(|_| RuntimePersistenceError::SessionRepositoryUnavailable)?;
        if let Some(current) = bound.as_ref()
            && current.database.descriptor() != replacement.database.descriptor()
        {
            return Err(RuntimePersistenceError::WorkspaceAlreadyBound);
        }
        Ok(PreparedWorkspaceBinding {
            bound,
            replacement: Some(replacement),
        })
    }

    pub fn unbind_workspace(&self, workspace_id: &str) -> Result<(), RuntimePersistenceError> {
        let mut bound = self
            .bound
            .write()
            .map_err(|_| RuntimePersistenceError::SessionRepositoryUnavailable)?;
        match bound.as_ref() {
            Some(current) if current.workspace_id == workspace_id => {
                *bound = None;
                Ok(())
            }
            Some(_) => Err(RuntimePersistenceError::WorkspaceBindingMismatch),
            None => Ok(()),
        }
    }

    pub fn bound_workspace_id(&self) -> Result<Option<String>, RuntimePersistenceError> {
        self.bound
            .read()
            .map(|bound| {
                bound
                    .as_ref()
                    .map(|repository| repository.workspace_id.clone())
            })
            .map_err(|_| RuntimePersistenceError::SessionRepositoryUnavailable)
    }

    /// Returns the core-owned working-copy root for production runtime
    /// composition. The root is derived from the already-bound Workspace
    /// database descriptor; no renderer-supplied filesystem path participates.
    pub fn bound_workspace_root(
        &self,
    ) -> Result<Option<std::path::PathBuf>, RuntimePersistenceError> {
        Ok(self
            .bound_workspace_binding()?
            .map(|(_, workspace_root)| workspace_root))
    }

    pub fn bound_workspace_binding(
        &self,
    ) -> Result<Option<(String, std::path::PathBuf)>, RuntimePersistenceError> {
        self.bound
            .read()
            .map_err(|_| RuntimePersistenceError::SessionRepositoryUnavailable)?
            .as_ref()
            .map(|repository| {
                let workspace_root = repository
                    .database
                    .descriptor()
                    .path
                    .parent()
                    .and_then(std::path::Path::parent)
                    .map(std::path::Path::to_path_buf)
                    .ok_or(RuntimePersistenceError::WrongDatabase)?;
                Ok((repository.workspace_id.clone(), workspace_root))
            })
            .transpose()
    }

    pub(crate) fn bound_active_project_exists(
        &self,
        project_id: &str,
    ) -> Result<bool, RuntimePersistenceError> {
        let repository = self
            .repository()
            .map_err(|_| RuntimePersistenceError::SessionRepositoryUnavailable)?;
        Ok(repository.database.active_project_exists(project_id)?)
    }

    fn repository(&self) -> Result<SqliteSessionRepository, SessionRepositoryError> {
        self.bound
            .read()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .clone()
            .ok_or(SessionRepositoryError::Unavailable)
    }
}

impl SessionRepository for DeferredSessionRepository {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError> {
        self.repository()?.load(session_id)
    }

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError> {
        self.repository()?.list()
    }

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError> {
        self.repository()?.create(record)
    }

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError> {
        self.repository()?
            .compare_and_swap(session_id, expected_revision, replacement)
    }
}

impl SqliteSessionRepository {
    pub fn new(database: Arc<DatabaseActor>) -> Result<Self, SessionRepositoryError> {
        let workspace_id = match &database.descriptor().kind {
            DatabaseKind::Workspace { workspace_id } => workspace_id.clone(),
            DatabaseKind::App => return Err(SessionRepositoryError::Unavailable),
        };
        Ok(Self {
            database,
            workspace_id,
        })
    }

    fn encode_for_write(
        &self,
        record: &SessionRecord,
    ) -> Result<(WorkspaceSessionDocumentRecord, String), SessionRepositoryError> {
        record
            .validate()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let binding = record
            .binding()
            .ok_or(SessionRepositoryError::Unavailable)?;
        if binding.workspace_id != self.workspace_id {
            return Err(SessionRepositoryError::Unavailable);
        }
        let active_project_id = binding
            .project_id
            .as_ref()
            .filter(|project_id| !project_id.trim().is_empty())
            .cloned()
            .ok_or(SessionRepositoryError::Unavailable)?;
        let canonical_document =
            serde_json::to_string(record).map_err(|_| SessionRepositoryError::Unavailable)?;
        Ok((
            WorkspaceSessionDocumentRecord {
                workspace_id: self.workspace_id.clone(),
                session_id: record.session_id.clone(),
                revision: record.revision,
                canonical_document,
                updated_at_ms: record.updated_at_ms,
            },
            active_project_id,
        ))
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError> {
        let Some(document) = self
            .database
            .session_document(session_id)
            .map_err(map_session_database_error)?
        else {
            return Ok(None);
        };
        let record: SessionRecord = serde_json::from_str(&document.canonical_document)
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        record
            .validate()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        if record.session_id != document.session_id
            || record.revision != document.revision
            || record.updated_at_ms != document.updated_at_ms
        {
            return Err(SessionRepositoryError::Unavailable);
        }
        Ok(Some(record))
    }

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError> {
        self.database
            .session_documents()
            .map_err(map_session_database_error)?
            .into_iter()
            .map(|document| {
                let record: SessionRecord = serde_json::from_str(&document.canonical_document)
                    .map_err(|_| SessionRepositoryError::Unavailable)?;
                record
                    .validate()
                    .map_err(|_| SessionRepositoryError::Unavailable)?;
                if record.session_id != document.session_id
                    || record.revision != document.revision
                    || record.updated_at_ms != document.updated_at_ms
                {
                    return Err(SessionRepositoryError::Unavailable);
                }
                Ok(record)
            })
            .collect()
    }

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError> {
        let (document, active_project_id) = self.encode_for_write(record)?;
        let title = record
            .title
            .as_ref()
            .filter(|title| !title.trim().is_empty())
            .cloned()
            .ok_or(SessionRepositoryError::Unavailable)?;
        let created_at =
            i64::try_from(record.created_at_ms).map_err(|_| SessionRepositoryError::Unavailable)?;
        let updated_at =
            i64::try_from(record.updated_at_ms).map_err(|_| SessionRepositoryError::Unavailable)?;
        self.database
            .promote_session_document(
                document,
                ChatRecord {
                    workspace_id: self.workspace_id.clone(),
                    project_id: active_project_id,
                    chat_id: record.session_id.clone(),
                    title,
                    created_at,
                    updated_at,
                    lifecycle_state: LifecycleState::Active,
                    inactivated_at: None,
                },
            )
            .map(|_| ())
            .map_err(map_session_database_error)
    }

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError> {
        if replacement.session_id != session_id {
            return Err(SessionRepositoryError::Conflict);
        }
        let (document, active_project_id) = self.encode_for_write(replacement)?;
        self.database
            .compare_and_swap_session_document(document, expected_revision, active_project_id)
            .map(|_| ())
            .map_err(map_session_database_error)
    }
}

fn map_session_database_error(error: DatabaseError) -> SessionRepositoryError {
    match error {
        DatabaseError::Conflict(_) => SessionRepositoryError::Conflict,
        _ => SessionRepositoryError::Unavailable,
    }
}

#[derive(Clone)]
pub struct ProviderStateStore {
    database: Arc<DatabaseActor>,
}

impl ProviderStateStore {
    pub fn new(database: Arc<DatabaseActor>) -> Result<Self, RuntimePersistenceError> {
        if !matches!(database.descriptor().kind, DatabaseKind::App) {
            return Err(RuntimePersistenceError::WrongDatabase);
        }
        Ok(Self { database })
    }

    pub fn save(
        &self,
        service: &ProviderService,
        expected_generation: Option<u64>,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        self.save_snapshot(&service.snapshot(), expected_generation, updated_at_ms)
    }

    pub fn save_snapshot(
        &self,
        snapshot: &ProviderSnapshot,
        expected_generation: Option<u64>,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        let canonical_document =
            serde_json::to_string(snapshot).map_err(|_| RuntimePersistenceError::Serialization)?;
        self.database
            .save_runtime_state_document(
                RuntimeStateDocumentRecord {
                    document_kind: PROVIDER_DOCUMENT_KIND.into(),
                    document_id: PROVIDER_DOCUMENT_ID.into(),
                    generation: snapshot.generation,
                    canonical_document,
                    updated_at_ms,
                },
                expected_generation,
            )
            .map_err(RuntimePersistenceError::Database)
    }

    pub fn load(&self) -> Result<Option<ProviderService>, RuntimePersistenceError> {
        let Some(document) = self
            .database
            .runtime_state_document(PROVIDER_DOCUMENT_KIND, PROVIDER_DOCUMENT_ID)
            .map_err(RuntimePersistenceError::Database)?
        else {
            return Ok(None);
        };
        let snapshot: ProviderSnapshot = serde_json::from_str(&document.canonical_document)
            .map_err(|_| RuntimePersistenceError::Serialization)?;
        if snapshot.generation != document.generation {
            return Err(RuntimePersistenceError::GenerationMismatch);
        }
        ProviderService::restore(snapshot)
            .map(Some)
            .map_err(RuntimePersistenceError::Provider)
    }
}

#[derive(Clone)]
pub struct SupervisorStateStore {
    database: Arc<DatabaseActor>,
}

impl SupervisorStateStore {
    pub fn new(database: Arc<DatabaseActor>) -> Result<Self, RuntimePersistenceError> {
        if !matches!(database.descriptor().kind, DatabaseKind::App) {
            return Err(RuntimePersistenceError::WrongDatabase);
        }
        Ok(Self { database })
    }

    pub fn save(
        &self,
        supervisor: &RuntimeSupervisor,
        expected_generation: Option<u64>,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        self.save_snapshot(&supervisor.snapshot(), expected_generation, updated_at_ms)
    }

    pub fn save_snapshot(
        &self,
        snapshot: &SupervisorSnapshot,
        expected_generation: Option<u64>,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        let canonical_document =
            serde_json::to_string(snapshot).map_err(|_| RuntimePersistenceError::Serialization)?;
        self.database
            .save_runtime_state_document(
                RuntimeStateDocumentRecord {
                    document_kind: SUPERVISOR_DOCUMENT_KIND.into(),
                    document_id: SUPERVISOR_DOCUMENT_ID.into(),
                    generation: snapshot.state_generation,
                    canonical_document,
                    updated_at_ms,
                },
                expected_generation,
            )
            .map_err(RuntimePersistenceError::Database)
    }

    pub fn load(
        &self,
        requirements: impl IntoIterator<Item = CompatibilityRequirement>,
    ) -> Result<Option<RuntimeSupervisor>, RuntimePersistenceError> {
        let Some(document) = self
            .database
            .runtime_state_document(SUPERVISOR_DOCUMENT_KIND, SUPERVISOR_DOCUMENT_ID)
            .map_err(RuntimePersistenceError::Database)?
        else {
            return Ok(None);
        };
        let snapshot: SupervisorSnapshot = serde_json::from_str(&document.canonical_document)
            .map_err(|_| RuntimePersistenceError::Serialization)?;
        if snapshot.state_generation != document.generation {
            return Err(RuntimePersistenceError::GenerationMismatch);
        }
        let persisted_generation = snapshot.state_generation;
        let supervisor = RuntimeSupervisor::restore(requirements, snapshot)
            .map_err(RuntimePersistenceError::Supervisor)?;
        let recovered = supervisor.snapshot();
        if recovered.state_generation > persisted_generation {
            let canonical_document = serde_json::to_string(&recovered)
                .map_err(|_| RuntimePersistenceError::Serialization)?;
            self.database
                .save_runtime_state_document(
                    RuntimeStateDocumentRecord {
                        document_kind: SUPERVISOR_DOCUMENT_KIND.into(),
                        document_id: SUPERVISOR_DOCUMENT_ID.into(),
                        generation: recovered.state_generation,
                        canonical_document,
                        updated_at_ms: document.updated_at_ms.saturating_add(1),
                    },
                    Some(persisted_generation),
                )
                .map_err(RuntimePersistenceError::Database)?;
        }
        Ok(Some(supervisor))
    }
}

#[derive(Clone)]
pub struct RuntimeControlPlaneStore {
    database: Arc<DatabaseActor>,
}

impl RuntimeControlPlaneStore {
    pub fn new(database: Arc<DatabaseActor>) -> Result<Self, RuntimePersistenceError> {
        if !matches!(database.descriptor().kind, DatabaseKind::App) {
            return Err(RuntimePersistenceError::WrongDatabase);
        }
        Ok(Self { database })
    }

    pub fn save(
        &self,
        supervisor: &RuntimeSupervisor,
        capabilities: &CapabilityEvidenceRegistry,
        expected_revision: u64,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        self.save_snapshots(
            supervisor.snapshot(),
            capabilities.snapshot(),
            expected_revision,
            updated_at_ms,
        )
    }

    pub fn save_snapshots(
        &self,
        supervisor: SupervisorSnapshot,
        capabilities: CapabilityEvidenceSnapshot,
        expected_revision: u64,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimePersistenceError> {
        let revision = expected_revision
            .checked_add(1)
            .ok_or(RuntimePersistenceError::GenerationMismatch)?;
        let snapshot = RuntimeControlPlaneSnapshot {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            revision,
            supervisor,
            capabilities,
        };
        let canonical_document =
            serde_json::to_string(&snapshot).map_err(|_| RuntimePersistenceError::Serialization)?;
        self.database
            .save_runtime_state_document(
                RuntimeStateDocumentRecord {
                    document_kind: CONTROL_PLANE_DOCUMENT_KIND.into(),
                    document_id: CONTROL_PLANE_DOCUMENT_ID.into(),
                    generation: revision,
                    canonical_document,
                    updated_at_ms,
                },
                (expected_revision != 0).then_some(expected_revision),
            )
            .map_err(RuntimePersistenceError::Database)?;
        Ok(revision)
    }

    pub fn load(
        &self,
        requirements: impl IntoIterator<Item = CompatibilityRequirement>,
    ) -> Result<Option<RestoredRuntimeControlPlane>, RuntimePersistenceError> {
        let Some(document) = self
            .database
            .runtime_state_document(CONTROL_PLANE_DOCUMENT_KIND, CONTROL_PLANE_DOCUMENT_ID)
            .map_err(RuntimePersistenceError::Database)?
        else {
            return Ok(None);
        };
        let snapshot: RuntimeControlPlaneSnapshot =
            serde_json::from_str(&document.canonical_document)
                .map_err(|_| RuntimePersistenceError::Serialization)?;
        if snapshot.schema_version != CONTROL_PLANE_SCHEMA_VERSION
            || snapshot.revision != document.generation
        {
            return Err(RuntimePersistenceError::GenerationMismatch);
        }
        let original_supervisor = snapshot.supervisor.clone();
        let original_capabilities = snapshot.capabilities.clone();
        let supervisor = RuntimeSupervisor::restore(requirements, snapshot.supervisor)
            .map_err(RuntimePersistenceError::Supervisor)?;
        let capabilities = CapabilityEvidenceRegistry::restore(snapshot.capabilities)
            .map_err(RuntimePersistenceError::CapabilityEvidence)?;
        let recovered = supervisor.snapshot() != original_supervisor
            || capabilities.snapshot() != original_capabilities;
        let revision = if recovered {
            self.save(
                &supervisor,
                &capabilities,
                snapshot.revision,
                document.updated_at_ms.saturating_add(1),
            )?
        } else {
            snapshot.revision
        };
        Ok(Some(RestoredRuntimeControlPlane {
            supervisor,
            capabilities,
            revision,
        }))
    }
}

#[derive(Debug, Error)]
pub enum RuntimePersistenceError {
    #[error("runtime state store requires the app database")]
    WrongDatabase,
    #[error("a different Workspace database is already bound")]
    WorkspaceAlreadyBound,
    #[error("the requested Workspace does not own the current binding")]
    WorkspaceBindingMismatch,
    #[error("the deferred session repository is unavailable")]
    SessionRepositoryUnavailable,
    #[error("runtime state serialization failed")]
    Serialization,
    #[error("runtime state document generation mismatches its payload")]
    GenerationMismatch,
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
    #[error(transparent)]
    CapabilityEvidence(#[from] CapabilityEvidenceError),
    #[error(transparent)]
    Session(#[from] SessionError),
}
