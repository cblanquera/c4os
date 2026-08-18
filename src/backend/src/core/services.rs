//! Cohesive cross-module service integration owned by the Rust core.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use super::configuration::ConfigurationDiagnosticCode;
use super::configuration::{
    ConfigurationError, ConfigurationScope, ConfigurationService, ConfigurationUpdate,
    ConfigurationWatcherNotice, ConfigurationWatcherPlan, EffectiveConfigurationSnapshot,
    LastKnownGoodDocument, MAX_CONFIGURATION_BYTES, ManagedCeilings,
    ParentDirectoryConfigurationWatcher, SecurityConstraints, WatchedConfiguration,
    compensate_scope_write, recover_missing_scope_file, replace_legacy_scope_placeholder,
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
    acquire_workspace_writer_lock, commit_rename, commit_sync_directory,
    create_untitled_working_copy, persist_canonical_working_manifest,
    preflight_workspace_archive_source, prepare_open_workspace_archive,
    prepare_workspace_archive_save,
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
    errors: Arc<Mutex<AppConfigurationErrorState>>,
    activation_coordinator: Arc<Mutex<Option<AppConfigurationActivationCoordinator>>>,
}

#[derive(Clone)]
struct AppConfigurationActivationCoordinator {
    gate: Arc<Mutex<()>>,
    observer: Arc<dyn Fn(Option<LastKnownGoodDocument>) -> Result<(), ()> + Send + Sync>,
}

#[derive(Default)]
struct AppConfigurationErrorState {
    message: Option<&'static str>,
    recovery_required: bool,
}

impl AppConfigurationErrorState {
    fn projected(&self) -> Option<&'static str> {
        self.message
    }

    fn set(&mut self, message: &'static str) {
        if !self.recovery_required {
            self.message = Some(message);
        }
    }

    fn set_policy_reconciled(&mut self, message: &'static str) {
        self.message = Some(message);
        self.recovery_required = false;
    }

    fn set_recovery_required(&mut self, message: &'static str) {
        self.message = Some(message);
        self.recovery_required = true;
    }

    fn clear(&mut self, policy_reconciled: bool) {
        if policy_reconciled || !self.recovery_required {
            self.message = None;
            self.recovery_required = false;
        }
    }
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
        let errors = Arc::new(Mutex::new(AppConfigurationErrorState::default()));
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
        let callback_error = Arc::clone(&errors);
        let callback_activation = Arc::clone(&activation_coordinator);
        let watcher = ParentDirectoryConfigurationWatcher::start(
            plan,
            move |notice| match notice {
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
                                        error.set("configuration policy activation gate failed");
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
                        let mut policy_reconciled = false;
                        if let Ok((ConfigurationUpdate::Activated { snapshot, record }, prior)) =
                            &result
                            && let Some(activation) = activation.as_ref()
                        {
                            if (activation.observer)(Some(record.clone())).is_err() {
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
                                    if policy_compensated {
                                        error.set_policy_reconciled(
                                            "configuration policy activation failed and was rolled back",
                                        );
                                    } else {
                                        error.set_recovery_required(
                                            "configuration policy activation failed; recovery is required",
                                        );
                                    }
                                }
                            } else {
                                policy_reconciled = true;
                            }
                        }
                        if let Ok(mut error) = callback_error.lock() {
                            match &result {
                                Err(_) => error.set("configuration watcher activation failed"),
                                Ok((ConfigurationUpdate::Rejected { .. }, _)) => {
                                    error.set("configuration watcher edit was rejected");
                                }
                                Ok((
                                    ConfigurationUpdate::Activated { .. }
                                    | ConfigurationUpdate::Unchanged { .. }
                                    | ConfigurationUpdate::DeduplicatedSelfWrite { .. },
                                    _,
                                )) if !observer_failed => {
                                    error.clear(policy_reconciled);
                                }
                                Ok(_) => {}
                            }
                        }
                    }
                }
                ConfigurationWatcherNotice::Error(_) => {
                    if let Ok(mut error) = callback_error.lock() {
                        error.set("configuration watcher failed");
                    }
                }
            },
        )?;

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
            errors,
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
        self.errors
            .lock()
            .ok()
            .and_then(|errors| errors.projected())
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
        if observer(self.last_known_good()?).is_err() {
            if let Ok(mut errors) = self.errors.lock() {
                errors.set_recovery_required(
                    "configuration policy activation failed; recovery is required",
                );
            }
            return Err(ConfigurationPersistenceError::RecoveryRequired);
        }
        if let Ok(mut errors) = self.errors.lock() {
            errors.clear(true);
        }
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

#[derive(Default)]
struct WorkspaceConfigurationErrorState {
    refresh_error: Option<&'static str>,
    watcher_error: Option<&'static str>,
    target_errors: BTreeMap<WatchedConfiguration, WorkspaceConfigurationTargetError>,
}

#[derive(Clone, Copy)]
struct WorkspaceConfigurationTargetError {
    message: &'static str,
    recovery_required: bool,
}

impl WorkspaceConfigurationErrorState {
    fn projected(&self) -> Option<&'static str> {
        self.watcher_error.or(self.refresh_error).or_else(|| {
            self.target_errors
                .values()
                .next()
                .map(|error| error.message)
        })
    }

    fn set_target(&mut self, target: WatchedConfiguration, error: &'static str) {
        if self
            .target_errors
            .get(&target)
            .is_some_and(|error| error.recovery_required)
        {
            return;
        }
        self.target_errors.insert(
            target,
            WorkspaceConfigurationTargetError {
                message: error,
                recovery_required: false,
            },
        );
    }

    fn set_target_policy_reconciled(&mut self, target: WatchedConfiguration, error: &'static str) {
        self.target_errors.insert(
            target,
            WorkspaceConfigurationTargetError {
                message: error,
                recovery_required: false,
            },
        );
    }

    fn set_target_recovery_required(&mut self, target: WatchedConfiguration, error: &'static str) {
        self.target_errors.insert(
            target,
            WorkspaceConfigurationTargetError {
                message: error,
                recovery_required: true,
            },
        );
    }

    fn clear_target(&mut self, target: &WatchedConfiguration, policy_reconciled: bool) {
        if policy_reconciled
            || self
                .target_errors
                .get(target)
                .is_some_and(|error| !error.recovery_required)
        {
            self.target_errors.remove(target);
        }
    }
}

#[derive(Default)]
struct WorkspaceConfigurationRefreshState {
    epoch: u64,
    pending_epoch: Option<u64>,
    pending_reconciliation_targets: BTreeSet<WatchedConfiguration>,
    worker_running: bool,
}

impl WorkspaceConfigurationRefreshState {
    fn begin(&mut self) -> u64 {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.epoch = 1;
        }
        self.pending_epoch = Some(self.epoch);
        self.epoch
    }
}

fn settle_workspace_configuration_refresh_epoch(
    state: &mut WorkspaceConfigurationRefreshState,
    errors: &mut WorkspaceConfigurationErrorState,
    epoch: u64,
    succeeded: bool,
) -> bool {
    if state.pending_epoch != Some(epoch) {
        return state.pending_epoch.is_some();
    }
    if succeeded && state.pending_reconciliation_targets.is_empty() {
        state.pending_epoch = None;
        errors.refresh_error = None;
    } else {
        errors.refresh_error = Some(WORKSPACE_CONFIGURATION_TARGET_REFRESH_FAILED);
    }
    state.pending_epoch.is_some()
}

/// Long-lived configuration authority retained by every writable Workspace.
/// It serializes every scope identity through one generation allocator while
/// watcher callbacks and UI saves publish to the same SQLite LKG store.
struct ManagedWorkspaceConfiguration {
    database: Arc<DatabaseActor>,
    root: PathBuf,
    workspace_id: Uuid,
    state: Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    watcher: Arc<Mutex<ParentDirectoryConfigurationWatcher>>,
    errors: Arc<Mutex<WorkspaceConfigurationErrorState>>,
    activation_coordinator: Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
    refresh_state: Arc<Mutex<WorkspaceConfigurationRefreshState>>,
    refresh_gate: Arc<Mutex<()>>,
    refresh_retry_alive: Arc<AtomicBool>,
    refresh_retry_workers: Mutex<Vec<thread::JoinHandle<()>>>,
}

#[derive(Clone)]
struct WorkspaceConfigurationActivationCoordinator {
    gate: Arc<Mutex<()>>,
    observer: Arc<dyn Fn() -> Result<(), ()> + Send + Sync>,
}

#[derive(Clone)]
struct WorkspaceConfigurationRefreshContext {
    database: Arc<DatabaseActor>,
    root: PathBuf,
    workspace_id: Uuid,
    state: Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    watcher: Arc<Mutex<ParentDirectoryConfigurationWatcher>>,
    errors: Arc<Mutex<WorkspaceConfigurationErrorState>>,
    activation_coordinator: Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
    refresh_state: Arc<Mutex<WorkspaceConfigurationRefreshState>>,
    refresh_gate: Arc<Mutex<()>>,
}

const WORKSPACE_CONFIGURATION_TARGET_REFRESH_FAILED: &str =
    "Workspace configuration watcher target refresh failed";

impl WorkspaceConfigurationRefreshContext {
    fn refresh(&self) -> Result<Vec<WatchedConfiguration>, ConfigurationPersistenceError> {
        refresh_workspace_configuration_targets(
            &self.database,
            &self.root,
            self.workspace_id,
            &self.state,
            &self.watcher,
        )
    }

    fn attempt(&self, epoch: u64) -> Option<bool> {
        let _gate = self.refresh_gate.lock().ok()?;
        if self
            .refresh_state
            .lock()
            .ok()
            .is_none_or(|state| state.pending_epoch != Some(epoch))
        {
            return None;
        }
        let added_targets = match self.refresh() {
            Ok(targets) => targets,
            Err(_) => return Some(false),
        };
        let reconciliation_targets = {
            let mut refresh = self.refresh_state.lock().ok()?;
            refresh.pending_reconciliation_targets.extend(added_targets);
            if refresh.pending_epoch != Some(epoch) {
                return None;
            }
            refresh
                .pending_reconciliation_targets
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        };
        for target in reconciliation_targets {
            if reconcile_workspace_configuration_target(
                &self.database,
                &self.root,
                self.workspace_id,
                &self.state,
                &self.errors,
                &self.activation_coordinator,
                target.clone(),
            )
            .is_err()
            {
                return Some(false);
            }
            let mut refresh = self.refresh_state.lock().ok()?;
            refresh.pending_reconciliation_targets.remove(&target);
            if refresh.pending_epoch != Some(epoch) {
                return None;
            }
        }
        Some(true)
    }

    fn settle(&self, epoch: u64, succeeded: bool) -> bool {
        let Ok(mut state) = self.refresh_state.lock() else {
            return false;
        };
        if state.pending_epoch != Some(epoch) {
            return state.pending_epoch.is_some();
        }
        let Ok(mut errors) = self.errors.lock() else {
            return false;
        };
        settle_workspace_configuration_refresh_epoch(&mut state, &mut errors, epoch, succeeded)
    }
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
        let errors = Arc::new(Mutex::new(WorkspaceConfigurationErrorState::default()));
        let activation_coordinator = Arc::new(Mutex::new(
            None::<WorkspaceConfigurationActivationCoordinator>,
        ));
        let watcher = start_workspace_configuration_watcher(
            Arc::clone(&database),
            root.clone(),
            workspace_id,
            Arc::clone(&state),
            Arc::clone(&errors),
            Arc::clone(&activation_coordinator),
            &identities,
        )?;
        Ok(Self {
            database,
            root,
            workspace_id,
            state,
            watcher: Arc::new(Mutex::new(watcher)),
            errors,
            activation_coordinator,
            refresh_state: Arc::new(Mutex::new(WorkspaceConfigurationRefreshState::default())),
            refresh_gate: Arc::new(Mutex::new(())),
            refresh_retry_alive: Arc::new(AtomicBool::new(true)),
            refresh_retry_workers: Mutex::new(Vec::new()),
        })
    }

    fn refresh_context(&self) -> WorkspaceConfigurationRefreshContext {
        WorkspaceConfigurationRefreshContext {
            database: Arc::clone(&self.database),
            root: self.root.clone(),
            workspace_id: self.workspace_id,
            state: Arc::clone(&self.state),
            watcher: Arc::clone(&self.watcher),
            errors: Arc::clone(&self.errors),
            activation_coordinator: Arc::clone(&self.activation_coordinator),
            refresh_state: Arc::clone(&self.refresh_state),
            refresh_gate: Arc::clone(&self.refresh_gate),
        }
    }

    fn schedule_target_refresh_retry(&self) {
        let should_spawn = self.refresh_state.lock().is_ok_and(|mut state| {
            if state.pending_epoch.is_none() || state.worker_running {
                false
            } else {
                state.worker_running = true;
                true
            }
        });
        if !should_spawn {
            return;
        }

        let context = self.refresh_context();
        let alive = Arc::clone(&self.refresh_retry_alive);
        let spawn = thread::Builder::new()
            .name("c4os-workspace-configuration-refresh".into())
            .spawn(move || {
                let mut delay = Duration::from_millis(25);
                loop {
                    thread::sleep(delay);
                    if !alive.load(Ordering::Acquire) {
                        if let Ok(mut state) = context.refresh_state.lock() {
                            state.worker_running = false;
                        }
                        return;
                    }
                    let epoch = match context.refresh_state.lock() {
                        Ok(mut state) => match state.pending_epoch {
                            Some(epoch) => epoch,
                            None => {
                                state.worker_running = false;
                                return;
                            }
                        },
                        Err(_) => return,
                    };
                    let succeeded = context.attempt(epoch);
                    if let Some(succeeded) = succeeded {
                        context.settle(epoch, succeeded);
                        if succeeded {
                            delay = Duration::from_millis(25);
                        } else {
                            delay = delay.saturating_mul(2).min(Duration::from_millis(500));
                        }
                    }
                    let mut state = match context.refresh_state.lock() {
                        Ok(state) => state,
                        Err(_) => return,
                    };
                    if state.pending_epoch.is_none() {
                        state.worker_running = false;
                        return;
                    }
                }
            });
        match spawn {
            Ok(worker) => {
                register_workspace_configuration_refresh_worker(
                    &self.refresh_retry_workers,
                    worker,
                );
            }
            Err(_) => {
                if let Ok(mut state) = self.refresh_state.lock() {
                    state.worker_running = false;
                }
            }
        }
    }

    /// Refreshing watcher coverage is ancillary once an operation has made a
    /// durable commit. Preserve the committed success and surface a degraded
    /// watcher diagnostic while a single owned retry worker repairs coverage
    /// without falsely telling the caller that the state change failed.
    fn refresh_targets_after_commit(&self) {
        let epoch = match self.refresh_state.lock() {
            Ok(mut state) => state.begin(),
            Err(_) => return,
        };
        let context = self.refresh_context();
        let succeeded = context.attempt(epoch).unwrap_or(false);
        if context.settle(epoch, succeeded) {
            self.schedule_target_refresh_retry();
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
        let target = WatchedConfiguration {
            scope: identity.scope,
            path: path.clone(),
        };
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
            if let Ok(mut errors) = self.errors.lock() {
                if policy_compensated {
                    errors.set_target_policy_reconciled(
                        target.clone(),
                        "Workspace configuration policy activation failed and was rolled back",
                    );
                } else {
                    errors.set_target_recovery_required(
                        target.clone(),
                        "Workspace configuration policy activation failed; recovery is required",
                    );
                }
            }
            drop(_activation_guard);
            self.refresh_targets_after_commit();
            return Err(if policy_compensated {
                ConfigurationPersistenceError::PolicyActivation
            } else {
                ConfigurationPersistenceError::RecoveryRequired
            });
        }
        if let Ok(mut errors) = self.errors.lock() {
            match &update {
                ConfigurationUpdate::Activated { .. } => {
                    errors.clear_target(&target, activation.is_some());
                }
                ConfigurationUpdate::Unchanged { .. } => {
                    errors.clear_target(&target, false);
                }
                ConfigurationUpdate::Rejected { .. }
                | ConfigurationUpdate::DeduplicatedSelfWrite { .. } => {}
            }
        }
        drop(_activation_guard);
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
        self.errors
            .lock()
            .ok()
            .and_then(|errors| errors.projected())
    }
}

fn register_workspace_configuration_refresh_worker(
    workers: &Mutex<Vec<thread::JoinHandle<()>>>,
    worker: thread::JoinHandle<()>,
) {
    let finished = {
        let mut workers = workers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut finished = Vec::new();
        let mut index = 0;
        while index < workers.len() {
            if workers[index].is_finished() {
                finished.push(workers.swap_remove(index));
            } else {
                index += 1;
            }
        }
        workers.push(worker);
        finished
    };
    for worker in finished {
        let _ = worker.join();
    }
}

impl Drop for ManagedWorkspaceConfiguration {
    fn drop(&mut self) {
        self.refresh_retry_alive.store(false, Ordering::Release);
        if let Ok(mut state) = self.refresh_state.lock() {
            state.pending_epoch = None;
        }
        if let Ok(workers) = self.refresh_retry_workers.get_mut() {
            for worker in workers.drain(..) {
                let _ = worker.join();
            }
        }
    }
}

fn refresh_workspace_configuration_targets(
    database: &Arc<DatabaseActor>,
    root: &Path,
    workspace_id: Uuid,
    state: &Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    watcher: &Arc<Mutex<ParentDirectoryConfigurationWatcher>>,
) -> Result<Vec<WatchedConfiguration>, ConfigurationPersistenceError> {
    let (snapshot, _) = database.complete_workspace_snapshot(true)?;
    let identities = workspace_configuration_identities(&snapshot, workspace_id)?;
    ensure_workspace_configuration_parents(root, &identities)?;
    {
        let mut state = state
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        synchronize_workspace_configuration_services(
            &mut state,
            root,
            workspace_id,
            &snapshot,
            &identities,
        )?;
    }
    let targets = workspace_configuration_targets(root, &identities)?;
    let added_targets = {
        let mut watcher = watcher
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        let added_targets = targets
            .iter()
            .filter(|target| !watcher.plan().targets.contains(*target))
            .cloned()
            .collect::<Vec<_>>();
        watcher
            .extend_targets(targets)
            .map_err(ConfigurationPersistenceError::Configuration)?;
        added_targets
    };
    Ok(added_targets)
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
        self.workspace.recovery_notice = None;
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
    if persisted.is_none() {
        migrate_legacy_app_configuration_placeholder(home)?;
    }
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

/// Upgrades the exact comment-only file written by pre-schema builds. Other
/// invalid or externally edited documents remain untouched for explicit user
/// recovery through the strict configuration surface.
fn migrate_legacy_app_configuration_placeholder(
    home: &C4osHomeLayout,
) -> Result<(), ConfigurationPersistenceError> {
    let path = home.app_configuration();
    let Some(bytes) = read_optional_configuration(&path)? else {
        return Ok(());
    };
    if bytes.as_slice() != b"# C4OS configuration\n" {
        return Ok(());
    }
    let canonical_toml = toml::to_string(&super::configuration::ConfigurationDocument::default())
        .map_err(|_| ConfigurationPersistenceError::InvalidRecovery)?;
    replace_legacy_scope_placeholder(ConfigurationScope::App, &path, &bytes, &canonical_toml, 0)?;
    Ok(())
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

struct ReconciledMissingWorkspaceConfiguration {
    prior: Option<LastKnownGoodDocument>,
    update: ConfigurationUpdate,
}

fn reconcile_missing_workspace_configuration(
    state: &mut WorkspaceConfigurationCoordinatorState,
    identity: WorkspaceConfigurationIdentity,
    path: &Path,
) -> Result<Option<ReconciledMissingWorkspaceConfiguration>, ConfigurationPersistenceError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let service = state
                .services
                .get_mut(&identity)
                .ok_or(ConfigurationPersistenceError::InvalidScope)?;
            let prior = service.last_known_good(identity.scope).cloned();
            if let Some(record) = prior.as_ref() {
                recover_missing_scope_file(
                    identity.scope,
                    path,
                    &record.canonical_toml,
                    record.activated_generation,
                )?;
            }
            Ok(Some(ReconciledMissingWorkspaceConfiguration {
                prior,
                update: ConfigurationUpdate::Unchanged {
                    snapshot: service.snapshot(),
                },
            }))
        }
        _ => Ok(None),
    }
}

fn workspace_configuration_targets(
    root: &Path,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<Vec<WatchedConfiguration>, ConfigurationPersistenceError> {
    identities
        .iter()
        .map(|identity| {
            Ok(WatchedConfiguration {
                scope: identity.scope,
                path: configuration_path(root, identity.scope, &identity.persisted_scope_id())
                    .map_err(|_| ConfigurationPersistenceError::InvalidScope)?,
            })
        })
        .collect()
}

fn start_workspace_configuration_watcher(
    database: Arc<DatabaseActor>,
    root: PathBuf,
    workspace_id: Uuid,
    state: Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    errors: Arc<Mutex<WorkspaceConfigurationErrorState>>,
    activation_coordinator: Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
    identities: &BTreeSet<WorkspaceConfigurationIdentity>,
) -> Result<ParentDirectoryConfigurationWatcher, ConfigurationPersistenceError> {
    let targets = workspace_configuration_targets(&root, identities)?;
    let plan = ConfigurationWatcherPlan::new(targets).map_err(configuration_io)?;
    ParentDirectoryConfigurationWatcher::start(plan, move |notice| match notice {
        ConfigurationWatcherNotice::Changed(targets) => {
            if let Ok(mut errors) = errors.lock() {
                errors.watcher_error = None;
            }
            for target in targets {
                let _ = reconcile_workspace_configuration_target(
                    &database,
                    &root,
                    workspace_id,
                    &state,
                    &errors,
                    &activation_coordinator,
                    target,
                );
            }
        }
        ConfigurationWatcherNotice::Error(_) => {
            if let Ok(mut errors) = errors.lock() {
                errors.watcher_error = Some("Workspace configuration watcher failed");
            }
        }
    })
    .map_err(ConfigurationPersistenceError::Configuration)
}

enum WorkspaceConfigurationTargetReconciliation {
    Updated,
    ContentRejected,
}

fn reconcile_workspace_configuration_target(
    database: &Arc<DatabaseActor>,
    root: &Path,
    workspace_id: Uuid,
    state: &Arc<Mutex<WorkspaceConfigurationCoordinatorState>>,
    errors: &Arc<Mutex<WorkspaceConfigurationErrorState>>,
    activation_coordinator: &Arc<Mutex<Option<WorkspaceConfigurationActivationCoordinator>>>,
    target: WatchedConfiguration,
) -> Result<WorkspaceConfigurationTargetReconciliation, ConfigurationPersistenceError> {
    let activation = activation_coordinator
        .lock()
        .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?
        .clone();
    let _activation_guard = match activation.as_ref() {
        Some(coordinator) => match coordinator.gate.lock() {
            Ok(guard) => Some(guard),
            Err(_) => {
                if let Ok(mut errors) = errors.lock() {
                    errors.set_target(
                        target,
                        "Workspace configuration policy activation gate failed",
                    );
                }
                return Err(ConfigurationPersistenceError::CoordinatorUnavailable);
            }
        },
        None => None,
    };
    let identity = workspace_configuration_identity_for_target(root, workspace_id, &target)?;
    let missing = {
        let mut state = state
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        reconcile_missing_workspace_configuration(&mut state, identity, &target.path)?
    };
    let result = if let Some(missing) = missing {
        Ok((identity, missing.prior, missing.update))
    } else {
        let text = match stable_scope_text(identity.scope, &target.path) {
            Ok(text) => text,
            Err(diagnostic) => {
                if let Ok(mut state) = state.lock()
                    && let Some(service) = state.services.get_mut(&identity)
                {
                    service.record_diagnostic(diagnostic);
                }
                if let Ok(mut errors) = errors.lock() {
                    errors.set_target(target, "Workspace configuration watcher activation failed");
                }
                return Ok(WorkspaceConfigurationTargetReconciliation::ContentRejected);
            }
        };
        let mut state = state
            .lock()
            .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
        let prior = state
            .services
            .get(&identity)
            .and_then(|service| service.last_known_good(identity.scope).cloned());
        commit_external_workspace_configuration(
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
        )
        .map(|update| (identity, prior, update))
    };
    let policy_result = match (&result, activation.as_ref()) {
        (Ok((_, _, ConfigurationUpdate::Activated { .. })), Some(coordinator)) => {
            (coordinator.observer)().map(|_| true)
        }
        _ => Ok(false),
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
                        toml::to_string(&super::configuration::ConfigurationDocument::default())
                            .ok()
                    });
                rollback_text.is_some_and(|rollback_text| {
                    state.lock().is_ok_and(|mut state| {
                        compensate_workspace_configuration_activation(
                            database,
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
    let mut errors = errors
        .lock()
        .map_err(|_| ConfigurationPersistenceError::CoordinatorUnavailable)?;
    if policy_result.is_err() {
        if compensated {
            errors.set_target_policy_reconciled(
                target,
                "Workspace configuration policy activation failed and was rolled back",
            );
            return Err(ConfigurationPersistenceError::PolicyActivation);
        }
        errors.set_target_recovery_required(
            target,
            "Workspace configuration policy activation failed; recovery is required",
        );
        return Err(ConfigurationPersistenceError::RecoveryRequired);
    }
    match &result {
        Err(_) | Ok((_, _, ConfigurationUpdate::Rejected { .. })) => {
            errors.set_target(target, "Workspace configuration watcher activation failed");
        }
        Ok((_, _, ConfigurationUpdate::Activated { .. })) => {
            errors.clear_target(&target, policy_result.unwrap_or(false));
        }
        Ok((_, _, ConfigurationUpdate::Unchanged { .. })) => {
            errors.clear_target(&target, false);
        }
        Ok((_, _, ConfigurationUpdate::DeduplicatedSelfWrite { .. })) => {}
    }
    drop(errors);
    result.map(|_| WorkspaceConfigurationTargetReconciliation::Updated)
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

/// Injectable creation checkpoints used by host-level fault matrices. The
/// production entrypoint supplies a no-op lifecycle; alternate callers can
/// deterministically reject coordinator startup or one cleanup attempt without
/// gaining any Workspace persistence authority.
#[doc(hidden)]
pub trait WorkspaceCreationLifecycle: Send + Sync {
    fn before_configuration_coordinator_start(&self) -> WorkspaceResult<()> {
        Ok(())
    }

    fn before_cleanup(&self) -> io::Result<()> {
        Ok(())
    }
}

struct ProductionWorkspaceCreationLifecycle;

impl WorkspaceCreationLifecycle for ProductionWorkspaceCreationLifecycle {}

struct PendingWorkspaceCreation {
    root: PathBuf,
    preserve_empty_root: bool,
    recovery_root: Option<(PathBuf, bool)>,
    previous_active: Option<PathBuf>,
    workspace_parent: Option<PathBuf>,
    recovery_parent: Option<PathBuf>,
    finished: bool,
    lifecycle: Arc<dyn WorkspaceCreationLifecycle>,
}

impl PendingWorkspaceCreation {
    fn with_lifecycle(
        root: &Path,
        lifecycle: Arc<dyn WorkspaceCreationLifecycle>,
    ) -> WorkspaceResult<Self> {
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
            previous_active: None,
            workspace_parent: None,
            recovery_parent: None,
            finished: false,
            lifecycle,
        })
    }

    fn with_preserved_active(
        home: &C4osHomeLayout,
        root: &Path,
        lifecycle: Arc<dyn WorkspaceCreationLifecycle>,
    ) -> WorkspaceResult<Self> {
        let metadata = match fs::symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Self::with_lifecycle(root, lifecycle);
            }
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(WorkspaceError::Conflict(
                "untitled working-copy destination is not a directory".into(),
            ));
        }
        if fs::read_dir(root)?.next().is_none() {
            return Self::with_lifecycle(root, lifecycle);
        }

        let workspace_parent = root
            .parent()
            .ok_or_else(|| WorkspaceError::Conflict("active Workspace has no parent".into()))?
            .to_path_buf();
        let recovery_parent = home.workspace_recovery_root();
        let previous_active = recovery_parent.join(format!("preserved-active-{}", Uuid::new_v4()));
        commit_rename(root, &previous_active)?;
        if let Err(error) = commit_sync_directory(&recovery_parent)
            .and_then(|_| commit_sync_directory(&workspace_parent))
        {
            return match Self::restore_previous_active(
                root,
                &previous_active,
                &workspace_parent,
                &recovery_parent,
            ) {
                Ok(()) => Err(error.into()),
                Err(rollback_error) => Err(WorkspaceError::CommitRecovery {
                    operation: "prepare create Workspace",
                    recovery_path: previous_active,
                    message: rollback_error.to_string(),
                }),
            };
        }

        let mut pending = match Self::with_lifecycle(root, lifecycle) {
            Ok(pending) => pending,
            Err(error) => {
                return match Self::restore_previous_active(
                    root,
                    &previous_active,
                    &workspace_parent,
                    &recovery_parent,
                ) {
                    Ok(()) => Err(error),
                    Err(rollback_error) => Err(WorkspaceError::CommitRecovery {
                        operation: "prepare create Workspace",
                        recovery_path: previous_active,
                        message: rollback_error.to_string(),
                    }),
                };
            }
        };
        pending.previous_active = Some(previous_active);
        pending.workspace_parent = Some(workspace_parent);
        pending.recovery_parent = Some(recovery_parent);
        Ok(pending)
    }

    fn before_configuration_coordinator_start(&self) -> WorkspaceResult<()> {
        self.lifecycle.before_configuration_coordinator_start()
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
        self.lifecycle.before_cleanup()?;
        let recovery_result = self
            .recovery_root
            .as_ref()
            .map_or(Ok(()), |(root, preserve)| {
                Self::cleanup_root(root, *preserve)
            });
        let active_result = Self::cleanup_root(&self.root, self.preserve_empty_root);
        let restore_result = match (
            self.previous_active.as_ref(),
            self.workspace_parent.as_ref(),
            self.recovery_parent.as_ref(),
        ) {
            (Some(previous), Some(workspace_parent), Some(recovery_parent)) => {
                Self::restore_previous_active(
                    &self.root,
                    previous,
                    workspace_parent,
                    recovery_parent,
                )
            }
            _ => Ok(()),
        };
        recovery_result.and(active_result).and(restore_result)
    }

    fn restore_previous_active(
        root: &Path,
        previous_active: &Path,
        workspace_parent: &Path,
        recovery_parent: &Path,
    ) -> io::Result<()> {
        commit_rename(previous_active, root)?;
        commit_sync_directory(recovery_parent)?;
        commit_sync_directory(workspace_parent)
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
    create_workspace_from_project_with_lifecycle(
        home,
        project_folder,
        project_display_name,
        workspace_display_name,
        current_app_version,
        lock_owner,
        created_at,
        Arc::new(ProductionWorkspaceCreationLifecycle),
    )
}

/// Runs the public Workspace creation transaction with deterministic lifecycle
/// checkpoints. This is intentionally not exposed through the renderer or
/// command layer; it exists so host integration tests can prove rollback and
/// recovery behavior at otherwise unreachable operating-system boundaries.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub fn create_workspace_from_project_with_lifecycle(
    home: &C4osHomeLayout,
    project_folder: &Path,
    project_display_name: &str,
    workspace_display_name: &str,
    current_app_version: &str,
    lock_owner: WorkspaceLockOwner,
    created_at: i64,
    lifecycle: Arc<dyn WorkspaceCreationLifecycle>,
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
    let mut pending_creation =
        PendingWorkspaceCreation::with_preserved_active(home, &working_root, lifecycle)?;
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
        pending_creation.before_configuration_coordinator_start()?;
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

/// Restores an unsaved active working copy only when a fresh native picker
/// grant names its exact Project folder. A different Project returns the
/// caller to the create path, which preserves the prior working copy before
/// switching. Startup restoration deliberately drops Project trust; this
/// explicit user action re-establishes trust for the matched Project without
/// treating persisted paths as authority.
pub fn restore_active_workspace_for_project(
    home: &C4osHomeLayout,
    project_folder: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
    lock_owner: WorkspaceLockOwner,
) -> WorkspaceResult<Option<ActiveWorkspace>> {
    if !project_folder.is_dir() {
        return Err(WorkspaceError::InvalidProject(
            "Project folder is unavailable".into(),
        ));
    }
    let Some(mut workspace) =
        restore_active_workspace(home, true, current_app_version, limits, lock_owner)?
    else {
        return Ok(None);
    };
    let selected = fs::canonicalize(project_folder)?;
    let (snapshot, _) = workspace
        .database
        .complete_workspace_snapshot(false)
        .map_err(database_conflict)?;
    let matching = snapshot
        .projects
        .iter()
        .filter(|project| project.lifecycle_state == LifecycleState::Active)
        .filter_map(|project| {
            let current = fs::canonicalize(&project.current_path).ok()?;
            (current == selected).then_some(project.project_id.as_str())
        })
        .collect::<Vec<_>>();
    let [project_id] = matching.as_slice() else {
        return Ok(None);
    };
    let project_id = Uuid::parse_str(project_id)
        .map_err(|_| WorkspaceError::Conflict("active Project identity is invalid".into()))?;
    workspace.resolve_project_trust(project_id, true)?;
    Ok(Some(workspace))
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
    fn restore_app_configuration_migrates_the_legacy_comment_only_placeholder() {
        let temp = TempDir::new().expect("temporary legacy configuration root");
        let home = C4osHomeLayout::new(temp.path());
        fs::write(home.app_configuration(), "# C4OS configuration\n").expect("legacy placeholder");
        let (database, _) =
            DatabaseActor::start(DatabaseDescriptor::app(temp.path())).expect("app database");

        let configuration = restore_app_configuration(
            &database,
            &home,
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("legacy placeholder migration");

        let record = configuration
            .last_known_good(ConfigurationScope::App)
            .expect("migrated app configuration");
        assert!(record.canonical_toml.starts_with("schema_version = 1\n"));
        assert_eq!(
            fs::read_to_string(home.app_configuration()).expect("migrated file"),
            record.canonical_toml
        );
        assert!(
            database
                .app_configuration_lkg()
                .expect("persisted app configuration")
                .is_some()
        );
    }

    #[test]
    fn restore_app_configuration_does_not_replace_a_similar_external_document() {
        let temp = TempDir::new().expect("temporary external configuration root");
        let home = C4osHomeLayout::new(temp.path());
        let external = "  # C4OS configuration\n";
        fs::write(home.app_configuration(), external).expect("external configuration");
        let (database, _) =
            DatabaseActor::start(DatabaseDescriptor::app(temp.path())).expect("app database");

        let configuration = restore_app_configuration(
            &database,
            &home,
            ManagedCeilings::default(),
            SecurityConstraints::default(),
        )
        .expect("rejected external configuration remains recoverable");

        assert!(
            configuration
                .last_known_good(ConfigurationScope::App)
                .is_none()
        );
        assert_eq!(
            fs::read_to_string(home.app_configuration()).expect("preserved external file"),
            external
        );
        assert!(
            database
                .app_configuration_lkg()
                .expect("no persisted replacement")
                .is_none()
        );
    }

    #[test]
    fn stale_refresh_success_cannot_clear_a_newer_failed_epoch() {
        let mut state = WorkspaceConfigurationRefreshState::default();
        let mut errors = WorkspaceConfigurationErrorState::default();
        let first = state.begin();
        let second = state.begin();
        assert!(settle_workspace_configuration_refresh_epoch(
            &mut state,
            &mut errors,
            second,
            false,
        ));

        assert!(settle_workspace_configuration_refresh_epoch(
            &mut state,
            &mut errors,
            first,
            true,
        ));
        assert_eq!(state.pending_epoch, Some(second));
        assert_eq!(
            errors.refresh_error,
            Some(WORKSPACE_CONFIGURATION_TARGET_REFRESH_FAILED)
        );

        assert!(!settle_workspace_configuration_refresh_epoch(
            &mut state,
            &mut errors,
            second,
            true,
        ));
        assert_eq!(state.pending_epoch, None);
        assert_eq!(errors.refresh_error, None);
    }

    #[test]
    fn successful_target_cannot_clear_an_unrelated_target_error() {
        let temp = TempDir::new().expect("temporary target-error root");
        let project = WatchedConfiguration {
            scope: ConfigurationScope::Project,
            path: temp.path().join("project/config.toml"),
        };
        let chat = WatchedConfiguration {
            scope: ConfigurationScope::Chat,
            path: temp.path().join("chat/config.toml"),
        };
        let mut errors = WorkspaceConfigurationErrorState::default();
        errors.set_target(
            project.clone(),
            "Workspace configuration watcher activation failed",
        );
        errors.set_target_recovery_required(
            chat.clone(),
            "Workspace configuration policy activation failed; recovery is required",
        );

        errors.clear_target(&project, false);

        assert!(!errors.target_errors.contains_key(&project));
        assert_eq!(
            errors.target_errors.get(&chat).map(|error| error.message),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
        assert_eq!(
            errors.projected(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
    }

    #[test]
    fn refresh_worker_registration_reaps_completed_handles_and_retains_live_workers() {
        let workers = Mutex::new(Vec::new());
        let completed = thread::spawn(|| {});
        while !completed.is_finished() {
            thread::yield_now();
        }
        let (release_live, wait_for_release) = std::sync::mpsc::channel();
        let live = thread::spawn(move || {
            wait_for_release.recv().expect("live worker release");
        });
        {
            let mut registered = workers.lock().expect("worker registry");
            registered.push(completed);
            registered.push(live);
        }

        register_workspace_configuration_refresh_worker(&workers, thread::spawn(|| {}));

        assert_eq!(
            workers.lock().expect("reaped worker registry").len(),
            2,
            "one completed handle is reaped while the live and new workers remain registered"
        );
        release_live.send(()).expect("release live worker");
        let remaining = workers
            .into_inner()
            .expect("remaining worker registry after assertion");
        for worker in remaining {
            worker.join().expect("registered worker completion");
        }
    }

    #[test]
    fn missing_workspace_configuration_is_benign_without_lkg_and_recovers_with_lkg() {
        let temp = TempDir::new().expect("temporary configuration root");
        let identity = WorkspaceConfigurationIdentity {
            scope: ConfigurationScope::Chat,
            scope_id: Uuid::new_v4(),
        };
        let path = temp.path().join("chat/config.toml");
        let mut state = WorkspaceConfigurationCoordinatorState {
            services: BTreeMap::from([(
                identity,
                ConfigurationService::new(
                    ManagedCeilings::default(),
                    SecurityConstraints::default(),
                ),
            )]),
            global_generation: 0,
        };

        let missing = reconcile_missing_workspace_configuration(&mut state, identity, &path)
            .expect("missing unconfigured target")
            .expect("missing target is reconciled");
        assert!(missing.prior.is_none());
        assert!(matches!(
            missing.update,
            ConfigurationUpdate::Unchanged { .. }
        ));
        assert!(
            !path.exists(),
            "a scope without LKG remains intentionally absent"
        );

        let service = state.services.get_mut(&identity).expect("Chat service");
        service
            .save_scope_text(
                ConfigurationScope::Chat,
                &path,
                "schema_version = 1\ndefault_environment = \"fixture\"\n",
                0,
            )
            .expect("seed Chat LKG");
        let canonical = service
            .last_known_good(ConfigurationScope::Chat)
            .expect("seeded Chat LKG")
            .canonical_toml
            .clone();
        fs::remove_file(&path).expect("remove LKG-owned target");

        let missing = reconcile_missing_workspace_configuration(&mut state, identity, &path)
            .expect("missing configured target")
            .expect("missing configured target is reconciled");
        assert!(missing.prior.is_some());
        assert!(matches!(
            missing.update,
            ConfigurationUpdate::Unchanged { .. }
        ));
        assert_eq!(
            fs::read_to_string(&path).expect("recovered Chat configuration"),
            canonical
        );
    }

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
    fn app_recovery_required_error_requires_policy_reconciliation_to_clear() {
        let mut errors = AppConfigurationErrorState::default();
        errors
            .set_recovery_required("configuration policy activation failed; recovery is required");

        errors.set("configuration watcher edit was rejected");
        errors.clear(false);
        assert_eq!(
            errors.projected(),
            Some("configuration policy activation failed; recovery is required")
        );

        errors.clear(true);
        assert_eq!(errors.projected(), None);
    }

    #[test]
    fn app_compensation_self_write_preserves_recovery_required_until_observer_reconciliation() {
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
        let reject = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let observer_rejection = Arc::clone(&reject);
        let observer_calls = Arc::clone(&calls);
        managed
            .set_activation_observer(
                Arc::new(Mutex::new(())),
                Arc::new(move |_| {
                    observer_calls.fetch_add(1, Ordering::SeqCst);
                    if observer_rejection.load(Ordering::SeqCst) {
                        Err(())
                    } else {
                        Ok(())
                    }
                }),
            )
            .expect("initial observer reconciliation");

        reject.store(true, Ordering::SeqCst);
        let path = home.app_configuration();
        fs::create_dir_all(path.parent().expect("configuration parent"))
            .expect("configuration parent");
        fs::write(
            &path,
            "schema_version = 1\ndefault_approval_preset = \"approve_for_me\"\n",
        )
        .expect("externally activated App configuration");
        let recovery_deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            if calls.load(Ordering::SeqCst) >= 3
                && managed.last_error()
                    == Some("configuration policy activation failed; recovery is required")
            {
                break;
            }
            assert!(
                std::time::Instant::now() < recovery_deadline,
                "failed observer compensation did not enter recovery-required state"
            );
            thread::sleep(Duration::from_millis(25));
        }

        thread::sleep(Duration::from_millis(550));
        assert_eq!(
            managed.last_error(),
            Some("configuration policy activation failed; recovery is required"),
            "the compensated self-write callback must not clear unproven recovery"
        );

        managed
            .set_activation_observer(Arc::new(Mutex::new(())), Arc::new(|_| Ok(())))
            .expect("successful current-LKG observer reconciliation");
        assert_eq!(managed.last_error(), None);
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
        let mut pending = PendingWorkspaceCreation::with_lifecycle(
            &active,
            Arc::new(ProductionWorkspaceCreationLifecycle),
        )
        .expect("pending Workspace creation");
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

    #[test]
    fn workspace_target_refresh_retry_reconciles_configuration_present_before_repair() {
        let temp = TempDir::new().expect("temporary Workspace home");
        let temp_root = temp
            .path()
            .canonicalize()
            .expect("canonical temporary Workspace root");
        let home = C4osHomeLayout::new(temp_root.join("home"));
        let project = temp_root.join("project");
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
                label: "workspace-refresh-preexisting-configuration-test".into(),
            },
            1,
        )
        .expect("active Workspace");
        let project_id = workspace.manifest().projects[0].project_id;
        let chat_id = Uuid::new_v4();
        let chat_configuration =
            WorkspaceLayout::new(workspace.working_root()).chat_configuration(chat_id);
        let chat_parent = chat_configuration
            .parent()
            .expect("Chat configuration parent")
            .to_path_buf();
        fs::write(&chat_parent, b"blocks watcher coverage").expect("watcher target blocker");
        workspace
            .configuration
            .refresh_retry_alive
            .store(false, Ordering::Release);

        workspace
            .create_chat(project_id, chat_id, "Durably Created", 2)
            .expect("durable Chat creation");
        let stopped_deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            let running = workspace
                .configuration
                .refresh_state
                .lock()
                .expect("refresh state")
                .worker_running;
            if !running {
                break;
            }
            assert!(
                std::time::Instant::now() < stopped_deadline,
                "disabled refresh worker did not stop"
            );
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            workspace.configuration_last_error(),
            Some(WORKSPACE_CONFIGURATION_TARGET_REFRESH_FAILED)
        );

        let prepared_parent = workspace
            .working_root()
            .join(".prepared-chat-configuration");
        fs::create_dir(&prepared_parent).expect("prepared Chat configuration parent");
        fs::write(
            prepared_parent.join(
                chat_configuration
                    .file_name()
                    .expect("Chat configuration file name"),
            ),
            "schema_version = 1\ndefault_environment = \"recovered.chat\"\n",
        )
        .expect("preexisting Chat configuration");
        fs::remove_file(&chat_parent).expect("remove watcher target blocker");
        fs::rename(prepared_parent, &chat_parent)
            .expect("atomically restore the preexisting configuration parent");

        workspace
            .configuration
            .refresh_retry_alive
            .store(true, Ordering::Release);
        workspace.configuration.schedule_target_refresh_retry();
        let recovery_deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let snapshot = workspace
                .snapshot(SnapshotQuery::new(20).expect("snapshot query"))
                .expect("Workspace snapshot");
            if snapshot.configurations.iter().any(|record| {
                record.scope_kind == "chat"
                    && record.scope_id == chat_id.to_string()
                    && record.canonical_document.contains("recovered.chat")
            }) {
                break;
            }
            assert!(
                std::time::Instant::now() < recovery_deadline,
                "retry did not reconcile the configuration present before target repair: {:?}",
                workspace.configuration_last_error()
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(workspace.configuration_last_error(), None);
    }

    #[test]
    fn workspace_target_refresh_settles_persistent_oversized_configuration_with_one_diagnostic() {
        let temp = TempDir::new().expect("temporary Workspace home");
        let temp_root = temp
            .path()
            .canonicalize()
            .expect("canonical temporary Workspace root");
        let home = C4osHomeLayout::new(temp_root.join("home"));
        let project = temp_root.join("project");
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
                label: "workspace-refresh-oversized-configuration-test".into(),
            },
            1,
        )
        .expect("active Workspace");
        let project_id = workspace.manifest().projects[0].project_id;
        let chat_id = Uuid::new_v4();
        let chat_configuration =
            WorkspaceLayout::new(workspace.working_root()).chat_configuration(chat_id);
        let chat_parent = chat_configuration
            .parent()
            .expect("Chat configuration parent")
            .to_path_buf();
        fs::write(&chat_parent, b"blocks watcher coverage").expect("watcher target blocker");
        workspace
            .configuration
            .refresh_retry_alive
            .store(false, Ordering::Release);
        workspace
            .create_chat(project_id, chat_id, "Durably Created", 2)
            .expect("durable Chat creation");
        let stopped_deadline = std::time::Instant::now() + Duration::from_secs(2);
        while workspace
            .configuration
            .refresh_state
            .lock()
            .expect("refresh state")
            .worker_running
        {
            assert!(
                std::time::Instant::now() < stopped_deadline,
                "disabled refresh worker did not stop"
            );
            thread::sleep(Duration::from_millis(5));
        }

        let prepared_parent = workspace
            .working_root()
            .join(".prepared-oversized-configuration");
        fs::create_dir(&prepared_parent).expect("prepared Chat configuration parent");
        fs::write(
            prepared_parent.join(
                chat_configuration
                    .file_name()
                    .expect("Chat configuration file name"),
            ),
            vec![b'x'; MAX_CONFIGURATION_BYTES + 1],
        )
        .expect("persistent oversized Chat configuration");
        fs::remove_file(&chat_parent).expect("remove watcher target blocker");
        fs::rename(prepared_parent, &chat_parent)
            .expect("atomically restore the oversized configuration parent");

        workspace
            .configuration
            .refresh_retry_alive
            .store(true, Ordering::Release);
        workspace.configuration.schedule_target_refresh_retry();
        let settle_deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let settled = {
                let refresh = workspace
                    .configuration
                    .refresh_state
                    .lock()
                    .expect("refresh state");
                refresh.pending_epoch.is_none()
                    && refresh.pending_reconciliation_targets.is_empty()
                    && !refresh.worker_running
            };
            if settled {
                break;
            }
            assert!(
                std::time::Instant::now() < settle_deadline,
                "persistent content rejection kept the refresh retry active"
            );
            thread::sleep(Duration::from_millis(10));
        }

        let identity = WorkspaceConfigurationIdentity {
            scope: ConfigurationScope::Chat,
            scope_id: chat_id,
        };
        let diagnostic_count = || {
            workspace
                .configuration
                .state
                .lock()
                .expect("configuration state")
                .services
                .get(&identity)
                .expect("Chat configuration service")
                .diagnostics()
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == ConfigurationDiagnosticCode::SizeLimitExceeded
                })
                .count()
        };
        assert_eq!(diagnostic_count(), 1);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration watcher activation failed")
        );
        assert_eq!(
            workspace
                .configuration
                .errors
                .lock()
                .expect("configuration errors")
                .refresh_error,
            None
        );

        thread::sleep(Duration::from_millis(550));
        assert_eq!(diagnostic_count(), 1);
        let refresh = workspace
            .configuration
            .refresh_state
            .lock()
            .expect("refresh state");
        assert_eq!(refresh.pending_epoch, None);
        assert!(refresh.pending_reconciliation_targets.is_empty());
        assert!(!refresh.worker_running);
    }

    fn workspace_with_recovery_required_policy_error(
        label: &'static str,
    ) -> (TempDir, ActiveWorkspace, Arc<AtomicUsize>) {
        let temp = TempDir::new().expect("temporary Workspace home");
        let temp_root = temp
            .path()
            .canonicalize()
            .expect("canonical temporary Workspace root");
        let home = C4osHomeLayout::new(temp_root.join("home"));
        let project = temp_root.join("project");
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
                label: label.into(),
            },
            1,
        )
        .expect("active Workspace");
        let calls = Arc::new(AtomicUsize::new(0));
        let observer_calls = Arc::clone(&calls);
        workspace
            .set_configuration_activation_observer(
                Arc::new(Mutex::new(())),
                Arc::new(move || {
                    observer_calls.fetch_add(1, Ordering::SeqCst);
                    Err(())
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
            Err(ConfigurationPersistenceError::RecoveryRequired)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
        (temp, workspace, calls)
    }

    #[test]
    fn workspace_no_op_save_does_not_clear_recovery_required_policy_error() {
        let (_temp, mut workspace, calls) =
            workspace_with_recovery_required_policy_error("workspace-sticky-no-op-save-test");
        let workspace_id = workspace.manifest().workspace_id;
        let mut service = workspace
            .restore_configuration_stack(
                None,
                None,
                ManagedCeilings::default(),
                SecurityConstraints::default(),
            )
            .expect("compensated Workspace configuration");
        let generation = service.snapshot().generation;
        let canonical = service
            .last_known_good(ConfigurationScope::Workspace)
            .expect("compensated Workspace LKG")
            .canonical_toml
            .clone();

        let update = workspace
            .save_configuration_scope(
                &mut service,
                ConfigurationScope::Workspace,
                workspace_id,
                &canonical,
                generation,
                3,
            )
            .expect("same-content save");

        assert!(matches!(update, ConfigurationUpdate::Unchanged { .. }));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
    }

    #[test]
    fn workspace_same_content_watcher_reconciliation_does_not_clear_recovery_required_policy_error()
    {
        let (_temp, workspace, calls) = workspace_with_recovery_required_policy_error(
            "workspace-sticky-watcher-reconciliation-test",
        );
        let target = WatchedConfiguration {
            scope: ConfigurationScope::Workspace,
            path: WorkspaceLayout::new(workspace.working_root()).workspace_configuration(),
        };
        let first = reconcile_workspace_configuration_target(
            &workspace.configuration.database,
            &workspace.configuration.root,
            workspace.configuration.workspace_id,
            &workspace.configuration.state,
            &workspace.configuration.errors,
            &workspace.configuration.activation_coordinator,
            target.clone(),
        )
        .expect("consume compensated self-write");
        assert!(matches!(
            first,
            WorkspaceConfigurationTargetReconciliation::Updated
        ));
        let same_content = reconcile_workspace_configuration_target(
            &workspace.configuration.database,
            &workspace.configuration.root,
            workspace.configuration.workspace_id,
            &workspace.configuration.state,
            &workspace.configuration.errors,
            &workspace.configuration.activation_coordinator,
            target,
        )
        .expect("same-content watcher reconciliation");

        assert!(matches!(
            same_content,
            WorkspaceConfigurationTargetReconciliation::Updated
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
    }

    #[test]
    fn workspace_content_rejection_and_same_content_reconciliation_preserve_recovery_required_error()
     {
        let (_temp, workspace, calls) = workspace_with_recovery_required_policy_error(
            "workspace-sticky-content-rejection-test",
        );
        let target = WatchedConfiguration {
            scope: ConfigurationScope::Workspace,
            path: WorkspaceLayout::new(workspace.working_root()).workspace_configuration(),
        };
        let canonical = fs::read_to_string(&target.path).expect("compensated Workspace content");
        let self_write = reconcile_workspace_configuration_target(
            &workspace.configuration.database,
            &workspace.configuration.root,
            workspace.configuration.workspace_id,
            &workspace.configuration.state,
            &workspace.configuration.errors,
            &workspace.configuration.activation_coordinator,
            target.clone(),
        )
        .expect("consume compensated self-write");
        assert!(matches!(
            self_write,
            WorkspaceConfigurationTargetReconciliation::Updated
        ));

        fs::write(&target.path, vec![b'x'; MAX_CONFIGURATION_BYTES + 1])
            .expect("oversized Workspace configuration");
        let rejection = reconcile_workspace_configuration_target(
            &workspace.configuration.database,
            &workspace.configuration.root,
            workspace.configuration.workspace_id,
            &workspace.configuration.state,
            &workspace.configuration.errors,
            &workspace.configuration.activation_coordinator,
            target.clone(),
        )
        .expect("reject oversized Workspace configuration");
        assert!(matches!(
            rejection,
            WorkspaceConfigurationTargetReconciliation::ContentRejected
        ));
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );

        fs::write(&target.path, canonical).expect("restore compensated Workspace content");
        let same_content = reconcile_workspace_configuration_target(
            &workspace.configuration.database,
            &workspace.configuration.root,
            workspace.configuration.workspace_id,
            &workspace.configuration.state,
            &workspace.configuration.errors,
            &workspace.configuration.activation_coordinator,
            target,
        )
        .expect("same-content watcher reconciliation");
        assert!(matches!(
            same_content,
            WorkspaceConfigurationTargetReconciliation::Updated
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            workspace.configuration_last_error(),
            Some("Workspace configuration policy activation failed; recovery is required")
        );
    }
}
