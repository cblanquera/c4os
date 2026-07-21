pub mod conversation;
pub mod core;
pub mod execution;
pub mod platform;
pub mod protocol;
pub mod runtime;
pub mod security;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use execution::environment::TrustedProjectRoot;
use execution::git::{
    ActiveProjectRepository, BranchControlVisibility, GitBranchMenuSnapshot, GitBranchOperation,
    GitBranchOutcome, GitBranchRequest, GitError, GitOperationAuthorization, ProductionGitRunner,
    execute_branch_operation, inspect_branch_control, snapshot_branch_menu,
};
use platform::{
    ColorScheme, INITIAL_REVEAL_FALLBACK_MS, InitialThemeSnapshot, InitialThemeSource,
    NativePickerRequest, NativePickerSelection, OPEN_SETTINGS_COMMAND_ID,
    PLATFORM_CONTRACT_VERSION, PickerGrantRegistry, PickerGrantSnapshot, PickerObjectKind,
    PickerOutcome, PickerPurpose, PlatformCapabilities, PlatformService, PlatformSnapshot,
    PlatformTarget, SETTINGS_ACCELERATOR, SETTINGS_MENU_ITEM_ID, SETTINGS_ROUTE,
};
use protocol::{
    AttachmentId, AttemptId, ConversationActivitySnapshot, ConversationAttachmentPreviewInput,
    ConversationAttachmentPreviewSnapshot, ConversationAttachmentSnapshot,
    ConversationAttemptSnapshot, ConversationBranchApprovalAnswer, ConversationBranchApprovalInput,
    ConversationBranchControlSnapshot, ConversationBranchInput, ConversationBranchOperation,
    ConversationBranchSnapshot, ConversationDraftInput, ConversationDraftSnapshot,
    ConversationModelSnapshot, ConversationProjectSnapshot, ConversationRetryInput,
    ConversationSessionSnapshot, ConversationSessionSummarySnapshot, ConversationSnapshot,
    ConversationSubmitInput, ConversationTurnSnapshot, EnvironmentId, FoundationSnapshot,
    PendingConversationSnapshot, PickerGrantId, ProjectId, ProtocolEnvelope, ProtocolError,
    ProtocolErrorCode, RequestId, RuntimeId, SessionId, SnapshotRequest, StateGeneration, TurnId,
    WorkspaceId, WorkspaceRecentSnapshot, WorkspaceStartSnapshot,
};
use runtime::action_bridge::{
    RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization, RuntimeExecutionReceipt,
    RuntimeGatewayDecision,
};
use runtime::broker_worker::BrokerActionApplication;
use runtime::capability::{
    AttachmentMediaType, AttachmentRequirement, CapabilityDescriptor, CapabilityKey,
    DraftRequirements, NumericCapabilityKey, PolicyPreflight, effective_intersection,
};
use runtime::capability_evidence::{
    CapabilityEvidenceError, CapabilityEvidenceRegistry, CapabilityRouteEpoch,
};
use runtime::coordinator::{
    CoordinatedFirstSubmission, CoordinatedRetry, CoordinatedTurn, CoordinatorError,
    CoordinatorOperation, ModelPreflight, RuntimeCoordinator, RuntimeCoordinatorSnapshot,
};
use runtime::dispatch::{
    AppliedDispatchEvent, AttachmentPreflightResolution, BrokerDispatchAuthority,
    CoordinatedCancellation, CoordinatedFirstDispatch, CoordinatedRetryDispatch,
    CoordinatedTurnDispatch, DispatchError, DispatchIdentity, FirstDispatchOptions,
    RetryDispatchOptions, RuntimeDispatchPeer, RuntimeDispatchRegistry, TurnDispatchOptions,
    coordinate_cancellation, coordinate_first_dispatch, coordinate_polled_events,
    coordinate_recovery, coordinate_retry_dispatch, coordinate_turn_dispatch,
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
    AttachmentSnapshot, FirstSubmission, MessageReplyContextSnapshot, RetryRequest,
    RuntimeKind as SessionRuntimeKind, SessionError, SessionRecord, SessionService,
    TerminalAttemptOutcome, TurnSubmission,
};
use runtime::supervisor::{
    CompatibilityState, HealthState, RuntimeInstallation, RuntimeSupervisor,
};
use security::authorization::{
    ApprovalAnswer, AuthorizationToken, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction,
    CanonicalRisk, LiveAuthorityState,
};
use security::gateway::{
    ApprovalResponse, ExecutionPermit, GatewayProposal, NormalizedActionResult,
    NormalizedActionStatus,
};
use security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence, PolicyConfiguration,
    RepositoryState,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
#[cfg(all(debug_assertions, unix))]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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
    active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    conversation_operation: Mutex<()>,
    conversation: Mutex<ConversationApplicationState>,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    runtime_production: Arc<ManagedProductionRuntime>,
    runtime: Arc<RuntimeApplicationService>,
    platform: PlatformService,
    platform_snapshot: PlatformSnapshot,
    picker_grants: Mutex<PickerGrantRegistry>,
    conversation_drop: Mutex<NativeConversationDropState>,
    conversation_branch: Mutex<NativeConversationBranchState>,
}

#[derive(Debug, Default)]
struct NativeConversationDropState {
    paths: Vec<PathBuf>,
    grants: Vec<PickerGrantSnapshot>,
}

#[derive(Default)]
struct NativeConversationBranchState {
    cached: Option<CachedConversationBranchControl>,
    pending: BTreeMap<String, PendingConversationBranchOperation>,
    operation_status: Option<String>,
    operation_message: Option<String>,
}

#[derive(Clone)]
struct CachedConversationBranchControl {
    project_id: String,
    checked_at_ms: u64,
    repository: ActiveProjectRepository,
    menu: GitBranchMenuSnapshot,
}

#[derive(Clone)]
struct PendingConversationBranchOperation {
    project_id: String,
    branch: String,
    operation: ConversationBranchOperation,
    repository: ActiveProjectRepository,
    request: GitBranchRequest,
    action: CanonicalAction,
    live: LiveAuthorityState,
}

const CONVERSATION_FILE_DROP_EVENT: &str = "c4os://conversation/file-drop";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeConversationFileDropEvent {
    phase: &'static str,
    grants: Vec<PickerGrantSnapshot>,
}

#[derive(Clone, Debug)]
struct ConversationApplicationState {
    generation: u64,
    persistence_generation: Option<u64>,
    active_project_id: Option<String>,
    active_session_id: Option<String>,
    pending: conversation::PendingChatState,
    pending_attachments: Vec<AttachmentSnapshot>,
    pending_next_attachment_reference: u32,
    drafts: BTreeMap<String, DurableConversationDraft>,
}

impl Default for ConversationApplicationState {
    fn default() -> Self {
        Self {
            generation: 0,
            persistence_generation: None,
            active_project_id: None,
            active_session_id: None,
            pending: conversation::PendingChatState::Inactive,
            pending_attachments: Vec::new(),
            pending_next_attachment_reference: 1,
            drafts: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DurableConversationDraft {
    prompt: String,
    attachments: Vec<AttachmentSnapshot>,
    #[serde(default)]
    next_attachment_reference: u32,
    provider_id: Option<String>,
    model_id: Option<String>,
    reasoning_mode: Option<String>,
    mode: String,
    reply_target_id: Option<String>,
}

impl Default for DurableConversationDraft {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            attachments: Vec::new(),
            next_attachment_reference: 1,
            provider_id: None,
            model_id: None,
            reasoning_mode: None,
            mode: String::new(),
            reply_target_id: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DurableConversationDocument {
    schema_version: u16,
    active_project_id: Option<String>,
    active_session_id: Option<String>,
    drafts: BTreeMap<String, DurableConversationDraft>,
}

impl ConversationApplicationState {
    fn restore(
        snapshot: Option<&core::database::WorkspaceSnapshot>,
        persisted: Option<&core::database::WorkspaceConversationStateRecord>,
    ) -> Self {
        let Some(snapshot) = snapshot else {
            return Self::default();
        };
        let fallback_project_id = snapshot
            .projects
            .iter()
            .filter(|project| project.lifecycle_state == core::database::LifecycleState::Active)
            .min_by_key(|project| (project.position, project.project_id.as_str()))
            .map(|project| project.project_id.clone());
        let fallback_session_id = fallback_project_id.as_deref().and_then(|project_id| {
            snapshot
                .chats
                .iter()
                .find(|chat| {
                    chat.project_id == project_id
                        && chat.lifecycle_state == core::database::LifecycleState::Active
                })
                .map(|chat| chat.chat_id.clone())
        });
        let restored = persisted
            .and_then(|record| {
                serde_json::from_str::<DurableConversationDocument>(&record.canonical_document).ok()
            })
            .filter(|document| validate_durable_conversation_document(document, snapshot));
        let active_project_id = restored
            .as_ref()
            .and_then(|document| document.active_project_id.clone())
            .or(fallback_project_id);
        let active_session_id = restored
            .as_ref()
            .and_then(|document| document.active_session_id.clone())
            .or_else(|| {
                active_project_id.as_deref().and_then(|project_id| {
                    snapshot
                        .chats
                        .iter()
                        .find(|chat| {
                            chat.project_id == project_id
                                && chat.lifecycle_state == core::database::LifecycleState::Active
                        })
                        .map(|chat| chat.chat_id.clone())
                })
            })
            .or(fallback_session_id);
        let mut drafts = restored.map(|document| document.drafts).unwrap_or_default();
        for draft in drafts.values_mut() {
            migrate_attachment_references(draft);
        }
        Self {
            generation: snapshot.generation,
            persistence_generation: persisted.map(|record| record.generation),
            active_project_id,
            active_session_id,
            pending: conversation::PendingChatState::Inactive,
            pending_attachments: Vec::new(),
            pending_next_attachment_reference: 1,
            drafts,
        }
    }

    fn advance(&mut self, floor: u64) -> Result<u64, ProtocolError> {
        self.generation = self.generation.max(floor).checked_add(1).ok_or_else(|| {
            ProtocolError::new(
                ProtocolErrorCode::Internal,
                "Conversation generation is exhausted",
                false,
            )
        })?;
        Ok(self.generation)
    }

    fn active_draft(&self) -> DurableConversationDraft {
        if !matches!(self.pending, conversation::PendingChatState::Inactive) {
            let (prompt, reply_target_id) = match &self.pending {
                conversation::PendingChatState::Draft(draft) => {
                    (draft.prompt.clone(), draft.reply_target_id.clone())
                }
                conversation::PendingChatState::PromotionRequested(promotion) => {
                    (promotion.prompt.clone().unwrap_or_default(), None)
                }
                conversation::PendingChatState::Inactive => unreachable!(),
            };
            return DurableConversationDraft {
                prompt,
                attachments: self.pending_attachments.clone(),
                next_attachment_reference: self.pending_next_attachment_reference,
                mode: "chat".into(),
                reply_target_id,
                ..DurableConversationDraft::default()
            };
        }
        self.active_session_id
            .as_ref()
            .and_then(|session_id| self.drafts.get(session_id))
            .cloned()
            .unwrap_or_else(|| DurableConversationDraft {
                mode: "chat".into(),
                ..DurableConversationDraft::default()
            })
    }

    fn persist(
        &mut self,
        database: &core::database::DatabaseActor,
        workspace_id: &str,
        updated_at_ms: u64,
    ) -> Result<u64, core::database::DatabaseError> {
        if !matches!(self.pending, conversation::PendingChatState::Inactive) {
            return Ok(self.generation);
        }
        let document = DurableConversationDocument {
            schema_version: 1,
            active_project_id: self.active_project_id.clone(),
            active_session_id: self.active_session_id.clone(),
            drafts: self.drafts.clone(),
        };
        let canonical_document = serde_json::to_string(&document)
            .map_err(|error| core::database::DatabaseError::Validation(error.to_string()))?;
        let generation = self
            .persistence_generation
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| {
                core::database::DatabaseError::Validation(
                    "conversation persistence generation is exhausted".into(),
                )
            })?;
        let durable_generation = database.save_conversation_state(
            core::database::WorkspaceConversationStateRecord {
                workspace_id: workspace_id.into(),
                generation,
                canonical_document,
                updated_at_ms,
            },
            self.persistence_generation,
        )?;
        self.persistence_generation = Some(generation);
        self.generation = self.generation.max(durable_generation);
        Ok(durable_generation)
    }
}

fn migrate_attachment_references(draft: &mut DurableConversationDraft) {
    let mut next_reference = draft.next_attachment_reference.max(1);
    for attachment in &draft.attachments {
        if attachment.original_reference > 0 {
            next_reference = next_reference.max(attachment.original_reference.saturating_add(1));
        }
    }
    for attachment in &mut draft.attachments {
        if attachment.original_reference == 0 {
            attachment.original_reference = next_reference;
            next_reference = next_reference.saturating_add(1);
        }
    }
    draft.next_attachment_reference = next_reference;
}

fn attachment_reference_ledger(
    attachments: &[AttachmentSnapshot],
    next_reference: u32,
) -> Result<conversation::AttachmentDraft, conversation::AttachmentDraftError> {
    conversation::AttachmentDraft::restore(
        attachments
            .iter()
            .map(|attachment| conversation::DraftAttachment {
                attachment_id: attachment.attachment_id.clone(),
                stable_reference: attachment.stable_reference.clone(),
                display_name: attachment.display_name.clone(),
                original_reference: attachment.original_reference,
            })
            .collect(),
        next_reference,
    )
}

fn reconcile_attachment_references(
    retained: &mut Vec<AttachmentSnapshot>,
    imported: Vec<AttachmentSnapshot>,
    next_reference: &mut u32,
) -> Result<(), conversation::AttachmentDraftError> {
    let mut ledger = attachment_reference_ledger(retained, *next_reference)?;
    for mut attachment in imported {
        ledger = ledger.add(conversation::DraftAttachmentInput {
            attachment_id: attachment.attachment_id.clone(),
            stable_reference: attachment.stable_reference.clone(),
            display_name: attachment.display_name.clone(),
        })?;
        attachment.original_reference = ledger
            .items()
            .iter()
            .find(|item| item.attachment_id == attachment.attachment_id)
            .map(|item| item.original_reference)
            .ok_or(conversation::AttachmentDraftError::InvalidDraft)?;
        retained.push(attachment);
    }
    *next_reference = ledger.next_reference();
    Ok(())
}

fn validate_durable_conversation_document(
    document: &DurableConversationDocument,
    snapshot: &core::database::WorkspaceSnapshot,
) -> bool {
    if document.schema_version != 1 || document.drafts.len() > 250 {
        return false;
    }
    let active_projects = snapshot
        .projects
        .iter()
        .filter(|project| project.lifecycle_state == core::database::LifecycleState::Active)
        .map(|project| project.project_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let active_chats = snapshot
        .chats
        .iter()
        .filter(|chat| chat.lifecycle_state == core::database::LifecycleState::Active)
        .map(|chat| (chat.chat_id.as_str(), chat.project_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    if document
        .active_project_id
        .as_deref()
        .is_some_and(|project_id| !active_projects.contains(project_id))
        || document
            .active_session_id
            .as_deref()
            .is_some_and(|session_id| {
                active_chats.get(session_id).copied() != document.active_project_id.as_deref()
            })
    {
        return false;
    }
    document.drafts.iter().all(|(session_id, draft)| {
        let mut normalized_draft = draft.clone();
        migrate_attachment_references(&mut normalized_draft);
        active_chats.contains_key(session_id.as_str())
            && draft.prompt.len() <= protocol::MAX_TEXT_BYTES
            && !draft.prompt.contains('\0')
            && draft.attachments.len() <= protocol::MAX_ATTACHMENTS
            && matches!(
                draft.mode.as_str(),
                "chat" | "files" | "browser" | "terminal"
            )
            && draft
                .provider_id
                .as_deref()
                .is_none_or(|value| !value.trim().is_empty() && value.len() <= 256)
            && draft
                .model_id
                .as_deref()
                .is_none_or(|value| !value.trim().is_empty() && value.len() <= 512)
            && draft
                .reasoning_mode
                .as_deref()
                .is_none_or(|value| matches!(value, "off" | "low" | "medium" | "high"))
            && draft
                .reply_target_id
                .as_deref()
                .is_none_or(|value| !value.trim().is_empty() && value.len() <= 512)
            && attachment_reference_ledger(
                &normalized_draft.attachments,
                normalized_draft.next_attachment_reference,
            )
            .is_ok()
    })
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationFirstDispatchIntent {
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
    pub submitted_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationTurnDispatchIntent {
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
    pub reply_context: Option<MessageReplyContextSnapshot>,
    pub submitted_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnRuntimeDispatchIntent {
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
    pub reply_context: Option<MessageReplyContextSnapshot>,
    pub draft: DraftRequirements,
    pub submitted_at_ms: u64,
    pub preflight_at_ms: u64,
}

enum ConversationSubmissionDispatch {
    First(ConversationFirstDispatchIntent),
    Turn(ConversationTurnDispatchIntent),
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

    /// Resolves the exact runtime-bound effective descriptors that may be
    /// selected by the active Chat. Catalog claims never become composer
    /// capability truth when process evidence is absent or stale.
    fn effective_conversation_models(
        &self,
        preferred_runtime_id: Option<&str>,
        now_ms: u64,
    ) -> Result<BTreeMap<(String, String), CapabilityDescriptor>, RuntimeApplicationError> {
        let coordinator = self.coordinator()?;
        let snapshot = coordinator.snapshot(now_ms);
        let runtime_id = preferred_runtime_id
            .filter(|runtime_id| {
                snapshot.runtimes.records.iter().any(|runtime| {
                    runtime.installation.runtime_id == **runtime_id
                        && runtime.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                        && runtime.health == HealthState::Healthy
                })
            })
            .map(ToOwned::to_owned)
            .or_else(|| {
                snapshot
                    .runtimes
                    .records
                    .iter()
                    .filter(|runtime| {
                        runtime.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                            && runtime.health == HealthState::Healthy
                    })
                    .map(|runtime| runtime.installation.runtime_id.clone())
                    .min()
            });
        let Some(runtime_id) = runtime_id else {
            return Ok(BTreeMap::new());
        };
        let capabilities = self.capabilities()?;
        let mut effective = BTreeMap::new();
        for provider in snapshot
            .providers
            .providers
            .iter()
            .filter(|provider| provider.profile.enabled)
        {
            for model in provider
                .models
                .values()
                .filter(|model| model.is_production_ready_at(now_ms))
            {
                let Ok(route) = capabilities.route_for_runtime_model(
                    &runtime_id,
                    &provider.profile.provider_id,
                    &provider.profile.endpoint.endpoint_id,
                    &model.model_id,
                    now_ms,
                ) else {
                    continue;
                };
                let Ok(layers) = capabilities.layers(&route, now_ms) else {
                    continue;
                };
                let Ok(descriptor) = effective_intersection(&layers, now_ms) else {
                    continue;
                };
                effective.insert(
                    (provider.profile.provider_id.clone(), model.model_id.clone()),
                    descriptor,
                );
            }
        }
        Ok(effective)
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

    pub fn discard_provisional(
        &self,
        expected_coordinator_generation: u64,
        session_id: &str,
        discarded_at_ms: u64,
    ) -> Result<CoordinatorOperation<SessionRecord>, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(
            &coordinator,
            expected_coordinator_generation,
            discarded_at_ms,
        )?;
        Ok(coordinator.discard_provisional(session_id)?)
    }

    pub fn durable_sessions(&self) -> Result<Vec<SessionRecord>, RuntimeApplicationError> {
        Ok(self.raw_coordinator()?.durable_sessions()?)
    }

    pub fn session(&self, session_id: &str) -> Result<SessionRecord, RuntimeApplicationError> {
        Ok(self.raw_coordinator()?.session(session_id)?)
    }

    /// Builds the first Chat dispatch from current Rust-owned provider,
    /// capability, process, resource, and policy truth. The renderer may
    /// choose among exposed provider/model identities but cannot provide any
    /// authority snapshot or generation used by the run.
    pub fn dispatch_conversation_first(
        &self,
        intent: ConversationFirstDispatchIntent,
    ) -> Result<CoordinatedFirstDispatch, RuntimeApplicationError> {
        let title = conversation::derive_chat_title(
            intent.prompt.as_deref(),
            intent
                .attachments
                .iter()
                .map(|attachment| attachment.display_name.as_str()),
        )
        .ok_or(CoordinatorError::Session(SessionError::EmptySubmission))?;
        let (coordinator_generation, provider_id, model_id, authority, draft) = {
            let coordinator = self.coordinator()?;
            let snapshot = coordinator.snapshot(intent.submitted_at_ms);
            let (provider_id, model_id) = resolve_conversation_provider_model(
                &snapshot,
                intent.provider_id.as_deref(),
                intent.model_id.as_deref(),
                intent.submitted_at_ms,
            )?;
            let capabilities = self.capabilities()?;
            let (runtime_id, effective) = snapshot
                .runtimes
                .records
                .iter()
                .filter(|runtime| {
                    runtime.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                        && runtime.health == HealthState::Healthy
                })
                .find_map(|runtime| {
                    let runtime_id = runtime.installation.runtime_id.clone();
                    let route = capability_route_for_runtime(
                        &coordinator,
                        &capabilities,
                        &runtime_id,
                        &provider_id,
                        &model_id,
                        intent.submitted_at_ms,
                    )
                    .ok()?;
                    let layers = capabilities.layers(&route, intent.submitted_at_ms).ok()?;
                    let effective = effective_intersection(&layers, intent.submitted_at_ms).ok()?;
                    Some((runtime_id, effective))
                })
                .ok_or(CoordinatorError::RuntimeNotReady)?;
            let authority = AuthorityMintIntent {
                workspace_id: intent.workspace_id.clone(),
                project_id: Some(intent.project_id.clone()),
                runtime_id: runtime_id.clone(),
                expected_authority_generation: self.dispatch_authority.generation()?,
                expected_capability_generation: capabilities.generation(),
            };
            let installed_resources = self
                .dispatch_authority
                .installed_resource_preflight_for_active_project(&authority, &effective)?;
            let policy = self
                .policy_authority
                .lock()
                .map_err(|_| RuntimeApplicationError::Unavailable)?;
            let requested_output_tokens = effective
                .numeric_maximum(NumericCapabilityKey::OutputTokens)
                .unwrap_or(1_024)
                .clamp(1, 4_096);
            let estimated_input_tokens = intent
                .prompt
                .as_deref()
                .map_or(0, |prompt| prompt.chars().count() as u64 / 4)
                .saturating_add(intent.attachments.len() as u64)
                .max(1);
            let draft = DraftRequirements {
                attachments: intent
                    .attachments
                    .iter()
                    .map(|attachment| AttachmentRequirement {
                        attachment_id: attachment.attachment_id.clone(),
                        media_type: attachment_media_type(&attachment.media_type),
                        mime_type: attachment.media_type.clone(),
                        bytes: attachment.byte_length,
                    })
                    .collect(),
                reasoning_mode: intent.reasoning_mode.clone(),
                requires_tools: false,
                requires_json_schema: false,
                prefers_streaming: effective.feature_state(CapabilityKey::Streaming).usable(),
                estimated_input_tokens,
                requested_output_tokens,
                installed_resources,
                policy: PolicyPreflight {
                    snapshot_id: format!(
                        "policy-{}-{}",
                        policy.policy_version, policy.revocation_epoch
                    ),
                    version: policy.policy_version,
                    tool_use_allowed: false,
                    attachment_conversion_allowed: false,
                },
            };
            (snapshot.generation, provider_id, model_id, authority, draft)
        };
        let submission_id = Uuid::new_v4();
        self.dispatch_first(
            coordinator_generation,
            FirstRuntimeDispatchIntent {
                authority,
                provider_id,
                model_id,
                session_id: intent.session_id,
                turn_id: format!("turn-{submission_id}"),
                attempt_id: format!("attempt-{submission_id}"),
                authorization_scope_id: format!("authorization-{submission_id}"),
                correlation_id: format!("run-{submission_id}"),
                prompt: intent.prompt,
                attachments: intent.attachments,
                draft,
                submitted_at_ms: intent.submitted_at_ms,
                preflight_at_ms: intent.submitted_at_ms,
            },
            FirstDispatchOptions {
                title,
                credential_reference: None,
                credential_lease_id: None,
                attachment_resolution: AttachmentPreflightResolution::NotRequired,
                broker_authority: None,
            },
        )
    }

    /// Builds a follow-up dispatch from the saved Chat binding plus current
    /// Rust-owned provider, capability, resource, process, and policy truth.
    /// Workspace, Project, runtime, adapter, and environment remain fixed to
    /// the first durable submission.
    pub fn dispatch_conversation_turn(
        &self,
        intent: ConversationTurnDispatchIntent,
    ) -> Result<CoordinatedTurnDispatch, RuntimeApplicationError> {
        let (coordinator_generation, provider_id, model_id, authority, draft) = {
            let coordinator = self.coordinator()?;
            let session = coordinator.session(&intent.session_id)?;
            let binding = session.binding().ok_or(DispatchError::InvalidRequest)?;
            if binding.workspace_id != intent.workspace_id
                || binding.project_id.as_deref() != Some(intent.project_id.as_str())
            {
                return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
            }
            let snapshot = coordinator.snapshot(intent.submitted_at_ms);
            let (provider_id, model_id) = resolve_conversation_provider_model(
                &snapshot,
                intent.provider_id.as_deref(),
                intent.model_id.as_deref(),
                intent.submitted_at_ms,
            )?;
            if !snapshot.runtimes.records.iter().any(|runtime| {
                runtime.installation.runtime_id == binding.runtime_id
                    && runtime.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                    && runtime.health == HealthState::Healthy
            }) {
                return Err(CoordinatorError::RuntimeNotReady.into());
            }
            let capabilities = self.capabilities()?;
            let route = capability_route_for_runtime(
                &coordinator,
                &capabilities,
                &binding.runtime_id,
                &provider_id,
                &model_id,
                intent.submitted_at_ms,
            )?;
            let layers = capabilities.layers(&route, intent.submitted_at_ms)?;
            let effective = effective_intersection(&layers, intent.submitted_at_ms)
                .map_err(CoordinatorError::Capability)?;
            let authority = AuthorityMintIntent {
                workspace_id: intent.workspace_id.clone(),
                project_id: Some(intent.project_id.clone()),
                runtime_id: binding.runtime_id.clone(),
                expected_authority_generation: self.dispatch_authority.generation()?,
                expected_capability_generation: capabilities.generation(),
            };
            let installed_resources = self
                .dispatch_authority
                .installed_resource_preflight_for_active_project(&authority, &effective)?;
            let policy = self
                .policy_authority
                .lock()
                .map_err(|_| RuntimeApplicationError::Unavailable)?;
            let requested_output_tokens = effective
                .numeric_maximum(NumericCapabilityKey::OutputTokens)
                .unwrap_or(1_024)
                .clamp(1, 4_096);
            let estimated_input_tokens = intent
                .prompt
                .as_deref()
                .map_or(0, |prompt| prompt.chars().count() as u64 / 4)
                .saturating_add(intent.attachments.len() as u64)
                .saturating_add(
                    intent
                        .reply_context
                        .as_ref()
                        .map_or(0, |reply| reply.source_excerpt.chars().count() as u64 / 4),
                )
                .max(1);
            let draft = DraftRequirements {
                attachments: intent
                    .attachments
                    .iter()
                    .map(|attachment| AttachmentRequirement {
                        attachment_id: attachment.attachment_id.clone(),
                        media_type: attachment_media_type(&attachment.media_type),
                        mime_type: attachment.media_type.clone(),
                        bytes: attachment.byte_length,
                    })
                    .collect(),
                reasoning_mode: intent.reasoning_mode.clone(),
                requires_tools: false,
                requires_json_schema: false,
                prefers_streaming: effective.feature_state(CapabilityKey::Streaming).usable(),
                estimated_input_tokens,
                requested_output_tokens,
                installed_resources,
                policy: PolicyPreflight {
                    snapshot_id: format!(
                        "policy-{}-{}",
                        policy.policy_version, policy.revocation_epoch
                    ),
                    version: policy.policy_version,
                    tool_use_allowed: false,
                    attachment_conversion_allowed: false,
                },
            };
            (snapshot.generation, provider_id, model_id, authority, draft)
        };
        let submission_id = Uuid::new_v4();
        self.dispatch_turn(
            coordinator_generation,
            TurnRuntimeDispatchIntent {
                authority,
                provider_id,
                model_id,
                session_id: intent.session_id,
                turn_id: format!("turn-{submission_id}"),
                attempt_id: format!("attempt-{submission_id}"),
                authorization_scope_id: format!("authorization-{submission_id}"),
                correlation_id: format!("run-{submission_id}"),
                prompt: intent.prompt,
                attachments: intent.attachments,
                reply_context: intent.reply_context,
                draft,
                submitted_at_ms: intent.submitted_at_ms,
                preflight_at_ms: intent.submitted_at_ms,
            },
            TurnDispatchOptions {
                credential_reference: None,
                credential_lease_id: None,
                attachment_resolution: AttachmentPreflightResolution::NotRequired,
                broker_authority: None,
            },
        )
    }

    /// Retries one terminal Conversation attempt with a fresh run identity.
    /// The immutable Chat binding and original turn content stay coordinator
    /// owned; only an already-selected production route and reasoning choice
    /// may differ for the new attempt.
    pub fn retry_conversation_attempt(
        &self,
        session_id: &str,
        parent_attempt_id: &str,
        provider_id: Option<String>,
        model_id: Option<String>,
        reasoning_mode: Option<String>,
        created_at_ms: u64,
    ) -> Result<CoordinatedRetryDispatch, RuntimeApplicationError> {
        let (coordinator_generation, provider_id, model_id, authority, draft) = {
            let coordinator = self.coordinator()?;
            let session = coordinator.session(session_id)?;
            let binding = session
                .binding()
                .cloned()
                .ok_or(DispatchError::InvalidRequest)?;
            let parent = session
                .attempt(parent_attempt_id)
                .ok_or(DispatchError::InvalidRequest)?;
            let turn = session
                .turn(&parent.turn_id)
                .cloned()
                .ok_or(DispatchError::InvalidRequest)?;
            let requested_provider_id =
                provider_id.unwrap_or_else(|| parent.context.model_route.provider_id.clone());
            let requested_model_id =
                model_id.unwrap_or_else(|| parent.context.model_route.model_id.clone());
            let snapshot = coordinator.snapshot(created_at_ms);
            let provider = snapshot
                .providers
                .providers
                .iter()
                .filter(|provider| provider.profile.enabled)
                .find(|provider| {
                    provider.profile.provider_id == requested_provider_id
                        && provider.selected_model_id.as_deref()
                            == Some(requested_model_id.as_str())
                })
                .ok_or(CoordinatorError::ModelRouteUnavailable)?;
            if !snapshot.runtimes.records.iter().any(|runtime| {
                runtime.installation.runtime_id == binding.runtime_id
                    && runtime.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                    && runtime.health == HealthState::Healthy
            }) {
                return Err(CoordinatorError::RuntimeNotReady.into());
            }
            let capabilities = self.capabilities()?;
            let route = capability_route_for_runtime(
                &coordinator,
                &capabilities,
                &binding.runtime_id,
                &provider.profile.provider_id,
                &requested_model_id,
                created_at_ms,
            )?;
            let layers = capabilities.layers(&route, created_at_ms)?;
            let effective = effective_intersection(&layers, created_at_ms)
                .map_err(CoordinatorError::Capability)?;
            let authority = AuthorityMintIntent {
                workspace_id: binding.workspace_id.clone(),
                project_id: binding.project_id.clone(),
                runtime_id: binding.runtime_id.clone(),
                expected_authority_generation: self.dispatch_authority.generation()?,
                expected_capability_generation: capabilities.generation(),
            };
            let installed_resources = self
                .dispatch_authority
                .installed_resource_preflight_for_active_project(&authority, &effective)?;
            let policy = self
                .policy_authority
                .lock()
                .map_err(|_| RuntimeApplicationError::Unavailable)?;
            let requested_output_tokens = effective
                .numeric_maximum(NumericCapabilityKey::OutputTokens)
                .unwrap_or(1_024)
                .clamp(1, 4_096);
            let estimated_input_tokens = turn
                .prompt
                .as_deref()
                .map_or(0, |prompt| prompt.chars().count() as u64 / 4)
                .saturating_add(turn.attachments.len() as u64)
                .max(1);
            let draft = DraftRequirements {
                attachments: turn
                    .attachments
                    .iter()
                    .map(|attachment| AttachmentRequirement {
                        attachment_id: attachment.attachment_id.clone(),
                        media_type: attachment_media_type(&attachment.media_type),
                        mime_type: attachment.media_type.clone(),
                        bytes: attachment.byte_length,
                    })
                    .collect(),
                reasoning_mode,
                requires_tools: false,
                requires_json_schema: false,
                prefers_streaming: effective.feature_state(CapabilityKey::Streaming).usable(),
                estimated_input_tokens,
                requested_output_tokens,
                installed_resources,
                policy: PolicyPreflight {
                    snapshot_id: format!(
                        "policy-{}-{}",
                        policy.policy_version, policy.revocation_epoch
                    ),
                    version: policy.policy_version,
                    tool_use_allowed: false,
                    attachment_conversion_allowed: false,
                },
            };
            (
                snapshot.generation,
                provider.profile.provider_id.clone(),
                requested_model_id,
                authority,
                draft,
            )
        };
        let retry_id = Uuid::new_v4();
        self.retry_dispatch(
            coordinator_generation,
            RetryRuntimeDispatchIntent {
                authority,
                provider_id,
                model_id,
                session_id: session_id.into(),
                parent_attempt_id: parent_attempt_id.into(),
                attempt_id: format!("attempt-{retry_id}"),
                authorization_scope_id: format!("authorization-{retry_id}"),
                correlation_id: format!("run-{retry_id}"),
                automatic: false,
                reviewed_unknown_effect: false,
                draft,
                created_at_ms,
                preflight_at_ms: created_at_ms,
            },
            RetryDispatchOptions {
                credential_reference: None,
                credential_lease_id: None,
                attachment_resolution: AttachmentPreflightResolution::NotRequired,
                broker_authority: None,
            },
        )
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

    pub fn dispatch_turn(
        &self,
        expected_coordinator_generation: u64,
        intent: TurnRuntimeDispatchIntent,
        mut options: TurnDispatchOptions,
    ) -> Result<CoordinatedTurnDispatch, RuntimeApplicationError> {
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
        let preflight = coordinator.model_preflight(
            &intent.provider_id,
            &intent.model_id,
            &capability_layers,
            &intent.draft,
            intent.preflight_at_ms,
        )?;
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
            intent.submitted_at_ms,
        )?;
        let request = CoordinatedTurn {
            submission: TurnSubmission {
                session_id: intent.session_id,
                turn_id: intent.turn_id,
                attempt_id: intent.attempt_id,
                authorization_scope_id: intent.authorization_scope_id,
                correlation_id: intent.correlation_id,
                process_generation: minted.process_generation,
                prompt: intent.prompt,
                attachments: intent.attachments,
                reply_context: intent.reply_context,
                context: minted.context,
                submitted_at_ms: intent.submitted_at_ms,
            },
            provider_id: intent.provider_id,
            selected_model_id: intent.model_id,
            capability_layers,
            draft: intent.draft,
            preflight_at_ms: intent.preflight_at_ms,
        };
        Ok(coordinate_turn_dispatch(
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
    let model = provider
        .models
        .get(model_id)
        .filter(|model| model.is_production_ready_at(now_ms))
        .ok_or(CoordinatorError::ModelRouteUnavailable)?;
    Ok(model.capabilities.route.clone())
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
    if !provider
        .models
        .get(model_id)
        .is_some_and(|model| model.is_production_ready_at(now_ms))
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

fn resolve_conversation_provider_model(
    snapshot: &RuntimeCoordinatorSnapshot,
    requested_provider_id: Option<&str>,
    requested_model_id: Option<&str>,
    now_ms: u64,
) -> Result<(String, String), RuntimeApplicationError> {
    snapshot
        .providers
        .providers
        .iter()
        .filter(|provider| {
            provider.profile.enabled
                && requested_provider_id
                    .is_none_or(|provider_id| provider.profile.provider_id == provider_id)
        })
        .find_map(|provider| {
            let model_id = requested_model_id.or(provider.selected_model_id.as_deref())?;
            provider
                .models
                .get(model_id)
                .filter(|model| model.is_production_ready_at(now_ms))
                .map(|model| (provider.profile.provider_id.clone(), model.model_id.clone()))
        })
        .ok_or_else(|| CoordinatorError::ModelRouteUnavailable.into())
}

fn attachment_media_type(mime_type: &str) -> AttachmentMediaType {
    if mime_type.starts_with("image/") {
        AttachmentMediaType::Image
    } else if mime_type.starts_with("audio/") {
        AttachmentMediaType::Audio
    } else if mime_type.starts_with("video/") {
        AttachmentMediaType::Video
    } else if mime_type == "application/pdf" {
        AttachmentMediaType::Pdf
    } else {
        AttachmentMediaType::OtherFile
    }
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
    protocol::validate_snapshot_request(request)
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

fn conversation_drop_is_active(core: &AppCoreState) -> bool {
    core.conversation.lock().is_ok_and(|conversation| {
        conversation.active_session_id.is_some() && conversation.active_draft().mode == "chat"
    })
}

fn clear_conversation_drop(core: &AppCoreState) {
    let grant_ids = core
        .conversation_drop
        .lock()
        .map(|mut state| {
            state.paths.clear();
            std::mem::take(&mut state.grants)
                .into_iter()
                .map(|grant| grant.grant_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Ok(mut registry) = core.picker_grants.lock() {
        for grant_id in grant_ids {
            let _ = registry.take(&grant_id);
        }
    }
}

fn register_conversation_drop(
    core: &AppCoreState,
    paths: &[PathBuf],
) -> Option<Vec<PickerGrantSnapshot>> {
    clear_conversation_drop(core);
    if paths.is_empty() || paths.len() > platform::MAX_PICKER_SELECTIONS {
        return None;
    }
    let normalized = paths
        .iter()
        .map(|path| normalize_picker_path(path, PickerPurpose::AttachChatFiles))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let selections = normalized
        .iter()
        .cloned()
        .map(|path| NativePickerSelection::new(path, PickerObjectKind::File))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let request = NativePickerRequest::new(
        RequestId::new(format!("drop-request-{}", Uuid::new_v4())).ok()?,
        PickerPurpose::AttachChatFiles,
    );
    let grant_ids = (0..selections.len())
        .map(|_| PickerGrantId::new(format!("picker-grant-{}", Uuid::new_v4())))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let issued_at_ms = current_time_ms().ok()?;
    let grants = core
        .picker_grants
        .lock()
        .ok()?
        .register_batch(&request, &selections, grant_ids, issued_at_ms)
        .ok()?;
    let mut state = core.conversation_drop.lock().ok()?;
    state.paths = normalized;
    state.grants = grants.clone();
    Some(grants)
}

fn take_conversation_drop(
    core: &AppCoreState,
    paths: &[PathBuf],
) -> Option<Vec<PickerGrantSnapshot>> {
    let normalized = paths
        .iter()
        .map(|path| normalize_picker_path(path, PickerPurpose::AttachChatFiles))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let matches_enter = core
        .conversation_drop
        .lock()
        .is_ok_and(|state| state.paths == normalized);
    if !matches_enter {
        register_conversation_drop(core, paths)?;
    }
    let mut state = core.conversation_drop.lock().ok()?;
    state.paths.clear();
    Some(std::mem::take(&mut state.grants))
}

fn emit_conversation_drop<R: tauri::Runtime>(
    window: &tauri::Window<R>,
    phase: &'static str,
    grants: Vec<PickerGrantSnapshot>,
) {
    let _ = window.emit(
        CONVERSATION_FILE_DROP_EVENT,
        NativeConversationFileDropEvent { phase, grants },
    );
}

fn handle_conversation_drop_event<R: tauri::Runtime>(
    window: &tauri::Window<R>,
    event: &tauri::WindowEvent,
) {
    if window.label() != "main" {
        return;
    }
    let core = window.state::<AppCoreState>();
    if !conversation_drop_is_active(&core) {
        clear_conversation_drop(&core);
        return;
    }
    match event {
        tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Enter { paths, .. }) => {
            if let Some(grants) = register_conversation_drop(&core, paths) {
                emit_conversation_drop(window, "enter", grants);
            } else {
                emit_conversation_drop(window, "leave", Vec::new());
            }
        }
        tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) => {
            if let Some(grants) = take_conversation_drop(&core, paths) {
                emit_conversation_drop(window, "drop", grants);
            } else {
                clear_conversation_drop(&core);
                emit_conversation_drop(window, "leave", Vec::new());
            }
        }
        tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Leave) => {
            clear_conversation_drop(&core);
            emit_conversation_drop(window, "leave", Vec::new());
        }
        _ => {}
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
fn conversation_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

const MAX_CONVERSATION_ATTACHMENT_PREVIEW_BYTES: u64 = 2 * 1024 * 1024;

fn require_exact_conversation_generation(
    request: &SnapshotRequest,
    snapshot: &ConversationSnapshot,
    message: &'static str,
) -> Result<(), ProtocolError> {
    if request.expected_generation == snapshot.generation {
        Ok(())
    } else {
        Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::StaleGeneration,
            message,
            true,
        ))
    }
}

#[tauri::command]
fn conversation_attachment_preview(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationAttachmentPreviewInput,
) -> Result<ProtocolEnvelope<ConversationAttachmentPreviewSnapshot>, protocol::StructuredCoreError>
{
    validate_snapshot_request(&request)?;
    if input.stable_reference.is_empty()
        || input.stable_reference.len() > protocol::MAX_IDENTIFIER_BYTES
        || !input
            .stable_reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The attachment preview reference is invalid",
            false,
        ));
    }
    let snapshot = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    require_exact_conversation_generation(
        &request,
        &snapshot,
        "Conversation state changed before loading the attachment preview",
    )?;
    let attachment = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .active_draft()
        .attachments
        .into_iter()
        .find(|attachment| {
            attachment.attachment_id == input.attachment_id.as_str()
                && attachment.stable_reference == input.stable_reference
        })
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The draft attachment preview is unavailable",
                false,
            )
        })?;
    if attachment.byte_length > MAX_CONVERSATION_ATTACHMENT_PREVIEW_BYTES
        || !matches!(
            attachment.media_type.as_str(),
            "image/png" | "image/jpeg" | "image/gif" | "image/webp"
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The draft attachment has no bounded image preview",
            false,
        ));
    }
    let (workspace_id, workspace_root) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            active.working_root().to_path_buf(),
        )
    };
    let materializer = runtime::attachment_materializer::WorkspaceAttachmentMaterializer::bind(
        workspace_id,
        workspace_root,
    )
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let plan = materializer
        .materialize(std::slice::from_ref(&attachment))
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The immutable attachment preview failed integrity verification",
                true,
            )
        })?;
    let verified = plan
        .attachments()
        .first()
        .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
    let payload = ConversationAttachmentPreviewSnapshot {
        attachment_id: input.attachment_id,
        media_type: attachment.media_type.clone(),
        data_url: format!(
            "data:{};base64,{}",
            attachment.media_type,
            BASE64_STANDARD.encode(verified.content())
        ),
    };
    protocol::snapshot_envelope(request, snapshot.generation, payload)
}

#[tauri::command]
fn conversation_request_branch(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationBranchInput,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    require_exact_conversation_generation(
        &request,
        &before,
        "Conversation state changed before the Git branch request",
    )?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .clone();
    let cached =
        refresh_conversation_branch_control(&core, &workspace, &conversation, now_ms, true)
            .ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::NotFound,
                    "The active Project has no scoped Git Branch control",
                    false,
                )
            })?;
    if core
        .conversation_branch
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .pending
        .values()
        .any(|pending| pending.project_id == cached.project_id)
    {
        return Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "A Git branch approval is already pending for this Project",
            false,
        ));
    }
    if input.operation == ConversationBranchOperation::Switch
        && cached.menu.current_branch.as_deref() == Some(input.branch.trim())
    {
        let mut state = core
            .conversation_branch
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        state.operation_status = Some("switched".into());
        state.operation_message = Some(format!("{} is already active.", input.branch.trim()));
        drop(state);
        let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
        return protocol::conversation_snapshot(request, payload);
    }
    let (pending, facts, live) = prepare_conversation_branch_operation(
        &core,
        &workspace,
        &conversation,
        cached,
        input,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .propose_direct_action(&facts, pending.action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => {
            let mut state = core
                .conversation_branch
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            state.operation_status = Some("denied".into());
            state.operation_message = Some("Policy denied the Git branch operation.".into());
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            let prompt_id = prompt.prompt_id.clone();
            let operation_label = match pending.operation {
                ConversationBranchOperation::Switch => "switching to",
                ConversationBranchOperation::Create => "creating",
            };
            let mut state = core
                .conversation_branch
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            state.operation_status = Some("pending".into());
            state.operation_message = Some(format!(
                "Approval is required before {operation_label} {}.",
                pending.branch
            ));
            state.pending.insert(prompt_id, pending);
        }
        GatewayProposal::Authorized { token, .. } => {
            execute_conversation_branch_operation(
                &core,
                pending,
                token,
                None,
                live,
                now_ms,
                request.correlation_id.clone(),
            )?;
        }
    }
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_answer_branch_approval(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationBranchApprovalInput,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    require_exact_conversation_generation(
        &request,
        &before,
        "Conversation state changed before answering the Git branch approval",
    )?;
    let pending = core
        .conversation_branch
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .pending
        .get(&input.prompt_id)
        .cloned()
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Git branch approval is unavailable",
                false,
            )
        })?;
    let active_project_id = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .active_project_id
        .clone();
    if active_project_id.as_deref() != Some(pending.project_id.as_str()) {
        return Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The active Project changed before the Git branch approval",
            true,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let answer = match input.answer {
        ConversationBranchApprovalAnswer::Allow => ApprovalAnswer::Allow,
        ConversationBranchApprovalAnswer::Deny => ApprovalAnswer::Deny,
    };
    let response = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .answer_direct_approval(&input.prompt_id, answer, now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    match response {
        ApprovalResponse::Denied { prompt } => {
            if prompt.action != pending.action {
                return Err(workspace_state_unavailable(request.correlation_id.clone()));
            }
            let mut state = core
                .conversation_branch
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            state.pending.remove(&input.prompt_id);
            state.operation_status = Some("denied".into());
            state.operation_message = Some("The Git branch operation was cancelled.".into());
        }
        ApprovalResponse::Authorized { prompt, token } => {
            if prompt.action != pending.action {
                return Err(workspace_state_unavailable(request.correlation_id.clone()));
            }
            core.conversation_branch
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                .pending
                .remove(&input.prompt_id);
            execute_conversation_branch_operation(
                &core,
                pending.clone(),
                token,
                Some(&input.prompt_id),
                pending.live,
                now_ms,
                request.correlation_id.clone(),
            )?;
        }
    }
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_begin_pending(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let correlation_id = request.correlation_id.clone();
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, correlation_id.clone())?;
    if !workspace_snapshot.projects.iter().any(|project| {
        project.project_id == project_id.as_str()
            && project.lifecycle_state == core::database::LifecycleState::Active
    }) {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::NotFound,
            "The active Project is unavailable",
            false,
        ));
    }
    let session_id = format!("chat-{}", Uuid::new_v4());
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before creating a pending Chat",
    )?;
    let mut conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let replacement = conversation::begin_pending_chat(
        &conversation.pending,
        session_id.clone(),
        project_id.as_str(),
        conversation.active_session_id.clone(),
    )
    .map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "A pending Chat already exists",
            false,
        )
    })?;
    core.runtime
        .create_provisional(runtime_generation, session_id.clone(), now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    conversation.pending = replacement;
    conversation.active_project_id = Some(project_id.as_str().into());
    conversation.active_session_id = match &conversation.pending {
        conversation::PendingChatState::Draft(draft) => Some(draft.session_id.clone()),
        _ => None,
    };
    conversation.pending_attachments.clear();
    conversation.pending_next_attachment_reference = 1;
    conversation.advance(current_generation)?;
    drop(conversation);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_cancel_pending(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before cancelling the pending Chat",
    )?;
    let mut conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let cancellation = conversation::cancel_pending_chat(&conversation.pending).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "No cancellable pending Chat exists",
            false,
        )
    })?;
    let pending_session_id = match &conversation.pending {
        conversation::PendingChatState::Draft(draft) => draft.session_id.clone(),
        _ => unreachable!("cancellation accepted only a draft"),
    };
    core.runtime
        .discard_provisional(runtime_generation, &pending_session_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    conversation.pending = cancellation.state;
    conversation.pending_attachments.clear();
    conversation.pending_next_attachment_reference = 1;
    conversation.active_session_id = cancellation.restore_session_id;
    conversation.advance(current_generation)?;
    drop(conversation);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_update_draft(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationDraftInput,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if input.prompt.len() > protocol::MAX_TEXT_BYTES
        || input.prompt.contains('\0')
        || input.picker_grant_ids.len() > protocol::MAX_ATTACHMENTS
        || input.retained_attachment_ids.len() > protocol::MAX_ATTACHMENTS
        || !matches!(
            input.mode.as_str(),
            "chat" | "files" | "browser" | "terminal"
        )
        || input
            .reasoning_mode
            .as_deref()
            .is_some_and(|value| !matches!(value, "off" | "low" | "medium" | "high"))
        || !valid_conversation_route_selection(
            input.provider_id.as_deref(),
            input.model_id.as_deref(),
        )
        || input
            .reply_target_id
            .as_deref()
            .is_some_and(|value| !valid_conversation_identifier(value, false))
        || (input.mode != "chat"
            && (!input.picker_grant_ids.is_empty() || !input.retained_attachment_ids.is_empty()))
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The composer draft is invalid",
            false,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before saving the composer draft",
    )?;
    let retained_attachment_ids = input
        .retained_attachment_ids
        .iter()
        .map(|attachment_id| attachment_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    {
        let conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let draft = conversation.active_draft();
        if retained_attachment_ids.len() != input.retained_attachment_ids.len()
            || retained_attachment_ids.iter().any(|attachment_id| {
                !draft
                    .attachments
                    .iter()
                    .any(|attachment| attachment.attachment_id == *attachment_id)
            })
        {
            return Err(platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The composer attachment selection is stale",
                true,
            ));
        }
    }
    let (workspace_id, workspace_root, database) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            active.working_root().to_path_buf(),
            Arc::clone(active.database_actor()),
        )
    };
    let imported = import_attachment_picker_grants(
        &core,
        &input.picker_grant_ids,
        now_ms,
        &workspace_root,
        request.correlation_id.clone(),
    )?;
    let persisted_generation = {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        if matches!(
            &conversation.pending,
            conversation::PendingChatState::PromotionRequested(_)
        ) {
            return Err(platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The pending Chat is already being promoted",
                true,
            ));
        }
        if matches!(
            &conversation.pending,
            conversation::PendingChatState::Inactive
        ) {
            let session_id = conversation.active_session_id.clone().ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "No active Chat can own this composer draft",
                    false,
                )
            })?;
            if !workspace_snapshot.chats.iter().any(|chat| {
                chat.chat_id == session_id
                    && chat.lifecycle_state == core::database::LifecycleState::Active
            }) {
                return Err(platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The draft Chat is unavailable",
                    true,
                ));
            }
            let draft = conversation.drafts.entry(session_id).or_default();
            draft.attachments.retain(|attachment| {
                retained_attachment_ids.contains(attachment.attachment_id.as_str())
            });
            reconcile_attachment_references(
                &mut draft.attachments,
                imported,
                &mut draft.next_attachment_reference,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The composer attachment references are invalid",
                    true,
                )
            })?;
            draft.prompt = input.prompt;
            draft.provider_id = input.provider_id;
            draft.model_id = input.model_id;
            draft.reasoning_mode = input.reasoning_mode;
            draft.mode = input.mode;
            draft.reply_target_id = input.reply_target_id;
            conversation
                .persist(&database, &workspace_id, now_ms)
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        } else {
            conversation.pending_attachments.retain(|attachment| {
                retained_attachment_ids.contains(attachment.attachment_id.as_str())
            });
            let mut next_reference = conversation.pending_next_attachment_reference;
            reconcile_attachment_references(
                &mut conversation.pending_attachments,
                imported,
                &mut next_reference,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The pending attachment references are invalid",
                    true,
                )
            })?;
            conversation.pending_next_attachment_reference = next_reference;
            let attachment_ids = conversation
                .pending_attachments
                .iter()
                .map(|attachment| attachment.attachment_id.clone())
                .collect();
            conversation.pending = conversation::update_pending_chat_draft(
                &conversation.pending,
                input.prompt,
                attachment_ids,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "No editable pending Chat exists",
                    false,
                )
            })?;
            current_generation
        }
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(current_generation.max(persisted_generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_submit(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationSubmitInput,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if input
        .prompt
        .as_ref()
        .is_some_and(|prompt| prompt.len() > protocol::MAX_TEXT_BYTES || prompt.contains('\0'))
        || input.picker_grant_ids.len() > protocol::MAX_ATTACHMENTS
        || input.retained_attachment_ids.len() > protocol::MAX_ATTACHMENTS
        || !matches!(
            input.resume_mode.as_str(),
            "chat" | "files" | "browser" | "terminal"
        )
        || input
            .reasoning_mode
            .as_deref()
            .is_some_and(|value| !matches!(value, "off" | "low" | "medium" | "high"))
        || !valid_conversation_route_selection(
            input.provider_id.as_deref(),
            input.model_id.as_deref(),
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The Chat submission exceeds its bound",
            false,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before submission",
    )?;
    let (current_generation, reply_target) = {
        let conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let retained_attachment_ids = input
            .retained_attachment_ids
            .iter()
            .map(|attachment_id| attachment_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let active_draft = conversation.active_draft();
        if retained_attachment_ids.len() != input.retained_attachment_ids.len()
            || retained_attachment_ids.iter().any(|attachment_id| {
                !active_draft
                    .attachments
                    .iter()
                    .any(|attachment| attachment.attachment_id == *attachment_id)
            })
        {
            return Err(platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The pending attachment selection is stale",
                true,
            ));
        }
        let reply_target = matches!(
            &conversation.pending,
            conversation::PendingChatState::Inactive
        )
        .then(|| active_draft.reply_target_id.clone())
        .flatten()
        .zip(conversation.active_session_id.clone());
        (current_generation, reply_target)
    };
    let (workspace_id, workspace_root) = core
        .runtime
        .sessions
        .bound_workspace_binding()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
    let conversation_database = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        Arc::clone(
            active
                .as_ref()
                .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
                .database_actor(),
        )
    };
    let imported = import_attachment_picker_grants(
        &core,
        &input.picker_grant_ids,
        now_ms,
        &workspace_root,
        request.correlation_id.clone(),
    )?;
    let reply_context = reply_target
        .map(|(target_id, session_id)| {
            message_reply_context(
                &core.runtime,
                &session_id,
                &target_id,
                request.correlation_id.clone(),
            )
        })
        .transpose()?;

    let (dispatch_intent, promotion_floor) = {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let retained_attachment_ids = input
            .retained_attachment_ids
            .iter()
            .map(|attachment_id| attachment_id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let intent = if matches!(
            &conversation.pending,
            conversation::PendingChatState::Inactive
        ) {
            let project_id = conversation.active_project_id.clone().ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "No active Project can receive this Chat turn",
                    false,
                )
            })?;
            let session_id = conversation.active_session_id.clone().ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "No active Chat can receive this turn",
                    false,
                )
            })?;
            if !workspace_snapshot.chats.iter().any(|chat| {
                chat.chat_id == session_id
                    && chat.project_id == project_id
                    && chat.lifecycle_state == core::database::LifecycleState::Active
            }) {
                return Err(platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The active Chat binding is stale",
                    true,
                ));
            }
            let draft = conversation
                .drafts
                .entry(session_id.clone())
                .or_insert_with(|| DurableConversationDraft {
                    mode: "chat".into(),
                    ..DurableConversationDraft::default()
                });
            draft.attachments.retain(|attachment| {
                retained_attachment_ids.contains(attachment.attachment_id.as_str())
            });
            reconcile_attachment_references(
                &mut draft.attachments,
                imported,
                &mut draft.next_attachment_reference,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Chat attachment references are invalid",
                    true,
                )
            })?;
            draft.prompt = input.prompt.clone().unwrap_or_default();
            draft.provider_id = input.provider_id.clone();
            draft.model_id = input.model_id.clone();
            draft.reasoning_mode = input.reasoning_mode.clone();
            if draft.prompt.trim().is_empty() && draft.attachments.is_empty() {
                return Err(platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "A Chat submission requires text or an attachment",
                    false,
                ));
            }
            ConversationSubmissionDispatch::Turn(ConversationTurnDispatchIntent {
                workspace_id: workspace_id.clone(),
                project_id,
                session_id,
                prompt: (!draft.prompt.trim().is_empty()).then(|| draft.prompt.clone()),
                attachments: draft.attachments.clone(),
                provider_id: draft.provider_id.clone(),
                model_id: draft.model_id.clone(),
                reasoning_mode: draft.reasoning_mode.clone(),
                reply_context,
                submitted_at_ms: now_ms,
            })
        } else {
            conversation.pending_attachments.retain(|attachment| {
                retained_attachment_ids.contains(attachment.attachment_id.as_str())
            });
            let mut next_reference = conversation.pending_next_attachment_reference;
            reconcile_attachment_references(
                &mut conversation.pending_attachments,
                imported,
                &mut next_reference,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The pending attachment references are invalid",
                    true,
                )
            })?;
            conversation.pending_next_attachment_reference = next_reference;
            let attachment_ids = conversation
                .pending_attachments
                .iter()
                .map(|attachment| attachment.attachment_id.clone())
                .collect();
            conversation.pending = conversation::update_pending_chat_draft(
                &conversation.pending,
                input.prompt.clone().unwrap_or_default(),
                attachment_ids,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "No editable pending Chat exists",
                    false,
                )
            })?;
            conversation.pending = conversation::request_pending_chat_promotion(
                &conversation.pending,
                conversation
                    .pending_attachments
                    .iter()
                    .map(|attachment| attachment.display_name.as_str()),
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "A first Chat submission requires text or an attachment",
                    false,
                )
            })?;
            let conversation::PendingChatState::PromotionRequested(promotion) =
                &conversation.pending
            else {
                unreachable!("promotion request returned the promotion state")
            };
            ConversationSubmissionDispatch::First(ConversationFirstDispatchIntent {
                workspace_id: workspace_id.clone(),
                project_id: promotion.project_id.clone(),
                session_id: promotion.session_id.clone(),
                prompt: promotion.prompt.clone(),
                attachments: conversation.pending_attachments.clone(),
                provider_id: input.provider_id.clone(),
                model_id: input.model_id.clone(),
                reasoning_mode: input.reasoning_mode.clone(),
                submitted_at_ms: now_ms,
            })
        };
        conversation.advance(current_generation)?;
        (intent, current_generation)
    };

    let first_submission = matches!(&dispatch_intent, ConversationSubmissionDispatch::First(_));
    let dispatch_generation = match dispatch_intent {
        ConversationSubmissionDispatch::First(intent) => core
            .runtime
            .dispatch_conversation_first(intent)
            .map(|dispatch| match dispatch {
                CoordinatedFirstDispatch::Accepted {
                    coordinator_generation,
                    ..
                }
                | CoordinatedFirstDispatch::Rejected {
                    coordinator_generation,
                    ..
                } => coordinator_generation,
            }),
        ConversationSubmissionDispatch::Turn(intent) => core
            .runtime
            .dispatch_conversation_turn(intent)
            .map(|dispatch| match dispatch {
                CoordinatedTurnDispatch::Accepted {
                    coordinator_generation,
                    ..
                }
                | CoordinatedTurnDispatch::Rejected {
                    coordinator_generation,
                    ..
                } => coordinator_generation,
            }),
    };
    let dispatch_generation = match dispatch_generation {
        Ok(generation) => generation,
        Err(_) => {
            let mut conversation = core
                .conversation
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            if first_submission {
                conversation.pending =
                    conversation::fail_pending_chat_promotion(&conversation.pending)
                        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            } else {
                conversation
                    .persist(&conversation_database, &workspace_id, now_ms)
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            }
            conversation.advance(promotion_floor)?;
            return Err(platform_boundary_error(
                request.correlation_id,
                ProtocolErrorCode::Conflict,
                "The selected runtime route cannot accept this Chat submission",
                true,
            ));
        }
    };
    {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let resume_draft = DurableConversationDraft {
            prompt: String::new(),
            attachments: Vec::new(),
            next_attachment_reference: 1,
            provider_id: input.provider_id.clone(),
            model_id: input.model_id.clone(),
            reasoning_mode: input.reasoning_mode.clone(),
            mode: input.resume_mode.clone(),
            reply_target_id: None,
        };
        if first_submission {
            let completion = conversation::complete_pending_chat(&conversation.pending)
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            conversation.pending = completion.state;
            conversation.active_project_id = Some(completion.active_project_id);
            conversation.active_session_id = Some(completion.active_session_id.clone());
            conversation
                .drafts
                .insert(completion.active_session_id, resume_draft);
        } else if let Some(session_id) = conversation.active_session_id.clone() {
            conversation.drafts.insert(session_id, resume_draft);
        }
        conversation.pending_attachments.clear();
        conversation.pending_next_attachment_reference = 1;
        let persisted_generation = conversation
            .persist(&conversation_database, &workspace_id, now_ms)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation.advance(
            dispatch_generation
                .max(workspace_snapshot.generation)
                .max(persisted_generation),
        )?;
    }
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

/// Applies the same bounded route-identity rules before a draft is persisted
/// and again before it can become dispatch intent.
fn valid_conversation_route_selection(provider_id: Option<&str>, model_id: Option<&str>) -> bool {
    if provider_id.is_some() != model_id.is_some() {
        return false;
    }
    provider_id.is_none_or(|value| valid_conversation_identifier(value, false))
        && model_id.is_none_or(|value| valid_conversation_identifier(value, true))
}

fn valid_conversation_identifier(value: &str, allow_route_separator: bool) -> bool {
    !value.is_empty()
        && value.len() <= protocol::MAX_IDENTIFIER_BYTES
        && value.as_bytes().iter().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
                || (allow_route_separator && *byte == b'/')
        })
}

#[cfg(test)]
mod conversation_input_validation_tests {
    use super::{
        conversation_authority_generation, reconcile_attachment_references,
        resolve_conversation_provider_model, valid_conversation_identifier,
        valid_conversation_route_selection,
    };
    use crate::runtime::capability::{
        CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityLayer, ModelLifecycle,
        RouteIdentity,
    };
    use crate::runtime::coordinator::RuntimeCoordinatorSnapshot;
    use crate::runtime::provider::{
        ModelRoute, PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION, PROVIDER_SCHEMA_VERSION,
        ProviderEndpoint, ProviderKind, ProviderModelDeclaration, ProviderProfile, ProviderRecord,
        ProviderSnapshot, ProviderTestStatus, RouteAvailability,
    };
    use crate::runtime::session::AttachmentSnapshot;
    use crate::runtime::supervisor::SupervisorSnapshot;
    use crate::security::credentials::CredentialReference;
    use std::collections::BTreeMap;

    #[test]
    fn conversation_authority_generation_covers_every_published_domain() {
        assert_eq!(conversation_authority_generation(9, 1, 2, 3, 4, 5), 9);
        assert_eq!(conversation_authority_generation(1, 9, 2, 3, 4, 5), 9);
        assert_eq!(conversation_authority_generation(1, 2, 9, 3, 4, 5), 9);
        assert_eq!(conversation_authority_generation(1, 2, 3, 9, 4, 5), 9);
        assert_eq!(conversation_authority_generation(1, 2, 3, 4, 9, 5), 9);
        assert_eq!(conversation_authority_generation(1, 2, 3, 4, 5, 9), 9);
    }

    #[test]
    fn durable_attachment_references_survive_removal_without_reuse() {
        let attachment = |id: &str, original_reference: u32| AttachmentSnapshot {
            attachment_id: id.into(),
            stable_reference: format!("workspace-blob:sha256:{}:v1", id.repeat(64)),
            display_name: format!("{id}.txt"),
            media_type: "text/plain".into(),
            byte_length: 1,
            content_sha256: format!("sha256:{}", id.repeat(64)),
            snapshot_version: 1,
            original_reference,
        };
        let mut retained = vec![attachment("a", 1)];
        let mut next_reference = 3;

        reconcile_attachment_references(
            &mut retained,
            vec![attachment("c", 0)],
            &mut next_reference,
        )
        .expect("reconcile references");

        assert_eq!(
            retained
                .iter()
                .map(|item| item.original_reference)
                .collect::<Vec<_>>(),
            [1, 3]
        );
        assert_eq!(next_reference, 4);
    }

    #[test]
    fn route_selection_requires_a_bounded_provider_model_pair() {
        assert!(valid_conversation_route_selection(None, None));
        assert!(valid_conversation_route_selection(
            Some("openai"),
            Some("openai/gpt-5")
        ));
        assert!(!valid_conversation_route_selection(Some("openai"), None));
        assert!(!valid_conversation_route_selection(
            None,
            Some("openai/gpt-5")
        ));
        assert!(!valid_conversation_route_selection(
            Some("open ai"),
            Some("openai/gpt-5")
        ));
    }

    #[test]
    fn reply_identity_does_not_accept_route_separators_or_control_bytes() {
        assert!(valid_conversation_identifier("turn:accepted-1", false));
        assert!(!valid_conversation_identifier("turn/escaped", false));
        assert!(!valid_conversation_identifier("turn\0escaped", false));
        assert!(valid_conversation_identifier("openai/gpt-5", true));
    }

    #[test]
    fn explicit_production_ready_model_does_not_have_to_be_provider_default() {
        let model = |model_id: &str| ModelRoute {
            model_id: model_id.into(),
            display_name: model_id.into(),
            recommendation_rank: 0,
            availability: RouteAvailability::Available,
            checked_at_ms: 10,
            capabilities: CapabilityDescriptor {
                schema_version: CAPABILITY_SCHEMA_VERSION,
                layer: CapabilityLayer::Declared,
                route: RouteIdentity {
                    provider_id: "provider-one".into(),
                    endpoint_id: "endpoint-one".into(),
                    provider_model_id: model_id.into(),
                    model_revision: "revision-one".into(),
                    adapter_kind: "opencode".into(),
                    adapter_version: "1.0.0".into(),
                    runtime_kind: "opencode".into(),
                    native_runtime_version: "1.0.0".into(),
                    session_configuration_sha256: format!("sha256:{}", "1".repeat(64)),
                },
                lifecycle: ModelLifecycle::Active,
                features: BTreeMap::new(),
                numeric_limits: BTreeMap::new(),
                raw_evidence_sha256: format!("sha256:{}", "2".repeat(64)),
            },
            provider_declaration: Some(ProviderModelDeclaration {
                schema_version: PROVIDER_MODEL_DECLARATION_SCHEMA_VERSION,
                provider_model_id: model_id.into(),
                model_revision: "revision-one".into(),
                lifecycle: ModelLifecycle::Active,
                features: BTreeMap::new(),
                numeric_limits: BTreeMap::new(),
                raw_catalog_sha256: format!("sha256:{}", "3".repeat(64)),
                declared_at_ms: 10,
                expires_at_ms: 100,
            }),
        };
        let snapshot = RuntimeCoordinatorSnapshot {
            generation: 1,
            providers: ProviderSnapshot {
                generation: 1,
                providers: vec![ProviderRecord {
                    profile: ProviderProfile {
                        schema_version: PROVIDER_SCHEMA_VERSION,
                        provider_id: "provider-one".into(),
                        kind: ProviderKind::OpenRouter,
                        display_name: "Provider One".into(),
                        endpoint: ProviderEndpoint {
                            endpoint_id: "endpoint-one".into(),
                            base_url: "https://example.com".into(),
                            api_kind: "openai-compatible".into(),
                        },
                        credential_reference: serde_json::from_str::<CredentialReference>(
                            "\"credential-one\"",
                        )
                        .expect("credential reference"),
                        enabled: true,
                    },
                    test_status: ProviderTestStatus::Untested,
                    connection_evidence: None,
                    models: BTreeMap::from([
                        ("model-a".into(), model("model-a")),
                        ("model-b".into(), model("model-b")),
                    ]),
                    selected_model_id: Some("model-a".into()),
                    generation: 1,
                }],
            },
            runtimes: SupervisorSnapshot {
                state_generation: 0,
                records: Vec::new(),
                events: Vec::new(),
                trace_events_dropped: 0,
            },
            onboarding_ready: false,
        };

        assert_eq!(
            resolve_conversation_provider_model(
                &snapshot,
                Some("provider-one"),
                Some("model-b"),
                20,
            )
            .expect("explicit model route"),
            ("provider-one".into(), "model-b".into()),
        );
        assert_eq!(
            resolve_conversation_provider_model(&snapshot, Some("provider-one"), None, 20)
                .expect("default model route"),
            ("provider-one".into(), "model-a".into()),
        );
    }
}

#[tauri::command]
fn conversation_cancel_attempt(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    attempt_id: AttemptId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before cancellation",
    )?;
    let session_id = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .active_session_id
        .clone()
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "No active Chat attempt can be cancelled",
                false,
            )
        })?;
    let session = core.runtime.session(&session_id).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::NotFound,
            "The active Chat attempt is unavailable",
            false,
        )
    })?;
    if session.active_attempt_id.as_deref() != Some(attempt_id.as_str()) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Only the active Chat attempt can be cancelled",
            false,
        ));
    }
    let attempt = session.attempt(attempt_id.as_str()).ok_or_else(|| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::NotFound,
            "The active Chat attempt is unavailable",
            false,
        )
    })?;
    let identity = DispatchIdentity {
        workspace_id: attempt.context.workspace_id.clone(),
        environment_id: attempt.context.environment.environment_id.clone(),
        session_id: session.session_id.clone(),
        turn_id: attempt.turn_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        correlation_id: attempt.correlation_id.clone(),
        runtime_id: attempt.context.runtime_id.clone(),
        runtime_kind: match attempt.context.runtime_kind {
            SessionRuntimeKind::OpenCode => runtime::supervisor::RuntimeKind::OpenCode,
            SessionRuntimeKind::Pi => runtime::supervisor::RuntimeKind::Pi,
        },
        adapter_version: attempt.context.adapter.adapter_version.clone(),
        native_version: attempt.context.adapter.native_version.clone(),
        process_generation: attempt.process_generation,
    };
    let generation = core
        .runtime
        .cancel_dispatch(runtime_generation, &identity, now_ms)
        .map(|outcome| match outcome {
            CoordinatedCancellation::Cancelled {
                coordinator_generation,
                ..
            }
            | CoordinatedCancellation::Interrupted {
                coordinator_generation,
                ..
            } => coordinator_generation,
        })
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The active runtime could not cancel this Chat attempt",
                true,
            )
        })?;
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(current_generation.max(generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_retry_attempt(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ConversationRetryInput,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    if input
        .provider_id
        .as_ref()
        .is_some_and(|value| value.is_empty() || value.len() > protocol::MAX_IDENTIFIER_BYTES)
        || input
            .model_id
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.len() > protocol::MAX_IDENTIFIER_BYTES)
        || input
            .reasoning_mode
            .as_deref()
            .is_some_and(|value| !matches!(value, "off" | "low" | "medium" | "high"))
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The Chat retry controls are invalid",
            false,
        ));
    }
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before retry",
    )?;
    let session_id = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .active_session_id
        .clone()
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "No active Chat attempt can be retried",
                false,
            )
        })?;
    let generation = core
        .runtime
        .retry_conversation_attempt(
            &session_id,
            input.parent_attempt_id.as_str(),
            input.provider_id,
            input.model_id,
            input.reasoning_mode,
            now_ms,
        )
        .map(|dispatch| match dispatch {
            CoordinatedRetryDispatch::Accepted {
                coordinator_generation,
                ..
            }
            | CoordinatedRetryDispatch::Rejected {
                coordinator_generation,
                ..
            } => coordinator_generation,
        })
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The selected runtime route cannot retry this Chat attempt",
                true,
            )
        })?;
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(current_generation.max(generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_activate_session(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    session_id: SessionId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let chat = workspace_snapshot
        .chats
        .iter()
        .find(|chat| {
            chat.chat_id == session_id.as_str()
                && chat.lifecycle_state == core::database::LifecycleState::Active
        })
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The selected Chat is unavailable",
                false,
            )
        })?;
    let (workspace_id, database) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            Arc::clone(active.database_actor()),
        )
    };
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before Chat activation",
    )?;
    let mut conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if !matches!(
        conversation.pending,
        conversation::PendingChatState::Inactive
    ) {
        return Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "Cancel or submit the pending Chat before selecting another Chat",
            false,
        ));
    }
    conversation.active_project_id = Some(chat.project_id.clone());
    conversation.active_session_id = Some(chat.chat_id.clone());
    let persisted_generation = conversation
        .persist(&database, &workspace_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    conversation.advance(current_generation.max(persisted_generation))?;
    drop(conversation);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_activate_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    if !workspace_snapshot.projects.iter().any(|project| {
        project.project_id == project_id.as_str()
            && project.lifecycle_state == core::database::LifecycleState::Active
    }) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::NotFound,
            "The selected Project is unavailable",
            false,
        ));
    }
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before Project activation",
    )?;
    let (workspace_id, database) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            Arc::clone(active.database_actor()),
        )
    };
    let mut conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if !matches!(
        conversation.pending,
        conversation::PendingChatState::Inactive
    ) {
        return Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "Cancel or submit the pending Chat before selecting another Project",
            false,
        ));
    }
    conversation.active_project_id = Some(project_id.as_str().into());
    conversation.active_session_id = workspace_snapshot
        .chats
        .iter()
        .find(|chat| {
            chat.project_id == project_id.as_str()
                && chat.lifecycle_state == core::database::LifecycleState::Active
        })
        .map(|chat| chat.chat_id.clone());
    let persisted_generation = conversation
        .persist(&database, &workspace_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    conversation.advance(current_generation.max(persisted_generation))?;
    drop(conversation);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_add_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    picker_grant_id: PickerGrantId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before adding the Project",
    )?;
    let path = take_folder_picker_grant(
        &core,
        &picker_grant_id,
        PickerPurpose::OpenProjectFolder,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let display_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| invalid_picker_selection(request.correlation_id.clone()))?
        .to_owned();
    let generation = {
        let mut active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_mut()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .add_project(&path, &display_name, true)
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The selected Project folder could not be added",
                    false,
                )
            })?
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(generation.max(current_generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_relocate_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
    picker_grant_id: PickerGrantId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    if !workspace_snapshot.projects.iter().any(|project| {
        project.project_id == project_id.as_str()
            && project.lifecycle_state == core::database::LifecycleState::Active
    }) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::NotFound,
            "The Project to relocate is unavailable",
            false,
        ));
    }
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before relocating the Project",
    )?;
    let path = take_folder_picker_grant(
        &core,
        &picker_grant_id,
        PickerPurpose::RelocateProjectFolder,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let project_uuid = Uuid::parse_str(project_id.as_str()).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidIdentifier,
            "The Project identifier is invalid",
            false,
        )
    })?;
    let generation = {
        let mut active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_mut()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .relocate_project(project_uuid, &path, true)
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The selected folder could not relocate this Project",
                    false,
                )
            })?
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(generation.max(current_generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_rename_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
    display_name: String,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if display_name.trim().is_empty()
        || display_name.contains('\0')
        || display_name.len() > core::database::MAX_PROJECT_DISPLAY_NAME_BYTES
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The Project name is invalid",
            false,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before renaming the Project",
    )?;
    let generation = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .database_actor()
            .rename_project(project_id.as_str(), display_name.trim())
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Project could not be renamed",
                    true,
                )
            })?
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(generation.max(current_generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_copy_project_path(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    conversation_project_native_action(core, request, project_id, ProjectNativeAction::CopyPath)
}

#[tauri::command]
fn conversation_reveal_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    conversation_project_native_action(core, request, project_id, ProjectNativeAction::Reveal)
}

#[derive(Clone, Copy)]
enum ProjectNativeAction {
    CopyPath,
    Reveal,
}

fn conversation_project_native_action(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
    action: ProjectNativeAction,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before the Project action",
    )?;
    let project = workspace_snapshot
        .projects
        .iter()
        .find(|project| {
            project.project_id == project_id.as_str()
                && project.lifecycle_state == core::database::LifecycleState::Active
        })
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Project is unavailable",
                false,
            )
        })?;
    #[cfg(target_os = "macos")]
    match action {
        ProjectNativeAction::CopyPath => {
            let mut child = Command::new("/usr/bin/pbcopy")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            child
                .stdin
                .as_mut()
                .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
                .write_all(project.current_path.as_bytes())
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            if !child
                .wait()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                .success()
            {
                return Err(workspace_state_unavailable(request.correlation_id));
            }
        }
        ProjectNativeAction::Reveal => {
            Command::new("/usr/bin/open")
                .arg("-R")
                .arg(&project.current_path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (action, project);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_reorder_projects(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    ordered_project_ids: Vec<ProjectId>,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let active_ids = workspace_snapshot
        .projects
        .iter()
        .filter(|project| project.lifecycle_state == core::database::LifecycleState::Active)
        .map(|project| project.project_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let requested_ids = ordered_project_ids
        .iter()
        .map(ProjectId::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    if requested_ids != active_ids || requested_ids.len() != ordered_project_ids.len() {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Project order must contain every active Project exactly once",
            true,
        ));
    }
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before reordering Projects",
    )?;
    let ordered = ordered_project_ids
        .iter()
        .map(|project_id| {
            Uuid::parse_str(project_id.as_str()).map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::InvalidIdentifier,
                    "A Project identifier is invalid",
                    false,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let generation = {
        let mut active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_mut()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .reorder_projects(&ordered)
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "Project order could not be persisted",
                    true,
                )
            })?
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(generation.max(current_generation))?;
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_inactivate_project(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let (workspace_id, conversation_database) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            Arc::clone(active.database_actor()),
        )
    };
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before removing the Project",
    )?;
    let (plan, pending_session_id) = {
        let conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let pending_project_id = match &conversation.pending {
            conversation::PendingChatState::Draft(draft) => Some(draft.project_id.clone()),
            conversation::PendingChatState::PromotionRequested(promotion) => {
                Some(promotion.project_id.clone())
            }
            conversation::PendingChatState::Inactive => None,
        };
        let pending_session_id = match &conversation.pending {
            conversation::PendingChatState::Draft(draft) => Some(draft.session_id.clone()),
            conversation::PendingChatState::PromotionRequested(promotion) => {
                Some(promotion.session_id.clone())
            }
            conversation::PendingChatState::Inactive => None,
        };
        let plan = conversation::plan_inactivation(
            &navigation_projects(&workspace_snapshot),
            &navigation_sessions(&workspace_snapshot),
            &conversation::ActiveNavigation {
                active_project_id: conversation.active_project_id.clone(),
                active_session_id: conversation.active_session_id.clone(),
                pending_project_id,
            },
            conversation::InactivationTarget::Project(project_id.as_str().into()),
        )
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The Project cannot be removed from the current navigation state",
                true,
            )
        })?;
        (plan, pending_session_id)
    };
    if plan.cancel_pending_chat {
        core.runtime
            .discard_provisional(
                runtime_generation,
                pending_session_id
                    .as_deref()
                    .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?,
                now_ms,
            )
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    }
    let project_uuid = Uuid::parse_str(project_id.as_str()).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidIdentifier,
            "The Project identifier is invalid",
            false,
        )
    })?;
    let generation = {
        let mut active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_mut()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .inactivate_project(
                project_uuid,
                i64::try_from(now_ms)
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Project could not be removed",
                    true,
                )
            })?
    };
    {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation.active_project_id = plan.next_navigation.active_project_id;
        conversation.active_session_id = plan.next_navigation.active_session_id;
        if plan.cancel_pending_chat {
            conversation.pending = conversation::PendingChatState::Inactive;
            conversation.pending_attachments.clear();
            conversation.pending_next_attachment_reference = 1;
        }
        let persisted_generation = conversation
            .persist(&conversation_database, &workspace_id, now_ms)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation.advance(generation.max(current_generation).max(persisted_generation))?;
    }
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_inactivate_session(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    session_id: SessionId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let workspace_snapshot = active_workspace_snapshot(&core, request.correlation_id.clone())?;
    let (workspace_id, conversation_database) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            active.manifest().workspace_id.to_string(),
            Arc::clone(active.database_actor()),
        )
    };
    let runtime_generation = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .generation;
    let current_generation = require_conversation_generation(
        &core,
        &request,
        workspace_snapshot.generation,
        runtime_generation,
        "Conversation state changed before removing the Chat",
    )?;
    let plan = {
        let conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation::plan_inactivation(
            &navigation_projects(&workspace_snapshot),
            &navigation_sessions(&workspace_snapshot),
            &conversation::ActiveNavigation {
                active_project_id: conversation.active_project_id.clone(),
                active_session_id: conversation.active_session_id.clone(),
                pending_project_id: None,
            },
            conversation::InactivationTarget::Session(session_id.as_str().into()),
        )
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The Chat cannot be removed from the current navigation state",
                true,
            )
        })?
    };
    let generation = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(request.correlation_id.clone()))?
            .database_actor()
            .inactivate(
                core::database::InactiveEntity::Chat {
                    chat_id: session_id.as_str().into(),
                },
                i64::try_from(now_ms)
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?,
            )
            .map_err(|_| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Chat could not be removed",
                    true,
                )
            })?
    };
    {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation.active_project_id = plan.next_navigation.active_project_id;
        conversation.active_session_id = plan.next_navigation.active_session_id;
        conversation.drafts.remove(session_id.as_str());
        let persisted_generation = conversation
            .persist(&conversation_database, &workspace_id, now_ms)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        conversation.advance(generation.max(current_generation).max(persisted_generation))?;
    }
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

fn require_conversation_generation(
    core: &AppCoreState,
    request: &SnapshotRequest,
    workspace_generation: u64,
    runtime_generation: u64,
    message: &'static str,
) -> Result<u64, ProtocolError> {
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (runtime, capability_generation) = core
        .runtime
        .snapshot_with_capability_generation(now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let current = conversation_authority_generation(
        conversation.generation,
        workspace_generation,
        runtime_generation.max(runtime.generation),
        runtime.providers.generation,
        runtime.runtimes.state_generation,
        capability_generation,
    );
    if request.expected_generation.0 == current {
        Ok(current)
    } else {
        Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::StaleGeneration,
            message,
            true,
        ))
    }
}

fn conversation_authority_generation(
    conversation_generation: u64,
    workspace_generation: u64,
    coordinator_generation: u64,
    provider_generation: u64,
    supervisor_generation: u64,
    capability_generation: u64,
) -> u64 {
    conversation_generation
        .max(workspace_generation)
        .max(coordinator_generation)
        .max(provider_generation)
        .max(supervisor_generation)
        .max(capability_generation)
}

fn take_folder_picker_grant(
    core: &AppCoreState,
    picker_grant_id: &PickerGrantId,
    purpose: PickerPurpose,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<PathBuf, ProtocolError> {
    let grant = core
        .picker_grants
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .take(picker_grant_id)
        .ok_or_else(|| invalid_picker_selection(correlation_id.clone()))?;
    if grant.purpose() != purpose
        || grant.object_kind() != PickerObjectKind::Folder
        || grant.issued_at_ms() > now_ms
        || now_ms.saturating_sub(grant.issued_at_ms()) > 10 * 60 * 1_000
    {
        return Err(invalid_picker_selection(correlation_id));
    }
    Ok(grant.path().to_path_buf())
}

fn import_attachment_picker_grants(
    core: &AppCoreState,
    picker_grant_ids: &[PickerGrantId],
    now_ms: u64,
    workspace_root: &Path,
    correlation_id: protocol::CorrelationId,
) -> Result<Vec<AttachmentSnapshot>, ProtocolError> {
    if picker_grant_ids.is_empty() {
        return Ok(Vec::new());
    }
    let grants = core
        .picker_grants
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .take_batch(picker_grant_ids)
        .map_err(|_| invalid_picker_selection(correlation_id.clone()))?;
    let imports = grants
        .into_iter()
        .map(|grant| {
            if grant.purpose() != PickerPurpose::AttachChatFiles
                || grant.object_kind() != PickerObjectKind::File
                || grant.issued_at_ms() > now_ms
                || now_ms.saturating_sub(grant.issued_at_ms()) > 10 * 60 * 1_000
            {
                return Err(invalid_picker_selection(correlation_id.clone()));
            }
            let display_name = grant
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.trim().is_empty())
                .ok_or_else(|| invalid_picker_selection(correlation_id.clone()))?
                .to_owned();
            Ok(runtime::attachment_materializer::NativeAttachmentImport {
                attachment_id: format!("attachment-{}", Uuid::new_v4()),
                source_path: grant.path().to_path_buf(),
                display_name,
            })
        })
        .collect::<Result<Vec<_>, ProtocolError>>()?;
    runtime::attachment_materializer::import_native_attachments(workspace_root, &imports).map_err(
        |_| {
            platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::InvalidPayload,
                "A selected attachment could not be imported safely",
                false,
            )
        },
    )
}

fn navigation_projects(
    snapshot: &core::database::WorkspaceSnapshot,
) -> Vec<conversation::NavigationProject> {
    snapshot
        .projects
        .iter()
        .map(|project| conversation::NavigationProject {
            project_id: project.project_id.clone(),
            display_name: project.display_name.clone(),
            position: project.position,
            is_active: project.lifecycle_state == core::database::LifecycleState::Active,
        })
        .collect()
}

fn navigation_sessions(
    snapshot: &core::database::WorkspaceSnapshot,
) -> Vec<conversation::NavigationSession> {
    snapshot
        .chats
        .iter()
        .enumerate()
        .map(|(position, chat)| conversation::NavigationSession {
            session_id: chat.chat_id.clone(),
            project_id: chat.project_id.clone(),
            title: chat.title.clone(),
            position: i64::try_from(position).unwrap_or(i64::MAX),
            is_active: chat.lifecycle_state == core::database::LifecycleState::Active,
        })
        .collect()
}

fn active_workspace_snapshot(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<core::database::WorkspaceSnapshot, ProtocolError> {
    let workspace = core
        .active_workspace
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let workspace = workspace
        .as_ref()
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    workspace
        .snapshot(
            core::database::SnapshotQuery::new(core::database::MAX_READ_RECORDS)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id))
}

const CONVERSATION_BRANCH_REFRESH_MS: u64 = 2_000;
const TRUSTED_GIT_PROGRAM: &str = "/usr/bin/git";

fn refresh_conversation_branch_control(
    core: &AppCoreState,
    workspace: &core::database::WorkspaceSnapshot,
    conversation: &ConversationApplicationState,
    now_ms: u64,
    force: bool,
) -> Option<CachedConversationBranchControl> {
    let project_id = conversation.active_project_id.as_deref()?;
    let project = workspace.projects.iter().find(|project| {
        project.project_id == project_id
            && project.lifecycle_state == core::database::LifecycleState::Active
            && project.path_state != core::database::ProjectPathState::Missing
    })?;
    if !force {
        let cached = core
            .conversation_branch
            .lock()
            .ok()?
            .cached
            .as_ref()
            .filter(|cached| {
                cached.project_id == project_id
                    && now_ms.saturating_sub(cached.checked_at_ms) < CONVERSATION_BRANCH_REFRESH_MS
            })
            .cloned();
        if cached.is_some() {
            return cached;
        }
    }

    let root = TrustedProjectRoot::open(Path::new(&project.current_path)).ok()?;
    let mut runner = ProductionGitRunner::default();
    let repository =
        match inspect_branch_control(root, Path::new(TRUSTED_GIT_PROGRAM), &mut runner).ok()? {
            BranchControlVisibility::Visible(repository) => repository,
            BranchControlVisibility::HiddenNonRepository
            | BranchControlVisibility::HiddenRepositoryCrossesProjectBoundary { .. } => {
                if let Ok(mut state) = core.conversation_branch.lock()
                    && state
                        .cached
                        .as_ref()
                        .map(|cached| cached.project_id.as_str())
                        == Some(project_id)
                {
                    state.cached = None;
                    state.operation_status = None;
                    state.operation_message = None;
                }
                return None;
            }
        };
    let menu = snapshot_branch_menu(&repository, &mut runner).ok()?;
    let cached = CachedConversationBranchControl {
        project_id: project_id.to_owned(),
        checked_at_ms: now_ms,
        repository,
        menu,
    };
    if let Ok(mut state) = core.conversation_branch.lock() {
        let changed_project = state
            .cached
            .as_ref()
            .is_some_and(|current| current.project_id != project_id);
        state.cached = Some(cached.clone());
        if changed_project {
            state.operation_status = None;
            state.operation_message = None;
        }
    }
    Some(cached)
}

fn conversation_branch_snapshot(
    core: &AppCoreState,
    workspace: &core::database::WorkspaceSnapshot,
    conversation: &ConversationApplicationState,
    now_ms: u64,
) -> Option<ConversationBranchControlSnapshot> {
    let cached = refresh_conversation_branch_control(core, workspace, conversation, now_ms, false)?;
    let state = core.conversation_branch.lock().ok()?;
    let pending_approval_id = state
        .pending
        .iter()
        .find(|(_, pending)| pending.project_id == cached.project_id)
        .map(|(prompt_id, _)| prompt_id.clone());
    Some(ConversationBranchControlSnapshot {
        current_branch: cached.menu.current_branch.clone(),
        branches: cached
            .menu
            .branches
            .iter()
            .map(|branch| ConversationBranchSnapshot {
                name: branch.name.clone(),
                target_oid: branch.target_oid.clone(),
                selected: cached.menu.current_branch.as_deref() == Some(branch.name.as_str()),
            })
            .collect(),
        pending_approval_id,
        operation_status: state.operation_status.clone(),
        operation_message: state.operation_message.clone(),
    })
}

fn prepare_conversation_branch_operation(
    core: &AppCoreState,
    workspace: &core::database::WorkspaceSnapshot,
    conversation: &ConversationApplicationState,
    cached: CachedConversationBranchControl,
    input: ConversationBranchInput,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<
    (
        PendingConversationBranchOperation,
        ActionFacts,
        LiveAuthorityState,
    ),
    ProtocolError,
> {
    let branch = input.branch.trim();
    if branch.is_empty() || branch.len() > 255 || branch.chars().any(char::is_control) {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The Git branch name is invalid",
            false,
        ));
    }
    let requested_operation = input.operation;
    let operation = match requested_operation {
        ConversationBranchOperation::Switch => {
            let target = cached
                .menu
                .branches
                .iter()
                .find(|candidate| candidate.name == branch)
                .ok_or_else(|| {
                    platform_boundary_error(
                        correlation_id.clone(),
                        ProtocolErrorCode::NotFound,
                        "The selected local Git branch is unavailable",
                        false,
                    )
                })?;
            GitBranchOperation::Switch {
                branch: branch.to_owned(),
                expected_target_oid: target.target_oid.clone(),
            }
        }
        ConversationBranchOperation::Create => GitBranchOperation::Create {
            branch: branch.to_owned(),
        },
    };
    let request_id = format!("git-request-{}", Uuid::new_v4().as_simple());
    let request = GitBranchRequest {
        request_id: request_id.clone(),
        repository_root: cached.repository.repository_root().to_path_buf(),
        expected_state: cached.menu.state.clone(),
        operation,
    };
    let binding = request
        .authorization_binding_sha256(&cached.repository)
        .map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::InvalidPayload,
                "The Git branch request is invalid",
                false,
            )
        })?;
    let authority = core
        .runtime
        .policy_authority
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let runtime_snapshot = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let workspace_id = workspace
        .workspace
        .as_ref()
        .map(|workspace| workspace.workspace_id.clone())
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let session_id = conversation
        .active_session_id
        .clone()
        .unwrap_or_else(|| "workspace-navigation".into());
    let live = LiveAuthorityState {
        process_generation: 1,
        configuration_version: runtime_snapshot.generation.max(1),
        policy_version: authority.policy_version,
        revocation_epoch: authority.revocation_epoch,
    };
    drop(authority);
    let canonical_target = cached.repository.repository_root().display().to_string();
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: request_id.clone(),
        tool_call_id: format!("tool-call-{request_id}"),
        tool: "c4os.git".into(),
        arguments: serde_json::json!({"gitRequestSha256": binding}),
        risk: CanonicalRisk::Medium,
        requested_authority: BTreeSet::from(["git.branch".into()]),
        canonical_target: canonical_target.clone(),
        target_version: request
            .authorization_binding_sha256(&cached.repository)
            .map_err(|_| {
                platform_boundary_error(
                    correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "The Git branch request is invalid",
                    false,
                )
            })?,
        workspace_id: workspace_id.clone(),
        session_id: session_id.clone(),
        run_id: format!("branch-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    let facts = ActionFacts {
        action_kind: "git.branch".into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::Git,
        effects: BTreeSet::from([ActionEffect::Modify]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::User,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: RepositoryState::VersionControlled,
        inside_active_project: true,
        canonical_target,
        workspace_id,
        session_id,
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    Ok((
        PendingConversationBranchOperation {
            project_id: cached.project_id,
            branch: branch.to_owned(),
            operation: requested_operation,
            repository: cached.repository,
            request,
            action,
            live,
        },
        facts,
        live,
    ))
}

fn normalized_git_branch_result(
    outcome: &Result<GitBranchOutcome, GitError>,
    canonical_target: &str,
    completed_at_ms: u64,
) -> NormalizedActionResult {
    match outcome {
        Ok(GitBranchOutcome::Switched { .. }) => NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "git-branch-switched".into(),
            exit_code: Some(0),
            changed_targets: vec![canonical_target.to_owned()],
            output_sha256: None,
            completed_at_ms,
        },
        Ok(GitBranchOutcome::Created { .. }) => NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "git-branch-created".into(),
            exit_code: Some(0),
            changed_targets: vec![canonical_target.to_owned()],
            output_sha256: None,
            completed_at_ms,
        },
        Ok(GitBranchOutcome::Blocked { .. }) => NormalizedActionResult {
            status: NormalizedActionStatus::Failed,
            result_code: "git-branch-conflict".into(),
            exit_code: Some(1),
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        },
        Err(_) => NormalizedActionResult {
            status: NormalizedActionStatus::Failed,
            result_code: "git-branch-failed".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms,
        },
    }
}

fn bounded_git_path(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_control() {
                '\u{fffd}'
            } else {
                character
            }
        })
        .take(512)
        .collect()
}

fn record_conversation_branch_outcome(
    core: &AppCoreState,
    outcome: &GitBranchOutcome,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let mut state = core
        .conversation_branch
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    state.cached = None;
    match outcome {
        GitBranchOutcome::Switched {
            branch,
            preserved_dirty_state,
        } => {
            state.operation_status = Some("switched".into());
            state.operation_message = Some(if *preserved_dirty_state {
                format!("Switched to {branch}; existing worktree changes were preserved.")
            } else {
                format!("Switched to {branch}.")
            });
        }
        GitBranchOutcome::Created {
            branch,
            preserved_dirty_state,
        } => {
            state.operation_status = Some("created".into());
            state.operation_message = Some(if *preserved_dirty_state {
                format!("Created {branch}; existing worktree changes were preserved.")
            } else {
                format!("Created {branch}.")
            });
        }
        GitBranchOutcome::Blocked {
            conflicting_paths, ..
        } => {
            state.operation_status = Some("blocked".into());
            let paths = conflicting_paths
                .iter()
                .map(|path| bounded_git_path(path))
                .collect::<Vec<_>>()
                .join(", ");
            state.operation_message = Some(
                format!(
                    "Git preserved the worktree and blocked the branch change because these paths conflict: {paths}"
                )
                .chars()
                .take(4_096)
                .collect(),
            );
        }
    }
    Ok(())
}

fn execute_conversation_branch_operation(
    core: &AppCoreState,
    pending: PendingConversationBranchOperation,
    token: AuthorizationToken,
    approval_prompt_id: Option<&str>,
    live: LiveAuthorityState,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let PendingConversationBranchOperation {
        repository,
        request,
        action,
        ..
    } = pending;
    let canonical_target = action.canonical_target.clone();
    let repository_for_effect = repository.clone();
    let request_for_effect = request.clone();
    let mut runner = ProductionGitRunner::default();
    let mut branch_outcome = None;
    core.runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator.execute_direct_action(
                &token,
                &action,
                live,
                approval_prompt_id,
                now_ms,
                |permit| {
                    let outcome = GitOperationAuthorization::from_gateway(
                        permit,
                        &repository_for_effect,
                        request_for_effect,
                    )
                    .and_then(|authorization| {
                        execute_branch_operation(
                            &repository_for_effect,
                            request,
                            authorization,
                            &mut runner,
                        )
                    });
                    let normalized = normalized_git_branch_result(
                        &outcome,
                        &canonical_target,
                        now_ms.saturating_add(1),
                    );
                    branch_outcome = Some(outcome);
                    normalized
                },
            )?;
            Ok(())
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let outcome = branch_outcome
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?
        .map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The Git branch operation did not complete safely",
                false,
            )
        })?;
    record_conversation_branch_outcome(core, &outcome, correlation_id)?;
    Ok(())
}

fn build_conversation_snapshot(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<ConversationSnapshot, ProtocolError> {
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let workspace = active_workspace_snapshot(core, correlation_id.clone())?;
    let state = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .clone();
    let (runtime, capability_generation) = core
        .runtime
        .snapshot_with_capability_generation(now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let durable_sessions = core
        .runtime
        .durable_sessions()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let active_session_record = state.active_session_id.as_deref().and_then(|session_id| {
        durable_sessions
            .iter()
            .find(|record| record.session_id == session_id)
    });
    let preferred_runtime_id = active_session_record
        .and_then(SessionRecord::binding)
        .map(|binding| binding.runtime_id.as_str());
    let effective_models = core
        .runtime
        .effective_conversation_models(preferred_runtime_id, now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let active_record = active_session_record
        .map(project_session_snapshot)
        .transpose()?;
    let pending = match &state.pending {
        conversation::PendingChatState::Inactive => None,
        conversation::PendingChatState::Draft(draft) => Some(PendingConversationSnapshot {
            session_id: SessionId::new(draft.session_id.clone())?,
            project_id: ProjectId::new(draft.project_id.clone())?,
            title: "New Chat".into(),
            attachments: state
                .pending_attachments
                .iter()
                .enumerate()
                .map(|(index, attachment)| project_attachment_snapshot(attachment, index + 1))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        conversation::PendingChatState::PromotionRequested(promotion) => {
            Some(PendingConversationSnapshot {
                session_id: SessionId::new(promotion.session_id.clone())?,
                project_id: ProjectId::new(promotion.project_id.clone())?,
                title: promotion.title.clone(),
                attachments: state
                    .pending_attachments
                    .iter()
                    .enumerate()
                    .map(|(index, attachment)| project_attachment_snapshot(attachment, index + 1))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    };
    let draft = state.active_draft();
    let active_model_route = draft
        .provider_id
        .clone()
        .zip(draft.model_id.clone())
        .or_else(|| {
            runtime
                .providers
                .providers
                .iter()
                .filter(|provider| provider.profile.enabled)
                .find_map(|provider| {
                    provider
                        .selected_model_id
                        .as_ref()
                        .map(|model_id| (provider.profile.provider_id.clone(), model_id.clone()))
                })
        });
    let workspace_record = workspace.workspace.as_ref();
    let branch_control = conversation_branch_snapshot(core, &workspace, &state, now_ms);
    let branch_project_id = branch_control
        .as_ref()
        .and_then(|_| state.active_project_id.clone());
    Ok(ConversationSnapshot {
        protocol_version: protocol::PROTOCOL_VERSION,
        generation: StateGeneration(conversation_authority_generation(
            state.generation,
            workspace.generation,
            runtime.generation,
            runtime.providers.generation,
            runtime.runtimes.state_generation,
            capability_generation,
        )),
        authority: "rust-core".into(),
        workspace_id: workspace_record
            .map(|record| WorkspaceId::new(record.workspace_id.clone()))
            .transpose()?,
        workspace_name: workspace_record.map(|record| record.display_name.clone()),
        active_project_id: state.active_project_id.map(ProjectId::new).transpose()?,
        active_session_id: state.active_session_id.map(SessionId::new).transpose()?,
        pending,
        draft: ConversationDraftSnapshot {
            prompt: draft.prompt,
            attachments: draft
                .attachments
                .iter()
                .enumerate()
                .map(|(index, attachment)| project_attachment_snapshot(attachment, index + 1))
                .collect::<Result<Vec<_>, _>>()?,
            next_attachment_reference: draft.next_attachment_reference,
            provider_id: draft.provider_id,
            model_id: draft.model_id,
            reasoning_mode: draft.reasoning_mode,
            mode: draft.mode,
            reply_target_id: draft.reply_target_id,
        },
        projects: workspace
            .projects
            .iter()
            .filter(|project| project.lifecycle_state == core::database::LifecycleState::Active)
            .map(|project| {
                Ok(ConversationProjectSnapshot {
                    project_id: ProjectId::new(project.project_id.clone())?,
                    display_name: project.display_name.clone(),
                    path_state: match project.path_state {
                        core::database::ProjectPathState::Found => "found",
                        core::database::ProjectPathState::Missing => "missing",
                        core::database::ProjectPathState::Relocated => "relocated",
                    }
                    .into(),
                    position: project.position,
                    git_versioned: branch_project_id.as_deref()
                        == Some(project.project_id.as_str()),
                })
            })
            .collect::<Result<Vec<_>, ProtocolError>>()?,
        sessions: workspace
            .chats
            .iter()
            .filter(|chat| chat.lifecycle_state == core::database::LifecycleState::Active)
            .map(|chat| {
                Ok(ConversationSessionSummarySnapshot {
                    session_id: SessionId::new(chat.chat_id.clone())?,
                    project_id: ProjectId::new(chat.project_id.clone())?,
                    title: chat.title.clone(),
                    updated_at_ms: u64::try_from(chat.updated_at).map_err(|_| {
                        ProtocolError::new(
                            ProtocolErrorCode::InvalidPayload,
                            "Chat timestamp is invalid",
                            false,
                        )
                    })?,
                })
            })
            .collect::<Result<Vec<_>, ProtocolError>>()?,
        active_conversation: active_record,
        models: runtime
            .providers
            .providers
            .iter()
            .filter(|provider| provider.profile.enabled)
            .flat_map(|provider| {
                let active_model_route = active_model_route.as_ref();
                let effective_models = &effective_models;
                provider
                    .models
                    .values()
                    .filter(move |model| model.is_production_ready_at(now_ms))
                    .map(move |model| {
                        let effective = effective_models
                            .get(&(provider.profile.provider_id.clone(), model.model_id.clone()));
                        ConversationModelSnapshot {
                            provider_id: provider.profile.provider_id.clone(),
                            provider_name: provider.profile.display_name.clone(),
                            model_id: model.model_id.clone(),
                            selected: active_model_route.is_some_and(|(provider_id, model_id)| {
                                provider_id == &provider.profile.provider_id
                                    && model_id == &model.model_id
                            }),
                            available: effective.is_some_and(|descriptor| {
                                descriptor.lifecycle
                                    != runtime::capability::ModelLifecycle::Unavailable
                            }),
                            supports_vision: effective.is_some_and(|descriptor| {
                                descriptor.feature_state(CapabilityKey::InputImage).usable()
                            }),
                            supports_tools: effective.is_some_and(|descriptor| {
                                descriptor
                                    .feature_state(CapabilityKey::ToolCalling)
                                    .usable()
                            }),
                            supports_reasoning: effective.is_some_and(|descriptor| {
                                descriptor.feature_state(CapabilityKey::Reasoning).usable()
                            }),
                            supports_audio: effective.is_some_and(|descriptor| {
                                descriptor.feature_state(CapabilityKey::InputAudio).usable()
                            }),
                            context_tokens: effective
                                .and_then(|descriptor| {
                                    descriptor.numeric_maximum(NumericCapabilityKey::ContextTokens)
                                })
                                .unwrap_or(0),
                        }
                    })
            })
            .collect(),
        branch_control,
    })
}

fn project_attachment_snapshot(
    attachment: &AttachmentSnapshot,
    fallback_reference: usize,
) -> Result<ConversationAttachmentSnapshot, ProtocolError> {
    let fallback_reference = u32::try_from(fallback_reference).map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Attachment reference is invalid",
            false,
        )
    })?;
    Ok(ConversationAttachmentSnapshot {
        attachment_id: AttachmentId::new(attachment.attachment_id.clone())?,
        display_name: attachment.display_name.clone(),
        media_type: attachment.media_type.clone(),
        byte_length: attachment.byte_length,
        stable_reference: attachment.stable_reference.clone(),
        original_reference: if attachment.original_reference == 0 {
            fallback_reference
        } else {
            attachment.original_reference
        },
    })
}

fn project_session_snapshot(
    record: &SessionRecord,
) -> Result<ConversationSessionSnapshot, ProtocolError> {
    Ok(ConversationSessionSnapshot {
        session_id: SessionId::new(record.session_id.clone())?,
        title: record.title.clone(),
        turns: record
            .turns
            .iter()
            .map(|turn| {
                Ok(ConversationTurnSnapshot {
                    turn_id: TurnId::new(turn.turn_id.clone())?,
                    prompt: turn.prompt.clone(),
                    attachments: turn
                        .attachments
                        .iter()
                        .enumerate()
                        .map(|(index, attachment)| {
                            project_attachment_snapshot(attachment, index + 1)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    submitted_at_ms: turn.submitted_at_ms,
                })
            })
            .collect::<Result<Vec<_>, ProtocolError>>()?,
        attempts: record
            .attempts
            .iter()
            .map(project_attempt_snapshot)
            .collect::<Result<Vec<_>, _>>()?,
        active_attempt_id: record
            .active_attempt_id
            .clone()
            .map(AttemptId::new)
            .transpose()?,
    })
}

fn project_attempt_snapshot(
    attempt: &runtime::session::RunAttemptRecord,
) -> Result<ConversationAttemptSnapshot, ProtocolError> {
    use conversation::{
        ProjectedActivityKind, ProjectedRunStatus, RuntimeEventInput, RuntimeEventKindInput,
        RuntimeRunInput, RuntimeRunStatusInput,
    };
    use runtime::session::{RunAttemptStatus, RunEventKind};
    let projected = conversation::project_runtime_run(RuntimeRunInput {
        attempt_id: attempt.attempt_id.clone(),
        turn_id: attempt.turn_id.clone(),
        status: match attempt.status {
            RunAttemptStatus::Dispatching => RuntimeRunStatusInput::Dispatching,
            RunAttemptStatus::Streaming { .. } => RuntimeRunStatusInput::Streaming,
            RunAttemptStatus::CancellationRequested { .. } => {
                RuntimeRunStatusInput::CancellationRequested
            }
            RunAttemptStatus::Completed { .. } => RuntimeRunStatusInput::Completed,
            RunAttemptStatus::Failed { .. } => RuntimeRunStatusInput::Failed,
            RunAttemptStatus::Interrupted { .. } => RuntimeRunStatusInput::Interrupted,
            RunAttemptStatus::Cancelled { .. } => RuntimeRunStatusInput::Cancelled,
        },
        events: attempt
            .events
            .iter()
            .map(|event| RuntimeEventInput {
                sequence: event.sequence,
                kind: match event.kind {
                    RunEventKind::Status => RuntimeEventKindInput::Status,
                    RunEventKind::TextDelta => RuntimeEventKindInput::TextDelta,
                    RunEventKind::ReasoningDelta => RuntimeEventKindInput::ReasoningDelta,
                    RunEventKind::ReasoningSummary => RuntimeEventKindInput::SafeReasoningSummary,
                    RunEventKind::WorkActivity => RuntimeEventKindInput::WorkActivity,
                    RunEventKind::ActionIntent => RuntimeEventKindInput::ActionIntent,
                    RunEventKind::ActionProgress => RuntimeEventKindInput::ActionProgress,
                    RunEventKind::ActionResult => RuntimeEventKindInput::ActionResult,
                    RunEventKind::Media => RuntimeEventKindInput::Media,
                    RunEventKind::Usage => RuntimeEventKindInput::Usage,
                    RunEventKind::Error => RuntimeEventKindInput::Error,
                    RunEventKind::Completion => RuntimeEventKindInput::Completion,
                },
                payload: event.payload.clone(),
            })
            .collect(),
    });
    let (input_tokens, output_tokens) = attempt
        .events
        .iter()
        .rev()
        .find(|event| event.kind == RunEventKind::Usage)
        .and_then(|event| parse_usage_tokens(&event.payload))
        .unwrap_or((0, 0));
    let finished_at_ms = match &attempt.status {
        RunAttemptStatus::Completed { completed_at_ms } => Some(*completed_at_ms),
        RunAttemptStatus::Failed { failed_at_ms, .. } => Some(*failed_at_ms),
        RunAttemptStatus::Interrupted {
            interrupted_at_ms, ..
        } => Some(*interrupted_at_ms),
        RunAttemptStatus::Cancelled { cancelled_at_ms } => Some(*cancelled_at_ms),
        RunAttemptStatus::Dispatching
        | RunAttemptStatus::Streaming { .. }
        | RunAttemptStatus::CancellationRequested { .. } => None,
    };
    let started_at_ms = attempt
        .events
        .first()
        .map(|event| event.recorded_at_ms)
        .unwrap_or(attempt.created_at_ms);
    Ok(ConversationAttemptSnapshot {
        attempt_id: AttemptId::new(projected.attempt_id)?,
        turn_id: TurnId::new(projected.turn_id)?,
        status: match projected.status {
            ProjectedRunStatus::Starting => "starting",
            ProjectedRunStatus::Working => "working",
            ProjectedRunStatus::Cancelling => "cancelling",
            ProjectedRunStatus::Completed => "completed",
            ProjectedRunStatus::Failed => "failed",
            ProjectedRunStatus::Interrupted => "interrupted",
            ProjectedRunStatus::Cancelled => "cancelled",
        }
        .into(),
        assistant_markdown: projected.assistant_markdown,
        activities: projected
            .activities
            .into_iter()
            .map(|activity| ConversationActivitySnapshot {
                sequence: activity.sequence,
                kind: match activity.kind {
                    ProjectedActivityKind::Status => "status",
                    ProjectedActivityKind::Work => "work",
                    ProjectedActivityKind::ReasoningSummary => "reasoning-summary",
                    ProjectedActivityKind::Action => "action",
                    ProjectedActivityKind::Media => "media",
                    ProjectedActivityKind::Usage => "usage",
                    ProjectedActivityKind::Error => "error",
                    ProjectedActivityKind::Completion => "completion",
                }
                .into(),
                label: activity.label.into(),
                detail: activity.detail,
            })
            .collect(),
        runtime_id: RuntimeId::new(attempt.context.runtime_id.clone())?,
        runtime_kind: match attempt.context.runtime_kind {
            SessionRuntimeKind::OpenCode => "open-code",
            SessionRuntimeKind::Pi => "pi",
        }
        .into(),
        environment_id: EnvironmentId::new(attempt.context.environment.environment_id.clone())?,
        provider_id: attempt.context.model_route.provider_id.clone(),
        model_id: attempt.context.model_route.model_id.clone(),
        adapter_id: attempt.context.adapter.adapter_id.clone(),
        input_tokens,
        output_tokens,
        duration_ms: finished_at_ms.map(|finished| finished.saturating_sub(started_at_ms)),
    })
}

fn parse_usage_tokens(payload: &str) -> Option<(u64, u64)> {
    let mut input = None;
    let mut output = None;
    for field in payload.split(';') {
        let (name, value) = field.split_once('=')?;
        let value = value.parse::<u64>().ok()?;
        match name {
            "input" => input = Some(value),
            "output" => output = Some(value),
            _ => return None,
        }
    }
    Some((input?, output?))
}

fn message_reply_context(
    runtime: &RuntimeApplicationService,
    session_id: &str,
    target_id: &str,
    correlation_id: protocol::CorrelationId,
) -> Result<MessageReplyContextSnapshot, ProtocolError> {
    let record = runtime
        .session(session_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let (target_kind, source) = if let Some(turn) = record.turn(target_id) {
        let source = turn.prompt.clone().unwrap_or_else(|| {
            turn.attachments
                .iter()
                .map(|attachment| format!("[{}]", attachment.display_name))
                .collect::<Vec<_>>()
                .join(" ")
        });
        ("user-message", source)
    } else if let Some(attempt) = record.attempt(target_id) {
        if !attempt.status.is_terminal() {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "Wait for the response to finish before replying to it",
                true,
            ));
        }
        (
            "assistant-message",
            project_attempt_snapshot(attempt)?.assistant_markdown,
        )
    } else {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::NotFound,
            "The Reply target is unavailable",
            false,
        ));
    };
    if source.trim().is_empty() {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The Reply target has no stable content",
            false,
        ));
    }
    let source_excerpt = source.chars().take(4_096).collect::<String>();
    Ok(MessageReplyContextSnapshot {
        target_id: target_id.into(),
        target_kind: target_kind.into(),
        source_sha256: sha256_bytes(source.as_bytes()),
        source_excerpt,
    })
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

const DEBUG_ACCEPTANCE_HOME_ARGUMENT: &str = "--c4os-acceptance-home";

/// Selects an isolated native-acceptance home only in debug builds. Release
/// binaries reject the switch and always retain the ordinary production home.
fn c4os_home_for_startup(default_home: PathBuf) -> Result<PathBuf, std::io::Error> {
    #[cfg(debug_assertions)]
    {
        if let Some(path) = debug_acceptance_home_from_args()? {
            return Ok(path);
        }
    }
    #[cfg(not(debug_assertions))]
    if std::env::args_os().any(|argument| argument == DEBUG_ACCEPTANCE_HOME_ARGUMENT) {
        return Err(std::io::Error::other(
            "the native acceptance home is unavailable in release builds",
        ));
    }
    Ok(default_home)
}

#[cfg(debug_assertions)]
fn debug_acceptance_home_from_args() -> Result<Option<PathBuf>, std::io::Error> {
    let mut arguments = std::env::args_os().skip(1);
    let mut selected = None;
    while let Some(argument) = arguments.next() {
        if argument != DEBUG_ACCEPTANCE_HOME_ARGUMENT {
            continue;
        }
        if selected.is_some() {
            return Err(std::io::Error::other(
                "the native acceptance home may be supplied only once",
            ));
        }
        let value = arguments.next().ok_or_else(|| {
            std::io::Error::other("the native acceptance home requires an absolute path")
        })?;
        selected = Some(validate_debug_acceptance_home(Path::new(&value))?);
    }
    Ok(selected)
}

#[cfg(all(debug_assertions, unix))]
fn validate_debug_acceptance_home(path: &Path) -> Result<PathBuf, std::io::Error> {
    if !path.is_absolute() {
        return Err(std::io::Error::other(
            "the native acceptance home must be absolute",
        ));
    }
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(std::io::Error::other(
            "the native acceptance home must be an owned directory",
        ));
    }
    let canonical = path.canonicalize()?;
    if canonical != path {
        return Err(std::io::Error::other(
            "the native acceptance home cannot traverse aliases or symlinks",
        ));
    }
    let permitted = [std::env::temp_dir(), PathBuf::from("/private/tmp")]
        .into_iter()
        .filter_map(|candidate| candidate.canonicalize().ok())
        .any(|root| canonical.starts_with(root));
    if !permitted {
        return Err(std::io::Error::other(
            "the native acceptance home must remain under the system temporary root",
        ));
    }
    if metadata.mode() & 0o777 != 0o700 || metadata.uid() != unsafe { libc::geteuid() } {
        return Err(std::io::Error::other(
            "the native acceptance home must be owned by this user with mode 0700",
        ));
    }
    Ok(canonical)
}

#[cfg(all(debug_assertions, not(unix)))]
fn validate_debug_acceptance_home(_path: &Path) -> Result<PathBuf, std::io::Error> {
    Err(std::io::Error::other(
        "the native acceptance home is unavailable on this target",
    ))
}

#[cfg(all(test, debug_assertions, unix))]
mod debug_acceptance_home_tests {
    use super::validate_debug_acceptance_home;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn isolated_home_requires_canonical_owned_mode_0700_directory() {
        let temporary = TempDir::new().expect("temporary acceptance root");
        fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700))
            .expect("acceptance permissions");
        let canonical = temporary.path().canonicalize().expect("canonical home");
        assert_eq!(
            validate_debug_acceptance_home(&canonical).expect("accepted home"),
            canonical
        );

        fs::set_permissions(&canonical, fs::Permissions::from_mode(0o755))
            .expect("loosen acceptance permissions");
        assert!(validate_debug_acceptance_home(&canonical).is_err());
    }
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
        .on_window_event(handle_conversation_drop_event)
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
            let c4os_home = c4os_home_for_startup(app.path().home_dir()?.join(".c4os"))?;
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
            if let Some(workspace) = &active_workspace {
                runtime.bind_workspace(Arc::clone(workspace.database_actor()))?;
            } else {
                runtime.clear_runtime_installations_without_workspace(now_ms)?;
            }
            let conversation = if let Some(workspace) = &active_workspace {
                let query = core::database::SnapshotQuery::new(core::database::MAX_READ_RECORDS)?;
                let snapshot = workspace.snapshot(query)?;
                let persisted = workspace.database_actor().conversation_state()?;
                ConversationApplicationState::restore(Some(&snapshot), persisted.as_ref())
            } else {
                ConversationApplicationState::default()
            };
            let active_workspace = Arc::new(Mutex::new(active_workspace));
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let runtime_production = Arc::new(ManagedProductionRuntime::default());
            app.manage(AppCoreState {
                database,
                _configuration: Mutex::new(configuration),
                active_workspace: Arc::clone(&active_workspace),
                conversation_operation: Mutex::new(()),
                conversation: Mutex::new(conversation),
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                runtime_production: Arc::clone(&runtime_production),
                runtime: Arc::clone(&runtime),
                platform,
                platform_snapshot,
                picker_grants: Mutex::new(PickerGrantRegistry::default()),
                conversation_drop: Mutex::new(NativeConversationDropState::default()),
                conversation_branch: Mutex::new(NativeConversationBranchState::default()),
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
            conversation_snapshot,
            conversation_attachment_preview,
            conversation_request_branch,
            conversation_answer_branch_approval,
            conversation_begin_pending,
            conversation_cancel_pending,
            conversation_update_draft,
            conversation_submit,
            conversation_cancel_attempt,
            conversation_retry_attempt,
            conversation_activate_session,
            conversation_activate_project,
            conversation_add_project,
            conversation_relocate_project,
            conversation_rename_project,
            conversation_copy_project_path,
            conversation_reveal_project,
            conversation_reorder_projects,
            conversation_inactivate_project,
            conversation_inactivate_session,
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
