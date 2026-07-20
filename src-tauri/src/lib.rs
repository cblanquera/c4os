pub mod core;
pub mod execution;
pub mod platform;
pub mod protocol;
pub mod runtime;
pub mod security;

use platform::{
    ColorScheme, INITIAL_REVEAL_FALLBACK_MS, InitialThemeSnapshot, InitialThemeSource,
    NativePickerRequest, NativePickerSelection, OPEN_SETTINGS_COMMAND_ID,
    PLATFORM_CONTRACT_VERSION, PickerGrantRegistry, PickerObjectKind, PickerOutcome, PickerPurpose,
    PlatformCapabilities, PlatformService, PlatformSnapshot, PlatformTarget, SETTINGS_ACCELERATOR,
    SETTINGS_MENU_ITEM_ID, SETTINGS_ROUTE,
};
use protocol::{
    FoundationSnapshot, ProtocolEnvelope, ProtocolError, ProtocolErrorCode, SnapshotRequest,
    StateGeneration, WorkspaceId, WorkspaceRecentSnapshot, WorkspaceStartSnapshot,
};
use runtime::action_bridge::{
    RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization, RuntimeExecutionReceipt,
    RuntimeGatewayDecision,
};
use runtime::broker_worker::BrokerActionApplication;
use runtime::capability::DraftRequirements;
use runtime::capability_evidence::{
    CapabilityEvidenceError, CapabilityEvidenceRegistry, CapabilityRouteEpoch,
};
use runtime::coordinator::{
    CoordinatedFirstSubmission, CoordinatedRetry, CoordinatorError, CoordinatorOperation,
    ModelPreflight, RuntimeCoordinator, RuntimeCoordinatorSnapshot,
};
use runtime::dispatch::{
    AppliedDispatchEvent, AttachmentPreflightResolution, BrokerDispatchAuthority,
    CoordinatedCancellation, CoordinatedFirstDispatch, CoordinatedRetryDispatch, DispatchError,
    DispatchIdentity, FirstDispatchOptions, RetryDispatchOptions, RuntimeDispatchPeer,
    RuntimeDispatchRegistry, coordinate_cancellation, coordinate_first_dispatch,
    coordinate_polled_events, coordinate_recovery, coordinate_retry_dispatch,
};
use runtime::dispatch_authority::{
    AuthoritativeResource, AuthoritativeResources, AuthorityMintIntent, DispatchAuthorityError,
    DispatchAuthorityRegistry, RuntimeProcessTruth, WorkerDispatchAuthority,
    authoritative_route_configuration,
};
use runtime::opencode_native::{OPENCODE_C4OS_TOOL_IDS, sha256_bytes};
use runtime::persistence::{
    DeferredSessionRepository, ProviderStateStore, RuntimeControlPlaneStore,
    RuntimePersistenceError, SupervisorStateStore,
};
use runtime::provider::{ProviderProbe, ProviderProfile, ProviderTestReport};
use runtime::session::{
    AttachmentSnapshot, FirstSubmission, RetryRequest, RuntimeKind as SessionRuntimeKind,
    SessionRecord, SessionService, TerminalAttemptOutcome,
};
use runtime::supervisor::{
    CompatibilityState, HealthState, RuntimeInstallation, RuntimeSupervisor,
};
use security::authorization::{ApprovalAnswer, LiveAuthorityState};
use security::gateway::{ExecutionPermit, NormalizedActionResult, NormalizedActionStatus};
use security::policy::{
    ActionEffect, ActionRequestOrigin, ActionReversibility, ActionScope, ActionSensitivity,
    ActionSurface, PolicyConfiguration, RepositoryState,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{
    HELP_SUBMENU_ID, Menu, MenuItemBuilder, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, FilePath};
use thiserror::Error;
use uuid::Uuid;

const BROKER_CONTEXT_TTL_MS: u64 = 24 * 60 * 60 * 1_000;

/// Exact non-secret broker-tool resources installed by the C4OS production
/// composition. Native peers and renderer intents cannot extend this set.
pub fn production_broker_authoritative_resources() -> AuthoritativeResources {
    let broker_contract_sha256 =
        sha256_bytes(b"c4os.broker-tools.v2\0c4os_propose_action\0c4os_read_resource");
    AuthoritativeResources {
        version: 1,
        records: OPENCODE_C4OS_TOOL_IDS
            .iter()
            .copied()
            .map(|tool_id| {
                (
                    tool_id.into(),
                    AuthoritativeResource {
                        kind: "broker-tool".into(),
                        version: 1,
                        sha256: broker_contract_sha256.clone(),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
struct ProductionWindowFocusFacility {
    app: tauri::AppHandle,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl runtime::opencode_broker::InstalledBrokerFacility for ProductionWindowFocusFacility {
    fn current_target_version(&mut self) -> Option<String> {
        self.app
            .get_webview_window("main")
            .map(|_| format!("c4os-{}", env!("CARGO_PKG_VERSION")))
    }

    fn execute(&mut self, permit: ExecutionPermit) -> NormalizedActionResult {
        let completed_at_ms = permit.consumed_at_ms().saturating_add(1);
        let canonical_target = permit.action().canonical_target.clone();
        let focused = self
            .app
            .get_webview_window("main")
            .is_some_and(|window| window.set_focus().is_ok());
        NormalizedActionResult {
            status: if focused {
                NormalizedActionStatus::Succeeded
            } else {
                NormalizedActionStatus::Failed
            },
            result_code: if focused {
                "c4os-window-focused"
            } else {
                "c4os-window-focus-failed"
            }
            .into(),
            exit_code: None,
            changed_targets: focused.then_some(canonical_target).into_iter().collect(),
            output_sha256: None,
            completed_at_ms,
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn production_broker_classification(
    surface: ActionSurface,
    effects: std::collections::BTreeSet<ActionEffect>,
    canonical_target: &str,
) -> runtime::opencode_broker::InstalledBrokerClassification {
    runtime::opencode_broker::InstalledBrokerClassification {
        surface,
        effects,
        scope: ActionScope::Workspace,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: true,
        canonical_target: canonical_target.into(),
        normalized_arguments: serde_json::Map::new(),
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn install_production_broker_facilities(
    bootstrap: &runtime::production::RuntimeProductionBootstrap,
    app: &tauri::AppHandle,
) -> Result<(), runtime::production::RuntimeProductionError> {
    bootstrap.install_broker_action(
        "window.focus",
        "main",
        serde_json::Map::new(),
        production_broker_classification(
            ActionSurface::Desktop,
            std::collections::BTreeSet::from([ActionEffect::Control]),
            "desktop:window:main",
        ),
        Box::new(ProductionWindowFocusFacility { app: app.clone() }),
    )?;
    Ok(())
}

struct AppCoreState {
    database: Arc<core::database::DatabaseActor>,
    _configuration: Mutex<core::services::ManagedAppConfiguration>,
    _active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    runtime_production: Arc<ManagedProductionRuntime>,
    runtime: Arc<RuntimeApplicationService>,
    platform: PlatformService,
    platform_snapshot: PlatformSnapshot,
    picker_grants: Mutex<PickerGrantRegistry>,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
type ProductionRuntimeApplication = runtime::production_application::RuntimeProductionApplication<
    runtime::production::RuntimeProductionBootstrap,
>;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
struct ManagedPublication<T> {
    application: Mutex<Option<Arc<T>>>,
    initialization_error: Mutex<Option<String>>,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<T> Default for ManagedPublication<T> {
    fn default() -> Self {
        Self {
            application: Mutex::new(None),
            initialization_error: Mutex::new(None),
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<T> ManagedPublication<T> {
    fn load(&self, correlation_id: protocol::CorrelationId) -> Result<Arc<T>, ProtocolError> {
        self.application
            .lock()
            .ok()
            .and_then(|application| application.as_ref().cloned())
            .ok_or_else(|| runtime_production_unavailable(correlation_id))
    }

    fn publish(&self, application: Arc<T>) {
        if let Ok(mut current) = self.application.lock() {
            *current = Some(application);
        }
    }

    fn fail(&self, error: String) {
        if let Ok(mut current) = self.initialization_error.lock() {
            *current = Some(error);
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
type ManagedProductionRuntime = ManagedPublication<ProductionRuntimeApplication>;

const PLATFORM_SETTINGS_EVENT: &str = "c4os://platform/open-settings";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeSettingsEvent {
    contract_version: u16,
    command_id: &'static str,
    route: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlatformRevealSnapshot {
    revealed: bool,
    fallback: bool,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionRuntimeShutdown {
    runtime_id: String,
    coordinator_generation: u64,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionRuntimeApprovalSettlement {
    runtime_id: String,
    correlation_id: String,
    prompt_id: String,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductionRuntimePump {
    runtime_id: String,
    pumped_events: usize,
    coordinator_generation: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeCoreSnapshot {
    authority: &'static str,
    provider_generation: u64,
    capability_generation: u64,
    runtime_generation: u64,
    onboarding_ready: bool,
    providers: Vec<RuntimeProviderSummary>,
    runtimes: Vec<RuntimeProcessSummary>,
    pending_approvals: Vec<RuntimeApprovalSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeApprovalSummary {
    runtime_id: String,
    correlation_id: String,
    prompt_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeProviderSummary {
    provider_id: String,
    display_name: String,
    enabled: bool,
    test_status: runtime::provider::ProviderTestStatus,
    model_count: usize,
    selected_model_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeProcessSummary {
    runtime_id: String,
    runtime_kind: runtime::supervisor::RuntimeKind,
    native_version: String,
    lifecycle: runtime::supervisor::RuntimeLifecycle,
    health: runtime::supervisor::HealthState,
    process_generation: u64,
}

/// Rust-owned application boundary used by Tauri commands and supervised
/// workers. It keeps every runtime domain behind the coordinator's one
/// monotonic generation and never exposes the effect executor to the renderer.
pub struct RuntimeApplicationService {
    coordinator: Mutex<RuntimeCoordinator<DeferredSessionRepository>>,
    app_database: Arc<core::database::DatabaseActor>,
    failed_transaction: Mutex<Option<RuntimeCoordinatorSnapshot>>,
    capabilities: Mutex<CapabilityEvidenceRegistry>,
    policy_authority: Mutex<RuntimePolicyAuthority>,
    dispatch_authority: DispatchAuthorityRegistry,
    dispatch: Mutex<RuntimeDispatchRegistry>,
    sessions: DeferredSessionRepository,
    provider_store: ProviderStateStore,
    control_plane_store: RuntimeControlPlaneStore,
    control_plane_revision: Mutex<u64>,
}

#[derive(Clone, Copy)]
struct RuntimePolicyAuthority {
    policy_version: u64,
    revocation_epoch: u64,
}

/// Coordinator-held broker transaction used for renderer-triggered approval
/// continuation. The exact generation remains locked from the CAS check
/// through gateway mutation and effect settlement, so a stale answer cannot
/// take effect after an intervening authority change.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) struct RuntimeBrokerTransaction<'a> {
    coordinator: MutexGuard<'a, RuntimeCoordinator<DeferredSessionRepository>>,
    policy_authority: &'a Mutex<RuntimePolicyAuthority>,
    dispatch: &'a Mutex<RuntimeDispatchRegistry>,
}

/// Narrow first-submit intent accepted by the Rust application boundary. The
/// caller supplies user-authored content and CAS identities only; all durable
/// binding, capability, resource, and process snapshots are minted inside the
/// application service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstRuntimeDispatchIntent {
    pub authority: AuthorityMintIntent,
    pub provider_id: String,
    pub model_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub attempt_id: String,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    pub draft: DraftRequirements,
    pub submitted_at_ms: u64,
    pub preflight_at_ms: u64,
}

/// Narrow retry intent. The immutable session binding and current attempt
/// context are loaded and re-minted from durable/core-owned state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryRuntimeDispatchIntent {
    pub authority: AuthorityMintIntent,
    pub provider_id: String,
    pub model_id: String,
    pub session_id: String,
    pub parent_attempt_id: String,
    pub attempt_id: String,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub automatic: bool,
    pub reviewed_unknown_effect: bool,
    pub draft: DraftRequirements,
    pub created_at_ms: u64,
    pub preflight_at_ms: u64,
}

impl RuntimeApplicationService {
    pub fn restore(
        app_database: Arc<core::database::DatabaseActor>,
        now_ms: u64,
    ) -> Result<Self, RuntimeApplicationError> {
        let provider_store = ProviderStateStore::new(Arc::clone(&app_database))?;
        let providers = provider_store.load()?.unwrap_or_default();
        let control_plane_store = RuntimeControlPlaneStore::new(Arc::clone(&app_database))?;
        let restored_control_plane =
            control_plane_store.load(runtime::supervisor::pinned_compatibility())?;
        let (supervisor, capabilities, control_plane_revision) =
            if let Some(restored) = restored_control_plane {
                (
                    restored.supervisor,
                    restored.capabilities,
                    restored.revision,
                )
            } else {
                let legacy_supervisor_store = SupervisorStateStore::new(Arc::clone(&app_database))?;
                (
                    legacy_supervisor_store
                        .load(runtime::supervisor::pinned_compatibility())?
                        .unwrap_or_else(RuntimeSupervisor::pinned),
                    CapabilityEvidenceRegistry::new(),
                    0,
                )
            };
        let gateway = security::gateway::ActionGateway::restore(
            PolicyConfiguration::default(),
            Arc::clone(&app_database),
            now_ms,
        )?;
        let sessions = DeferredSessionRepository::new();
        let coordinator = RuntimeCoordinator::new(
            providers,
            supervisor,
            SessionService::new(sessions.clone()),
            gateway,
        );
        Ok(Self {
            coordinator: Mutex::new(coordinator),
            app_database,
            failed_transaction: Mutex::new(None),
            capabilities: Mutex::new(capabilities),
            policy_authority: Mutex::new(RuntimePolicyAuthority {
                policy_version: 1,
                revocation_epoch: 0,
            }),
            dispatch_authority: DispatchAuthorityRegistry::new(),
            dispatch: Mutex::new(RuntimeDispatchRegistry::new()),
            sessions,
            provider_store,
            control_plane_store,
            control_plane_revision: Mutex::new(control_plane_revision),
        })
    }

    pub fn bind_workspace(
        &self,
        workspace_database: Arc<core::database::DatabaseActor>,
    ) -> Result<(), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        self.sessions.bind_workspace(workspace_database)?;
        coordinator.note_core_authority_change()?;
        Ok(())
    }

    /// Publishes an exact Workspace repository binding and its complete
    /// runtime installation set as one durable startup epoch.
    pub fn bind_workspace_runtime_installations(
        &self,
        workspace_database: Arc<core::database::DatabaseActor>,
        installations: Vec<RuntimeInstallation>,
        checked_at_ms: u64,
    ) -> Result<
        CoordinatorOperation<std::collections::BTreeMap<String, CompatibilityState>>,
        RuntimeApplicationError,
    > {
        let mut coordinator = self.coordinator()?;
        let previous = coordinator.snapshot(checked_at_ms);
        let prepared = self
            .sessions
            .prepare_workspace_binding(workspace_database)?;
        let workspace_id = prepared.workspace_id().to_owned();
        let capabilities = self.capabilities()?;
        if capabilities.has_active_processes() {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let operation = coordinator.replace_workspace_runtime_installations(
            &workspace_id,
            installations,
            checked_at_ms,
        )?;
        let snapshot = coordinator.snapshot(checked_at_ms);
        if let Err(error) = self.save_control_plane_snapshots(
            snapshot.runtimes,
            capabilities.snapshot(),
            checked_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        prepared.commit();
        Ok(operation)
    }

    pub fn clear_runtime_installations_without_workspace(
        &self,
        checked_at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, RuntimeApplicationError> {
        if self.sessions.bound_workspace_id()?.is_some() {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let mut coordinator = self.coordinator()?;
        let previous = coordinator.snapshot(checked_at_ms);
        let capabilities = self.capabilities()?;
        if capabilities.has_active_processes() {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let operation = coordinator.clear_runtime_installations()?;
        let snapshot = coordinator.snapshot(checked_at_ms);
        if let Err(error) = self.save_control_plane_snapshots(
            snapshot.runtimes,
            capabilities.snapshot(),
            checked_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        Ok(operation)
    }

    pub fn unbind_workspace(&self, workspace_id: &str) -> Result<(), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        if !self.dispatch()?.is_empty() {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        self.sessions.unbind_workspace(workspace_id)?;
        coordinator.discard_all_provisionals()?;
        self.dispatch_authority.invalidate_workspace(workspace_id)?;
        coordinator.note_core_authority_change()?;
        Ok(())
    }

    pub fn bound_workspace_id(&self) -> Result<Option<String>, RuntimeApplicationError> {
        Ok(self.sessions.bound_workspace_id()?)
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn reserve_runtime_operation(
        &self,
        expected_coordinator_generation: u64,
        at_ms: u64,
    ) -> Result<u64, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        Ok(coordinator.note_core_authority_change()?)
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn begin_runtime_broker_transaction(
        &self,
        expected_coordinator_generation: u64,
        at_ms: u64,
    ) -> Result<RuntimeBrokerTransaction<'_>, RuntimeApplicationError> {
        let coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        Ok(RuntimeBrokerTransaction {
            coordinator,
            policy_authority: &self.policy_authority,
            dispatch: &self.dispatch,
        })
    }

    /// Derives the complete production binding from app-owned durable state.
    /// The caller supplies only a runtime identity and the already-reserved
    /// generation; filesystem roots and the launch identity never cross the
    /// renderer boundary.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn production_runtime_preparation(
        &self,
        runtime_id: &str,
        process_generation: u64,
        checked_at_ms: u64,
    ) -> Result<
        (
            runtime::production::ProductionRuntimeBinding,
            Vec<runtime::production::ProductionProviderRoute>,
        ),
        RuntimeApplicationError,
    > {
        let snapshot = self.coordinator()?.snapshot(checked_at_ms);
        let record = snapshot
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if record.process_generation != process_generation
            || record.process_id.is_some()
            || record.lifecycle != runtime::supervisor::RuntimeLifecycle::Starting
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let (workspace_id, workspace_root) = self
            .sessions
            .bound_workspace_binding()?
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if record.installation.workspace_id != workspace_id {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let c4os_home = self
            .app_database
            .descriptor()
            .path
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let binding = runtime::production::ProductionRuntimeBinding::from_core(
            runtime::production::CoreRuntimeBinding {
                runtime_id: record.installation.runtime_id.clone(),
                runtime_kind: record.installation.runtime_kind,
                workspace_id,
                workspace_root,
                c4os_home,
                process_generation,
                launch_id: format!(
                    "launch-{}-{process_generation}-{checked_at_ms}",
                    record.installation.runtime_id
                ),
            },
        )
        .map_err(RuntimeApplicationError::from)?;
        let mut provider_routes = Vec::new();
        for record in snapshot.providers.providers {
            if !record.profile.enabled {
                continue;
            }
            let Some(selected_model_id) = record.selected_model_id.as_deref() else {
                continue;
            };
            if !record.models.get(selected_model_id).is_some_and(|model| {
                model.is_production_ready_at(checked_at_ms)
                    && (binding.runtime_kind() == runtime::supervisor::RuntimeKind::Pi
                        || model.capabilities.route.runtime_kind == binding.runtime_kind().as_str())
            }) {
                continue;
            }
            provider_routes.push(runtime::production::ProductionProviderRoute::from_record(
                record,
                binding.runtime_kind(),
                checked_at_ms,
            )?);
        }
        Ok((binding, provider_routes))
    }

    pub fn snapshot(
        &self,
        now_ms: u64,
    ) -> Result<RuntimeCoordinatorSnapshot, RuntimeApplicationError> {
        let coordinator = self.raw_coordinator()?;
        if let Some(mut snapshot) = self.failed_transaction()?.clone() {
            snapshot.onboarding_ready = snapshot.providers.onboarding_ready_at(now_ms);
            return Ok(snapshot);
        }
        Ok(coordinator.snapshot(now_ms))
    }

    pub fn snapshot_with_capability_generation(
        &self,
        now_ms: u64,
    ) -> Result<(RuntimeCoordinatorSnapshot, u64), RuntimeApplicationError> {
        let coordinator = self.raw_coordinator()?;
        let mut snapshot = if let Some(snapshot) = self.failed_transaction()?.clone() {
            snapshot
        } else {
            coordinator.snapshot(now_ms)
        };
        snapshot.onboarding_ready = snapshot.providers.onboarding_ready_at(now_ms);
        let capability_generation = self.capabilities()?.generation();
        Ok((snapshot, capability_generation))
    }

    pub fn model_preflight(
        &self,
        provider_id: &str,
        model_id: &str,
        draft: &DraftRequirements,
        now_ms: u64,
    ) -> Result<ModelPreflight, RuntimeApplicationError> {
        let coordinator = self.coordinator()?;
        let route = capability_route_from_coordinator(&coordinator, provider_id, model_id, now_ms)?;
        let capabilities = self.capabilities()?;
        let layers = capabilities.layers(&route, now_ms)?;
        Ok(coordinator.model_preflight(provider_id, model_id, &layers, draft, now_ms)?)
    }

    /// Publishes a complete process-bound capability batch through one
    /// coordinator and evidence generation transition. No route is visible
    /// until all three typed layers validate for the exact native process.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_process_capabilities(
        &self,
        expected_coordinator_generation: u64,
        expected_capability_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        epochs: Vec<CapabilityRouteEpoch>,
        published_at_ms: u64,
    ) -> Result<(u64, u64), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            published_at_ms,
        )?;
        let previous = coordinator.snapshot(published_at_ms);
        let runtime = previous
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if runtime.process_generation != process_generation
            || runtime.process_id.is_none()
            || !matches!(
                runtime.lifecycle,
                runtime::supervisor::RuntimeLifecycle::Ready
                    | runtime::supervisor::RuntimeLifecycle::Degraded
            )
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        let capability_generation = candidate.replace_process_routes(
            expected_capability_generation,
            runtime_id,
            process_generation,
            epochs,
        )?;
        let coordinator_generation = coordinator.note_core_authority_change()?;
        let snapshot = coordinator.snapshot(published_at_ms);
        if let Err(error) = self.save_control_plane_snapshots(
            snapshot.runtimes,
            candidate.snapshot(),
            published_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        *capabilities = candidate;
        Ok((coordinator_generation, capability_generation))
    }

    pub fn invalidate_process_capabilities(
        &self,
        expected_coordinator_generation: u64,
        expected_capability_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        invalidated_at_ms: u64,
    ) -> Result<(u64, u64), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            invalidated_at_ms,
        )?;
        let previous = coordinator.snapshot(invalidated_at_ms);
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        let capability_generation = candidate.invalidate_process(
            expected_capability_generation,
            runtime_id,
            process_generation,
        )?;
        let coordinator_generation = coordinator.note_core_authority_change()?;
        let snapshot = coordinator.snapshot(invalidated_at_ms);
        if let Err(error) = self.save_control_plane_snapshots(
            snapshot.runtimes,
            candidate.snapshot(),
            invalidated_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        *capabilities = candidate;
        Ok((coordinator_generation, capability_generation))
    }

    pub fn capability_evidence_generation(&self) -> Result<u64, RuntimeApplicationError> {
        Ok(self.capabilities()?.generation())
    }

    /// Publishes policy/revocation authority atomically with the coordinator
    /// generation. Policy workers must advance this state whenever policy is
    /// tightened so retained native workers can never reuse an old authority.
    pub fn publish_runtime_policy_authority(
        &self,
        expected_coordinator_generation: u64,
        expected_policy_version: u64,
        policy_version: u64,
        revocation_epoch: u64,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            updated_at_ms,
        )?;
        let mut authority = self
            .policy_authority
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        if authority.policy_version != expected_policy_version
            || policy_version <= authority.policy_version
            || revocation_epoch < authority.revocation_epoch
        {
            return Err(RuntimeApplicationError::InvalidPolicyAuthority);
        }
        coordinator.note_core_authority_change()?;
        *authority = RuntimePolicyAuthority {
            policy_version,
            revocation_epoch,
        };
        Ok(coordinator.snapshot(updated_at_ms).generation)
    }

    #[cfg(test)]
    fn publish_capability_generation_with_barrier(
        &self,
        candidate_ready: std::sync::mpsc::SyncSender<()>,
        continue_publication: std::sync::mpsc::Receiver<()>,
    ) -> Result<(), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        candidate.advance_generation_for_atomic_publication_test();
        let _ = candidate_ready.send(());
        let _ = continue_publication.recv();
        coordinator.note_core_authority_change()?;
        *capabilities = candidate;
        Ok(())
    }

    /// Installs exact configuration/resource authority from a Rust worker.
    /// The CAS mutation also advances the one application-wide generation so
    /// any renderer intent observed before the change becomes stale.
    pub fn install_dispatch_authority(
        &self,
        expected_coordinator_generation: u64,
        expected_authority_generation: u64,
        authority: WorkerDispatchAuthority,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            updated_at_ms,
        )?;
        let generation = self
            .dispatch_authority
            .install_from_worker(expected_authority_generation, authority)?;
        coordinator.note_core_authority_change()?;
        Ok(generation)
    }

    pub fn dispatch_authority_generation(&self) -> Result<u64, RuntimeApplicationError> {
        Ok(self.dispatch_authority.generation()?)
    }

    /// Rust-only peer installation boundary. Registrations are exact-bound to
    /// one workspace, runtime kind/version, and process generation before any
    /// dispatch may reach them.
    pub fn register_dispatch_peer(
        &self,
        peer: impl RuntimeDispatchPeer + 'static,
    ) -> Result<(), RuntimeApplicationError> {
        Ok(self.dispatch()?.register(peer)?)
    }

    pub fn save_provider(
        &self,
        expected_coordinator_generation: u64,
        profile: ProviderProfile,
        expected_provider_generation: u64,
        updated_at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            updated_at_ms,
        )?;
        self.require_persisted_generation(
            "provider-snapshot",
            "providers",
            persistence_expectation(expected_provider_generation),
        )?;
        let previous = coordinator.snapshot(updated_at_ms);
        let operation = coordinator.save_provider(profile, expected_provider_generation)?;
        let snapshot = coordinator.snapshot(updated_at_ms);
        if let Err(error) = self.provider_store.save_snapshot(
            &snapshot.providers,
            persistence_expectation(expected_provider_generation),
            updated_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error.into());
        }
        Ok(operation)
    }

    pub fn test_provider(
        &self,
        expected_coordinator_generation: u64,
        provider_id: &str,
        expected_provider_generation: u64,
        attempted_at_ms: u64,
        probe: &mut impl ProviderProbe,
    ) -> Result<CoordinatorOperation<ProviderTestReport>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            attempted_at_ms,
        )?;
        self.require_persisted_generation(
            "provider-snapshot",
            "providers",
            persistence_expectation(expected_provider_generation),
        )?;
        let previous = coordinator.snapshot(attempted_at_ms);
        let operation = coordinator.test_provider(
            provider_id,
            expected_provider_generation,
            attempted_at_ms,
            probe,
        )?;
        let snapshot = coordinator.snapshot(attempted_at_ms);
        if let Err(error) = self.provider_store.save_snapshot(
            &snapshot.providers,
            persistence_expectation(expected_provider_generation),
            attempted_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error.into());
        }
        Ok(operation)
    }

    pub fn select_model(
        &self,
        expected_coordinator_generation: u64,
        provider_id: &str,
        model_id: &str,
        expected_provider_generation: u64,
        updated_at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            updated_at_ms,
        )?;
        self.require_persisted_generation(
            "provider-snapshot",
            "providers",
            persistence_expectation(expected_provider_generation),
        )?;
        let previous = coordinator.snapshot(updated_at_ms);
        let operation =
            coordinator.select_model(provider_id, model_id, expected_provider_generation)?;
        let snapshot = coordinator.snapshot(updated_at_ms);
        if let Err(error) = self.provider_store.save_snapshot(
            &snapshot.providers,
            persistence_expectation(expected_provider_generation),
            updated_at_ms,
        ) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error.into());
        }
        Ok(operation)
    }

    pub fn register_runtime(
        &self,
        expected_coordinator_generation: u64,
        installation: RuntimeInstallation,
        checked_at_ms: u64,
    ) -> Result<CoordinatorOperation<CompatibilityState>, RuntimeApplicationError> {
        if self
            .sessions
            .bound_workspace_id()?
            .is_some_and(|workspace_id| workspace_id != installation.workspace_id)
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            checked_at_ms,
        )?;
        let previous = coordinator.snapshot(checked_at_ms);
        let operation = coordinator.register_runtime(installation, checked_at_ms)?;
        let snapshot = coordinator.snapshot(checked_at_ms);
        if let Err(error) = self.save_current_control_plane(snapshot.runtimes, checked_at_ms) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        Ok(operation)
    }

    pub fn start_runtime(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let operation = coordinator.start_runtime(runtime_id, at_ms)?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) = self.save_current_control_plane(snapshot.runtimes.clone(), at_ms) {
            if let Some(process_id) = newly_started_process_id(&previous, &snapshot, runtime_id) {
                terminate_uncommitted_process_group(process_id);
                reap_uncommitted_runtime_process(
                    &mut coordinator,
                    runtime_id,
                    operation.value,
                    at_ms,
                );
            }
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        Ok(operation)
    }

    /// Durably reserves the next generation for a production adapter that
    /// owns its native child handle. No process is spawned and no ready state
    /// is published by this transition.
    pub fn reserve_managed_runtime_start(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<u64>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let operation = coordinator.reserve_managed_runtime_start(runtime_id, at_ms)?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) = self.save_current_control_plane(snapshot.runtimes, at_ms) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        Ok(operation)
    }

    /// Publishes a production-owned process only after the exact adapter has
    /// completed its native health check. The process identity and ready state
    /// become externally visible in the same durable supervisor transition.
    #[allow(clippy::too_many_arguments)]
    pub fn attach_managed_runtime_process(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        process_id: u32,
        health: HealthState,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, RuntimeApplicationError> {
        let expected_capability_generation = self.capability_evidence_generation()?;
        self.attach_managed_runtime_process_with_capabilities(
            expected_coordinator_generation,
            expected_capability_generation,
            runtime_id,
            process_generation,
            process_id,
            health,
            Vec::new(),
            at_ms,
        )
        .map(|(operation, _, _)| operation)
    }

    /// Atomically publishes native readiness and the complete typed route
    /// epoch for that exact process. An empty route vector is still an
    /// explicit healthy zero-provider epoch and therefore participates in
    /// shutdown invalidation and generation CAS.
    #[allow(clippy::too_many_arguments)]
    pub fn attach_managed_runtime_process_with_capabilities(
        &self,
        expected_coordinator_generation: u64,
        expected_capability_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        process_id: u32,
        health: HealthState,
        epochs: Vec<CapabilityRouteEpoch>,
        at_ms: u64,
    ) -> Result<(CoordinatorOperation<()>, u64, u64), RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let workspace_id = previous
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .map(|record| record.installation.workspace_id.clone())
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        let authorities = if epochs.is_empty() {
            Vec::new()
        } else {
            self.production_dispatch_authorities(runtime_id, process_generation, &epochs)?
        };
        let capability_generation = candidate.replace_process_routes(
            expected_capability_generation,
            runtime_id,
            process_generation,
            epochs,
        )?;
        let operation = coordinator.attach_managed_runtime_process(
            runtime_id,
            process_generation,
            process_id,
            health,
            at_ms,
        )?;
        let expected_authority_generation = self.dispatch_authority.generation()?;
        let authority_generation = self.dispatch_authority.replace_runtime_from_worker(
            expected_authority_generation,
            &workspace_id,
            runtime_id,
            process_generation,
            authorities,
        )?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) =
            self.save_control_plane_snapshots(snapshot.runtimes, candidate.snapshot(), at_ms)
        {
            let _ = self.dispatch_authority.invalidate_runtime_process(
                authority_generation,
                &workspace_id,
                runtime_id,
                process_generation,
            );
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        *capabilities = candidate;
        Ok((operation, capability_generation, authority_generation))
    }

    /// Compensates a failed production preparation before any native process
    /// was attached to the authoritative supervisor record.
    pub fn abort_managed_runtime_start(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let operation =
            coordinator.abort_managed_runtime_start(runtime_id, process_generation, at_ms)?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) = self.save_current_control_plane(snapshot.runtimes, at_ms) {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        Ok(operation)
    }

    /// Records protocol-aware shutdown only after the production peer has
    /// terminated its exact native process group.
    pub fn finish_managed_runtime_shutdown(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let workspace_id = previous
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .map(|record| record.installation.workspace_id.clone())
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        let capability_generation = candidate.process_generation(runtime_id);
        if let Some(current) = capability_generation {
            if current != process_generation {
                return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
            }
            candidate.invalidate_process(candidate.generation(), runtime_id, process_generation)?;
        }
        let operation =
            coordinator.finish_managed_runtime_shutdown(runtime_id, process_generation, at_ms)?;
        let authority_generation = self.dispatch_authority.generation()?;
        self.dispatch_authority.invalidate_runtime_process(
            authority_generation,
            &workspace_id,
            runtime_id,
            process_generation,
        )?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) =
            self.save_control_plane_snapshots(snapshot.runtimes, candidate.snapshot(), at_ms)
        {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        *capabilities = candidate;
        Ok(operation)
    }

    pub fn record_runtime_health(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        process_generation: u64,
        health: HealthState,
        at_ms: u64,
    ) -> Result<CoordinatorOperation<()>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, at_ms)?;
        let previous = coordinator.snapshot(at_ms);
        let mut capabilities = self.capabilities()?;
        let mut candidate = capabilities.clone();
        if health != HealthState::Healthy
            && candidate.process_generation(runtime_id) == Some(process_generation)
        {
            candidate.invalidate_process(candidate.generation(), runtime_id, process_generation)?;
        }
        let operation =
            coordinator.record_runtime_health(runtime_id, process_generation, health, at_ms)?;
        let snapshot = coordinator.snapshot(at_ms);
        if let Err(error) =
            self.save_control_plane_snapshots(snapshot.runtimes, candidate.snapshot(), at_ms)
        {
            self.quarantine_failed_transaction(previous)?;
            return Err(error);
        }
        *capabilities = candidate;
        Ok(operation)
    }

    pub fn create_provisional(
        &self,
        expected_coordinator_generation: u64,
        session_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Result<CoordinatorOperation<SessionRecord>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            created_at_ms,
        )?;
        Ok(coordinator.create_provisional(session_id, created_at_ms)?)
    }

    /// Worker-only first dispatch. Capability evidence is resolved from the
    /// application-owned registry and exact native readiness is checked before
    /// the provisional Chat can become durable.
    pub fn dispatch_first(
        &self,
        expected_coordinator_generation: u64,
        intent: FirstRuntimeDispatchIntent,
        mut options: FirstDispatchOptions,
    ) -> Result<CoordinatedFirstDispatch, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            intent.preflight_at_ms,
        )?;
        self.require_bound_active_project(
            &intent.authority.workspace_id,
            intent.authority.project_id.as_deref(),
        )?;
        // Capability evidence is held through the coordinator mutation so a
        // producer cannot commit new evidence between minting and dispatch.
        let capabilities = self.capabilities()?;
        let route = capability_route_for_runtime(
            &coordinator,
            &capabilities,
            &intent.authority.runtime_id,
            &intent.provider_id,
            &intent.model_id,
            intent.preflight_at_ms,
        )?;
        let capability_generation = capabilities.generation();
        let capability_layers = capabilities.layers(&route, intent.preflight_at_ms)?;
        let preflight = coordinator.model_preflight(
            &intent.provider_id,
            &intent.model_id,
            &capability_layers,
            &intent.draft,
            intent.preflight_at_ms,
        )?;
        // Credential authority is derived from the selected ProviderProfile
        // by the registered production peer. Worker/caller options cannot
        // substitute either the opaque reference or an operation lease.
        options.credential_reference = None;
        options.credential_lease_id = None;
        options.attachment_resolution = self.production_attachment_resolution(
            &intent.authority.workspace_id,
            &intent.attachments,
        )?;
        let mut dispatch = self.dispatch()?;
        let process = runtime_process_truth(&dispatch.registration(&intent.authority.runtime_id)?);
        let minted = self
            .dispatch_authority
            .mint_first_binding_for_active_project(
                &intent.authority,
                &preflight.effective_capabilities,
                capability_generation,
                &process,
                intent.submitted_at_ms,
            )?;
        options.broker_authority = self.production_broker_authority(
            minted.binding.runtime_kind,
            minted.binding.initial_configuration.version,
            intent.submitted_at_ms,
        )?;
        let request = CoordinatedFirstSubmission {
            submission: FirstSubmission {
                session_id: intent.session_id,
                turn_id: intent.turn_id,
                attempt_id: intent.attempt_id,
                authorization_scope_id: intent.authorization_scope_id,
                correlation_id: intent.correlation_id,
                process_generation: minted.process_generation,
                prompt: intent.prompt,
                attachments: intent.attachments,
                binding: minted.binding,
                submitted_at_ms: intent.submitted_at_ms,
            },
            provider_id: intent.provider_id,
            selected_model_id: intent.model_id,
            capability_layers,
            draft: intent.draft,
            preflight_at_ms: intent.preflight_at_ms,
        };
        Ok(coordinate_first_dispatch(
            &mut coordinator,
            &mut dispatch,
            request,
            options,
        )?)
    }

    /// Drains only events normalized by the registered Rust-owned peer and
    /// applies them against the exact active attempt identity.
    pub fn poll_runtime_events(
        &self,
        expected_coordinator_generation: u64,
        runtime_id: &str,
        recorded_at_ms: u64,
    ) -> Result<Vec<AppliedDispatchEvent>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            recorded_at_ms,
        )?;
        let mut dispatch = self.dispatch()?;
        Ok(coordinate_polled_events(
            &mut coordinator,
            &mut dispatch,
            runtime_id,
            recorded_at_ms,
        )?)
    }

    pub fn cancel_dispatch(
        &self,
        expected_coordinator_generation: u64,
        identity: &DispatchIdentity,
        requested_at_ms: u64,
    ) -> Result<CoordinatedCancellation, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            requested_at_ms,
        )?;
        let mut dispatch = self.dispatch()?;
        Ok(coordinate_cancellation(
            &mut coordinator,
            &mut dispatch,
            identity,
            requested_at_ms,
        )?)
    }

    pub fn retry_dispatch(
        &self,
        expected_coordinator_generation: u64,
        intent: RetryRuntimeDispatchIntent,
        mut options: RetryDispatchOptions,
    ) -> Result<CoordinatedRetryDispatch, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            intent.preflight_at_ms,
        )?;
        self.require_bound_active_project(
            &intent.authority.workspace_id,
            intent.authority.project_id.as_deref(),
        )?;
        let capabilities = self.capabilities()?;
        let route = capability_route_for_runtime(
            &coordinator,
            &capabilities,
            &intent.authority.runtime_id,
            &intent.provider_id,
            &intent.model_id,
            intent.preflight_at_ms,
        )?;
        let capability_generation = capabilities.generation();
        let capability_layers = capabilities.layers(&route, intent.preflight_at_ms)?;
        let session = coordinator.session(&intent.session_id)?;
        let binding = session
            .binding()
            .cloned()
            .ok_or(DispatchError::InvalidRequest)?;
        let retry_attachments = session
            .attempt(&intent.parent_attempt_id)
            .and_then(|attempt| session.turn(&attempt.turn_id))
            .map(|turn| turn.attachments.clone())
            .ok_or(DispatchError::InvalidRequest)?;
        let preflight = coordinator.model_preflight(
            &intent.provider_id,
            &intent.model_id,
            &capability_layers,
            &intent.draft,
            intent.preflight_at_ms,
        )?;
        options.credential_reference = None;
        options.credential_lease_id = None;
        options.attachment_resolution = self
            .production_attachment_resolution(&intent.authority.workspace_id, &retry_attachments)?;
        let mut dispatch = self.dispatch()?;
        let process = runtime_process_truth(&dispatch.registration(&intent.authority.runtime_id)?);
        let minted = self
            .dispatch_authority
            .mint_retry_context_for_active_project(
                &intent.authority,
                &binding,
                &preflight.effective_capabilities,
                capability_generation,
                &process,
            )?;
        options.broker_authority = self.production_broker_authority(
            minted.context.runtime_kind,
            minted.context.configuration.version,
            intent.created_at_ms,
        )?;
        let request = CoordinatedRetry {
            request: RetryRequest {
                session_id: intent.session_id,
                parent_attempt_id: intent.parent_attempt_id,
                attempt_id: intent.attempt_id,
                authorization_scope_id: intent.authorization_scope_id,
                correlation_id: intent.correlation_id,
                process_generation: minted.process_generation,
                context: minted.context,
                automatic: intent.automatic,
                reviewed_unknown_effect: intent.reviewed_unknown_effect,
                created_at_ms: intent.created_at_ms,
            },
            provider_id: intent.provider_id,
            selected_model_id: intent.model_id,
            capability_layers,
            draft: intent.draft,
            preflight_at_ms: intent.preflight_at_ms,
        };
        Ok(coordinate_retry_dispatch(
            &mut coordinator,
            &mut dispatch,
            request,
            options,
        )?)
    }

    pub fn recover_interrupted(
        &self,
        expected_coordinator_generation: u64,
        recovered_at_ms: u64,
    ) -> Result<CoordinatorOperation<Vec<SessionRecord>>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            recovered_at_ms,
        )?;
        let mut dispatch = self.dispatch()?;
        Ok(coordinate_recovery(
            &mut coordinator,
            &mut dispatch,
            recovered_at_ms,
        )?)
    }

    fn coordinator(
        &self,
    ) -> Result<
        MutexGuard<'_, RuntimeCoordinator<DeferredSessionRepository>>,
        RuntimeApplicationError,
    > {
        let coordinator = self.raw_coordinator()?;
        if self.failed_transaction()?.is_some() {
            return Err(RuntimeApplicationError::Unavailable);
        }
        Ok(coordinator)
    }

    fn raw_coordinator(
        &self,
    ) -> Result<
        MutexGuard<'_, RuntimeCoordinator<DeferredSessionRepository>>,
        RuntimeApplicationError,
    > {
        self.coordinator
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)
    }

    fn failed_transaction(
        &self,
    ) -> Result<MutexGuard<'_, Option<RuntimeCoordinatorSnapshot>>, RuntimeApplicationError> {
        self.failed_transaction
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)
    }

    fn require_persisted_generation(
        &self,
        document_kind: &str,
        document_id: &str,
        expected_generation: Option<u64>,
    ) -> Result<(), RuntimeApplicationError> {
        let actual_generation = self
            .app_database
            .runtime_state_document(document_kind, document_id)
            .map_err(RuntimePersistenceError::Database)?
            .map(|document| document.generation);
        if actual_generation == expected_generation {
            return Ok(());
        }
        Err(
            RuntimePersistenceError::Database(core::database::DatabaseError::Conflict(format!(
                "runtime document {document_kind}/{document_id} expected generation \
             {expected_generation:?}, found {actual_generation:?}"
            )))
            .into(),
        )
    }

    fn quarantine_failed_transaction(
        &self,
        previous: RuntimeCoordinatorSnapshot,
    ) -> Result<(), RuntimeApplicationError> {
        let mut failed = self.failed_transaction()?;
        if failed.is_none() {
            *failed = Some(previous);
        }
        Ok(())
    }

    fn save_control_plane_snapshots(
        &self,
        supervisor: runtime::supervisor::SupervisorSnapshot,
        capabilities: runtime::capability_evidence::CapabilityEvidenceSnapshot,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimeApplicationError> {
        let mut revision = self
            .control_plane_revision
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        let committed = self.control_plane_store.save_snapshots(
            supervisor,
            capabilities,
            *revision,
            updated_at_ms,
        )?;
        *revision = committed;
        Ok(committed)
    }

    fn save_current_control_plane(
        &self,
        supervisor: runtime::supervisor::SupervisorSnapshot,
        updated_at_ms: u64,
    ) -> Result<u64, RuntimeApplicationError> {
        let capabilities = self.capabilities()?.snapshot();
        self.save_control_plane_snapshots(supervisor, capabilities, updated_at_ms)
    }

    fn production_dispatch_authorities(
        &self,
        runtime_id: &str,
        process_generation: u64,
        epochs: &[CapabilityRouteEpoch],
    ) -> Result<Vec<WorkerDispatchAuthority>, RuntimeApplicationError> {
        let registration = self.dispatch()?.registration(runtime_id)?.clone();
        if registration.descriptor.process_generation != process_generation {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let runtime_kind = match registration.descriptor.runtime_kind {
            runtime::supervisor::RuntimeKind::OpenCode => SessionRuntimeKind::OpenCode,
            runtime::supervisor::RuntimeKind::Pi => SessionRuntimeKind::Pi,
        };
        let resources = production_broker_authoritative_resources();
        epochs
            .iter()
            .map(|epoch| {
                if epoch.runtime_id() != runtime_id
                    || epoch.process_generation() != process_generation
                {
                    return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
                }
                Ok(WorkerDispatchAuthority {
                    workspace_id: registration.workspace_id.clone(),
                    project_id: None,
                    runtime_id: runtime_id.into(),
                    runtime_kind,
                    adapter: runtime::session::AdapterBinding {
                        adapter_id: registration.descriptor.runtime_kind.as_str().into(),
                        adapter_version: registration.descriptor.adapter_version.clone(),
                        native_version: registration.descriptor.native_version.clone(),
                    },
                    environment: runtime::session::ExecutionEnvironmentBinding {
                        environment_id: "local".into(),
                        environment_kind: "local".into(),
                        host_alias: None,
                    },
                    route: epoch.route().clone(),
                    configuration: authoritative_route_configuration(epoch.route())?,
                    resources: resources.clone(),
                    process_generation,
                })
            })
            .collect()
    }

    fn capabilities(
        &self,
    ) -> Result<MutexGuard<'_, CapabilityEvidenceRegistry>, RuntimeApplicationError> {
        self.capabilities
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)
    }

    pub(crate) fn dispatch(
        &self,
    ) -> Result<MutexGuard<'_, RuntimeDispatchRegistry>, RuntimeApplicationError> {
        self.dispatch
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)
    }

    fn production_attachment_resolution(
        &self,
        workspace_id: &str,
        attachments: &[AttachmentSnapshot],
    ) -> Result<AttachmentPreflightResolution, RuntimeApplicationError> {
        if attachments.is_empty() {
            return Ok(AttachmentPreflightResolution::NotRequired);
        }
        let (bound_workspace_id, workspace_root) = self
            .sessions
            .bound_workspace_binding()?
            .ok_or(RuntimeApplicationError::InvalidAttachmentBinding)?;
        if bound_workspace_id != workspace_id {
            return Err(RuntimeApplicationError::InvalidAttachmentBinding);
        }
        let materializer = runtime::attachment_materializer::WorkspaceAttachmentMaterializer::bind(
            bound_workspace_id,
            workspace_root,
        )?;
        Ok(AttachmentPreflightResolution::Direct(
            materializer.materialize(attachments)?,
        ))
    }

    fn require_bound_workspace(&self, workspace_id: &str) -> Result<(), RuntimeApplicationError> {
        if self.sessions.bound_workspace_id()?.as_deref() == Some(workspace_id) {
            Ok(())
        } else {
            Err(RuntimeApplicationError::InvalidManagedRuntimeBinding)
        }
    }

    fn require_bound_active_project(
        &self,
        workspace_id: &str,
        project_id: Option<&str>,
    ) -> Result<(), RuntimeApplicationError> {
        self.require_bound_workspace(workspace_id)?;
        let project_id = project_id.ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if self.sessions.bound_active_project_exists(project_id)? {
            Ok(())
        } else {
            Err(RuntimeApplicationError::InvalidManagedRuntimeBinding)
        }
    }

    fn production_broker_authority(
        &self,
        runtime_kind: SessionRuntimeKind,
        configuration_version: u64,
        active_from_ms: u64,
    ) -> Result<Option<BrokerDispatchAuthority>, RuntimeApplicationError> {
        if runtime_kind != SessionRuntimeKind::OpenCode {
            return Ok(None);
        }
        let authority = self
            .policy_authority
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        Ok(Some(BrokerDispatchAuthority {
            request_origin: ActionRequestOrigin::RuntimeTool,
            configuration_version,
            policy_version: authority.policy_version,
            revocation_epoch: authority.revocation_epoch,
            active_from_ms,
            expires_at_ms: active_from_ms.saturating_add(BROKER_CONTEXT_TTL_MS),
        }))
    }
}

fn capability_route_from_coordinator(
    coordinator: &RuntimeCoordinator<DeferredSessionRepository>,
    provider_id: &str,
    model_id: &str,
    now_ms: u64,
) -> Result<runtime::capability::RouteIdentity, RuntimeApplicationError> {
    let snapshot = coordinator.snapshot(now_ms).providers;
    let provider = snapshot
        .providers
        .iter()
        .find(|record| record.profile.provider_id == provider_id)
        .ok_or(CoordinatorError::ModelRouteUnavailable)?;
    if provider.selected_model_id.as_deref() != Some(model_id) {
        return Err(CoordinatorError::ModelRouteUnavailable.into());
    }
    Ok(provider
        .models
        .get(model_id)
        .ok_or(CoordinatorError::ModelRouteUnavailable)?
        .capabilities
        .route
        .clone())
}

fn capability_route_for_runtime(
    coordinator: &RuntimeCoordinator<DeferredSessionRepository>,
    capabilities: &CapabilityEvidenceRegistry,
    runtime_id: &str,
    provider_id: &str,
    model_id: &str,
    now_ms: u64,
) -> Result<runtime::capability::RouteIdentity, RuntimeApplicationError> {
    let snapshot = coordinator.snapshot(now_ms).providers;
    let provider = snapshot
        .providers
        .iter()
        .find(|record| record.profile.provider_id == provider_id)
        .ok_or(CoordinatorError::ModelRouteUnavailable)?;
    if provider.selected_model_id.as_deref() != Some(model_id)
        || !provider.models.contains_key(model_id)
    {
        return Err(CoordinatorError::ModelRouteUnavailable.into());
    }
    Ok(capabilities.route_for_runtime_model(
        runtime_id,
        provider_id,
        &provider.profile.endpoint.endpoint_id,
        model_id,
        now_ms,
    )?)
}

fn newly_started_process_id(
    previous: &RuntimeCoordinatorSnapshot,
    replacement: &RuntimeCoordinatorSnapshot,
    runtime_id: &str,
) -> Option<u32> {
    let previous_process_id = previous
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == runtime_id)
        .and_then(|record| record.process_id);
    replacement
        .runtimes
        .records
        .iter()
        .find(|record| record.installation.runtime_id == runtime_id)
        .and_then(|record| record.process_id)
        .filter(|process_id| Some(*process_id) != previous_process_id)
}

/// A worker that starts before an unexpected database failure never becomes
/// authoritative. Kill its complete process group immediately; the poisoned
/// coordinator retains the `Child` only so its normal drop path can reap it.
#[cfg(unix)]
fn terminate_uncommitted_process_group(process_id: u32) {
    let target = format!("-{process_id}");
    let _ = Command::new("/bin/kill")
        .args(["-TERM", target.as_str()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(20));
    let _ = Command::new("/bin/kill")
        .args(["-KILL", target.as_str()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn terminate_uncommitted_process_group(_process_id: u32) {}

fn reap_uncommitted_runtime_process(
    coordinator: &mut RuntimeCoordinator<DeferredSessionRepository>,
    runtime_id: &str,
    process_generation: u64,
    at_ms: u64,
) {
    for offset in 1..=10 {
        thread::sleep(Duration::from_millis(5));
        let checked_at_ms = at_ms.saturating_add(offset);
        let _ = coordinator.record_runtime_health(
            runtime_id,
            process_generation,
            HealthState::Unhealthy,
            checked_at_ms,
        );
        let process_is_retained = coordinator
            .snapshot(checked_at_ms)
            .runtimes
            .records
            .iter()
            .find(|record| record.installation.runtime_id == runtime_id)
            .is_some_and(|record| record.process_id.is_some());
        if !process_is_retained {
            break;
        }
    }
}

fn runtime_process_truth(
    registration: &runtime::dispatch::RuntimePeerRegistration,
) -> RuntimeProcessTruth {
    RuntimeProcessTruth {
        runtime_id: registration.runtime_id.clone(),
        runtime_kind: match registration.descriptor.runtime_kind {
            runtime::supervisor::RuntimeKind::OpenCode => SessionRuntimeKind::OpenCode,
            runtime::supervisor::RuntimeKind::Pi => SessionRuntimeKind::Pi,
        },
        adapter_version: registration.descriptor.adapter_version.clone(),
        native_version: registration.descriptor.native_version.clone(),
        process_generation: registration.descriptor.process_generation,
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl RuntimeBrokerTransaction<'_> {
    pub(crate) fn dispatch(
        &self,
    ) -> Result<MutexGuard<'_, RuntimeDispatchRegistry>, RuntimeApplicationError> {
        self.dispatch
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)
    }

    pub(crate) fn pi_gateway_authority_for_identity(
        &self,
        identity: &DispatchIdentity,
    ) -> Result<runtime::production::PiGatewayAuthorityContext, RuntimeApplicationError> {
        let record = self.coordinator.session(&identity.session_id)?;
        let attempt = record
            .attempt(&identity.attempt_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if attempt.process_generation != identity.process_generation
            || attempt.context.runtime_id != identity.runtime_id
            || attempt.context.workspace_id != identity.workspace_id
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let authority = self
            .policy_authority
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        let eligible_tool_ids = attempt
            .context
            .resources
            .resource_ids
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        if eligible_tool_ids.len() != attempt.context.resources.resource_ids.len()
            || eligible_tool_ids
                .iter()
                .any(|tool| !OPENCODE_C4OS_TOOL_IDS.contains(&tool.as_str()))
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        Ok(runtime::production::PiGatewayAuthorityContext {
            request_origin: ActionRequestOrigin::RuntimeTool,
            configuration_version: attempt.context.configuration.version,
            policy_version: authority.policy_version,
            revocation_epoch: authority.revocation_epoch,
            eligible_tool_ids,
        })
    }

    pub(crate) fn record_pi_fallback_cancellation(
        &mut self,
        identity: &DispatchIdentity,
        cancelled_at_ms: u64,
    ) -> Result<(), RuntimeApplicationError> {
        self.coordinator
            .cancel_runtime_actions(&identity.attempt_id, cancelled_at_ms)?;
        self.coordinator.request_cancellation(
            &identity.session_id,
            &identity.attempt_identity(),
            cancelled_at_ms,
        )?;
        self.coordinator.finish_attempt(
            &identity.session_id,
            &identity.attempt_identity(),
            TerminalAttemptOutcome::Cancelled,
            cancelled_at_ms.saturating_add(1),
        )?;
        Ok(())
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl BrokerActionApplication for RuntimeBrokerTransaction<'_> {
    type Error = RuntimeApplicationError;

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error> {
        Ok(self
            .coordinator
            .propose_runtime_action(proposal, now_ms)?
            .value)
    }

    fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, Self::Error> {
        Ok(self
            .coordinator
            .answer_runtime_approval(prompt_id, answer, now_ms)?
            .value)
    }

    fn execute_runtime_action<F>(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: F,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>
    where
        F: FnOnce(ExecutionPermit) -> NormalizedActionResult,
    {
        Ok(self
            .coordinator
            .execute_runtime_action(authorization, live, now_ms, effect)?
            .value)
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.coordinator.cancel_runtime_actions(run_id, now_ms)?;
        Ok(())
    }
}

/// The authenticated broker worker talks to the same coordinator instance as
/// provider, session, and runtime supervision. It receives decisions and
/// sealed authorizations directly; none are serialized through Tauri or the
/// renderer boundary.
impl BrokerActionApplication for RuntimeApplicationService {
    type Error = RuntimeApplicationError;

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error> {
        Ok(self
            .coordinator()?
            .propose_runtime_action(proposal, now_ms)?
            .value)
    }

    fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, Self::Error> {
        Ok(self
            .coordinator()?
            .answer_runtime_approval(prompt_id, answer, now_ms)?
            .value)
    }

    fn execute_runtime_action<F>(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: F,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>
    where
        F: FnOnce(ExecutionPermit) -> NormalizedActionResult,
    {
        Ok(self
            .coordinator()?
            .execute_runtime_action(authorization, live, now_ms, effect)?
            .value)
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.coordinator()?.cancel_runtime_actions(run_id, now_ms)?;
        Ok(())
    }
}

/// Runtime worker threads keep only a shared handle to the Rust application
/// service. All mutation remains serialized by the service-owned coordinator;
/// the Arc itself carries no authority and never crosses IPC.
impl BrokerActionApplication for Arc<RuntimeApplicationService> {
    type Error = RuntimeApplicationError;

    fn propose_runtime_action(
        &mut self,
        proposal: RuntimeActionProposal,
        now_ms: u64,
    ) -> Result<RuntimeGatewayDecision, Self::Error> {
        Ok(self
            .coordinator()?
            .propose_runtime_action(proposal, now_ms)?
            .value)
    }

    fn answer_runtime_approval(
        &mut self,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<RuntimeApprovalDecision, Self::Error> {
        Ok(self
            .coordinator()?
            .answer_runtime_approval(prompt_id, answer, now_ms)?
            .value)
    }

    fn execute_runtime_action<F>(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
        effect: F,
    ) -> Result<RuntimeExecutionReceipt, Self::Error>
    where
        F: FnOnce(ExecutionPermit) -> NormalizedActionResult,
    {
        Ok(self
            .coordinator()?
            .execute_runtime_action(authorization, live, now_ms, effect)?
            .value)
    }

    fn cancel_runtime_run(&mut self, run_id: &str, now_ms: u64) -> Result<(), Self::Error> {
        self.coordinator()?.cancel_runtime_actions(run_id, now_ms)?;
        Ok(())
    }
}

fn persistence_expectation(generation: u64) -> Option<u64> {
    (generation != 0).then_some(generation)
}

fn require_coordinator_generation(
    coordinator: &RuntimeCoordinator<DeferredSessionRepository>,
    expected: u64,
    now_ms: u64,
) -> Result<(), RuntimeApplicationError> {
    let current = coordinator.snapshot(now_ms).generation;
    if expected == current {
        Ok(())
    } else {
        Err(RuntimeApplicationError::Generation { expected, current })
    }
}

#[derive(Debug, Error)]
pub enum RuntimeApplicationError {
    #[error("runtime application state is unavailable")]
    Unavailable,
    #[error("runtime command expected coordinator generation {expected}, found {current}")]
    Generation { expected: u64, current: u64 },
    #[error("runtime production binding does not match durable application state")]
    InvalidManagedRuntimeBinding,
    #[error("runtime policy authority transition is invalid")]
    InvalidPolicyAuthority,
    #[error("runtime attachment binding does not match the active Workspace")]
    InvalidAttachmentBinding,
    #[error(transparent)]
    AttachmentMaterialization(
        #[from] runtime::attachment_materializer::AttachmentMaterializationError,
    ),
    #[error(transparent)]
    Coordinator(#[from] CoordinatorError),
    #[error(transparent)]
    RuntimeBridge(#[from] runtime::action_bridge::RuntimeBridgeError),
    #[error(transparent)]
    CapabilityEvidence(#[from] CapabilityEvidenceError),
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
    #[error(transparent)]
    DispatchAuthority(#[from] DispatchAuthorityError),
    #[error(transparent)]
    Persistence(#[from] RuntimePersistenceError),
    #[error(transparent)]
    Gateway(#[from] security::gateway::ActionGatewayError),
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[error(transparent)]
    Production(#[from] runtime::production::RuntimeProductionError),
}

struct CredentialServiceState {
    _vault: Option<security::credentials::CredentialVault>,
    _requires_explicit_fallback: bool,
}

impl CredentialServiceState {
    fn vault(&self) -> Option<security::credentials::CredentialVault> {
        self._vault.clone()
    }

    #[cfg(target_os = "macos")]
    fn initialize(
        c4os_home: &std::path::Path,
    ) -> Result<Self, security::credentials::CredentialVaultError> {
        use security::credentials::{
            CredentialVault, CredentialVaultError, MacOsInstallationKeyStore,
        };

        let key_store = MacOsInstallationKeyStore::new("com.c4os.desktop");
        match CredentialVault::open_or_create_with_installation_key(
            c4os_home.join("vault/credentials.vault"),
            &key_store,
        ) {
            Ok(vault) => Ok(Self {
                _vault: Some(vault),
                _requires_explicit_fallback: false,
            }),
            Err(CredentialVaultError::KeychainUnavailable) => Ok(Self {
                _vault: None,
                _requires_explicit_fallback: true,
            }),
            Err(error) => Err(error),
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn initialize(
        _c4os_home: &std::path::Path,
    ) -> Result<Self, security::credentials::CredentialVaultError> {
        Ok(Self {
            _vault: None,
            _requires_explicit_fallback: true,
        })
    }
}

fn platform_boundary_error(
    correlation_id: protocol::CorrelationId,
    code: ProtocolErrorCode,
    message: &'static str,
    retryable: bool,
) -> ProtocolError {
    ProtocolError::new(code, message, retryable).with_correlation(correlation_id)
}

fn invalid_picker_selection(correlation_id: protocol::CorrelationId) -> ProtocolError {
    platform_boundary_error(
        correlation_id,
        ProtocolErrorCode::InvalidPayload,
        "The native picker selection is invalid",
        false,
    )
}

fn validate_snapshot_request(
    request: &SnapshotRequest,
) -> Result<(), protocol::StructuredCoreError> {
    protocol::snapshot_envelope(request.clone(), StateGeneration::default(), ()).map(|_| ())
}

fn native_picker_selection(
    selected: FilePath,
    purpose: PickerPurpose,
    correlation_id: protocol::CorrelationId,
) -> Result<NativePickerSelection, ProtocolError> {
    let selected = selected
        .into_path()
        .map_err(|_| invalid_picker_selection(correlation_id.clone()))?;
    let object_kind = purpose.policy().object_kind;
    let normalized = normalize_picker_path(&selected, purpose)
        .map_err(|_| invalid_picker_selection(correlation_id.clone()))?;
    NativePickerSelection::new(normalized, object_kind)
        .map_err(|_| invalid_picker_selection(correlation_id))
}

fn normalize_picker_path(path: &Path, purpose: PickerPurpose) -> Result<PathBuf, std::io::Error> {
    if !path.is_absolute() {
        return Err(std::io::Error::other("picker path must be absolute"));
    }

    if purpose == PickerPurpose::SaveWorkspaceArchive {
        let file_name = path
            .file_name()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| std::io::Error::other("picker target has no file name"))?;
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("picker target has no parent"))?;
        let parent = std::fs::canonicalize(parent)?;
        let normalized = parent.join(file_name);
        if normalized.exists() {
            let metadata = std::fs::symlink_metadata(&normalized)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(std::io::Error::other("picker target is not a regular file"));
            }
        }
        return Ok(normalized);
    }

    let selected_metadata = std::fs::symlink_metadata(path)?;
    if selected_metadata.file_type().is_symlink() {
        return Err(std::io::Error::other("picker target is a symbolic link"));
    }
    let normalized = std::fs::canonicalize(path)?;
    let metadata = std::fs::metadata(&normalized)?;
    match purpose.policy().object_kind {
        PickerObjectKind::File if metadata.is_file() => Ok(normalized),
        PickerObjectKind::Folder if metadata.is_dir() => Ok(normalized),
        _ => Err(std::io::Error::other("picker target kind does not match")),
    }
}

#[tauri::command]
fn platform_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<PlatformSnapshot>, protocol::StructuredCoreError> {
    protocol::snapshot_envelope(
        request,
        StateGeneration::default(),
        core.platform_snapshot.clone(),
    )
}

#[tauri::command]
fn platform_reveal_main(
    app: tauri::AppHandle,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<PlatformRevealSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let correlation_id = request.correlation_id.clone();
    let window = app.get_webview_window("main").ok_or_else(|| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::Unavailable,
            "The main window is unavailable",
            true,
        )
    })?;
    window.show().map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::Unavailable,
            "The main window could not be revealed",
            true,
        )
    })?;
    window.set_focus().map_err(|_| {
        platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Unavailable,
            "The main window could not be focused",
            true,
        )
    })?;
    protocol::snapshot_envelope(
        request,
        StateGeneration::default(),
        PlatformRevealSnapshot {
            revealed: true,
            fallback: false,
        },
    )
}

#[tauri::command]
async fn platform_pick(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    picker: NativePickerRequest,
) -> Result<ProtocolEnvelope<PickerOutcome>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    if picker.request_id != request.request_id {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::CorrelationMismatch,
            "The picker request identity did not match its envelope",
            false,
        ));
    }
    picker
        .validate()
        .map_err(|_| invalid_picker_selection(request.correlation_id.clone()))?;

    let dialog = app.dialog().file().set_title(match picker.purpose {
        PickerPurpose::OpenProjectFolder => "Open Project Folder",
        PickerPurpose::RelocateProjectFolder => "Relocate Project Folder",
        PickerPurpose::OpenWorkspaceArchive => "Open C4OS Workspace",
        PickerPurpose::SaveWorkspaceArchive => "Save C4OS Workspace",
        PickerPurpose::AttachChatFiles => "Attach Files",
        PickerPurpose::OpenFile => "Open File",
    });
    let selected = match picker.purpose {
        PickerPurpose::OpenProjectFolder | PickerPurpose::RelocateProjectFolder => {
            dialog.blocking_pick_folder().map(|path| vec![path])
        }
        PickerPurpose::OpenWorkspaceArchive => dialog
            .add_filter("C4OS Workspace", &["zip"])
            .blocking_pick_file()
            .map(|path| vec![path]),
        PickerPurpose::SaveWorkspaceArchive => dialog
            .add_filter("C4OS Workspace", &["zip"])
            .set_file_name("Workspace.c4os.zip")
            .blocking_save_file()
            .map(|path| vec![path]),
        PickerPurpose::AttachChatFiles => dialog.blocking_pick_files(),
        PickerPurpose::OpenFile => dialog.blocking_pick_file().map(|path| vec![path]),
    };

    let Some(selected) = selected else {
        let outcome = core
            .platform
            .cancelled_picker(&picker)
            .map_err(|_| invalid_picker_selection(request.correlation_id.clone()))?;
        return protocol::snapshot_envelope(request, StateGeneration::default(), outcome);
    };
    let selections = selected
        .into_iter()
        .map(|path| native_picker_selection(path, picker.purpose, request.correlation_id.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let grant_ids = (0..selections.len())
        .map(|_| protocol::PickerGrantId::new(format!("picker-grant-{}", Uuid::new_v4())))
        .collect::<Result<Vec<_>, _>>()?;
    let issued_at_ms = current_time_ms().map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Internal,
            "The picker grant clock is unavailable",
            true,
        )
    })?;
    let snapshots = {
        let mut registry = core.picker_grants.lock().map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Internal,
                "The picker grant registry is unavailable",
                true,
            )
        })?;
        registry
            .register_batch(&picker, &selections, grant_ids.clone(), issued_at_ms)
            .map_err(|_| invalid_picker_selection(request.correlation_id.clone()))?
    };
    let outcome = match core
        .platform
        .selected_picker(&picker, &selections, snapshots)
    {
        Ok(outcome) => outcome,
        Err(_) => {
            if let Ok(mut registry) = core.picker_grants.lock() {
                for grant_id in &grant_ids {
                    let _ = registry.take(grant_id);
                }
            }
            return Err(invalid_picker_selection(request.correlation_id));
        }
    };
    protocol::snapshot_envelope(request, StateGeneration::default(), outcome)
}

#[tauri::command]
fn foundation_snapshot(
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<FoundationSnapshot>, protocol::StructuredCoreError> {
    protocol::foundation_snapshot(request)
}

#[tauri::command]
fn workspace_start_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<WorkspaceStartSnapshot>, protocol::StructuredCoreError> {
    let correlation_id = request.correlation_id.clone();
    let state = core::services::load_workspace_start_state(&core.database)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let recents = state
        .recents
        .into_iter()
        .map(|recent| {
            Ok(WorkspaceRecentSnapshot {
                workspace_id: WorkspaceId::new(recent.workspace_id)?,
                display_name: recent.display_name,
                last_opened_at: recent.last_opened_at,
                is_missing: recent.is_missing,
            })
        })
        .collect::<Result<Vec<_>, ProtocolError>>()?;

    protocol::workspace_start_snapshot(
        request,
        WorkspaceStartSnapshot {
            protocol_version: protocol::PROTOCOL_VERSION,
            generation: StateGeneration(state.generation),
            authority: "rust-core".into(),
            recents,
        },
    )
}

#[tauri::command]
fn runtime_core_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<RuntimeCoreSnapshot>, protocol::StructuredCoreError> {
    let correlation_id = request.correlation_id.clone();
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let (runtime, capability_generation) = core
        .runtime
        .snapshot_with_capability_generation(now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    let generation = StateGeneration(runtime.generation);
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let pending_approvals = core
        .runtime_production
        .load(request.correlation_id.clone())?
        .pending_approvals()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .into_iter()
        .map(|approval| RuntimeApprovalSummary {
            runtime_id: approval.runtime_id,
            correlation_id: approval.correlation_id,
            prompt_id: approval.prompt_id,
        })
        .collect();
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let pending_approvals = Vec::new();
    let payload = RuntimeCoreSnapshot {
        authority: "rust-core",
        provider_generation: runtime.providers.generation,
        capability_generation,
        runtime_generation: runtime.runtimes.state_generation,
        onboarding_ready: runtime.onboarding_ready,
        providers: runtime
            .providers
            .providers
            .into_iter()
            .map(|record| RuntimeProviderSummary {
                provider_id: record.profile.provider_id,
                display_name: record.profile.display_name,
                enabled: record.profile.enabled,
                test_status: record.test_status,
                model_count: record.models.len(),
                selected_model_id: record.selected_model_id,
            })
            .collect(),
        runtimes: runtime
            .runtimes
            .records
            .into_iter()
            .map(|record| RuntimeProcessSummary {
                runtime_id: record.installation.runtime_id,
                runtime_kind: record.installation.runtime_kind,
                native_version: record.installation.native_version,
                lifecycle: record.lifecycle,
                health: record.health,
                process_generation: record.process_generation,
            })
            .collect(),
        pending_approvals,
    };
    protocol::snapshot_envelope(request, generation, payload)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tauri::command]
fn runtime_production_activate(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    runtime_id: String,
) -> Result<
    ProtocolEnvelope<runtime::production_application::ActivatedProductionRuntime>,
    protocol::StructuredCoreError,
> {
    let correlation_id = request.correlation_id.clone();
    let now_ms =
        current_time_ms().map_err(|_| runtime_production_unavailable(correlation_id.clone()))?;
    let current_generation = StateGeneration(
        core.runtime
            .snapshot(now_ms)
            .map_err(|_| runtime_production_unavailable(correlation_id.clone()))?
            .generation,
    );
    let _ = protocol::snapshot_envelope(request.clone(), current_generation, ())?;
    let activated = core
        .runtime_production
        .load(correlation_id.clone())?
        .activate_runtime(request.expected_generation.0, &runtime_id, now_ms)
        .map_err(|_| runtime_production_unavailable(correlation_id))?;
    let generation = StateGeneration(activated.coordinator_generation);
    protocol::snapshot_envelope(request, generation, activated)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tauri::command]
fn runtime_production_shutdown(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    runtime_id: String,
) -> Result<ProtocolEnvelope<ProductionRuntimeShutdown>, protocol::StructuredCoreError> {
    let correlation_id = request.correlation_id.clone();
    let now_ms =
        current_time_ms().map_err(|_| runtime_production_unavailable(correlation_id.clone()))?;
    let current_generation = StateGeneration(
        core.runtime
            .snapshot(now_ms)
            .map_err(|_| runtime_production_unavailable(correlation_id.clone()))?
            .generation,
    );
    let _ = protocol::snapshot_envelope(request.clone(), current_generation, ())?;
    let coordinator_generation = core
        .runtime_production
        .load(correlation_id.clone())?
        .shutdown_runtime(request.expected_generation.0, &runtime_id, now_ms)
        .map_err(|_| runtime_production_unavailable(correlation_id))?;
    protocol::snapshot_envelope(
        request,
        StateGeneration(coordinator_generation),
        ProductionRuntimeShutdown {
            runtime_id,
            coordinator_generation,
        },
    )
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tauri::command]
fn runtime_production_pump(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    runtime_id: String,
) -> Result<ProtocolEnvelope<ProductionRuntimePump>, protocol::StructuredCoreError> {
    let request_correlation = request.correlation_id.clone();
    let now_ms = current_time_ms()
        .map_err(|_| runtime_production_unavailable(request_correlation.clone()))?;
    let current_generation = StateGeneration(
        core.runtime
            .snapshot(now_ms)
            .map_err(|_| runtime_production_unavailable(request_correlation.clone()))?
            .generation,
    );
    let _ = protocol::snapshot_envelope(request.clone(), current_generation, ())?;
    let (coordinator_generation, pumped_events) = core
        .runtime_production
        .load(request_correlation.clone())?
        .pump_runtime_once_expected(request.expected_generation.0, &runtime_id, now_ms)
        .map_err(|_| runtime_production_unavailable(request_correlation))?;
    protocol::snapshot_envelope(
        request,
        StateGeneration(coordinator_generation),
        ProductionRuntimePump {
            runtime_id,
            pumped_events,
            coordinator_generation,
        },
    )
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[tauri::command]
fn runtime_production_answer_approval(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    runtime_id: String,
    correlation_id: String,
    prompt_id: String,
    answer: ApprovalAnswer,
) -> Result<ProtocolEnvelope<ProductionRuntimeApprovalSettlement>, protocol::StructuredCoreError> {
    let request_correlation = request.correlation_id.clone();
    let now_ms = current_time_ms()
        .map_err(|_| runtime_production_unavailable(request_correlation.clone()))?;
    let current_generation = StateGeneration(
        core.runtime
            .snapshot(now_ms)
            .map_err(|_| runtime_production_unavailable(request_correlation.clone()))?
            .generation,
    );
    let _ = protocol::snapshot_envelope(request.clone(), current_generation, ())?;
    core.runtime_production
        .load(request_correlation.clone())?
        .answer_runtime_approval(
            request.expected_generation.0,
            &runtime_id,
            &correlation_id,
            &prompt_id,
            answer,
            now_ms,
        )
        .map_err(|_| runtime_production_unavailable(request_correlation))?;
    let generation = StateGeneration(
        core.runtime
            .snapshot(now_ms)
            .map_err(|_| runtime_production_unavailable(request.correlation_id.clone()))?
            .generation,
    );
    protocol::snapshot_envelope(
        request,
        generation,
        ProductionRuntimeApprovalSettlement {
            runtime_id,
            correlation_id,
            prompt_id,
        },
    )
}

fn current_time_ms() -> Result<u64, std::io::Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_millis()
        .try_into()
        .map_err(|_| std::io::Error::other("system time exceeds u64 milliseconds"))
}

fn workspace_state_unavailable(correlation_id: protocol::CorrelationId) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorCode::Unavailable,
        "Workspace state is unavailable",
        true,
    )
    .with_correlation(correlation_id)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn runtime_production_unavailable(correlation_id: protocol::CorrelationId) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorCode::Unavailable,
        "Production runtime state is unavailable",
        true,
    )
    .with_correlation(correlation_id)
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn start_runtime_production_driver(
    application: &Arc<ProductionRuntimeApplication>,
) -> Result<(), std::io::Error> {
    let application = Arc::downgrade(application);
    thread::Builder::new()
        .name("c4os-production-runtime-driver".into())
        .spawn(move || {
            while let Some(application) = application.upgrade() {
                if let Ok(now_ms) = current_time_ms() {
                    let _ = application.pump_all_once(now_ms);
                }
                thread::sleep(Duration::from_millis(25));
            }
        })?;
    Ok(())
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn start_runtime_production_initialization(
    app: tauri::AppHandle,
    resource_dir: PathBuf,
    c4os_home: PathBuf,
    active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    runtime: Arc<RuntimeApplicationService>,
    managed: Arc<ManagedProductionRuntime>,
) -> Result<(), std::io::Error> {
    thread::Builder::new()
        .name("c4os-production-runtime-initialization".into())
        .spawn(move || {
            let initialize = || -> Result<Arc<ProductionRuntimeApplication>, String> {
                let vault = CredentialServiceState::initialize(&c4os_home)
                    .map_err(|error| error.to_string())?
                    .vault();
                let bootstrap =
                    runtime::production::RuntimeProductionBootstrap::new(resource_dir, vault)
                        .map_err(|error| error.to_string())?;
                install_production_broker_facilities(&bootstrap, &app)
                    .map_err(|error| error.to_string())?;

                let workspace_binding = active_workspace
                    .lock()
                    .map_err(|_| "active Workspace state is unavailable".to_string())?
                    .as_ref()
                    .map(|workspace| {
                        (
                            workspace.manifest().workspace_id.to_string(),
                            Arc::clone(workspace.database_actor()),
                        )
                    });
                let now_ms = current_time_ms().map_err(|error| error.to_string())?;
                if let Some((workspace_id, database)) = workspace_binding {
                    let installations = bootstrap
                        .runtime_installations(&workspace_id, &c4os_home)
                        .map_err(|error| error.to_string())?;
                    runtime
                        .bind_workspace_runtime_installations(
                            database,
                            installations.into_iter().collect(),
                            now_ms,
                        )
                        .map_err(|error| error.to_string())?;
                } else {
                    runtime
                        .clear_runtime_installations_without_workspace(now_ms)
                        .map_err(|error| error.to_string())?;
                }

                let application = Arc::new(
                    runtime::production_application::RuntimeProductionApplication::new(
                        Arc::clone(&runtime),
                        bootstrap,
                    ),
                );
                start_runtime_production_driver(&application).map_err(|error| error.to_string())?;
                Ok(application)
            };

            match initialize() {
                Ok(application) => managed.publish(application),
                Err(error) => managed.fail(error),
            }
        })?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .menu(|app| {
            let settings = MenuItemBuilder::with_id(SETTINGS_MENU_ITEM_ID, "Settings…")
                .accelerator(SETTINGS_ACCELERATOR)
                .build(app)?;
            let app_menu = Submenu::with_items(
                app,
                app.package_info().name.clone(),
                true,
                &[
                    &PredefinedMenuItem::about(app, None, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &settings,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::services(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;
            let file_menu = Submenu::with_items(
                app,
                "File",
                true,
                &[&PredefinedMenuItem::close_window(app, None)?],
            )?;
            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;
            let view_menu = Submenu::with_items(
                app,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(app, None)?],
            )?;
            let window_menu = Submenu::with_id_and_items(
                app,
                WINDOW_SUBMENU_ID,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?;
            let help_menu = Submenu::with_id_and_items(app, HELP_SUBMENU_ID, "Help", true, &[])?;
            Menu::with_items(
                app,
                &[
                    &app_menu,
                    &file_menu,
                    &edit_menu,
                    &view_menu,
                    &window_menu,
                    &help_menu,
                ],
            )
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() != SETTINGS_MENU_ITEM_ID {
                return;
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.emit(
                    PLATFORM_SETTINGS_EVENT,
                    NativeSettingsEvent {
                        contract_version: PLATFORM_CONTRACT_VERSION,
                        command_id: OPEN_SETTINGS_COMMAND_ID,
                        route: SETTINGS_ROUTE,
                    },
                );
                let _ = window.set_focus();
            }
        })
        .setup(|app| {
            let main_window = app
                .get_webview_window("main")
                .ok_or_else(|| std::io::Error::other("main window is unavailable"))?;
            let initial_theme = match main_window.theme() {
                Ok(tauri::Theme::Dark) => InitialThemeSnapshot::new(
                    ColorScheme::Dark,
                    InitialThemeSource::MacosAppearance,
                ),
                Ok(tauri::Theme::Light) => InitialThemeSnapshot::new(
                    ColorScheme::Light,
                    InitialThemeSource::MacosAppearance,
                ),
                _ => InitialThemeSnapshot::new(
                    ColorScheme::Light,
                    InitialThemeSource::SemanticFallback,
                ),
            };
            let platform = PlatformService::qualify_installed(
                PlatformTarget::current_build(),
                PlatformCapabilities {
                    native_application_menu: true,
                    native_settings_shortcut: true,
                    native_file_picker: true,
                    native_folder_picker: true,
                    native_workspace_picker: true,
                    standard_window_decorations: true,
                },
            )
            .map_err(|error| std::io::Error::other(error.to_string()))?;
            let platform_snapshot = platform.initial_snapshot(initial_theme);
            let c4os_home = app.path().home_dir()?.join(".c4os");
            let home_layout = core::workspace::C4osHomeLayout::new(&c4os_home);
            let (database, _) = core::database::DatabaseActor::start(
                core::database::DatabaseDescriptor::app(&c4os_home),
            )?;
            let database = Arc::new(database);
            let configuration = core::services::ManagedAppConfiguration::start(
                Arc::clone(&database),
                home_layout.clone(),
                core::configuration::ManagedCeilings::default(),
                core::configuration::SecurityConstraints::default(),
            )?;
            let now_ms = current_time_ms()?;
            let active_workspace = core::services::restore_active_workspace(
                &home_layout,
                configuration
                    .snapshot()?
                    .configuration
                    .restore_last_workspace,
                env!("CARGO_PKG_VERSION"),
                core::workspace::ArchiveLimits::default(),
                core::workspace::WorkspaceLockOwner {
                    process_id: std::process::id(),
                    app_instance_id: Uuid::new_v4(),
                    acquired_unix_ms: now_ms,
                    label: "c4os-production".into(),
                },
            )?;
            let runtime = Arc::new(RuntimeApplicationService::restore(
                Arc::clone(&database),
                now_ms,
            )?);
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let runtime_resource_dir = app.path().resource_dir()?;
            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            if let Some(workspace) = &active_workspace {
                runtime.bind_workspace(Arc::clone(workspace.database_actor()))?;
            } else {
                runtime.clear_runtime_installations_without_workspace(now_ms)?;
            }
            let active_workspace = Arc::new(Mutex::new(active_workspace));
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let runtime_production = Arc::new(ManagedProductionRuntime::default());
            app.manage(AppCoreState {
                database,
                _configuration: Mutex::new(configuration),
                _active_workspace: Arc::clone(&active_workspace),
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                runtime_production: Arc::clone(&runtime_production),
                runtime: Arc::clone(&runtime),
                platform,
                platform_snapshot,
                picker_grants: Mutex::new(PickerGrantRegistry::default()),
            });
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            start_runtime_production_initialization(
                app.handle().clone(),
                runtime_resource_dir,
                c4os_home,
                active_workspace,
                runtime,
                runtime_production,
            )?;
            let fallback_app = app.handle().clone();
            thread::Builder::new()
                .name("c4os-initial-reveal-fallback".into())
                .spawn(move || {
                    thread::sleep(Duration::from_millis(INITIAL_REVEAL_FALLBACK_MS));
                    let dispatcher = fallback_app.clone();
                    let _ = dispatcher.run_on_main_thread(move || {
                        let Some(window) = fallback_app.get_webview_window("main") else {
                            return;
                        };
                        if !window.is_visible().unwrap_or(false) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    });
                })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform_snapshot,
            platform_reveal_main,
            platform_pick,
            foundation_snapshot,
            workspace_start_snapshot,
            runtime_core_snapshot,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            runtime_production_activate,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            runtime_production_shutdown,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            runtime_production_pump,
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            runtime_production_answer_approval
        ])
        .run(tauri::generate_context!())
        .expect("C4OS application runtime failed");
}

#[cfg(test)]
mod atomic_capability_publication_tests {
    use super::*;
    use std::sync::mpsc::{RecvTimeoutError, sync_channel};
    use std::thread;
    use std::time::Duration;

    use tempfile::TempDir;

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn managed_publication_fails_closed_until_one_complete_value_is_ready() {
        let managed = ManagedPublication::<u8>::default();
        let unavailable = managed
            .load(protocol::CorrelationId::new("runtime-pending").unwrap())
            .unwrap_err();
        assert_eq!(unavailable.code, ProtocolErrorCode::Unavailable);

        managed.publish(Arc::new(7));
        assert_eq!(
            *managed
                .load(protocol::CorrelationId::new("runtime-ready").unwrap())
                .unwrap(),
            7
        );
    }

    #[test]
    fn capability_candidate_cannot_be_observed_under_the_old_coordinator_generation() {
        let temporary = TempDir::new().unwrap();
        let (database, _) = core::database::DatabaseActor::start(
            core::database::DatabaseDescriptor::app(temporary.path()),
        )
        .unwrap();
        let service = Arc::new(
            RuntimeApplicationService::restore(Arc::new(database), 1_721_300_000_000).unwrap(),
        );
        let (candidate_ready_tx, candidate_ready_rx) = sync_channel(0);
        let (continue_tx, continue_rx) = sync_channel(0);
        let writer_service = Arc::clone(&service);
        let writer = thread::spawn(move || {
            writer_service
                .publish_capability_generation_with_barrier(candidate_ready_tx, continue_rx)
                .unwrap();
        });
        candidate_ready_rx.recv().unwrap();

        let (observed_tx, observed_rx) = sync_channel(0);
        let reader_service = Arc::clone(&service);
        let reader = thread::spawn(move || {
            let (snapshot, capability_generation) = reader_service
                .snapshot_with_capability_generation(1_721_300_000_001)
                .unwrap();
            let coordinator_generation = snapshot.generation;
            observed_tx
                .send((coordinator_generation, capability_generation))
                .unwrap();
        });

        assert_eq!(
            observed_rx.recv_timeout(Duration::from_millis(50)),
            Err(RecvTimeoutError::Timeout),
            "the coordinator guard must keep readers behind the unpublished capability candidate"
        );
        continue_tx.send(()).unwrap();
        assert_eq!(
            observed_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            (1, 1)
        );
        writer.join().unwrap();
        reader.join().unwrap();
    }
}
