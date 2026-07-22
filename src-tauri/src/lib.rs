pub mod artifact;
pub mod browser;
pub mod conversation;
pub mod core;
pub mod execution;
pub mod extension;
pub mod mcp;
pub mod platform;
pub mod protocol;
pub mod runtime;
pub mod security;

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod production_mcp_broker_tests;
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod production_mcp_sampling_tests;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use execution::environment::TrustedProjectRoot;
use execution::filesystem::{
    ExpectedFileState, FileVersion as ProjectFileVersion,
    FolderEntryKind as ProjectFolderEntryKind, ProjectFilesystem, ProjectFilesystemError,
    ProjectFilesystemLimits,
};
use execution::git::{
    ActiveProjectRepository, BranchControlVisibility, GitBranchMenuSnapshot, GitBranchOperation,
    GitBranchOutcome, GitBranchRequest, GitError, GitOperationAuthorization, ProductionGitRunner,
    execute_branch_operation, inspect_branch_control, snapshot_branch_menu,
};
use execution::terminal::{
    MAX_TERMINAL_DRAIN_BYTES, MAX_TERMINAL_DRAIN_EVENTS, TerminalAcknowledgeRequest,
    TerminalCommandIdentity as SupervisedTerminalCommandIdentity, TerminalCompletedSessionRecord,
    TerminalDimensions as PtyDimensions, TerminalDrainRequest, TerminalEventKind,
    TerminalExecuteRequest, TerminalResizeRequest, TerminalRestartRecord, TerminalSessionKey,
    TerminalStdinRequest, TerminalStopRequest, TerminalSupervisor,
};
use platform::{
    ColorScheme, INITIAL_REVEAL_FALLBACK_MS, InitialThemeSnapshot, InitialThemeSource,
    NativePickerRequest, NativePickerSelection, OPEN_SETTINGS_COMMAND_ID,
    PLATFORM_CONTRACT_VERSION, PickerGrantRegistry, PickerGrantSnapshot, PickerObjectKind,
    PickerOutcome, PickerPurpose, PlatformCapabilities, PlatformService, PlatformSnapshot,
    PlatformTarget, SETTINGS_ACCELERATOR, SETTINGS_MENU_ITEM_ID, SETTINGS_ROUTE,
};
use protocol::{
    ArtifactApprovalAnswer, ArtifactApprovalInput, ArtifactBreadcrumbSnapshot,
    ArtifactBrowserIdentityInput, ArtifactBrowserNavigateInput, ArtifactBrowserNavigationIntent,
    ArtifactBrowserNoticeSnapshot, ArtifactBrowserOpenInput, ArtifactBrowserSnapshot,
    ArtifactBrowserViewportInput, ArtifactContextExpandInput, ArtifactFileConflictInput,
    ArtifactFileConflictResolution, ArtifactFileDraftInput, ArtifactFileSnapshot,
    ArtifactFileStateSnapshot, ArtifactFolderEntrySnapshot, ArtifactFolderListingSnapshot,
    ArtifactFolderNavigateInput, ArtifactFolderSelectInput, ArtifactFolderSnapshot,
    ArtifactHistorySnapshot, ArtifactMutationInput, ArtifactOpenInput,
    ArtifactProviderStateSnapshot, ArtifactReplyInput, ArtifactResourceVersionSnapshot,
    ArtifactShellStatusSnapshot, ArtifactSnapshot, ArtifactTerminalOperationInput,
    ArtifactTerminalOutputAckInput, ArtifactTerminalResizeInput, ArtifactTerminalRunInput,
    ArtifactTerminalSnapshot, ArtifactTerminalStdinInput, ArtifactWorkspaceSnapshot, AttachmentId,
    AttemptId, ConversationActivitySnapshot, ConversationArtifactCapabilitySnapshot,
    ConversationArtifactContextSegmentSnapshot, ConversationArtifactContextSnapshot,
    ConversationAttachmentPreviewInput, ConversationAttachmentPreviewSnapshot,
    ConversationAttachmentSnapshot, ConversationAttemptSnapshot, ConversationBranchApprovalAnswer,
    ConversationBranchApprovalInput, ConversationBranchControlSnapshot, ConversationBranchInput,
    ConversationBranchOperation, ConversationBranchSnapshot, ConversationDraftInput,
    ConversationDraftSnapshot, ConversationMcpProvenanceSnapshot,
    ConversationMcpToolProvenanceSnapshot, ConversationModelSnapshot, ConversationProjectSnapshot,
    ConversationRetryInput, ConversationSessionSnapshot, ConversationSessionSummarySnapshot,
    ConversationSnapshot, ConversationSubmitInput, ConversationTurnSnapshot, EnvironmentId,
    FoundationSnapshot, PendingConversationSnapshot, PickerGrantId, ProjectId, ProtocolEnvelope,
    ProtocolError, ProtocolErrorCode, RequestId, RuntimeId, SessionId, SnapshotRequest,
    StateGeneration, TurnId, WorkspaceId, WorkspaceRecentSnapshot, WorkspaceStartSnapshot,
};
use runtime::action_bridge::{
    RuntimeActionProposal, RuntimeApprovalDecision, RuntimeAuthorization,
    RuntimeEffectCompletionCertainty, RuntimeEffectResult, RuntimeExecutionReceipt,
    RuntimeGatewayDecision,
};
use runtime::broker_worker::{BrokerActionApplication, BrokerDeferredStart, BrokerDeferredTicket};
use runtime::capability::{
    AttachmentMediaType, AttachmentRequirement, CapabilityDescriptor, CapabilityKey,
    DraftRequirements, NumericCapabilityKey, PolicyPreflight, PreflightOutcome,
    effective_intersection,
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
    PeerSamplingRequest, PeerSamplingResult, RetryDispatchOptions, RuntimeDispatchPeer,
    RuntimeDispatchRegistry, TurnDispatchOptions, coordinate_cancellation,
    coordinate_first_dispatch, coordinate_polled_events, coordinate_recovery,
    coordinate_retry_dispatch, coordinate_turn_dispatch,
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
use runtime::pi::PiSamplingMessage;
use runtime::provider::{ProviderProbe, ProviderProfile, ProviderTestReport};
use runtime::session::{
    AttachmentSnapshot, FirstSubmission, MAX_REPLY_SOURCE_EXCERPT_BYTES,
    MessageReplyContextSnapshot, RetryRequest, RuntimeKind as SessionRuntimeKind, SessionError,
    SessionRecord, SessionService, SkillContextSnapshot, TerminalAttemptOutcome, TurnSubmission,
};
use runtime::supervisor::{
    CompatibilityState, HealthState, RuntimeInstallation, RuntimeSupervisor,
};
use security::authorization::{
    ApprovalAnswer, AuthorizationToken, CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction,
    CanonicalRisk, LiveAuthorityState,
};
use security::gateway::{
    ActionEffectLease, ApprovalResponse, ExecutionPermit, GatewayProposal, NormalizedActionResult,
    NormalizedActionStatus,
};
use security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ClassificationConfidence, PolicyConfiguration,
    PolicyDecision, RepositoryState,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
#[cfg(all(debug_assertions, unix))]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, Weak, atomic::AtomicBool};
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
struct ProductionMcpDeferredJob {
    result: std::sync::mpsc::Receiver<RuntimeEffectResult>,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[derive(Clone, Default)]
struct ProductionMcpCancellationRegistry {
    jobs: Arc<Mutex<BTreeMap<String, ProductionMcpCancellationRegistration>>>,
    quiescing_servers: Arc<Mutex<BTreeSet<String>>>,
    credential_servers: Arc<Mutex<BTreeMap<String, BTreeSet<String>>>>,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
struct ProductionMcpCancellationRegistration {
    server_id: String,
    cancellation: mcp::transport::McpCancellation,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl ProductionMcpCancellationRegistry {
    fn register(
        &self,
        ticket: String,
        server_id: String,
        cancellation: mcp::transport::McpCancellation,
    ) -> Result<(), ()> {
        let quiescing = self.quiescing_servers.lock().map_err(|_| ())?;
        if quiescing.contains(&server_id) {
            return Err(());
        }
        let mut jobs = self.jobs.lock().map_err(|_| ())?;
        if jobs.contains_key(&ticket) {
            return Err(());
        }
        jobs.insert(
            ticket,
            ProductionMcpCancellationRegistration {
                server_id,
                cancellation,
            },
        );
        Ok(())
    }

    /// Atomically blocks later registrations before signalling every current
    /// operation. Lifecycle mutation holds the service mutex while calling
    /// this, so no execution can cross the quiescing boundary unobserved.
    fn quiesce_server(&self, server_id: &str) -> usize {
        let Ok(mut quiescing) = self.quiescing_servers.lock() else {
            return 0;
        };
        quiescing.insert(server_id.to_owned());
        let cancellations = self
            .jobs
            .lock()
            .map(|jobs| {
                jobs.values()
                    .filter(|job| job.server_id == server_id)
                    .map(|job| job.cancellation.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        drop(quiescing);
        for cancellation in &cancellations {
            cancellation.cancel();
        }
        cancellations.len()
    }

    fn allow_server(&self, server_id: &str) {
        if let Ok(mut quiescing) = self.quiescing_servers.lock() {
            quiescing.remove(server_id);
        }
    }

    fn complete(&self, ticket: &str) {
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.remove(ticket);
        }
    }

    fn cancel_ticket(&self, ticket: &str) -> bool {
        self.jobs
            .lock()
            .ok()
            .and_then(|jobs| jobs.get(ticket).map(|job| job.cancellation.clone()))
            .is_some_and(|cancellation| {
                cancellation.cancel();
                true
            })
    }

    fn refresh_credential_bindings(&self, snapshot: &mcp::McpServiceSnapshot) {
        let mut bindings = BTreeMap::<String, BTreeSet<String>>::new();
        for server in &snapshot.servers {
            for reference in mcp_server_vault_references(server) {
                bindings
                    .entry(reference)
                    .or_default()
                    .insert(server.server_id.clone());
            }
        }
        if let Ok(mut current) = self.credential_servers.lock() {
            *current = bindings;
        }
    }

    fn cancel_credential(&self, credential_reference: &str) -> usize {
        let servers = self
            .credential_servers
            .lock()
            .ok()
            .and_then(|bindings| bindings.get(credential_reference).cloned())
            .unwrap_or_default();
        servers
            .iter()
            .map(|server_id| self.quiesce_server(server_id))
            .sum()
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn mcp_server_vault_references(server: &mcp::McpServerSnapshot) -> BTreeSet<String> {
    let mut references = BTreeSet::new();
    let mut include = |secret: &mcp::McpSecretReference| {
        if let mcp::McpSecretReference::Vault {
            credential_reference,
        } = secret
        {
            references.insert(credential_reference.clone());
        }
    };
    match &server.transport {
        mcp::McpTransportDefinition::Stdio { environment, .. } => {
            for binding in environment {
                if let mcp::McpEnvironmentSource::Secret { reference } = &binding.source {
                    include(reference);
                }
            }
        }
        mcp::McpTransportDefinition::StreamableHttp {
            bearer, headers, ..
        } => {
            if let Some(bearer) = bearer {
                include(bearer);
            }
            for header in headers {
                if let mcp::McpHeaderSource::Secret { reference } = &header.source {
                    include(reference);
                }
            }
        }
    }
    references
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
struct ProductionMcpDeferredFacility<R, A, F>
where
    R: mcp::service::McpRepository,
    A: mcp::service::McpAuthority,
    F: mcp::transport::McpTransportFactory,
{
    service: Arc<tokio::sync::Mutex<mcp::service::McpService<R, A, F>>>,
    cancellations: ProductionMcpCancellationRegistry,
    sampling_parents: mcp::production_sampling::McpSamplingParentRegistry,
    jobs: BTreeMap<String, ProductionMcpDeferredJob>,
    next_ticket: u64,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<R, A, F> ProductionMcpDeferredFacility<R, A, F>
where
    R: mcp::service::McpRepository,
    A: mcp::service::McpAuthority,
    F: mcp::transport::McpTransportFactory,
{
    fn new(
        service: Arc<tokio::sync::Mutex<mcp::service::McpService<R, A, F>>>,
        cancellations: ProductionMcpCancellationRegistry,
        sampling_parents: mcp::production_sampling::McpSamplingParentRegistry,
    ) -> Self {
        Self {
            service,
            cancellations,
            sampling_parents,
            jobs: BTreeMap::new(),
            next_ticket: 1,
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<R, A, F> runtime::opencode_broker::InstalledDeferredBrokerFacility
    for ProductionMcpDeferredFacility<R, A, F>
where
    R: mcp::service::McpRepository + 'static,
    A: mcp::service::McpAuthority + 'static,
    F: mcp::transport::McpTransportFactory + 'static,
{
    fn start(&mut self, permit: ExecutionPermit) -> BrokerDeferredStart {
        if self.jobs.len() >= 128 {
            return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "mcp-broker-capacity",
                permit.consumed_at_ms().saturating_add(1),
            ));
        }
        let ticket_value = format!("mcp-job-{}", self.next_ticket);
        self.next_ticket = self.next_ticket.saturating_add(1).max(1);
        let ticket = match BrokerDeferredTicket::new(ticket_value.clone()) {
            Ok(ticket) => ticket,
            Err(_) => {
                return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                    "mcp-broker-ticket-invalid",
                    permit.consumed_at_ms().saturating_add(1),
                ));
            }
        };
        let cancellation = mcp::transport::McpCancellation::default();
        let task_cancellation = cancellation.clone();
        let service = Arc::clone(&self.service);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let started_at_ms = permit.consumed_at_ms();
        let canonical_target = permit.action().canonical_target.clone();
        let server_id = permit
            .action()
            .arguments
            .get("resolved")
            .and_then(serde_json::Value::as_object)
            .and_then(|resolved| resolved.get("serverId"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let Some(_server_id) = server_id else {
            return BrokerDeferredStart::Rejected(NormalizedActionResult::denied(
                "mcp-broker-server-binding-invalid",
                started_at_ms.saturating_add(1),
            ));
        };
        let cancellations = self.cancellations.clone();
        let sampling_parents = self.sampling_parents.clone();
        let parent_action = permit.action().clone();
        let task_ticket = ticket_value.clone();
        tauri::async_runtime::spawn(async move {
            let prepared = service
                .lock()
                .await
                .begin_tool_pre_authorized(&permit, started_at_ms);
            let invocation = match prepared {
                Ok(prepared) => {
                    let server_id = prepared.server_id().to_owned();
                    if cancellations
                        .register(
                            task_ticket.clone(),
                            server_id.clone(),
                            task_cancellation.clone(),
                        )
                        .is_err()
                    {
                        task_cancellation.cancel();
                    }
                    let parent_guard = sampling_parents.register(&server_id, parent_action);
                    let raw = match parent_guard {
                        Ok(_guard) => prepared.execute(task_cancellation).await,
                        Err(error) => Err(error),
                    };
                    service
                        .lock()
                        .await
                        .finish_tool_pre_authorized(prepared, raw, started_at_ms)
                        .await
                }
                Err(error) => Err(error),
            };
            cancellations.complete(&task_ticket);
            let result = production_mcp_effect_result(
                invocation,
                canonical_target,
                started_at_ms.saturating_add(1),
            );
            let _ = sender.send(result);
        });
        self.jobs
            .insert(ticket_value, ProductionMcpDeferredJob { result: receiver });
        BrokerDeferredStart::Started(ticket)
    }

    fn poll(&mut self, ticket: &BrokerDeferredTicket) -> Option<RuntimeEffectResult> {
        let job = self.jobs.get(ticket.as_str())?;
        match job.result.try_recv() {
            Ok(result) => {
                self.jobs.remove(ticket.as_str());
                self.cancellations.complete(ticket.as_str());
                Some(result)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.jobs.remove(ticket.as_str());
                self.cancellations.complete(ticket.as_str());
                Some(production_mcp_effect_result(
                    Err(mcp::McpError::StateUnavailable),
                    "mcp-tool:unknown".into(),
                    1,
                ))
            }
        }
    }

    fn cancel(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        self.jobs.contains_key(ticket.as_str()) && self.cancellations.cancel_ticket(ticket.as_str())
    }

    fn abandon(&mut self, ticket: &BrokerDeferredTicket) -> bool {
        let existed = self.jobs.remove(ticket.as_str()).is_some();
        let cancelled = self.cancellations.cancel_ticket(ticket.as_str());
        self.cancellations.complete(ticket.as_str());
        existed || cancelled
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
impl<R, A, F> Drop for ProductionMcpDeferredFacility<R, A, F>
where
    R: mcp::service::McpRepository,
    A: mcp::service::McpAuthority,
    F: mcp::transport::McpTransportFactory,
{
    fn drop(&mut self) {
        for ticket in self.jobs.keys() {
            let _ = self.cancellations.cancel_ticket(ticket);
            self.cancellations.complete(ticket);
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn production_mcp_effect_result(
    invocation: Result<mcp::McpInvocationSnapshot, mcp::McpError>,
    canonical_target: String,
    fallback_completed_at_ms: u64,
) -> RuntimeEffectResult {
    match invocation {
        Ok(invocation) => {
            let mut payload =
                runtime::opencode_sdk::sanitize_broker_result_payload(invocation.redacted_content);
            let mut encoded = serde_json::to_vec(&payload).unwrap_or_default();
            let broker_output_limited = encoded.is_empty()
                || encoded.len() > runtime::action_bridge::MAX_BROKER_MODEL_PAYLOAD_BYTES;
            if broker_output_limited {
                payload = serde_json::json!({
                    "error": { "code": "broker_output_limit_exceeded" }
                });
                encoded = serde_json::to_vec(&payload).unwrap_or_default();
            }
            let output_sha256 = sha256_bytes(&encoded);
            let result_code = if broker_output_limited {
                "mcp-tool-output-limited"
            } else {
                match invocation.status {
                    mcp::McpInvocationStatus::Succeeded => "mcp-tool-succeeded",
                    mcp::McpInvocationStatus::Failed => "mcp-tool-failed",
                    mcp::McpInvocationStatus::Cancelled => "mcp-tool-cancelled",
                    mcp::McpInvocationStatus::TimedOut => "mcp-tool-timed-out",
                    mcp::McpInvocationStatus::OutputLimitExceeded => "mcp-tool-output-limited",
                    mcp::McpInvocationStatus::Denied => "mcp-tool-denied",
                    mcp::McpInvocationStatus::Stale => "mcp-tool-stale",
                }
            };
            let uncertain = matches!(
                invocation.status,
                mcp::McpInvocationStatus::Cancelled | mcp::McpInvocationStatus::TimedOut
            );
            RuntimeEffectResult::with_certainty(
                NormalizedActionResult {
                    status: if uncertain {
                        NormalizedActionStatus::UnknownAfterInterruption
                    } else if invocation.status == mcp::McpInvocationStatus::Succeeded
                        && !broker_output_limited
                    {
                        NormalizedActionStatus::Succeeded
                    } else {
                        NormalizedActionStatus::Failed
                    },
                    result_code: result_code.into(),
                    exit_code: None,
                    changed_targets: vec![canonical_target],
                    output_sha256: Some(output_sha256),
                    completed_at_ms: invocation.completed_at_ms.max(1),
                },
                Some(payload),
                if uncertain {
                    RuntimeEffectCompletionCertainty::Unknown
                } else {
                    RuntimeEffectCompletionCertainty::Completed
                },
            )
        }
        Err(error) => {
            let (code, certainty) = match error {
                mcp::McpError::TimedOut => (
                    "mcp-tool-timed-out",
                    RuntimeEffectCompletionCertainty::Unknown,
                ),
                mcp::McpError::Cancelled
                | mcp::McpError::Transport(_)
                | mcp::McpError::Persistence(_)
                | mcp::McpError::StateUnavailable => (
                    "mcp-tool-status-unknown",
                    RuntimeEffectCompletionCertainty::Unknown,
                ),
                _ => (
                    "mcp-tool-rejected",
                    RuntimeEffectCompletionCertainty::ProvenNotCompleted,
                ),
            };
            let payload = serde_json::json!({ "error": { "code": code } });
            let encoded = serde_json::to_vec(&payload).unwrap_or_default();
            RuntimeEffectResult::with_certainty(
                NormalizedActionResult {
                    status: if certainty == RuntimeEffectCompletionCertainty::Unknown {
                        NormalizedActionStatus::UnknownAfterInterruption
                    } else {
                        NormalizedActionStatus::Failed
                    },
                    result_code: code.into(),
                    exit_code: None,
                    changed_targets: Vec::new(),
                    output_sha256: Some(sha256_bytes(&encoded)),
                    completed_at_ms: fallback_completed_at_ms.max(1),
                },
                Some(payload),
                certainty,
            )
        }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn install_production_broker_facilities(
    bootstrap: &runtime::production::RuntimeProductionBootstrap,
    app: &tauri::AppHandle,
    mcp: Arc<tokio::sync::Mutex<ProductionMcpService>>,
    mcp_cancellations: ProductionMcpCancellationRegistry,
    sampling_parents: mcp::production_sampling::McpSamplingParentRegistry,
) -> Result<(), runtime::production::RuntimeProductionError> {
    bootstrap.install_deferred_broker_facility(Box::new(ProductionMcpDeferredFacility::new(
        mcp,
        mcp_cancellations,
        sampling_parents,
    )))?;
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn initialize_extension_hook_supervisor(
    c4os_home: &Path,
    resource_root: &Path,
) -> Result<extension::hook::HookSupervisor, std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let scratch_root = c4os_home.join("extensions/hook-scratch");
    if scratch_root.exists() {
        let metadata = std::fs::symlink_metadata(&scratch_root)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(std::io::Error::other(
                "extension hook scratch root is not a private directory",
            ));
        }
    } else {
        std::fs::create_dir(&scratch_root)?;
    }
    std::fs::set_permissions(&scratch_root, std::fs::Permissions::from_mode(0o700))?;
    let trusted_runtime = runtime::production::verified_node_runtime_executable(resource_root)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    extension::hook::HookSupervisor::new(extension::hook::HookSupervisorPolicy {
        trusted_runtime,
        runtime_read_roots: vec![resource_root.to_path_buf()],
        scratch_root,
    })
    .map_err(|error| std::io::Error::other(error.to_string()))
}

type ProductionMcpService = mcp::service::McpService<
    mcp::database::DatabaseMcpRepository,
    mcp::authority::ProductionMcpAuthority,
    mcp::transport::RmcpTransportFactory,
>;

struct ProductionMcpCredentialObserver {
    service: Weak<tokio::sync::Mutex<ProductionMcpService>>,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    cancellations: ProductionMcpCancellationRegistry,
}

impl security::credentials::CredentialMutationObserver for ProductionMcpCredentialObserver {
    fn credential_mutated(
        &self,
        credential_reference: &security::credentials::CredentialReference,
        kind: security::credentials::CredentialMutationKind,
    ) {
        let Some(service) = self.service.upgrade() else {
            return;
        };
        let credential_reference = credential_reference.to_string();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        self.cancellations.cancel_credential(&credential_reference);
        let reason = match kind {
            security::credentials::CredentialMutationKind::Replaced => "credential_replaced",
            security::credentials::CredentialMutationKind::Removed => "credential_removed",
        };
        tauri::async_runtime::spawn(async move {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|duration| duration.as_millis().try_into().ok())
                .unwrap_or(1);
            let _ = service
                .lock()
                .await
                .invalidate_credential_reference(&credential_reference, reason, now_ms)
                .await;
        });
    }
}

struct AppCoreState {
    database: Arc<core::database::DatabaseActor>,
    c4os_home: PathBuf,
    bundled_skill_root: PathBuf,
    mcp: Arc<tokio::sync::Mutex<ProductionMcpService>>,
    _mcp_credential_observer: Option<Arc<dyn security::credentials::CredentialMutationObserver>>,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    mcp_cancellations: ProductionMcpCancellationRegistry,
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    mcp_sampling_approvals: mcp::production_sampling::ProductionMcpSamplingApprovalRegistry,
    extensions: Mutex<extension::service::ExtensionService>,
    hook_supervisor: Mutex<Option<extension::hook::HookSupervisor>>,
    configuration: Mutex<core::services::ManagedAppConfiguration>,
    active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    conversation_operation: Mutex<()>,
    artifact_operation: Arc<Mutex<()>>,
    conversation: Mutex<ConversationApplicationState>,
    artifact: Arc<Mutex<ArtifactApplicationState>>,
    terminal: Arc<Mutex<TerminalSupervisor>>,
    browser_profiles: Mutex<browser::profile::BrowserProfileRegistry>,
    browser_events: Arc<Mutex<browser::native::NativeBrowserEventQueue>>,
    _terminal_reconciliation: TerminalReconciliationDriver,
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

#[derive(Default)]
struct ArtifactApplicationState {
    pending_writes: BTreeMap<String, PendingArtifactWrite>,
    pending_terminal_actions: BTreeMap<String, PendingArtifactTerminalAction>,
    pending_browser_actions: BTreeMap<String, PendingArtifactBrowserAction>,
    browser_native_requests: BTreeMap<String, PendingNativeBrowserRequest>,
    browser_ephemeral_clear_operations: BTreeMap<String, String>,
    browser_targets: BTreeMap<String, BrowserTransientTarget>,
    browser_pending_navigation: BTreeMap<String, browser::native::NativeBrowserAction>,
    browser_mount_generations: BTreeMap<String, u64>,
    browser_mounted_artifacts: BTreeSet<String>,
    browser_active_identity: Option<browser::native::NativeBrowserIdentity>,
    browser_notices: BTreeMap<String, Vec<ArtifactBrowserNoticeSnapshot>>,
    terminal_event_cursors: BTreeMap<(String, String, u64), u64>,
    terminal_ack_cursors: BTreeMap<String, u64>,
    terminal_output_lines: BTreeMap<String, Vec<u8>>,
    terminal_redacted_output_lines: BTreeSet<String>,
    terminal_sensitive_output_lines: BTreeSet<String>,
    terminal_cleanup_sessions: BTreeMap<String, String>,
}

#[derive(Clone)]
struct BrowserTransientTarget {
    target: artifact::BrowserNavigationTarget,
}

#[derive(Clone)]
struct PendingNativeBrowserRequest {
    artifact_id: String,
    record_revision: u64,
    scope: ActiveArtifactScope,
    controller_generation: u64,
    mount_generation: u64,
    display_url: String,
    navigation_sha256: String,
    observed_at_ms: u64,
}

#[derive(Clone)]
struct PendingPersistentBrowserClear {
    scope: browser::profile::PersistentProfileScope,
    profile_id: String,
    registry_generation: u64,
    data_generation: u64,
}

#[derive(Clone)]
struct PendingArtifactBrowserAction {
    artifact_id: String,
    record_revision: u64,
    scope: ActiveArtifactScope,
    action: CanonicalAction,
    live: LiveAuthorityState,
    payload: PendingArtifactBrowserPayload,
}

#[derive(Clone)]
enum PendingArtifactBrowserPayload {
    Open {
        target: artifact::BrowserNavigationTarget,
    },
    Navigate {
        intent: artifact::BrowserNavigationIntent,
        navigation_sha256: String,
        controller_generation: u64,
        mount_generation: u64,
    },
    NavigateTo {
        target: artifact::BrowserNavigationTarget,
        controller_generation: u64,
        mount_generation: u64,
    },
    Recover {
        target: artifact::BrowserNavigationTarget,
        controller_generation: u64,
        mount_generation: u64,
    },
    NativeRequest {
        request_id: String,
        controller_generation: u64,
        mount_generation: u64,
        navigation_sha256: String,
    },
    ClearData {
        environment: artifact::BrowserEnvironmentReference,
        persistent: Option<PendingPersistentBrowserClear>,
        operation_id: String,
        controller_generation: u64,
        mount_generation: u64,
    },
}

#[derive(Clone)]
struct TerminalOutputRedactionCheckpoint {
    pending: Option<Vec<u8>>,
    oversized: bool,
    sensitive: bool,
}

struct TerminalReconciliationDriver {
    stop: Option<std::sync::mpsc::Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

impl Drop for TerminalReconciliationDriver {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

trait TerminalReconciliationResources {
    fn artifact_state(&self) -> &Mutex<ArtifactApplicationState>;
    fn terminal_supervisor(&self) -> &Mutex<TerminalSupervisor>;
    fn runtime_service(&self) -> &RuntimeApplicationService;
}

impl TerminalReconciliationResources for AppCoreState {
    fn artifact_state(&self) -> &Mutex<ArtifactApplicationState> {
        &self.artifact
    }

    fn terminal_supervisor(&self) -> &Mutex<TerminalSupervisor> {
        &self.terminal
    }

    fn runtime_service(&self) -> &RuntimeApplicationService {
        &self.runtime
    }
}

#[derive(Clone)]
struct BackgroundTerminalReconciliation {
    artifact: Arc<Mutex<ArtifactApplicationState>>,
    terminal: Arc<Mutex<TerminalSupervisor>>,
    runtime: Arc<RuntimeApplicationService>,
}

impl TerminalReconciliationResources for BackgroundTerminalReconciliation {
    fn artifact_state(&self) -> &Mutex<ArtifactApplicationState> {
        &self.artifact
    }

    fn terminal_supervisor(&self) -> &Mutex<TerminalSupervisor> {
        &self.terminal
    }

    fn runtime_service(&self) -> &RuntimeApplicationService {
        &self.runtime
    }
}

#[derive(Clone)]
struct PendingArtifactTerminalAction {
    artifact_id: String,
    record_revision: u64,
    scope: ActiveArtifactScope,
    action: CanonicalAction,
    live: LiveAuthorityState,
    payload: PendingArtifactTerminalPayload,
}

#[derive(Clone)]
enum PendingArtifactTerminalPayload {
    Run {
        terminal_session_id: String,
        command_id: String,
        command_sequence: u64,
        command: String,
        shell_path: String,
        environment: execution::environment::ExecutionEnvironmentIdentity,
        process_generation: u64,
        columns: u16,
        rows: u16,
    },
    Stdin {
        process_generation: u64,
        text: String,
    },
    Resize {
        process_generation: u64,
        columns: u16,
        rows: u16,
    },
    Stop {
        process_generation: u64,
    },
}

#[derive(Clone)]
struct PendingArtifactWrite {
    artifact_id: String,
    record_revision: u64,
    project_filesystem: ProjectFilesystem,
    relative_path: PathBuf,
    expected_file_version: ProjectFileVersion,
    content: String,
    action: CanonicalAction,
    live: LiveAuthorityState,
}

struct ArtifactWritePreparation {
    project_version: ProjectFileVersion,
    relative_path: PathBuf,
    content: String,
    request_origin: ActionRequestOrigin,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
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
    #[serde(default)]
    artifact_reply_capture: Option<DurableArtifactReplyCapture>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DurableArtifactReplyCapture {
    artifact_id: String,
    artifact_record_revision: u64,
    resource_version: ArtifactResourceVersionSnapshot,
    selected_text: Option<String>,
    selected_entry_id: Option<String>,
    #[serde(default)]
    browser_page_context: Option<DurableBrowserPageContext>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DurableBrowserPageContext {
    selected_text: Option<String>,
    visible_text: Option<String>,
    extracted_content: Option<String>,
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
            artifact_reply_capture: None,
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
            && draft.artifact_reply_capture.as_ref().is_none_or(|capture| {
                draft.reply_target_id.as_deref() == Some(capture.artifact_id.as_str())
                    && protocol::ArtifactId::new(capture.artifact_id.clone()).is_ok()
                    && capture.artifact_record_revision > 0
                    && capture.resource_version.sequence > 0
                    && capture.resource_version.observed_at_ms > 0
                    && capture
                        .resource_version
                        .sha256
                        .strip_prefix("sha256:")
                        .is_some_and(|hex| {
                            hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
                        })
                    && capture.selected_text.as_ref().is_none_or(|value| {
                        !value.is_empty()
                            && value.len() <= protocol::MAX_TEXT_BYTES
                            && !value.contains('\0')
                    })
                    && capture
                        .selected_entry_id
                        .as_ref()
                        .is_none_or(|value| !value.trim().is_empty() && value.len() <= 512)
                    && !(capture.selected_text.is_some() && capture.selected_entry_id.is_some())
                    && capture.browser_page_context.as_ref().is_none_or(|page| {
                        capture.selected_text.is_none()
                            && capture.selected_entry_id.is_none()
                            && [
                                page.selected_text.as_ref(),
                                page.visible_text.as_ref(),
                                page.extracted_content.as_ref(),
                            ]
                            .into_iter()
                            .flatten()
                            .all(|value| {
                                !value.trim().is_empty()
                                    && value.len()
                                        <= browser::native::MAX_NATIVE_BROWSER_CONTEXT_FIELD_BYTES
                                    && !value.contains('\0')
                            })
                    })
            })
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
    approval_kind: String,
    summary: String,
    server_id: Option<String>,
    provider_id: Option<String>,
    model_id: Option<String>,
    max_tokens: Option<u32>,
    expires_at_ms: Option<u64>,
    message_count: Option<usize>,
    input_bytes: Option<usize>,
    has_system_prompt: Option<bool>,
    parent_operation: Option<String>,
    disclosure_scope: Option<String>,
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

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[derive(Clone)]
pub(crate) struct RuntimeMcpSamplingIntent {
    pub context: mcp::transport::McpSamplingContext,
    pub parent_action: CanonicalAction,
    pub messages: Vec<PiSamplingMessage>,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub request_sha256: String,
    pub sampling_id: String,
    pub cancelled: Arc<AtomicBool>,
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub(crate) struct PreparedRuntimeMcpSampling {
    pub facts: ActionFacts,
    pub action: CanonicalAction,
    pub live: LiveAuthorityState,
    pub peer: PeerSamplingRequest,
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
    pub skill_context: Vec<SkillContextSnapshot>,
    pub mcp_turn: Option<mcp::McpTurnSnapshot>,
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
    pub skill_context: Vec<SkillContextSnapshot>,
    pub mcp_turn: Option<mcp::McpTurnSnapshot>,
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
    pub skill_context: Vec<SkillContextSnapshot>,
    pub mcp_turn: Option<mcp::McpTurnSnapshot>,
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
    pub skill_context: Vec<SkillContextSnapshot>,
    pub mcp_turn: Option<mcp::McpTurnSnapshot>,
    pub reply_context: Option<MessageReplyContextSnapshot>,
    pub draft: DraftRequirements,
    pub submitted_at_ms: u64,
    pub preflight_at_ms: u64,
}

enum ConversationSubmissionDispatch {
    First(Box<ConversationFirstDispatchIntent>),
    Turn(Box<ConversationTurnDispatchIntent>),
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

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn prepare_mcp_sampling(
        &self,
        intent: &RuntimeMcpSamplingIntent,
        now_ms: u64,
    ) -> Result<PreparedRuntimeMcpSampling, RuntimeApplicationError> {
        let coordinator = self.coordinator()?;
        let session = coordinator.session(&intent.parent_action.session_id)?;
        if session.active_attempt_id.as_deref() != Some(intent.parent_action.run_id.as_str()) {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let attempt = session
            .attempt(&intent.parent_action.run_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let binding = session
            .binding()
            .cloned()
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if attempt.status.is_terminal()
            || attempt.context.runtime_kind != runtime::session::RuntimeKind::Pi
            || attempt.context.workspace_id != intent.parent_action.workspace_id
            || attempt.context.runtime_id != intent.parent_action.runtime_id
            || attempt.context.environment.environment_id != intent.parent_action.environment_id
            || attempt.process_generation != intent.parent_action.process_generation
            || attempt.correlation_id.is_empty()
            || intent.context.authority_id
                != intent
                    .parent_action
                    .plugin_or_mcp_id
                    .clone()
                    .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let snapshot = coordinator.snapshot(now_ms);
        let runtime_record = snapshot
            .runtimes
            .records
            .iter()
            .find(|record| {
                record.installation.runtime_id == attempt.context.runtime_id
                    && record.process_generation == attempt.process_generation
                    && record.lifecycle == runtime::supervisor::RuntimeLifecycle::Ready
                    && record.health == HealthState::Healthy
            })
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let provider = snapshot
            .providers
            .providers
            .iter()
            .find(|record| record.profile.provider_id == attempt.context.model_route.provider_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let selected_model_id = provider
            .selected_model_id
            .as_deref()
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        let selected_model = provider
            .models
            .get(selected_model_id)
            .filter(|model| model.is_production_ready_at(now_ms))
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if selected_model.capabilities.route.provider_model_id
            != attempt.context.model_route.model_id
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let capabilities = self.capabilities()?;
        let route = capability_route_for_runtime(
            &coordinator,
            &capabilities,
            &runtime_record.installation.runtime_id,
            &provider.profile.provider_id,
            selected_model_id,
            now_ms,
        )?;
        let layers = capabilities.layers(&route, now_ms)?;
        let effective =
            effective_intersection(&layers, now_ms).map_err(CoordinatorError::Capability)?;
        if effective.route.provider_id != attempt.context.model_route.provider_id
            || effective.route.endpoint_id != attempt.context.model_route.endpoint_id
            || effective.route.provider_model_id != attempt.context.model_route.model_id
            || effective.route.model_revision != attempt.context.model_route.model_revision
            || effective
                .numeric_maximum(NumericCapabilityKey::OutputTokens)
                .is_none_or(|maximum| u64::from(intent.max_tokens) > maximum)
            || (intent.temperature.is_some()
                && !effective.feature_state(CapabilityKey::Temperature).usable())
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let authority = AuthorityMintIntent {
            workspace_id: attempt.context.workspace_id.clone(),
            project_id: attempt.context.project_id.clone(),
            runtime_id: attempt.context.runtime_id.clone(),
            expected_authority_generation: self.dispatch_authority.generation()?,
            expected_capability_generation: capabilities.generation(),
        };
        let process = {
            let dispatch = self.dispatch()?;
            runtime_process_truth(&dispatch.registration(&attempt.context.runtime_id)?)
        };
        let minted = self
            .dispatch_authority
            .mint_retry_context_for_active_project(
                &authority,
                &binding,
                &effective,
                capabilities.generation(),
                &process,
            )?;
        if minted.context.workspace_id != attempt.context.workspace_id
            || minted.context.project_id != attempt.context.project_id
            || minted.context.runtime_id != attempt.context.runtime_id
            || minted.context.runtime_kind != attempt.context.runtime_kind
            || minted.context.adapter != attempt.context.adapter
            || minted.context.environment != attempt.context.environment
            || minted.context.model_route != attempt.context.model_route
            || minted.context.configuration != attempt.context.configuration
            || minted.context.resources != attempt.context.resources
            || minted.context.capabilities.version != attempt.context.capabilities.version
            || minted.process_generation != attempt.process_generation
        {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let installed_resources = self
            .dispatch_authority
            .installed_resource_preflight_for_active_project(&authority, &effective)?;
        let policy = self
            .policy_authority
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        let estimated_input_tokens = intent
            .messages
            .iter()
            .map(|message| message.text.chars().count() as u64)
            .sum::<u64>()
            .saturating_add(
                intent
                    .system_prompt
                    .as_deref()
                    .map_or(0, |value| value.chars().count() as u64),
            )
            .div_ceil(4)
            .max(1);
        let draft = DraftRequirements {
            attachments: Vec::new(),
            reasoning_mode: None,
            requires_tools: false,
            requires_json_schema: false,
            prefers_streaming: false,
            estimated_input_tokens,
            requested_output_tokens: u64::from(intent.max_tokens),
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
        let preflight = coordinator.model_preflight(
            &provider.profile.provider_id,
            selected_model_id,
            &layers,
            &draft,
            now_ms,
        )?;
        if !matches!(preflight.outcome, PreflightOutcome::Ready { .. }) {
            return Err(RuntimeApplicationError::InvalidManagedRuntimeBinding);
        }
        let arguments = serde_json::json!({
            "requestSha256": intent.request_sha256,
            "serverId": intent.context.server_id,
            "authorityId": intent.context.authority_id,
            "definitionSha256": intent.context.definition_sha256,
            "lifecycleGeneration": intent.context.lifecycle_generation,
            "parentRunId": intent.parent_action.run_id,
            "providerId": provider.profile.provider_id,
            "modelId": effective.route.provider_model_id,
            "modelRevision": effective.route.model_revision,
            "capabilitySha256": effective.raw_evidence_sha256,
            "maxTokens": intent.max_tokens,
            "temperature": intent.temperature,
        });
        let target_version = sha256_bytes(
            &serde_json::to_vec(&arguments)
                .map_err(|_| RuntimeApplicationError::InvalidManagedRuntimeBinding)?,
        );
        let canonical_target = format!(
            "mcp-sampling:{}:{}:{}",
            intent.context.authority_id,
            provider.profile.provider_id,
            effective.route.provider_model_id
        );
        let action = CanonicalAction {
            schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
            action_id: format!("mcp-sampling-{}", intent.sampling_id),
            tool_call_id: format!("mcp-sampling-call-{}", intent.sampling_id),
            tool: "mcp.sampling.create-message".into(),
            arguments,
            risk: CanonicalRisk::High,
            requested_authority: BTreeSet::from([
                "network.execute".into(),
                "network.publish".into(),
            ]),
            canonical_target: canonical_target.clone(),
            target_version,
            workspace_id: attempt.context.workspace_id.clone(),
            session_id: intent.parent_action.session_id.clone(),
            run_id: format!("mcp-sampling-run-{}", intent.sampling_id),
            runtime_id: attempt.context.runtime_id.clone(),
            environment_id: attempt.context.environment.environment_id.clone(),
            plugin_or_mcp_id: Some(intent.context.authority_id.clone()),
            process_generation: attempt.process_generation,
            configuration_version: attempt.context.configuration.version,
            policy_version: policy.policy_version,
            revocation_epoch: policy.revocation_epoch,
        };
        let facts = ActionFacts {
            action_kind: "mcp.sampling.create-message".into(),
            native_tool: action.tool.clone(),
            surface: ActionSurface::Network,
            effects: BTreeSet::from([ActionEffect::Execute, ActionEffect::Publish]),
            scope: ActionScope::Remote,
            initiator: ActionInitiator::McpServer,
            sensitivity: ActionSensitivity::Private,
            reversibility: ActionReversibility::Reversible,
            confidence: ClassificationConfidence::Known,
            request_origin: ActionRequestOrigin::McpSampling,
            repository_state: RepositoryState::NotApplicable,
            inside_active_project: true,
            canonical_target,
            workspace_id: action.workspace_id.clone(),
            session_id: action.session_id.clone(),
            runtime_id: action.runtime_id.clone(),
            environment_id: action.environment_id.clone(),
            plugin_or_mcp_id: action.plugin_or_mcp_id.clone(),
            target_resolved: true,
            authenticated: true,
            trusted_root: true,
            explicit_scope_grant: true,
            sandbox_allows: true,
            declaration_exceeded: false,
        };
        let live = LiveAuthorityState {
            process_generation: action.process_generation,
            configuration_version: action.configuration_version,
            policy_version: policy.policy_version,
            revocation_epoch: policy.revocation_epoch,
        };
        let peer = PeerSamplingRequest {
            identity: DispatchIdentity {
                workspace_id: action.workspace_id.clone(),
                environment_id: action.environment_id.clone(),
                session_id: format!("mcp-sampling-session-{}", intent.sampling_id),
                turn_id: format!("mcp-sampling-turn-{}", intent.sampling_id),
                attempt_id: format!("mcp-sampling-attempt-{}", intent.sampling_id),
                correlation_id: format!("mcp-sampling-correlation-{}", intent.sampling_id),
                runtime_id: action.runtime_id.clone(),
                runtime_kind: runtime::supervisor::RuntimeKind::Pi,
                adapter_version: attempt.context.adapter.adapter_version.clone(),
                native_version: attempt.context.adapter.native_version.clone(),
                process_generation: attempt.process_generation,
            },
            model: runtime::dispatch::DispatchModelRoute {
                provider_id: provider.profile.provider_id.clone(),
                model_id: effective.route.provider_model_id.clone(),
                credential_reference: None,
                credential_lease_id: None,
            },
            messages: intent.messages.clone(),
            system_prompt: intent.system_prompt.clone(),
            max_tokens: intent.max_tokens,
            temperature: intent.temperature,
            timeout_ms: intent.context.timeout_ms,
            cancelled: Arc::clone(&intent.cancelled),
        };
        Ok(PreparedRuntimeMcpSampling {
            facts,
            action,
            live,
            peer,
        })
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn execute_mcp_sampling(
        &self,
        request: &PeerSamplingRequest,
    ) -> Result<PeerSamplingResult, RuntimeApplicationError> {
        Ok(self.dispatch()?.sample(request)?)
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

    pub fn revoke_plugin_authority(
        &self,
        plugin_id: &str,
        now_ms: u64,
    ) -> Result<usize, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        Ok(coordinator
            .revoke_plugin_authority(plugin_id, now_ms)?
            .value)
    }

    pub fn revoke_plugin_or_mcp_authority(
        &self,
        identity: &str,
        now_ms: u64,
    ) -> Result<usize, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        Ok(coordinator
            .revoke_plugin_or_mcp_authority(identity, now_ms)?
            .value)
    }

    pub(crate) fn propose_direct_action(
        &self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .propose_direct_action(facts, action, now_ms)?
            .value)
    }

    pub(crate) fn propose_direct_trust_confirmation(
        &self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .propose_direct_trust_confirmation(facts, action, now_ms)?
            .value)
    }

    pub(crate) fn propose_direct_sampling_confirmation(
        &self,
        facts: &ActionFacts,
        action: CanonicalAction,
        now_ms: u64,
    ) -> Result<GatewayProposal, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .propose_direct_sampling_confirmation(facts, action, now_ms)?
            .value)
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub(crate) fn answer_direct_approval_expected(
        &self,
        expected_coordinator_generation: u64,
        prompt_id: &str,
        answer: ApprovalAnswer,
        now_ms: u64,
    ) -> Result<ApprovalResponse, RuntimeApplicationError> {
        let mut coordinator = self.coordinator()?;
        require_coordinator_generation(&coordinator, expected_coordinator_generation, now_ms)?;
        Ok(coordinator
            .answer_direct_approval(prompt_id, answer, now_ms)?
            .value)
    }

    pub(crate) fn cancel_direct_action_run(
        &self,
        run_id: &str,
        now_ms: u64,
    ) -> Result<usize, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .cancel_direct_action_run(run_id, now_ms)?
            .value)
    }

    pub(crate) fn begin_direct_action_effect(
        &self,
        token: &AuthorizationToken,
        action: &CanonicalAction,
        live: LiveAuthorityState,
        approval_prompt_id: Option<&str>,
        now_ms: u64,
    ) -> Result<ActionEffectLease, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .begin_direct_action_effect(token, action, live, approval_prompt_id, now_ms)?
            .value)
    }

    pub(crate) fn complete_direct_action_effect(
        &self,
        lease: ActionEffectLease,
        result: NormalizedActionResult,
    ) -> Result<NormalizedActionResult, RuntimeApplicationError> {
        Ok(self
            .coordinator()?
            .complete_direct_action_effect(lease, result)?
            .value)
    }

    pub(crate) fn complete_direct_action_effect_retryable(
        &self,
        lease: &mut ActionEffectLease,
        result: &NormalizedActionResult,
    ) -> Result<(), RuntimeApplicationError> {
        self.coordinator()?
            .complete_direct_action_effect_retryable(lease, result)?;
        Ok(())
    }

    pub(crate) fn current_direct_live_authority(
        &self,
        process_generation: u64,
        configuration_version: u64,
    ) -> Result<LiveAuthorityState, RuntimeApplicationError> {
        if process_generation == 0 || configuration_version == 0 {
            return Err(RuntimeApplicationError::InvalidPolicyAuthority);
        }
        let authority = self
            .policy_authority
            .lock()
            .map_err(|_| RuntimeApplicationError::Unavailable)?;
        Ok(LiveAuthorityState {
            process_generation,
            configuration_version,
            policy_version: authority.policy_version,
            revocation_epoch: authority.revocation_epoch,
        })
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
                skill_context: intent.skill_context,
                mcp_turn: intent.mcp_turn.filter(|_| {
                    draft
                        .installed_resources
                        .tool_ids
                        .contains(runtime::opencode::C4OS_ACTION_PROPOSAL_TOOL)
                }),
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
                skill_context: intent.skill_context,
                mcp_turn: intent.mcp_turn.filter(|_| {
                    draft
                        .installed_resources
                        .tool_ids
                        .contains(runtime::opencode::C4OS_ACTION_PROPOSAL_TOOL)
                }),
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
                skill_context: intent.skill_context,
                mcp_turn: intent.mcp_turn.clone(),
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
                skill_context: intent.skill_context,
                reply_context: intent.reply_context,
                mcp_turn: intent.mcp_turn.clone(),
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
        let turn = record
            .turn(&attempt.turn_id)
            .ok_or(RuntimeApplicationError::InvalidManagedRuntimeBinding)?;
        if turn.mcp_turn.as_ref().is_some_and(|snapshot| {
            snapshot.workspace_id != attempt.context.workspace_id
                || snapshot.project_id != attempt.context.project_id.as_deref().unwrap_or_default()
                || snapshot.session_id != identity.session_id
        }) {
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
            mcp_turn: turn.mcp_turn.clone(),
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

    fn begin_runtime_action_effect(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<runtime::action_bridge::RuntimeActionEffectLease, Self::Error> {
        Ok(self
            .coordinator
            .begin_runtime_action_effect(authorization, live, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect(
        &mut self,
        lease: runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator
            .complete_runtime_action_effect(lease, result, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect_retryable(
        &mut self,
        lease: &mut runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator
            .complete_runtime_action_effect_retryable(lease, result, now_ms)?
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

    fn begin_runtime_action_effect(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<runtime::action_bridge::RuntimeActionEffectLease, Self::Error> {
        Ok(self
            .coordinator()?
            .begin_runtime_action_effect(authorization, live, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect(
        &mut self,
        lease: runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator()?
            .complete_runtime_action_effect(lease, result, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect_retryable(
        &mut self,
        lease: &mut runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator()?
            .complete_runtime_action_effect_retryable(lease, result, now_ms)?
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

    fn begin_runtime_action_effect(
        &mut self,
        authorization: RuntimeAuthorization,
        live: LiveAuthorityState,
        now_ms: u64,
    ) -> Result<runtime::action_bridge::RuntimeActionEffectLease, Self::Error> {
        Ok(self
            .coordinator()?
            .begin_runtime_action_effect(authorization, live, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect(
        &mut self,
        lease: runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator()?
            .complete_runtime_action_effect(lease, result, now_ms)?
            .value)
    }

    fn complete_runtime_action_effect_retryable(
        &mut self,
        lease: &mut runtime::action_bridge::RuntimeActionEffectLease,
        result: runtime::action_bridge::RuntimeEffectResult,
        now_ms: u64,
    ) -> Result<RuntimeExecutionReceipt, Self::Error> {
        Ok(self
            .coordinator()?
            .complete_runtime_action_effect_retryable(lease, result, now_ms)?
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

fn extension_boundary_error(
    error: extension::ExtensionError,
    correlation_id: protocol::CorrelationId,
) -> ProtocolError {
    let (code, message, retryable) = match error {
        extension::ExtensionError::InvalidInput | extension::ExtensionError::BoundExceeded => (
            ProtocolErrorCode::InvalidPayload,
            "The Extension request exceeds its bound",
            false,
        ),
        extension::ExtensionError::Conflict => (
            ProtocolErrorCode::Conflict,
            "Extension state changed before the operation completed",
            true,
        ),
        extension::ExtensionError::MutableContent
        | extension::ExtensionError::VerificationFailed
        | extension::ExtensionError::UntrustedOrigin
        | extension::ExtensionError::Revoked
        | extension::ExtensionError::Incompatible
        | extension::ExtensionError::HookDenied => (
            ProtocolErrorCode::Forbidden,
            "The Extension failed its trust or policy checks",
            false,
        ),
        extension::ExtensionError::UnsupportedHookTarget => (
            ProtocolErrorCode::Unavailable,
            "Extension hooks are unavailable on this target",
            false,
        ),
        extension::ExtensionError::HookTimedOut
        | extension::ExtensionError::HookOutputExceeded
        | extension::ExtensionError::InvalidState
        | extension::ExtensionError::Unavailable
        | extension::ExtensionError::Database(_)
        | extension::ExtensionError::Io(_)
        | extension::ExtensionError::Toml(_)
        | extension::ExtensionError::Json(_) => (
            ProtocolErrorCode::Unavailable,
            "Extension state is unavailable",
            true,
        ),
    };
    platform_boundary_error(correlation_id, code, message, retryable)
}

const MAX_EXTENSION_HOOK_ANNOTATION_BYTES: usize = 4 * 1024;

enum ExtensionHookMediation {
    Applied(SkillContextSnapshot),
    Denied(&'static str),
}

fn mediate_extension_hook_proposal(
    core: &AppCoreState,
    hook: &extension::service::PreparedExtensionHook,
    proposal: &extension::hook::HookEffectProposal,
    workspace_id: &str,
    session_id: &str,
    within_turn_context_bounds: bool,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<ExtensionHookMediation, ProtocolError> {
    let annotation = (proposal.kind == "context.annotation")
        .then(|| proposal.payload.as_object())
        .flatten()
        .filter(|payload| payload.len() == 1)
        .and_then(|payload| payload.get("text"))
        .and_then(serde_json::Value::as_str)
        .filter(|text| {
            !text.trim().is_empty()
                && text.len() <= MAX_EXTENSION_HOOK_ANNOTATION_BYTES
                && !text.contains('\0')
        });
    let declared = hook.grants.iter().any(|grant| grant == &proposal.kind);
    let recognized = annotation.is_some();
    let permitted_shape = recognized && declared && within_turn_context_bounds;
    let payload_sha256 = sha256_bytes(&serde_json::to_vec(&proposal.payload).map_err(|_| {
        extension_boundary_error(
            extension::ExtensionError::HookDenied,
            correlation_id.clone(),
        )
    })?);
    let live = current_artifact_live_authority(core, now_ms, correlation_id.clone())?;
    let action_id = format!("extension-hook-{}", Uuid::new_v4().as_simple());
    let canonical_target = format!(
        "extension:{}:{}:{}",
        hook.activation.package_id, hook.contract.hook_id, hook.activation.package_digest
    );
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: if recognized {
            "c4os.extension.context-annotation".into()
        } else {
            "c4os.extension.unsupported-proposal".into()
        },
        arguments: serde_json::json!({
            "hookId": hook.contract.hook_id,
            "packageDigest": hook.activation.package_digest,
            "payloadSha256": payload_sha256,
            "proposalKind": proposal.kind,
            "summarySha256": sha256_bytes(proposal.summary.as_bytes()),
        }),
        risk: if recognized {
            CanonicalRisk::Low
        } else {
            CanonicalRisk::Unknown
        },
        requested_authority: BTreeSet::from([proposal.kind.clone()]),
        canonical_target: canonical_target.clone(),
        target_version: hook.activation.package_digest.clone(),
        workspace_id: workspace_id.into(),
        session_id: session_id.into(),
        run_id: format!("extension-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: Some(hook.activation.package_id.clone()),
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        extension_boundary_error(
            extension::ExtensionError::HookDenied,
            correlation_id.clone(),
        )
    })?;
    let facts = ActionFacts {
        action_kind: proposal.kind.clone(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::C4os,
        effects: BTreeSet::from([ActionEffect::Control]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Plugin,
        sensitivity: if recognized {
            ActionSensitivity::Ordinary
        } else {
            ActionSensitivity::Unknown
        },
        reversibility: if recognized {
            ActionReversibility::Reversible
        } else {
            ActionReversibility::Unknown
        },
        confidence: if recognized {
            ClassificationConfidence::Known
        } else {
            ClassificationConfidence::Ambiguous
        },
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: if recognized {
            RepositoryState::NotApplicable
        } else {
            RepositoryState::Unknown
        },
        inside_active_project: true,
        canonical_target: canonical_target.clone(),
        workspace_id: workspace_id.into(),
        session_id: session_id.into(),
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: action.plugin_or_mcp_id.clone(),
        target_resolved: recognized,
        authenticated: false,
        trusted_root: true,
        explicit_scope_grant: declared,
        sandbox_allows: true,
        declaration_exceeded: !permitted_shape,
    };
    let proposal_result = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .propose_direct_action(&facts, action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let token = match proposal_result {
        GatewayProposal::Denied { .. } => {
            return Ok(ExtensionHookMediation::Denied(
                "Action Gateway denied the exact proposal",
            ));
        }
        GatewayProposal::PendingApproval { .. } => {
            return Ok(ExtensionHookMediation::Denied(
                "Action Gateway is awaiting explicit approval",
            ));
        }
        GatewayProposal::Authorized { token, .. } => token,
    };
    let annotation = annotation.ok_or_else(|| {
        extension_boundary_error(
            extension::ExtensionError::HookDenied,
            correlation_id.clone(),
        )
    })?;
    let annotation_sha256 = sha256_bytes(annotation.as_bytes());
    let executed = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .execute_direct_action(&token, &action, live, None, now_ms, |_permit| {
                    NormalizedActionResult {
                        status: NormalizedActionStatus::Succeeded,
                        result_code: "extension-context-annotation-applied".into(),
                        exit_code: Some(0),
                        changed_targets: vec![canonical_target.clone()],
                        output_sha256: Some(annotation_sha256.clone()),
                        completed_at_ms: now_ms,
                    }
                })
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    if executed.status != NormalizedActionStatus::Succeeded {
        return Ok(ExtensionHookMediation::Denied(
            "Action Gateway did not consume the proposal",
        ));
    }
    Ok(ExtensionHookMediation::Applied(SkillContextSnapshot {
        identity: format!(
            "plugin:{}:hook:{}:annotation",
            hook.activation.package_id, hook.contract.hook_id
        ),
        package_id: Some(hook.activation.package_id.clone()),
        entrypoint_sha256: annotation_sha256,
        instructions: annotation.into(),
        referenced_resources: Vec::new(),
    }))
}

fn dispatch_reviewed_extension_hooks(
    core: &AppCoreState,
    event: &str,
    payload: serde_json::Value,
    workspace_id: &str,
    session_id: &str,
    available_context_slots: usize,
    available_context_bytes: usize,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<Vec<SkillContextSnapshot>, ProtocolError> {
    let (generation, prepared, supervisor) = {
        let mut extensions = core
            .extensions
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let current_generation = extensions.snapshot().generation;
        let initially_prepared = extensions
            .prepared_hooks(event)
            .map_err(|error| extension_boundary_error(error, correlation_id.clone()))?;
        if initially_prepared.is_empty() {
            return Ok(Vec::new());
        }
        let supervisor = core
            .hook_supervisor
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .clone()
            .ok_or_else(|| {
                extension_boundary_error(
                    extension::ExtensionError::UnsupportedHookTarget,
                    correlation_id.clone(),
                )
            })?;
        let package_ids = initially_prepared
            .iter()
            .map(|hook| hook.activation.package_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let execution = extensions
            .begin_hook_execution(current_generation, &package_ids, now_ms)
            .map_err(|error| extension_boundary_error(error, correlation_id.clone()))?;
        let prepared = match extensions.prepared_hooks(event) {
            Ok(prepared) => prepared,
            Err(error) => {
                let records = initially_prepared
                    .iter()
                    .map(|hook| extension::service::ExtensionHookExecutionRecord {
                        package_id: hook.activation.package_id.clone(),
                        hook_id: hook.contract.hook_id.clone(),
                        succeeded: false,
                        detail: "execution contract could not be rebound after durable start"
                            .into(),
                    })
                    .collect::<Vec<_>>();
                extensions
                    .record_hook_execution_batch(execution.generation, &records, now_ms)
                    .map_err(|record_error| {
                        extension_boundary_error(record_error, correlation_id.clone())
                    })?;
                return Err(extension_boundary_error(error, correlation_id));
            }
        };
        (execution.generation, prepared, supervisor)
    };
    let mut records = Vec::with_capacity(prepared.len());
    let mut context = Vec::new();
    let mut context_bytes = 0usize;
    let mut denied = false;
    let mut mediation_error = None;
    for hook in prepared {
        let envelope = extension::hook::HookEventEnvelope {
            protocol_version: extension::EXTENSION_HOOK_PROTOCOL_VERSION,
            operation_id: format!("hook-{}", Uuid::new_v4().as_simple()),
            package_id: hook.activation.package_id.clone(),
            package_digest: hook.activation.package_digest.clone(),
            event: event.into(),
            payload: payload.clone(),
        };
        let outcome = supervisor.run(
            &hook.activation,
            &hook.contract,
            &hook.package_root,
            &envelope,
        );
        let (succeeded, detail) = match outcome {
            Ok(result) if result.proposals.is_empty() => (
                true,
                format!(
                    "completed in {} ms with no effect proposals",
                    result.duration_ms
                ),
            ),
            Ok(result) => {
                let Some(proposal) = result.proposals.first() else {
                    records.push(extension::service::ExtensionHookExecutionRecord {
                        package_id: hook.activation.package_id,
                        hook_id: hook.contract.hook_id,
                        succeeded: false,
                        detail: "worker returned an inconsistent proposal batch".into(),
                    });
                    denied = true;
                    continue;
                };
                let proposed_bytes = proposal
                    .payload
                    .get("text")
                    .and_then(serde_json::Value::as_str)
                    .map_or(0, str::len);
                let within_turn_context_bounds = context.len() < available_context_slots
                    && context_bytes.saturating_add(proposed_bytes) <= available_context_bytes;
                match mediate_extension_hook_proposal(
                    core,
                    &hook,
                    proposal,
                    workspace_id,
                    session_id,
                    within_turn_context_bounds,
                    now_ms,
                    correlation_id.clone(),
                ) {
                    Ok(ExtensionHookMediation::Applied(annotation)) => {
                        context_bytes = context_bytes.saturating_add(annotation.instructions.len());
                        context.push(annotation);
                        (
                            true,
                            format!(
                                "completed in {} ms; one proposal consumed by Action Gateway",
                                result.duration_ms
                            ),
                        )
                    }
                    Ok(ExtensionHookMediation::Denied(reason)) => {
                        denied = true;
                        (false, reason.into())
                    }
                    Err(error) => {
                        denied = true;
                        if mediation_error.is_none() {
                            mediation_error = Some(error);
                        }
                        (
                            false,
                            "Action Gateway failed closed before consuming the proposal".into(),
                        )
                    }
                }
            }
            Err(error) => {
                denied = true;
                (false, format!("worker failed closed: {error}"))
            }
        };
        records.push(extension::service::ExtensionHookExecutionRecord {
            package_id: hook.activation.package_id,
            hook_id: hook.contract.hook_id,
            succeeded,
            detail,
        });
    }
    core.extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .record_hook_execution_batch(generation, &records, now_ms)
        .map_err(|error| extension_boundary_error(error, correlation_id.clone()))?;
    if let Some(error) = mediation_error {
        return Err(error);
    }
    if denied {
        return Err(extension_boundary_error(
            extension::ExtensionError::HookDenied,
            correlation_id,
        ));
    }
    Ok(context)
}

fn require_extension_generation(
    request: &SnapshotRequest,
    current: u64,
) -> Result<(), ProtocolError> {
    if request.expected_generation.0 == current {
        return Ok(());
    }
    Err(platform_boundary_error(
        request.correlation_id.clone(),
        if request.expected_generation.0 < current {
            ProtocolErrorCode::StaleGeneration
        } else {
            ProtocolErrorCode::FutureGeneration
        },
        "Extension state changed before the operation",
        request.expected_generation.0 < current,
    ))
}

fn current_extension_skill_roots(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<Vec<extension::skill_sources::SkillSourceRoot>, ProtocolError> {
    let active_project_id = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .active_project_id
        .clone();
    let mut roots = vec![
        extension::skill_sources::SkillSourceRoot {
            source_kind: extension::ExtensionSourceKind::UserGlobal,
            source_id: "user-global".into(),
            root: core.c4os_home.join("skills/user"),
            trusted: true,
        },
        extension::skill_sources::SkillSourceRoot {
            source_kind: extension::ExtensionSourceKind::Bundled,
            source_id: "c4os-bundled".into(),
            root: core.bundled_skill_root.clone(),
            trusted: true,
        },
    ];
    let active_workspace = core
        .active_workspace
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let Some(workspace) = active_workspace.as_ref() else {
        return Ok(roots);
    };
    let workspace_id = workspace.manifest().workspace_id.to_string();
    roots.push(extension::skill_sources::SkillSourceRoot {
        source_kind: extension::ExtensionSourceKind::WorkspaceLocal,
        source_id: workspace_id,
        root: workspace.working_root().join("skills"),
        trusted: true,
    });
    let Some(active_project_id) = active_project_id else {
        return Ok(roots);
    };
    let query = core::database::SnapshotQuery::new(core::database::MAX_READ_RECORDS)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let snapshot = workspace
        .snapshot(query)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let Some(project) = snapshot.projects.iter().find(|project| {
        project.project_id == active_project_id
            && project.lifecycle_state == core::database::LifecycleState::Active
            && project.path_state != core::database::ProjectPathState::Missing
    }) else {
        return Ok(roots);
    };
    let project_id = Uuid::parse_str(&project.project_id)
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    roots.push(extension::skill_sources::SkillSourceRoot {
        source_kind: extension::ExtensionSourceKind::ProjectLocal,
        source_id: project.project_id.clone(),
        root: Path::new(&project.current_path).join(".c4os/skills"),
        trusted: workspace.is_project_trusted(project_id),
    });
    Ok(roots)
}

fn synchronize_extension_skill_sources(
    core: &AppCoreState,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<extension::ExtensionServiceSnapshot, ProtocolError> {
    let roots = current_extension_skill_roots(core, correlation_id.clone())?;
    core.extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .synchronize_skill_source_roots(roots, now_ms)
        .map_err(|error| extension_boundary_error(error, correlation_id))
}

fn extension_snapshot_envelope(
    core: &AppCoreState,
    request: SnapshotRequest,
    mut snapshot: extension::ExtensionServiceSnapshot,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, ProtocolError> {
    snapshot.active_workers = core
        .hook_supervisor
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .as_ref()
        .map(extension::hook::HookSupervisor::active_worker_count)
        .transpose()
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?
        .unwrap_or(0);
    snapshot
        .validate()
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    protocol::snapshot_envelope(request, StateGeneration(snapshot.generation), snapshot)
}

#[tauri::command]
fn extension_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot =
        synchronize_extension_skill_sources(&core, now_ms, request.correlation_id.clone())?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_add_marketplace(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::MarketplaceSourceInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    require_extension_generation(&request, extensions.snapshot().generation)?;
    let snapshot = extensions
        .add_marketplace(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_refresh_catalogs(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    require_extension_generation(&request, extensions.snapshot().generation)?;
    let snapshot = extensions
        .refresh_catalogs(request.expected_generation.0, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

fn require_extension_input_generation(
    request: &SnapshotRequest,
    expected_generation: u64,
) -> Result<(), ProtocolError> {
    if request.expected_generation.0 != expected_generation {
        return Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidGeneration,
            "The Extension input generation does not match its request",
            false,
        ));
    }
    Ok(())
}

fn mutate_extension_package(
    core: &AppCoreState,
    request: &SnapshotRequest,
    input: extension::service::ExtensionPackageInput,
    now_ms: u64,
    operation: impl FnOnce(
        &mut extension::service::ExtensionService,
        extension::service::ExtensionPackageInput,
        u64,
    )
        -> Result<extension::ExtensionServiceSnapshot, extension::ExtensionError>,
) -> Result<extension::ExtensionServiceSnapshot, ProtocolError> {
    require_extension_input_generation(request, input.expected_generation)?;
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    operation(&mut extensions, input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))
}

fn invalidate_extension_generation(
    core: &AppCoreState,
    package_id: &str,
    generation: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    if let Some(supervisor) = core
        .hook_supervisor
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .as_ref()
    {
        supervisor
            .invalidate_package_generation(package_id, generation)
            .map_err(|error| extension_boundary_error(error, correlation_id))?;
    }
    Ok(())
}

macro_rules! extension_package_command {
    ($command:ident, $method:ident) => {
        #[tauri::command]
        fn $command(
            core: tauri::State<'_, AppCoreState>,
            request: SnapshotRequest,
            input: extension::service::ExtensionPackageInput,
        ) -> Result<
            ProtocolEnvelope<extension::ExtensionServiceSnapshot>,
            protocol::StructuredCoreError,
        > {
            validate_snapshot_request(&request)?;
            let now_ms = current_time_ms()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            let snapshot =
                mutate_extension_package(&core, &request, input, now_ms, |service, input, now| {
                    service.$method(input, now)
                })?;
            extension_snapshot_envelope(&core, request, snapshot)
        }
    };
}

extension_package_command!(extension_install_disabled, install_disabled);
extension_package_command!(extension_enable, enable);
extension_package_command!(extension_stage_update, stage_update);

macro_rules! extension_package_command_with_worker_termination {
    ($command:ident, $method:ident, $mutation:ident) => {
        #[tauri::command]
        fn $command(
            core: tauri::State<'_, AppCoreState>,
            request: SnapshotRequest,
            input: extension::service::ExtensionPackageInput,
        ) -> Result<
            ProtocolEnvelope<extension::ExtensionServiceSnapshot>,
            protocol::StructuredCoreError,
        > {
            validate_snapshot_request(&request)?;
            require_extension_input_generation(&request, input.expected_generation)?;
            let package_id = input.package_id.clone();
            let invalidated_generation = input.expected_generation;
            let now_ms = current_time_ms()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            let mut extensions = core
                .extensions
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            require_extension_generation(&request, extensions.snapshot().generation)?;
            extensions
                .preflight_package_mutation(
                    &input,
                    extension::service::ExtensionPackageMutation::$mutation,
                )
                .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
            core.runtime
                .revoke_plugin_authority(&package_id, now_ms)
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            invalidate_extension_generation(
                &core,
                &package_id,
                invalidated_generation,
                request.correlation_id.clone(),
            )?;
            let snapshot = extensions
                .$method(input, now_ms)
                .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
            extension_snapshot_envelope(&core, request, snapshot)
        }
    };
}

extension_package_command_with_worker_termination!(extension_disable, disable, Disable);
extension_package_command_with_worker_termination!(
    extension_activate_update,
    activate_staged_update,
    ActivateStagedUpdate
);
extension_package_command_with_worker_termination!(extension_rollback, rollback, Rollback);
extension_package_command_with_worker_termination!(extension_uninstall, uninstall, Uninstall);

#[tauri::command]
fn extension_revoke(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionRevocationInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let package_id = input.package_id.clone();
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    require_extension_generation(&request, extensions.snapshot().generation)?;
    extensions
        .preflight_package_mutation(
            &extension::service::ExtensionPackageInput {
                expected_generation: input.expected_generation,
                package_id: package_id.clone(),
            },
            extension::service::ExtensionPackageMutation::Revoke,
        )
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    if input.reason.trim().is_empty() || input.reason.len() > 1_024 {
        return Err(extension_boundary_error(
            extension::ExtensionError::InvalidInput,
            request.correlation_id.clone(),
        )
        .into());
    }
    core.runtime
        .revoke_plugin_authority(&package_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    invalidate_extension_generation(
        &core,
        &package_id,
        input.expected_generation,
        request.correlation_id.clone(),
    )?;
    let snapshot = extensions
        .revoke(
            extension::service::ExtensionPackageInput {
                expected_generation: input.expected_generation,
                package_id: package_id.clone(),
            },
            &input.reason,
            now_ms,
        )
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_revoke_key(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionKeyRevocationInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    require_extension_generation(&request, extensions.snapshot().generation)?;
    if input.reason.trim().is_empty() || input.reason.len() > 1_024 {
        return Err(extension_boundary_error(
            extension::ExtensionError::InvalidInput,
            request.correlation_id.clone(),
        )
        .into());
    }
    let package_ids = extensions
        .package_ids_signed_by_key(&input.key_id)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    for package_id in &package_ids {
        core.runtime
            .revoke_plugin_authority(package_id, now_ms)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        invalidate_extension_generation(
            &core,
            package_id,
            input.expected_generation,
            request.correlation_id.clone(),
        )?;
    }
    let snapshot = extensions
        .revoke_key(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_set_skill_enabled(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionSkillAvailabilityInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .set_skill_enabled(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_select_skill(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionSkillInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .select_skill(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_customize_skill(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionSkillInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .customize_skill(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_review_hook(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionHookReviewInput,
) -> Result<ProtocolEnvelope<extension::ExtensionServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .review_hook(input, now_ms)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    extension_snapshot_envelope(&core, request, snapshot)
}

#[tauri::command]
fn extension_load_skill(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionSkillInput,
) -> Result<
    ProtocolEnvelope<extension::service::SkillInstructionsSnapshot>,
    protocol::StructuredCoreError,
> {
    validate_snapshot_request(&request)?;
    require_extension_input_generation(&request, input.expected_generation)?;
    let mut extensions = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    require_extension_generation(&request, extensions.snapshot().generation)?;
    let payload = extensions
        .load_skill_instructions(&input.skill_identity)
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
    protocol::snapshot_envelope(request, StateGeneration(input.expected_generation), payload)
}

#[tauri::command]
fn extension_publisher_link(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: extension::service::ExtensionPublisherLinkInput,
) -> Result<ProtocolEnvelope<String>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let (snapshot, payload) = {
        let extensions = core
            .extensions
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let snapshot = extensions.snapshot();
        let payload = extensions
            .publisher_link(&input)
            .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?;
        (snapshot, payload)
    };
    require_extension_generation(&request, snapshot.generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let live = current_artifact_live_authority(&core, now_ms, request.correlation_id.clone())?;
    let action_id = format!("publisher-link-{}", Uuid::new_v4().as_simple());
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: "c4os.desktop.open-url".into(),
        arguments: serde_json::json!({
            "packageId": input.package_id,
            "link": input.link,
            "urlSha256": sha256_bytes(payload.as_bytes()),
        }),
        risk: CanonicalRisk::Low,
        requested_authority: BTreeSet::from(["desktop.open-url".into()]),
        canonical_target: payload.clone(),
        target_version: sha256_bytes(payload.as_bytes()),
        workspace_id: "c4os-app".into(),
        session_id: "settings".into(),
        run_id: format!("settings-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: Some(input.package_id.clone()),
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "The publisher link could not be bound to an exact action",
            false,
        )
    })?;
    let facts = ActionFacts {
        action_kind: "desktop.open-publisher-link".into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::Desktop,
        effects: BTreeSet::from([ActionEffect::Control]),
        scope: ActionScope::Remote,
        initiator: ActionInitiator::User,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target: payload.clone(),
        workspace_id: action.workspace_id.clone(),
        session_id: action.session_id.clone(),
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: action.plugin_or_mcp_id.clone(),
        target_resolved: true,
        authenticated: false,
        trusted_root: false,
        explicit_scope_grant: true,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .propose_direct_action(&facts, action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (token, prompt_id) = match proposal {
        GatewayProposal::Denied { .. } => {
            return Err(platform_boundary_error(
                request.correlation_id,
                ProtocolErrorCode::Forbidden,
                "Policy denied opening the publisher link",
                false,
            ));
        }
        GatewayProposal::Authorized { token, .. } => (token, None),
        GatewayProposal::PendingApproval { prompt, .. } => {
            // This Settings click is the user's one-time approval for the
            // exact package-bound URL; the Action Gateway still consumes the
            // resulting permit before `/usr/bin/open` can run.
            let prompt_id = prompt.prompt_id.clone();
            let response = core
                .runtime
                .coordinator()
                .and_then(|mut coordinator| {
                    coordinator
                        .answer_direct_approval(&prompt_id, ApprovalAnswer::Allow, now_ms)
                        .map(|operation| operation.value)
                        .map_err(Into::into)
                })
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            match response {
                ApprovalResponse::Authorized { token, prompt }
                    if prompt.prompt_id == prompt_id && prompt.action == action =>
                {
                    (token, Some(prompt_id))
                }
                _ => return Err(workspace_state_unavailable(request.correlation_id)),
            }
        }
    };
    let mut effect_result = None;
    let canonical_target = action.canonical_target.clone();
    core.runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator.execute_direct_action(
                &token,
                &action,
                live,
                prompt_id.as_deref(),
                now_ms,
                |_permit| {
                    let result = Command::new("/usr/bin/open")
                        .arg(&canonical_target)
                        .env_clear()
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .map(|status| status.success());
                    let succeeded = matches!(result, Ok(true));
                    effect_result = Some(result);
                    NormalizedActionResult {
                        status: if succeeded {
                            NormalizedActionStatus::Succeeded
                        } else {
                            NormalizedActionStatus::Failed
                        },
                        result_code: if succeeded {
                            "publisher-link-opened"
                        } else {
                            "publisher-link-open-failed"
                        }
                        .into(),
                        exit_code: succeeded.then_some(0),
                        changed_targets: if succeeded {
                            vec![canonical_target.clone()]
                        } else {
                            Vec::new()
                        },
                        output_sha256: None,
                        completed_at_ms: now_ms.saturating_add(1),
                    }
                },
            )?;
            Ok(())
        })
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    if !matches!(effect_result, Some(Ok(true))) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Unavailable,
            "The system browser could not open the publisher link",
            true,
        ));
    }
    protocol::snapshot_envelope(request, StateGeneration(snapshot.generation), payload)
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
        PickerPurpose::OpenFolder => "Open Folder",
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
        PickerPurpose::OpenFolder => dialog.blocking_pick_folder().map(|path| vec![path]),
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

#[derive(Clone)]
struct ActiveArtifactScope {
    workspace_id: String,
    project_id: String,
    session_id: String,
    project_name: String,
    project_root: TrustedProjectRoot,
    filesystem: ProjectFilesystem,
    database: Arc<core::database::DatabaseActor>,
}

fn active_artifact_scope(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<ActiveArtifactScope, ProtocolError> {
    let workspace = active_workspace_snapshot(core, correlation_id.clone())?;
    let workspace_id = workspace
        .workspace
        .as_ref()
        .map(|record| record.workspace_id.clone())
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let project_id = conversation
        .active_project_id
        .clone()
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let session_id = conversation
        .active_session_id
        .clone()
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    drop(conversation);
    let project = workspace
        .projects
        .iter()
        .find(|project| {
            project.project_id == project_id
                && project.lifecycle_state == core::database::LifecycleState::Active
                && project.path_state != core::database::ProjectPathState::Missing
        })
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let active_chat = workspace.chats.iter().any(|chat| {
        chat.chat_id == session_id
            && chat.project_id == project_id
            && chat.lifecycle_state == core::database::LifecycleState::Active
    });
    if !active_chat {
        return Err(workspace_state_unavailable(correlation_id));
    }
    let project_root =
        TrustedProjectRoot::open(Path::new(&project.current_path)).map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Unavailable,
                "The active Project root is unavailable",
                true,
            )
        })?;
    let limits = ProjectFilesystemLimits::new(
        artifact::file::MAX_FILE_CONTENT_BYTES as u64,
        artifact::folder::MAX_FOLDER_ENTRIES,
        execution::filesystem::MAX_PROJECT_FOLDER_NAME_BYTES,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let filesystem =
        ProjectFilesystem::bind_with_limits(project_root.clone(), limits).map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Unavailable,
                "The active Project filesystem is unavailable",
                true,
            )
        })?;
    let database = core
        .active_workspace
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .as_ref()
        .map(|workspace| Arc::clone(workspace.database_actor()))
        .ok_or_else(|| workspace_state_unavailable(correlation_id))?;
    Ok(ActiveArtifactScope {
        workspace_id,
        project_id,
        session_id,
        project_name: project.display_name.clone(),
        project_root,
        filesystem,
        database,
    })
}

fn require_exact_artifact_generation(
    request: &SnapshotRequest,
    snapshot: &ArtifactWorkspaceSnapshot,
) -> Result<(), ProtocolError> {
    if request.expected_generation == snapshot.generation {
        Ok(())
    } else {
        Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::StaleGeneration,
            "Artifact state changed before the operation",
            true,
        ))
    }
}

fn project_relative_picker_path(
    selected: &Path,
    scope: &ActiveArtifactScope,
    correlation_id: protocol::CorrelationId,
) -> Result<PathBuf, ProtocolError> {
    let relative = selected
        .strip_prefix(scope.project_root.canonical_root())
        .map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Forbidden,
                "Select a target inside the active Project",
                false,
            )
        })?;
    if relative
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
        && !relative.as_os_str().is_empty()
    {
        return Err(invalid_picker_selection(correlation_id));
    }
    Ok(relative.to_path_buf())
}

fn take_artifact_picker_grant(
    core: &AppCoreState,
    picker_grant_id: &PickerGrantId,
    purpose: PickerPurpose,
    object_kind: PickerObjectKind,
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
        || grant.object_kind() != object_kind
        || grant.issued_at_ms() > now_ms
        || now_ms.saturating_sub(grant.issued_at_ms()) > 10 * 60 * 1_000
    {
        return Err(invalid_picker_selection(correlation_id));
    }
    Ok(grant.path().to_path_buf())
}

fn serialize_artifact_record(
    record: &artifact::ArtifactRecord,
) -> Result<core::database::WorkspaceArtifactDocumentRecord, ProtocolError> {
    record.validate().map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Artifact record is invalid",
            false,
        )
    })?;
    let canonical_document = serde_json::to_string(record).map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::Internal,
            "Artifact record could not be serialized",
            true,
        )
    })?;
    Ok(core::database::WorkspaceArtifactDocumentRecord {
        workspace_id: record.workspace_id.clone(),
        project_id: record.project_id.clone(),
        session_id: record.session_id.clone(),
        artifact_id: record.artifact_id.clone(),
        provider_kind: record.provider.type_id.clone(),
        provider_version: record.provider.schema_version,
        state_schema_version: record.schema_version,
        revision: record.record_revision,
        canonical_document,
        updated_at_ms: record.updated_at_ms,
    })
}

fn deserialize_artifact_record(
    record: core::database::WorkspaceArtifactDocumentRecord,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::ArtifactRecord, ProtocolError> {
    let artifact = serde_json::from_str::<artifact::ArtifactRecord>(&record.canonical_document)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    artifact
        .validate()
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    Ok(artifact)
}

fn load_active_artifact_record(
    scope: &ActiveArtifactScope,
    artifact_id: &protocol::ArtifactId,
    base_revision: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::ArtifactRecord, ProtocolError> {
    let record = scope
        .database
        .artifact_document(artifact_id.as_str())
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .ok_or_else(|| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Artifact is unavailable",
                false,
            )
        })?;
    let artifact = deserialize_artifact_record(record, correlation_id.clone())?;
    if artifact.project_id != scope.project_id
        || artifact.session_id != scope.session_id
        || artifact.record_revision != base_revision
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The Artifact changed before the operation",
            true,
        ));
    }
    Ok(artifact)
}

fn persist_artifact_record(
    scope: &ActiveArtifactScope,
    record: &artifact::ArtifactRecord,
    expected_revision: Option<u64>,
    correlation_id: protocol::CorrelationId,
) -> Result<u64, ProtocolError> {
    persist_artifact_record_to_database(&scope.database, record, expected_revision, correlation_id)
}

fn persist_artifact_record_to_database(
    database: &core::database::DatabaseActor,
    record: &artifact::ArtifactRecord,
    expected_revision: Option<u64>,
    correlation_id: protocol::CorrelationId,
) -> Result<u64, ProtocolError> {
    let document = serialize_artifact_record(record)?;
    database
        .save_artifact_document(document, expected_revision)
        .map_err(|error| {
            platform_boundary_error(
                correlation_id,
                if matches!(error, core::database::DatabaseError::Conflict(_)) {
                    ProtocolErrorCode::Conflict
                } else {
                    ProtocolErrorCode::Internal
                },
                "The Artifact could not be committed",
                true,
            )
        })
}

fn advance_artifact_record(
    record: &mut artifact::ArtifactRecord,
    kind: artifact::ArtifactHistoryKind,
    now_ms: u64,
) -> Result<(), ProtocolError> {
    record.record_revision = record.record_revision.checked_add(1).ok_or_else(|| {
        ProtocolError::new(
            ProtocolErrorCode::Internal,
            "Artifact revision is exhausted",
            false,
        )
    })?;
    record.updated_at_ms = now_ms;
    let state_document = serde_json::to_vec(&record.state).map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::Internal,
            "Artifact state could not be hashed",
            true,
        )
    })?;
    record
        .append_history(artifact::ArtifactHistoryEntry {
            record_revision: record.record_revision,
            resource_version: record.resource_version(),
            state_sha256: sha256_bytes(&state_document),
            kind,
            recorded_at_ms: now_ms,
        })
        .map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Artifact history transition is invalid",
                false,
            )
        })?;
    record.validate().map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Artifact transition is invalid",
            false,
        )
    })
}

fn normalized_project_relative_text(
    path: &Path,
    correlation_id: protocol::CorrelationId,
) -> Result<String, ProtocolError> {
    path.to_str()
        .map(|value| value.replace(std::path::MAIN_SEPARATOR, "/"))
        .filter(|value| {
            value.len() <= artifact::file::MAX_PROJECT_RELATIVE_PATH_BYTES
                && !value.contains('\0')
                && !value.contains('\\')
        })
        .ok_or_else(|| invalid_picker_selection(correlation_id))
}

fn artifact_breadcrumbs(
    project_name: &str,
    relative_path: &str,
    include_leaf: bool,
) -> Vec<artifact::FolderBreadcrumb> {
    let mut breadcrumbs = vec![artifact::FolderBreadcrumb {
        label: project_name.into(),
        project_relative_path: String::new(),
    }];
    let components = relative_path.split('/').filter(|part| !part.is_empty());
    let mut current = String::new();
    let parts = components.collect::<Vec<_>>();
    let included = if include_leaf {
        parts.len()
    } else {
        parts.len().saturating_sub(1)
    };
    for part in parts.into_iter().take(included) {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(part);
        breadcrumbs.push(artifact::FolderBreadcrumb {
            label: part.into(),
            project_relative_path: current.clone(),
        });
    }
    breadcrumbs
}

fn protocol_breadcrumbs_for_file(
    project_name: &str,
    relative_path: &str,
) -> Vec<ArtifactBreadcrumbSnapshot> {
    let mut breadcrumbs = artifact_breadcrumbs(project_name, relative_path, true)
        .into_iter()
        .map(|breadcrumb| ArtifactBreadcrumbSnapshot {
            id: breadcrumb.project_relative_path,
            label: breadcrumb.label,
            is_current: false,
        })
        .collect::<Vec<_>>();
    if let Some(last) = breadcrumbs.last_mut() {
        last.is_current = true;
    }
    breadcrumbs
}

fn protocol_breadcrumbs_for_folder(
    breadcrumbs: &[artifact::FolderBreadcrumb],
) -> Vec<ArtifactBreadcrumbSnapshot> {
    breadcrumbs
        .iter()
        .enumerate()
        .map(|(index, breadcrumb)| ArtifactBreadcrumbSnapshot {
            id: breadcrumb.project_relative_path.clone(),
            label: breadcrumb.label.clone(),
            is_current: index + 1 == breadcrumbs.len(),
        })
        .collect()
}

fn file_media_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "css" => "text/css",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "js" | "mjs" | "cjs" => "text/javascript",
        "json" => "application/json",
        "md" | "markdown" => "text/markdown",
        "rs" => "text/rust",
        "toml" => "application/toml",
        "ts" | "tsx" => "text/typescript",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        _ => "text/plain",
    }
}

fn domain_file_state(
    scope: &ActiveArtifactScope,
    relative_path: &Path,
    sequence: u64,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::FileArtifactState, ProtocolError> {
    let read = scope.filesystem.read_utf8(relative_path).map_err(|error| {
        artifact_filesystem_error(error, correlation_id.clone(), "The File could not be read")
    })?;
    let relative = normalized_project_relative_text(read.relative_path(), correlation_id.clone())?;
    let display_name = read
        .relative_path()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_picker_selection(correlation_id.clone()))?;
    let live_version = artifact::FileLiveVersion::new_with_target_version(
        sequence,
        read.version().target_version(),
        read.version().content_sha256(),
        read.version().byte_length(),
        now_ms,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    artifact::FileArtifactState::new(
        artifact::FileResourceReference::new(&relative, &relative)
            .map_err(|_| invalid_picker_selection(correlation_id.clone()))?,
        display_name,
        file_media_type(read.relative_path()),
        read.content(),
        live_version,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id))
}

fn domain_folder_state(
    scope: &ActiveArtifactScope,
    relative_path: &Path,
    sequence: u64,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::FolderArtifactState, ProtocolError> {
    let listing = scope
        .filesystem
        .list_folder(relative_path)
        .map_err(|error| {
            artifact_filesystem_error(
                error,
                correlation_id.clone(),
                "The Folder could not be listed",
            )
        })?;
    let relative =
        normalized_project_relative_text(listing.relative_path(), correlation_id.clone())?;
    let display_path = if relative.is_empty() {
        scope.project_name.clone()
    } else {
        relative.clone()
    };
    let mut entries = listing
        .entries()
        .iter()
        .filter_map(|entry| {
            let kind = match entry.kind() {
                ProjectFolderEntryKind::Directory => artifact::FolderEntryKind::Folder,
                ProjectFolderEntryKind::File => artifact::FolderEntryKind::File,
                ProjectFolderEntryKind::Symlink | ProjectFolderEntryKind::Other => return None,
            };
            let project_relative_path = if relative.is_empty() {
                entry.name().to_owned()
            } else {
                format!("{relative}/{}", entry.name())
            };
            let entry_digest = sha256_bytes(project_relative_path.as_bytes());
            Some(artifact::FolderEntry {
                entry_id: format!("entry-{}", entry_digest.trim_start_matches("sha256:")),
                name: entry.name().into(),
                kind,
                project_relative_path,
                display_metadata: entry.byte_length().map(|bytes| format!("{bytes} bytes")),
                byte_length: entry.byte_length(),
                modified_at_ms: None,
                content_sha256: None,
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        (
            entry.name.to_ascii_lowercase(),
            entry.name.clone(),
            entry.kind,
        )
    });
    let breadcrumbs = artifact_breadcrumbs(&scope.project_name, &relative, true);
    let listing_version = artifact::FolderListingVersion::from_target_version(
        sequence,
        listing.target_version(),
        &entries,
        now_ms,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    artifact::FolderArtifactState::new(
        artifact::folder::FolderResourceReference::new(relative, display_path)
            .map_err(|_| invalid_picker_selection(correlation_id.clone()))?,
        breadcrumbs,
        entries,
        listing_version,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id))
}

fn artifact_filesystem_error(
    error: ProjectFilesystemError,
    correlation_id: protocol::CorrelationId,
    message: &'static str,
) -> ProtocolError {
    let (code, retryable) = match error {
        ProjectFilesystemError::Conflict | ProjectFilesystemError::TargetChanged => {
            (ProtocolErrorCode::Conflict, true)
        }
        ProjectFilesystemError::TargetUnavailable => (ProtocolErrorCode::NotFound, false),
        ProjectFilesystemError::SymlinkRejected
        | ProjectFilesystemError::InvalidPath
        | ProjectFilesystemError::NotDirectory
        | ProjectFilesystemError::NotRegularFile => (ProtocolErrorCode::Forbidden, false),
        _ => (ProtocolErrorCode::Unavailable, true),
    };
    platform_boundary_error(correlation_id, code, message, retryable)
}

fn new_artifact_record(
    scope: &ActiveArtifactScope,
    provider: artifact::ArtifactProviderDescriptor,
    state: artifact::ArtifactState,
    operation_kind: &str,
    now_ms: u64,
) -> Result<artifact::ArtifactRecord, ProtocolError> {
    new_artifact_record_with_id(
        scope,
        format!("artifact-{}", Uuid::new_v4().as_simple()),
        provider,
        state,
        operation_kind,
        now_ms,
    )
}

fn new_artifact_record_with_id(
    scope: &ActiveArtifactScope,
    artifact_id: String,
    provider: artifact::ArtifactProviderDescriptor,
    state: artifact::ArtifactState,
    operation_kind: &str,
    now_ms: u64,
) -> Result<artifact::ArtifactRecord, ProtocolError> {
    let mut record = artifact::ArtifactRecord {
        schema_version: artifact::ARTIFACT_SCHEMA_VERSION,
        artifact_id,
        workspace_id: scope.workspace_id.clone(),
        project_id: scope.project_id.clone(),
        session_id: scope.session_id.clone(),
        provider,
        source: artifact::ArtifactSource::DirectOperation {
            operation_id: format!("{operation_kind}-{}", Uuid::new_v4().as_simple()),
        },
        record_revision: 1,
        lifecycle: artifact::ArtifactLifecycle::Ready,
        state,
        history: Vec::new(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let state_document = serde_json::to_vec(&record.state).map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::Internal,
            "Artifact creation state could not be hashed",
            true,
        )
    })?;
    let initial_history_kind = if matches!(&record.state, artifact::ArtifactState::Terminal(_)) {
        artifact::ArtifactHistoryKind::CommandQueued
    } else {
        artifact::ArtifactHistoryKind::Created
    };
    record
        .append_history(artifact::ArtifactHistoryEntry {
            record_revision: 1,
            resource_version: record.resource_version(),
            state_sha256: sha256_bytes(&state_document),
            kind: initial_history_kind,
            recorded_at_ms: now_ms,
        })
        .map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Artifact creation history is invalid",
                false,
            )
        })?;
    record.validate().map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Artifact creation is invalid",
            false,
        )
    })?;
    Ok(record)
}

fn artifact_resource_snapshot(
    version: artifact::ArtifactResourceVersion,
) -> ArtifactResourceVersionSnapshot {
    ArtifactResourceVersionSnapshot {
        sequence: version.sequence,
        sha256: version.sha256,
        observed_at_ms: version.observed_at_ms,
    }
}

fn artifact_history_kind(kind: artifact::ArtifactHistoryKind) -> &'static str {
    match kind {
        artifact::ArtifactHistoryKind::Created => "created",
        artifact::ArtifactHistoryKind::ResourceRefreshed => "resourceRefreshed",
        artifact::ArtifactHistoryKind::DraftChanged => "draftChanged",
        artifact::ArtifactHistoryKind::DraftDiscarded => "draftDiscarded",
        artifact::ArtifactHistoryKind::ProposalChanged => "proposalChanged",
        artifact::ArtifactHistoryKind::SaveRequested => "saveRequested",
        artifact::ArtifactHistoryKind::SaveCompleted => "saveCompleted",
        artifact::ArtifactHistoryKind::ConflictObserved => "conflictObserved",
        artifact::ArtifactHistoryKind::RecoveryChanged => "recoveryChanged",
        artifact::ArtifactHistoryKind::NavigationChanged => "navigationChanged",
        artifact::ArtifactHistoryKind::ReplySubmitted => "replySubmitted",
        artifact::ArtifactHistoryKind::Converted => "converted",
        artifact::ArtifactHistoryKind::CommandQueued => "commandQueued",
        artifact::ArtifactHistoryKind::CommandStarted => "commandStarted",
        artifact::ArtifactHistoryKind::OutputAppended => "outputAppended",
        artifact::ArtifactHistoryKind::InputSubmitted => "inputSubmitted",
        artifact::ArtifactHistoryKind::TerminalResized => "terminalResized",
        artifact::ArtifactHistoryKind::StopRequested => "stopRequested",
        artifact::ArtifactHistoryKind::CommandCompleted => "commandCompleted",
        artifact::ArtifactHistoryKind::CommandInterrupted => "commandInterrupted",
        artifact::ArtifactHistoryKind::CommandFailed => "commandFailed",
    }
}

fn artifact_source_label(source: &artifact::ArtifactSource) -> String {
    match source {
        artifact::ArtifactSource::DirectOperation { .. } => "Direct operation".into(),
        artifact::ArtifactSource::RunAttempt { run_id, .. } => format!("Run {run_id}"),
    }
}

fn artifact_status(
    record: &artifact::ArtifactRecord,
    pending_approval_id: Option<&str>,
) -> ArtifactShellStatusSnapshot {
    match &record.lifecycle {
        artifact::ArtifactLifecycle::Loading { .. } => ArtifactShellStatusSnapshot {
            kind: "loading".into(),
            message: Some("Loading artifact…".into()),
        },
        artifact::ArtifactLifecycle::Error { message, .. } => ArtifactShellStatusSnapshot {
            kind: "error".into(),
            message: Some(message.clone()),
        },
        artifact::ArtifactLifecycle::Degraded { message, .. } => ArtifactShellStatusSnapshot {
            kind: "degraded".into(),
            message: Some(message.clone()),
        },
        artifact::ArtifactLifecycle::UnknownVersion { .. } => ArtifactShellStatusSnapshot {
            kind: "degraded".into(),
            message: Some("This artifact version is not supported by this C4OS build.".into()),
        },
        artifact::ArtifactLifecycle::Ready => {
            let recovery = matches!(
                &record.state,
                artifact::ArtifactState::File(file) if file.recovery.is_some()
            ) || matches!(
                &record.state,
                artifact::ArtifactState::Terminal(terminal)
                    if matches!(&terminal.status, artifact::TerminalCommandStatus::Recovery { .. })
            ) || matches!(
                &record.state,
                artifact::ArtifactState::Browser(browser)
                    if matches!(&browser.phase, artifact::BrowserPhase::Recovery { .. })
            );
            let terminal_failure = match &record.state {
                artifact::ArtifactState::Terminal(terminal) => {
                    if let artifact::TerminalCommandStatus::Failed { message, .. } =
                        &terminal.status
                    {
                        Some(message.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let browser_failure = match &record.state {
                artifact::ArtifactState::Browser(browser) => {
                    matches!(&browser.phase, artifact::BrowserPhase::Error { .. })
                        .then(|| "The Browser operation could not be completed.".to_owned())
                }
                _ => None,
            };
            if recovery {
                ArtifactShellStatusSnapshot {
                    kind: "recovery".into(),
                    message: Some(match &record.state {
                        artifact::ArtifactState::Terminal(_) => {
                            "The Terminal command was retained after its live process could not be recovered."
                                .into()
                        }
                        artifact::ArtifactState::Browser(_) => {
                            "The Browser controller is ready to recover.".into()
                        }
                        _ => "The draft was retained after the save could not complete.".into(),
                    }),
                }
            } else if let Some(message) = terminal_failure.or(browser_failure) {
                ArtifactShellStatusSnapshot {
                    kind: "error".into(),
                    message: Some(message),
                }
            } else if pending_approval_id.is_some() {
                ArtifactShellStatusSnapshot {
                    kind: "ready".into(),
                    message: Some(match &record.state {
                        artifact::ArtifactState::Terminal(_) => {
                            "Approval is required before this Terminal operation can continue."
                                .into()
                        }
                        artifact::ArtifactState::Browser(_) => {
                            "Approval is required before this Browser operation can continue."
                                .into()
                        }
                        _ => "Approval is required before saving this File.".into(),
                    }),
                }
            } else {
                ArtifactShellStatusSnapshot {
                    kind: "ready".into(),
                    message: None,
                }
            }
        }
    }
}

fn file_pending_content(file: &artifact::FileArtifactState) -> String {
    file.proposal
        .as_ref()
        .filter(|proposal| matches!(proposal.status, artifact::FileProposalStatus::Pending))
        .map(|proposal| proposal.proposed_content.clone())
        .or_else(|| file.draft.as_ref().map(|draft| draft.content.clone()))
        .unwrap_or_else(|| file.content.clone())
}

fn file_protocol_state(
    file: &artifact::FileArtifactState,
    pending_approval_id: Option<&str>,
) -> ArtifactFileStateSnapshot {
    if let Some(conflict) = &file.conflict {
        return ArtifactFileStateSnapshot::Conflict {
            content: file.content.clone(),
            draft: file_pending_content(file),
            conflict_message: conflict.message.clone(),
            current_version_label: format!("Version {}", conflict.observed_live.sequence),
        };
    }
    if let Some(recovery) = &file.recovery {
        let recovery_message = match recovery {
            artifact::FileRecoveryState::SaveFailed { message, .. } => message.clone(),
            artifact::FileRecoveryState::Restored { .. } => {
                "The live File was restored after an interrupted save.".into()
            }
        };
        return ArtifactFileStateSnapshot::Recovery {
            content: file.content.clone(),
            draft: file_pending_content(file),
            recovery_message,
        };
    }
    if let Some(proposal) = &file.proposal
        && matches!(proposal.status, artifact::FileProposalStatus::Pending)
    {
        if pending_approval_id.is_some() {
            return ArtifactFileStateSnapshot::Approval {
                content: file.content.clone(),
                proposed_content: proposal.proposed_content.clone(),
                proposal_diff: (!proposal.unified_diff.is_empty())
                    .then(|| proposal.unified_diff.clone()),
                approval_summary: "Review and allow this exact File write.".into(),
            };
        }
        return ArtifactFileStateSnapshot::Proposed {
            content: file.content.clone(),
            proposed_content: proposal.proposed_content.clone(),
            proposal_diff: (!proposal.unified_diff.is_empty())
                .then(|| proposal.unified_diff.clone()),
            proposal_summary: "Proposed File change".into(),
        };
    }
    if pending_approval_id.is_some()
        && let Some(draft) = &file.draft
        && draft.dirty
    {
        return ArtifactFileStateSnapshot::Approval {
            content: file.content.clone(),
            proposed_content: draft.content.clone(),
            proposal_diff: None,
            approval_summary: "Review and allow this exact File write.".into(),
        };
    }
    if let Some(draft) = &file.draft {
        return if draft.dirty {
            ArtifactFileStateSnapshot::Dirty {
                content: file.content.clone(),
                draft: draft.content.clone(),
            }
        } else {
            ArtifactFileStateSnapshot::Edit {
                content: file.content.clone(),
                draft: draft.content.clone(),
            }
        };
    }
    ArtifactFileStateSnapshot::Read {
        content: file.content.clone(),
    }
}

fn browser_protocol_title(browser: &artifact::BrowserArtifactState) -> String {
    browser
        .current_title()
        .map(str::to_owned)
        .or_else(|| {
            artifact::BrowserNavigationTarget::parse(browser.current_display_url())
                .ok()
                .and_then(|target| target.navigation_url().host_str().map(str::to_owned))
        })
        .unwrap_or_else(|| "Browser".into())
}

fn artifact_protocol_snapshot(
    record: artifact::ArtifactRecord,
    pending_approval_id: Option<String>,
    browser_pending_operation: Option<String>,
    browser_pending_target_url: Option<String>,
    project_name: &str,
    browser_mount_generation: Option<u64>,
    browser_notices: Vec<ArtifactBrowserNoticeSnapshot>,
) -> Result<ArtifactSnapshot, ProtocolError> {
    let resource_version = artifact_resource_snapshot(record.resource_version());
    let history = record
        .history
        .iter()
        .map(|entry| ArtifactHistorySnapshot {
            record_revision: entry.record_revision,
            kind: artifact_history_kind(entry.kind).into(),
            recorded_at_ms: entry.recorded_at_ms,
            resource_version: artifact_resource_snapshot(entry.resource_version.clone()),
        })
        .collect();
    let (title, provider_state) = match &record.state {
        artifact::ArtifactState::File(file) => (
            file.display_name.clone(),
            ArtifactProviderStateSnapshot::File(ArtifactFileSnapshot {
                breadcrumbs: protocol_breadcrumbs_for_file(
                    project_name,
                    &file.resource.project_relative_path,
                ),
                language_label: Some(file.media_type.clone()),
                state: file_protocol_state(file, pending_approval_id.as_deref()),
                version_label: Some(format!("Version {}", file.live_version.sequence)),
            }),
        ),
        artifact::ArtifactState::Folder(folder) => (
            folder.resource.display_path.clone(),
            ArtifactProviderStateSnapshot::Folder(ArtifactFolderSnapshot {
                breadcrumbs: protocol_breadcrumbs_for_folder(&folder.breadcrumbs),
                entries: folder
                    .entries
                    .iter()
                    .map(|entry| ArtifactFolderEntrySnapshot {
                        id: entry.entry_id.clone(),
                        kind: match entry.kind {
                            artifact::FolderEntryKind::File => "file",
                            artifact::FolderEntryKind::Folder => "folder",
                        }
                        .into(),
                        metadata: entry.display_metadata.clone(),
                        name: entry.name.clone(),
                    })
                    .collect(),
                listing: ArtifactFolderListingSnapshot {
                    phase: match &record.lifecycle {
                        artifact::ArtifactLifecycle::Ready => "ready",
                        artifact::ArtifactLifecycle::Loading { .. } => "loading",
                        artifact::ArtifactLifecycle::Error { .. }
                        | artifact::ArtifactLifecycle::Degraded { .. } => "error",
                        artifact::ArtifactLifecycle::UnknownVersion { .. } => "error",
                    }
                    .into(),
                    message: match &record.lifecycle {
                        artifact::ArtifactLifecycle::Loading { .. } => {
                            Some("Loading folder…".into())
                        }
                        artifact::ArtifactLifecycle::Error { message, .. }
                        | artifact::ArtifactLifecycle::Degraded { message, .. } => {
                            Some(message.clone())
                        }
                        artifact::ArtifactLifecycle::UnknownVersion { .. } => Some(
                            "This folder provider version is not supported by this C4OS build."
                                .into(),
                        ),
                        artifact::ArtifactLifecycle::Ready => None,
                    },
                },
                listing_limit: artifact::folder::MAX_FOLDER_ENTRIES as u32,
                selected_entry_id: folder
                    .selection
                    .as_ref()
                    .map(|selection| selection.entry_id.clone()),
            }),
        ),
        artifact::ArtifactState::Browser(browser) => (
            browser_protocol_title(browser),
            ArtifactProviderStateSnapshot::Browser(ArtifactBrowserSnapshot {
                current_url: browser.current_display_url().to_owned(),
                page_title: browser_protocol_title(browser),
                phase: browser.phase.phase().into(),
                refreshing: matches!(browser.phase, artifact::BrowserPhase::Loading { .. })
                    && browser.controller_event_sequence > 1,
                can_go_back: browser.can_go_back(),
                can_go_forward: browser.can_go_forward(),
                controller_generation: browser.controller_generation,
                mount_generation: browser_mount_generation.unwrap_or(1),
                environment_scope: match browser.environment.scope {
                    artifact::BrowserEnvironmentScope::AppWide => "all-browsers",
                    artifact::BrowserEnvironmentScope::WorkspaceProject => "per-project",
                    artifact::BrowserEnvironmentScope::Chat => "per-chat-session",
                    artifact::BrowserEnvironmentScope::None => "none",
                }
                .into(),
                pending_operation: browser_pending_operation,
                pending_target_url: browser_pending_target_url,
                notices: browser_notices,
            }),
        ),
        artifact::ArtifactState::Terminal(terminal) => {
            let (status_message, shell_replaced) = match &terminal.status {
                artifact::TerminalCommandStatus::Failed { message, .. }
                | artifact::TerminalCommandStatus::Recovery { message, .. } => (
                    Some(message.clone()),
                    matches!(
                        &terminal.status,
                        artifact::TerminalCommandStatus::Recovery { .. }
                    ),
                ),
                artifact::TerminalCommandStatus::Interrupted { shell_replaced, .. } => {
                    (Some("The command was interrupted.".into()), *shell_replaced)
                }
                _ => (None, false),
            };
            let mut phase = terminal.status.phase().to_owned();
            if pending_approval_id.is_some()
                && matches!(
                    &terminal.status,
                    artifact::TerminalCommandStatus::Queued { .. }
                )
            {
                phase = "approvalWaiting".into();
            }
            let running = matches!(
                &terminal.status,
                artifact::TerminalCommandStatus::Running {
                    stop_requested_at_ms: None,
                    ..
                }
            );
            let stdin_ready = matches!(
                &terminal.status,
                artifact::TerminalCommandStatus::Running {
                    stdin_ready: true,
                    stop_requested_at_ms: None,
                    ..
                }
            );
            let prompt_ready = matches!(
                &terminal.status,
                artifact::TerminalCommandStatus::Completed { .. }
                    | artifact::TerminalCommandStatus::Interrupted {
                        shell_replaced: false,
                        ..
                    }
            ) && terminal.process.shell_process_id.is_some();
            (
                format!("$ {}", terminal.command),
                ArtifactProviderStateSnapshot::Terminal(ArtifactTerminalSnapshot {
                    terminal_session_id: terminal.identity.terminal_session_id.clone(),
                    command_id: terminal.identity.command_id.clone(),
                    command_sequence: terminal.identity.command_sequence,
                    command: terminal.command.clone(),
                    working_directory_display: terminal.working_directory_display.clone(),
                    shell_path: terminal.process.shell_path.clone(),
                    environment_id: terminal.process.environment_id.clone(),
                    environment_generation: terminal.process.environment_generation,
                    process_generation: terminal.process.process_generation,
                    shell_process_id: terminal.process.shell_process_id,
                    foreground_process_group_id: terminal.process.foreground_process_group_id,
                    columns: terminal.dimensions.columns,
                    rows: terminal.dimensions.rows,
                    output_base64: terminal.output.retained_base64.clone(),
                    output_text: terminal.output.safe_text().map_err(|_| {
                        ProtocolError::new(
                            ProtocolErrorCode::InvalidPayload,
                            "Terminal semantic output is unavailable",
                            false,
                        )
                    })?,
                    output_sequence: terminal.output.sequence,
                    retained_bytes: terminal.output.retained_bytes,
                    dropped_bytes: terminal.output.dropped_bytes,
                    phase,
                    exit_code: terminal.status.exit_code(),
                    status_message,
                    stdin_ready,
                    stop_available: running && pending_approval_id.is_none(),
                    prompt_ready,
                    shell_replaced,
                }),
            )
        }
        artifact::ArtifactState::Unknown(_) => (
            record.provider.label.clone(),
            ArtifactProviderStateSnapshot::Unknown,
        ),
    };
    let focus_supported = record.provider.focus == artifact::ArtifactFocusCapability::Focusable
        && !matches!(
            &record.lifecycle,
            artifact::ArtifactLifecycle::Loading { .. }
                | artifact::ArtifactLifecycle::UnknownVersion { .. }
        );
    Ok(ArtifactSnapshot {
        artifact_id: protocol::ArtifactId::new(record.artifact_id.clone())?,
        project_id: ProjectId::new(record.project_id.clone())?,
        session_id: SessionId::new(record.session_id.clone())?,
        provider_type: record.provider.type_id.clone(),
        provider_version: record.provider.schema_version,
        state_schema_version: record.schema_version,
        record_revision: record.record_revision,
        title,
        focus_supported,
        status: artifact_status(&record, pending_approval_id.as_deref()),
        pending_approval_id,
        source_label: artifact_source_label(&record.source),
        resource_version,
        history,
        provider_state,
    })
}

fn browser_notice_from_native_event(
    event: &browser::native::NativeBrowserEvent,
) -> Option<ArtifactBrowserNoticeSnapshot> {
    let (kind, title, message) = match &event.kind {
        browser::native::NativeBrowserEventKind::PopupBlocked { .. } => (
            "warning",
            "Popup blocked",
            "A new-window request was blocked. Browser sub-tabs are not available.",
        ),
        browser::native::NativeBrowserEventKind::DownloadBlocked { .. } => (
            "warning",
            "Download blocked",
            "A website download was blocked because no approved download target exists.",
        ),
        browser::native::NativeBrowserEventKind::FormSubmissionBlocked { .. } => (
            "warning",
            "Form submission blocked",
            "This Browser build does not replay POST form submissions across the authorization boundary.",
        ),
        browser::native::NativeBrowserEventKind::MediaPermissionPrompt { origin, permission } => {
            return Some(ArtifactBrowserNoticeSnapshot {
                id: format!(
                    "browser-notice-{}-{}",
                    event.controller_generation, event.notice_sequence
                ),
                kind: "permission".into(),
                title: format!("{permission} requested"),
                message: format!(
                    "{origin} requested {permission}. WebKit and macOS own the permission prompt."
                ),
            });
        }
        browser::native::NativeBrowserEventKind::MediaPermissionDenied { origin, permission } => {
            return Some(ArtifactBrowserNoticeSnapshot {
                id: format!(
                    "browser-notice-{}-{}",
                    event.controller_generation, event.notice_sequence
                ),
                kind: "warning".into(),
                title: format!("{permission} denied"),
                message: format!(
                    "C4OS policy denied {permission} for {origin}; no WebKit device grant was issued."
                ),
            });
        }
        browser::native::NativeBrowserEventKind::ControllerFailed { .. } => (
            "error",
            "Browser unavailable",
            "The native Browser controller could not complete the operation.",
        ),
        _ => return None,
    };
    Some(ArtifactBrowserNoticeSnapshot {
        id: format!(
            "browser-notice-{}-{}",
            event.controller_generation, event.notice_sequence
        ),
        kind: kind.into(),
        title: title.into(),
        message: message.into(),
    })
}

fn complete_browser_profile_clear(
    core: &AppCoreState,
    profile_id: &str,
    operation_id: &str,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let mut registry = core
        .browser_profiles
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let snapshot = registry.snapshot();
    let Some(profile) = snapshot
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
    else {
        return Ok(());
    };
    let browser::profile::PersistentProfileLifecycle::ClearPending {
        operation_id: pending_operation,
        target_data_generation,
    } = &profile.lifecycle
    else {
        return Ok(());
    };
    if pending_operation != operation_id {
        return Err(workspace_state_unavailable(correlation_id));
    }
    registry
        .complete_clear(
            snapshot.generation,
            &profile.scope,
            operation_id,
            *target_data_generation,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    Ok(())
}

fn reconcile_native_browser_events(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let (events, dropped, dropped_state_identities) = {
        let mut queue = core
            .browser_events
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let dropped = queue.take_dropped();
        let dropped_state_identities = queue.take_dropped_state_identities();
        (queue.drain(), dropped, dropped_state_identities)
    };
    let dropped_state_identity_set = dropped_state_identities
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for (artifact_id, controller_generation, _mount_generation) in &dropped_state_identities {
        let Some(document) = scope
            .database
            .artifact_document(artifact_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        else {
            continue;
        };
        let mut record = deserialize_artifact_record(document, correlation_id.clone())?;
        let artifact::ArtifactState::Browser(browser_state) = &mut record.state else {
            continue;
        };
        if browser_state.controller_generation != *controller_generation {
            continue;
        }
        let observed_at_ms = current_time_ms()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .max(browser_state.version.observed_at_ms);
        browser_state
            .begin_recovery(
                artifact::BrowserRecoveryCode::ControllerRecreated,
                observed_at_ms,
            )
            .and_then(|_| {
                browser_state.install_controller_generation(
                    browser_state.controller_generation.saturating_add(1),
                    observed_at_ms,
                )
            })
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let expected_revision = record.record_revision;
        advance_artifact_record(
            &mut record,
            artifact::ArtifactHistoryKind::RecoveryChanged,
            observed_at_ms,
        )?;
        persist_artifact_record_to_database(
            &scope.database,
            &record,
            Some(expected_revision),
            correlation_id.clone(),
        )?;
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let mount = state
            .browser_mount_generations
            .entry(artifact_id.clone())
            .or_insert(1);
        *mount = mount.saturating_add(1).max(1);
        state.browser_mounted_artifacts.remove(artifact_id);
        let notices = state
            .browser_notices
            .entry(artifact_id.clone())
            .or_default();
        notices.push(ArtifactBrowserNoticeSnapshot {
            id: format!("browser-event-overflow-{controller_generation}"),
            kind: "error".into(),
            title: "Browser controller recovered".into(),
            message: "Native Browser events exceeded their bound, so C4OS replaced the controller generation before accepting more state."
                .into(),
        });
        if notices.len() > 32 {
            let excess = notices.len() - 32;
            notices.drain(..excess);
        }
    }
    if dropped > 0 {
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        for notices in state.browser_notices.values_mut() {
            notices.push(ArtifactBrowserNoticeSnapshot {
                id: format!("browser-event-overflow-{dropped}"),
                kind: "error".into(),
                title: "Browser event recovery required".into(),
                message: "The native Browser event bound was exceeded; reopen the focused Browser."
                    .into(),
            });
            if notices.len() > 32 {
                let excess = notices.len() - 32;
                notices.drain(..excess);
            }
        }
    }
    for event in events {
        if dropped_state_identity_set.contains(&(
            event.artifact_id.clone(),
            event.controller_generation,
            event.mount_generation,
        )) {
            continue;
        }
        match &event.kind {
            browser::native::NativeBrowserEventKind::DataCleared {
                profile_id,
                operation_id,
            } => {
                complete_browser_profile_clear(
                    core,
                    profile_id,
                    operation_id,
                    correlation_id.clone(),
                )?;
                continue;
            }
            browser::native::NativeBrowserEventKind::DataClearFailed { .. } => {
                continue;
            }
            browser::native::NativeBrowserEventKind::EphemeralDataCleared {
                artifact_id,
                operation_id,
            } => {
                let expected = core
                    .artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                    .browser_ephemeral_clear_operations
                    .remove(artifact_id);
                if expected.as_deref() != Some(operation_id) {
                    continue;
                }
                let Some(document) = scope
                    .database
                    .artifact_document(artifact_id)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                else {
                    continue;
                };
                let mut record = deserialize_artifact_record(document, correlation_id.clone())?;
                let artifact::ArtifactState::Browser(browser_state) = &mut record.state else {
                    continue;
                };
                browser_state
                    .clear_ephemeral_environment(event.observed_at_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let expected_revision = record.record_revision;
                advance_artifact_record(
                    &mut record,
                    artifact::ArtifactHistoryKind::RecoveryChanged,
                    event.observed_at_ms,
                )?;
                persist_artifact_record_to_database(
                    &scope.database,
                    &record,
                    Some(expected_revision),
                    correlation_id.clone(),
                )?;
                let mut state = core
                    .artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let mount = state
                    .browser_mount_generations
                    .entry(artifact_id.clone())
                    .or_insert(1);
                *mount = mount.saturating_add(1).max(1);
                state.browser_mounted_artifacts.remove(artifact_id);
                if state
                    .browser_active_identity
                    .as_ref()
                    .is_some_and(|identity| identity.artifact_id == artifact_id.as_str())
                {
                    state.browser_active_identity = None;
                }
                continue;
            }
            _ => {}
        }
        if let Some(notice) = browser_notice_from_native_event(&event) {
            let mut state = core
                .artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            let notices = state
                .browser_notices
                .entry(event.artifact_id.clone())
                .or_default();
            notices.push(notice);
            if notices.len() > 32 {
                let excess = notices.len() - 32;
                notices.drain(..excess);
            }
        }
        let Some(document) = scope
            .database
            .artifact_document(&event.artifact_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        else {
            continue;
        };
        let mut record = deserialize_artifact_record(document, correlation_id.clone())?;
        let artifact::ArtifactState::Browser(browser_state) = &mut record.state else {
            continue;
        };
        if browser_state.controller_generation != event.controller_generation {
            continue;
        }
        if let browser::native::NativeBrowserEventKind::NavigationRequested {
            request_id,
            display_url,
            navigation_sha256,
        } = &event.kind
        {
            let mut state = core
                .artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            let identity = browser::native::NativeBrowserIdentity {
                artifact_id: event.artifact_id.clone(),
                controller_generation: event.controller_generation,
                mount_generation: event.mount_generation,
            };
            if state.browser_active_identity.as_ref() == Some(&identity) {
                if state.browser_native_requests.len() == 64
                    && let Some(oldest_key) = state.browser_native_requests.keys().next().cloned()
                {
                    state.browser_native_requests.remove(&oldest_key);
                }
                state.browser_native_requests.insert(
                    request_id.clone(),
                    PendingNativeBrowserRequest {
                        artifact_id: event.artifact_id.clone(),
                        record_revision: record.record_revision,
                        scope: scope.clone(),
                        controller_generation: event.controller_generation,
                        mount_generation: event.mount_generation,
                        display_url: display_url.clone(),
                        navigation_sha256: navigation_sha256.clone(),
                        observed_at_ms: event.observed_at_ms,
                    },
                );
            }
            continue;
        }
        let mut history_kind = None;
        match &event.kind {
            browser::native::NativeBrowserEventKind::NavigationStarted {
                display_url,
                navigation_sha256,
                kind,
            } => {
                let Some(sequence) = event.state_event_sequence else {
                    continue;
                };
                let target = artifact::BrowserNavigationTarget::parse(display_url)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let metadata = artifact::BrowserControllerEventMeta::new(
                    event.controller_generation,
                    sequence,
                    event.observed_at_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                browser_state
                    .start_navigation_bound(metadata, &target, *kind, navigation_sha256)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                history_kind = Some(artifact::ArtifactHistoryKind::NavigationChanged);
            }
            browser::native::NativeBrowserEventKind::NavigationFinished {
                display_url,
                navigation_sha256,
                kind,
                title,
                ..
            } => {
                let Some(sequence) = event.state_event_sequence else {
                    continue;
                };
                let target = artifact::BrowserNavigationTarget::parse(display_url)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let metadata = artifact::BrowserControllerEventMeta::new(
                    event.controller_generation,
                    sequence,
                    event.observed_at_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                browser_state
                    .mark_ready_bound(
                        metadata,
                        &target,
                        *kind,
                        navigation_sha256,
                        title.as_deref(),
                    )
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                history_kind = Some(artifact::ArtifactHistoryKind::ResourceRefreshed);
            }
            browser::native::NativeBrowserEventKind::NavigationBlocked => {
                let Some(sequence) = event.state_event_sequence else {
                    continue;
                };
                let metadata = artifact::BrowserControllerEventMeta::new(
                    event.controller_generation,
                    sequence,
                    event.observed_at_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                browser_state
                    .cancel_navigation(metadata)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                history_kind = Some(artifact::ArtifactHistoryKind::NavigationChanged);
            }
            browser::native::NativeBrowserEventKind::NavigationFailed { .. } => {
                let Some(sequence) = event.state_event_sequence else {
                    continue;
                };
                let metadata = artifact::BrowserControllerEventMeta::new(
                    event.controller_generation,
                    sequence,
                    event.observed_at_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                browser_state
                    .fail_from_controller(
                        metadata,
                        artifact::BrowserErrorCode::NavigationFailed,
                        true,
                    )
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                history_kind = Some(artifact::ArtifactHistoryKind::RecoveryChanged);
            }
            browser::native::NativeBrowserEventKind::WebContentProcessTerminated { .. } => {
                let Some(sequence) = event.state_event_sequence else {
                    continue;
                };
                let metadata = artifact::BrowserControllerEventMeta::new(
                    event.controller_generation,
                    sequence,
                    event.observed_at_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                browser_state
                    .recover_from_controller(
                        metadata,
                        artifact::BrowserRecoveryCode::WebContentProcessTerminated,
                    )
                    .and_then(|_| {
                        browser_state.install_controller_generation(
                            event.controller_generation.saturating_add(1),
                            event.observed_at_ms,
                        )
                    })
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let mut state = core
                    .artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                let mount = state
                    .browser_mount_generations
                    .entry(event.artifact_id.clone())
                    .or_insert(1);
                *mount = mount.saturating_add(1).max(1);
                history_kind = Some(artifact::ArtifactHistoryKind::RecoveryChanged);
            }
            browser::native::NativeBrowserEventKind::ControllerFailed { .. } => {
                let transition = match browser_state.phase {
                    artifact::BrowserPhase::Queued { .. } => browser_state.fail_before_controller(
                        artifact::BrowserErrorCode::ControllerUnavailable,
                        true,
                        event.observed_at_ms,
                    ),
                    _ => browser_state.begin_recovery(
                        artifact::BrowserRecoveryCode::ControllerRecreated,
                        event.observed_at_ms,
                    ),
                };
                transition.map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                history_kind = Some(artifact::ArtifactHistoryKind::RecoveryChanged);
            }
            _ => {}
        }
        if let Some(kind) = history_kind {
            let expected_revision = record.record_revision;
            let rebind_native_request = matches!(
                &event.kind,
                browser::native::NativeBrowserEventKind::NavigationBlocked
            );
            advance_artifact_record(&mut record, kind, event.observed_at_ms)?;
            persist_artifact_record_to_database(
                &scope.database,
                &record,
                Some(expected_revision),
                correlation_id.clone(),
            )?;
            if rebind_native_request {
                let mut state = core
                    .artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                for request in state
                    .browser_native_requests
                    .values_mut()
                    .filter(|request| {
                        request.artifact_id.as_str() == event.artifact_id.as_str()
                            && request.controller_generation == event.controller_generation
                            && request.mount_generation == event.mount_generation
                            && request.record_revision == expected_revision
                    })
                {
                    request.record_revision = record.record_revision;
                }
            }
        }
    }
    Ok(())
}

fn reconcile_browser_profile_generations(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    records: &mut [artifact::ArtifactRecord],
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let profiles = core
        .browser_profiles
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .snapshot()
        .profiles;
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    for record in records {
        let artifact::ArtifactState::Browser(browser_state) = &mut record.state else {
            continue;
        };
        let Some(profile_scope) =
            persistent_profile_scope_for_reference(&browser_state.environment)
        else {
            continue;
        };
        let Some(profile) = profiles.iter().find(|profile| {
            profile.scope == profile_scope
                && matches!(
                    profile.lifecycle,
                    browser::profile::PersistentProfileLifecycle::Ready
                )
        }) else {
            continue;
        };
        if profile.data_generation <= browser_state.environment.generation {
            continue;
        }
        let observed_at_ms = now_ms.max(browser_state.version.observed_at_ms);
        browser_state
            .rebind_cleared_environment(profile.data_generation, observed_at_ms)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let expected_revision = record.record_revision;
        advance_artifact_record(
            record,
            artifact::ArtifactHistoryKind::RecoveryChanged,
            observed_at_ms,
        )?;
        persist_artifact_record_to_database(
            &scope.database,
            record,
            Some(expected_revision),
            correlation_id.clone(),
        )?;
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let mount = state
            .browser_mount_generations
            .entry(record.artifact_id.clone())
            .or_insert(1);
        *mount = mount.saturating_add(1).max(1);
        state.browser_mounted_artifacts.remove(&record.artifact_id);
        if state
            .browser_active_identity
            .as_ref()
            .is_some_and(|identity| identity.artifact_id == record.artifact_id.as_str())
        {
            state.browser_active_identity = None;
        }
    }
    Ok(())
}

fn build_artifact_workspace_snapshot(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<ArtifactWorkspaceSnapshot, ProtocolError> {
    let workspace = active_workspace_snapshot(core, correlation_id.clone())?;
    let workspace_id = workspace
        .workspace
        .as_ref()
        .map(|record| record.workspace_id.clone())
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let conversation = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .clone();
    let active_project_id = conversation.active_project_id.clone();
    let active_session_id = conversation.active_session_id.clone();
    let project_name = active_project_id
        .as_deref()
        .and_then(|project_id| {
            workspace
                .projects
                .iter()
                .find(|project| project.project_id == project_id)
        })
        .map(|project| project.display_name.as_str())
        .unwrap_or("Project");
    let database = core
        .active_workspace
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .as_ref()
        .map(|active| Arc::clone(active.database_actor()))
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let documents = if let Some(session_id) = active_session_id.as_deref() {
        database
            .artifact_documents_for_session(session_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
    } else {
        Vec::new()
    };
    let mut records = documents
        .into_iter()
        .map(|document| deserialize_artifact_record(document, correlation_id.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    if active_project_id.is_some() && active_session_id.is_some() {
        let scope = active_artifact_scope(core, correlation_id.clone())?;
        let now_ms =
            current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        reconcile_native_browser_events(core, &scope, correlation_id.clone())?;
        records = scope
            .database
            .artifact_documents_for_session(&scope.session_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .into_iter()
            .map(|document| deserialize_artifact_record(document, correlation_id.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        reconcile_browser_profile_generations(core, &scope, &mut records, correlation_id.clone())?;
        reconcile_active_terminal_records(core, &scope, &mut records, correlation_id.clone())?;
        for record in &mut records {
            if record.workspace_id == workspace_id
                && active_project_id.as_deref() == Some(record.project_id.as_str())
                && active_session_id.as_deref() == Some(record.session_id.as_str())
            {
                reconcile_interrupted_terminal_approval(
                    core,
                    &scope,
                    record,
                    now_ms,
                    correlation_id.clone(),
                )?;
                reconcile_interrupted_file_save_request(
                    core,
                    &scope,
                    record,
                    now_ms,
                    correlation_id.clone(),
                )?;
            }
        }
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        for record in &records {
            if matches!(record.state, artifact::ArtifactState::Browser(_)) {
                state
                    .browser_mount_generations
                    .entry(record.artifact_id.clone())
                    .or_insert(1);
            }
        }
    }
    let (
        pending_by_artifact,
        browser_pending_operations,
        browser_mount_generations,
        browser_notices,
    ) = {
        let state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let pending = state
            .pending_writes
            .iter()
            .map(|(prompt_id, pending)| (pending.artifact_id.clone(), prompt_id.clone()))
            .chain(
                state
                    .pending_terminal_actions
                    .iter()
                    .map(|(prompt_id, pending)| (pending.artifact_id.clone(), prompt_id.clone())),
            )
            .chain(
                state
                    .pending_browser_actions
                    .iter()
                    .map(|(prompt_id, pending)| (pending.artifact_id.clone(), prompt_id.clone())),
            )
            .collect::<BTreeMap<_, _>>();
        let browser_operations = state
            .pending_browser_actions
            .values()
            .map(|pending| {
                let operation = match &pending.payload {
                    PendingArtifactBrowserPayload::Open { .. } => "open",
                    PendingArtifactBrowserPayload::Navigate { intent, .. } => match intent {
                        artifact::BrowserNavigationIntent::Back => "back",
                        artifact::BrowserNavigationIntent::Forward => "forward",
                        artifact::BrowserNavigationIntent::Refresh => "refresh",
                    },
                    PendingArtifactBrowserPayload::NavigateTo { .. } => "reply-navigation",
                    PendingArtifactBrowserPayload::Recover { .. } => "refresh",
                    PendingArtifactBrowserPayload::NativeRequest { .. } => "website-navigation",
                    PendingArtifactBrowserPayload::ClearData { .. } => "clear-data",
                };
                let target_url = pending
                    .action
                    .arguments
                    .get("displayUrl")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned);
                (
                    pending.artifact_id.clone(),
                    (operation.to_owned(), target_url),
                )
            })
            .collect::<BTreeMap<_, _>>();
        (
            pending,
            browser_operations,
            state.browser_mount_generations.clone(),
            state.browser_notices.clone(),
        )
    };
    let artifacts = records
        .into_iter()
        .filter_map(|record| {
            (active_project_id.as_deref() == Some(record.project_id.as_str())
                && active_session_id.as_deref() == Some(record.session_id.as_str()))
            .then(|| {
                let browser_pending = browser_pending_operations.get(&record.artifact_id).cloned();
                artifact_protocol_snapshot(
                    record.clone(),
                    pending_by_artifact.get(&record.artifact_id).cloned(),
                    browser_pending
                        .as_ref()
                        .map(|(operation, _)| operation.clone()),
                    browser_pending.and_then(|(_, target_url)| target_url),
                    project_name,
                    browser_mount_generations.get(&record.artifact_id).copied(),
                    browser_notices
                        .get(&record.artifact_id)
                        .cloned()
                        .unwrap_or_default(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let focused_artifact_id = database
        .artifact_ui_state()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .map(|record| {
            serde_json::from_str::<artifact::ArtifactWorkspaceUiState>(&record.canonical_document)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))
        })
        .transpose()?
        .and_then(|state| state.focused_artifact_id)
        .filter(|artifact_id| {
            artifacts.iter().any(|artifact| {
                artifact.artifact_id.as_str() == artifact_id && artifact.focus_supported
            })
        })
        .map(protocol::ArtifactId::new)
        .transpose()?;
    Ok(ArtifactWorkspaceSnapshot {
        protocol_version: protocol::PROTOCOL_VERSION,
        generation: StateGeneration(workspace.generation),
        authority: "rust-core".into(),
        workspace_id: Some(WorkspaceId::new(workspace_id)?),
        active_project_id: active_project_id.map(ProjectId::new).transpose()?,
        active_session_id: active_session_id.map(SessionId::new).transpose()?,
        focused_artifact_id,
        artifacts,
    })
}

/// Reconciles completed model output for immutable File Reply turns into the
/// durable proposal state. The source is the Rust-owned session/event journal;
/// no renderer payload participates. A stale artifact revision or live target
/// is ignored, leaving the assistant response visible without creating an
/// actionable proposal against changed bytes.
fn reconcile_completed_file_reply_proposals(
    core: &AppCoreState,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    use runtime::session::RunAttemptStatus;

    let scope = active_artifact_scope(core, correlation_id.clone())?;
    let sessions = core
        .runtime
        .durable_sessions()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let Some(session) = sessions
        .iter()
        .find(|session| session.session_id == scope.session_id)
    else {
        return Ok(());
    };
    let mut candidates = Vec::new();
    for attempt in &session.attempts {
        let RunAttemptStatus::Completed { completed_at_ms } = attempt.status else {
            continue;
        };
        let Some(turn) = session
            .turns
            .iter()
            .find(|turn| turn.turn_id == attempt.turn_id)
        else {
            continue;
        };
        let Some(context) = turn
            .reply_context
            .as_ref()
            .and_then(|reply| reply.artifact_context.as_ref())
            .filter(|context| context.provider_type == "file")
        else {
            continue;
        };
        let assistant_markdown = project_attempt_snapshot(attempt)?.assistant_markdown;
        let Some(proposed_content) =
            artifact::file::file_reply_proposed_content(&assistant_markdown)
        else {
            continue;
        };
        candidates.push((
            completed_at_ms,
            attempt.attempt_id.as_str(),
            context,
            proposed_content,
        ));
    }
    candidates.sort_by_key(|(completed_at_ms, ..)| *completed_at_ms);

    for (completed_at_ms, attempt_id, context, proposed_content) in candidates {
        let Some(document) = scope
            .database
            .artifact_document(&context.artifact_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        else {
            continue;
        };
        let mut record = deserialize_artifact_record(document, correlation_id.clone())?;
        if record.workspace_id != scope.workspace_id
            || record.project_id != scope.project_id
            || record.session_id != scope.session_id
            || record.record_revision != context.artifact_record_revision
        {
            continue;
        }
        let artifact::ArtifactState::File(file) = &mut record.state else {
            continue;
        };
        if file.live_version.as_resource_version() != context.captured_live_version {
            continue;
        }
        let proposal_digest = sha256_bytes(attempt_id.as_bytes());
        let proposal_id = format!("file-proposal-{}", &proposal_digest[7..39]);
        if file
            .proposal
            .as_ref()
            .is_some_and(|proposal| proposal.proposal_id == proposal_id)
        {
            continue;
        }
        let diff = artifact::file::file_reply_unified_diff(
            &file.resource.project_relative_path,
            &file.content,
            &proposed_content,
        );
        if diff.is_empty() {
            continue;
        }
        file.install_proposal(artifact::FileProposal {
            proposal_id,
            base_live_version: file.live_version.clone(),
            proposed_content,
            unified_diff: diff,
            status: artifact::FileProposalStatus::Pending,
            created_at_ms: completed_at_ms,
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let expected_revision = record.record_revision;
        advance_artifact_record(
            &mut record,
            artifact::ArtifactHistoryKind::ProposalChanged,
            completed_at_ms,
        )?;
        persist_artifact_record(
            &scope,
            &record,
            Some(expected_revision),
            correlation_id.clone(),
        )?;
    }
    Ok(())
}

/// Reconciles one explicit Browser Reply navigation envelope against the
/// immutable captured artifact revision, then sends the exact target through
/// the same Action Gateway used by direct Browser operations.
fn reconcile_completed_browser_reply_navigation(
    core: &AppCoreState,
    app: &tauri::AppHandle,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    use runtime::session::RunAttemptStatus;

    let scope = active_artifact_scope(core, correlation_id.clone())?;
    let sessions = core
        .runtime
        .durable_sessions()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let Some(session) = sessions
        .iter()
        .find(|session| session.session_id == scope.session_id)
    else {
        return Ok(());
    };
    let mut candidates = Vec::new();
    for attempt in &session.attempts {
        let RunAttemptStatus::Completed { completed_at_ms } = attempt.status else {
            continue;
        };
        let Some(turn) = session
            .turns
            .iter()
            .find(|turn| turn.turn_id == attempt.turn_id)
        else {
            continue;
        };
        let Some(context) = turn
            .reply_context
            .as_ref()
            .and_then(|reply| reply.artifact_context.as_ref())
            .filter(|context| context.provider_type == "browser")
        else {
            continue;
        };
        let assistant_markdown = project_attempt_snapshot(attempt)?.assistant_markdown;
        let Some(target) = artifact::browser::browser_reply_navigation_target(&assistant_markdown)
        else {
            continue;
        };
        candidates.push((
            completed_at_ms,
            attempt.attempt_id.as_str(),
            context,
            target,
        ));
    }
    candidates.sort_by_key(|(completed_at_ms, ..)| *completed_at_ms);

    for (completed_at_ms, attempt_id, context, target) in candidates {
        if core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .pending_browser_actions
            .values()
            .any(|pending| pending.artifact_id == context.artifact_id)
        {
            continue;
        }
        let Some(document) = scope
            .database
            .artifact_document(&context.artifact_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        else {
            continue;
        };
        let mut record = deserialize_artifact_record(document, correlation_id.clone())?;
        if record.workspace_id != scope.workspace_id
            || record.project_id != scope.project_id
            || record.session_id != scope.session_id
            || record.record_revision != context.artifact_record_revision
        {
            continue;
        }
        let artifact::ArtifactState::Browser(browser_state) = &mut record.state else {
            continue;
        };
        if browser_state.as_resource_version() != context.captured_live_version
            || browser_state.has_processed_reply_attempt(attempt_id)
            || !matches!(
                browser_state.phase,
                artifact::BrowserPhase::Ready { .. }
                    | artifact::BrowserPhase::Error { .. }
                    | artifact::BrowserPhase::Recovery { .. }
            )
        {
            continue;
        }
        let observed_at_ms = completed_at_ms.max(browser_state.version.observed_at_ms);
        browser_state
            .record_reply_attempt(attempt_id, observed_at_ms)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let controller_generation = browser_state.controller_generation;
        let expected_revision = record.record_revision;
        advance_artifact_record(
            &mut record,
            artifact::ArtifactHistoryKind::ReplySubmitted,
            observed_at_ms,
        )?;
        persist_artifact_record(
            &scope,
            &record,
            Some(expected_revision),
            correlation_id.clone(),
        )?;
        let mount_generation = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .browser_mount_generations
            .get(&record.artifact_id)
            .copied()
            .unwrap_or(1);
        let (pending, facts) = prepare_browser_action(
            core,
            &scope,
            &record,
            PendingArtifactBrowserPayload::NavigateTo {
                target: target.clone(),
                controller_generation,
                mount_generation,
            },
            "browser.reply.navigate",
            target.display_url(),
            &target.navigation_sha256(),
            ActionInitiator::Agent,
            ActionRequestOrigin::ArtifactReplyProposal,
            observed_at_ms,
            correlation_id.clone(),
        )?;
        propose_browser_action(
            core,
            app,
            pending,
            facts,
            observed_at_ms,
            correlation_id.clone(),
        )?;
    }
    Ok(())
}

fn reconcile_pending_native_browser_requests(
    core: &AppCoreState,
    app: &tauri::AppHandle,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let active_scope = active_artifact_scope(core, correlation_id.clone())?;
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let mut candidates = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .browser_native_requests
        .iter()
        .map(|(request_id, request)| (request_id.clone(), request.clone()))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(_, request)| request.observed_at_ms);

    for (request_id, request) in candidates {
        let identity = browser::native::NativeBrowserIdentity {
            artifact_id: request.artifact_id.clone(),
            controller_generation: request.controller_generation,
            mount_generation: request.mount_generation,
        };
        let same_scope = request.scope.workspace_id == active_scope.workspace_id
            && request.scope.project_id == active_scope.project_id
            && request.scope.session_id == active_scope.session_id;
        let active_identity = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .browser_active_identity
            .as_ref()
            == Some(&identity);
        let expired = request.observed_at_ms < now_ms.saturating_sub(120_000);
        if !same_scope || !active_identity || expired {
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                .browser_native_requests
                .remove(&request_id);
            let _ = browser::native::dispatch_action(
                app,
                Arc::clone(&core.browser_events),
                identity,
                browser::native::NativeBrowserAction::DiscardNavigationRequest { request_id },
            );
            continue;
        }
        if core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .pending_browser_actions
            .values()
            .any(|pending| pending.artifact_id == request.artifact_id)
        {
            continue;
        }
        let record = load_scoped_artifact_record(
            &request.scope,
            &request.artifact_id,
            correlation_id.clone(),
        )?;
        let current_identity_matches = matches!(
            &record.state,
            artifact::ArtifactState::Browser(browser)
                if browser.controller_generation == request.controller_generation
                    && record.record_revision == request.record_revision
        );
        if !current_identity_matches {
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                .browser_native_requests
                .remove(&request_id);
            let _ = browser::native::dispatch_action(
                app,
                Arc::clone(&core.browser_events),
                identity,
                browser::native::NativeBrowserAction::DiscardNavigationRequest { request_id },
            );
            continue;
        }
        let proposed_at_ms = now_ms.max(request.observed_at_ms);
        let (pending, facts) = prepare_browser_action(
            core,
            &request.scope,
            &record,
            PendingArtifactBrowserPayload::NativeRequest {
                request_id: request_id.clone(),
                controller_generation: request.controller_generation,
                mount_generation: request.mount_generation,
                navigation_sha256: request.navigation_sha256.clone(),
            },
            "browser.page.navigate",
            &request.display_url,
            &request.navigation_sha256,
            ActionInitiator::Runtime,
            ActionRequestOrigin::RuntimeTool,
            proposed_at_ms,
            correlation_id.clone(),
        )?;
        propose_browser_action(
            core,
            app,
            pending,
            facts,
            proposed_at_ms,
            correlation_id.clone(),
        )?;
    }
    Ok(())
}

fn save_artifact_ui_state(
    scope: &ActiveArtifactScope,
    focused: Option<&artifact::ArtifactRecord>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<u64, ProtocolError> {
    let current = scope
        .database
        .artifact_ui_state()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let (mut state, expected_revision) = if let Some(current) = current {
        let state =
            serde_json::from_str::<artifact::ArtifactWorkspaceUiState>(&current.canonical_document)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        (state, Some(current.revision))
    } else {
        (
            artifact::ArtifactWorkspaceUiState {
                schema_version: artifact::ARTIFACT_SCHEMA_VERSION,
                workspace_id: scope.workspace_id.clone(),
                revision: 1,
                focused_artifact_id: focused.map(|record| record.artifact_id.clone()),
                updated_at_ms: now_ms,
            },
            None,
        )
    };
    if expected_revision.is_some() {
        match focused {
            Some(record) => state.focus(record, now_ms),
            None => state.clear_focus(now_ms),
        }
        .map_err(|_| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "Artifact focus changed before the operation",
                true,
            )
        })?;
    } else {
        state
            .validate()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        if let Some(record) = focused {
            state.validate_against(record).map_err(|_| {
                platform_boundary_error(
                    correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Artifact cannot receive focus",
                    false,
                )
            })?;
        }
    }
    let canonical_document = serde_json::to_string(&state)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    scope
        .database
        .save_artifact_ui_state(
            core::database::WorkspaceArtifactUiStateRecord {
                workspace_id: scope.workspace_id.clone(),
                revision: state.revision,
                canonical_document,
                updated_at_ms: state.updated_at_ms,
            },
            expected_revision,
        )
        .map_err(|error| {
            platform_boundary_error(
                correlation_id,
                if matches!(error, core::database::DatabaseError::Conflict(_)) {
                    ProtocolErrorCode::Conflict
                } else {
                    ProtocolErrorCode::Internal
                },
                "Artifact focus could not be committed",
                true,
            )
        })
}

fn clear_persisted_artifact_focus(
    database: &core::database::DatabaseActor,
    workspace_id: &str,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<Option<u64>, ProtocolError> {
    let Some(current) = database
        .artifact_ui_state()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
    else {
        return Ok(None);
    };
    let mut state =
        serde_json::from_str::<artifact::ArtifactWorkspaceUiState>(&current.canonical_document)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    if state.focused_artifact_id.is_none() {
        return Ok(None);
    }
    state.clear_focus(now_ms).map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "Artifact focus changed before Chat navigation",
            true,
        )
    })?;
    let canonical_document = serde_json::to_string(&state)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    database
        .save_artifact_ui_state(
            core::database::WorkspaceArtifactUiStateRecord {
                workspace_id: workspace_id.into(),
                revision: state.revision,
                canonical_document,
                updated_at_ms: state.updated_at_ms,
            },
            Some(current.revision),
        )
        .map(Some)
        .map_err(|_| {
            platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "Artifact focus could not be cleared for Chat navigation",
                true,
            )
        })
}

#[tauri::command]
fn artifact_snapshot(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _artifact_operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    reconcile_native_browser_events(&core, &scope, request.correlation_id.clone())?;
    reconcile_pending_native_browser_requests(&core, &app, request.correlation_id.clone())?;
    reconcile_completed_file_reply_proposals(&core, request.correlation_id.clone())?;
    reconcile_completed_browser_reply_navigation(&core, &app, request.correlation_id.clone())?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn terminal_key(scope: &ActiveArtifactScope) -> Result<TerminalSessionKey, ProtocolError> {
    TerminalSessionKey::new(scope.workspace_id.clone(), scope.session_id.clone()).map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "The Terminal session identity is invalid",
            false,
        )
    })
}

fn load_scoped_artifact_record(
    scope: &ActiveArtifactScope,
    artifact_id: &str,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::ArtifactRecord, ProtocolError> {
    let document = scope
        .database
        .artifact_document(artifact_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .ok_or_else(|| {
            platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Terminal artifact is unavailable",
                false,
            )
        })?;
    let record = deserialize_artifact_record(document, correlation_id.clone())?;
    if record.workspace_id != scope.workspace_id
        || record.project_id != scope.project_id
        || record.session_id != scope.session_id
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The Terminal artifact owner changed",
            true,
        ));
    }
    Ok(record)
}

fn terminal_command_contains_secret_material(command: &str) -> bool {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        let normalized = token
            .trim_matches(|character: char| matches!(character, '\'' | '"' | ',' | ';'))
            .to_ascii_lowercase();
        if normalized.starts_with("authorization:") {
            let following = tokens
                .iter()
                .skip(index + 1)
                .take(2)
                .copied()
                .collect::<Vec<_>>()
                .join(" ");
            if !normalized.contains('$') && !following.contains('$') {
                return true;
            }
        }
        if normalized.contains("begin-private-key") || normalized.contains("begin-rsa-private-key")
        {
            return true;
        }
        if let Some((name, value)) = normalized.split_once('=') {
            let name = name.trim_start_matches('-').replace('-', "_");
            if terminal_secret_name(&name) && !value.is_empty() && !value.starts_with('$') {
                return true;
            }
        }
        let option = normalized.trim_start_matches('-').replace('-', "_");
        if matches!(
            option.as_str(),
            "password" | "passwd" | "token" | "secret" | "api_key" | "access_token"
        ) && tokens.get(index + 1).is_some_and(|value| {
            !value.is_empty() && !value.starts_with('$') && !value.starts_with('-')
        }) {
            return true;
        }
        if let Some((_, authority)) = normalized.split_once("://")
            && authority
                .split('@')
                .next()
                .is_some_and(|userinfo| userinfo.contains(':'))
            && authority.contains('@')
        {
            return true;
        }
    }
    false
}

fn terminal_secret_name(name: &str) -> bool {
    let normalized = name.trim().trim_start_matches('-').replace('-', "_");
    normalized == "passwd"
        || normalized == "authorization"
        || normalized.contains("password")
        || normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("access_key")
}

fn terminal_command_references_secret_environment(command: &str) -> bool {
    let bytes = command.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'$' {
            index += 1;
            continue;
        }
        let mut start = index + 1;
        let braced = bytes.get(start) == Some(&b'{');
        if braced {
            start += 1;
        }
        let mut end = start;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            end += 1;
        }
        if end > start
            && (!braced || bytes.get(end) == Some(&b'}'))
            && terminal_secret_name(&command[start..end].to_ascii_lowercase())
        {
            return true;
        }
        index = end.max(index + 1);
    }
    false
}

fn terminal_line_contains_secret_material(line: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(line).to_ascii_lowercase();
    if lower.contains("private key-----") || lower.contains("authorization: bearer ") {
        return true;
    }
    if let Some((name, value)) = lower.split_once('=').or_else(|| lower.split_once(':'))
        && terminal_secret_name(name.split_whitespace().last().unwrap_or(name))
        && !value.trim().is_empty()
    {
        return true;
    }
    [
        "password",
        "passwd",
        "token",
        "secret",
        "api_key",
        "api-key",
        "apikey",
        "access_token",
        "access-token",
        "authorization",
    ]
    .iter()
    .any(|marker| {
        lower.find(marker).is_some_and(|position| {
            let suffix = &lower[position + marker.len()..];
            let Some(separator) = suffix.chars().next() else {
                return false;
            };
            if !matches!(separator, ':' | '=') {
                return false;
            }
            !suffix[separator.len_utf8()..].trim().is_empty()
        })
    })
}

fn redact_terminal_output_line(line: &[u8]) -> Vec<u8> {
    if !terminal_line_contains_secret_material(line) {
        return line.to_vec();
    }
    let ending = if line.ends_with(b"\r\n") {
        b"\r\n".as_slice()
    } else if line.ends_with(b"\r") {
        b"\r".as_slice()
    } else if line.ends_with(b"\n") {
        b"\n".as_slice()
    } else {
        b"".as_slice()
    };
    let mut redacted = b"[sensitive Terminal output redacted]".to_vec();
    redacted.extend_from_slice(ending);
    redacted
}

fn oversized_terminal_output_line(ending: &[u8]) -> Vec<u8> {
    let mut redacted = b"[oversized Terminal output line redacted]".to_vec();
    redacted.extend_from_slice(ending);
    redacted
}

const TERMINAL_SENSITIVE_OUTPUT_TRIGGERS: &[&[u8]] = &[
    b"password",
    b"passwd",
    b"token",
    b"secret",
    b"api_key",
    b"api-key",
    b"apikey",
    b"api_token",
    b"api-token",
    b"access_key",
    b"access-key",
    b"secret_access_key",
    b"secret-access-key",
    b"access_token",
    b"access-token",
    b"authorization",
    b"private key-----",
];

fn terminal_output_record_boundary(byte: u8) -> bool {
    matches!(byte, b'\r' | b'\n')
}

fn earliest_terminal_sensitive_hold(bytes: &[u8]) -> Option<usize> {
    TERMINAL_SENSITIVE_OUTPUT_TRIGGERS
        .iter()
        .filter_map(|trigger| {
            bytes
                .windows(trigger.len())
                .enumerate()
                .find_map(|(position, window)| {
                    if !window
                        .iter()
                        .zip(trigger.iter())
                        .all(|(actual, expected)| actual.to_ascii_lowercase() == *expected)
                    {
                        return None;
                    }
                    if *trigger == b"private key-----" {
                        return Some(position);
                    }
                    bytes
                        .get(position + trigger.len())
                        .is_none_or(|byte| matches!(byte, b':' | b'='))
                        .then_some(position)
                })
        })
        .min()
}

fn terminal_sensitive_trigger_suffix(bytes: &[u8]) -> usize {
    let maximum = TERMINAL_SENSITIVE_OUTPUT_TRIGGERS
        .iter()
        .map(|trigger| trigger.len())
        .max()
        .unwrap_or(0)
        .min(bytes.len());
    (1..=maximum)
        .rev()
        .find(|length| {
            let suffix = &bytes[bytes.len() - length..];
            TERMINAL_SENSITIVE_OUTPUT_TRIGGERS.iter().any(|trigger| {
                suffix.len() < trigger.len()
                    && suffix
                        .iter()
                        .zip(trigger.iter())
                        .all(|(actual, expected)| actual.to_ascii_lowercase() == *expected)
            })
        })
        .unwrap_or(0)
}

fn terminal_output_redaction_checkpoint(
    state: &ArtifactApplicationState,
    command_id: &str,
) -> TerminalOutputRedactionCheckpoint {
    TerminalOutputRedactionCheckpoint {
        pending: state.terminal_output_lines.get(command_id).cloned(),
        oversized: state.terminal_redacted_output_lines.contains(command_id),
        sensitive: state.terminal_sensitive_output_lines.contains(command_id),
    }
}

fn restore_terminal_output_redaction(
    state: &mut ArtifactApplicationState,
    command_id: &str,
    checkpoint: TerminalOutputRedactionCheckpoint,
) {
    state.terminal_output_lines.remove(command_id);
    state.terminal_redacted_output_lines.remove(command_id);
    state.terminal_sensitive_output_lines.remove(command_id);
    if let Some(pending) = checkpoint.pending {
        state
            .terminal_output_lines
            .insert(command_id.to_owned(), pending);
    }
    if checkpoint.oversized {
        state
            .terminal_redacted_output_lines
            .insert(command_id.to_owned());
    }
    if checkpoint.sensitive {
        state
            .terminal_sensitive_output_lines
            .insert(command_id.to_owned());
    }
}

fn retain_redacted_terminal_output(
    state: &mut ArtifactApplicationState,
    command_id: &str,
    bytes: &[u8],
    flush: bool,
) -> Vec<u8> {
    let mut pending = state
        .terminal_output_lines
        .remove(command_id)
        .unwrap_or_default();
    let mut oversized = state.terminal_redacted_output_lines.remove(command_id);
    let mut sensitive = state.terminal_sensitive_output_lines.remove(command_id);
    let mut redacted = Vec::with_capacity(bytes.len());
    let mut offset = 0;
    while offset < bytes.len() {
        let boundary = bytes[offset..]
            .iter()
            .position(|byte| terminal_output_record_boundary(*byte));
        let end = boundary.map_or(bytes.len(), |position| offset + position + 1);
        let segment = &bytes[offset..end];
        let record_ended = segment
            .last()
            .is_some_and(|byte| terminal_output_record_boundary(*byte));
        offset = end;
        if oversized {
            if record_ended {
                redacted.extend(oversized_terminal_output_line(
                    &segment[segment.len() - 1..],
                ));
                oversized = false;
            }
            continue;
        }
        if sensitive {
            if record_ended {
                redacted.push(segment[segment.len() - 1]);
                sensitive = false;
            }
            continue;
        }
        pending.extend_from_slice(segment);
        if record_ended {
            redacted.extend(redact_terminal_output_line(&pending));
            pending.clear();
        } else if terminal_line_contains_secret_material(&pending) {
            if let Some(position) = earliest_terminal_sensitive_hold(&pending) {
                redacted.extend_from_slice(&pending[..position]);
            }
            redacted.extend_from_slice(b"[sensitive Terminal output redacted]");
            pending.clear();
            sensitive = true;
        } else if pending.len() > artifact::MAX_TERMINAL_OUTPUT_BYTES {
            pending.clear();
            oversized = true;
        } else if let Some(position) = earliest_terminal_sensitive_hold(&pending) {
            if position > 0 {
                redacted.extend(pending.drain(..position));
            }
        } else {
            let retained = terminal_sensitive_trigger_suffix(&pending);
            let released = pending.len() - retained;
            if released > 0 {
                redacted.extend(pending.drain(..released));
            }
        }
    }
    if flush {
        if oversized {
            redacted.extend(oversized_terminal_output_line(b""));
        } else if sensitive {
            // The redaction marker was emitted as soon as the sensitive record
            // became classifiable. There are no retained secret bytes to add.
        } else if !pending.is_empty() {
            redacted.extend(redact_terminal_output_line(&pending));
        }
    } else {
        if !pending.is_empty() {
            state
                .terminal_output_lines
                .insert(command_id.to_owned(), pending);
        }
        if oversized {
            state
                .terminal_redacted_output_lines
                .insert(command_id.to_owned());
        }
        if sensitive {
            state
                .terminal_sensitive_output_lines
                .insert(command_id.to_owned());
        }
    }
    redacted
}

fn persist_terminal_event_transition(
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    kind: artifact::ArtifactHistoryKind,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let expected_revision = record.record_revision;
    advance_artifact_record(record, kind, now_ms)?;
    persist_artifact_record(scope, record, Some(expected_revision), correlation_id)?;
    Ok(())
}

fn cancel_stale_terminal_controls(
    artifact: &Mutex<ArtifactApplicationState>,
    runtime: &RuntimeApplicationService,
    artifact_id: &str,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let pending = {
        let state = artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        state
            .pending_terminal_actions
            .iter()
            .filter(|(_, pending)| {
                pending.artifact_id == artifact_id
                    && !matches!(&pending.payload, PendingArtifactTerminalPayload::Run { .. })
            })
            .map(|(prompt_id, pending)| (prompt_id.clone(), pending.action.run_id.clone()))
            .collect::<Vec<_>>()
    };
    for (prompt_id, run_id) in &pending {
        let cancelled = runtime
            .coordinator()
            .and_then(|mut coordinator| {
                coordinator
                    .cancel_direct_action_run(run_id, now_ms)
                    .map(|operation| operation.value)
                    .map_err(Into::into)
            })
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        if cancelled == 0 {
            return Err(platform_boundary_error(
                correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The stale Terminal operation could not be cancelled",
                true,
            ));
        }
        artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .pending_terminal_actions
            .remove(prompt_id);
    }
    artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id))?
        .terminal_cleanup_sessions
        .remove(artifact_id);
    Ok(())
}

fn reconcile_active_terminal_records(
    core: &impl TerminalReconciliationResources,
    scope: &ActiveArtifactScope,
    records: &mut [artifact::ArtifactRecord],
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let Some(latest) = records
        .iter()
        .filter_map(|record| match &record.state {
            artifact::ArtifactState::Terminal(terminal) => Some((record, terminal)),
            _ => None,
        })
        .max_by_key(|(_, terminal)| terminal.identity.command_sequence)
    else {
        return Ok(());
    };
    let key = terminal_key(scope)?;
    let mut supervised = core
        .terminal_supervisor()
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .live_session(&key);
    if supervised.is_none()
        && matches!(
            &latest.1.status,
            artifact::TerminalCommandStatus::Running { .. }
        )
    {
        let environment = execution::environment::ExecutionEnvironmentIdentity::new(
            execution::environment::ExecutionEnvironmentKind::Local,
            latest.1.process.environment_id.clone(),
            latest.1.process.environment_generation,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        core.terminal_supervisor()
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .reconcile_restart(TerminalRestartRecord {
                key: key.clone(),
                command: SupervisedTerminalCommandIdentity::new(
                    latest.1.identity.terminal_session_id.clone(),
                    latest.1.identity.command_id.clone(),
                    latest.1.identity.command_sequence,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                process_generation: latest.1.process.process_generation,
                environment,
                dimensions: PtyDimensions::new(
                    latest.1.dimensions.columns,
                    latest.1.dimensions.rows,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                trusted_project_root: scope.project_root.clone(),
                shell_path: PathBuf::from(&latest.1.process.shell_path),
                working_directory: PathBuf::from(&latest.1.working_directory_display),
                persisted_process_id: latest.1.process.shell_process_id,
            })
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        supervised = core
            .terminal_supervisor()
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .live_session(&key);
    }
    let Some(supervised) = supervised else {
        return Ok(());
    };
    let binding = terminal_event_drain_binding(latest.1, &supervised)
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    let cursor_key = (
        scope.workspace_id.clone(),
        scope.session_id.clone(),
        binding.process_generation,
    );
    let cursor = core
        .artifact_state()
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .terminal_event_cursors
        .get(&cursor_key)
        .copied()
        .unwrap_or(0);
    let events = match core
        .terminal_supervisor()
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .drain_events(TerminalDrainRequest {
            key: key.clone(),
            terminal_session_id: binding.terminal_session_id.clone(),
            process_generation: binding.process_generation,
            after_chunk_sequence: cursor,
            maximum_events: MAX_TERMINAL_DRAIN_EVENTS,
            maximum_bytes: MAX_TERMINAL_DRAIN_BYTES,
        }) {
        Ok(events) => events,
        Err(execution::terminal::TerminalError::SessionNotFound) => return Ok(()),
        Err(_) => return Err(workspace_state_unavailable(correlation_id)),
    };
    for event in events {
        let Some(record) = records.iter_mut().find(|record| {
            matches!(
                &record.state,
                artifact::ArtifactState::Terminal(terminal)
                    if terminal.identity.command_id == event.command_id
                        && terminal.identity.command_sequence == event.command_sequence
            )
        }) else {
            return Err(workspace_state_unavailable(correlation_id.clone()));
        };
        let now_ms = current_time_ms()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .max(record.updated_at_ms);
        let command_became_terminal = matches!(
            &event.kind,
            TerminalEventKind::Completed { .. }
                | TerminalEventKind::Interrupted130 { .. }
                | TerminalEventKind::RecoveredInterrupted { .. }
                | TerminalEventKind::Failed { .. }
        );
        let mut event_is_durable = !matches!(&event.kind, TerminalEventKind::Output { .. });
        match event.kind {
            TerminalEventKind::Started {
                process_id,
                foreground_process_group_id,
                working_directory,
            } => {
                let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                    unreachable!()
                };
                if matches!(
                    &terminal.status,
                    artifact::TerminalCommandStatus::Queued { .. }
                ) {
                    terminal.working_directory_display =
                        working_directory.to_string_lossy().into_owned();
                    terminal
                        .mark_running(
                            process_id,
                            u32::try_from(foreground_process_group_id).ok(),
                            true,
                            now_ms,
                        )
                        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                    persist_terminal_event_transition(
                        scope,
                        record,
                        artifact::ArtifactHistoryKind::CommandStarted,
                        now_ms,
                        correlation_id.clone(),
                    )?;
                }
            }
            TerminalEventKind::Output {
                bytes,
                dropped_bytes_before,
            } => {
                let (redacted, redaction_checkpoint) = {
                    let mut state = core
                        .artifact_state()
                        .lock()
                        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                    let checkpoint =
                        terminal_output_redaction_checkpoint(&state, &event.command_id);
                    (
                        retain_redacted_terminal_output(
                            &mut state,
                            &event.command_id,
                            &bytes,
                            false,
                        ),
                        checkpoint,
                    )
                };
                if !redacted.is_empty() || dropped_bytes_before > 0 {
                    let transition = (|| -> Result<(), ProtocolError> {
                        let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                            unreachable!()
                        };
                        terminal
                            .append_output_with_dropped(&redacted, dropped_bytes_before, now_ms)
                            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                        persist_terminal_event_transition(
                            scope,
                            record,
                            artifact::ArtifactHistoryKind::OutputAppended,
                            now_ms,
                            correlation_id.clone(),
                        )
                    })();
                    if let Err(error) = transition {
                        let mut state = core
                            .artifact_state()
                            .lock()
                            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                        restore_terminal_output_redaction(
                            &mut state,
                            &event.command_id,
                            redaction_checkpoint,
                        );
                        return Err(error);
                    }
                    event_is_durable = true;
                }
            }
            TerminalEventKind::Completed {
                exit_code,
                working_directory,
            } => {
                flush_terminal_output(
                    core.artifact_state(),
                    scope,
                    record,
                    &event.command_id,
                    now_ms,
                    correlation_id.clone(),
                )?;
                let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                    unreachable!()
                };
                terminal
                    .complete(exit_code, working_directory.to_string_lossy(), now_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                persist_terminal_event_transition(
                    scope,
                    record,
                    artifact::ArtifactHistoryKind::CommandCompleted,
                    now_ms,
                    correlation_id.clone(),
                )?;
            }
            TerminalEventKind::Interrupted130 {
                working_directory,
                shell_replaced,
            } => {
                flush_terminal_output(
                    core.artifact_state(),
                    scope,
                    record,
                    &event.command_id,
                    now_ms,
                    correlation_id.clone(),
                )?;
                let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                    unreachable!()
                };
                append_terminal_interrupt_marker(terminal, now_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                terminal.working_directory_display =
                    working_directory.to_string_lossy().into_owned();
                terminal
                    .interrupt(shell_replaced, now_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                persist_terminal_event_transition(
                    scope,
                    record,
                    artifact::ArtifactHistoryKind::CommandInterrupted,
                    now_ms,
                    correlation_id.clone(),
                )?;
            }
            TerminalEventKind::RecoveredInterrupted {
                working_directory, ..
            } => {
                let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                    unreachable!()
                };
                terminal.working_directory_display =
                    working_directory.to_string_lossy().into_owned();
                terminal
                    .recover(
                        "process-restart",
                        "The previous Terminal process was not trusted after restart.",
                        now_ms,
                    )
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                persist_terminal_event_transition(
                    scope,
                    record,
                    artifact::ArtifactHistoryKind::RecoveryChanged,
                    now_ms,
                    correlation_id.clone(),
                )?;
            }
            TerminalEventKind::Failed {
                code,
                message,
                shell_replaced: _,
            } => {
                flush_terminal_output(
                    core.artifact_state(),
                    scope,
                    record,
                    &event.command_id,
                    now_ms,
                    correlation_id.clone(),
                )?;
                let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                    unreachable!()
                };
                terminal
                    .fail(code, message, true, now_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                persist_terminal_event_transition(
                    scope,
                    record,
                    artifact::ArtifactHistoryKind::CommandFailed,
                    now_ms,
                    correlation_id.clone(),
                )?;
            }
        }
        let mut state = core
            .artifact_state()
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        state
            .terminal_event_cursors
            .insert(cursor_key.clone(), event.chunk_sequence);
        if event_is_durable {
            state
                .terminal_ack_cursors
                .insert(record.artifact_id.clone(), event.chunk_sequence);
        }
        if command_became_terminal {
            state
                .terminal_cleanup_sessions
                .insert(record.artifact_id.clone(), scope.session_id.clone());
        }
        drop(state);
        if event_is_durable {
            core.terminal_supervisor()
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                .acknowledge_output(TerminalAcknowledgeRequest {
                    key: key.clone(),
                    terminal_session_id: binding.terminal_session_id.clone(),
                    process_generation: binding.process_generation,
                    through_chunk_sequence: event.chunk_sequence,
                })
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        }
    }
    let cleanup_artifact_ids = core
        .artifact_state()
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .terminal_cleanup_sessions
        .iter()
        .filter(|(_, session_id)| *session_id == &scope.session_id)
        .map(|(artifact_id, _)| artifact_id.clone())
        .collect::<Vec<_>>();
    for artifact_id in cleanup_artifact_ids {
        let now_ms =
            current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        cancel_stale_terminal_controls(
            core.artifact_state(),
            core.runtime_service(),
            &artifact_id,
            now_ms,
            correlation_id.clone(),
        )?;
    }
    Ok(())
}

fn recover_unsupervised_terminal_records(
    database: &core::database::DatabaseActor,
    records: &mut [artifact::ArtifactRecord],
    correlation_id: protocol::CorrelationId,
) -> Result<bool, ProtocolError> {
    let mut recovered = false;
    for record in records {
        let now_ms = current_time_ms()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .max(record.updated_at_ms);
        let Some(expected_revision) = transition_unsupervised_terminal_record(record, now_ms)?
        else {
            continue;
        };
        persist_artifact_record_to_database(
            database,
            record,
            Some(expected_revision),
            correlation_id.clone(),
        )?;
        recovered = true;
    }
    Ok(recovered)
}

fn transition_unsupervised_terminal_record(
    record: &mut artifact::ArtifactRecord,
    now_ms: u64,
) -> Result<Option<u64>, ProtocolError> {
    let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
        return Ok(None);
    };
    if !matches!(
        &terminal.status,
        artifact::TerminalCommandStatus::Running { .. }
    ) {
        return Ok(None);
    }
    terminal
        .recover(
            "process-restart",
            "The previous Terminal process was not trusted after restart.",
            now_ms,
        )
        .map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::Internal,
                "Terminal recovery failed",
                true,
            )
        })?;
    let expected_revision = record.record_revision;
    advance_artifact_record(
        record,
        artifact::ArtifactHistoryKind::RecoveryChanged,
        now_ms,
    )?;
    Ok(Some(expected_revision))
}

fn pump_supervised_terminal_sessions_once(
    active_workspace: &Mutex<Option<core::services::ActiveWorkspace>>,
    artifact_operation: &Mutex<()>,
    resources: &BackgroundTerminalReconciliation,
    include_restart_recovery: bool,
) -> Result<usize, ProtocolError> {
    let correlation_id = protocol::CorrelationId::new("terminal-reconciliation-driver")
        .expect("the production Terminal reconciliation correlation id is valid");
    let _operation = artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let (workspace_id, database, snapshot) = {
        let active = active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let Some(active) = active.as_ref() else {
            return Ok(0);
        };
        let query = core::database::SnapshotQuery::new(core::database::MAX_READ_RECORDS)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .including_inactive();
        (
            active.manifest().workspace_id.to_string(),
            Arc::clone(active.database_actor()),
            active
                .snapshot(query)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
        )
    };
    let supervisor_snapshots = resources
        .terminal_supervisor()
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .session_snapshots();
    let supervised_session_ids = supervisor_snapshots
        .iter()
        .filter(|session| session.key.workspace_id == workspace_id)
        .map(|session| session.key.chat_id.clone())
        .collect::<BTreeSet<_>>();
    let mut session_ids = supervisor_snapshots
        .into_iter()
        .filter(|session| {
            session.key.workspace_id == workspace_id
                && (session.active_command.is_some() || session.pending_event_count > 0)
        })
        .map(|session| session.key.chat_id)
        .collect::<BTreeSet<_>>();
    session_ids.extend(
        resources
            .artifact_state()
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .terminal_cleanup_sessions
            .values()
            .cloned(),
    );
    if include_restart_recovery {
        session_ids.extend(
            database
                .artifact_session_ids_for_provider("terminal")
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
        );
    }

    let limits = ProjectFilesystemLimits::new(
        artifact::file::MAX_FILE_CONTENT_BYTES as u64,
        artifact::folder::MAX_FOLDER_ENTRIES,
        execution::filesystem::MAX_PROJECT_FOLDER_NAME_BYTES,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let mut reconciled = 0;
    for session_id in session_ids {
        let documents = database
            .artifact_documents_for_session(&session_id)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let mut records = documents
            .into_iter()
            .map(|document| deserialize_artifact_record(document, correlation_id.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        if !records
            .iter()
            .any(|record| matches!(&record.state, artifact::ArtifactState::Terminal(_)))
        {
            continue;
        }
        let cleanup_artifact_ids = resources
            .artifact_state()
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
            .terminal_cleanup_sessions
            .iter()
            .filter(|(_, cleanup_session_id)| *cleanup_session_id == &session_id)
            .map(|(artifact_id, _)| artifact_id.clone())
            .collect::<Vec<_>>();
        for artifact_id in cleanup_artifact_ids {
            let now_ms = current_time_ms()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            cancel_stale_terminal_controls(
                resources.artifact_state(),
                resources.runtime_service(),
                &artifact_id,
                now_ms,
                correlation_id.clone(),
            )?;
        }
        if include_restart_recovery && !supervised_session_ids.contains(&session_id) {
            recover_unsupervised_terminal_records(&database, &mut records, correlation_id.clone())?;
            reconciled += 1;
            continue;
        }
        let Some(chat) = snapshot
            .chats
            .iter()
            .find(|chat| chat.chat_id == session_id)
        else {
            continue;
        };
        let Some(project) = snapshot.projects.iter().find(|project| {
            project.project_id == chat.project_id
                && project.path_state != core::database::ProjectPathState::Missing
        }) else {
            continue;
        };
        let Ok(project_root) = TrustedProjectRoot::open(Path::new(&project.current_path)) else {
            continue;
        };
        let Ok(filesystem) = ProjectFilesystem::bind_with_limits(project_root.clone(), limits)
        else {
            continue;
        };
        let scope = ActiveArtifactScope {
            workspace_id: workspace_id.clone(),
            project_id: project.project_id.clone(),
            session_id: session_id.clone(),
            project_name: project.display_name.clone(),
            project_root,
            filesystem,
            database: Arc::clone(&database),
        };
        reconcile_active_terminal_records(resources, &scope, &mut records, correlation_id.clone())?;
        reconciled += 1;
    }
    Ok(reconciled)
}

fn start_terminal_reconciliation_driver(
    active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    artifact_operation: Arc<Mutex<()>>,
    artifact: Arc<Mutex<ArtifactApplicationState>>,
    terminal: Arc<Mutex<TerminalSupervisor>>,
    runtime: Arc<RuntimeApplicationService>,
) -> Result<TerminalReconciliationDriver, std::io::Error> {
    let resources = BackgroundTerminalReconciliation {
        artifact,
        terminal,
        runtime,
    };
    let (stop, stop_rx) = std::sync::mpsc::channel();
    let join = thread::Builder::new()
        .name("c4os-terminal-reconciliation-driver".into())
        .spawn(move || {
            let mut recovered_workspace_id = None;
            loop {
                let workspace_id = active_workspace.lock().ok().and_then(|active| {
                    active
                        .as_ref()
                        .map(|workspace| workspace.manifest().workspace_id.to_string())
                });
                let include_restart_recovery = workspace_id != recovered_workspace_id;
                if pump_supervised_terminal_sessions_once(
                    &active_workspace,
                    &artifact_operation,
                    &resources,
                    include_restart_recovery,
                )
                .is_ok()
                    && include_restart_recovery
                {
                    recovered_workspace_id = workspace_id;
                }
                match stop_rx.recv_timeout(Duration::from_millis(25)) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        })?;
    Ok(TerminalReconciliationDriver {
        stop: Some(stop),
        join: Some(join),
    })
}

fn append_terminal_interrupt_marker(
    terminal: &mut artifact::TerminalArtifactState,
    observed_at_ms: u64,
) -> Result<(), artifact::TerminalStateError> {
    let retained = terminal.output.retained_bytes()?;
    if retained.ends_with(b"^C\r\n") || retained.ends_with(b"^C\n") {
        return Ok(());
    }
    terminal.append_output(b"^C\r\n", observed_at_ms)
}

fn terminal_event_drain_binding(
    latest: &artifact::TerminalArtifactState,
    supervised: &execution::terminal::TerminalLiveSessionSnapshot,
) -> Option<execution::terminal::TerminalLiveSessionSnapshot> {
    if supervised.terminal_session_id != latest.identity.terminal_session_id {
        return None;
    }
    let generation_matches = if matches!(
        supervised.lifecycle,
        execution::terminal::TerminalLifecycle::Live
    ) {
        supervised.process_generation == latest.process.process_generation
    } else {
        supervised.process_generation == latest.process.process_generation
            || (matches!(
                &latest.status,
                artifact::TerminalCommandStatus::Queued { .. }
            ) && supervised
                .process_generation
                .checked_add(1)
                .is_some_and(|generation| generation == latest.process.process_generation)
                && supervised.next_command_sequence == latest.identity.command_sequence)
    };
    generation_matches.then(|| supervised.clone())
}

fn reconcile_interrupted_terminal_approval(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let artifact::ArtifactState::Terminal(terminal) = &record.state else {
        return Ok(());
    };
    if !matches!(
        &terminal.status,
        artifact::TerminalCommandStatus::Queued { .. }
    ) {
        return Ok(());
    }
    let already_requeued = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .pending_terminal_actions
        .values()
        .any(|pending| pending.artifact_id == record.artifact_id);
    if already_requeued {
        return Ok(());
    }
    let key = terminal_key(scope)?;
    let supervised = core
        .terminal
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .live_session(&key);
    if let Some(live) = supervised.as_ref() {
        if terminal_run_was_dispatched(terminal, live) {
            return Ok(());
        }
        if !terminal_run_can_be_requeued(terminal, live) {
            let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                unreachable!()
            };
            terminal
                .recover(
                    "approval-restart",
                    "Terminal approval lost its exact shell binding. Review the command and run it again; C4OS did not repeat it.",
                    now_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            return persist_terminal_event_transition(
                scope,
                record,
                artifact::ArtifactHistoryKind::RecoveryChanged,
                now_ms,
                correlation_id,
            );
        }
    } else {
        let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
            unreachable!()
        };
        terminal
            .recover(
                "approval-restart",
                "Terminal approval was interrupted by restart. Review the command and run it again; C4OS did not repeat it.",
                now_ms,
            )
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        return persist_terminal_event_transition(
            scope,
            record,
            artifact::ArtifactHistoryKind::RecoveryChanged,
            now_ms,
            correlation_id,
        );
    }
    if terminal_command_contains_secret_material(&terminal.command) {
        return Err(workspace_state_unavailable(correlation_id));
    }
    let environment = execution::environment::ExecutionEnvironmentIdentity::new(
        execution::environment::ExecutionEnvironmentKind::Local,
        terminal.process.environment_id.clone(),
        terminal.process.environment_generation,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let command_sha256 = sha256_bytes(terminal.command.as_bytes());
    let (pending, facts) = prepare_terminal_action(
        core,
        scope,
        record,
        PendingArtifactTerminalPayload::Run {
            terminal_session_id: terminal.identity.terminal_session_id.clone(),
            command_id: terminal.identity.command_id.clone(),
            command_sequence: terminal.identity.command_sequence,
            command: terminal.command.clone(),
            shell_path: terminal.process.shell_path.clone(),
            environment,
            process_generation: terminal.process.process_generation,
            columns: terminal.dimensions.columns,
            rows: terminal.dimensions.rows,
        },
        "terminal.execute",
        "terminal.execute",
        ActionEffect::Execute,
        CanonicalRisk::Medium,
        ClassificationConfidence::Ambiguous,
        serde_json::json!({
            "artifactId": record.artifact_id,
            "terminalSessionId": terminal.identity.terminal_session_id,
            "commandId": terminal.identity.command_id,
            "commandSequence": terminal.identity.command_sequence,
            "commandSha256": command_sha256,
            "commandByteLength": terminal.command.len(),
            "columns": terminal.dimensions.columns,
            "rows": terminal.dimensions.rows,
            "terminalProcessGeneration": terminal.process.process_generation,
        }),
        now_ms,
        correlation_id.clone(),
    )?;
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .requeue_interrupted_direct_approval(&facts, pending.action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => {
            let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                unreachable!()
            };
            terminal
                .fail(
                    "policy-denied",
                    "Policy denied the interrupted Terminal command.",
                    false,
                    now_ms,
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            persist_terminal_event_transition(
                scope,
                record,
                artifact::ArtifactHistoryKind::RecoveryChanged,
                now_ms,
                correlation_id,
            )
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id))?
                .pending_terminal_actions
                .insert(prompt.prompt_id, pending);
            Ok(())
        }
        GatewayProposal::Authorized { .. } => Err(workspace_state_unavailable(correlation_id)),
    }
}

fn terminal_run_was_dispatched(
    terminal: &artifact::TerminalArtifactState,
    live: &execution::terminal::TerminalLiveSessionSnapshot,
) -> bool {
    if !matches!(live.lifecycle, execution::terminal::TerminalLifecycle::Live)
        || live.terminal_session_id != terminal.identity.terminal_session_id
        || live.process_generation != terminal.process.process_generation
    {
        return false;
    }
    let exact_active_command = live.active_command.as_ref().is_some_and(|command| {
        command.terminal_session_id == terminal.identity.terminal_session_id
            && command.command_id == terminal.identity.command_id
            && command.command_sequence == terminal.identity.command_sequence
    });
    exact_active_command
        || (live.active_command.is_none()
            && live.next_command_sequence > terminal.identity.command_sequence
            && live.pending_event_count > 0)
}

fn terminal_run_can_be_requeued(
    terminal: &artifact::TerminalArtifactState,
    live: &execution::terminal::TerminalLiveSessionSnapshot,
) -> bool {
    if live.terminal_session_id != terminal.identity.terminal_session_id
        || live.active_command.is_some()
        || live.next_command_sequence != terminal.identity.command_sequence
    {
        return false;
    }
    if matches!(live.lifecycle, execution::terminal::TerminalLifecycle::Live) {
        live.process_generation == terminal.process.process_generation
    } else {
        live.process_generation
            .checked_add(1)
            .is_some_and(|generation| generation == terminal.process.process_generation)
    }
}

fn flush_terminal_output(
    artifact: &Mutex<ArtifactApplicationState>,
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    command_id: &str,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let (redacted, redaction_checkpoint) = {
        let mut state = artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let checkpoint = terminal_output_redaction_checkpoint(&state, command_id);
        (
            retain_redacted_terminal_output(&mut state, command_id, &[], true),
            checkpoint,
        )
    };
    if redacted.is_empty() {
        return Ok(());
    }
    let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
        return Err(workspace_state_unavailable(correlation_id));
    };
    let transition = terminal
        .append_output(&redacted, now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))
        .and_then(|()| {
            persist_terminal_event_transition(
                scope,
                record,
                artifact::ArtifactHistoryKind::OutputAppended,
                now_ms,
                correlation_id.clone(),
            )
        });
    if let Err(error) = transition {
        let mut state = artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        restore_terminal_output_redaction(&mut state, command_id, redaction_checkpoint);
        return Err(error);
    }
    Ok(())
}

struct TerminalCommandPlan {
    terminal_session_id: String,
    command_id: String,
    command_sequence: u64,
    process_generation: u64,
    shell_path: String,
    working_directory: PathBuf,
    environment: execution::environment::ExecutionEnvironmentIdentity,
}

fn plan_terminal_command(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    correlation_id: protocol::CorrelationId,
) -> Result<TerminalCommandPlan, ProtocolError> {
    let documents = scope
        .database
        .artifact_documents_for_session(&scope.session_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let mut terminals = documents
        .into_iter()
        .map(|document| deserialize_artifact_record(document, correlation_id.clone()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|record| match record.state {
            artifact::ArtifactState::Terminal(terminal) => Some(terminal),
            _ => None,
        })
        .collect::<Vec<_>>();
    terminals.sort_by_key(|terminal| terminal.identity.command_sequence);
    let key = terminal_key(scope)?;
    let supervised = core
        .terminal
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .live_session(&key);
    if let Some(supervised) = supervised {
        if supervised.active_command.is_some() {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "Finish or Stop the active Terminal command before running another",
                false,
            ));
        }
        return Ok(TerminalCommandPlan {
            terminal_session_id: supervised.terminal_session_id,
            command_id: format!("terminal-command-{}", Uuid::new_v4().as_simple()),
            command_sequence: supervised.next_command_sequence,
            process_generation: if matches!(
                supervised.lifecycle,
                execution::terminal::TerminalLifecycle::Live
            ) {
                supervised.process_generation
            } else {
                supervised
                    .process_generation
                    .checked_add(1)
                    .ok_or_else(|| {
                        ProtocolError::new(
                            ProtocolErrorCode::InvalidGeneration,
                            "The Terminal process generation is exhausted",
                            false,
                        )
                    })?
            },
            shell_path: supervised.shell_path.to_string_lossy().into_owned(),
            working_directory: supervised.working_directory,
            environment: supervised.environment,
        });
    }
    let Some(previous) = terminals.last() else {
        return Ok(TerminalCommandPlan {
            terminal_session_id: format!("terminal-session-{}", Uuid::new_v4().as_simple()),
            command_id: format!("terminal-command-{}", Uuid::new_v4().as_simple()),
            command_sequence: 1,
            process_generation: 1,
            shell_path: "/bin/zsh".into(),
            working_directory: scope.project_root.canonical_root().to_path_buf(),
            environment: execution::environment::ExecutionEnvironmentIdentity::new(
                execution::environment::ExecutionEnvironmentKind::Local,
                "desktop",
                1,
            )
            .map_err(|_| workspace_state_unavailable(correlation_id))?,
        });
    };
    if !previous.status.is_terminal() {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The previous Terminal command must be recovered before running another",
            true,
        ));
    }
    let environment = execution::environment::ExecutionEnvironmentIdentity::new(
        execution::environment::ExecutionEnvironmentKind::Local,
        previous.process.environment_id.clone(),
        previous.process.environment_generation,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let previous_identity = SupervisedTerminalCommandIdentity::new(
        previous.identity.terminal_session_id.clone(),
        previous.identity.command_id.clone(),
        previous.identity.command_sequence,
    )
    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let restored = core
        .terminal
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .restore_completed_session(TerminalCompletedSessionRecord {
            key: key.clone(),
            command: previous_identity,
            process_generation: previous.process.process_generation,
            environment: environment.clone(),
            dimensions: PtyDimensions::new(previous.dimensions.columns, previous.dimensions.rows)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
            trusted_project_root: scope.project_root.clone(),
            shell_path: PathBuf::from(&previous.process.shell_path),
            working_directory: PathBuf::from(&previous.working_directory_display),
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    Ok(TerminalCommandPlan {
        terminal_session_id: restored.terminal_session_id,
        command_id: format!("terminal-command-{}", Uuid::new_v4().as_simple()),
        command_sequence: restored.next_command_sequence,
        process_generation: restored.process_generation.checked_add(1).ok_or_else(|| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidGeneration,
                "The Terminal process generation is exhausted",
                false,
            )
        })?,
        shell_path: restored.shell_path.to_string_lossy().into_owned(),
        working_directory: restored.working_directory,
        environment,
    })
}

#[allow(clippy::too_many_arguments)]
fn prepare_terminal_action(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &artifact::ArtifactRecord,
    payload: PendingArtifactTerminalPayload,
    action_kind: &str,
    authority: &str,
    effect: ActionEffect,
    risk: CanonicalRisk,
    confidence: ClassificationConfidence,
    arguments: serde_json::Value,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(PendingArtifactTerminalAction, ActionFacts), ProtocolError> {
    let live = current_artifact_live_authority(core, now_ms, correlation_id.clone())?;
    let terminal_process_generation = match &payload {
        PendingArtifactTerminalPayload::Run {
            process_generation, ..
        }
        | PendingArtifactTerminalPayload::Stdin {
            process_generation, ..
        }
        | PendingArtifactTerminalPayload::Resize {
            process_generation, ..
        }
        | PendingArtifactTerminalPayload::Stop {
            process_generation, ..
        } => *process_generation,
    };
    let action_id = format!("terminal-action-{}", Uuid::new_v4().as_simple());
    let canonical_target = scope
        .project_root
        .canonical_root()
        .to_string_lossy()
        .into_owned();
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: "c4os.terminal".into(),
        arguments,
        risk,
        requested_authority: BTreeSet::from([authority.into()]),
        canonical_target: canonical_target.clone(),
        target_version: format!(
            "terminal-{}-process-{}",
            scope.session_id, terminal_process_generation
        ),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        run_id: format!("terminal-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: None,
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "The Terminal operation could not be bound to an exact action",
            false,
        )
    })?;
    let facts = ActionFacts {
        action_kind: action_kind.into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::Terminal,
        effects: BTreeSet::from([effect]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::User,
        sensitivity: if confidence == ClassificationConfidence::Known {
            ActionSensitivity::Ordinary
        } else {
            ActionSensitivity::Unknown
        },
        reversibility: if confidence == ClassificationConfidence::Known {
            ActionReversibility::Reversible
        } else {
            ActionReversibility::Unknown
        },
        confidence,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: active_project_repository_state(scope),
        inside_active_project: true,
        canonical_target,
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
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
        PendingArtifactTerminalAction {
            artifact_id: record.artifact_id.clone(),
            record_revision: record.record_revision,
            scope: scope.clone(),
            action,
            live,
            payload,
        },
        facts,
    ))
}

fn persist_terminal_failure(
    pending: &PendingArtifactTerminalAction,
    code: &str,
    message: &str,
    retryable: bool,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let mut record =
        load_scoped_artifact_record(&pending.scope, &pending.artifact_id, correlation_id.clone())?;
    let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
        return Err(workspace_state_unavailable(correlation_id));
    };
    if terminal.status.is_terminal() {
        return Ok(());
    }
    let transition_at_ms = now_ms.max(record.updated_at_ms);
    terminal
        .fail(code, message, retryable, transition_at_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    persist_terminal_event_transition(
        &pending.scope,
        &mut record,
        artifact::ArtifactHistoryKind::CommandFailed,
        transition_at_ms,
        correlation_id,
    )
}

fn execute_terminal_action(
    core: &AppCoreState,
    pending: PendingArtifactTerminalAction,
    token: AuthorizationToken,
    approval_prompt_id: Option<&str>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let record =
        load_scoped_artifact_record(&pending.scope, &pending.artifact_id, correlation_id.clone())?;
    if record.record_revision < pending.record_revision {
        return Err(workspace_state_unavailable(correlation_id));
    }
    let artifact::ArtifactState::Terminal(terminal_state) = &record.state else {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The approved Artifact is no longer a Terminal command",
            true,
        ));
    };
    let key = terminal_key(&pending.scope)?;
    let terminal_session_id = terminal_state.identity.terminal_session_id.clone();
    let command_id = terminal_state.identity.command_id.clone();
    let command_sequence = terminal_state.identity.command_sequence;
    let mut effect_result: Option<Result<(), String>> = None;
    let canonical_target = pending.action.canonical_target.clone();
    let completed_at_ms = now_ms.saturating_add(1);
    core.runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator.execute_direct_action(
                &token,
                &pending.action,
                pending.live,
                approval_prompt_id,
                now_ms,
                |_permit| {
                    let result = (|| {
                        let mut supervisor = core
                            .terminal
                            .lock()
                            .map_err(|_| "Terminal supervisor unavailable".to_owned())?;
                        match &pending.payload {
                            PendingArtifactTerminalPayload::Run {
                                terminal_session_id: expected_session_id,
                                command_id: expected_command_id,
                                command_sequence: expected_command_sequence,
                                command,
                                shell_path,
                                environment,
                                process_generation,
                                columns,
                                rows,
                            } => {
                                if expected_session_id != &terminal_session_id
                                    || expected_command_id != &command_id
                                    || expected_command_sequence != &command_sequence
                                    || command != &terminal_state.command
                                {
                                    return Err("Terminal command binding changed".into());
                                }
                                if let Some(live) = supervisor.live_session(&key)
                                    && matches!(
                                        live.lifecycle,
                                        execution::terminal::TerminalLifecycle::Live
                                    )
                                    && live.dimensions
                                        != PtyDimensions::new(*columns, *rows)
                                            .map_err(|error| error.to_string())?
                                {
                                    supervisor
                                        .resize_authorized(TerminalResizeRequest {
                                            key: key.clone(),
                                            terminal_session_id: expected_session_id.clone(),
                                            process_generation: *process_generation,
                                            dimensions: PtyDimensions::new(*columns, *rows)
                                                .map_err(|error| error.to_string())?,
                                        })
                                        .map_err(|error| error.to_string())?;
                                }
                                supervisor
                                    .execute_authorized(TerminalExecuteRequest {
                                        key: key.clone(),
                                        command: SupervisedTerminalCommandIdentity::new(
                                            expected_session_id.clone(),
                                            expected_command_id.clone(),
                                            *expected_command_sequence,
                                        )
                                        .map_err(|error| error.to_string())?,
                                        process_generation: *process_generation,
                                        environment: environment.clone(),
                                        dimensions: PtyDimensions::new(*columns, *rows)
                                            .map_err(|error| error.to_string())?,
                                        trusted_project_root: pending.scope.project_root.clone(),
                                        shell_path: PathBuf::from(shell_path),
                                        command_line: command.clone(),
                                    })
                                    .map_err(|error| error.to_string())?;
                                Ok(())
                            }
                            PendingArtifactTerminalPayload::Stdin {
                                process_generation,
                                text,
                            } => {
                                let mut bytes = text.as_bytes().to_vec();
                                bytes.push(b'\n');
                                supervisor
                                    .submit_stdin_authorized(TerminalStdinRequest {
                                        key: key.clone(),
                                        command: SupervisedTerminalCommandIdentity::new(
                                            terminal_session_id.clone(),
                                            command_id.clone(),
                                            command_sequence,
                                        )
                                        .map_err(|error| error.to_string())?,
                                        process_generation: *process_generation,
                                        bytes,
                                    })
                                    .map_err(|error| error.to_string())
                            }
                            PendingArtifactTerminalPayload::Resize {
                                process_generation,
                                columns,
                                rows,
                            } => supervisor
                                .resize_authorized(TerminalResizeRequest {
                                    key: key.clone(),
                                    terminal_session_id: terminal_session_id.clone(),
                                    process_generation: *process_generation,
                                    dimensions: PtyDimensions::new(*columns, *rows)
                                        .map_err(|error| error.to_string())?,
                                })
                                .map_err(|error| error.to_string()),
                            PendingArtifactTerminalPayload::Stop { process_generation } => {
                                supervisor
                                    .stop_authorized(TerminalStopRequest {
                                        key: key.clone(),
                                        command: SupervisedTerminalCommandIdentity::new(
                                            terminal_session_id.clone(),
                                            command_id.clone(),
                                            command_sequence,
                                        )
                                        .map_err(|error| error.to_string())?,
                                        process_generation: *process_generation,
                                    })
                                    .map(|_| ())
                                    .map_err(|error| error.to_string())
                            }
                        }
                    })();
                    let normalized = match &result {
                        Ok(()) => NormalizedActionResult {
                            status: NormalizedActionStatus::Succeeded,
                            result_code: "terminal-operation-succeeded".into(),
                            exit_code: Some(0),
                            changed_targets: vec![canonical_target.clone()],
                            output_sha256: None,
                            completed_at_ms,
                        },
                        Err(_) => NormalizedActionResult {
                            status: NormalizedActionStatus::Failed,
                            result_code: "terminal-operation-failed".into(),
                            exit_code: None,
                            changed_targets: Vec::new(),
                            output_sha256: None,
                            completed_at_ms,
                        },
                    };
                    effect_result = Some(result);
                    normalized
                },
            )?;
            Ok(())
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let result =
        effect_result.ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    if let Err(message) = result {
        if matches!(&pending.payload, PendingArtifactTerminalPayload::Run { .. }) {
            persist_terminal_failure(
                &pending,
                "terminal-operation-failed",
                &message,
                true,
                completed_at_ms,
                correlation_id,
            )?;
            return Ok(());
        }
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The authorized Terminal operation no longer matched the live process",
            true,
        ));
    }
    match &pending.payload {
        PendingArtifactTerminalPayload::Run { .. } => {}
        PendingArtifactTerminalPayload::Stdin { .. }
        | PendingArtifactTerminalPayload::Resize { .. }
        | PendingArtifactTerminalPayload::Stop { .. } => {
            let mut record = load_scoped_artifact_record(
                &pending.scope,
                &pending.artifact_id,
                correlation_id.clone(),
            )?;
            let artifact::ArtifactState::Terminal(terminal) = &mut record.state else {
                return Err(workspace_state_unavailable(correlation_id));
            };
            let (kind, transition) = match &pending.payload {
                PendingArtifactTerminalPayload::Stdin { .. } => (
                    artifact::ArtifactHistoryKind::InputSubmitted,
                    terminal.note_input_submitted(completed_at_ms),
                ),
                PendingArtifactTerminalPayload::Resize { columns, rows, .. } => (
                    artifact::ArtifactHistoryKind::TerminalResized,
                    artifact::TerminalDimensions::new(*columns, *rows)
                        .and_then(|dimensions| terminal.resize(dimensions, completed_at_ms)),
                ),
                PendingArtifactTerminalPayload::Stop { .. } => (
                    artifact::ArtifactHistoryKind::StopRequested,
                    terminal.request_stop(completed_at_ms),
                ),
                PendingArtifactTerminalPayload::Run { .. } => unreachable!(),
            };
            transition.map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            persist_terminal_event_transition(
                &pending.scope,
                &mut record,
                kind,
                completed_at_ms,
                correlation_id,
            )?;
        }
    }
    Ok(())
}

fn propose_terminal_action(
    core: &AppCoreState,
    mut pending: PendingArtifactTerminalAction,
    facts: ActionFacts,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .propose_direct_action(&facts, pending.action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => {
            if matches!(&pending.payload, PendingArtifactTerminalPayload::Run { .. }) {
                persist_terminal_failure(
                    &pending,
                    "policy-denied",
                    "Policy denied the Terminal operation.",
                    false,
                    now_ms,
                    correlation_id,
                )
            } else {
                Ok(())
            }
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            pending.record_revision = load_scoped_artifact_record(
                &pending.scope,
                &pending.artifact_id,
                correlation_id.clone(),
            )?
            .record_revision;
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id))?
                .pending_terminal_actions
                .insert(prompt.prompt_id, pending);
            Ok(())
        }
        GatewayProposal::Authorized { token, .. } => {
            execute_terminal_action(core, pending, token, None, now_ms, correlation_id)
        }
    }
}

fn resolve_browser_environment(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    artifact_id: &str,
    correlation_id: protocol::CorrelationId,
) -> Result<
    (
        artifact::BrowserEnvironmentReference,
        browser::native::NativeBrowserDataStore,
    ),
    ProtocolError,
> {
    let app_configuration = core
        .configuration
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .last_known_good()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let project_id = Uuid::parse_str(&scope.project_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let chat_id = Uuid::parse_str(&scope.session_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let environment = core
        .active_workspace
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .as_ref()
        .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?
        .restore_effective_configuration_snapshot(
            app_configuration,
            Some(project_id),
            Some(chat_id),
            core::configuration::ManagedCeilings::default(),
            core::configuration::SecurityConstraints::default(),
        )
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .configuration
        .browser_environment;
    let persistent_scope = browser::profile::persistent_profile_scope(
        environment,
        &scope.workspace_id,
        &scope.project_id,
        &scope.session_id,
    );
    let Some(persistent_scope) = persistent_scope else {
        return Ok((
            artifact::BrowserEnvironmentReference::ephemeral(artifact_id, 1)
                .map_err(|_| workspace_state_unavailable(correlation_id))?,
            browser::native::NativeBrowserDataStore::Ephemeral,
        ));
    };
    let mut registry = core
        .browser_profiles
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let generation = registry.snapshot().generation;
    let resolved = registry
        .resolve(generation, persistent_scope)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    if !matches!(
        resolved.profile.lifecycle,
        browser::profile::PersistentProfileLifecycle::Ready
    ) {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "Browser data is being cleared; wait before opening this environment",
            true,
        ));
    }
    let reference = match resolved.profile.scope {
        browser::profile::PersistentProfileScope::AppWide => {
            artifact::BrowserEnvironmentReference::app_wide(resolved.profile.data_generation)
        }
        browser::profile::PersistentProfileScope::WorkspaceProject {
            workspace_id,
            project_id,
        } => artifact::BrowserEnvironmentReference::workspace_project(
            workspace_id,
            project_id,
            resolved.profile.data_generation,
        ),
        browser::profile::PersistentProfileScope::Chat {
            workspace_id,
            chat_id,
        } => artifact::BrowserEnvironmentReference::chat(
            workspace_id,
            scope.project_id.clone(),
            chat_id,
            resolved.profile.data_generation,
        ),
    }
    .map_err(|_| workspace_state_unavailable(correlation_id))?;
    Ok((
        reference,
        browser::native::NativeBrowserDataStore::Persistent {
            profile_id: resolved.profile.profile_id,
        },
    ))
}

fn browser_data_store_for_reference(
    core: &AppCoreState,
    reference: &artifact::BrowserEnvironmentReference,
    correlation_id: protocol::CorrelationId,
) -> Result<browser::native::NativeBrowserDataStore, ProtocolError> {
    let Some(scope) = persistent_profile_scope_for_reference(reference) else {
        return Ok(browser::native::NativeBrowserDataStore::Ephemeral);
    };
    let registry = core
        .browser_profiles
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let profile = registry
        .snapshot()
        .profiles
        .into_iter()
        .find(|profile| profile.scope == scope && profile.data_generation == reference.generation)
        .filter(|profile| {
            matches!(
                profile.lifecycle,
                browser::profile::PersistentProfileLifecycle::Ready
            )
        })
        .ok_or_else(|| {
            platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "The Browser Environment generation changed",
                true,
            )
        })?;
    Ok(browser::native::NativeBrowserDataStore::Persistent {
        profile_id: profile.profile_id,
    })
}

fn persistent_profile_scope_for_reference(
    reference: &artifact::BrowserEnvironmentReference,
) -> Option<browser::profile::PersistentProfileScope> {
    match reference.scope {
        artifact::BrowserEnvironmentScope::AppWide => {
            Some(browser::profile::PersistentProfileScope::AppWide)
        }
        artifact::BrowserEnvironmentScope::WorkspaceProject => {
            Some(browser::profile::PersistentProfileScope::WorkspaceProject {
                workspace_id: reference.workspace_id.clone()?,
                project_id: reference.project_id.clone()?,
            })
        }
        artifact::BrowserEnvironmentScope::Chat => {
            Some(browser::profile::PersistentProfileScope::Chat {
                workspace_id: reference.workspace_id.clone()?,
                chat_id: reference.session_id.clone()?,
            })
        }
        artifact::BrowserEnvironmentScope::None => None,
    }
}

fn effective_browser_media_permission_policy(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<browser::native::NativeBrowserPermissionPolicy, ProtocolError> {
    let facts = ActionFacts {
        action_kind: "browser.permission.media".into(),
        native_tool: "webkit.permission.media".into(),
        surface: ActionSurface::Browser,
        effects: BTreeSet::from([ActionEffect::Capture, ActionEffect::Listen]),
        scope: ActionScope::System,
        initiator: ActionInitiator::Runtime,
        sensitivity: ActionSensitivity::Private,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::RuntimeTool,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target: "browser-permission:media".into(),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: false,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    let resolution = core
        .runtime
        .coordinator()
        .map_err(|_| workspace_state_unavailable(correlation_id))?
        .resolve_direct_policy(&facts, now_ms);
    Ok(if resolution.decision == PolicyDecision::Deny {
        browser::native::NativeBrowserPermissionPolicy::Deny
    } else {
        browser::native::NativeBrowserPermissionPolicy::PlatformDefault
    })
}

#[allow(clippy::too_many_arguments)]
fn prepare_browser_action(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &artifact::ArtifactRecord,
    payload: PendingArtifactBrowserPayload,
    action_kind: &str,
    display_url: &str,
    navigation_sha256: &str,
    initiator: ActionInitiator,
    request_origin: ActionRequestOrigin,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(PendingArtifactBrowserAction, ActionFacts), ProtocolError> {
    let live = current_artifact_live_authority(core, now_ms, correlation_id.clone())?;
    let action_id = format!("browser-action-{}", Uuid::new_v4().as_simple());
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: "c4os.browser".into(),
        arguments: serde_json::json!({
            "artifactId": record.artifact_id,
            "intent": action_kind,
            "displayUrl": display_url,
            "navigationSha256": navigation_sha256,
        }),
        risk: CanonicalRisk::Low,
        requested_authority: BTreeSet::from(["browser.navigate".into()]),
        canonical_target: format!("browser-target:{navigation_sha256}"),
        target_version: format!("browser-record-{}", record.record_revision),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        run_id: format!("browser-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: None,
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "The Browser operation could not be bound to an exact action",
            false,
        )
    })?;
    let facts = ActionFacts {
        action_kind: action_kind.into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::Browser,
        effects: BTreeSet::from([ActionEffect::Read]),
        scope: ActionScope::Remote,
        initiator,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target: format!("browser-target:{navigation_sha256}"),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: false,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    Ok((
        PendingArtifactBrowserAction {
            artifact_id: record.artifact_id.clone(),
            record_revision: record.record_revision,
            scope: scope.clone(),
            action,
            live,
            payload,
        },
        facts,
    ))
}

#[allow(clippy::too_many_arguments)]
fn prepare_browser_clear_action(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &artifact::ArtifactRecord,
    payload: PendingArtifactBrowserPayload,
    environment: &artifact::BrowserEnvironmentReference,
    canonical_target: String,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(PendingArtifactBrowserAction, ActionFacts), ProtocolError> {
    let live = current_artifact_live_authority(core, now_ms, correlation_id.clone())?;
    let action_id = format!("browser-action-{}", Uuid::new_v4().as_simple());
    let scope_label = match environment.scope {
        artifact::BrowserEnvironmentScope::AppWide => "all-browsers",
        artifact::BrowserEnvironmentScope::WorkspaceProject => "per-project",
        artifact::BrowserEnvironmentScope::Chat => "per-chat-session",
        artifact::BrowserEnvironmentScope::None => "none",
    };
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: "c4os.browser".into(),
        arguments: serde_json::json!({
            "artifactId": record.artifact_id,
            "intent": "browser.data.clear",
            "environmentScope": scope_label,
            "dataGeneration": environment.generation,
        }),
        risk: CanonicalRisk::Medium,
        requested_authority: BTreeSet::from(["browser.clear-data".into()]),
        canonical_target: canonical_target.clone(),
        target_version: format!("browser-record-{}", record.record_revision),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        run_id: format!("browser-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: None,
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "The Browser data clear could not be bound to an exact action",
            false,
        )
    })?;
    let facts = ActionFacts {
        action_kind: "browser.data.clear".into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::Browser,
        effects: BTreeSet::from([ActionEffect::Delete]),
        scope: ActionScope::ExternalLocal,
        initiator: ActionInitiator::User,
        sensitivity: ActionSensitivity::Private,
        reversibility: ActionReversibility::Destructive,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target,
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: false,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    Ok((
        PendingArtifactBrowserAction {
            artifact_id: record.artifact_id.clone(),
            record_revision: record.record_revision,
            scope: scope.clone(),
            action,
            live,
            payload,
        },
        facts,
    ))
}

fn persist_browser_policy_denial(
    pending: &PendingArtifactBrowserAction,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    if !matches!(pending.payload, PendingArtifactBrowserPayload::Open { .. }) {
        return Ok(());
    }
    let mut record =
        load_scoped_artifact_record(&pending.scope, &pending.artifact_id, correlation_id.clone())?;
    let artifact::ArtifactState::Browser(browser) = &mut record.state else {
        return Err(workspace_state_unavailable(correlation_id));
    };
    browser
        .fail_before_controller(artifact::BrowserErrorCode::PolicyDenied, false, now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let expected = record.record_revision;
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::RecoveryChanged,
        now_ms,
    )?;
    persist_artifact_record(&pending.scope, &record, Some(expected), correlation_id)?;
    Ok(())
}

fn discard_native_browser_request(
    core: &AppCoreState,
    app: &tauri::AppHandle,
    pending: &PendingArtifactBrowserAction,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let PendingArtifactBrowserPayload::NativeRequest {
        request_id,
        controller_generation,
        mount_generation,
        ..
    } = &pending.payload
    else {
        return Ok(());
    };
    let identity = browser::native::NativeBrowserIdentity {
        artifact_id: pending.artifact_id.clone(),
        controller_generation: *controller_generation,
        mount_generation: *mount_generation,
    };
    let active = {
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        state.browser_native_requests.remove(request_id);
        let active = state.browser_active_identity.as_ref() == Some(&identity);
        let notices = state
            .browser_notices
            .entry(pending.artifact_id.clone())
            .or_default();
        notices.push(ArtifactBrowserNoticeSnapshot {
            id: format!("browser-navigation-denied-{request_id}"),
            kind: "warning".into(),
            title: "Navigation blocked".into(),
            message: "The website-requested navigation was not authorized.".into(),
        });
        if notices.len() > 32 {
            let excess = notices.len() - 32;
            notices.drain(..excess);
        }
        active
    };
    if active {
        let _ = browser::native::dispatch_action(
            app,
            Arc::clone(&core.browser_events),
            identity,
            browser::native::NativeBrowserAction::DiscardNavigationRequest {
                request_id: request_id.clone(),
            },
        );
    }
    Ok(())
}

fn execute_browser_action(
    core: &AppCoreState,
    app: &tauri::AppHandle,
    pending: PendingArtifactBrowserAction,
    token: AuthorizationToken,
    approval_prompt_id: Option<&str>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let record =
        load_scoped_artifact_record(&pending.scope, &pending.artifact_id, correlation_id.clone())?;
    if record.record_revision != pending.record_revision
        || !matches!(record.state, artifact::ArtifactState::Browser(_))
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The approved Browser operation changed before execution",
            true,
        ));
    }
    let mut effect_result: Option<Result<(), String>> = None;
    let mut native_action = None;
    let mut profile_clear_dispatch = None;
    let mut ephemeral_clear_dispatch = None;
    let canonical_target = pending.action.canonical_target.clone();
    let completed_at_ms = now_ms.saturating_add(1);
    core.runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator.execute_direct_action(
                &token,
                &pending.action,
                pending.live,
                approval_prompt_id,
                now_ms,
                |_permit| {
                    let result = (|| {
                        let mut state = core
                            .artifact
                            .lock()
                            .map_err(|_| "Browser application state unavailable".to_owned())?;
                        match &pending.payload {
                            PendingArtifactBrowserPayload::Open { target } => {
                                state.browser_targets.insert(
                                    pending.artifact_id.clone(),
                                    BrowserTransientTarget {
                                        target: target.clone(),
                                    },
                                );
                                state
                                    .browser_mount_generations
                                    .entry(pending.artifact_id.clone())
                                    .or_insert(1);
                                state.browser_notices.remove(&pending.artifact_id);
                            }
                            PendingArtifactBrowserPayload::Navigate {
                                intent,
                                navigation_sha256,
                                controller_generation,
                                mount_generation,
                            } => {
                                let artifact::ArtifactState::Browser(browser) = &record.state else {
                                    return Err("Browser record changed".into());
                                };
                                if browser.controller_generation != *controller_generation
                                    || state
                                        .browser_mount_generations
                                        .get(&pending.artifact_id)
                                        .copied()
                                        .unwrap_or(1)
                                        != *mount_generation
                                {
                                    return Err("Browser controller identity changed".into());
                                }
                                let action = match intent {
                                    artifact::BrowserNavigationIntent::Back => {
                                        browser::native::NativeBrowserAction::Back {
                                            expected_navigation_sha256: navigation_sha256.clone(),
                                        }
                                    }
                                    artifact::BrowserNavigationIntent::Forward => {
                                        browser::native::NativeBrowserAction::Forward {
                                            expected_navigation_sha256: navigation_sha256.clone(),
                                        }
                                    }
                                    artifact::BrowserNavigationIntent::Refresh => {
                                        browser::native::NativeBrowserAction::Refresh {
                                            expected_navigation_sha256: navigation_sha256.clone(),
                                        }
                                    }
                                };
                                if state
                                    .browser_mounted_artifacts
                                    .contains(&pending.artifact_id)
                                {
                                    native_action = Some((
                                        browser::native::NativeBrowserIdentity {
                                            artifact_id: pending.artifact_id.clone(),
                                            controller_generation: *controller_generation,
                                            mount_generation: *mount_generation,
                                        },
                                        action,
                                    ));
                                } else {
                                    state
                                        .browser_pending_navigation
                                        .insert(pending.artifact_id.clone(), action);
                                }
                            }
                            PendingArtifactBrowserPayload::NavigateTo {
                                target,
                                controller_generation,
                                mount_generation,
                            }
                            | PendingArtifactBrowserPayload::Recover {
                                target,
                                controller_generation,
                                mount_generation,
                            } => {
                                let artifact::ArtifactState::Browser(browser) = &record.state else {
                                    return Err("Browser record changed".into());
                                };
                                if browser.controller_generation != *controller_generation
                                    || state
                                        .browser_mount_generations
                                        .get(&pending.artifact_id)
                                        .copied()
                                        .unwrap_or(1)
                                        != *mount_generation
                                {
                                    return Err("Browser controller identity changed".into());
                                }
                                state.browser_targets.insert(
                                    pending.artifact_id.clone(),
                                    BrowserTransientTarget {
                                        target: target.clone(),
                                    },
                                );
                                if state
                                    .browser_mounted_artifacts
                                    .contains(&pending.artifact_id)
                                {
                                    native_action = Some((
                                        browser::native::NativeBrowserIdentity {
                                            artifact_id: pending.artifact_id.clone(),
                                            controller_generation: *controller_generation,
                                            mount_generation: *mount_generation,
                                        },
                                        browser::native::NativeBrowserAction::NavigateTo {
                                            target: target.clone(),
                                        },
                                    ));
                                } else if matches!(
                                    browser.phase,
                                    artifact::BrowserPhase::Recovery {
                                        code: artifact::BrowserRecoveryCode::ApplicationRelaunch,
                                        ..
                                    }
                                ) {
                                    state
                                        .browser_pending_navigation
                                        .remove(&pending.artifact_id);
                                    let mount = state
                                        .browser_mount_generations
                                        .entry(pending.artifact_id.clone())
                                        .or_insert(1);
                                    *mount = mount.saturating_add(1).max(1);
                                } else {
                                    state.browser_pending_navigation.insert(
                                        pending.artifact_id.clone(),
                                        browser::native::NativeBrowserAction::NavigateTo {
                                            target: target.clone(),
                                        },
                                    );
                                }
                            }
                            PendingArtifactBrowserPayload::NativeRequest {
                                request_id,
                                controller_generation,
                                mount_generation,
                                navigation_sha256,
                            } => {
                                let artifact::ArtifactState::Browser(browser) = &record.state else {
                                    return Err("Browser record changed".into());
                                };
                                let identity = browser::native::NativeBrowserIdentity {
                                    artifact_id: pending.artifact_id.clone(),
                                    controller_generation: *controller_generation,
                                    mount_generation: *mount_generation,
                                };
                                if browser.controller_generation != *controller_generation
                                    || state.browser_active_identity.as_ref() != Some(&identity)
                                    || state
                                        .browser_native_requests
                                        .get(request_id)
                                        .is_none_or(|request| {
                                            request.navigation_sha256 != *navigation_sha256
                                                || request.artifact_id != pending.artifact_id
                                        })
                                {
                                    return Err("Browser navigation request changed".into());
                                }
                                state.browser_native_requests.remove(request_id);
                                native_action = Some((
                                    identity,
                                    browser::native::NativeBrowserAction::AuthorizeNavigationRequest {
                                        request_id: request_id.clone(),
                                        expected_navigation_sha256: navigation_sha256.clone(),
                                    },
                                ));
                            }
                            PendingArtifactBrowserPayload::ClearData {
                                environment,
                                persistent,
                                operation_id,
                                controller_generation,
                                mount_generation,
                            } => {
                                let artifact::ArtifactState::Browser(browser) = &record.state else {
                                    return Err("Browser record changed".into());
                                };
                                if &browser.environment != environment
                                    || browser.controller_generation != *controller_generation
                                    || state
                                        .browser_mount_generations
                                        .get(&pending.artifact_id)
                                        .copied()
                                        .unwrap_or(1)
                                        != *mount_generation
                                {
                                    return Err("Browser Environment changed".into());
                                }
                                if let Some(clear) = persistent {
                                    let mut registry = core
                                        .browser_profiles
                                        .lock()
                                        .map_err(|_| "Browser profile registry unavailable".to_owned())?;
                                    let marked = registry
                                        .mark_clear_pending(
                                            clear.registry_generation,
                                            &clear.scope,
                                            clear.data_generation,
                                            operation_id.clone(),
                                        )
                                        .map_err(|_| "Browser profile clear changed".to_owned())?;
                                    if marked.profile_id != clear.profile_id {
                                        return Err("Browser profile identifier changed".into());
                                    }
                                    profile_clear_dispatch = Some((
                                        clear.profile_id.clone(),
                                        operation_id.clone(),
                                    ));
                                } else {
                                    state.browser_ephemeral_clear_operations.insert(
                                        pending.artifact_id.clone(),
                                        operation_id.clone(),
                                    );
                                    ephemeral_clear_dispatch = Some((
                                        pending.artifact_id.clone(),
                                        operation_id.clone(),
                                    ));
                                }
                                state.browser_native_requests.retain(|_, request| {
                                    request.artifact_id != pending.artifact_id
                                });
                                state.browser_pending_navigation.remove(&pending.artifact_id);
                                state.browser_mounted_artifacts.remove(&pending.artifact_id);
                                if state
                                    .browser_active_identity
                                    .as_ref()
                                    .is_some_and(|identity| {
                                        identity.artifact_id == pending.artifact_id
                                    })
                                {
                                    state.browser_active_identity = None;
                                }
                                let mount = state
                                    .browser_mount_generations
                                    .entry(pending.artifact_id.clone())
                                    .or_insert(1);
                                *mount = mount.saturating_add(1).max(1);
                            }
                        }
                        Ok(())
                    })();
                    let succeeded = result.is_ok();
                    effect_result = Some(result);
                    NormalizedActionResult {
                        status: if succeeded {
                            NormalizedActionStatus::Succeeded
                        } else {
                            NormalizedActionStatus::Failed
                        },
                        result_code: if succeeded {
                            "browser-operation-authorized"
                        } else {
                            "browser-operation-rejected"
                        }
                        .into(),
                        exit_code: succeeded.then_some(0),
                        changed_targets: if succeeded {
                            vec![canonical_target.clone()]
                        } else {
                            Vec::new()
                        },
                        output_sha256: None,
                        completed_at_ms,
                    }
                },
            )
            .map(|_| ())
            .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    effect_result
        .unwrap_or_else(|| Err("Browser action permit was not executed".into()))
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    if let Some((identity, action)) = native_action {
        browser::native::dispatch_action(app, Arc::clone(&core.browser_events), identity, action)
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    }
    if let Some((profile_id, operation_id)) = profile_clear_dispatch {
        browser::native::dispatch_clear_profile(
            app,
            Arc::clone(&core.browser_events),
            profile_id,
            operation_id,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    }
    if let Some((artifact_id, operation_id)) = ephemeral_clear_dispatch {
        browser::native::dispatch_destroy_ephemeral(
            app,
            Arc::clone(&core.browser_events),
            artifact_id,
            operation_id,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    }
    save_artifact_ui_state(&pending.scope, Some(&record), now_ms, correlation_id)?;
    Ok(())
}

fn propose_browser_action(
    core: &AppCoreState,
    app: &tauri::AppHandle,
    mut pending: PendingArtifactBrowserAction,
    facts: ActionFacts,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    if core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .pending_browser_actions
        .values()
        .any(|active| active.artifact_id == pending.artifact_id)
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "Resolve the pending Browser approval before starting another Browser operation",
            true,
        ));
    }
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .propose_direct_action(&facts, pending.action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => {
            discard_native_browser_request(core, app, &pending, correlation_id.clone())?;
            persist_browser_policy_denial(&pending, now_ms, correlation_id)
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            pending.record_revision = load_scoped_artifact_record(
                &pending.scope,
                &pending.artifact_id,
                correlation_id.clone(),
            )?
            .record_revision;
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id))?
                .pending_browser_actions
                .insert(prompt.prompt_id, pending);
            Ok(())
        }
        GatewayProposal::Authorized { token, .. } => {
            execute_browser_action(core, app, pending, token, None, now_ms, correlation_id)
        }
    }
}

#[tauri::command]
fn artifact_open_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserOpenInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let target = artifact::BrowserNavigationTarget::parse(&input.address).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "Enter a valid HTTP or HTTPS Browser address without embedded credentials",
            false,
        )
    })?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact_id = format!("artifact-{}", Uuid::new_v4().as_simple());
    let (environment, _data_store) =
        resolve_browser_environment(&core, &scope, &artifact_id, request.correlation_id.clone())?;
    let state = artifact::BrowserArtifactState::new_queued(environment, &target, 1, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let record = new_artifact_record_with_id(
        &scope,
        artifact_id,
        artifact::ArtifactProviderDescriptor::browser(),
        artifact::ArtifactState::Browser(Box::new(state)),
        "open-browser",
        now_ms,
    )?;
    persist_artifact_record(&scope, &record, None, request.correlation_id.clone())?;
    let (pending, facts) = prepare_browser_action(
        &core,
        &scope,
        &record,
        PendingArtifactBrowserPayload::Open {
            target: target.clone(),
        },
        "browser.open",
        target.display_url(),
        &target.navigation_sha256(),
        ActionInitiator::User,
        ActionRequestOrigin::DirectUserEdit,
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_browser_action(
        &core,
        &app,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_navigate_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserNavigateInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Browser(browser_state) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a Browser artifact accepts navigation",
            false,
        ));
    };
    if browser_state.controller_generation != input.controller_generation {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::StaleGeneration,
            "The Browser controller generation changed",
            true,
        ));
    }
    let intent = match input.intent {
        ArtifactBrowserNavigationIntent::Back => artifact::BrowserNavigationIntent::Back,
        ArtifactBrowserNavigationIntent::Forward => artifact::BrowserNavigationIntent::Forward,
        ArtifactBrowserNavigationIntent::Refresh => artifact::BrowserNavigationIntent::Refresh,
    };
    browser_state
        .validate_navigation_intent(intent)
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The Browser navigation is no longer available",
                true,
            )
        })?;
    let mount_generation = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .browser_mount_generations
        .get(&record.artifact_id)
        .copied()
        .unwrap_or(1);
    if mount_generation != input.mount_generation {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::StaleGeneration,
            "The Browser mount generation changed",
            true,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let navigation_entry = match intent {
        artifact::BrowserNavigationIntent::Back => {
            &browser_state.history[browser_state.current_history_index as usize - 1]
        }
        artifact::BrowserNavigationIntent::Forward => {
            &browser_state.history[browser_state.current_history_index as usize + 1]
        }
        artifact::BrowserNavigationIntent::Refresh => browser_state.current_entry(),
    };
    let navigation_target = artifact::BrowserNavigationTarget::parse(&navigation_entry.display_url)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let relaunch_refresh = matches!(intent, artifact::BrowserNavigationIntent::Refresh)
        && matches!(
            browser_state.phase,
            artifact::BrowserPhase::Recovery {
                code: artifact::BrowserRecoveryCode::ApplicationRelaunch,
                ..
            }
        );
    let (payload, target_version_sha256) = if relaunch_refresh {
        (
            PendingArtifactBrowserPayload::Recover {
                target: navigation_target.clone(),
                controller_generation: input.controller_generation,
                mount_generation: input.mount_generation,
            },
            navigation_target.navigation_sha256(),
        )
    } else {
        (
            PendingArtifactBrowserPayload::Navigate {
                intent,
                navigation_sha256: navigation_entry.navigation_sha256.clone(),
                controller_generation: input.controller_generation,
                mount_generation: input.mount_generation,
            },
            navigation_entry.navigation_sha256.clone(),
        )
    };
    let (pending, facts) = prepare_browser_action(
        &core,
        &scope,
        &record,
        payload,
        match intent {
            artifact::BrowserNavigationIntent::Back => "browser.back",
            artifact::BrowserNavigationIntent::Forward => "browser.forward",
            artifact::BrowserNavigationIntent::Refresh => "browser.refresh",
        },
        navigation_target.display_url(),
        &target_version_sha256,
        ActionInitiator::User,
        ActionRequestOrigin::DirectUserEdit,
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_browser_action(
        &core,
        &app,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_clear_browser_data(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserIdentityInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    if before.focused_artifact_id.as_ref() != Some(&input.artifact_id) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Only the focused Browser can clear its Browser Environment",
            true,
        ));
    }
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let browser_state = require_browser_controller_identity(
        &core,
        &record,
        input.controller_generation,
        input.mount_generation,
        request.correlation_id.clone(),
    )?;
    let operation_id = Uuid::new_v4().to_string();
    let (persistent, canonical_target) = if let Some(profile_scope) =
        persistent_profile_scope_for_reference(&browser_state.environment)
    {
        let registry = core
            .browser_profiles
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let snapshot = registry.snapshot();
        let profile = snapshot
            .profiles
            .iter()
            .find(|profile| {
                profile.scope == profile_scope
                    && profile.data_generation == browser_state.environment.generation
                    && matches!(
                        profile.lifecycle,
                        browser::profile::PersistentProfileLifecycle::Ready
                    )
            })
            .ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "The Browser Environment generation changed",
                    true,
                )
            })?;
        (
            Some(PendingPersistentBrowserClear {
                scope: profile_scope,
                profile_id: profile.profile_id.clone(),
                registry_generation: snapshot.generation,
                data_generation: profile.data_generation,
            }),
            format!(
                "browser-profile:{}:generation:{}",
                profile.profile_id, profile.data_generation
            ),
        )
    } else {
        (
            None,
            format!(
                "browser-ephemeral:{}:generation:{}",
                record.artifact_id, browser_state.environment.generation
            ),
        )
    };
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (pending, facts) = prepare_browser_clear_action(
        &core,
        &scope,
        &record,
        PendingArtifactBrowserPayload::ClearData {
            environment: browser_state.environment.clone(),
            persistent,
            operation_id,
            controller_generation: input.controller_generation,
            mount_generation: input.mount_generation,
        },
        &browser_state.environment,
        canonical_target,
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_browser_action(
        &core,
        &app,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn require_browser_controller_identity<'a>(
    core: &AppCoreState,
    record: &'a artifact::ArtifactRecord,
    controller_generation: u64,
    mount_generation: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<&'a artifact::BrowserArtifactState, ProtocolError> {
    let artifact::ArtifactState::Browser(browser_state) = &record.state else {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "The Artifact is not a Browser",
            false,
        ));
    };
    let active_mount = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .browser_mount_generations
        .get(&record.artifact_id)
        .copied()
        .unwrap_or(1);
    if browser_state.controller_generation != controller_generation
        || active_mount != mount_generation
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::StaleGeneration,
            "The Browser controller identity changed",
            true,
        ));
    }
    Ok(browser_state)
}

#[tauri::command]
fn artifact_mount_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserViewportInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    if before.focused_artifact_id.as_ref() != Some(&input.artifact_id) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Only the focused Browser can mount a native surface",
            true,
        ));
    }
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let (already_mounted, cached) = {
        let state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            state
                .browser_mounted_artifacts
                .contains(&record.artifact_id),
            state.browser_targets.get(&record.artifact_id).cloned(),
        )
    };
    if !already_mounted
        && matches!(
            &record.state,
            artifact::ArtifactState::Browser(browser)
                if matches!(browser.phase, artifact::BrowserPhase::Error { retryable: false, .. })
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "This Browser operation was denied or cannot be retried",
            false,
        ));
    }
    let requires_application_relaunch = !already_mounted
        && cached.is_none()
        && matches!(
            &record.state,
            artifact::ArtifactState::Browser(browser)
                if !matches!(
                    browser.phase,
                    artifact::BrowserPhase::Recovery {
                        code: artifact::BrowserRecoveryCode::ApplicationRelaunch,
                        ..
                    }
                )
        );
    let requires_controller_recovery = !already_mounted
        && cached.is_some()
        && matches!(
            &record.state,
            artifact::ArtifactState::Browser(browser)
                if matches!(
                    browser.phase,
                    artifact::BrowserPhase::Loading { .. }
                        | artifact::BrowserPhase::Ready { .. }
                        | artifact::BrowserPhase::Error { retryable: true, .. }
                )
        );
    if requires_application_relaunch || requires_controller_recovery {
        let now_ms = current_time_ms()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let artifact::ArtifactState::Browser(browser) = &mut record.state else {
            unreachable!();
        };
        let recovery_code = if requires_application_relaunch {
            artifact::BrowserRecoveryCode::ApplicationRelaunch
        } else {
            artifact::BrowserRecoveryCode::ControllerRecreated
        };
        browser
            .begin_recovery(recovery_code, now_ms)
            .and_then(|_| {
                browser.install_controller_generation(
                    browser.controller_generation.saturating_add(1),
                    now_ms,
                )
            })
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let expected = record.record_revision;
        advance_artifact_record(
            &mut record,
            artifact::ArtifactHistoryKind::RecoveryChanged,
            now_ms,
        )?;
        persist_artifact_record(
            &scope,
            &record,
            Some(expected),
            request.correlation_id.clone(),
        )?;
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        let mount = state
            .browser_mount_generations
            .entry(record.artifact_id.clone())
            .or_insert(1);
        *mount = mount.saturating_add(1).max(1);
        drop(state);
        let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
        return protocol::artifact_workspace_snapshot(request, payload);
    }
    if cached.is_none() && !already_mounted {
        // Application-relaunch recovery is a composed waiting state, not a
        // failed native mount. Refresh creates the next authorization-bound
        // transient target and advances the mount generation.
        let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
        return protocol::artifact_workspace_snapshot(request, payload);
    }
    let browser_state = require_browser_controller_identity(
        &core,
        &record,
        input.controller_generation,
        input.mount_generation,
        request.correlation_id.clone(),
    )?;
    let data_store = browser_data_store_for_reference(
        &core,
        &browser_state.environment,
        request.correlation_id.clone(),
    )?;
    let media_permission_policy = effective_browser_media_permission_policy(
        &core,
        &scope,
        current_time_ms()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?,
        request.correlation_id.clone(),
    )?;
    let (initial_target, recover_existing_target, initial_kind) = match browser_state.phase {
        artifact::BrowserPhase::Queued { .. } => (
            Some(
                cached
                    .clone()
                    .expect("authorization-backed Browser target was required above")
                    .target,
            ),
            false,
            artifact::BrowserNavigationKind::Initial,
        ),
        artifact::BrowserPhase::Recovery {
            code:
                artifact::BrowserRecoveryCode::ProfileCleared
                | artifact::BrowserRecoveryCode::ApplicationRelaunch,
            ..
        } => (
            Some(
                cached
                    .clone()
                    .expect("an approved Browser recovery target was required above")
                    .target,
            ),
            false,
            artifact::BrowserNavigationKind::Recovery,
        ),
        artifact::BrowserPhase::Recovery { .. } => {
            (None, true, artifact::BrowserNavigationKind::Recovery)
        }
        _ => (None, false, artifact::BrowserNavigationKind::New),
    };
    browser::native::dispatch_mount(
        &app,
        Arc::clone(&core.browser_events),
        browser::native::NativeBrowserMountRequest {
            artifact_id: record.artifact_id.clone(),
            controller_generation: input.controller_generation,
            mount_generation: input.mount_generation,
            rect: browser::native::NativeBrowserRect {
                x: input.x,
                y: input.y,
                width: input.width,
                height: input.height,
            },
            data_store,
            initial_target,
            recover_existing_target,
            initial_kind,
            media_permission_policy,
            focus: input.focus,
        },
    )
    .map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Unavailable,
            "The native Browser surface could not be mounted",
            true,
        )
    })?;
    let pending_navigation = {
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        if let Some(previous) =
            state
                .browser_active_identity
                .replace(browser::native::NativeBrowserIdentity {
                    artifact_id: record.artifact_id.clone(),
                    controller_generation: input.controller_generation,
                    mount_generation: input.mount_generation,
                })
        {
            state
                .browser_mounted_artifacts
                .remove(&previous.artifact_id);
        }
        state
            .browser_mounted_artifacts
            .insert(record.artifact_id.clone());
        state.browser_pending_navigation.remove(&record.artifact_id)
    };
    if let Some(action) = pending_navigation {
        browser::native::dispatch_action(
            &app,
            Arc::clone(&core.browser_events),
            browser::native::NativeBrowserIdentity {
                artifact_id: record.artifact_id,
                controller_generation: input.controller_generation,
                mount_generation: input.mount_generation,
            },
            action,
        )
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    }
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_resize_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserViewportInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    if before.focused_artifact_id.as_ref() != Some(&input.artifact_id) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Only the focused Browser can resize a native surface",
            true,
        ));
    }
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    require_browser_controller_identity(
        &core,
        &record,
        input.controller_generation,
        input.mount_generation,
        request.correlation_id.clone(),
    )?;
    let requested_identity = browser::native::NativeBrowserIdentity {
        artifact_id: record.artifact_id.clone(),
        controller_generation: input.controller_generation,
        mount_generation: input.mount_generation,
    };
    if core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .browser_active_identity
        .as_ref()
        != Some(&requested_identity)
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The native Browser surface is no longer mounted",
            true,
        ));
    }
    browser::native::dispatch_geometry(
        &app,
        requested_identity,
        browser::native::NativeBrowserRect {
            x: input.x,
            y: input.y,
            width: input.width,
            height: input.height,
        },
    )
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    protocol::artifact_workspace_snapshot(request, before)
}

fn browser_identity_from_input(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    input: &ArtifactBrowserIdentityInput,
    correlation_id: protocol::CorrelationId,
) -> Result<browser::native::NativeBrowserIdentity, ProtocolError> {
    let record = load_active_artifact_record(
        scope,
        &input.artifact_id,
        input.base_record_revision,
        correlation_id.clone(),
    )?;
    require_browser_controller_identity(
        core,
        &record,
        input.controller_generation,
        input.mount_generation,
        correlation_id,
    )?;
    Ok(browser::native::NativeBrowserIdentity {
        artifact_id: record.artifact_id,
        controller_generation: input.controller_generation,
        mount_generation: input.mount_generation,
    })
}

#[tauri::command]
fn artifact_focus_native_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserIdentityInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    if before.focused_artifact_id.as_ref() != Some(&input.artifact_id) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Only the focused Browser can receive native focus",
            true,
        ));
    }
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let identity =
        browser_identity_from_input(&core, &scope, &input, request.correlation_id.clone())?;
    if core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .browser_active_identity
        .as_ref()
        != Some(&identity)
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The native Browser surface is no longer mounted",
            true,
        ));
    }
    browser::native::dispatch_action(
        &app,
        Arc::clone(&core.browser_events),
        identity,
        browser::native::NativeBrowserAction::Focus,
    )
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    protocol::artifact_workspace_snapshot(request, before)
}

#[tauri::command]
fn artifact_detach_browser(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactBrowserIdentityInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    let identity = browser::native::NativeBrowserIdentity {
        artifact_id: input.artifact_id.as_str().to_owned(),
        controller_generation: input.controller_generation,
        mount_generation: input.mount_generation,
    };
    let is_active = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .browser_active_identity
        .as_ref()
        == Some(&identity);
    if !is_active {
        return protocol::artifact_workspace_snapshot(request, before);
    }
    browser::native::dispatch_detach(&app, identity)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut state = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    state
        .browser_mounted_artifacts
        .remove(input.artifact_id.as_str());
    state.browser_active_identity = None;
    drop(state);
    protocol::artifact_workspace_snapshot(request, before)
}

#[tauri::command]
fn artifact_run_terminal(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactTerminalRunInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    if terminal_command_contains_secret_material(&input.command) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Terminal commands cannot contain inline credential material; use an environment or credential reference",
            false,
        ));
    }
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let plan = plan_terminal_command(&core, &scope, request.correlation_id.clone())?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let command_sha256 = sha256_bytes(input.command.as_bytes());
    let terminal = artifact::TerminalArtifactState::new_queued(
        artifact::TerminalCommandIdentity {
            terminal_session_id: plan.terminal_session_id.clone(),
            command_id: plan.command_id.clone(),
            command_sequence: plan.command_sequence,
        },
        input.command.clone(),
        plan.working_directory.to_string_lossy(),
        artifact::TerminalDimensions::new(input.columns, input.rows)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?,
        artifact::TerminalProcessProvenance {
            shell_path: plan.shell_path.clone(),
            environment_id: plan.environment.environment_id.clone(),
            environment_generation: plan.environment.generation,
            process_generation: plan.process_generation,
            shell_process_id: None,
            foreground_process_group_id: None,
        },
        now_ms,
    )
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let record = new_artifact_record(
        &scope,
        artifact::ArtifactProviderDescriptor::terminal(),
        artifact::ArtifactState::Terminal(Box::new(terminal)),
        "run-terminal",
        now_ms,
    )?;
    persist_artifact_record(&scope, &record, None, request.correlation_id.clone())?;
    let (pending, facts) = prepare_terminal_action(
        &core,
        &scope,
        &record,
        PendingArtifactTerminalPayload::Run {
            terminal_session_id: plan.terminal_session_id,
            command_id: plan.command_id,
            command_sequence: plan.command_sequence,
            command: input.command.clone(),
            shell_path: plan.shell_path,
            environment: plan.environment,
            process_generation: plan.process_generation,
            columns: input.columns,
            rows: input.rows,
        },
        "terminal.execute",
        "terminal.execute",
        ActionEffect::Execute,
        CanonicalRisk::Medium,
        ClassificationConfidence::Ambiguous,
        serde_json::json!({
            "artifactId": record.artifact_id,
            "terminalSessionId": match &record.state {
                artifact::ArtifactState::Terminal(terminal) => terminal.identity.terminal_session_id.clone(),
                _ => unreachable!(),
            },
            "commandId": match &record.state {
                artifact::ArtifactState::Terminal(terminal) => terminal.identity.command_id.clone(),
                _ => unreachable!(),
            },
            "commandSequence": plan.command_sequence,
            "commandSha256": command_sha256,
            "commandByteLength": input.command.len(),
            "columns": input.columns,
            "rows": input.rows,
            "terminalProcessGeneration": plan.process_generation,
        }),
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_terminal_action(
        &core,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn require_no_pending_terminal_action(
    core: &AppCoreState,
    artifact_id: &protocol::ArtifactId,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    if core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .pending_terminal_actions
        .values()
        .any(|pending| pending.artifact_id == artifact_id.as_str())
    {
        Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "Resolve the pending Terminal approval before another operation",
            false,
        ))
    } else {
        Ok(())
    }
}

#[tauri::command]
fn artifact_terminal_stdin(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactTerminalStdinInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_terminal_action(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Terminal(terminal) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a running Terminal command accepts process input",
            false,
        ));
    };
    if terminal.process.process_generation != input.process_generation
        || !matches!(
            &terminal.status,
            artifact::TerminalCommandStatus::Running {
                stdin_ready: true,
                stop_requested_at_ms: None,
                ..
            }
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Terminal input no longer targets the live command",
            true,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut exact_bytes = input.text.as_bytes().to_vec();
    exact_bytes.push(b'\n');
    let (pending, facts) = prepare_terminal_action(
        &core,
        &scope,
        &record,
        PendingArtifactTerminalPayload::Stdin {
            process_generation: input.process_generation,
            text: input.text,
        },
        "terminal.input",
        "terminal.input",
        ActionEffect::Control,
        CanonicalRisk::Medium,
        ClassificationConfidence::Ambiguous,
        serde_json::json!({
            "artifactId": record.artifact_id,
            "commandId": terminal.identity.command_id,
            "terminalProcessGeneration": input.process_generation,
            "inputSha256": sha256_bytes(&exact_bytes),
            "inputByteLength": exact_bytes.len(),
        }),
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_terminal_action(
        &core,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_terminal_resize(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactTerminalResizeInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_terminal_action(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Terminal(terminal) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a running Terminal command can be resized",
            false,
        ));
    };
    if terminal.process.process_generation != input.process_generation
        || !matches!(
            &terminal.status,
            artifact::TerminalCommandStatus::Running { .. }
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Terminal resize no longer targets the live process",
            true,
        ));
    }
    if terminal.dimensions.columns == input.columns && terminal.dimensions.rows == input.rows {
        let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
        return protocol::artifact_workspace_snapshot(request, payload);
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (pending, facts) = prepare_terminal_action(
        &core,
        &scope,
        &record,
        PendingArtifactTerminalPayload::Resize {
            process_generation: input.process_generation,
            columns: input.columns,
            rows: input.rows,
        },
        "terminal.resize",
        "terminal.resize",
        ActionEffect::Control,
        CanonicalRisk::Low,
        ClassificationConfidence::Known,
        serde_json::json!({
            "artifactId": record.artifact_id,
            "commandId": terminal.identity.command_id,
            "terminalProcessGeneration": input.process_generation,
            "columns": input.columns,
            "rows": input.rows,
        }),
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_terminal_action(
        &core,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_terminal_stop(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactTerminalOperationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_terminal_action(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Terminal(terminal) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a running Terminal command can be stopped",
            false,
        ));
    };
    if terminal.process.process_generation != input.process_generation
        || !matches!(
            &terminal.status,
            artifact::TerminalCommandStatus::Running {
                stop_requested_at_ms: None,
                ..
            }
        )
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "Terminal Stop no longer targets the live command",
            true,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (pending, facts) = prepare_terminal_action(
        &core,
        &scope,
        &record,
        PendingArtifactTerminalPayload::Stop {
            process_generation: input.process_generation,
        },
        "terminal.stop",
        "terminal.stop",
        ActionEffect::Control,
        CanonicalRisk::Medium,
        ClassificationConfidence::Known,
        serde_json::json!({
            "artifactId": record.artifact_id,
            "commandId": terminal.identity.command_id,
            "terminalProcessGeneration": input.process_generation,
        }),
        now_ms,
        request.correlation_id.clone(),
    )?;
    propose_terminal_action(
        &core,
        pending,
        facts,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_terminal_ack_output(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactTerminalOutputAckInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_scoped_artifact_record(
        &scope,
        input.artifact_id.as_str(),
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Terminal(terminal) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only Terminal output can be acknowledged",
            false,
        ));
    };
    if terminal.process.process_generation != input.process_generation
        || terminal.output.sequence != input.output_sequence
    {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::StaleGeneration,
            "Terminal output changed before acknowledgement",
            true,
        ));
    }
    let through_chunk_sequence = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .terminal_ack_cursors
        .get(&record.artifact_id)
        .copied();
    let Some(through_chunk_sequence) = through_chunk_sequence else {
        if terminal_output_requires_ack_cursor(terminal) {
            return Err(workspace_state_unavailable(request.correlation_id.clone()));
        }
        let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
        return protocol::artifact_workspace_snapshot(request, payload);
    };
    core.terminal
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .acknowledge_output(TerminalAcknowledgeRequest {
            key: terminal_key(&scope)?,
            terminal_session_id: terminal.identity.terminal_session_id.clone(),
            process_generation: input.process_generation,
            through_chunk_sequence,
        })
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn terminal_output_requires_ack_cursor(terminal: &artifact::TerminalArtifactState) -> bool {
    terminal.output.sequence != 1
        || terminal.output.retained_bytes != 0
        || terminal.output.dropped_bytes != 0
        || !terminal.output.retained_base64.is_empty()
}

#[tauri::command]
fn artifact_open_file(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactOpenInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let selected = take_artifact_picker_grant(
        &core,
        &input.picker_grant_id,
        PickerPurpose::OpenFile,
        PickerObjectKind::File,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let relative = project_relative_picker_path(&selected, &scope, request.correlation_id.clone())?;
    let state = domain_file_state(&scope, &relative, 1, now_ms, request.correlation_id.clone())?;
    let record = new_artifact_record(
        &scope,
        artifact::ArtifactProviderDescriptor::file(),
        artifact::ArtifactState::File(Box::new(state)),
        "open-file",
        now_ms,
    )?;
    persist_artifact_record(&scope, &record, None, request.correlation_id.clone())?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_open_folder(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactOpenInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let selected = take_artifact_picker_grant(
        &core,
        &input.picker_grant_id,
        PickerPurpose::OpenFolder,
        PickerObjectKind::Folder,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let relative = project_relative_picker_path(&selected, &scope, request.correlation_id.clone())?;
    let state = domain_folder_state(&scope, &relative, 1, now_ms, request.correlation_id.clone())?;
    let record = new_artifact_record(
        &scope,
        artifact::ArtifactProviderDescriptor::folder(),
        artifact::ArtifactState::Folder(Box::new(state)),
        "open-folder",
        now_ms,
    )?;
    persist_artifact_record(&scope, &record, None, request.correlation_id.clone())?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn require_no_pending_artifact_write(
    core: &AppCoreState,
    artifact_id: &protocol::ArtifactId,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let process_pending = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .pending_writes
        .values()
        .any(|pending| pending.artifact_id == artifact_id.as_str());
    let scope = active_artifact_scope(core, correlation_id.clone())?;
    let durable_pending = scope
        .database
        .artifact_document(artifact_id.as_str())
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .map(|document| deserialize_artifact_record(document, correlation_id.clone()))
        .transpose()?
        .is_some_and(|record| match record.state {
            artifact::ArtifactState::File(file) => file.pending_save_requested_at_ms.is_some(),
            artifact::ArtifactState::Folder(_)
            | artifact::ArtifactState::Browser(_)
            | artifact::ArtifactState::Terminal(_)
            | artifact::ArtifactState::Unknown(_) => false,
        });
    if process_pending || durable_pending {
        Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "Resolve the pending File write approval before changing this Artifact",
            false,
        ))
    } else {
        Ok(())
    }
}

fn detach_active_native_browser(
    app: &tauri::AppHandle,
    core: &AppCoreState,
    except_artifact_id: Option<&str>,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let identity = {
        let mut state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        if state
            .browser_active_identity
            .as_ref()
            .is_some_and(|identity| Some(identity.artifact_id.as_str()) == except_artifact_id)
        {
            return Ok(());
        }
        let identity = state.browser_active_identity.take();
        if let Some(identity) = &identity {
            state
                .browser_mounted_artifacts
                .remove(&identity.artifact_id);
        }
        identity
    };
    if let Some(identity) = identity {
        browser::native::dispatch_detach(app, identity)
            .map_err(|_| workspace_state_unavailable(correlation_id))?;
    }
    Ok(())
}

#[tauri::command]
fn artifact_focus(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    detach_active_native_browser(
        &app,
        &core,
        Some(record.artifact_id.as_str()),
        request.correlation_id.clone(),
    )?;
    save_artifact_ui_state(
        &scope,
        Some(&record),
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_close_focus(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let ephemeral_close = before
        .focused_artifact_id
        .as_ref()
        .map(|artifact_id| artifact_id.as_str())
        .and_then(|artifact_id| {
            load_scoped_artifact_record(&scope, artifact_id, request.correlation_id.clone()).ok()
        })
        .filter(|record| {
            matches!(
                &record.state,
                artifact::ArtifactState::Browser(browser)
                    if matches!(
                        browser.environment.scope,
                        artifact::BrowserEnvironmentScope::None
                    )
            )
        })
        .map(|record| (record.artifact_id, Uuid::new_v4().to_string()));
    detach_active_native_browser(&app, &core, None, request.correlation_id.clone())?;
    if let Some((artifact_id, operation_id)) = ephemeral_close {
        core.artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
            .browser_ephemeral_clear_operations
            .insert(artifact_id.clone(), operation_id.clone());
        browser::native::dispatch_destroy_ephemeral(
            &app,
            Arc::clone(&core.browser_events),
            artifact_id,
            operation_id,
        )
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    }
    save_artifact_ui_state(&scope, None, now_ms, request.correlation_id.clone())?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_begin_file_edit(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can enter edit mode",
            false,
        ));
    };
    if file.draft.is_some() {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The File already has a retained draft",
            false,
        ));
    }
    file.begin_draft(now_ms).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The File could not enter edit mode",
            false,
        )
    })?;
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::DraftChanged,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_update_file_draft(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactFileDraftInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can retain a draft",
            false,
        ));
    };
    file.update_draft(input.content, now_ms).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The File draft changed before this update",
            true,
        )
    })?;
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::DraftChanged,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_discard_file_draft(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can discard a draft",
            false,
        ));
    };
    if file.draft.is_none() {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The File has no retained draft",
            false,
        ));
    }
    file.discard_draft();
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::DraftDiscarded,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_reject_file_proposal(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can reject a proposal",
            false,
        ));
    };
    file.reject_proposal(now_ms).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The File proposal is no longer pending",
            true,
        )
    })?;
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::ProposalChanged,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_resolve_file_conflict(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactFileConflictInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can resolve a save conflict",
            false,
        ));
    };
    if file.conflict.is_none() {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The File has no unresolved save conflict",
            false,
        ));
    }
    let retained_content = file_pending_content(file);
    let relative_path = PathBuf::from(&file.resource.project_relative_path);
    let live = scope
        .filesystem
        .read_utf8(&relative_path)
        .map_err(|error| {
            artifact_filesystem_error(
                error,
                request.correlation_id.clone(),
                "The live File could not be reloaded safely",
            )
        })?;
    let live_version = artifact::FileLiveVersion::new_with_target_version(
        file.live_version.sequence.saturating_add(1),
        live.version().target_version(),
        live.version().content_sha256(),
        live.version().byte_length(),
        now_ms,
    )
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    file.restore_live_snapshot(live.content().to_owned(), live_version, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    match input.resolution {
        ArtifactFileConflictResolution::ReloadCurrent => file.discard_draft(),
        ArtifactFileConflictResolution::KeepDraft => {
            file.begin_draft(now_ms)
                .and_then(|()| file.update_draft(retained_content, now_ms))
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        }
    }
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::RecoveryChanged,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_refresh_folder(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let expected_revision = record.record_revision;
    let artifact::ArtifactState::Folder(folder) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a Folder artifact can refresh its listing",
            false,
        ));
    };
    let relative = PathBuf::from(&folder.resource.project_relative_path);
    let sequence = folder.listing_version.sequence.saturating_add(1);
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let refreshed = domain_folder_state(
        &scope,
        &relative,
        sequence,
        now_ms,
        request.correlation_id.clone(),
    )?;
    record.state = artifact::ArtifactState::Folder(Box::new(refreshed));
    advance_artifact_record(
        &mut record,
        artifact::ArtifactHistoryKind::ResourceRefreshed,
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

fn artifact_folder_navigation_allowed(record: &artifact::ArtifactRecord, target: &str) -> bool {
    match &record.state {
        artifact::ArtifactState::Folder(folder) => folder
            .breadcrumbs
            .iter()
            .any(|breadcrumb| breadcrumb.project_relative_path == target),
        artifact::ArtifactState::File(file) => {
            let parent = file
                .resource
                .project_relative_path
                .rsplit_once('/')
                .map_or("", |(parent, _)| parent);
            target.is_empty() || target == parent || parent.starts_with(&format!("{target}/"))
        }
        artifact::ArtifactState::Browser(_)
        | artifact::ArtifactState::Terminal(_)
        | artifact::ArtifactState::Unknown(_) => false,
    }
}

#[tauri::command]
fn artifact_navigate_folder(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactFolderNavigateInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    if !artifact_folder_navigation_allowed(&record, &input.project_relative_path) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Forbidden,
            "The requested Folder is outside this artifact's breadcrumb authority",
            false,
        ));
    }
    let expected_revision = record.record_revision;
    let sequence = record.resource_version().sequence.saturating_add(1);
    let converted = matches!(record.state, artifact::ArtifactState::File(_));
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let state = domain_folder_state(
        &scope,
        Path::new(&input.project_relative_path),
        sequence,
        now_ms,
        request.correlation_id.clone(),
    )?;
    record.provider = artifact::ArtifactProviderDescriptor::folder();
    record.state = artifact::ArtifactState::Folder(Box::new(state));
    advance_artifact_record(
        &mut record,
        if converted {
            artifact::ArtifactHistoryKind::Converted
        } else {
            artifact::ArtifactHistoryKind::NavigationChanged
        },
        now_ms,
    )?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_select_folder_entry(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactFolderSelectInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::Folder(folder) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a Folder artifact has selectable entries",
            false,
        ));
    };
    let entry = folder
        .entries
        .iter()
        .find(|entry| entry.entry_id == input.entry_id)
        .cloned()
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Folder entry is unavailable",
                true,
            )
        })?;
    let mut source_folder = folder.clone();
    source_folder.select(&entry.entry_id).map_err(|_| {
        platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The Folder selection changed before the operation",
            true,
        )
    })?;
    let expected_revision = record.record_revision;
    let sequence = folder.listing_version.sequence.saturating_add(1);
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (provider, state, history_kind) = match entry.kind {
        artifact::FolderEntryKind::Folder => (
            artifact::ArtifactProviderDescriptor::folder(),
            artifact::ArtifactState::Folder(Box::new(domain_folder_state(
                &scope,
                Path::new(&entry.project_relative_path),
                sequence,
                now_ms,
                request.correlation_id.clone(),
            )?)),
            artifact::ArtifactHistoryKind::NavigationChanged,
        ),
        artifact::FolderEntryKind::File => {
            let file = domain_file_state(
                &scope,
                Path::new(&entry.project_relative_path),
                sequence,
                now_ms,
                request.correlation_id.clone(),
            )?;
            source_folder
                .convert_selected_file(Some(file.live_version.clone()), now_ms)
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            (
                artifact::ArtifactProviderDescriptor::file(),
                artifact::ArtifactState::File(Box::new(file)),
                artifact::ArtifactHistoryKind::Converted,
            )
        }
    };
    record.provider = provider;
    record.state = state;
    advance_artifact_record(&mut record, history_kind, now_ms)?;
    persist_artifact_record(
        &scope,
        &record,
        Some(expected_revision),
        request.correlation_id.clone(),
    )?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
async fn artifact_reply(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactReplyInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _conversation_operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let _artifact_operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    if matches!(record.state, artifact::ArtifactState::Unknown(_)) {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "This artifact version cannot provide Reply context",
            false,
        ));
    }
    let mut capture = durable_artifact_reply_capture(
        &record,
        input.selected_text,
        input.selected_entry_id,
        request.correlation_id.clone(),
    )?;
    let browser_capture = if let artifact::ArtifactState::Browser(browser) = &record.state {
        let identity = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
            .browser_active_identity
            .clone()
            .filter(|identity| {
                identity.artifact_id == record.artifact_id
                    && identity.controller_generation == browser.controller_generation
            })
            .ok_or_else(|| {
                platform_boundary_error(
                    request.correlation_id.clone(),
                    ProtocolErrorCode::Conflict,
                    "Focus the Browser before capturing immutable Reply context",
                    true,
                )
            })?;
        Some((identity, browser.current_navigation_sha256().to_owned()))
    } else {
        None
    };
    drop(record);
    drop(scope);
    drop(before);
    drop(_artifact_operation);
    drop(_conversation_operation);
    if let Some((identity, expected_navigation_sha256)) = browser_capture {
        let capture_app = app.clone();
        let page = tauri::async_runtime::spawn_blocking(move || {
            browser::native::capture_context(&capture_app, identity, expected_navigation_sha256)
        })
        .await
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .map_err(|_| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::Conflict,
                "The Browser page changed or could not be captured safely",
                true,
            )
        })?;
        capture.browser_page_context = Some(DurableBrowserPageContext {
            selected_text: page.selected_text,
            visible_text: page.visible_text,
            extracted_content: page.extracted_content,
        });
    }
    let _conversation_operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let _artifact_operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let current = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &current)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    require_current_artifact_reply_capture(&record, &capture, request.correlation_id.clone())?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let generation = {
        let mut conversation = core
            .conversation
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        if !matches!(
            conversation.pending,
            conversation::PendingChatState::Inactive
        ) || conversation.active_session_id.as_deref() != Some(scope.session_id.as_str())
        {
            return Err(platform_boundary_error(
                request.correlation_id,
                ProtocolErrorCode::Conflict,
                "Reply requires the active durable Chat",
                true,
            ));
        }
        let draft = conversation
            .drafts
            .entry(scope.session_id.clone())
            .or_insert_with(|| DurableConversationDraft {
                mode: "chat".into(),
                ..DurableConversationDraft::default()
            });
        draft.mode = "chat".into();
        draft.reply_target_id = Some(record.artifact_id.clone());
        draft.artifact_reply_capture = Some(capture);
        conversation
            .persist(&scope.database, &scope.workspace_id, now_ms)
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
    };
    core.conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .advance(generation)?;
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_expand_context(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactContextExpandInput,
) -> Result<ProtocolEnvelope<ConversationArtifactContextSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _artifact_operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    if artifact_resource_snapshot(record.resource_version()) != input.expected_resource_version {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::Conflict,
            "The Artifact changed before context expansion",
            true,
        ));
    }
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let context = capture_artifact_record_context(
        &record,
        input.selected_text.as_deref(),
        input.selected_entry_id.as_deref(),
        input.maximum_bytes as usize,
        now_ms,
        request.correlation_id.clone(),
    )?;
    let payload = project_artifact_context_snapshot(&context)?;
    let active_session_id = SessionId::new(scope.session_id)?;
    protocol::artifact_context_snapshot(request, payload, before.generation, &active_session_id)
}

fn durable_artifact_reply_capture(
    record: &artifact::ArtifactRecord,
    selected_text: Option<String>,
    selected_entry_id: Option<String>,
    correlation_id: protocol::CorrelationId,
) -> Result<DurableArtifactReplyCapture, ProtocolError> {
    match &record.state {
        artifact::ArtifactState::File(file) => {
            if selected_entry_id.is_some()
                || selected_text
                    .as_deref()
                    .is_some_and(|selection| !file_pending_content(file).contains(selection))
            {
                return Err(platform_boundary_error(
                    correlation_id,
                    ProtocolErrorCode::Conflict,
                    "The selected File context is stale or invalid",
                    true,
                ));
            }
        }
        artifact::ArtifactState::Folder(folder) => {
            if selected_text.is_some()
                || selected_entry_id.as_deref().is_some_and(|entry_id| {
                    !folder
                        .entries
                        .iter()
                        .any(|entry| entry.entry_id == entry_id)
                })
            {
                return Err(platform_boundary_error(
                    correlation_id,
                    ProtocolErrorCode::Conflict,
                    "The selected Folder context is stale or invalid",
                    true,
                ));
            }
        }
        artifact::ArtifactState::Browser(_) => {
            if selected_entry_id.is_some() || selected_text.is_some() {
                return Err(platform_boundary_error(
                    correlation_id,
                    ProtocolErrorCode::Conflict,
                    "Selected Browser page content is unavailable from the no-page-IPC boundary",
                    false,
                ));
            }
        }
        artifact::ArtifactState::Terminal(terminal) => {
            let output = terminal.output.safe_text().map_err(|_| {
                platform_boundary_error(
                    correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "The selected Terminal output is unavailable",
                    false,
                )
            })?;
            let selection_safe = terminal
                .output
                .is_safe_for_automatic_context()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                && !terminal_command_references_secret_environment(&terminal.command);
            if selected_entry_id.is_some()
                || selected_text
                    .as_deref()
                    .is_some_and(|selection| !selection_safe || !output.contains(selection))
            {
                return Err(platform_boundary_error(
                    correlation_id,
                    ProtocolErrorCode::Conflict,
                    "The selected Terminal output is stale or invalid",
                    true,
                ));
            }
        }
        artifact::ArtifactState::Unknown(_) => {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::InvalidPayload,
                "This artifact version cannot provide Reply context",
                false,
            ));
        }
    }
    Ok(DurableArtifactReplyCapture {
        artifact_id: record.artifact_id.clone(),
        artifact_record_revision: record.record_revision,
        resource_version: artifact_resource_snapshot(record.resource_version()),
        selected_text,
        selected_entry_id,
        browser_page_context: None,
    })
}

fn require_current_artifact_reply_capture(
    record: &artifact::ArtifactRecord,
    capture: &DurableArtifactReplyCapture,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    if capture.artifact_id != record.artifact_id
        || capture.artifact_record_revision != record.record_revision
        || capture.resource_version != artifact_resource_snapshot(record.resource_version())
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The selected Artifact Reply context changed before submission",
            true,
        ));
    }
    durable_artifact_reply_capture(
        record,
        capture.selected_text.clone(),
        capture.selected_entry_id.clone(),
        correlation_id.clone(),
    )?;
    match &record.state {
        artifact::ArtifactState::Browser(_) if capture.browser_page_context.is_none() => {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "Recapture the Browser Reply context from the focused page",
                true,
            ));
        }
        artifact::ArtifactState::Browser(_) => {}
        artifact::ArtifactState::File(_)
        | artifact::ArtifactState::Folder(_)
        | artifact::ArtifactState::Terminal(_)
        | artifact::ArtifactState::Unknown(_)
            if capture.browser_page_context.is_some() =>
        {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::Conflict,
                "The captured Browser Reply context no longer matches its Artifact",
                true,
            ));
        }
        _ => {}
    }
    Ok(())
}

fn file_save_candidate(
    file: &artifact::FileArtifactState,
) -> Result<(artifact::FileLiveVersion, String, ActionRequestOrigin), ProtocolError> {
    if let Some(proposal) = &file.proposal
        && matches!(proposal.status, artifact::FileProposalStatus::Pending)
    {
        return Ok((
            proposal.base_live_version.clone(),
            proposal.proposed_content.clone(),
            ActionRequestOrigin::ArtifactReplyProposal,
        ));
    }
    if let Some(draft) = &file.draft
        && draft.dirty
    {
        return Ok((
            draft.base_live_version.clone(),
            draft.content.clone(),
            ActionRequestOrigin::DirectUserEdit,
        ));
    }
    Err(ProtocolError::new(
        ProtocolErrorCode::Conflict,
        "The File has no unsaved change to save",
        false,
    ))
}

fn active_project_repository_state(scope: &ActiveArtifactScope) -> RepositoryState {
    let mut runner = ProductionGitRunner::default();
    match inspect_branch_control(
        scope.project_root.clone(),
        Path::new(TRUSTED_GIT_PROGRAM),
        &mut runner,
    ) {
        Ok(BranchControlVisibility::Visible(_)) => RepositoryState::VersionControlled,
        Ok(
            BranchControlVisibility::HiddenNonRepository
            | BranchControlVisibility::HiddenRepositoryCrossesProjectBoundary { .. },
        ) => RepositoryState::NotVersionControlled,
        Err(_) => RepositoryState::Unknown,
    }
}

fn current_artifact_live_authority(
    core: &AppCoreState,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<LiveAuthorityState, ProtocolError> {
    let runtime_snapshot = core
        .runtime
        .snapshot(now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let authority = core
        .runtime
        .policy_authority
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id))?;
    Ok(LiveAuthorityState {
        process_generation: 1,
        configuration_version: runtime_snapshot.generation.max(1),
        policy_version: authority.policy_version,
        revocation_epoch: authority.revocation_epoch,
    })
}

fn prepare_artifact_write(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &artifact::ArtifactRecord,
    preparation: ArtifactWritePreparation,
) -> Result<(PendingArtifactWrite, ActionFacts), ProtocolError> {
    let ArtifactWritePreparation {
        project_version,
        relative_path,
        content,
        request_origin,
        now_ms,
        correlation_id,
    } = preparation;
    let live = current_artifact_live_authority(core, now_ms, correlation_id.clone())?;
    let relative_text = normalized_project_relative_text(&relative_path, correlation_id.clone())?;
    let canonical_target = scope
        .project_root
        .canonical_root()
        .join(&relative_path)
        .to_string_lossy()
        .into_owned();
    if canonical_target.is_empty() || canonical_target.chars().any(char::is_control) {
        return Err(workspace_state_unavailable(correlation_id));
    }
    let action_id = format!("file-write-{}", Uuid::new_v4().as_simple());
    let content_sha256 = sha256_bytes(content.as_bytes());
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: "c4os.file".into(),
        arguments: serde_json::json!({
            "artifactId": record.artifact_id.clone(),
            "projectRelativePath": relative_text,
            "contentSha256": content_sha256,
            "byteLength": content.len(),
        }),
        risk: CanonicalRisk::Medium,
        requested_authority: BTreeSet::from(["file.write".into()]),
        canonical_target: canonical_target.clone(),
        target_version: project_version.target_version(),
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
        run_id: format!("artifact-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: None,
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::InvalidPayload,
            "The File write could not be bound to an exact action",
            false,
        )
    })?;
    let facts = ActionFacts {
        action_kind: "file.write".into(),
        native_tool: action.tool.clone(),
        surface: ActionSurface::File,
        effects: BTreeSet::from([ActionEffect::Modify]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::User,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin,
        repository_state: active_project_repository_state(scope),
        inside_active_project: true,
        canonical_target,
        workspace_id: scope.workspace_id.clone(),
        session_id: scope.session_id.clone(),
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
        PendingArtifactWrite {
            artifact_id: record.artifact_id.clone(),
            record_revision: record.record_revision,
            project_filesystem: scope.filesystem.clone(),
            relative_path,
            expected_file_version: project_version,
            content,
            action,
            live,
        },
        facts,
    ))
}

fn persist_file_save_result(
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    result: artifact::FileSaveResult,
    history_kind: artifact::ArtifactHistoryKind,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let expected_revision = record.record_revision;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(workspace_state_unavailable(correlation_id));
    };
    file.apply_save_result(result).map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The File save result no longer matches its retained draft",
            true,
        )
    })?;
    advance_artifact_record(record, history_kind, now_ms)?;
    persist_artifact_record(scope, record, Some(expected_revision), correlation_id)?;
    Ok(())
}

fn persist_file_save_requested(
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let expected_revision = record.record_revision;
    let artifact::ArtifactState::File(file) = &mut record.state else {
        return Err(workspace_state_unavailable(correlation_id));
    };
    file.mark_save_requested(now_ms).map_err(|_| {
        platform_boundary_error(
            correlation_id.clone(),
            ProtocolErrorCode::Conflict,
            "The File no longer has an exact save candidate",
            true,
        )
    })?;
    advance_artifact_record(record, artifact::ArtifactHistoryKind::SaveRequested, now_ms)?;
    persist_artifact_record(scope, record, Some(expected_revision), correlation_id)?;
    Ok(())
}

fn reconcile_interrupted_file_save_request(
    core: &AppCoreState,
    scope: &ActiveArtifactScope,
    record: &mut artifact::ArtifactRecord,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let artifact::ArtifactState::File(file) = &record.state else {
        return Ok(());
    };
    if file.pending_save_requested_at_ms.is_none() {
        return Ok(());
    }
    let already_requeued = core
        .artifact
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .pending_writes
        .values()
        .any(|pending| pending.artifact_id == record.artifact_id);
    if already_requeued {
        return Ok(());
    }

    let (expected_live, content, request_origin) = file_save_candidate(file)?;
    let relative_path = PathBuf::from(&file.resource.project_relative_path);
    let live = match scope.filesystem.read_utf8(&relative_path) {
        Ok(live) => live,
        Err(_) => {
            return persist_file_save_result(
                scope,
                record,
                artifact::FileSaveResult::Failed {
                    code: "live-file-unavailable".into(),
                    message:
                        "The interrupted File save target is unavailable; the draft was retained."
                            .into(),
                    retryable: true,
                    failed_at_ms: now_ms,
                },
                artifact::ArtifactHistoryKind::RecoveryChanged,
                now_ms,
                correlation_id,
            );
        }
    };
    if live.version().target_version() != expected_live.target_version {
        let observed_live = artifact::FileLiveVersion::new_with_target_version(
            expected_live.sequence.saturating_add(1),
            live.version().target_version(),
            live.version().content_sha256(),
            live.version().byte_length(),
            now_ms,
        )
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        return persist_file_save_result(
            scope,
            record,
            artifact::FileSaveResult::Conflict {
                observed_live,
                message: "The live File changed while approval was interrupted.".into(),
                observed_at_ms: now_ms,
            },
            artifact::ArtifactHistoryKind::ConflictObserved,
            now_ms,
            correlation_id,
        );
    }

    let (pending, facts) = prepare_artifact_write(
        core,
        scope,
        record,
        ArtifactWritePreparation {
            project_version: live.version().clone(),
            relative_path,
            content,
            request_origin,
            now_ms,
            correlation_id: correlation_id.clone(),
        },
    )?;
    let proposal = core
        .runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator
                .requeue_interrupted_direct_approval(&facts, pending.action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => persist_file_save_result(
            scope,
            record,
            artifact::FileSaveResult::Failed {
                code: "policy-denied".into(),
                message: "Policy denied the interrupted File write; the draft was retained.".into(),
                retryable: false,
                failed_at_ms: now_ms,
            },
            artifact::ArtifactHistoryKind::RecoveryChanged,
            now_ms,
            correlation_id,
        ),
        GatewayProposal::PendingApproval { prompt, .. } => {
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(correlation_id))?
                .pending_writes
                .insert(prompt.prompt_id, pending);
            Ok(())
        }
        GatewayProposal::Authorized { .. } => Err(workspace_state_unavailable(correlation_id)),
    }
}

fn execute_artifact_write(
    core: &AppCoreState,
    pending: PendingArtifactWrite,
    token: AuthorizationToken,
    approval_prompt_id: Option<&str>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<(), ProtocolError> {
    let scope = active_artifact_scope(core, correlation_id.clone())?;
    let artifact_id = protocol::ArtifactId::new(pending.artifact_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &artifact_id,
        pending.record_revision,
        correlation_id.clone(),
    )?;
    let artifact::ArtifactState::File(file) = &record.state else {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The approved Artifact is no longer a File",
            true,
        ));
    };
    let (expected_live, content, _) = file_save_candidate(file)?;
    if content != pending.content
        || expected_live.target_version != pending.expected_file_version.target_version()
        || pending.action.target_version != pending.expected_file_version.target_version()
        || pending.action.workspace_id != scope.workspace_id
        || pending.action.session_id != scope.session_id
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The approved File write changed before execution",
            true,
        ));
    }
    let filesystem = pending.project_filesystem.clone();
    let relative_path = pending.relative_path.clone();
    let expected_version = pending.expected_file_version.clone();
    let content_for_effect = pending.content.clone();
    let canonical_target = pending.action.canonical_target.clone();
    let completed_at_ms = now_ms.saturating_add(1);
    let mut write_outcome = None;
    core.runtime
        .coordinator()
        .and_then(|mut coordinator| {
            coordinator.execute_direct_action(
                &token,
                &pending.action,
                pending.live,
                approval_prompt_id,
                now_ms,
                |_permit| {
                    let outcome = filesystem.write_utf8(
                        &relative_path,
                        &content_for_effect,
                        &ExpectedFileState::Existing(expected_version),
                    );
                    let normalized = match &outcome {
                        Ok(_) => NormalizedActionResult {
                            status: NormalizedActionStatus::Succeeded,
                            result_code: "file-write-succeeded".into(),
                            exit_code: Some(0),
                            changed_targets: vec![canonical_target.clone()],
                            output_sha256: Some(sha256_bytes(content_for_effect.as_bytes())),
                            completed_at_ms,
                        },
                        Err(
                            ProjectFilesystemError::Conflict
                            | ProjectFilesystemError::TargetChanged,
                        ) => NormalizedActionResult {
                            status: NormalizedActionStatus::Failed,
                            result_code: "file-write-conflict".into(),
                            exit_code: Some(1),
                            changed_targets: Vec::new(),
                            output_sha256: None,
                            completed_at_ms,
                        },
                        Err(_) => NormalizedActionResult {
                            status: NormalizedActionStatus::Failed,
                            result_code: "file-write-failed".into(),
                            exit_code: None,
                            changed_targets: Vec::new(),
                            output_sha256: None,
                            completed_at_ms,
                        },
                    };
                    write_outcome = Some(outcome);
                    normalized
                },
            )?;
            Ok(())
        })
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let outcome =
        write_outcome.ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
    match outcome {
        Ok(outcome) => {
            let version = artifact::FileLiveVersion::new_with_target_version(
                expected_live.sequence.saturating_add(1),
                outcome.version().target_version(),
                outcome.version().content_sha256(),
                outcome.version().byte_length(),
                completed_at_ms,
            )
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            persist_file_save_result(
                &scope,
                &mut record,
                artifact::FileSaveResult::Saved {
                    content: pending.content,
                    live_version: version,
                    saved_at_ms: completed_at_ms,
                },
                artifact::ArtifactHistoryKind::SaveCompleted,
                completed_at_ms,
                correlation_id,
            )
        }
        Err(ProjectFilesystemError::Conflict | ProjectFilesystemError::TargetChanged) => {
            match scope.filesystem.read_utf8(&pending.relative_path) {
                Ok(observed) => {
                    let observed_live = artifact::FileLiveVersion::new_with_target_version(
                        expected_live.sequence.saturating_add(1),
                        observed.version().target_version(),
                        observed.version().content_sha256(),
                        observed.version().byte_length(),
                        completed_at_ms,
                    )
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
                    persist_file_save_result(
                        &scope,
                        &mut record,
                        artifact::FileSaveResult::Conflict {
                            observed_live,
                            message: "The live File changed before the atomic save.".into(),
                            observed_at_ms: completed_at_ms,
                        },
                        artifact::ArtifactHistoryKind::ConflictObserved,
                        completed_at_ms,
                        correlation_id,
                    )
                }
                Err(_) => persist_file_save_result(
                    &scope,
                    &mut record,
                    artifact::FileSaveResult::Failed {
                        code: "live-file-unavailable".into(),
                        message: "The live File became unavailable; the draft was retained.".into(),
                        retryable: true,
                        failed_at_ms: completed_at_ms,
                    },
                    artifact::ArtifactHistoryKind::RecoveryChanged,
                    completed_at_ms,
                    correlation_id,
                ),
            }
        }
        Err(_) => persist_file_save_result(
            &scope,
            &mut record,
            artifact::FileSaveResult::Failed {
                code: "atomic-write-failed".into(),
                message: "The atomic File save failed; the draft was retained.".into(),
                retryable: true,
                failed_at_ms: completed_at_ms,
            },
            artifact::ArtifactHistoryKind::RecoveryChanged,
            completed_at_ms,
            correlation_id,
        ),
    }
}

#[tauri::command]
fn artifact_save_file(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactMutationInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    require_no_pending_artifact_write(&core, &input.artifact_id, request.correlation_id.clone())?;
    let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
    let mut record = load_active_artifact_record(
        &scope,
        &input.artifact_id,
        input.base_record_revision,
        request.correlation_id.clone(),
    )?;
    let artifact::ArtifactState::File(file) = &record.state else {
        return Err(platform_boundary_error(
            request.correlation_id,
            ProtocolErrorCode::InvalidPayload,
            "Only a File artifact can be saved",
            false,
        ));
    };
    let (expected_live, content, request_origin) = file_save_candidate(file)?;
    let relative_path = PathBuf::from(&file.resource.project_relative_path);
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let live = match scope.filesystem.read_utf8(&relative_path) {
        Ok(live) => live,
        Err(_) => {
            persist_file_save_result(
                &scope,
                &mut record,
                artifact::FileSaveResult::Failed {
                    code: "live-file-unavailable".into(),
                    message: "The live File is unavailable; the draft was retained.".into(),
                    retryable: true,
                    failed_at_ms: now_ms,
                },
                artifact::ArtifactHistoryKind::RecoveryChanged,
                now_ms,
                request.correlation_id.clone(),
            )?;
            let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
            return protocol::artifact_workspace_snapshot(request, payload);
        }
    };
    if live.version().target_version() != expected_live.target_version {
        let observed_live = artifact::FileLiveVersion::new_with_target_version(
            expected_live.sequence.saturating_add(1),
            live.version().target_version(),
            live.version().content_sha256(),
            live.version().byte_length(),
            now_ms,
        )
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        persist_file_save_result(
            &scope,
            &mut record,
            artifact::FileSaveResult::Conflict {
                observed_live,
                message: "The live File changed before the save request.".into(),
                observed_at_ms: now_ms,
            },
            artifact::ArtifactHistoryKind::ConflictObserved,
            now_ms,
            request.correlation_id.clone(),
        )?;
        let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
        return protocol::artifact_workspace_snapshot(request, payload);
    }
    let (mut pending, facts) = prepare_artifact_write(
        &core,
        &scope,
        &record,
        ArtifactWritePreparation {
            project_version: live.version().clone(),
            relative_path,
            content,
            request_origin,
            now_ms,
            correlation_id: request.correlation_id.clone(),
        },
    )?;
    persist_file_save_requested(&scope, &mut record, now_ms, request.correlation_id.clone())?;
    pending.record_revision = record.record_revision;
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
            persist_file_save_result(
                &scope,
                &mut record,
                artifact::FileSaveResult::Failed {
                    code: "policy-denied".into(),
                    message: "Policy denied the File write; the draft was retained.".into(),
                    retryable: false,
                    failed_at_ms: now_ms,
                },
                artifact::ArtifactHistoryKind::RecoveryChanged,
                now_ms,
                request.correlation_id.clone(),
            )?;
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            core.artifact
                .lock()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                .pending_writes
                .insert(prompt.prompt_id, pending);
        }
        GatewayProposal::Authorized { token, .. } => {
            execute_artifact_write(
                &core,
                pending,
                token,
                None,
                now_ms,
                request.correlation_id.clone(),
            )?;
        }
    }
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
}

#[tauri::command]
fn artifact_answer_approval(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: ArtifactApprovalInput,
) -> Result<ProtocolEnvelope<ArtifactWorkspaceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    input.validate()?;
    let _operation = core
        .artifact_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let before = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    require_exact_artifact_generation(&request, &before)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (pending_write, pending_terminal, pending_browser) = {
        let state = core
            .artifact
            .lock()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
        (
            state.pending_writes.get(&input.prompt_id).cloned(),
            state
                .pending_terminal_actions
                .get(&input.prompt_id)
                .cloned(),
            state.pending_browser_actions.get(&input.prompt_id).cloned(),
        )
    };
    let expected_action = pending_write
        .as_ref()
        .map(|pending| &pending.action)
        .or_else(|| pending_terminal.as_ref().map(|pending| &pending.action))
        .or_else(|| pending_browser.as_ref().map(|pending| &pending.action))
        .ok_or_else(|| {
            platform_boundary_error(
                request.correlation_id.clone(),
                ProtocolErrorCode::NotFound,
                "The Artifact approval is no longer active",
                false,
            )
        })?;
    let answer = match input.answer {
        ArtifactApprovalAnswer::Allow => ApprovalAnswer::Allow,
        ArtifactApprovalAnswer::Deny => ApprovalAnswer::Deny,
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
            if &prompt.action != expected_action {
                return Err(workspace_state_unavailable(request.correlation_id));
            }
            if let Some(pending) = pending_write {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_writes
                    .remove(&input.prompt_id);
                let scope = active_artifact_scope(&core, request.correlation_id.clone())?;
                let artifact_id = protocol::ArtifactId::new(pending.artifact_id.clone())?;
                let mut record = load_active_artifact_record(
                    &scope,
                    &artifact_id,
                    pending.record_revision,
                    request.correlation_id.clone(),
                )?;
                persist_file_save_result(
                    &scope,
                    &mut record,
                    artifact::FileSaveResult::Failed {
                        code: "approval-denied".into(),
                        message: "The File write was cancelled; the draft was retained.".into(),
                        retryable: false,
                        failed_at_ms: now_ms,
                    },
                    artifact::ArtifactHistoryKind::RecoveryChanged,
                    now_ms,
                    request.correlation_id.clone(),
                )?;
            } else if let Some(pending) = pending_terminal {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_terminal_actions
                    .remove(&input.prompt_id);
                if matches!(&pending.payload, PendingArtifactTerminalPayload::Run { .. }) {
                    persist_terminal_failure(
                        &pending,
                        "approval-denied",
                        "The Terminal operation was cancelled.",
                        false,
                        now_ms,
                        request.correlation_id.clone(),
                    )?;
                }
            } else if let Some(pending) = pending_browser {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_browser_actions
                    .remove(&input.prompt_id);
                discard_native_browser_request(
                    &core,
                    &app,
                    &pending,
                    request.correlation_id.clone(),
                )?;
                persist_browser_policy_denial(&pending, now_ms, request.correlation_id.clone())?;
            }
        }
        ApprovalResponse::Authorized { prompt, token } => {
            if &prompt.action != expected_action {
                return Err(workspace_state_unavailable(request.correlation_id));
            }
            if let Some(pending) = pending_write {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_writes
                    .remove(&input.prompt_id);
                execute_artifact_write(
                    &core,
                    pending,
                    token,
                    Some(&input.prompt_id),
                    now_ms,
                    request.correlation_id.clone(),
                )?;
            } else if let Some(pending) = pending_terminal {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_terminal_actions
                    .remove(&input.prompt_id);
                execute_terminal_action(
                    &core,
                    pending,
                    token,
                    Some(&input.prompt_id),
                    now_ms,
                    request.correlation_id.clone(),
                )?;
            } else if let Some(pending) = pending_browser {
                core.artifact
                    .lock()
                    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                    .pending_browser_actions
                    .remove(&input.prompt_id);
                execute_browser_action(
                    &core,
                    &app,
                    pending,
                    token,
                    Some(&input.prompt_id),
                    now_ms,
                    request.correlation_id.clone(),
                )?;
            }
        }
    }
    let payload = build_artifact_workspace_snapshot(&core, request.correlation_id.clone())?;
    protocol::artifact_workspace_snapshot(request, payload)
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
    let _artifact_operation = core
        .artifact_operation
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
            if draft.reply_target_id != input.reply_target_id {
                draft.artifact_reply_capture = None;
            }
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
    let (current_generation, reply_target, active_session_id) = {
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
        .then(|| {
            active_draft
                .reply_target_id
                .clone()
                .map(|target_id| (target_id, active_draft.artifact_reply_capture.clone()))
        })
        .flatten()
        .zip(conversation.active_session_id.clone())
        .map(|((target_id, capture), session_id)| (target_id, session_id, capture));
        (
            current_generation,
            reply_target,
            conversation.active_session_id.clone(),
        )
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
        .map(|(target_id, session_id, capture)| {
            if let Some(context) = artifact_reply_context(
                &core,
                &session_id,
                &target_id,
                capture.as_ref(),
                input.provider_id.as_deref().zip(input.model_id.as_deref()),
                now_ms,
                request.correlation_id.clone(),
            )? {
                Ok(context)
            } else {
                message_reply_context(
                    &core.runtime,
                    &session_id,
                    &target_id,
                    request.correlation_id.clone(),
                )
            }
        })
        .transpose()?;
    synchronize_extension_skill_sources(&core, now_ms, request.correlation_id.clone())?;
    let mut skill_context = core
        .extensions
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
        .active_turn_skills()
        .map_err(|error| extension_boundary_error(error, request.correlation_id.clone()))?
        .into_iter()
        .map(|skill| SkillContextSnapshot {
            identity: skill.identity.stable_id(),
            package_id: skill.package_id,
            entrypoint_sha256: skill.entrypoint_digest,
            instructions: skill.instructions,
            referenced_resources: skill.referenced_resources,
        })
        .collect::<Vec<_>>();
    let existing_skill_bytes = skill_context
        .iter()
        .map(|skill| skill.instructions.len())
        .sum::<usize>();
    let hook_context = dispatch_reviewed_extension_hooks(
        &core,
        "before-turn",
        serde_json::json!({
            "workspaceId": workspace_id,
            "promptSha256": input.prompt.as_deref().map(|prompt| sha256_bytes(prompt.as_bytes())),
            "promptBytes": input.prompt.as_deref().map_or(0, str::len),
            "retainedAttachmentCount": input.retained_attachment_ids.len(),
            "newAttachmentCount": input.picker_grant_ids.len(),
        }),
        &workspace_id,
        active_session_id.as_deref().unwrap_or("extension-hook"),
        runtime::session::MAX_SKILL_CONTEXTS_PER_TURN.saturating_sub(skill_context.len()),
        runtime::session::MAX_SKILL_CONTEXT_BYTES.saturating_sub(existing_skill_bytes),
        now_ms,
        request.correlation_id.clone(),
    )?;
    skill_context.extend(hook_context);

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
            let mcp_turn = tauri::async_runtime::block_on(async {
                core.mcp
                    .lock()
                    .await
                    .prepare_turn_snapshot(&workspace_id, &project_id, &session_id, now_ms)
                    .await
            })
            .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
            let mcp_turn = (!mcp_turn.tools.is_empty()).then_some(mcp_turn);
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
            ConversationSubmissionDispatch::Turn(Box::new(ConversationTurnDispatchIntent {
                workspace_id: workspace_id.clone(),
                project_id,
                session_id,
                prompt: (!draft.prompt.trim().is_empty()).then(|| draft.prompt.clone()),
                attachments: draft.attachments.clone(),
                skill_context: skill_context.clone(),
                mcp_turn,
                provider_id: draft.provider_id.clone(),
                model_id: draft.model_id.clone(),
                reasoning_mode: draft.reasoning_mode.clone(),
                reply_context,
                submitted_at_ms: now_ms,
            }))
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
            let mcp_turn = tauri::async_runtime::block_on(async {
                core.mcp
                    .lock()
                    .await
                    .prepare_turn_snapshot(
                        &workspace_id,
                        &promotion.project_id,
                        &promotion.session_id,
                        now_ms,
                    )
                    .await
            })
            .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
            let mcp_turn = (!mcp_turn.tools.is_empty()).then_some(mcp_turn);
            ConversationSubmissionDispatch::First(Box::new(ConversationFirstDispatchIntent {
                workspace_id: workspace_id.clone(),
                project_id: promotion.project_id.clone(),
                session_id: promotion.session_id.clone(),
                prompt: promotion.prompt.clone(),
                attachments: conversation.pending_attachments.clone(),
                skill_context: skill_context.clone(),
                mcp_turn,
                provider_id: input.provider_id.clone(),
                model_id: input.model_id.clone(),
                reasoning_mode: input.reasoning_mode.clone(),
                submitted_at_ms: now_ms,
            }))
        };
        conversation.advance(current_generation)?;
        (intent, current_generation)
    };

    let first_submission = matches!(&dispatch_intent, ConversationSubmissionDispatch::First(_));
    let dispatch_generation = match dispatch_intent {
        ConversationSubmissionDispatch::First(intent) => core
            .runtime
            .dispatch_conversation_first(*intent)
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
            .dispatch_conversation_turn(*intent)
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
            artifact_reply_capture: None,
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
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let cancellation = core
        .runtime_production
        .load(request.correlation_id.clone())?
        .cancel_dispatch(runtime_generation, &identity, now_ms);
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let cancellation = core
        .runtime
        .cancel_dispatch(runtime_generation, &identity, now_ms);
    let generation = cancellation
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
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    session_id: SessionId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let _artifact_operation = core
        .artifact_operation
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
    let selection_changed =
        conversation.active_session_id.as_deref() != Some(chat.chat_id.as_str());
    if selection_changed {
        detach_active_native_browser(&app, &core, None, request.correlation_id.clone())?;
    }
    conversation.active_project_id = Some(chat.project_id.clone());
    conversation.active_session_id = Some(chat.chat_id.clone());
    let persisted_generation = conversation
        .persist(&database, &workspace_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let focus_generation = if selection_changed {
        clear_persisted_artifact_focus(
            &database,
            &workspace_id,
            now_ms,
            request.correlation_id.clone(),
        )?
        .unwrap_or(0)
    } else {
        0
    };
    conversation.advance(
        current_generation
            .max(persisted_generation)
            .max(focus_generation),
    )?;
    drop(conversation);
    let payload = build_conversation_snapshot(&core, request.correlation_id.clone())?;
    protocol::conversation_snapshot(request, payload)
}

#[tauri::command]
fn conversation_activate_project(
    app: tauri::AppHandle,
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    project_id: ProjectId,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let _operation = core
        .conversation_operation
        .lock()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let _artifact_operation = core
        .artifact_operation
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
    let next_session_id = workspace_snapshot
        .chats
        .iter()
        .find(|chat| {
            chat.project_id == project_id.as_str()
                && chat.lifecycle_state == core::database::LifecycleState::Active
        })
        .map(|chat| chat.chat_id.clone());
    let selection_changed = conversation.active_project_id.as_deref() != Some(project_id.as_str())
        || conversation.active_session_id != next_session_id;
    if selection_changed {
        detach_active_native_browser(&app, &core, None, request.correlation_id.clone())?;
    }
    conversation.active_project_id = Some(project_id.as_str().into());
    conversation.active_session_id = next_session_id;
    let persisted_generation = conversation
        .persist(&database, &workspace_id, now_ms)
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let focus_generation = if selection_changed {
        clear_persisted_artifact_focus(
            &database,
            &workspace_id,
            now_ms,
            request.correlation_id.clone(),
        )?
        .unwrap_or(0)
    } else {
        0
    };
    conversation.advance(
        current_generation
            .max(persisted_generation)
            .max(focus_generation),
    )?;
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
    let _artifact_operation = core
        .artifact_operation
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
        plugin_or_mcp_id: None,
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
                    artifact_context: turn
                        .reply_context
                        .as_ref()
                        .and_then(|reply| reply.artifact_context.as_ref())
                        .map(project_artifact_context_snapshot)
                        .transpose()?,
                    mcp_provenance: turn.mcp_turn.as_ref().map(project_mcp_provenance),
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

fn project_mcp_provenance(snapshot: &mcp::McpTurnSnapshot) -> ConversationMcpProvenanceSnapshot {
    let server_count = snapshot
        .tools
        .iter()
        .map(|tool| tool.server_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    ConversationMcpProvenanceSnapshot {
        snapshot_id: snapshot.snapshot_id.clone(),
        server_count: u16::try_from(server_count).unwrap_or(u16::MAX),
        tool_count: u16::try_from(snapshot.tools.len()).unwrap_or(u16::MAX),
        omitted_tool_count: snapshot.omitted_tool_count,
        truncated: snapshot.truncated,
        tools: snapshot
            .tools
            .iter()
            .map(|tool| {
                let (source_kind, source_id) = match &tool.source {
                    mcp::McpDefinitionSource::User => ("user", None),
                    mcp::McpDefinitionSource::Plugin { package_id, .. } => {
                        ("plugin", Some(package_id.clone()))
                    }
                };
                ConversationMcpToolProvenanceSnapshot {
                    server_id: tool.server_id.clone(),
                    source_kind: source_kind.into(),
                    source_id,
                    tool_name: tool.tool_name.clone(),
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod mcp_provenance_projection_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn conversation_projection_exposes_only_safe_mcp_provenance() {
        let schema_canary = "credential-canary-in-schema";
        let definition_canary = format!("sha256:{}", "d".repeat(64));
        let snapshot = mcp::McpTurnSnapshot {
            snapshot_id: format!("mcp-turn:{}", "a".repeat(64)),
            service_generation: 7,
            captured_at_ms: 12,
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            tools: vec![mcp::McpTurnToolSnapshot {
                target_id: format!("mcp-tool:{}", "b".repeat(64)),
                server_id: "plugin-server".into(),
                source: mcp::McpDefinitionSource::Plugin {
                    package_id: "com.example.safe-plugin".into(),
                    declaration_id: "credential-canary-declaration".into(),
                },
                lifecycle_generation: 4,
                definition_sha256: definition_canary.clone(),
                transport_kind: mcp::McpTransportKind::Stdio,
                tool_name: "safe-tool".into(),
                title: Some("Credential canary title".into()),
                description: Some("Credential canary description".into()),
                input_schema: json!({ "secret": schema_canary }),
                input_schema_sha256: format!("sha256:{}", "c".repeat(64)),
                output_schema_sha256: Some(format!("sha256:{}", "e".repeat(64))),
            }],
            truncated: true,
            omitted_tool_count: 1,
            sha256: format!("sha256:{}", "f".repeat(64)),
        };

        let projected = project_mcp_provenance(&snapshot);
        assert_eq!(projected.server_count, 1);
        assert_eq!(projected.tool_count, 1);
        assert_eq!(projected.omitted_tool_count, 1);
        assert!(projected.truncated);
        assert_eq!(projected.tools[0].source_kind, "plugin");
        assert_eq!(
            projected.tools[0].source_id.as_deref(),
            Some("com.example.safe-plugin")
        );

        let serialized = serde_json::to_string(&projected).expect("serialize projection");
        assert!(serialized.contains("plugin-server"));
        assert!(serialized.contains("safe-tool"));
        for canary in [
            schema_canary,
            definition_canary.as_str(),
            "credential-canary-declaration",
            "Credential canary title",
            "Credential canary description",
        ] {
            assert!(!serialized.contains(canary), "leaked MCP canary: {canary}");
        }
    }
}

fn project_artifact_context_snapshot(
    context: &artifact::ArtifactContextSnapshot,
) -> Result<ConversationArtifactContextSnapshot, ProtocolError> {
    context.validate().map_err(|_| {
        ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Persisted Artifact Reply context is invalid",
            false,
        )
    })?;
    let (payload_kind, segments) = match &context.payload {
        artifact::ContextPayload::File { segments, .. } => ("file", segments),
        artifact::ContextPayload::Folder { segments, .. } => ("folder", segments),
        artifact::ContextPayload::Browser { segments, .. } => ("browser", segments),
        artifact::ContextPayload::Terminal { segments, .. } => ("terminal", segments),
    };
    Ok(ConversationArtifactContextSnapshot {
        snapshot_id: context.snapshot_id.clone(),
        stable_reference: context.stable_reference.clone(),
        artifact_id: protocol::ArtifactId::new(context.artifact_id.clone())?,
        project_id: ProjectId::new(context.project_id.clone())?,
        session_id: SessionId::new(context.session_id.clone())?,
        provider_type: context.provider_type.clone(),
        provider_version: context.provider_schema_version,
        artifact_record_revision: context.artifact_record_revision,
        captured_resource_version: ArtifactResourceVersionSnapshot {
            sequence: context.captured_live_version.sequence,
            sha256: context.captured_live_version.sha256.clone(),
            observed_at_ms: context.captured_live_version.observed_at_ms,
        },
        payload_kind: payload_kind.into(),
        segments: segments
            .iter()
            .map(|segment| ConversationArtifactContextSegmentSnapshot {
                priority: match segment.priority {
                    artifact::ContextPriority::Selection => "selection",
                    artifact::ContextPriority::VisibleOrCurrent => "visibleOrCurrent",
                    artifact::ContextPriority::Recent => "recent",
                    artifact::ContextPriority::Metadata => "metadata",
                }
                .into(),
                source: segment.source.clone(),
                text: segment.text.clone(),
                original_bytes: segment.original_bytes,
                omitted_bytes: segment.omitted_bytes,
            })
            .collect(),
        maximum_bytes: context.budget.maximum_bytes,
        used_bytes: context.budget.used_bytes,
        omitted_bytes: context.budget.omitted_bytes,
        omitted_segments: context.budget.omitted_segments,
        truncated: context.budget.truncated,
        unsaved: context.unsaved,
        redactions: context
            .redactions
            .iter()
            .map(|redaction| {
                match redaction {
                    artifact::ContextRedaction::Secrets => "secrets",
                    artifact::ContextRedaction::BrowserCredentials => "browserCredentials",
                    artifact::ContextRedaction::BrowserStorage => "browserStorage",
                    artifact::ContextRedaction::BrowserUnrelatedHistory => {
                        "browserUnrelatedHistory"
                    }
                    artifact::ContextRedaction::TerminalRawEnvironment => "terminalRawEnvironment",
                    artifact::ContextRedaction::TerminalPasswords => "terminalPasswords",
                    artifact::ContextRedaction::TerminalUnrelatedHistory => {
                        "terminalUnrelatedHistory"
                    }
                }
                .into()
            })
            .collect(),
        capabilities: context
            .capabilities
            .iter()
            .map(|capability| ConversationArtifactCapabilitySnapshot {
                capability_id: capability.capability_id.clone(),
                access: match capability.access {
                    artifact::CapabilityAccess::Readable => "readable",
                    artifact::CapabilityAccess::ApprovalRequired => "approvalRequired",
                    artifact::CapabilityAccess::Denied => "denied",
                    artifact::CapabilityAccess::Unknown => "unknown",
                }
                .into(),
                reason_code: capability.reason_code.clone(),
            })
            .collect(),
        captured_at_ms: context.captured_at_ms,
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

const MIN_ARTIFACT_REPLY_CONTEXT_BUDGET_BYTES: usize = 16 * 1_024;

fn artifact_reply_context_budget(
    core: &AppCoreState,
    session_id: &str,
    provider_id: Option<&str>,
    model_id: Option<&str>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<usize, ProtocolError> {
    let session = core
        .runtime
        .session(session_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let preferred_runtime_id = session.binding().map(|binding| binding.runtime_id.as_str());
    let effective = core
        .runtime
        .effective_conversation_models(preferred_runtime_id, now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let route = provider_id
        .zip(model_id)
        .map(|(provider, model)| (provider.to_owned(), model.to_owned()))
        .or_else(|| {
            core.runtime.snapshot(now_ms).ok().and_then(|runtime| {
                runtime
                    .providers
                    .providers
                    .iter()
                    .filter(|provider| provider.profile.enabled)
                    .find_map(|provider| {
                        provider.selected_model_id.as_ref().map(|model_id| {
                            (provider.profile.provider_id.clone(), model_id.clone())
                        })
                    })
            })
        });
    let context_tokens = route
        .as_ref()
        .and_then(|route| effective.get(route))
        .and_then(|descriptor| descriptor.numeric_maximum(NumericCapabilityKey::ContextTokens));
    let derived = context_tokens
        .and_then(|tokens| usize::try_from(tokens).ok())
        .map(|tokens| tokens.saturating_mul(4) / 8)
        .unwrap_or(MIN_ARTIFACT_REPLY_CONTEXT_BUDGET_BYTES);
    Ok(derived.clamp(
        MIN_ARTIFACT_REPLY_CONTEXT_BUDGET_BYTES,
        MAX_REPLY_SOURCE_EXCERPT_BYTES,
    ))
}

fn artifact_context_segments(payload: &artifact::ContextPayload) -> &[artifact::ContextSegment] {
    match payload {
        artifact::ContextPayload::File { segments, .. }
        | artifact::ContextPayload::Folder { segments, .. }
        | artifact::ContextPayload::Browser { segments, .. }
        | artifact::ContextPayload::Terminal { segments, .. } => segments,
    }
}

fn artifact_reply_context(
    core: &AppCoreState,
    session_id: &str,
    target_id: &str,
    reply_capture: Option<&DurableArtifactReplyCapture>,
    route: Option<(&str, &str)>,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<Option<MessageReplyContextSnapshot>, ProtocolError> {
    let (database, workspace_id) = {
        let active = core
            .active_workspace
            .lock()
            .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
        let active = active
            .as_ref()
            .ok_or_else(|| workspace_state_unavailable(correlation_id.clone()))?;
        (
            Arc::clone(active.database_actor()),
            active.manifest().workspace_id.to_string(),
        )
    };
    let Some(document) = database
        .artifact_document(target_id)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
    else {
        return Ok(None);
    };
    let record = deserialize_artifact_record(document, correlation_id.clone())?;
    let active_project_id = core
        .conversation
        .lock()
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
        .active_project_id
        .clone();
    if record.workspace_id != workspace_id
        || record.session_id != session_id
        || active_project_id.as_deref() != Some(record.project_id.as_str())
    {
        return Err(platform_boundary_error(
            correlation_id,
            ProtocolErrorCode::Conflict,
            "The Artifact Reply target is no longer active",
            true,
        ));
    }
    if let Some(capture) = reply_capture {
        require_current_artifact_reply_capture(&record, capture, correlation_id.clone())?;
    }
    let budget = artifact_reply_context_budget(
        core,
        session_id,
        route.map(|(provider_id, _)| provider_id),
        route.map(|(_, model_id)| model_id),
        now_ms,
        correlation_id.clone(),
    )?;
    let snapshot = capture_artifact_record_context_with_browser(
        &record,
        reply_capture.and_then(|capture| capture.selected_text.as_deref()),
        reply_capture.and_then(|capture| capture.selected_entry_id.as_deref()),
        reply_capture.and_then(|capture| capture.browser_page_context.as_ref()),
        budget,
        now_ms,
        correlation_id.clone(),
    )?;
    let snapshot_document = serde_json::to_vec(&snapshot)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let mut source_excerpt = artifact_context_segments(&snapshot.payload)
        .iter()
        .filter(|segment| !segment.text.is_empty())
        .map(|segment| format!("[{}]\n{}", segment.source, segment.text))
        .collect::<Vec<_>>()
        .join("\n\n");
    if source_excerpt.len() > MAX_REPLY_SOURCE_EXCERPT_BYTES {
        let mut boundary = MAX_REPLY_SOURCE_EXCERPT_BYTES;
        while !source_excerpt.is_char_boundary(boundary) {
            boundary = boundary.saturating_sub(1);
        }
        source_excerpt.truncate(boundary);
    }
    Ok(Some(MessageReplyContextSnapshot {
        target_id: record.artifact_id,
        target_kind: record.provider.type_id,
        source_sha256: sha256_bytes(&snapshot_document),
        source_excerpt,
        artifact_context: Some(snapshot),
    }))
}

fn capture_artifact_record_context(
    record: &artifact::ArtifactRecord,
    selected_text: Option<&str>,
    selected_entry_id: Option<&str>,
    maximum_bytes: usize,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::ArtifactContextSnapshot, ProtocolError> {
    capture_artifact_record_context_with_browser(
        record,
        selected_text,
        selected_entry_id,
        None,
        maximum_bytes,
        now_ms,
        correlation_id,
    )
}

fn capture_artifact_record_context_with_browser(
    record: &artifact::ArtifactRecord,
    selected_text: Option<&str>,
    selected_entry_id: Option<&str>,
    browser_page_context: Option<&DurableBrowserPageContext>,
    maximum_bytes: usize,
    now_ms: u64,
    correlation_id: protocol::CorrelationId,
) -> Result<artifact::ArtifactContextSnapshot, ProtocolError> {
    durable_artifact_reply_capture(
        record,
        selected_text.map(str::to_owned),
        selected_entry_id.map(str::to_owned),
        correlation_id.clone(),
    )?;
    let identity = artifact::ContextCaptureIdentity {
        snapshot_id: format!("artifact-context-{}", Uuid::new_v4().as_simple()),
        artifact_id: record.artifact_id.clone(),
        workspace_id: record.workspace_id.clone(),
        project_id: record.project_id.clone(),
        session_id: record.session_id.clone(),
        provider_type: record.provider.type_id.clone(),
        provider_schema_version: record.provider.schema_version,
        artifact_record_revision: record.record_revision,
        live_resource_version: record.resource_version(),
    };
    let readable = || artifact::CapabilitySummaryEntry {
        capability_id: "artifact.read".into(),
        access: artifact::CapabilityAccess::Readable,
        reason_code: None,
    };
    let mut selected_folder = None;
    let input = match &record.state {
        artifact::ArtifactState::File(file) => {
            let selected_text = selected_text
                .map(artifact::SafeContextText::new)
                .transpose()
                .map_err(|_| {
                    platform_boundary_error(
                        correlation_id.clone(),
                        ProtocolErrorCode::InvalidPayload,
                        "The selected File context is invalid",
                        false,
                    )
                })?;
            artifact::ContextCaptureInput::File(artifact::FileContextInput {
                state: file,
                selected_text,
                visible_text: None,
                recent_text: None,
                redactions: vec![artifact::ContextRedaction::Secrets],
                capabilities: vec![
                    readable(),
                    artifact::CapabilitySummaryEntry {
                        capability_id: "artifact.write".into(),
                        access: artifact::CapabilityAccess::ApprovalRequired,
                        reason_code: Some("live-version-revalidation".into()),
                    },
                ],
            })
        }
        artifact::ArtifactState::Folder(folder) => {
            if let Some(entry_id) = selected_entry_id {
                let mut folder = folder.as_ref().clone();
                folder.select(entry_id).map_err(|_| {
                    platform_boundary_error(
                        correlation_id.clone(),
                        ProtocolErrorCode::Conflict,
                        "The selected Folder context is stale",
                        true,
                    )
                })?;
                selected_folder = Some(folder);
            }
            artifact::ContextCaptureInput::Folder(artifact::FolderContextInput {
                state: selected_folder.as_ref().unwrap_or(folder),
                recent_text: None,
                redactions: vec![artifact::ContextRedaction::Secrets],
                capabilities: vec![readable()],
            })
        }
        artifact::ArtifactState::Browser(browser) => {
            if selected_text.is_some() || selected_entry_id.is_some() {
                return Err(platform_boundary_error(
                    correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "Browser Reply cannot capture page-selected content without page IPC",
                    false,
                ));
            }
            let recent_activity = browser
                .history
                .iter()
                .rev()
                .take(16)
                .rev()
                .map(|entry| {
                    entry
                        .title
                        .as_deref()
                        .map(|title| format!("{title} — {}", entry.display_url))
                        .unwrap_or_else(|| entry.display_url.clone())
                })
                .collect::<Vec<_>>()
                .join("\n");
            artifact::ContextCaptureInput::Browser(artifact::BrowserContextInput {
                current_url: artifact::SafeContextText::new(
                    browser.current_display_url().to_owned(),
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                title: artifact::SafeContextText::new(browser_protocol_title(browser))
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                navigation_state: artifact::SafeContextText::new(format!(
                    "historyIndex={};canGoBack={};canGoForward={};phase={}",
                    browser.current_history_index,
                    browser.can_go_back(),
                    browser.can_go_forward(),
                    browser.phase.phase(),
                ))
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                selected_text: browser_page_context
                    .and_then(|page| page.selected_text.clone())
                    .map(artifact::SafeContextText::new)
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                visible_text: browser_page_context
                    .and_then(|page| page.visible_text.clone())
                    .map(artifact::SafeContextText::new)
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                extracted_content: browser_page_context
                    .and_then(|page| page.extracted_content.clone())
                    .map(artifact::SafeContextText::new)
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                recent_activity: (!recent_activity.is_empty())
                    .then(|| artifact::SafeContextText::new(recent_activity))
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                capabilities: vec![
                    readable(),
                    artifact::CapabilitySummaryEntry {
                        capability_id: "browser.navigate".into(),
                        access: artifact::CapabilityAccess::ApprovalRequired,
                        reason_code: Some("action-gateway".into()),
                    },
                ],
            })
        }
        artifact::ArtifactState::Terminal(terminal) => {
            if selected_entry_id.is_some() {
                return Err(platform_boundary_error(
                    correlation_id.clone(),
                    ProtocolErrorCode::InvalidPayload,
                    "Terminal Reply cannot select a Folder entry",
                    false,
                ));
            }
            let output = terminal
                .output
                .safe_text()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            let automatic_output_safe = terminal
                .output
                .is_safe_for_automatic_context()
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                && !terminal_command_references_secret_environment(&terminal.command);
            let selected_output = selected_text
                .filter(|_| automatic_output_safe)
                .map(artifact::SafeContextText::new)
                .transpose()
                .map_err(|_| {
                    platform_boundary_error(
                        correlation_id.clone(),
                        ProtocolErrorCode::InvalidPayload,
                        "The selected Terminal context is invalid",
                        false,
                    )
                })?;
            let mut recent_start = output.len().saturating_sub(64 * 1_024);
            while !output.is_char_boundary(recent_start) {
                recent_start = recent_start.saturating_add(1);
            }
            artifact::ContextCaptureInput::Terminal(artifact::TerminalContextInput {
                command: artifact::SafeContextText::new(terminal.command.clone())
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                working_directory_display: artifact::SafeContextText::new(
                    terminal.working_directory_display.clone(),
                )
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                environment_id: terminal.process.environment_id.clone(),
                process_state: terminal.status.phase().into(),
                exit_code: terminal.status.exit_code(),
                selected_output,
                visible_output: automatic_output_safe
                    .then(|| artifact::SafeContextText::new(output.clone()))
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                recent_output_tail: automatic_output_safe
                    .then(|| artifact::SafeContextText::new(output[recent_start..].to_owned()))
                    .transpose()
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?,
                capabilities: vec![
                    readable(),
                    artifact::CapabilitySummaryEntry {
                        capability_id: "terminal.execute".into(),
                        access: artifact::CapabilityAccess::ApprovalRequired,
                        reason_code: Some("action-gateway".into()),
                    },
                ],
            })
        }
        artifact::ArtifactState::Unknown(_) => {
            return Err(platform_boundary_error(
                correlation_id,
                ProtocolErrorCode::InvalidPayload,
                "This Artifact version cannot be captured for Reply",
                false,
            ));
        }
    };
    artifact::capture_artifact_context(identity, input, maximum_bytes, now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id))
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
        artifact_context: None,
    })
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn runtime_sampling_approval_summary(
    approval: mcp::production_sampling::McpSamplingApprovalSummary,
) -> RuntimeApprovalSummary {
    RuntimeApprovalSummary {
        summary: format!(
            "MCP server {} requests up to {} tokens from {}/{} using {} bounded text message(s).",
            approval.server_id,
            approval.max_tokens,
            approval.provider_id,
            approval.model_id,
            approval.message_count,
        ),
        runtime_id: approval.runtime_id,
        correlation_id: approval.correlation_id,
        prompt_id: approval.prompt_id,
        approval_kind: "mcp-sampling".into(),
        server_id: Some(approval.server_id),
        provider_id: Some(approval.provider_id),
        model_id: Some(approval.model_id),
        max_tokens: Some(approval.max_tokens),
        expires_at_ms: Some(approval.expires_at_ms),
        message_count: Some(approval.message_count),
        input_bytes: Some(approval.input_bytes),
        has_system_prompt: Some(approval.has_system_prompt),
        parent_operation: Some(approval.parent_operation),
        disclosure_scope: Some(
            "Private active-operation text will be disclosed to the selected model provider; credentials remain operation-scoped and hidden."
                .into(),
        ),
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
// The approval bridge keeps every exact caller and prompt identity visible.
#[allow(clippy::too_many_arguments)]
fn answer_runtime_sampling_approval(
    runtime: &RuntimeApplicationService,
    approvals: &mcp::production_sampling::ProductionMcpSamplingApprovalRegistry,
    request_correlation: protocol::CorrelationId,
    expected_coordinator_generation: u64,
    runtime_id: &str,
    correlation_id: &str,
    prompt_id: &str,
    answer: ApprovalAnswer,
    now_ms: u64,
) -> Result<bool, ProtocolError> {
    approvals
        .answer(
            runtime,
            expected_coordinator_generation,
            runtime_id,
            correlation_id,
            prompt_id,
            answer,
            now_ms,
        )
        .map_err(|_| runtime_production_unavailable(request_correlation))
}

#[tauri::command]
fn runtime_core_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<RuntimeCoreSnapshot>, protocol::StructuredCoreError> {
    let correlation_id = request.correlation_id.clone();
    let now_ms =
        current_time_ms().map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let (runtime, capability_generation, mut pending_approvals, sampling_approvals) = {
        let mut attempts = 0usize;
        loop {
            let (runtime, capability_generation) = core
                .runtime
                .snapshot_with_capability_generation(now_ms)
                .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
            let pending_approvals = core
                .runtime_production
                .load(request.correlation_id.clone())?
                .pending_approvals()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?
                .into_iter()
                .map(|approval| RuntimeApprovalSummary {
                    summary: format!("Approval required by {}.", approval.runtime_id),
                    runtime_id: approval.runtime_id,
                    correlation_id: approval.correlation_id,
                    prompt_id: approval.prompt_id,
                    approval_kind: "runtime-effect".into(),
                    server_id: None,
                    provider_id: None,
                    model_id: None,
                    max_tokens: None,
                    expires_at_ms: None,
                    message_count: None,
                    input_bytes: None,
                    has_system_prompt: None,
                    parent_operation: None,
                    disclosure_scope: None,
                })
                .collect::<Vec<_>>();
            let stable_sampling = core
                .mcp_sampling_approvals
                .stable_summaries()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            if let Some((approval_revision, sampling_approvals)) = stable_sampling {
                let verified_generation = core
                    .runtime
                    .snapshot(now_ms)
                    .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?
                    .generation;
                if verified_generation == runtime.generation
                    && core.mcp_sampling_approvals.stable_revision() == Some(approval_revision)
                {
                    break (
                        runtime,
                        capability_generation,
                        pending_approvals,
                        sampling_approvals,
                    );
                }
            }
            attempts = attempts.saturating_add(1);
            if attempts >= 32 {
                return Err(workspace_state_unavailable(correlation_id));
            }
            std::thread::yield_now();
        }
    };
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let (runtime, capability_generation) = core
        .runtime
        .snapshot_with_capability_generation(now_ms)
        .map_err(|_| workspace_state_unavailable(correlation_id.clone()))?;
    let generation = StateGeneration(runtime.generation);
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pending_approvals.extend(
        sampling_approvals
            .into_iter()
            .map(runtime_sampling_approval_summary),
    );
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
    let sampling_answered = answer_runtime_sampling_approval(
        &core.runtime,
        &core.mcp_sampling_approvals,
        request_correlation.clone(),
        request.expected_generation.0,
        &runtime_id,
        &correlation_id,
        &prompt_id,
        answer,
        now_ms,
    )?;
    if !sampling_answered {
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
    }
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

fn require_mcp_input_generation(
    request: &SnapshotRequest,
    expected_generation: u64,
) -> Result<(), ProtocolError> {
    if request.expected_generation.0 == expected_generation {
        Ok(())
    } else {
        Err(platform_boundary_error(
            request.correlation_id.clone(),
            ProtocolErrorCode::InvalidGeneration,
            "The MCP request generations do not match",
            false,
        ))
    }
}

fn mcp_boundary_error(
    error: mcp::McpError,
    correlation_id: protocol::CorrelationId,
) -> ProtocolError {
    let (code, message, retryable) = match error {
        mcp::McpError::InvalidInput | mcp::McpError::InvalidState => (
            ProtocolErrorCode::InvalidPayload,
            "The MCP request is invalid for the current server state",
            false,
        ),
        mcp::McpError::BoundExceeded => (
            ProtocolErrorCode::PayloadTooLarge,
            "The MCP request exceeded a bounded limit",
            false,
        ),
        mcp::McpError::Conflict => (
            ProtocolErrorCode::Conflict,
            "MCP state changed before the operation completed",
            true,
        ),
        mcp::McpError::Untrusted | mcp::McpError::Denied | mcp::McpError::Revoked => (
            ProtocolErrorCode::Forbidden,
            "MCP policy or trust denied the operation",
            false,
        ),
        mcp::McpError::Disabled | mcp::McpError::NotReady => (
            ProtocolErrorCode::Conflict,
            "The MCP server is not ready for this operation",
            true,
        ),
        mcp::McpError::UnsupportedProtocol => (
            ProtocolErrorCode::Unavailable,
            "The MCP server did not negotiate the required protocol",
            false,
        ),
        mcp::McpError::TimedOut => (
            ProtocolErrorCode::Unavailable,
            "The MCP operation timed out",
            true,
        ),
        mcp::McpError::Cancelled => (
            ProtocolErrorCode::Conflict,
            "The MCP operation was cancelled",
            true,
        ),
        mcp::McpError::Transport(_)
        | mcp::McpError::Persistence(_)
        | mcp::McpError::Credential
        | mcp::McpError::StateUnavailable => (
            ProtocolErrorCode::Unavailable,
            "The native MCP service is unavailable",
            true,
        ),
    };
    platform_boundary_error(correlation_id, code, message, retryable)
}

fn mcp_snapshot_envelope(
    request: SnapshotRequest,
    snapshot: mcp::McpServiceSnapshot,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    let generation = StateGeneration(snapshot.generation);
    protocol::snapshot_envelope(request, generation, snapshot)
}

fn mcp_trust_envelope(
    request: SnapshotRequest,
    response: mcp::McpTrustResponse,
) -> Result<ProtocolEnvelope<mcp::McpTrustResponse>, protocol::StructuredCoreError> {
    let generation = StateGeneration(response.snapshot.generation);
    protocol::snapshot_envelope(request, generation, response)
}

fn prepare_mcp_trust(
    runtime: &RuntimeApplicationService,
    snapshot: &mcp::McpServiceSnapshot,
    server: &mcp::McpServerSnapshot,
    definition_sha256: String,
    _now_ms: u64,
) -> Result<(CanonicalAction, ActionFacts), mcp::McpError> {
    if server.trust != mcp::McpTrustState::Pending
        || server.lifecycle != mcp::McpLifecycle::Disabled
    {
        return Err(mcp::McpError::InvalidState);
    }
    let (workspace_id, session_id) = match &server.scope {
        mcp::McpScope::Application => ("c4os-settings".to_owned(), "mcp-settings".to_owned()),
        mcp::McpScope::Workspace { workspace_id }
        | mcp::McpScope::Project { workspace_id, .. }
        | mcp::McpScope::Chat { workspace_id, .. } => {
            let session_id = match &server.scope {
                mcp::McpScope::Chat { session_id, .. } => session_id.clone(),
                _ => "mcp-settings".to_owned(),
            };
            (workspace_id.clone(), session_id)
        }
    };
    let remote = matches!(
        server.transport,
        mcp::McpTransportDefinition::StreamableHttp { .. }
    );
    let credential_bound = match &server.transport {
        mcp::McpTransportDefinition::Stdio { environment, .. } => environment
            .iter()
            .any(|binding| !matches!(binding.source, mcp::McpEnvironmentSource::Literal { .. })),
        mcp::McpTransportDefinition::StreamableHttp { .. } => true,
    };
    let live = runtime
        .current_direct_live_authority(server.lifecycle_generation, snapshot.generation)
        .map_err(|_| mcp::McpError::StateUnavailable)?;
    let action_id = format!("mcp-trust-{}", Uuid::new_v4().as_simple());
    let target = format!("mcp-definition:{}:{definition_sha256}", server.server_id);
    let identity = format!(
        "mcp-{}",
        sha256_bytes(server.server_id.as_bytes()).trim_start_matches("sha256:")
    );
    let action = CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.clone(),
        tool_call_id: format!("tool-{action_id}"),
        tool: "mcp.definition.trust".into(),
        arguments: serde_json::json!({
            "definitionSha256": definition_sha256.clone(),
            "serverId": server.server_id,
            "transportKind": server.transport.kind(),
        }),
        risk: CanonicalRisk::High,
        requested_authority: BTreeSet::from(["mcp-definition-trust".into()]),
        canonical_target: target.clone(),
        target_version: definition_sha256.clone(),
        workspace_id: workspace_id.clone(),
        session_id: session_id.clone(),
        run_id: format!("mcp-trust-run-{}", Uuid::new_v4().as_simple()),
        runtime_id: "c4os-core".into(),
        environment_id: "desktop".into(),
        plugin_or_mcp_id: Some(identity.clone()),
        process_generation: live.process_generation,
        configuration_version: live.configuration_version,
        policy_version: live.policy_version,
        revocation_epoch: live.revocation_epoch,
    };
    action.validate().map_err(|_| mcp::McpError::InvalidState)?;
    let facts = ActionFacts {
        action_kind: action.tool.clone(),
        native_tool: action.tool.clone(),
        surface: if remote {
            ActionSurface::Network
        } else {
            ActionSurface::Process
        },
        effects: BTreeSet::from([ActionEffect::Control]),
        scope: if remote {
            ActionScope::Remote
        } else {
            ActionScope::ExternalLocal
        },
        initiator: ActionInitiator::User,
        sensitivity: if credential_bound {
            ActionSensitivity::Credential
        } else {
            ActionSensitivity::Ordinary
        },
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: RepositoryState::NotApplicable,
        inside_active_project: false,
        canonical_target: target,
        workspace_id,
        session_id,
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: Some(identity),
        target_resolved: true,
        authenticated: remote,
        trusted_root: false,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    };
    Ok((action, facts))
}

// Trust execution keeps the reviewed definition, action, token, and prompt
// identities separate at the final effect boundary.
#[allow(clippy::too_many_arguments)]
async fn execute_mcp_trust(
    core: &AppCoreState,
    server_id: String,
    service_generation: u64,
    definition_sha256: String,
    action_binding_sha256: String,
    action: CanonicalAction,
    token: AuthorizationToken,
    approval_prompt_id: &str,
    now_ms: u64,
) -> Result<mcp::McpTrustResponse, mcp::McpError> {
    {
        let service = core.mcp.lock().await;
        let snapshot = service.snapshot();
        let server = snapshot
            .servers
            .iter()
            .find(|server| server.server_id == server_id)
            .ok_or(mcp::McpError::Conflict)?;
        let pending = server
            .pending_trust_approval
            .as_ref()
            .ok_or(mcp::McpError::Conflict)?;
        if snapshot.generation != service_generation
            || service.definition_sha256(&server_id)? != definition_sha256
            || pending.prompt_id != approval_prompt_id
            || pending.action_binding_sha256 != action_binding_sha256
        {
            return Err(mcp::McpError::Conflict);
        }
    }
    let live = core
        .runtime
        .current_direct_live_authority(action.process_generation, action.configuration_version)
        .map_err(|_| mcp::McpError::StateUnavailable)?;
    let lease = core
        .runtime
        .begin_direct_action_effect(&token, &action, live, Some(approval_prompt_id), now_ms)
        .map_err(|_| mcp::McpError::Denied)?;
    let trust_result = core.mcp.lock().await.trust_server(
        &mcp::McpServerMutationInput {
            expected_generation: service_generation,
            server_id: server_id.clone(),
        },
        approval_prompt_id,
        &action_binding_sha256,
        &definition_sha256,
        now_ms,
    );
    let normalized = match &trust_result {
        Ok(_) => NormalizedActionResult {
            status: NormalizedActionStatus::Succeeded,
            result_code: "mcp-definition-trusted".into(),
            exit_code: None,
            changed_targets: vec![action.canonical_target.clone()],
            output_sha256: Some(definition_sha256.clone()),
            completed_at_ms: now_ms.max(1),
        },
        Err(_) => NormalizedActionResult {
            status: NormalizedActionStatus::Failed,
            result_code: "mcp-definition-trust-failed".into(),
            exit_code: None,
            changed_targets: Vec::new(),
            output_sha256: None,
            completed_at_ms: now_ms.max(1),
        },
    };
    core.runtime
        .complete_direct_action_effect(lease, normalized)
        .map_err(|_| mcp::McpError::StateUnavailable)?;
    let snapshot = trust_result?;
    Ok(mcp::McpTrustResponse {
        snapshot,
        status: mcp::McpTrustRequestStatus::Trusted,
        server_id,
        definition_sha256,
        prompt_id: None,
        prompt_expires_at_ms: None,
    })
}

#[tauri::command]
async fn mcp_snapshot(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    let snapshot = core.mcp.lock().await.snapshot();
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_save_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerDefinitionInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let server_id = input.server_id.clone();
    let snapshot = core
        .mcp
        .lock()
        .await
        .upsert_server(input, mcp::McpDefinitionSource::User, now_ms)
        .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        core.mcp_cancellations.allow_server(&server_id);
        core.mcp_cancellations
            .refresh_credential_bindings(&snapshot);
    }
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_request_trust(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpTrustRequestInput,
) -> Result<ProtocolEnvelope<mcp::McpTrustResponse>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (snapshot, server, definition_sha256) = {
        let service = core.mcp.lock().await;
        let snapshot = service.snapshot();
        if snapshot.generation != input.expected_generation {
            return Err(mcp_boundary_error(
                mcp::McpError::Conflict,
                request.correlation_id,
            ));
        }
        let server = snapshot
            .servers
            .iter()
            .find(|server| server.server_id == input.server_id)
            .cloned()
            .ok_or_else(|| {
                mcp_boundary_error(mcp::McpError::InvalidInput, request.correlation_id.clone())
            })?;
        let definition_sha256 = service
            .definition_sha256(&input.server_id)
            .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
        (snapshot, server, definition_sha256)
    };
    if server.trust == mcp::McpTrustState::Trusted {
        return mcp_trust_envelope(
            request,
            mcp::McpTrustResponse {
                snapshot,
                status: mcp::McpTrustRequestStatus::Trusted,
                server_id: input.server_id,
                definition_sha256,
                prompt_id: None,
                prompt_expires_at_ms: None,
            },
        );
    }
    if let Some(approval) = server.pending_trust_approval.as_ref()
        && approval.state == mcp::McpTrustApprovalState::Pending
        && approval.expires_at_ms > now_ms
        && approval.definition_sha256 == definition_sha256
    {
        return mcp_trust_envelope(
            request,
            mcp::McpTrustResponse {
                snapshot,
                status: mcp::McpTrustRequestStatus::PendingApproval,
                server_id: input.server_id,
                definition_sha256,
                prompt_id: Some(approval.prompt_id.clone()),
                prompt_expires_at_ms: Some(approval.expires_at_ms),
            },
        );
    }
    let recovering = server.pending_trust_approval.is_some();
    let (action, facts) = prepare_mcp_trust(
        &core.runtime,
        &snapshot,
        &server,
        definition_sha256.clone(),
        now_ms,
    )
    .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
    let proposal = if recovering {
        core.runtime.coordinator().and_then(|mut coordinator| {
            coordinator
                .requeue_interrupted_direct_approval(&facts, action.clone(), now_ms)
                .map(|operation| operation.value)
                .map_err(Into::into)
        })
    } else {
        core.runtime
            .propose_direct_trust_confirmation(&facts, action.clone(), now_ms)
    }
    .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    match proposal {
        GatewayProposal::Denied { .. } => {
            let snapshot = if let Some(approval) = server.pending_trust_approval {
                core.mcp
                    .lock()
                    .await
                    .clear_trust_approval(
                        &mcp::McpServerMutationInput {
                            expected_generation: snapshot.generation,
                            server_id: input.server_id.clone(),
                        },
                        &approval.prompt_id,
                        "policy_denied",
                        now_ms,
                    )
                    .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?
            } else {
                snapshot
            };
            mcp_trust_envelope(
                request,
                mcp::McpTrustResponse {
                    snapshot,
                    status: mcp::McpTrustRequestStatus::Denied,
                    server_id: input.server_id,
                    definition_sha256,
                    prompt_id: None,
                    prompt_expires_at_ms: None,
                },
            )
        }
        GatewayProposal::Authorized { .. } => {
            Err(workspace_state_unavailable(request.correlation_id))
        }
        GatewayProposal::PendingApproval { prompt, .. } => {
            let prompt_id = prompt.prompt_id.clone();
            let expires_at_ms = prompt.expires_at_ms;
            let action_binding_sha256 = prompt
                .action
                .binding_digest()
                .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
            let snapshot = core
                .mcp
                .lock()
                .await
                .record_trust_approval(
                    &mcp::McpServerMutationInput {
                        expected_generation: snapshot.generation,
                        server_id: input.server_id.clone(),
                    },
                    mcp::McpPendingTrustApproval {
                        prompt_id: prompt_id.clone(),
                        definition_sha256: definition_sha256.clone(),
                        action_binding_sha256,
                        action_configuration_version: action.configuration_version,
                        requested_at_ms: prompt.created_at_ms,
                        expires_at_ms,
                        state: mcp::McpTrustApprovalState::Pending,
                    },
                    now_ms,
                )
                .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
            mcp_trust_envelope(
                request,
                mcp::McpTrustResponse {
                    snapshot,
                    status: mcp::McpTrustRequestStatus::PendingApproval,
                    server_id: input.server_id,
                    definition_sha256,
                    prompt_id: Some(prompt_id),
                    prompt_expires_at_ms: Some(expires_at_ms),
                },
            )
        }
    }
}

#[tauri::command]
async fn mcp_answer_trust(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpTrustApprovalInput,
) -> Result<ProtocolEnvelope<mcp::McpTrustResponse>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let (snapshot, pending) = {
        let service = core.mcp.lock().await;
        let snapshot = service.snapshot();
        let pending = snapshot
            .servers
            .iter()
            .find(|server| server.server_id == input.server_id)
            .and_then(|server| server.pending_trust_approval.clone())
            .ok_or_else(|| {
                mcp_boundary_error(mcp::McpError::Conflict, request.correlation_id.clone())
            })?;
        (snapshot, pending)
    };
    if snapshot.generation != input.expected_generation
        || pending.prompt_id != input.prompt_id
        || pending.state != mcp::McpTrustApprovalState::Pending
        || pending.expires_at_ms <= now_ms
    {
        return Err(mcp_boundary_error(
            mcp::McpError::Conflict,
            request.correlation_id,
        ));
    }
    let answer = match input.answer {
        mcp::McpTrustApprovalAnswer::Allow => ApprovalAnswer::Allow,
        mcp::McpTrustApprovalAnswer::Deny => ApprovalAnswer::Deny,
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
    let response_binding = match &response {
        ApprovalResponse::Denied { prompt } | ApprovalResponse::Authorized { prompt, .. } => prompt
            .action
            .binding_digest()
            .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?,
    };
    if response_binding != pending.action_binding_sha256 {
        return Err(workspace_state_unavailable(request.correlation_id));
    }
    match response {
        ApprovalResponse::Denied { .. } => {
            let snapshot = core
                .mcp
                .lock()
                .await
                .clear_trust_approval(
                    &mcp::McpServerMutationInput {
                        expected_generation: snapshot.generation,
                        server_id: input.server_id.clone(),
                    },
                    &input.prompt_id,
                    "user_denied",
                    now_ms,
                )
                .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
            mcp_trust_envelope(
                request,
                mcp::McpTrustResponse {
                    snapshot,
                    status: mcp::McpTrustRequestStatus::Denied,
                    server_id: input.server_id,
                    definition_sha256: pending.definition_sha256,
                    prompt_id: None,
                    prompt_expires_at_ms: None,
                },
            )
        }
        ApprovalResponse::Authorized { prompt, token } => {
            let response = execute_mcp_trust(
                &core,
                input.server_id,
                snapshot.generation,
                pending.definition_sha256,
                pending.action_binding_sha256,
                prompt.action,
                token,
                &input.prompt_id,
                now_ms,
            )
            .await
            .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
            mcp_trust_envelope(request, response)
        }
    }
}

#[tauri::command]
async fn mcp_test_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerMutationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut service = core.mcp.lock().await;
    let previous_generation = service.snapshot().generation;
    let snapshot = match service.test_server(&input, now_ms).await {
        Ok(snapshot) => snapshot,
        Err(_error)
            if service.snapshot().generation > previous_generation
                && service.snapshot().servers.iter().any(|server| {
                    server.server_id == input.server_id
                        && server.lifecycle == mcp::McpLifecycle::Failed
                }) =>
        {
            service.snapshot()
        }
        Err(error) => {
            return Err(mcp_boundary_error(error, request.correlation_id.clone()));
        }
    };
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_enable_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerMutationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut service = core.mcp.lock().await;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    core.mcp_cancellations.allow_server(&input.server_id);
    let previous_generation = service.snapshot().generation;
    let snapshot = match service.enable_server(&input, now_ms).await {
        Ok(snapshot) => snapshot,
        Err(_error)
            if service.snapshot().generation > previous_generation
                && service.snapshot().servers.iter().any(|server| {
                    server.server_id == input.server_id
                        && server.lifecycle == mcp::McpLifecycle::Failed
                }) =>
        {
            service.snapshot()
        }
        Err(error) => {
            return Err(mcp_boundary_error(error, request.correlation_id.clone()));
        }
    };
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_recover_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerMutationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut service = core.mcp.lock().await;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    core.mcp_cancellations.allow_server(&input.server_id);
    let previous_generation = service.snapshot().generation;
    let snapshot = match service.recover_server(&input, now_ms).await {
        Ok(snapshot) => snapshot,
        Err(_error)
            if service.snapshot().generation > previous_generation
                && service.snapshot().servers.iter().any(|server| {
                    server.server_id == input.server_id
                        && server.lifecycle == mcp::McpLifecycle::Failed
                }) =>
        {
            service.snapshot()
        }
        Err(error) => {
            return Err(mcp_boundary_error(error, request.correlation_id.clone()));
        }
    };
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_disable_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerMutationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut service = core.mcp.lock().await;
    if service.snapshot().generation != input.expected_generation {
        return Err(mcp_boundary_error(
            mcp::McpError::Conflict,
            request.correlation_id,
        ));
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    core.mcp_cancellations.quiesce_server(&input.server_id);
    let snapshot = service
        .disable_server(&input, now_ms)
        .await
        .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_revoke_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerRevocationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let mut service = core.mcp.lock().await;
    if service.snapshot().generation != input.expected_generation {
        return Err(mcp_boundary_error(
            mcp::McpError::Conflict,
            request.correlation_id,
        ));
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    core.mcp_cancellations.quiesce_server(&input.server_id);
    let snapshot = service
        .revoke_server(&input, now_ms)
        .await
        .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
    mcp_snapshot_envelope(request, snapshot)
}

#[tauri::command]
async fn mcp_delete_server(
    core: tauri::State<'_, AppCoreState>,
    request: SnapshotRequest,
    input: mcp::McpServerMutationInput,
) -> Result<ProtocolEnvelope<mcp::McpServiceSnapshot>, protocol::StructuredCoreError> {
    validate_snapshot_request(&request)?;
    require_mcp_input_generation(&request, input.expected_generation)?;
    let now_ms = current_time_ms()
        .map_err(|_| workspace_state_unavailable(request.correlation_id.clone()))?;
    let snapshot = core
        .mcp
        .lock()
        .await
        .delete_server(&input, now_ms)
        .map_err(|error| mcp_boundary_error(error, request.correlation_id.clone()))?;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        core.mcp_cancellations.allow_server(&input.server_id);
        core.mcp_cancellations
            .refresh_credential_bindings(&snapshot);
    }
    mcp_snapshot_envelope(request, snapshot)
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
    credential_vault: Option<security::credentials::CredentialVault>,
    active_workspace: Arc<Mutex<Option<core::services::ActiveWorkspace>>>,
    runtime: Arc<RuntimeApplicationService>,
    mcp: Arc<tokio::sync::Mutex<ProductionMcpService>>,
    mcp_cancellations: ProductionMcpCancellationRegistry,
    sampling_parents: mcp::production_sampling::McpSamplingParentRegistry,
    managed: Arc<ManagedProductionRuntime>,
) -> Result<(), std::io::Error> {
    thread::Builder::new()
        .name("c4os-production-runtime-initialization".into())
        .spawn(move || {
            let initialize = || -> Result<Arc<ProductionRuntimeApplication>, String> {
                let bootstrap = runtime::production::RuntimeProductionBootstrap::new(
                    resource_dir,
                    credential_vault,
                )
                .map_err(|error| error.to_string())?;
                install_production_broker_facilities(
                    &bootstrap,
                    &app,
                    Arc::clone(&mcp),
                    mcp_cancellations,
                    sampling_parents,
                )
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
            let credential_vault = CredentialServiceState::initialize(&c4os_home)
                .map_err(|error| std::io::Error::other(error.to_string()))?
                .vault();
            let application_resource_dir = app.path().resource_dir()?;
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
            let browser_profiles = browser::profile::BrowserProfileRegistry::load(
                home_layout.browser_profile_registry(),
            )
            .map_err(|error| std::io::Error::other(error.to_string()))?;
            let pending_browser_profile_clears = browser_profiles
                .snapshot()
                .profiles
                .into_iter()
                .filter_map(|profile| {
                    let browser::profile::PersistentProfileLifecycle::ClearPending {
                        operation_id,
                        ..
                    } = profile.lifecycle
                    else {
                        return None;
                    };
                    Some((profile.profile_id, operation_id))
                })
                .collect::<Vec<_>>();
            let now_ms = current_time_ms()?;
            let extensions = extension::service::ExtensionService::restore(
                Arc::clone(&database),
                &c4os_home,
                now_ms,
            )
            .map_err(|error| std::io::Error::other(error.to_string()))?;
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
            let runtime_resource_dir = application_resource_dir.clone();
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let hook_supervisor = Some(initialize_extension_hook_supervisor(
                &c4os_home,
                &runtime_resource_dir,
            )?);
            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            let hook_supervisor = None;
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
            let mcp_sampling_broker = Arc::new(
                mcp::production_sampling::ProductionMcpSamplingBroker::new(Arc::clone(&runtime)),
            );
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let mcp_sampling_approvals = mcp_sampling_broker.approvals();
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let mcp_sampling_parents = mcp_sampling_broker.parents();
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let mcp_transport_factory = Arc::new(
                mcp::transport::RmcpTransportFactory::with_sampling(mcp_sampling_broker),
            );
            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            let mcp_transport_factory = Arc::new(mcp::transport::RmcpTransportFactory::default());
            let mcp = Arc::new(tokio::sync::Mutex::new(
                ProductionMcpService::restore(
                    Arc::new(mcp::database::DatabaseMcpRepository::new(
                        Arc::clone(&database),
                        credential_vault.clone(),
                    )),
                    Arc::new(
                        mcp::authority::ProductionMcpAuthority::new(
                            c4os_home.clone(),
                            credential_vault.clone(),
                            Arc::clone(&active_workspace),
                            Arc::clone(&runtime),
                        )
                        .map_err(|error| std::io::Error::other(error.to_string()))?,
                    ),
                    mcp_transport_factory,
                    now_ms,
                )
                .map_err(|error| std::io::Error::other(error.to_string()))?,
            ));
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let mcp_cancellations = ProductionMcpCancellationRegistry::default();
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            if let Ok(service) = mcp.try_lock() {
                mcp_cancellations.refresh_credential_bindings(&service.snapshot());
            }
            let mcp_credential_observer = credential_vault
                .as_ref()
                .map(|vault| {
                    let observer: Arc<dyn security::credentials::CredentialMutationObserver> =
                        Arc::new(ProductionMcpCredentialObserver {
                            service: Arc::downgrade(&mcp),
                            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                            cancellations: mcp_cancellations.clone(),
                        });
                    vault
                        .register_mutation_observer(Arc::clone(&observer))
                        .map_err(|error| std::io::Error::other(error.to_string()))?;
                    Ok::<_, std::io::Error>(observer)
                })
                .transpose()?;
            let artifact_operation = Arc::new(Mutex::new(()));
            let artifact = Arc::new(Mutex::new(ArtifactApplicationState::default()));
            let terminal = Arc::new(Mutex::new(TerminalSupervisor::new()));
            let browser_events = Arc::new(Mutex::new(
                browser::native::NativeBrowserEventQueue::default(),
            ));
            let terminal_reconciliation = start_terminal_reconciliation_driver(
                Arc::clone(&active_workspace),
                Arc::clone(&artifact_operation),
                Arc::clone(&artifact),
                Arc::clone(&terminal),
                Arc::clone(&runtime),
            )?;
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            let runtime_production = Arc::new(ManagedProductionRuntime::default());
            app.manage(AppCoreState {
                database,
                c4os_home: c4os_home.clone(),
                bundled_skill_root: application_resource_dir.join("skills"),
                mcp: Arc::clone(&mcp),
                _mcp_credential_observer: mcp_credential_observer,
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                mcp_cancellations: mcp_cancellations.clone(),
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                mcp_sampling_approvals: mcp_sampling_approvals.clone(),
                extensions: Mutex::new(extensions),
                hook_supervisor: Mutex::new(hook_supervisor),
                configuration: Mutex::new(configuration),
                active_workspace: Arc::clone(&active_workspace),
                conversation_operation: Mutex::new(()),
                artifact_operation,
                conversation: Mutex::new(conversation),
                artifact,
                terminal,
                browser_profiles: Mutex::new(browser_profiles),
                browser_events: Arc::clone(&browser_events),
                _terminal_reconciliation: terminal_reconciliation,
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                runtime_production: Arc::clone(&runtime_production),
                runtime: Arc::clone(&runtime),
                platform,
                platform_snapshot,
                picker_grants: Mutex::new(PickerGrantRegistry::default()),
                conversation_drop: Mutex::new(NativeConversationDropState::default()),
                conversation_branch: Mutex::new(NativeConversationBranchState::default()),
            });
            for (profile_id, operation_id) in pending_browser_profile_clears {
                browser::native::dispatch_clear_profile(
                    app.handle(),
                    Arc::clone(&browser_events),
                    profile_id,
                    operation_id,
                )
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            }
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            start_runtime_production_initialization(
                app.handle().clone(),
                runtime_resource_dir,
                c4os_home,
                credential_vault,
                active_workspace,
                runtime,
                mcp,
                mcp_cancellations,
                mcp_sampling_parents,
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
            extension_snapshot,
            extension_add_marketplace,
            extension_refresh_catalogs,
            extension_install_disabled,
            extension_enable,
            extension_disable,
            extension_stage_update,
            extension_activate_update,
            extension_rollback,
            extension_revoke,
            extension_revoke_key,
            extension_uninstall,
            extension_set_skill_enabled,
            extension_select_skill,
            extension_customize_skill,
            extension_review_hook,
            extension_load_skill,
            extension_publisher_link,
            mcp_snapshot,
            mcp_save_server,
            mcp_request_trust,
            mcp_answer_trust,
            mcp_test_server,
            mcp_enable_server,
            mcp_recover_server,
            mcp_disable_server,
            mcp_revoke_server,
            mcp_delete_server,
            foundation_snapshot,
            workspace_start_snapshot,
            conversation_snapshot,
            artifact_snapshot,
            artifact_run_terminal,
            artifact_terminal_stdin,
            artifact_terminal_resize,
            artifact_terminal_stop,
            artifact_terminal_ack_output,
            artifact_open_browser,
            artifact_navigate_browser,
            artifact_clear_browser_data,
            artifact_mount_browser,
            artifact_resize_browser,
            artifact_focus_native_browser,
            artifact_detach_browser,
            artifact_open_file,
            artifact_open_folder,
            artifact_focus,
            artifact_close_focus,
            artifact_begin_file_edit,
            artifact_update_file_draft,
            artifact_discard_file_draft,
            artifact_reject_file_proposal,
            artifact_resolve_file_conflict,
            artifact_save_file,
            artifact_answer_approval,
            artifact_refresh_folder,
            artifact_navigate_folder,
            artifact_select_folder_entry,
            artifact_reply,
            artifact_expand_context,
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

#[cfg(test)]
mod artifact_file_projection_tests {
    use super::*;

    #[test]
    fn pending_reply_proposal_is_the_exact_approval_candidate_over_a_retained_draft() {
        let live = artifact::FileLiveVersion::new_with_target_version(
            1,
            format!("sha256:{}", "a".repeat(64)),
            sha256_bytes(b"live"),
            4,
            10,
        )
        .unwrap();
        let mut file = artifact::FileArtifactState::new(
            artifact::FileResourceReference::new("notes.txt", "notes.txt").unwrap(),
            "notes.txt",
            "text/plain",
            "live",
            live.clone(),
        )
        .unwrap();
        file.begin_draft(11).unwrap();
        file.update_draft("retained user draft", 12).unwrap();
        file.install_proposal(artifact::FileProposal {
            proposal_id: "proposal-reply".into(),
            base_live_version: live,
            proposed_content: "agent proposal".into(),
            unified_diff: "-live\n+agent proposal\n".into(),
            status: artifact::FileProposalStatus::Pending,
            created_at_ms: 13,
        })
        .unwrap();

        assert_eq!(file_pending_content(&file), "agent proposal");
        let ArtifactFileStateSnapshot::Approval {
            proposed_content,
            proposal_diff,
            ..
        } = file_protocol_state(&file, Some("approval-reply"))
        else {
            panic!("the pending reply proposal must own approval presentation");
        };
        assert_eq!(proposed_content, "agent proposal");
        assert_eq!(proposal_diff.as_deref(), Some("-live\n+agent proposal\n"));
    }

    #[test]
    fn selected_file_reply_context_is_captured_first_and_rejects_stale_authority() {
        let file = artifact::FileArtifactState::new(
            artifact::FileResourceReference::new("notes.txt", "notes.txt").unwrap(),
            "notes.txt",
            "text/plain",
            "alpha selected omega",
            artifact::FileLiveVersion::new_with_target_version(
                1,
                format!("sha256:{}", "a".repeat(64)),
                sha256_bytes(b"alpha selected omega"),
                20,
                10,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = artifact::ArtifactRecord {
            schema_version: artifact::ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-file-selection".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: artifact::ArtifactProviderDescriptor::file(),
            source: artifact::ArtifactSource::DirectOperation {
                operation_id: "operation-1".into(),
            },
            record_revision: 1,
            lifecycle: artifact::ArtifactLifecycle::Ready,
            state: artifact::ArtifactState::File(Box::new(file)),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
        };
        let correlation = protocol::CorrelationId::new("artifact-selection-test").unwrap();
        let durable = durable_artifact_reply_capture(
            &record,
            Some("selected".into()),
            None,
            correlation.clone(),
        )
        .unwrap();
        let context = capture_artifact_record_context(
            &record,
            durable.selected_text.as_deref(),
            None,
            1_024,
            11,
            correlation.clone(),
        )
        .unwrap();
        let first = artifact_context_segments(&context.payload).first().unwrap();
        assert_eq!(first.priority, artifact::ContextPriority::Selection);
        assert_eq!(first.text, "selected");

        record.record_revision = 2;
        let stale =
            require_current_artifact_reply_capture(&record, &durable, correlation).unwrap_err();
        assert_eq!(stale.code, ProtocolErrorCode::Conflict);
    }

    #[test]
    fn browser_reply_uses_the_immutable_native_page_capture_in_priority_order() {
        let target = artifact::BrowserNavigationTarget::parse("https://example.com/docs").unwrap();
        let mut browser = artifact::BrowserArtifactState::new_queued(
            artifact::BrowserEnvironmentReference::chat("workspace-1", "project-1", "session-1", 1)
                .unwrap(),
            &target,
            7,
            10,
        )
        .unwrap();
        browser
            .start_navigation(
                artifact::BrowserControllerEventMeta::new(7, 1, 11).unwrap(),
                &target,
                artifact::BrowserNavigationKind::Initial,
            )
            .unwrap();
        browser
            .mark_ready(
                artifact::BrowserControllerEventMeta::new(7, 2, 12).unwrap(),
                &target,
                artifact::BrowserNavigationKind::Initial,
                Some("Example docs"),
            )
            .unwrap();
        let record = artifact::ArtifactRecord {
            schema_version: artifact::ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-browser-context".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: artifact::ArtifactProviderDescriptor::browser(),
            source: artifact::ArtifactSource::DirectOperation {
                operation_id: "operation-browser-context".into(),
            },
            record_revision: 1,
            lifecycle: artifact::ArtifactLifecycle::Ready,
            state: artifact::ArtifactState::Browser(Box::new(browser)),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 12,
        };
        let correlation = protocol::CorrelationId::new("browser-context-test").unwrap();
        let mut durable =
            durable_artifact_reply_capture(&record, None, None, correlation.clone()).unwrap();
        durable.browser_page_context = Some(DurableBrowserPageContext {
            selected_text: Some("selected page text".into()),
            visible_text: Some("visible page text".into()),
            extracted_content: Some("bounded extracted page text".into()),
        });
        require_current_artifact_reply_capture(&record, &durable, correlation.clone()).unwrap();
        let context = capture_artifact_record_context_with_browser(
            &record,
            None,
            None,
            durable.browser_page_context.as_ref(),
            4_096,
            13,
            correlation,
        )
        .unwrap();
        let segments = artifact_context_segments(&context.payload);
        assert_eq!(segments[0].source, "selected-text");
        assert_eq!(segments[0].text, "selected page text");
        assert_eq!(segments[1].source, "visible-text");
        assert_eq!(segments[2].source, "extracted-content");
    }
}

#[cfg(test)]
mod artifact_terminal_projection_tests {
    use super::*;

    fn queued_terminal() -> artifact::TerminalArtifactState {
        artifact::TerminalArtifactState::new_queued(
            artifact::TerminalCommandIdentity {
                terminal_session_id: "terminal-session-test".into(),
                command_id: "terminal-command-test".into(),
                command_sequence: 1,
            },
            "printf safe",
            "/project",
            artifact::TerminalDimensions::new(80, 24).unwrap(),
            artifact::TerminalProcessProvenance {
                shell_path: "/bin/zsh".into(),
                environment_id: "desktop".into(),
                environment_generation: 1,
                process_generation: 1,
                shell_process_id: None,
                foreground_process_group_id: None,
            },
            10,
        )
        .unwrap()
    }

    fn running_terminal(output: &[u8]) -> artifact::TerminalArtifactState {
        let mut terminal = queued_terminal();
        terminal.mark_running(42, Some(43), true, 11).unwrap();
        terminal.append_output(output, 12).unwrap();
        terminal
    }

    fn terminal_record(terminal: artifact::TerminalArtifactState) -> artifact::ArtifactRecord {
        artifact::ArtifactRecord {
            schema_version: artifact::ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-terminal-test".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: artifact::ArtifactProviderDescriptor::terminal(),
            source: artifact::ArtifactSource::DirectOperation {
                operation_id: "terminal-operation-test".into(),
            },
            record_revision: 1,
            lifecycle: artifact::ArtifactLifecycle::Ready,
            state: artifact::ArtifactState::Terminal(Box::new(terminal)),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 12,
        }
    }

    #[test]
    fn inline_credentials_are_rejected_but_environment_references_remain_allowed() {
        assert!(terminal_command_contains_secret_material(
            "curl -H 'Authorization: Bearer abc' https://example.test"
        ));
        assert!(terminal_command_contains_secret_material(
            "AWS_SECRET_ACCESS_KEY=plaintext deploy"
        ));
        assert!(terminal_command_contains_secret_material(
            "psql postgres://user:pass@example.test/db"
        ));
        assert!(!terminal_command_contains_secret_material(
            "curl -H \"Authorization: Bearer $TOKEN\" https://example.test"
        ));
        assert!(!terminal_command_contains_secret_material("echo password"));
        assert!(terminal_command_references_secret_environment(
            "printf '%s' \"${API_TOKEN}\""
        ));
        assert!(!terminal_command_references_secret_environment(
            "printf '%s' \"$ORDINARY_VALUE\""
        ));
    }

    #[test]
    fn queued_terminal_recovery_does_not_requeue_a_run_already_dispatched_to_the_live_shell() {
        let terminal = queued_terminal();
        let mut live = execution::terminal::TerminalLiveSessionSnapshot {
            key: TerminalSessionKey::new("workspace-1", "session-1").unwrap(),
            terminal_session_id: terminal.identity.terminal_session_id.clone(),
            lifecycle: execution::terminal::TerminalLifecycle::Live,
            process_generation: terminal.process.process_generation,
            process_id: Some(42),
            foreground_process_group_id: Some(43),
            active_command: Some(
                SupervisedTerminalCommandIdentity::new(
                    terminal.identity.terminal_session_id.clone(),
                    terminal.identity.command_id.clone(),
                    terminal.identity.command_sequence,
                )
                .unwrap(),
            ),
            next_command_sequence: terminal.identity.command_sequence + 1,
            environment: execution::environment::ExecutionEnvironmentIdentity::new(
                execution::environment::ExecutionEnvironmentKind::Local,
                terminal.process.environment_id.clone(),
                terminal.process.environment_generation,
            )
            .unwrap(),
            dimensions: PtyDimensions::new(80, 24).unwrap(),
            shell_path: PathBuf::from("/bin/zsh"),
            working_directory: PathBuf::from("/project"),
            pending_event_count: 0,
            pending_output_bytes: 0,
            output_bytes_dropped: 0,
        };

        assert!(terminal_run_was_dispatched(&terminal, &live));

        live.active_command = Some(
            SupervisedTerminalCommandIdentity::new(
                terminal.identity.terminal_session_id.clone(),
                "another-command",
                terminal.identity.command_sequence,
            )
            .unwrap(),
        );
        assert!(!terminal_run_was_dispatched(&terminal, &live));

        // A prior idle command leaves the next sequence equal to the queued
        // command, so a genuinely interrupted approval still requeues.
        live.active_command = None;
        live.next_command_sequence = terminal.identity.command_sequence;
        assert!(!terminal_run_was_dispatched(&terminal, &live));

        // A short command may complete after the empty event drain but before
        // recovery inspects the supervisor. Its queued events still prove the
        // command was dispatched and must be reconciled on the next poll.
        live.next_command_sequence = terminal.identity.command_sequence + 1;
        live.pending_event_count = 1;
        assert!(terminal_run_was_dispatched(&terminal, &live));

        // A dormant post-restart session is recovery evidence, not dispatch
        // evidence, even if it retained the advanced sequence.
        live.lifecycle = execution::terminal::TerminalLifecycle::DormantAfterRecovery;
        assert!(!terminal_run_was_dispatched(&terminal, &live));

        live.lifecycle = execution::terminal::TerminalLifecycle::Live;
        live.process_generation = terminal.process.process_generation + 1;
        assert!(!terminal_run_was_dispatched(&terminal, &live));

        live.process_generation = terminal.process.process_generation;
        live.terminal_session_id = "another-terminal-session".into();
        assert!(!terminal_run_was_dispatched(&terminal, &live));
    }

    #[test]
    fn unsupervised_restart_recovery_requires_no_project_root() {
        let mut record = terminal_record(running_terminal(b"partial"));
        let expected_revision = transition_unsupervised_terminal_record(&mut record, 20).unwrap();
        assert_eq!(expected_revision, Some(1));
        assert_eq!(record.record_revision, 2);
        assert_eq!(record.updated_at_ms, 20);
        let artifact::ArtifactState::Terminal(terminal) = &record.state else {
            unreachable!()
        };
        assert!(matches!(
            &terminal.status,
            artifact::TerminalCommandStatus::Recovery { code, .. }
                if code == "process-restart"
        ));
        assert_eq!(
            record.history.last().map(|event| &event.kind),
            Some(&artifact::ArtifactHistoryKind::RecoveryChanged)
        );
    }

    #[test]
    fn queued_restart_command_drains_the_retained_supervisor_generation() {
        let mut terminal = queued_terminal();
        terminal.identity.command_sequence = 3;
        terminal.process.process_generation = 2;
        let retained = execution::terminal::TerminalLiveSessionSnapshot {
            key: TerminalSessionKey::new("workspace-1", "session-1").unwrap(),
            terminal_session_id: terminal.identity.terminal_session_id.clone(),
            lifecycle: execution::terminal::TerminalLifecycle::DormantAfterRestart,
            process_generation: 1,
            process_id: None,
            foreground_process_group_id: None,
            active_command: None,
            next_command_sequence: terminal.identity.command_sequence,
            environment: execution::environment::ExecutionEnvironmentIdentity::new(
                execution::environment::ExecutionEnvironmentKind::Local,
                terminal.process.environment_id.clone(),
                terminal.process.environment_generation,
            )
            .unwrap(),
            dimensions: PtyDimensions::new(80, 24).unwrap(),
            shell_path: PathBuf::from("/bin/zsh"),
            working_directory: PathBuf::from("/project/nested"),
            pending_event_count: 0,
            pending_output_bytes: 0,
            output_bytes_dropped: 0,
        };

        let binding = terminal_event_drain_binding(&terminal, &retained).unwrap();
        assert_eq!(
            binding.terminal_session_id,
            terminal.identity.terminal_session_id
        );
        assert_eq!(binding.process_generation, 1);
        assert_eq!(binding.working_directory, PathBuf::from("/project/nested"));
        assert!(terminal_run_can_be_requeued(&terminal, &retained));

        let mut mismatched = retained.clone();
        mismatched.terminal_session_id = "another-terminal-session".into();
        assert!(!terminal_run_can_be_requeued(&terminal, &mismatched));
        mismatched = retained.clone();
        mismatched.next_command_sequence += 1;
        assert!(!terminal_run_can_be_requeued(&terminal, &mismatched));
        mismatched = retained;
        mismatched.process_generation += 1;
        assert!(!terminal_run_can_be_requeued(&terminal, &mismatched));
    }

    #[test]
    fn durable_output_redacts_complete_secret_lines_across_chunk_boundaries() {
        let mut state = ArtifactApplicationState::default();
        let mut durable = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-test",
            b"AWS_SECRET_ACCESS_",
            false,
        );
        assert_eq!(durable, b"AWS_");
        durable.extend(retain_redacted_terminal_output(
            &mut state,
            "terminal-command-test",
            b"KEY=plaintext\nvisible\n",
            false,
        ));
        assert_eq!(
            durable,
            b"AWS_[sensitive Terminal output redacted]\nvisible\n"
        );
        assert!(!String::from_utf8_lossy(&durable).contains("plaintext"));
    }

    #[test]
    fn durable_output_streams_safe_prefix_before_newline() {
        let mut state = ArtifactApplicationState::default();
        let first = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-streaming",
            b"tick",
            false,
        );
        assert_eq!(first, b"tick");
        assert!(state.terminal_output_lines.is_empty());

        let final_bytes =
            retain_redacted_terminal_output(&mut state, "terminal-command-streaming", b"", true);
        assert!(final_bytes.is_empty());
    }

    #[test]
    fn benign_trigger_like_output_streams_after_finite_lookahead() {
        let mut state = ArtifactApplicationState::default();
        let mut output = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-benign-triggers",
            b"tokenized passwordless authorizationless",
            false,
        );
        assert_eq!(output, b"tokenized passwordless authorizationles");
        output.extend(retain_redacted_terminal_output(
            &mut state,
            "terminal-command-benign-triggers",
            b"",
            true,
        ));
        assert_eq!(output, b"tokenized passwordless authorizationless");
        assert!(state.terminal_output_lines.is_empty());
    }

    #[test]
    fn split_secret_trigger_and_value_never_enter_durable_output() {
        let mut state = ArtifactApplicationState::default();
        let mut durable = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-split-secret",
            b"ordinary API_TO",
            false,
        );
        durable.extend(retain_redacted_terminal_output(
            &mut state,
            "terminal-command-split-secret",
            b"KEN=super-",
            false,
        ));
        durable.extend(retain_redacted_terminal_output(
            &mut state,
            "terminal-command-split-secret",
            b"secret\n",
            false,
        ));
        assert_eq!(durable, b"ordinary [sensitive Terminal output redacted]\n");
        assert!(!String::from_utf8_lossy(&durable).contains("super-"));
        assert!(!String::from_utf8_lossy(&durable).contains("secret\n"));
    }

    #[test]
    fn failed_secret_transition_restores_redactor_state_before_retry() {
        let mut state = ArtifactApplicationState::default();
        assert!(
            retain_redacted_terminal_output(
                &mut state,
                "terminal-command-secret-retry",
                b"TOK",
                false,
            )
            .is_empty()
        );
        let checkpoint =
            terminal_output_redaction_checkpoint(&state, "terminal-command-secret-retry");
        let first_attempt = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-secret-retry",
            b"EN=plaintext\n",
            false,
        );
        assert_eq!(first_attempt, b"[sensitive Terminal output redacted]\n");

        restore_terminal_output_redaction(&mut state, "terminal-command-secret-retry", checkpoint);
        let retry = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-secret-retry",
            b"EN=plaintext\n",
            false,
        );
        assert_eq!(retry, b"[sensitive Terminal output redacted]\n");
        assert!(!String::from_utf8_lossy(&retry).contains("plaintext"));
    }

    #[test]
    fn carriage_return_progress_streams_as_record_boundaries() {
        let mut state = ArtifactApplicationState::default();
        let first = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-progress",
            b"step 1\r",
            false,
        );
        let second = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-progress",
            b"step 2\r",
            false,
        );
        assert_eq!(first, b"step 1\r");
        assert_eq!(second, b"step 2\r");

        let cr =
            retain_redacted_terminal_output(&mut state, "terminal-command-crlf", b"line\r", false);
        let lf = retain_redacted_terminal_output(&mut state, "terminal-command-crlf", b"\n", false);
        assert_eq!([cr, lf].concat(), b"line\r\n");
    }

    #[test]
    fn carriage_return_secret_record_preserves_only_marker_and_boundary() {
        let mut state = ArtifactApplicationState::default();
        let mut durable = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-secret-progress",
            b"TOK",
            false,
        );
        durable.extend(retain_redacted_terminal_output(
            &mut state,
            "terminal-command-secret-progress",
            b"EN=hunter2\r",
            false,
        ));
        assert_eq!(durable, b"[sensitive Terminal output redacted]\r");
        assert!(!String::from_utf8_lossy(&durable).contains("hunter2"));
    }

    #[test]
    fn suspicious_unterminated_output_is_memory_bounded_and_fully_redacted() {
        let mut state = ArtifactApplicationState::default();
        let mut oversized = b"TOKEN=".to_vec();
        oversized.extend(vec![b' '; artifact::MAX_TERMINAL_OUTPUT_BYTES + 1]);
        assert!(
            retain_redacted_terminal_output(
                &mut state,
                "terminal-command-oversized",
                &oversized,
                false,
            )
            .is_empty()
        );
        assert!(
            !state
                .terminal_output_lines
                .contains_key("terminal-command-oversized")
        );
        assert!(
            state
                .terminal_redacted_output_lines
                .contains("terminal-command-oversized")
        );
        let output = retain_redacted_terminal_output(
            &mut state,
            "terminal-command-oversized",
            b"still-hidden\nvisible\n",
            false,
        );
        assert_eq!(
            output,
            b"[oversized Terminal output line redacted]\nvisible\n"
        );
        assert!(state.terminal_redacted_output_lines.is_empty());
    }

    #[test]
    fn control_sequence_output_cannot_be_renderer_selected_for_reply() {
        let record = terminal_record(running_terminal(b"\x1b[2Jhidden"));
        let error = durable_artifact_reply_capture(
            &record,
            Some("hidden".into()),
            None,
            protocol::CorrelationId::new("terminal-selection-test").unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.code, ProtocolErrorCode::Conflict);
    }

    #[test]
    fn secret_environment_references_exclude_output_from_reply_context() {
        let mut terminal = running_terminal(b"unlabelled-secret-value");
        terminal.command = "printf '%s' \"$API_TOKEN\"".into();
        let record = terminal_record(terminal);
        let correlation = protocol::CorrelationId::new("terminal-secret-context-test").unwrap();
        let error = durable_artifact_reply_capture(
            &record,
            Some("unlabelled-secret-value".into()),
            None,
            correlation.clone(),
        )
        .unwrap_err();
        assert_eq!(error.code, ProtocolErrorCode::Conflict);

        let snapshot =
            capture_artifact_record_context(&record, None, None, 1_024, 20, correlation).unwrap();
        let artifact::ContextPayload::Terminal { segments, .. } = snapshot.payload else {
            unreachable!()
        };
        assert_eq!(
            segments
                .iter()
                .map(|segment| segment.source.as_str())
                .collect::<Vec<_>>(),
            vec!["terminal-metadata"]
        );
        assert!(
            segments
                .iter()
                .all(|segment| !segment.text.contains("unlabelled-secret-value"))
        );
    }

    #[test]
    fn only_pristine_empty_terminal_output_can_skip_a_native_ack_cursor() {
        let pristine = queued_terminal();
        assert!(!terminal_output_requires_ack_cursor(&pristine));

        let mut sequenced = pristine.clone();
        sequenced.output.sequence = 2;
        assert!(terminal_output_requires_ack_cursor(&sequenced));

        let mut dropped = pristine.clone();
        dropped.output.dropped_bytes = 1;
        assert!(terminal_output_requires_ack_cursor(&dropped));

        let with_output = running_terminal(b"visible output");
        assert!(terminal_output_requires_ack_cursor(&with_output));
    }

    #[test]
    fn durable_interrupt_marker_is_appended_exactly_once() {
        let mut terminal = running_terminal(b"visible output\n");
        let before = terminal.output.sequence;

        append_terminal_interrupt_marker(&mut terminal, 13).unwrap();
        assert_eq!(terminal.output.sequence, before + 1);
        assert!(
            terminal
                .output
                .retained_bytes()
                .unwrap()
                .ends_with(b"^C\r\n")
        );

        append_terminal_interrupt_marker(&mut terminal, 14).unwrap();
        assert_eq!(terminal.output.sequence, before + 1);
    }
}
