//! Cohesive cross-module service integration owned by the Rust core.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::configuration::{
    ConfigurationError, ConfigurationScope, ConfigurationService, ConfigurationUpdate,
    ConfigurationWatcherNotice, ConfigurationWatcherPlan, EffectiveConfigurationSnapshot,
    LastKnownGoodDocument, MAX_CONFIGURATION_BYTES, ManagedCeilings,
    ParentDirectoryConfigurationWatcher, SecurityConstraints, WatchedConfiguration,
    compensate_scope_write, recover_missing_scope_file,
    resolve_effective_snapshot_from_last_known_good, stable_scope_text, validate_scope_document,
};
use super::database::{
    ChatRecord, ConfigurationSnapshotRecord, DatabaseActor, DatabaseDescriptor, DatabaseError,
    DatabaseSnapshot, InactiveEntity, LifecycleState, ProjectPathState, ProjectRecord,
    RecentWorkspaceRecord, SnapshotQuery, WorkspaceRecord, WorkspaceSnapshot,
    inspect_complete_workspace_database_read_only,
};
use super::workspace::{
    ArchiveLimits, C4osHomeLayout, PendingWorkspaceSave, PreparedOpenWorkspaceOutcome,
    ProjectReference, ReadOnlyWorkspace, SavedWorkspace, WORKSPACE_SCHEMA_VERSION, WorkspaceError,
    WorkspaceLayout, WorkspaceLockOwner, WorkspaceManifest, WorkspaceResult,
    WorkspaceSemanticValidationTarget, WorkspaceWriterLock, WritableWorkspace, WriterAccess,
    acquire_workspace_writer_lock, create_untitled_working_copy,
    persist_canonical_working_manifest, preflight_workspace_archive_source,
    prepare_open_workspace_archive, prepare_workspace_archive_save,
};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationPersistenceError {
    #[error("configuration operation failed: {0}")]
    Configuration(#[from] ConfigurationError),
    #[error("configuration persistence failed: {0}")]
    Database(#[from] DatabaseError),
    #[error("persisted configuration recovery is invalid")]
    InvalidRecovery,
    #[error("configuration scope is not owned by this persistence service")]
    InvalidScope,
    #[error("configuration rollback requires recovery before another activation")]
    RecoveryRequired,
    #[error("configuration policy activation failed")]
    PolicyActivation,
    #[error("managed configuration state is unavailable")]
    CoordinatorUnavailable,
}

/// Long-lived app configuration authority. The same service instance owns UI
/// writes, external-file activation, SQLite LKG publication, and watcher
/// deduplication for the lifetime of the Tauri application.
pub struct ManagedAppConfiguration {
    database: Arc<DatabaseActor>,
    home: C4osHomeLayout,
    service: Arc<Mutex<ConfigurationService>>,
    _watcher: ParentDirectoryConfigurationWatcher,
    last_error: Arc<Mutex<Option<&'static str>>>,
    activation_coordinator: Arc<Mutex<Option<AppConfigurationActivationCoordinator>>>,
}

#[derive(Clone)]
struct AppConfigurationActivationCoordinator {
    gate: Arc<Mutex<()>>,
    observer: Arc<dyn Fn(Option<LastKnownGoodDocument>) -> Result<(), ()> + Send + Sync>,
}

impl ManagedAppConfiguration {
    pub fn start(
        database: Arc<DatabaseActor>,
        home: C4osHomeLayout,
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Result<Self, ConfigurationPersistenceError> {
        let service = Arc::new(Mutex::new(restore_app_configuration(
            &database,
            &home,
            managed_ceilings,
            security_constraints,
        )?));
        let last_error = Arc::new(Mutex::new(None));
        let activation_coordinator =
            Arc::new(Mutex::new(None::<AppConfigurationActivationCoordinator>));
        let plan = ConfigurationWatcherPlan::new([WatchedConfiguration {
            scope: ConfigurationScope::App,
            path: home.app_configuration(),
        }])
        .map_err(configuration_io)?;

        let callback_service = Arc::clone(&service);
        let callback_database = Arc::clone(&database);
        let callback_home = home.clone();
        let callback_error = Arc::clone(&last_error);
        let callback_activation = Arc::clone(&activation_coordinator);
        let watcher =
            ParentDirectoryConfigurationWatcher::start(plan, move |notice| match notice {
                ConfigurationWatcherNotice::Changed(targets) => {
                    for target in targets {
                        let activation = callback_activation
                            .lock()
                            .ok()
                            .and_then(|coordinator| coordinator.clone());
                        let _activation_guard = match activation.as_ref() {
                            Some(coordinator) => match coordinator.gate.lock() {
                                Ok(guard) => Some(guard),
                                Err(_) => {
                                    if let Ok(mut error) = callback_error.lock() {
                                        *error =
                                            Some("configuration policy activation gate failed");
                                    }
                                    continue;
                                }
                            },
                            None => None,
                        };
                        let result = callback_service
                            .lock()
                            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)
                            .and_then(|mut configuration| {
                                let prior = configuration
                                    .last_known_good(ConfigurationScope::App)
                                    .cloned();
                                let update = reconcile_external_app_configuration(
                                    &callback_database,
                                    &mut configuration,
                                    &target.path,
                                    current_unix_seconds(),
                                )?;
                                Ok((update, prior))
                            });
                        let mut observer_failed = false;
                        if let Ok((ConfigurationUpdate::Activated { snapshot, record }, prior)) =
                            &result
                            && let Some(activation) = activation.as_ref()
                            && (activation.observer)(Some(record.clone())).is_err()
                        {
                            observer_failed = true;
                            let rollback_text = prior
                                .as_ref()
                                .map(|record| record.canonical_toml.clone())
                                .or_else(|| {
                                    toml::to_string(
                                        &super::configuration::ConfigurationDocument::default(),
                                    )
                                    .ok()
                                });
                            let compensated = rollback_text.is_some_and(|rollback_text| {
                                callback_service.lock().is_ok_and(|mut configuration| {
                                    save_app_configuration(
                                        &callback_database,
                                        &callback_home,
                                        &mut configuration,
                                        &rollback_text,
                                        snapshot.generation,
                                        current_unix_seconds(),
                                    )
                                    .is_ok()
                                })
                            });
                            let policy_compensated =
                                compensated && (activation.observer)(prior.clone()).is_ok();
                            if let Ok(mut error) = callback_error.lock() {
                                *error = Some(if policy_compensated {
                                    "configuration policy activation failed and was rolled back"
                                } else {
                                    "configuration policy activation failed; recovery is required"
                                });
                            }
                        }
                        if let Ok(mut error) = callback_error.lock() {
                            match &result {
                                Err(_) => *error = Some("configuration watcher activation failed"),
                                Ok((ConfigurationUpdate::Rejected { .. }, _)) => {
                                    *error = Some("configuration watcher edit was rejected")
                                }
                                Ok((
                                    ConfigurationUpdate::Activated { .. }
                                    | ConfigurationUpdate::Unchanged { .. }
                                    | ConfigurationUpdate::DeduplicatedSelfWrite { .. },
                                    _,
                                )) if !observer_failed => *error = None,
                                Ok(_) => {}
                            }
                        }
                    }
                }
                ConfigurationWatcherNotice::Error(_) => {
                    if let Ok(mut error) = callback_error.lock() {
                        *error = Some("configuration watcher failed");
                    }
                }
            })?;

        // Close the setup race between the initial stable read and watcher
        // registration. A self-write is deduplicated by the retained service.
        {
            let mut configuration = service
                .lock()
                .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
            reconcile_external_app_configuration(
                &database,
                &mut configuration,
                &home.app_configuration(),
                current_unix_seconds(),
            )?;
        }

        Ok(Self {
            database,
            home,
            service,
            _watcher: watcher,
            last_error,
            activation_coordinator,
        })
    }

    pub fn snapshot(
        &self,
    ) -> Result<EffectiveConfigurationSnapshot, ConfigurationPersistenceError> {
        self.service
            .lock()
            .map(|configuration| configuration.snapshot())
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)
    }

    pub fn last_known_good(
        &self,
    ) -> Result<Option<LastKnownGoodDocument>, ConfigurationPersistenceError> {
        self.service
            .lock()
            .map(|configuration| {
                configuration
                    .last_known_good(ConfigurationScope::App)
                    .cloned()
            })
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)
    }

    pub fn save(
        &self,
        text: &str,
        base_generation: u64,
        activated_at: i64,
    ) -> Result<ConfigurationUpdate, ConfigurationPersistenceError> {
        let mut configuration = self
            .service
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        save_app_configuration(
            &self.database,
            &self.home,
            &mut configuration,
            text,
            base_generation,
            activated_at,
        )
    }

    pub fn last_error(&self) -> Option<&'static str> {
        self.last_error.lock().ok().and_then(|error| *error)
    }

    pub fn set_activation_observer(
        &self,
        gate: Arc<Mutex<()>>,
        observer: Arc<dyn Fn(Option<LastKnownGoodDocument>) -> Result<(), ()> + Send + Sync>,
    ) -> Result<(), ConfigurationPersistenceError> {
        *self
            .activation_coordinator
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)? =
            Some(AppConfigurationActivationCoordinator {
                gate: Arc::clone(&gate),
                observer: Arc::clone(&observer),
            });
        let _guard = gate
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        observer(self.last_known_good()?)
            .map_err(|_| ConfigurationPersistenceError::RecoveryRequired)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WorkspaceConfigurationIdentity {
    scope: ConfigurationScope,
    scope_id: Uuid,
}

impl WorkspaceConfigurationIdentity {
    fn persisted_scope_id(self) -> String {
        self.scope_id.to_string()
    }
}

struct WorkspaceConfigurationCoordinatorState {
    services: BTreeMap<WorkspaceConfigurationIdentity, ConfigurationService>,
    global_generation: u64,
}

/// Long-lived configuration authority retained by every writable Workspace.
/// It serializes every scope identity through one generation allocator while
/// watcher callbacks and UI saves publish to the same SQLite LKG store.
struct ManagedWorkspaceConfiguration {
    database: Arc<DatabaseActor>,
    root: PathBuf,
    workspace_id: Uuid,
    state: Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    watcher: Mutex<ParentDirectoryConfigurationWatcher>,
    last_error: Arc<Mutex<Option<&'static str>>>,
    activation_coordinator: Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
}

#[derive(Clone)]
struct WorkspaceConfigurationActivationCoordinator {
    gate: Arc<Mutex<()>>,
    observer: Arc<dyn Fn() -> Result<(), ()> + Send + Sync>,
}

impl ManagedWorkspaceConfiguration {
    fn start(
        database: Arc<DatabaseActor>,
        root: PathBuf,
        workspace_id: Uuid,
    ) -> Result<Self, ConfigurationPersistenceError> {
        let (snapshot, _) = database.complete_workspace_snapshot(true)?;
        let identities = workspace_configuration_identities(&snapshot, workspace_id)?;
        ensure_workspace_configuration_parents(&root, &identities)?;
        let state = Arc::new(Mutex::new(workspace_configuration_state(
            &root,
            workspace_id,
            &snapshot,
            &identities,
        )?));
        let last_error = Arc::new(Mutex::new(None));
        let activation_coordinator = Arc::new(Mutex::new(
            None::<WorkspaceConfigurationActivationCoordinator>,
        ));
        let watcher = start_workspace_configuration_watcher(
            Arc::clone(&database),
            root.clone(),
            workspace_id,
            Arc::clone(&state),
            Arc::clone(&last_error),
            Arc::clone(&activation_coordinator),
            &identities,
        )?;
        Ok(Self {
            database,
            root,
            workspace_id,
            state,
            watcher: Mutex::new(watcher),
            last_error,
            activation_coordinator,
        })
    }

    fn refresh_targets(&self) -> Result<(), ConfigurationPersistenceError> {
        let (snapshot, _) = self.database.complete_workspace_snapshot(true)?;
        let identities = workspace_configuration_identities(&snapshot, self.workspace_id)?;
        ensure_workspace_configuration_parents(&self.root, &identities)?;
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
            synchronize_workspace_configuration_services(
                &mut state,
                &self.root,
                self.workspace_id,
                &snapshot,
                &identities,
            )?;
        }
        let replacement = start_workspace_configuration_watcher(
            Arc::clone(&self.database),
            self.root.clone(),
            self.workspace_id,
            Arc::clone(&self.state),
            Arc::clone(&self.last_error),
            Arc::clone(&self.activation_coordinator),
            &identities,
        )?;
        let mut watcher = self
            .watcher
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        *watcher = replacement;
        Ok(())
    }

    /// Refreshing watcher coverage is ancillary once an operation has made a
    /// durable commit. Preserve the committed success and surface a degraded
    /// watcher diagnostic so a later refresh can repair coverage without
    /// falsely telling the caller that the state change failed.
    fn refresh_targets_after_commit(&self) {
        if self.refresh_targets().is_err()
            && let Ok(mut error) = self.last_error.lock()
        {
            *error = Some("Workspace configuration watcher target refresh failed");
        }
    }

    fn save_scope(
        &self,
        identity: WorkspaceConfigurationIdentity,
        text: &str,
        base_generation: u64,
        activated_at: i64,
    ) -> Result<ConfigurationUpdate, ConfigurationPersistenceError> {
        let path = configuration_path(&self.root, identity.scope, &identity.persisted_scope_id())
            .map_err(|_| ConfigurationPersistenceError::InvalidScope)?;
        let activation = self
            .activation_coordinator
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?
            .clone();
        let _activation_guard = activation
            .as_ref()
            .map(|coordinator| {
                coordinator
                    .gate
                    .lock()
                    .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)
            })
            .transpose()?;
        let (prior, update) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
            if base_generation != state.global_generation {
                let global_generation = state.global_generation;
                let service = state
                    .services
                    .get_mut(&identity)
                    .ok_or(ConfigurationPersistenceError::InvalidScope)?;
                service.raise_generation_floor(global_generation);
                return service
                    .save_scope_text(identity.scope, &path, text, base_generation)
                    .map_err(ConfigurationPersistenceError::Configuration);
            }
            let global_generation = state.global_generation;
            let service = state
                .services
                .get_mut(&identity)
                .ok_or(ConfigurationPersistenceError::InvalidScope)?;
            service.raise_generation_floor(global_generation);
            let prior = service.last_known_good(identity.scope).cloned();
            let update = commit_ui_configuration(
                service,
                identity.scope,
                &path,
                text,
                base_generation,
                |record| {
                    self.database
                        .activate_configuration(ConfigurationSnapshotRecord {
                            workspace_id: self.workspace_id.to_string(),
                            scope_kind: identity.scope.as_str().into(),
                            scope_id: identity.persisted_scope_id(),
                            canonical_document: record.canonical_toml.clone(),
                            generation: record.activated_generation,
                            activated_at,
                        })
                        .map(|_| ())
                },
            )?;
            if let ConfigurationUpdate::Activated { record, .. } = &update {
                state.global_generation = record.activated_generation;
            }
            (prior, update)
        };
        if let (ConfigurationUpdate::Activated { snapshot, .. }, Some(activation)) =
            (&update, activation.as_ref())
            && (activation.observer)().is_err()
        {
            let rollback_text = prior
                .as_ref()
                .map(|record| record.canonical_toml.clone())
                .or_else(|| {
                    toml::to_string(&super::configuration::ConfigurationDocument::default()).ok()
                });
            let compensated = rollback_text.is_some_and(|rollback_text| {
                self.state.lock().is_ok_and(|mut state| {
                    compensate_workspace_configuration_activation(
                        &self.database,
                        self.workspace_id,
                        &mut state,
                        identity,
                        &path,
                        &rollback_text,
                        snapshot.generation,
                    )
                    .is_ok()
                })
            });
            let policy_compensated = compensated && (activation.observer)().is_ok();
            if let Ok(mut error) = self.last_error.lock() {
                *error = Some(if policy_compensated {
                    "Workspace configuration policy activation failed and was rolled back"
                } else {
                    "Workspace configuration policy activation failed; recovery is required"
                });
            }
            self.refresh_targets_after_commit();
            return Err(if policy_compensated {
                ConfigurationPersistenceError::PolicyActivation
            } else {
                ConfigurationPersistenceError::RecoveryRequired
            });
        }
        if let Ok(mut error) = self.last_error.lock() {
            *error = None;
        }
        self.refresh_targets_after_commit();
        Ok(update)
    }

    fn set_activation_observer(
        &self,
        gate: Arc<Mutex<()>>,
        observer: Arc<dyn Fn() -> Result<(), ()> + Send + Sync>,
    ) -> Result<(), ConfigurationPersistenceError> {
        *self
            .activation_coordinator
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)? =
            Some(WorkspaceConfigurationActivationCoordinator { gate, observer });
        Ok(())
    }

    fn effective_stack(
        &self,
        project_id: Option<Uuid>,
        chat_id: Option<Uuid>,
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Result<ConfigurationService, ConfigurationPersistenceError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        let mut records = Vec::new();
        for identity in [
            Some(WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Workspace,
                scope_id: self.workspace_id,
            }),
            project_id.map(|scope_id| WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Project,
                scope_id,
            }),
            chat_id.map(|scope_id| WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Chat,
                scope_id,
            }),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(record) = state
                .services
                .get(&identity)
                .and_then(|service| service.last_known_good(identity.scope))
            {
                records.push(record.clone());
            }
        }
        let mut service = ConfigurationService::from_last_known_good(
            records,
            managed_ceilings,
            security_constraints,
        )
        .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?;
        service.raise_generation_floor(state.global_generation);
        Ok(service)
    }

    fn last_error(&self) -> Option<&'static str> {
        self.last_error.lock().ok().and_then(|error| *error)
    }
}

fn workspace_configuration_state(
    root: &Path,
    workspace_id: Uuid,
    snapshot: &WorkspaceSnapshot,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<WorkspaceConfigurationCoordinatorState, ConfigurationPersistenceError> {
    let mut state = WorkspaceConfigurationCoordinatorState {
        services: BTreeMap::new(),
        global_generation: 0,
    };
    synchronize_workspace_configuration_services(
        &mut state,
        root,
        workspace_id,
        snapshot,
        identities,
    )?;
    Ok(state)
}

fn synchronize_workspace_configuration_services(
    state: &mut WorkspaceConfigurationCoordinatorState,
    root: &Path,
    workspace_id: Uuid,
    snapshot: &WorkspaceSnapshot,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<(), ConfigurationPersistenceError> {
    let mut persisted = BTreeMap::new();
    for record in &snapshot.configurations {
        let identity =
            workspace_configuration_identity(workspace_id, &record.scope_kind, &record.scope_id)?;
        let path = configuration_path(root, identity.scope, &record.scope_id)
            .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?;
        let lkg = LastKnownGoodDocument {
            scope: identity.scope,
            path,
            canonical_toml: record.canonical_document.clone(),
            activated_generation: record.generation,
            changed_keys: BTreeSet::new(),
        };
        if persisted.insert(identity, lkg).is_some() {
            return Err(ConfigurationPersistenceError::InvalidRecovery);
        }
        state.global_generation = state.global_generation.max(record.generation);
    }
    for identity in identities {
        if let Some(service) = state.services.get(identity) {
            match (
                service.last_known_good(identity.scope),
                persisted.remove(identity),
            ) {
                (Some(active), Some(durable))
                    if active.canonical_toml == durable.canonical_toml
                        && active.activated_generation == durable.activated_generation => {}
                (None, None) => {}
                _ => return Err(ConfigurationPersistenceError::InvalidRecovery),
            }
            continue;
        }
        let service = if let Some(record) = persisted.remove(identity) {
            ConfigurationService::from_last_known_good(
                [record],
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?
        } else {
            ConfigurationService::new(ManagedCeilings::default(), SecurityConstraints::default())
        };
        state.services.insert(*identity, service);
    }
    if !persisted.is_empty() {
        return Err(ConfigurationPersistenceError::InvalidRecovery);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecentWorkspaceSummary {
    pub workspace_id: String,
    pub display_name: String,
    pub last_opened_at: i64,
    pub is_missing: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceStartState {
    pub generation: u64,
    pub recents: Vec<RecentWorkspaceSummary>,
}

pub struct ActiveWorkspace {
    configuration: ManagedWorkspaceConfiguration,
    database: Arc<DatabaseActor>,
    trusted_projects: BTreeSet<Uuid>,
    // Declared last so the writer lock outlives the watcher and database actor
    // during field destruction.
    workspace: WritableWorkspace,
}

impl ActiveWorkspace {
    pub fn manifest(&self) -> &WorkspaceManifest {
        &self.workspace.manifest
    }

    pub fn working_root(&self) -> &Path {
        &self.workspace.working_root
    }

    pub fn recovery_notice(&self) -> Option<&super::workspace::RecoveryNotice> {
        self.workspace.recovery_notice.as_ref()
    }

    pub fn configuration_last_error(&self) -> Option<&'static str> {
        self.configuration.last_error()
    }

    pub fn set_configuration_activation_observer(
        &self,
        gate: Arc<Mutex<()>>,
        observer: Arc<dyn Fn() -> Result<(), ()> + Send + Sync>,
    ) -> Result<(), ConfigurationPersistenceError> {
        self.configuration.set_activation_observer(gate, observer)
    }

    /// Gives crate-owned production composition a shared, read-only handle to
    /// the exact database actor retained by this active Workspace.
    pub(crate) fn database_actor(&self) -> &Arc<DatabaseActor> {
        &self.database
    }

    pub fn snapshot(&self, query: SnapshotQuery) -> Result<WorkspaceSnapshot, DatabaseError> {
        match self.database.snapshot(query)? {
            DatabaseSnapshot::Workspace(snapshot) => Ok(snapshot),
            DatabaseSnapshot::App(_) => Err(DatabaseError::Validation(
                "active Workspace owns an app database".into(),
            )),
        }
    }

    pub fn is_project_trusted(&self, project_id: Uuid) -> bool {
        self.trusted_projects.contains(&project_id)
    }

    pub fn resolve_project_trust(
        &mut self,
        project_id: Uuid,
        trusted: bool,
    ) -> WorkspaceResult<()> {
        if !self
            .database
            .complete_workspace_snapshot(false)
            .map_err(database_conflict)?
            .0
            .projects
            .iter()
            .any(|project| project.project_id == project_id.to_string())
        {
            return Err(WorkspaceError::Conflict(
                "Project trust target does not exist".into(),
            ));
        }
        if trusted {
            self.trusted_projects.insert(project_id);
        } else {
            self.trusted_projects.remove(&project_id);
        }
        Ok(())
    }

    pub fn add_project(
        &mut self,
        project_folder: &Path,
        display_name: &str,
        trusted_picker_grant: bool,
    ) -> WorkspaceResult<u64> {
        if !project_folder.is_dir() {
            return Err(WorkspaceError::InvalidProject(
                "Project folder is unavailable".into(),
            ));
        }
        let reference = ProjectReference::from_folder(project_folder, display_name)?;
        let (snapshot, _) = self
            .database
            .complete_workspace_snapshot(false)
            .map_err(database_conflict)?;
        let generation = self
            .database
            .add_project(ProjectRecord {
                workspace_id: self.workspace.manifest.workspace_id.to_string(),
                project_id: reference.project_id.to_string(),
                display_name: reference.display_name,
                current_path: reference.last_known_path.clone(),
                last_known_path: reference.last_known_path,
                path_state: ProjectPathState::Found,
                position: i64::try_from(snapshot.projects.len()).map_err(|_| {
                    WorkspaceError::Conflict("Project position exceeds its bound".into())
                })?,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .map_err(database_conflict)?;
        self.configuration.refresh_targets_after_commit();
        if trusted_picker_grant {
            self.trusted_projects.insert(reference.project_id);
        }
        Ok(generation)
    }

    pub fn relocate_project(
        &mut self,
        project_id: Uuid,
        project_folder: &Path,
        trusted_picker_grant: bool,
    ) -> WorkspaceResult<u64> {
        if !project_folder.is_absolute() {
            return Err(WorkspaceError::InvalidProject(
                "Project path must be absolute".into(),
            ));
        }
        let path = project_folder
            .to_str()
            .ok_or_else(|| WorkspaceError::InvalidProject("Project path must be UTF-8".into()))?;
        let state = if project_folder.is_dir() {
            ProjectPathState::Relocated
        } else {
            ProjectPathState::Missing
        };
        let generation = self
            .database
            .update_project_path(project_id.to_string(), path, path, state)
            .map_err(database_conflict)?;
        self.trusted_projects.remove(&project_id);
        if trusted_picker_grant && project_folder.is_dir() {
            self.trusted_projects.insert(project_id);
        }
        Ok(generation)
    }

    pub fn reorder_projects(&mut self, ordered: &[Uuid]) -> WorkspaceResult<u64> {
        self.database
            .reorder_projects(ordered.iter().map(Uuid::to_string).collect())
            .map_err(database_conflict)
    }

    pub fn create_chat(
        &mut self,
        project_id: Uuid,
        chat_id: Uuid,
        title: &str,
        created_at: i64,
    ) -> WorkspaceResult<u64> {
        validate_display_name(title)?;
        let generation = self
            .database
            .add_chat(ChatRecord {
                workspace_id: self.workspace.manifest.workspace_id.to_string(),
                project_id: project_id.to_string(),
                chat_id: chat_id.to_string(),
                title: title.to_owned(),
                created_at,
                updated_at: created_at,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .map_err(database_conflict)?;
        self.configuration.refresh_targets_after_commit();
        Ok(generation)
    }

    pub fn save_configuration_scope(
        &mut self,
        configuration: &mut ConfigurationService,
        scope: ConfigurationScope,
        scope_id: Uuid,
        text: &str,
        base_generation: u64,
        activated_at: i64,
    ) -> Result<ConfigurationUpdate, ConfigurationPersistenceError> {
        let workspace_id = self.workspace.manifest.workspace_id;
        let path = match scope {
            ConfigurationScope::Workspace if scope_id == workspace_id => {
                WorkspaceLayout::new(&self.workspace.working_root).workspace_configuration()
            }
            ConfigurationScope::Project => {
                WorkspaceLayout::new(&self.workspace.working_root).project_configuration(scope_id)
            }
            ConfigurationScope::Chat => {
                WorkspaceLayout::new(&self.workspace.working_root).chat_configuration(scope_id)
            }
            ConfigurationScope::App | ConfigurationScope::Workspace => {
                return Err(ConfigurationPersistenceError::InvalidScope);
            }
        };
        self.configuration.save_scope(
            WorkspaceConfigurationIdentity { scope, scope_id },
            text,
            base_generation,
            activated_at,
        )?;
        Ok(configuration.reload_scope(scope, path))
    }

    pub fn restore_configuration_stack(
        &self,
        project_id: Option<Uuid>,
        chat_id: Option<Uuid>,
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Result<ConfigurationService, ConfigurationPersistenceError> {
        self.configuration.effective_stack(
            project_id,
            chat_id,
            managed_ceilings,
            security_constraints,
        )
    }

    pub fn restore_effective_configuration_snapshot(
        &self,
        app: Option<LastKnownGoodDocument>,
        project_id: Option<Uuid>,
        chat_id: Option<Uuid>,
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Result<EffectiveConfigurationSnapshot, ConfigurationPersistenceError> {
        let state = self
            .configuration
            .state
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        let mut records = app.into_iter().collect::<Vec<_>>();
        for identity in [
            Some(WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Workspace,
                scope_id: self.workspace.manifest.workspace_id,
            }),
            project_id.map(|scope_id| WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Project,
                scope_id,
            }),
            chat_id.map(|scope_id| WorkspaceConfigurationIdentity {
                scope: ConfigurationScope::Chat,
                scope_id,
            }),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(record) = state
                .services
                .get(&identity)
                .and_then(|service| service.last_known_good(identity.scope))
            {
                records.push(record.clone());
            }
        }
        drop(state);
        resolve_effective_snapshot_from_last_known_good(
            records,
            managed_ceilings,
            security_constraints,
        )
        .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)
    }

    pub fn inactivate_project(
        &mut self,
        project_id: Uuid,
        inactivated_at: i64,
    ) -> WorkspaceResult<u64> {
        let generation = self
            .database
            .inactivate(
                InactiveEntity::Project {
                    project_id: project_id.to_string(),
                },
                inactivated_at,
            )
            .map_err(database_conflict)?;
        self.trusted_projects.remove(&project_id);
        Ok(generation)
    }

    pub fn inactivate_chat(&mut self, chat_id: Uuid, inactivated_at: i64) -> WorkspaceResult<u64> {
        self.database
            .inactivate(
                InactiveEntity::Chat {
                    chat_id: chat_id.to_string(),
                },
                inactivated_at,
            )
            .map_err(database_conflict)
    }

    pub fn save(
        &mut self,
        app_database: &DatabaseActor,
        archive_path: &Path,
        current_app_version: &str,
        limits: ArchiveLimits,
        saved_at: i64,
    ) -> WorkspaceResult<SavedWorkspace> {
        validate_workspace_semantics(
            &self.workspace.working_root,
            &self.workspace.manifest,
            WorkspaceSemanticValidationTarget::ActiveRecovery,
        )?;
        let (snapshot, _) = self
            .database
            .complete_workspace_snapshot(false)
            .map_err(database_conflict)?;
        let display_name = snapshot
            .workspace
            .as_ref()
            .ok_or_else(|| WorkspaceError::Conflict("Workspace record is inactive".into()))?
            .display_name
            .clone();
        let pending = prepare_save_workspace_with_database(
            &self.database,
            &self.workspace.writer_lock,
            &self.workspace.working_root,
            archive_path,
            &self.workspace.manifest,
            current_app_version,
            limits,
        )?;
        app_database
            .record_recent_workspace(RecentWorkspaceRecord {
                workspace_id: pending.saved().manifest.workspace_id.to_string(),
                display_name,
                archive_path: archive_path.to_string_lossy().into_owned(),
                last_opened_at: saved_at,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .map_err(database_conflict)?;
        let saved = pending.commit();
        self.workspace.manifest = saved.manifest.clone();
        self.workspace.archive_path = archive_path.to_path_buf();
        Ok(saved)
    }

    pub fn inactivate_workspace(
        &mut self,
        app_database: &DatabaseActor,
        inactivated_at: i64,
    ) -> WorkspaceResult<u64> {
        let (before, _) = self
            .database
            .complete_workspace_snapshot(false)
            .map_err(database_conflict)?;
        let prior_workspace = before
            .workspace
            .as_ref()
            .ok_or_else(|| WorkspaceError::Conflict("Workspace record is inactive".into()))?
            .clone();
        let generation = self
            .database
            .inactivate(InactiveEntity::Workspace, inactivated_at)
            .map_err(database_conflict)?;
        if !self.workspace.archive_path.as_os_str().is_empty() {
            match app_database.record_recent_workspace(RecentWorkspaceRecord {
                workspace_id: self.workspace.manifest.workspace_id.to_string(),
                display_name: prior_workspace.display_name.clone(),
                archive_path: self.workspace.archive_path.to_string_lossy().into_owned(),
                last_opened_at: inactivated_at,
                lifecycle_state: LifecycleState::Inactive,
                inactivated_at: Some(inactivated_at),
            }) {
                Ok(_) => {}
                Err(error) => {
                    self.database
                        .compensate_workspace_inactivation(generation, prior_workspace)
                        .map_err(|_| {
                            WorkspaceError::Conflict(
                                "Workspace inactivation compensation failed; recovery is required"
                                    .into(),
                            )
                        })?;
                    return Err(database_conflict(error));
                }
            }
        }
        self.trusted_projects.clear();
        Ok(generation)
    }
}

pub enum WorkspaceServiceOpen {
    Writable(Box<ActiveWorkspace>),
    ReadOnly(ReadOnlyWorkspace),
}

/// Read the bounded, renderer-safe Workspace Start projection. Archive paths
/// remain Rust-owned and are reduced to a missing/present state.
pub fn load_workspace_start_state(
    database: &DatabaseActor,
) -> Result<WorkspaceStartState, DatabaseError> {
    let snapshot = database.snapshot(SnapshotQuery::new(3)?)?;
    let DatabaseSnapshot::App(snapshot) = snapshot else {
        return Err(DatabaseError::Validation(
            "Workspace Start opened a non-app database".into(),
        ));
    };
    let recents = snapshot
        .recents
        .into_iter()
        .map(|recent| RecentWorkspaceSummary {
            workspace_id: recent.workspace_id,
            display_name: recent.display_name,
            last_opened_at: recent.last_opened_at,
            is_missing: !Path::new(&recent.archive_path).is_file(),
        })
        .collect();
    Ok(WorkspaceStartState {
        generation: snapshot.generation,
        recents,
    })
}

pub fn restore_app_configuration(
    database: &DatabaseActor,
    home: &C4osHomeLayout,
    managed_ceilings: ManagedCeilings,
    security_constraints: SecurityConstraints,
) -> Result<ConfigurationService, ConfigurationPersistenceError> {
    let persisted = database.app_configuration_lkg()?;
    let mut configuration = if let Some(persisted) = persisted {
        ConfigurationService::from_last_known_good(
            [LastKnownGoodDocument {
                scope: ConfigurationScope::App,
                path: home.app_configuration(),
                canonical_toml: persisted.canonical_document,
                activated_generation: persisted.generation,
                changed_keys: BTreeSet::new(),
            }],
            managed_ceilings,
            security_constraints,
        )
        .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?
    } else {
        ConfigurationService::new(managed_ceilings, security_constraints)
    };
    reconcile_external_app_configuration(
        database,
        &mut configuration,
        &home.app_configuration(),
        current_unix_seconds(),
    )?;
    Ok(configuration)
}

pub fn save_app_configuration(
    database: &DatabaseActor,
    home: &C4osHomeLayout,
    configuration: &mut ConfigurationService,
    text: &str,
    base_generation: u64,
    activated_at: i64,
) -> Result<ConfigurationUpdate, ConfigurationPersistenceError> {
    commit_ui_configuration(
        configuration,
        ConfigurationScope::App,
        &home.app_configuration(),
        text,
        base_generation,
        |record| {
            database
                .activate_app_configuration(super::database::AppConfigurationSnapshotRecord {
                    canonical_document: record.canonical_toml.clone(),
                    generation: record.activated_generation,
                    activated_at,
                })
                .map(|_| ())
        },
    )
}

fn commit_ui_configuration<F>(
    configuration: &mut ConfigurationService,
    scope: ConfigurationScope,
    path: &Path,
    text: &str,
    base_generation: u64,
    persist: F,
) -> Result<ConfigurationUpdate, ConfigurationPersistenceError>
where
    F: FnOnce(&LastKnownGoodDocument) -> Result<(), DatabaseError>,
{
    let previous_bytes = read_optional_configuration(path)?;
    let checkpoint = configuration.clone();
    let update = configuration.save_scope_text(scope, path, text, base_generation)?;
    let ConfigurationUpdate::Activated { record, .. } = &update else {
        return Ok(update);
    };
    if let Err(error) = persist(record) {
        let compensation = compensate_scope_write(
            scope,
            path,
            record.canonical_toml.as_bytes(),
            previous_bytes.as_deref(),
            base_generation,
            record.activated_generation,
        );
        *configuration = checkpoint;
        if compensation.is_err() {
            configuration.record_persistence_failure(scope, path);
            return Err(ConfigurationPersistenceError::RecoveryRequired);
        }
        return Err(ConfigurationPersistenceError::Database(error));
    }
    Ok(update)
}

fn reconcile_external_app_configuration(
    database: &DatabaseActor,
    configuration: &mut ConfigurationService,
    path: &Path,
    activated_at: i64,
) -> Result<ConfigurationUpdate, ConfigurationPersistenceError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if let Some(record) = configuration
                .last_known_good(ConfigurationScope::App)
                .cloned()
            {
                recover_missing_scope_file(
                    ConfigurationScope::App,
                    path,
                    &record.canonical_toml,
                    record.activated_generation,
                )?;
            }
            Ok(ConfigurationUpdate::Unchanged {
                snapshot: configuration.snapshot(),
            })
        }
        Err(source) => Err(ConfigurationError::Io {
            operation: "inspect configuration",
            path: path.to_path_buf(),
            source,
        }
        .into()),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            Err(ConfigurationError::Io {
                operation: "inspect configuration",
                path: path.to_path_buf(),
                source: io::Error::new(
                    io::ErrorKind::InvalidData,
                    "configuration path must be a regular file",
                ),
            }
            .into())
        }
        Ok(_) => {
            commit_external_configuration(configuration, ConfigurationScope::App, path, |record| {
                database
                    .activate_app_configuration(super::database::AppConfigurationSnapshotRecord {
                        canonical_document: record.canonical_toml.clone(),
                        generation: record.activated_generation,
                        activated_at,
                    })
                    .map(|_| ())
            })
        }
    }
}

fn commit_external_configuration<F>(
    configuration: &mut ConfigurationService,
    scope: ConfigurationScope,
    path: &Path,
    persist: F,
) -> Result<ConfigurationUpdate, ConfigurationPersistenceError>
where
    F: FnOnce(&LastKnownGoodDocument) -> Result<(), DatabaseError>,
{
    let checkpoint = configuration.clone();
    let update = configuration.reload_scope(scope, path);
    match &update {
        ConfigurationUpdate::Activated { record, .. } => {
            if let Err(error) = persist(record) {
                *configuration = checkpoint;
                configuration.record_persistence_failure(scope, path);
                return Err(ConfigurationPersistenceError::Database(error));
            }
        }
        ConfigurationUpdate::Rejected { .. }
        | ConfigurationUpdate::Unchanged { .. }
        | ConfigurationUpdate::DeduplicatedSelfWrite { .. } => {}
    }
    Ok(update)
}

fn workspace_configuration_identities(
    snapshot: &WorkspaceSnapshot,
    workspace_id: Uuid,
) -> Result<BTreeSet<WorkspaceConfigurationIdentity>, ConfigurationPersistenceError> {
    let mut identities = BTreeSet::from([WorkspaceConfigurationIdentity {
        scope: ConfigurationScope::Workspace,
        scope_id: workspace_id,
    }]);
    for project in &snapshot.projects {
        identities.insert(WorkspaceConfigurationIdentity {
            scope: ConfigurationScope::Project,
            scope_id: Uuid::parse_str(&project.project_id)
                .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?,
        });
    }
    for chat in &snapshot.chats {
        identities.insert(WorkspaceConfigurationIdentity {
            scope: ConfigurationScope::Chat,
            scope_id: Uuid::parse_str(&chat.chat_id)
                .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?,
        });
    }
    for record in &snapshot.configurations {
        identities.insert(workspace_configuration_identity(
            workspace_id,
            &record.scope_kind,
            &record.scope_id,
        )?);
    }
    Ok(identities)
}

fn workspace_configuration_identity(
    workspace_id: Uuid,
    scope_kind: &str,
    scope_id: &str,
) -> Result<WorkspaceConfigurationIdentity, ConfigurationPersistenceError> {
    let scope_id =
        Uuid::parse_str(scope_id).map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?;
    let scope = match scope_kind {
        "workspace" if scope_id == workspace_id => ConfigurationScope::Workspace,
        "project" => ConfigurationScope::Project,
        "chat" => ConfigurationScope::Chat,
        _ => return Err(ConfigurationPersistenceError::InvalidRecovery),
    };
    Ok(WorkspaceConfigurationIdentity { scope, scope_id })
}

fn workspace_configuration_identity_for_target(
    root: &Path,
    workspace_id: Uuid,
    target: &WatchedConfiguration,
) -> Result<WorkspaceConfigurationIdentity, ConfigurationPersistenceError> {
    let scope_id = match target.scope {
        ConfigurationScope::Workspace
            if target.path == WorkspaceLayout::new(root).workspace_configuration() =>
        {
            workspace_id
        }
        ConfigurationScope::Project | ConfigurationScope::Chat => target
            .path
            .parent()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str())
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or(ConfigurationPersistenceError::InvalidScope)?,
        ConfigurationScope::App | ConfigurationScope::Workspace => {
            return Err(ConfigurationPersistenceError::InvalidScope);
        }
    };
    Ok(WorkspaceConfigurationIdentity {
        scope: target.scope,
        scope_id,
    })
}

fn ensure_workspace_configuration_parents(
    root: &Path,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<(), ConfigurationPersistenceError> {
    for identity in identities {
        let path = configuration_path(root, identity.scope, &identity.persisted_scope_id())
            .map_err(|_| ConfigurationPersistenceError::InvalidScope)?;
        let parent = path
            .parent()
            .ok_or(ConfigurationPersistenceError::InvalidScope)?;
        fs::create_dir_all(parent).map_err(|source| ConfigurationError::Io {
            operation: "create configuration watcher parent",
            path: parent.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn start_workspace_configuration_watcher(
    database: Arc<DatabaseActor>,
    root: PathBuf,
    workspace_id: Uuid,
    state: Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    last_error: Arc<Mutex<Option<&'static str>>>,
    activation_coordinator: Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<ParentDirectoryConfigurationWatcher, ConfigurationPersistenceError> {
    let targets = identities
        .iter()
        .map(|identity| {
            Ok(WatchedConfiguration {
                scope: identity.scope,
                path: configuration_path(&root, identity.scope, &identity.persisted_scope_id())
                    .map_err(|_| ConfigurationPersistenceError::InvalidScope)?,
            })
        })
        .collect::<Result<Vec<_>, ConfigurationPersistenceError>>()?;
    let plan = ConfigurationWatcherPlan::new(targets).map_err(configuration_io)?;
    ParentDirectoryConfigurationWatcher::start(plan, move |notice| match notice {
        ConfigurationWatcherNotice::Changed(targets) => {
            for target in targets {
                let activation = activation_coordinator
                    .lock()
                    .ok()
                    .and_then(|coordinator| coordinator.clone());
                let _activation_guard = match activation.as_ref() {
                    Some(coordinator) => match coordinator.gate.lock() {
                        Ok(guard) => Some(guard),
                        Err(_) => {
                            if let Ok(mut error) = last_error.lock() {
                                *error =
                                    Some("Workspace configuration policy activation gate failed");
                            }
                            continue;
                        }
                    },
                    None => None,
                };
                let result =
                    workspace_configuration_identity_for_target(&root, workspace_id, &target)
                        .and_then(|identity| {
                            let text = stable_scope_text(identity.scope, &target.path).map_err(
                                |diagnostic| {
                                    if let Ok(mut state) = state.lock()
                                        && let Some(service) = state.services.get_mut(&identity)
                                    {
                                        service.record_diagnostic(diagnostic);
                                    }
                                    ConfigurationPersistenceError::InvalidRecovery
                                },
                            )?;
                            let mut state = state.lock().map_err(|_| {
                                ConfigurationPersistenceError::CoordinatorUnavailable
                            })?;
                            let prior = state.services.get(&identity).and_then(|service| {
                                service.last_known_good(identity.scope).cloned()
                            });
                            let update = commit_external_workspace_configuration(
                                &mut state,
                                identity,
                                &target.path,
                                &text,
                                |record| {
                                    database
                                        .activate_configuration(ConfigurationSnapshotRecord {
                                            workspace_id: workspace_id.to_string(),
                                            scope_kind: identity.scope.as_str().into(),
                                            scope_id: identity.persisted_scope_id(),
                                            canonical_document: record.canonical_toml.clone(),
                                            generation: record.activated_generation,
                                            activated_at: current_unix_seconds(),
                                        })
                                        .map(|_| ())
                                },
                            )?;
                            Ok((identity, prior, update))
                        });
                let policy_result = match (&result, activation.as_ref()) {
                    (Ok((_, _, ConfigurationUpdate::Activated { .. })), Some(coordinator)) => {
                        (coordinator.observer)().map(Some)
                    }
                    _ => Ok(None),
                };
                let compensated = if policy_result.is_err() {
                    result
                        .as_ref()
                        .ok()
                        .and_then(|(identity, prior, update)| {
                            let ConfigurationUpdate::Activated { snapshot, .. } = update else {
                                return None;
                            };
                            Some((*identity, prior.clone(), snapshot.generation))
                        })
                        .is_some_and(|(identity, prior, base_generation)| {
                            let rollback_text = prior
                                .as_ref()
                                .map(|record| record.canonical_toml.clone())
                                .or_else(|| {
                                    toml::to_string(
                                        &super::configuration::ConfigurationDocument::default(),
                                    )
                                    .ok()
                                });
                            rollback_text.is_some_and(|rollback_text| {
                                state.lock().is_ok_and(|mut state| {
                                    compensate_workspace_configuration_activation(
                                        &database,
                                        workspace_id,
                                        &mut state,
                                        identity,
                                        &target.path,
                                        &rollback_text,
                                        base_generation,
                                    )
                                    .is_ok()
                                })
                            })
                        })
                        && activation
                            .as_ref()
                            .is_some_and(|coordinator| (coordinator.observer)().is_ok())
                } else {
                    false
                };
                if let Ok(mut error) = last_error.lock() {
                    if result.is_err() {
                        *error = Some("Workspace configuration watcher activation failed");
                    } else if policy_result.is_err() {
                        *error = Some(if compensated {
                            "Workspace configuration policy activation failed and was rolled back"
                        } else {
                            "Workspace configuration policy activation failed; recovery is required"
                        });
                    } else {
                        *error = None;
                    }
                }
            }
        }
        ConfigurationWatcherNotice::Error(_) => {
            if let Ok(mut error) = last_error.lock() {
                *error = Some("Workspace configuration watcher failed");
            }
        }
    })
    .map_err(ConfigurationPersistenceError::Configuration)
}

fn compensate_workspace_configuration_activation(
    database: &DatabaseActor,
    workspace_id: Uuid,
    state: &mut WorkspaceConfigurationCoordinatorState,
    identity: WorkspaceConfigurationIdentity,
    path: &Path,
    text: &str,
    base_generation: u64,
) -> Result<(), ConfigurationPersistenceError> {
    let global_generation = state.global_generation;
    let service = state
        .services
        .get_mut(&identity)
        .ok_or(ConfigurationPersistenceError::InvalidScope)?;
    service.raise_generation_floor(global_generation);
    let update = commit_ui_configuration(
        service,
        identity.scope,
        path,
        text,
        base_generation,
        |record| {
            database
                .activate_configuration(ConfigurationSnapshotRecord {
                    workspace_id: workspace_id.to_string(),
                    scope_kind: identity.scope.as_str().into(),
                    scope_id: identity.persisted_scope_id(),
                    canonical_document: record.canonical_toml.clone(),
                    generation: record.activated_generation,
                    activated_at: current_unix_seconds(),
                })
                .map(|_| ())
        },
    )?;
    if let ConfigurationUpdate::Activated { record, .. } = update {
        state.global_generation = record.activated_generation;
    }
    Ok(())
}

fn commit_external_workspace_configuration<F>(
    state: &mut WorkspaceConfigurationCoordinatorState,
    identity: WorkspaceConfigurationIdentity,
    path: &Path,
    text: &str,
    persist: F,
) -> Result<ConfigurationUpdate, ConfigurationPersistenceError>
where
    F: FnOnce(&LastKnownGoodDocument) -> Result<(), DatabaseError>,
{
    let validated = match validate_scope_document(identity.scope, path, text) {
        Ok(validated) => validated,
        Err(diagnostic) => {
            let service = state
                .services
                .get_mut(&identity)
                .ok_or(ConfigurationPersistenceError::InvalidScope)?;
            service.record_diagnostic(diagnostic.clone());
            return Ok(ConfigurationUpdate::Rejected {
                diagnostic,
                snapshot: service.snapshot(),
            });
        }
    };
    let prior = state
        .services
        .get(&identity)
        .ok_or(ConfigurationPersistenceError::InvalidScope)?
        .last_known_good(identity.scope)
        .cloned();
    if prior
        .as_ref()
        .is_some_and(|record| record.canonical_toml == validated.canonical_toml)
    {
        return Ok(state
            .services
            .get_mut(&identity)
            .ok_or(ConfigurationPersistenceError::InvalidScope)?
            .reconcile_external_text(identity.scope, path, text));
    }
    let next_generation = state
        .global_generation
        .checked_add(1)
        .ok_or(ConfigurationError::GenerationOverflow)?;
    let record = LastKnownGoodDocument {
        scope: identity.scope,
        path: path.to_path_buf(),
        canonical_toml: validated.canonical_toml,
        activated_generation: next_generation,
        changed_keys: BTreeSet::new(),
    };
    if let Err(error) = persist(&record) {
        let compensation = compensate_scope_write(
            identity.scope,
            path,
            text.as_bytes(),
            prior
                .as_ref()
                .map(|record| record.canonical_toml.as_bytes()),
            state.global_generation,
            next_generation,
        );
        let service = state
            .services
            .get_mut(&identity)
            .ok_or(ConfigurationPersistenceError::InvalidScope)?;
        service.record_persistence_failure(identity.scope, path);
        if compensation.is_err() {
            return Err(ConfigurationPersistenceError::RecoveryRequired);
        }
        return Err(ConfigurationPersistenceError::Database(error));
    }
    let service = state
        .services
        .get_mut(&identity)
        .ok_or(ConfigurationPersistenceError::InvalidScope)?;
    service.raise_generation_floor(state.global_generation);
    let update = service.reconcile_external_text(identity.scope, path, text);
    let ConfigurationUpdate::Activated {
        record: activated, ..
    } = &update
    else {
        return Err(ConfigurationPersistenceError::InvalidRecovery);
    };
    if activated.activated_generation != next_generation
        || activated.canonical_toml != record.canonical_toml
    {
        return Err(ConfigurationPersistenceError::InvalidRecovery);
    }
    state.global_generation = next_generation;
    Ok(update)
}

fn read_optional_configuration(path: &Path) -> Result<Option<Vec<u8>>, ConfigurationError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigurationError::Io {
                operation: "inspect configuration before save",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_CONFIGURATION_BYTES as u64
    {
        return Err(ConfigurationError::Io {
            operation: "inspect configuration before save",
            path: path.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidData,
                "configuration path is not a bounded regular file",
            ),
        });
    }
    let bytes = fs::read(path).map_err(|source| ConfigurationError::Io {
        operation: "read configuration before save",
        path: path.to_path_buf(),
        source,
    })?;
    Ok(Some(bytes))
}

fn configuration_io(source: io::Error) -> ConfigurationPersistenceError {
    ConfigurationError::Io {
        operation: "create configuration watcher plan",
        path: PathBuf::new(),
        source,
    }
    .into()
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or_default()
}

struct PendingWorkspaceCreation {
    root: PathBuf,
    preserve_empty_root: bool,
    recovery_root: Option<(PathBuf, bool)>,
    finished: bool,
}

impl PendingWorkspaceCreation {
    fn new(root: &Path) -> WorkspaceResult<Self> {
        let preserve_empty_root = match fs::symlink_metadata(root) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(WorkspaceError::Conflict(
                    "untitled working-copy destination is not a directory".into(),
                ));
            }
            Ok(_) => {
                if fs::read_dir(root)?.next().is_some() {
                    return Err(WorkspaceError::Conflict(
                        "untitled working-copy destination is not empty".into(),
                    ));
                }
                true
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            root: root.to_path_buf(),
            preserve_empty_root,
            recovery_root: None,
            finished: false,
        })
    }

    fn register_recovery_root(&mut self, root: PathBuf) -> WorkspaceResult<()> {
        let preserve_empty_root = validate_empty_creation_root(&root)?;
        self.recovery_root = Some((root, preserve_empty_root));
        Ok(())
    }

    fn cleanup_root(root: &Path, preserve_empty_root: bool) -> io::Result<()> {
        match fs::symlink_metadata(root) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
                fs::remove_dir_all(root)?;
            }
            Ok(_) => {
                return Err(io::Error::other(
                    "working-copy destination changed type during creation",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if preserve_empty_root {
            fs::create_dir_all(root)?;
        }
        Ok(())
    }

    fn cleanup(&self) -> io::Result<()> {
        let recovery_result = self
            .recovery_root
            .as_ref()
            .map_or(Ok(()), |(root, preserve)| {
                Self::cleanup_root(root, *preserve)
            });
        let active_result = Self::cleanup_root(&self.root, self.preserve_empty_root);
        recovery_result.and(active_result)
    }

    fn commit(mut self) {
        self.finished = true;
    }

    fn abort(mut self) -> io::Result<()> {
        self.cleanup()?;
        self.finished = true;
        Ok(())
    }
}

fn validate_empty_creation_root(root: &Path) -> WorkspaceResult<bool> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(
            WorkspaceError::Conflict("creation destination is not a directory".into()),
        ),
        Ok(_) => {
            if fs::read_dir(root)?.next().is_some() {
                return Err(WorkspaceError::Conflict(
                    "creation destination is not empty".into(),
                ));
            }
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

impl Drop for PendingWorkspaceCreation {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.cleanup();
        }
    }
}

/// Creates the authoritative untitled working copy from a user-granted local
/// folder. Repository cloning itself remains an Action Gateway effect; once a
/// clone has completed into a granted folder it enters through this same path.
pub fn create_workspace_from_project(
    home: &C4osHomeLayout,
    project_folder: &Path,
    project_display_name: &str,
    workspace_display_name: &str,
    current_app_version: &str,
    lock_owner: WorkspaceLockOwner,
    created_at: i64,
) -> WorkspaceResult<ActiveWorkspace> {
    validate_display_name(workspace_display_name)?;
    if !project_folder.is_dir() {
        return Err(WorkspaceError::InvalidProject(
            "Project folder is unavailable".into(),
        ));
    }
    home.ensure_roots()?;
    let writer_lock = match acquire_workspace_writer_lock(&home.workspace_lock(), lock_owner)? {
        WriterAccess::Writable(lock) => lock,
        WriterAccess::ReadOnly { .. } => {
            return Err(WorkspaceError::Conflict(
                "another C4OS instance owns the active Workspace".into(),
            ));
        }
    };
    let working_root = home.active_workspace();
    let mut pending_creation = PendingWorkspaceCreation::new(&working_root)?;
    let prepared = (|| {
        let manifest = create_untitled_working_copy(
            &writer_lock,
            &working_root,
            project_folder,
            project_display_name,
            current_app_version,
        )?;
        let workspace_id = manifest.workspace_id.to_string();
        let project = manifest.projects.first().ok_or_else(|| {
            WorkspaceError::Conflict("untitled Workspace has no Project reference".into())
        })?;

        let mut configuration =
            ConfigurationService::new(ManagedCeilings::default(), SecurityConstraints::default());
        let configuration_path = WorkspaceLayout::new(&working_root).workspace_configuration();
        let update = configuration
            .save_scope_text(
                ConfigurationScope::Workspace,
                &configuration_path,
                "schema_version = 1\n",
                0,
            )
            .map_err(|_| {
                WorkspaceError::Conflict("Workspace configuration could not be initialized".into())
            })?;
        let ConfigurationUpdate::Activated { record, .. } = update else {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration did not activate".into(),
            ));
        };

        let recovery_root = home.workspace_recovery_root().join(&workspace_id);
        pending_creation.register_recovery_root(recovery_root.clone())?;
        let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
            &working_root,
            &workspace_id,
            recovery_root,
        );
        let (database, _) = DatabaseActor::start(descriptor).map_err(database_conflict)?;
        database
            .create_workspace(WorkspaceRecord {
                workspace_id: workspace_id.clone(),
                display_name: workspace_display_name.to_owned(),
                created_at,
                updated_at: created_at,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .map_err(database_conflict)?;
        database
            .add_project(ProjectRecord {
                workspace_id: workspace_id.clone(),
                project_id: project.project_id.to_string(),
                display_name: project.display_name.clone(),
                current_path: project.last_known_path.clone(),
                last_known_path: project.last_known_path.clone(),
                path_state: ProjectPathState::Found,
                position: 0,
                lifecycle_state: LifecycleState::Active,
                inactivated_at: None,
            })
            .map_err(database_conflict)?;
        database
            .activate_configuration(ConfigurationSnapshotRecord {
                workspace_id: workspace_id.clone(),
                scope_kind: "workspace".into(),
                scope_id: workspace_id,
                canonical_document: record.canonical_toml,
                generation: record.activated_generation,
                activated_at: created_at,
            })
            .map_err(database_conflict)?;
        let (snapshot, _) = database
            .complete_workspace_snapshot(false)
            .map_err(database_conflict)?;
        let canonical = manifest_from_database_snapshot(&manifest, &snapshot)?;
        let canonical = persist_canonical_working_manifest(
            &writer_lock,
            &working_root,
            &canonical,
            current_app_version,
            ArchiveLimits::default(),
        )?;
        let trusted_projects = [project.project_id].into_iter().collect();
        let database = Arc::new(database);
        let managed_configuration = ManagedWorkspaceConfiguration::start(
            Arc::clone(&database),
            working_root.clone(),
            manifest.workspace_id,
        )
        .map_err(configuration_conflict)?;
        Ok((canonical, database, managed_configuration, trusted_projects))
    })();

    let (canonical, database, managed_configuration, trusted_projects) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            return match pending_creation.abort() {
                Ok(()) => Err(error),
                Err(cleanup_error) => Err(WorkspaceError::CommitRecovery {
                    operation: "create Workspace",
                    recovery_path: working_root,
                    message: cleanup_error.to_string(),
                }),
            };
        }
    };
    pending_creation.commit();
    Ok(ActiveWorkspace {
        workspace: WritableWorkspace {
            manifest: canonical,
            working_root,
            archive_path: PathBuf::new(),
            recovery_notice: None,
            writer_lock,
        },
        database,
        configuration: managed_configuration,
        trusted_projects,
    })
}

pub fn create_workspace_from_completed_clone(
    home: &C4osHomeLayout,
    cloned_project_folder: &Path,
    project_display_name: &str,
    workspace_display_name: &str,
    current_app_version: &str,
    lock_owner: WorkspaceLockOwner,
    created_at: i64,
) -> WorkspaceResult<ActiveWorkspace> {
    create_workspace_from_project(
        home,
        cloned_project_folder,
        project_display_name,
        workspace_display_name,
        current_app_version,
        lock_owner,
        created_at,
    )
}

/// Restores the authoritative active working copy for production startup.
///
/// Restoration is deliberately limited to `workspace/active`: recent archive
/// paths are not candidates, and Project trust is never reconstructed from
/// portable or durable Workspace state. The returned service retains both the
/// Workspace writer lock and its database writer ownership for its lifetime.
pub fn restore_active_workspace(
    home: &C4osHomeLayout,
    restore_requested: bool,
    current_app_version: &str,
    limits: ArchiveLimits,
    lock_owner: WorkspaceLockOwner,
) -> WorkspaceResult<Option<ActiveWorkspace>> {
    if !restore_requested {
        return Ok(None);
    }

    let working_root = home.active_workspace();
    let root_metadata = match fs::symlink_metadata(&working_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(WorkspaceError::Conflict(
            "active Workspace root is not an owned directory".into(),
        ));
    }

    let writer_lock = match acquire_workspace_writer_lock(&home.workspace_lock(), lock_owner)? {
        WriterAccess::Writable(lock) => lock,
        WriterAccess::ReadOnly { .. } => {
            return Err(WorkspaceError::Conflict(
                "another C4OS instance owns the active Workspace".into(),
            ));
        }
    };

    let manifest_path = WorkspaceLayout::new(&working_root).manifest();
    let manifest_metadata = fs::symlink_metadata(&manifest_path)?;
    if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
        return Err(WorkspaceError::Conflict(
            "active Workspace manifest is not an owned file".into(),
        ));
    }
    let manifest_bound = limits.max_manifest_bytes.min(limits.max_entry_bytes);
    if manifest_metadata.len() > manifest_bound {
        return Err(WorkspaceError::Conflict(
            "active Workspace manifest exceeds its configured bound".into(),
        ));
    }
    let mut manifest_bytes = Vec::new();
    fs::File::open(&manifest_path)?
        .take(manifest_bound.saturating_add(1))
        .read_to_end(&mut manifest_bytes)?;
    if manifest_bytes.len() as u64 > manifest_bound {
        return Err(WorkspaceError::Conflict(
            "active Workspace manifest exceeds its configured bound".into(),
        ));
    }
    let manifest_text = std::str::from_utf8(&manifest_bytes)
        .map_err(|_| WorkspaceError::Conflict("active Workspace manifest is not UTF-8".into()))?;
    let manifest: WorkspaceManifest = toml::from_str(manifest_text)?;

    // The archive-source preflight provides the existing structural manifest,
    // version, path, and bound validation without requiring stale active-copy
    // digests to match a database that may legitimately be ahead after a
    // crash. Recovery validation below is the SQLite semantic authority.
    preflight_workspace_archive_source(&working_root, &manifest, current_app_version, limits)?;
    let recovered = validate_workspace_semantics(
        &working_root,
        &manifest,
        WorkspaceSemanticValidationTarget::ActiveRecovery,
    )?;
    if recovered.workspace_id != manifest.workspace_id {
        return Err(WorkspaceError::Conflict(
            "active semantic Workspace identity mismatch".into(),
        ));
    }

    let workspace_id = recovered.workspace_id.to_string();
    let expected_database = WorkspaceLayout::new(&working_root).database();
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        &working_root,
        &workspace_id,
        home.workspace_recovery_root().join(&workspace_id),
    );
    if descriptor.path != expected_database {
        return Err(WorkspaceError::Conflict(
            "active Workspace database root binding is invalid".into(),
        ));
    }
    let (database, _) = DatabaseActor::start(descriptor).map_err(database_conflict)?;
    if database.descriptor().path != expected_database {
        return Err(WorkspaceError::Conflict(
            "active Workspace database actor owns an unexpected root".into(),
        ));
    }
    let (snapshot, _) = database
        .complete_workspace_snapshot(false)
        .map_err(database_conflict)?;
    let canonical = manifest_from_database_snapshot(&recovered, &snapshot)?;
    if canonical.workspace_id != recovered.workspace_id {
        return Err(WorkspaceError::Conflict(
            "active Workspace database identity changed during restoration".into(),
        ));
    }

    let database = Arc::new(database);
    let managed_configuration = ManagedWorkspaceConfiguration::start(
        Arc::clone(&database),
        working_root.clone(),
        canonical.workspace_id,
    )
    .map_err(configuration_conflict)?;

    Ok(Some(ActiveWorkspace {
        workspace: WritableWorkspace {
            manifest: canonical,
            working_root,
            archive_path: PathBuf::new(),
            recovery_notice: None,
            writer_lock,
        },
        database,
        configuration: managed_configuration,
        // Project trust is process-local picker authority and is never
        // reconstructed from the active working copy.
        trusted_projects: BTreeSet::new(),
    }))
}

/// Production archive-open path. Structural Zip checks are followed by
/// database, identity, scope-configuration, and cross-record validation before
/// the candidate can replace the active working copy.
pub fn open_workspace_with_database(
    app_database: &DatabaseActor,
    home: &C4osHomeLayout,
    archive_path: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
    lock_owner: WorkspaceLockOwner,
    opened_at: i64,
) -> WorkspaceResult<WorkspaceServiceOpen> {
    let opened = prepare_open_workspace_archive(
        home,
        archive_path,
        current_app_version,
        limits,
        lock_owner,
        validate_workspace_semantics,
    )?;
    match opened {
        PreparedOpenWorkspaceOutcome::ReadOnly(workspace) => {
            Ok(WorkspaceServiceOpen::ReadOnly(workspace))
        }
        PreparedOpenWorkspaceOutcome::Writable(pending) => {
            let workspace_id = pending.workspace().manifest.workspace_id.to_string();
            let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
                &pending.workspace().working_root,
                &workspace_id,
                home.workspace_recovery_root().join(&workspace_id),
            );
            let (database, _) = DatabaseActor::start(descriptor).map_err(|_| {
                WorkspaceError::Conflict("Workspace database could not be activated".into())
            })?;
            let database = Arc::new(database);
            let (snapshot, _) = database
                .complete_workspace_snapshot(false)
                .map_err(database_conflict)?;
            let display_name = snapshot
                .workspace
                .as_ref()
                .ok_or_else(|| {
                    WorkspaceError::Conflict("opened Workspace record is inactive".into())
                })?
                .display_name
                .clone();
            let managed_configuration = ManagedWorkspaceConfiguration::start(
                Arc::clone(&database),
                pending.workspace().working_root.clone(),
                pending.workspace().manifest.workspace_id,
            )
            .map_err(configuration_conflict)?;
            app_database
                .record_recent_workspace(RecentWorkspaceRecord {
                    workspace_id: workspace_id.clone(),
                    display_name,
                    archive_path: archive_path.to_string_lossy().into_owned(),
                    last_opened_at: opened_at,
                    lifecycle_state: LifecycleState::Active,
                    inactivated_at: None,
                })
                .map_err(database_conflict)?;
            let workspace = pending.commit();
            Ok(WorkspaceServiceOpen::Writable(Box::new(ActiveWorkspace {
                workspace,
                database,
                configuration: managed_configuration,
                // Trust is local runtime authority and is intentionally never
                // reconstructed from a portable Workspace archive.
                trusted_projects: BTreeSet::new(),
            })))
        }
    }
}

pub fn validate_workspace_semantics(
    root: &Path,
    manifest: &WorkspaceManifest,
    target: WorkspaceSemanticValidationTarget,
) -> WorkspaceResult<WorkspaceManifest> {
    let expected_workspace_id = manifest.workspace_id.to_string();
    let inspection = inspect_complete_workspace_database_read_only(
        WorkspaceLayout::new(root).database(),
        &expected_workspace_id,
        true,
    )
    .map_err(|_| WorkspaceError::Conflict("Workspace database semantics are invalid".into()))?;
    let snapshot = &inspection.snapshot;
    match target {
        WorkspaceSemanticValidationTarget::ArchiveCandidate
            if snapshot.generation != manifest.generation =>
        {
            return Err(WorkspaceError::Conflict(
                "Workspace archive generation disagrees with its database".into(),
            ));
        }
        WorkspaceSemanticValidationTarget::ActiveRecovery
            if snapshot.generation < manifest.generation =>
        {
            return Err(WorkspaceError::Conflict(
                "active Workspace database generation is behind its manifest".into(),
            ));
        }
        _ => {}
    }

    validate_configuration_records(root, manifest, snapshot)?;
    let canonical = manifest_from_database_snapshot(manifest, snapshot)?;
    if matches!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate)
        && canonical.projects != manifest.projects
    {
        return Err(WorkspaceError::Conflict(
            "Workspace archive Project projection disagrees with its database".into(),
        ));
    }
    Ok(canonical)
}

fn validate_configuration_records(
    root: &Path,
    manifest: &WorkspaceManifest,
    snapshot: &WorkspaceSnapshot,
) -> WorkspaceResult<()> {
    let project_ids: BTreeSet<_> = snapshot
        .projects
        .iter()
        .map(|project| project.project_id.as_str())
        .collect();
    let chat_ids: BTreeSet<_> = snapshot
        .chats
        .iter()
        .map(|chat| chat.chat_id.as_str())
        .collect();

    let mut expected_files = BTreeMap::new();
    let mut expected_scopes = BTreeSet::new();
    for record in &snapshot.configurations {
        let (scope, valid_identity) = match record.scope_kind.as_str() {
            "workspace" => (
                ConfigurationScope::Workspace,
                record.scope_id == manifest.workspace_id.to_string(),
            ),
            "project" => (
                ConfigurationScope::Project,
                project_ids.contains(record.scope_id.as_str()),
            ),
            "chat" => (
                ConfigurationScope::Chat,
                chat_ids.contains(record.scope_id.as_str()),
            ),
            _ => {
                return Err(WorkspaceError::Conflict(
                    "Workspace configuration scope is invalid".into(),
                ));
            }
        };
        if !valid_identity {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration references an unknown scope".into(),
            ));
        }
        let path = configuration_path(root, scope, &record.scope_id)?;
        if !expected_scopes.insert((scope, record.scope_id.clone())) {
            return Err(WorkspaceError::Conflict(
                "Workspace contains a duplicate configuration scope".into(),
            ));
        }
        let validated =
            validate_scope_document(scope, &path, &record.canonical_document).map_err(|_| {
                WorkspaceError::Conflict(
                    "Workspace last-known-good configuration is invalid".into(),
                )
            })?;
        if validated.canonical_toml != record.canonical_document {
            return Err(WorkspaceError::Conflict(
                "Workspace last-known-good configuration is not canonical".into(),
            ));
        }
        if expected_files
            .insert(
                path,
                (
                    scope,
                    record.scope_id.clone(),
                    record.canonical_document.as_str(),
                ),
            )
            .is_some()
        {
            return Err(WorkspaceError::Conflict(
                "Workspace contains duplicate configuration file authority".into(),
            ));
        }
    }

    let actual_files = configuration_files_on_disk(root, manifest.workspace_id)?;
    let workspace_path = WorkspaceLayout::new(root).workspace_configuration();
    if !actual_files.contains_key(&workspace_path) {
        return Err(WorkspaceError::Conflict(
            "Workspace configuration file is missing".into(),
        ));
    }
    for (path, (actual_scope, actual_scope_id)) in actual_files {
        let Some((expected_scope, expected_scope_id, canonical_lkg)) = expected_files.remove(&path)
        else {
            return Err(WorkspaceError::Conflict(
                "Workspace contains a configuration file without SQLite LKG authority".into(),
            ));
        };
        if actual_scope != expected_scope || actual_scope_id != expected_scope_id {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration scope identity disagrees with SQLite LKG".into(),
            ));
        }
        let text = read_bounded_utf8(&path)?;
        let validated = validate_scope_document(actual_scope, &path, &text).map_err(|_| {
            WorkspaceError::Conflict("Workspace configuration file is invalid".into())
        })?;
        if validated.canonical_toml != canonical_lkg {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration file disagrees with SQLite LKG".into(),
            ));
        }
    }
    if !expected_files.is_empty() {
        return Err(WorkspaceError::Conflict(
            "Workspace SQLite LKG scope has no configuration file".into(),
        ));
    }
    Ok(())
}

fn configuration_files_on_disk(
    root: &Path,
    workspace_id: Uuid,
) -> WorkspaceResult<BTreeMap<PathBuf, (ConfigurationScope, String)>> {
    const MAX_SCOPE_FILES: usize = 10_000;
    let layout = WorkspaceLayout::new(root);
    let mut files = BTreeMap::new();
    let workspace = layout.workspace_configuration();
    let configuration_root = root.join("config");
    if configuration_root.exists() {
        let metadata = fs::symlink_metadata(&configuration_root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration root is not a regular directory".into(),
            ));
        }
        for entry in fs::read_dir(&configuration_root)? {
            let entry = entry?;
            if entry.path() != workspace {
                return Err(WorkspaceError::Conflict(
                    "Workspace contains an unowned configuration entry".into(),
                ));
            }
        }
    }
    if workspace.exists() {
        insert_configuration_file(
            &mut files,
            workspace,
            ConfigurationScope::Workspace,
            workspace_id.to_string(),
        )?;
    }
    collect_scoped_configuration_files(
        &root.join("projects"),
        ConfigurationScope::Project,
        &mut files,
        MAX_SCOPE_FILES,
    )?;
    collect_scoped_configuration_files(
        &root.join("chats"),
        ConfigurationScope::Chat,
        &mut files,
        MAX_SCOPE_FILES,
    )?;
    Ok(files)
}

fn collect_scoped_configuration_files(
    parent: &Path,
    scope: ConfigurationScope,
    files: &mut BTreeMap<PathBuf, (ConfigurationScope, String)>,
    maximum: usize,
) -> WorkspaceResult<()> {
    if !parent.exists() {
        return Ok(());
    }
    let parent_metadata = fs::symlink_metadata(parent)?;
    if !parent_metadata.is_dir() || parent_metadata.file_type().is_symlink() {
        return Err(WorkspaceError::Conflict(
            "configuration scope root is not a regular directory".into(),
        ));
    }
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(WorkspaceError::Conflict(
                "configuration scope path is not a regular directory".into(),
            ));
        }
        let scope_id = entry
            .file_name()
            .into_string()
            .map_err(|_| WorkspaceError::Conflict("configuration scope ID is not UTF-8".into()))?;
        let configuration = entry.path().join("config.toml");
        if !configuration.exists() {
            continue;
        }
        Uuid::parse_str(&scope_id).map_err(|_| {
            WorkspaceError::Conflict("configuration file has an invalid scope ID".into())
        })?;
        insert_configuration_file(files, configuration, scope, scope_id)?;
        if files.len() > maximum {
            return Err(WorkspaceError::Conflict(
                "Workspace configuration file count exceeds its bound".into(),
            ));
        }
    }
    Ok(())
}

fn insert_configuration_file(
    files: &mut BTreeMap<PathBuf, (ConfigurationScope, String)>,
    path: PathBuf,
    scope: ConfigurationScope,
    scope_id: String,
) -> WorkspaceResult<()> {
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(WorkspaceError::Conflict(
            "configuration path is not a regular file".into(),
        ));
    }
    if files.insert(path, (scope, scope_id)).is_some() {
        return Err(WorkspaceError::Conflict(
            "Workspace contains a duplicate configuration file".into(),
        ));
    }
    Ok(())
}

fn configuration_path(
    root: &Path,
    scope: ConfigurationScope,
    scope_id: &str,
) -> WorkspaceResult<std::path::PathBuf> {
    let layout = WorkspaceLayout::new(root);
    match scope {
        ConfigurationScope::Workspace => Ok(layout.workspace_configuration()),
        ConfigurationScope::Project => Uuid::parse_str(scope_id)
            .map(|scope_id| layout.project_configuration(scope_id))
            .map_err(|_| WorkspaceError::Conflict("Project configuration ID is invalid".into())),
        ConfigurationScope::Chat => Uuid::parse_str(scope_id)
            .map(|scope_id| layout.chat_configuration(scope_id))
            .map_err(|_| WorkspaceError::Conflict("Chat configuration ID is invalid".into())),
        ConfigurationScope::App => Err(WorkspaceError::Conflict(
            "app configuration cannot be stored in a Workspace".into(),
        )),
    }
}

fn read_bounded_utf8(path: &Path) -> WorkspaceResult<String> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(MAX_CONFIGURATION_BYTES.min(64 * 1024));
    file.by_ref()
        .take(u64::try_from(MAX_CONFIGURATION_BYTES).unwrap_or(u64::MAX) + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_CONFIGURATION_BYTES {
        return Err(WorkspaceError::Conflict(
            "Workspace configuration exceeds its byte limit".into(),
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| WorkspaceError::Conflict("Workspace configuration is not UTF-8".into()))
}

/// Repack a Workspace only from its authoritative working copy and a
/// consistent SQLite online backup owned by the database actor.
pub fn save_workspace_with_database(
    database: &DatabaseActor,
    writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<SavedWorkspace> {
    Ok(prepare_save_workspace_with_database(
        database,
        writer_lock,
        working_root,
        archive_path,
        manifest,
        current_app_version,
        limits,
    )?
    .commit())
}

/// Prepare a database-consistent archive replacement while retaining the
/// previously validated destination until app-owned bookkeeping succeeds.
fn prepare_save_workspace_with_database(
    database: &DatabaseActor,
    writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<PendingWorkspaceSave> {
    let expected_database = WorkspaceLayout::new(working_root).database();
    if database.descriptor().path != expected_database {
        return Err(WorkspaceError::Conflict(
            "Workspace database actor does not own the active working copy".into(),
        ));
    }

    preflight_workspace_archive_source(working_root, manifest, current_app_version, limits)?;

    let archive_parent = archive_path
        .parent()
        .ok_or_else(|| WorkspaceError::Conflict("archive destination has no parent".into()))?;
    fs::create_dir_all(archive_parent)?;
    let barrier_staging = tempfile::Builder::new()
        .prefix(".c4os-database-barrier-")
        .tempdir_in(archive_parent)?;
    let barrier_database = barrier_staging.path().join("workspace.sqlite3");
    let barrier = database
        .backup_complete_workspace_snapshot_to(&barrier_database, false)
        .map_err(|_| {
            WorkspaceError::Conflict(
                "consistent Workspace database backup could not be completed".into(),
            )
        })?;
    let canonical_manifest = manifest_from_database_snapshot(manifest, &barrier.snapshot)?;

    prepare_workspace_archive_save(
        writer_lock,
        working_root,
        archive_path,
        &canonical_manifest,
        current_app_version,
        limits,
        |source, destination| {
            if source != expected_database {
                return Err(WorkspaceError::Conflict(
                    "Workspace archive requested an unexpected database source".into(),
                ));
            }
            fs::copy(&barrier_database, destination)?;
            fs::File::open(destination)?.sync_all()?;
            Ok(())
        },
    )
}

fn manifest_from_database_snapshot(
    prior: &WorkspaceManifest,
    snapshot: &WorkspaceSnapshot,
) -> WorkspaceResult<WorkspaceManifest> {
    if snapshot.truncated {
        return Err(WorkspaceError::Conflict(
            "Workspace is larger than the bounded archive snapshot".into(),
        ));
    }
    let workspace = snapshot.workspace.as_ref().ok_or_else(|| {
        WorkspaceError::Conflict("active Workspace database record is unavailable".into())
    })?;
    let workspace_id = Uuid::parse_str(&workspace.workspace_id)
        .map_err(|_| WorkspaceError::Conflict("Workspace database identity is invalid".into()))?;
    if workspace_id != prior.workspace_id {
        return Err(WorkspaceError::Conflict(
            "Workspace database and archive identities differ".into(),
        ));
    }
    let active_projects: Vec<_> = snapshot
        .projects
        .iter()
        .filter(|project| project.lifecycle_state == LifecycleState::Active)
        .collect();
    let mut positions = active_projects.iter().map(|project| project.position);
    if positions
        .by_ref()
        .enumerate()
        .any(|(expected, actual)| actual != i64::try_from(expected).unwrap_or(i64::MAX))
    {
        return Err(WorkspaceError::Conflict(
            "Workspace Project order is not contiguous".into(),
        ));
    }
    let projects = active_projects
        .iter()
        .map(|project| {
            Ok(ProjectReference {
                project_id: Uuid::parse_str(&project.project_id).map_err(|_| {
                    WorkspaceError::Conflict("Workspace Project identity is invalid".into())
                })?,
                display_name: project.display_name.clone(),
                last_known_path: project.last_known_path.clone(),
                trusted_root: false,
            })
        })
        .collect::<WorkspaceResult<Vec<_>>>()?;

    Ok(WorkspaceManifest {
        schema_version: WORKSPACE_SCHEMA_VERSION,
        workspace_id,
        generation: snapshot.generation,
        minimum_app_version: prior.minimum_app_version.clone(),
        projects,
        files: prior.files.clone(),
    })
}

fn database_conflict(_error: DatabaseError) -> WorkspaceError {
    WorkspaceError::Conflict("durable Workspace operation failed".into())
}

fn configuration_conflict(_error: ConfigurationPersistenceError) -> WorkspaceError {
    WorkspaceError::Conflict("durable Workspace configuration operation failed".into())
}

fn validate_display_name(value: &str) -> WorkspaceResult<()> {
    let value = value.trim();
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        Err(WorkspaceError::Conflict(
            "Workspace display name is invalid".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn app_configuration_observer_reconciles_registration_and_rolls_back_failed_external_policy() {
        let temp = TempDir::new().expect("temporary app configuration root");
        let home = C4osHomeLayout::new(temp.path());
        let (database, _) =
            DatabaseActor::start(DatabaseDescriptor::app(temp.path())).expect("app database");
        let managed = ManagedAppConfiguration::start(
            Arc::new(database),
            home.clone(),
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("managed app configuration");
        let initial = managed.snapshot().expect("initial configuration");
        let calls = Arc::new(AtomicUsize::new(0));
        let reject_external = Arc::new(AtomicBool::new(false));
        let observer_calls = Arc::clone(&calls);
        let observer_rejection = Arc::clone(&reject_external);
        managed
            .set_activation_observer(
                Arc::new(Mutex::new(())),
                Arc::new(move |record| {
                    observer_calls.fetch_add(1, Ordering::SeqCst);
                    if observer_rejection.load(Ordering::SeqCst)
                        && record
                            .as_ref()
                            .is_some_and(|record| record.canonical_toml.contains("approve_for_me"))
                    {
                        Err(())
                    } else {
                        Ok(())
                    }
                }),
            )
            .expect("observer registration");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        reject_external.store(true, Ordering::SeqCst);
        let path = home.app_configuration();
        fs::create_dir_all(path.parent().expect("configuration parent"))
            .expect("configuration parent");
        fs::write(
            &path,
            "schema_version = 1\ndefault_approval_preset = \"approve_for_me\"\n",
        )
        .expect("external configuration edit");

        for _ in 0..80 {
            if managed.last_error().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert_eq!(
            managed
                .snapshot()
                .expect("compensated configuration")
                .configuration
                .default_approval_preset,
            initial.configuration.default_approval_preset
        );
        assert!(calls.load(Ordering::SeqCst) >= 2);
        assert_eq!(
            managed.last_error(),
            Some("configuration policy activation failed and was rolled back")
        );
    }

    #[test]
    fn rejected_external_app_configuration_sets_and_valid_activation_clears_error_state() {
        let temp = TempDir::new().expect("temporary app configuration root");
        let home = C4osHomeLayout::new(temp.path());
        let (database, _) =
            DatabaseActor::start(DatabaseDescriptor::app(temp.path())).expect("app database");
        let managed = ManagedAppConfiguration::start(
            Arc::new(database),
            home.clone(),
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("managed app configuration");
        let path = home.app_configuration();
        fs::create_dir_all(path.parent().expect("configuration parent"))
            .expect("configuration parent");

        fs::write(&path, "schema_version = 999\n").expect("invalid external configuration edit");
        for _ in 0..80 {
            if managed.last_error().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert_eq!(
            managed.last_error(),
            Some("configuration watcher edit was rejected")
        );

        fs::write(
            &path,
            "schema_version = 1\nrestore_last_workspace = false\n",
        )
        .expect("valid external configuration edit");
        for _ in 0..80 {
            if managed.last_error().is_none() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert_eq!(managed.last_error(), None);
        assert!(
            !managed
                .snapshot()
                .expect("activated configuration")
                .configuration
                .restore_last_workspace
        );
    }

    #[test]
    fn aborted_workspace_creation_restores_active_and_recovery_roots() {
        let temp = TempDir::new().expect("temporary creation root");
        let active = temp.path().join("workspace/active");
        let recovery = temp.path().join("workspace/recovery/workspace-id");
        fs::create_dir_all(&active).expect("pre-existing empty active root");
        let mut pending =
            PendingWorkspaceCreation::new(&active).expect("pending Workspace creation");
        pending
            .register_recovery_root(recovery.clone())
            .expect("pending recovery root");
        fs::create_dir_all(active.join("state")).expect("provisional active state");
        fs::write(active.join("state/workspace.sqlite3"), b"provisional")
            .expect("provisional database");
        fs::create_dir_all(&recovery).expect("provisional recovery root");
        fs::write(recovery.join("migration.sqlite3"), b"provisional")
            .expect("provisional recovery artifact");

        pending.abort().expect("rollback provisional creation");

        assert!(active.is_dir(), "the prior empty active root is restored");
        assert_eq!(
            fs::read_dir(&active).expect("restored active root").count(),
            0
        );
        assert!(
            !recovery.exists(),
            "new per-Workspace recovery artifacts are removed"
        );
    }

    #[test]
    fn failed_lkg_persistence_compensates_file_and_in_memory_activation() {
        let temp = TempDir::new().expect("temporary configuration root");
        let path = temp.path().join("config.toml");
        let mut configuration =
            ConfigurationService::new(ManagedCeilings::default(), SecurityConstraints::default());

        let initial_failure = commit_ui_configuration(
            &mut configuration,
            ConfigurationScope::App,
            &path,
            "schema_version = 1\nrestore_last_workspace = true\n",
            0,
            |_| Err(DatabaseError::ActorStopped),
        );
        assert!(matches!(
            initial_failure,
            Err(ConfigurationPersistenceError::Database(
                DatabaseError::ActorStopped
            ))
        ));
        assert!(!path.exists());
        assert_eq!(configuration.snapshot().generation, 0);
        assert!(
            configuration
                .last_known_good(ConfigurationScope::App)
                .is_none()
        );

        commit_ui_configuration(
            &mut configuration,
            ConfigurationScope::App,
            &path,
            "schema_version = 1\nrestore_last_workspace = false\n",
            0,
            |_| Ok(()),
        )
        .expect("seed durable configuration");
        let durable_bytes = fs::read(&path).expect("durable configuration bytes");
        let durable_record = configuration
            .last_known_good(ConfigurationScope::App)
            .expect("durable LKG")
            .clone();

        let replacement_failure = commit_ui_configuration(
            &mut configuration,
            ConfigurationScope::App,
            &path,
            "schema_version = 1\nrestore_last_workspace = true\n",
            durable_record.activated_generation,
            |_| Err(DatabaseError::ActorStopped),
        );
        assert!(matches!(
            replacement_failure,
            Err(ConfigurationPersistenceError::Database(
                DatabaseError::ActorStopped
            ))
        ));
        assert_eq!(
            fs::read(&path).expect("compensated configuration bytes"),
            durable_bytes
        );
        assert_eq!(configuration.snapshot().generation, 1);
        assert_eq!(
            configuration
                .last_known_good(ConfigurationScope::App)
                .expect("restored LKG"),
            &durable_record
        );
    }

    #[test]
    fn external_workspace_scope_persistence_failures_compensate_every_scope_identity() {
        let temp = TempDir::new().expect("temporary Workspace configuration root");
        let workspace_id = Uuid::new_v4();
        let layout = WorkspaceLayout::new(temp.path());
        for (scope, scope_id, replacement) in [
            (
                ConfigurationScope::Workspace,
                workspace_id,
                "schema_version = 1\ndefault_runtime = \"external.runtime\"\n",
            ),
            (
                ConfigurationScope::Project,
                Uuid::new_v4(),
                "schema_version = 1\nmodel_route = \"external.project\"\n",
            ),
            (
                ConfigurationScope::Chat,
                Uuid::new_v4(),
                "schema_version = 1\ndefault_environment = \"external.chat\"\n",
            ),
        ] {
            let path = match scope {
                ConfigurationScope::Workspace => layout.workspace_configuration(),
                ConfigurationScope::Project => layout.project_configuration(scope_id),
                ConfigurationScope::Chat => layout.chat_configuration(scope_id),
                ConfigurationScope::App => unreachable!("fixture contains Workspace scopes"),
            };
            let identity = WorkspaceConfigurationIdentity { scope, scope_id };
            let mut service = ConfigurationService::new(
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            );
            service
                .save_scope_text(scope, &path, "schema_version = 1\n", 0)
                .expect("seed scope LKG");
            let durable = service
                .last_known_good(scope)
                .expect("seeded scope LKG")
                .clone();
            fs::write(&path, replacement).expect("external stable edit");
            let mut state = WorkspaceConfigurationCoordinatorState {
                services: BTreeMap::from([(identity, service)]),
                global_generation: 10,
            };

            let failure = commit_external_workspace_configuration(
                &mut state,
                identity,
                &path,
                replacement,
                |_| Err(DatabaseError::ActorStopped),
            );
            assert!(matches!(
                failure,
                Err(ConfigurationPersistenceError::Database(
                    DatabaseError::ActorStopped
                ))
            ));
            assert_eq!(
                fs::read_to_string(&path).expect("compensated scope file"),
                durable.canonical_toml
            );
            assert_eq!(state.global_generation, 10);
            let retained = state.services.get(&identity).expect("retained service");
            assert_eq!(
                retained.last_known_good(scope).expect("retained LKG"),
                &durable
            );
            assert_eq!(retained.diagnostics().len(), 1);
        }
    }

    #[test]
    fn workspace_watcher_reconciles_policy_and_compensates_a_rejected_activation() {
        let temp = TempDir::new().expect("temporary Workspace home");
        let home = C4osHomeLayout::new(temp.path().join("home"));
        let project = temp.path().join("project");
        fs::create_dir_all(&project).expect("fixture Project");
        let workspace = create_workspace_from_project(
            &home,
            &project,
            "Fixture Project",
            "Fixture Workspace",
            "0.1.0",
            WorkspaceLockOwner {
                process_id: std::process::id(),
                app_instance_id: Uuid::new_v4(),
                acquired_unix_ms: 1,
                label: "workspace-configuration-observer-test".into(),
            },
            1,
        )
        .expect("active Workspace");
        let path = WorkspaceLayout::new(workspace.working_root()).workspace_configuration();
        let initial_preset = workspace
            .restore_effective_configuration_snapshot(
                None,
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("initial effective configuration")
            .configuration
            .default_approval_preset;
        let observer_path = path.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let observer_calls = Arc::clone(&calls);
        workspace
            .set_configuration_activation_observer(
                Arc::new(Mutex::new(())),
                Arc::new(move || {
                    observer_calls.fetch_add(1, Ordering::SeqCst);
                    let text = fs::read_to_string(&observer_path).map_err(|_| ())?;
                    if text.contains("approve_for_me") {
                        Err(())
                    } else {
                        Ok(())
                    }
                }),
            )
            .expect("Workspace observer");

        fs::write(
            &path,
            "schema_version = 1\ndefault_approval_preset = \"approve_for_me\"\n",
        )
        .expect("external Workspace edit");
        for _ in 0..80 {
            if workspace.configuration_last_error().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }

        assert!(calls.load(Ordering::SeqCst) >= 2);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed and was rolled back")
        );
        let effective = workspace
            .restore_effective_configuration_snapshot(
                None,
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("compensated effective configuration");
        assert_eq!(
            effective.configuration.default_approval_preset,
            initial_preset
        );
    }

    #[test]
    fn workspace_programmatic_save_compensates_a_rejected_policy_activation() {
        let temp = TempDir::new().expect("temporary Workspace home");
        let home = C4osHomeLayout::new(temp.path().join("home"));
        let project = temp.path().join("project");
        fs::create_dir_all(&project).expect("fixture Project");
        let mut workspace = create_workspace_from_project(
            &home,
            &project,
            "Fixture Project",
            "Fixture Workspace",
            "0.1.0",
            WorkspaceLockOwner {
                process_id: std::process::id(),
                app_instance_id: Uuid::new_v4(),
                acquired_unix_ms: 1,
                label: "workspace-programmatic-configuration-observer-test".into(),
            },
            1,
        )
        .expect("active Workspace");
        let initial_preset = workspace
            .restore_effective_configuration_snapshot(
                None,
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("initial effective configuration")
            .configuration
            .default_approval_preset;
        let calls = Arc::new(AtomicUsize::new(0));
        let observer_calls = Arc::clone(&calls);
        workspace
            .set_configuration_activation_observer(
                Arc::new(Mutex::new(())),
                Arc::new(move || {
                    let call = observer_calls.fetch_add(1, Ordering::SeqCst);
                    if call == 0 { Err(()) } else { Ok(()) }
                }),
            )
            .expect("Workspace observer");
        let mut service = workspace
            .restore_configuration_stack(
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("Workspace configuration");
        let generation = service.snapshot().generation;
        let workspace_id = workspace.manifest().workspace_id;

        assert!(matches!(
            workspace.save_configuration_scope(
                &mut service,
                ConfigurationScope::Workspace,
                workspace_id,
                "schema_version = 1\ndefault_approval_preset = \"approve_for_me\"\n",
                generation,
                2,
            ),
            Err(ConfigurationPersistenceError::PolicyActivation)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let effective = workspace
            .restore_effective_configuration_snapshot(
                None,
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("compensated effective configuration");
        assert_eq!(
            effective.configuration.default_approval_preset,
            initial_preset
        );
    }
}
