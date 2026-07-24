//! Rust-owned local-development update orchestration and export-safe diagnostics.
//!
//! This module deliberately does not install or replace application/runtime
//! binaries. Application and native-runtime versions are reconciled from
//! compiled Rust authority at startup; Plugin activation is composed with the
//! existing ExtensionService immutable-store/selector authority by `lib.rs`.
//! The durable document records staged intent, interrupted activation journals,
//! last-known-good metadata, revocation, and bounded redacted diagnostics.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ts_rs::TS;

use crate::core::database::{
    DatabaseActor, DatabaseError, DiagnosticRecord, MAX_READ_RECORDS, RuntimeStateDocumentRecord,
};

pub const UPDATE_COORDINATOR_SCHEMA_VERSION: u16 = 1;
pub const MAX_UPDATE_COMPONENTS: usize = 128;
pub const MAX_UPDATE_CANDIDATES: usize = 256;
pub const MAX_UPDATE_PENDING_OPERATIONS: usize = 64;
pub const MAX_UPDATE_RECOVERY_NOTICES: usize = 64;
pub const MAX_DIAGNOSTIC_RECORDS: usize = MAX_READ_RECORDS;

const DOCUMENT_KIND: &str = "update-coordinator";
const DOCUMENT_ID: &str = "local-development";
const MAX_IDENTIFIER_BYTES: usize = 160;
const MAX_SUMMARY_BYTES: usize = 512;
const MAX_UPDATE_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum UpdateChannel {
    Application,
    Runtime,
    Plugin,
}

impl UpdateChannel {
    pub fn configuration_key(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Runtime => "runtimes",
            Self::Plugin => "plugins",
        }
    }

    pub(crate) fn diagnostic_category(self) -> DiagnosticCategory {
        match self {
            Self::Application => DiagnosticCategory::Update,
            Self::Runtime => DiagnosticCategory::Runtime,
            Self::Plugin => DiagnosticCategory::Plugin,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum UpdateLifecycleState {
    Current,
    Staged,
    Activating,
    Activated,
    RolledBack,
    Revoked,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum UpdateOperationState {
    Activating,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum UpdateRecoveryAction {
    RetryStage,
    ActivateStaged,
    RollbackLastKnownGood,
    RecoverInterrupted,
    RemoveRevocation,
    ReviewPluginUpdate,
    RebuildApplication,
    WaitForActiveRuns,
    RebuildRuntime,
    RebuildLastKnownGood,
    ReviewRuntimeCrashLoop,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    Update,
    Recovery,
    Configuration,
    Runtime,
    Plugin,
    Security,
    Persistence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateComponentSnapshot {
    pub channel: UpdateChannel,
    pub component_id: String,
    pub state: UpdateLifecycleState,
    pub current_version: String,
    pub candidate_version: Option<String>,
    pub last_known_good_version: Option<String>,
    pub staged_artifact_sha256: Option<String>,
    pub revoked: bool,
    pub recovery_action: Option<UpdateRecoveryAction>,
    pub failure_code: Option<String>,
    pub updated_at_ms: u64,
    #[serde(skip)]
    #[ts(skip)]
    compatibility_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdatePendingOperation {
    pub operation_id: String,
    pub correlation_id: String,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub from_version: String,
    pub to_version: String,
    pub state: UpdateOperationState,
    pub started_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRecoveryNotice {
    pub recovery_id: String,
    pub correlation_id: String,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub summary: String,
    pub action: UpdateRecoveryAction,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCandidateSnapshot {
    pub candidate_id: String,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub version: String,
    pub artifact_sha256: String,
    pub compatibility_sha256: String,
    pub discovered_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateDiagnosticRecord {
    pub diagnostic_id: String,
    pub correlation_id: String,
    pub category: DiagnosticCategory,
    pub severity: DiagnosticSeverity,
    pub component_boundary: String,
    pub message: String,
    pub recovery_action: Option<UpdateRecoveryAction>,
    pub created_at_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationalDiagnosticSignal {
    ConfigurationRecovery,
    RuntimeCrashLoop,
    RuntimeFailure,
    RuntimeDegraded,
    ProviderFailure,
    ExtensionFailure,
    ExtensionRevoked,
    ExtensionRollback,
    McpFailure,
    McpRevoked,
    McpRestarting,
    WorkspaceRecovery,
    BrowserClearPending,
    TerminalDormantRecovery,
    CredentialFallback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationalDiagnosticObservation {
    pub state_key: String,
    pub category: DiagnosticCategory,
    pub severity: DiagnosticSeverity,
    pub component_boundary: String,
    pub message: String,
    pub recovery_action: Option<UpdateRecoveryAction>,
}

impl OperationalDiagnosticObservation {
    #[doc(hidden)]
    pub fn from_signal(
        signal: OperationalDiagnosticSignal,
        authority_key: &str,
    ) -> Result<Self, UpdateError> {
        validate_identifier(authority_key)?;
        let (signal_key, category, severity, component_boundary, message, recovery_action) =
            match signal {
                OperationalDiagnosticSignal::ConfigurationRecovery => (
                    "configuration-recovery",
                    DiagnosticCategory::Configuration,
                    DiagnosticSeverity::Warning,
                    "configuration",
                    "Configuration is retaining last-known-good authority after an external failure",
                    None,
                ),
                OperationalDiagnosticSignal::RuntimeCrashLoop => (
                    "runtime-crash-loop",
                    DiagnosticCategory::Runtime,
                    DiagnosticSeverity::Error,
                    "runtime",
                    "Runtime crash-loop recovery review is required",
                    Some(UpdateRecoveryAction::ReviewRuntimeCrashLoop),
                ),
                OperationalDiagnosticSignal::RuntimeFailure => (
                    "runtime-failure",
                    DiagnosticCategory::Runtime,
                    DiagnosticSeverity::Error,
                    "runtime",
                    "Runtime health authority reports an unavailable runtime",
                    Some(UpdateRecoveryAction::RebuildRuntime),
                ),
                OperationalDiagnosticSignal::RuntimeDegraded => (
                    "runtime-degraded",
                    DiagnosticCategory::Runtime,
                    DiagnosticSeverity::Warning,
                    "runtime",
                    "Runtime health authority reports degraded operation",
                    None,
                ),
                OperationalDiagnosticSignal::ProviderFailure => (
                    "provider-failure",
                    DiagnosticCategory::Runtime,
                    DiagnosticSeverity::Error,
                    "provider",
                    "Provider connectivity authority reports a failed check",
                    None,
                ),
                OperationalDiagnosticSignal::ExtensionFailure => (
                    "extension-failure",
                    DiagnosticCategory::Plugin,
                    DiagnosticSeverity::Error,
                    "extension",
                    "Extension authority reports a failed package",
                    Some(UpdateRecoveryAction::ReviewPluginUpdate),
                ),
                OperationalDiagnosticSignal::ExtensionRevoked => (
                    "extension-revoked",
                    DiagnosticCategory::Security,
                    DiagnosticSeverity::Warning,
                    "extension",
                    "Extension trust authority reports a revoked package",
                    None,
                ),
                OperationalDiagnosticSignal::ExtensionRollback => (
                    "extension-rollback",
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    "extension",
                    "Extension authority retained a rolled-back package version",
                    Some(UpdateRecoveryAction::ReviewPluginUpdate),
                ),
                OperationalDiagnosticSignal::McpFailure => (
                    "mcp-failure",
                    DiagnosticCategory::Runtime,
                    DiagnosticSeverity::Error,
                    "mcp",
                    "MCP lifecycle authority reports a failed server",
                    None,
                ),
                OperationalDiagnosticSignal::McpRevoked => (
                    "mcp-revoked",
                    DiagnosticCategory::Security,
                    DiagnosticSeverity::Warning,
                    "mcp",
                    "MCP trust authority reports a revoked server",
                    None,
                ),
                OperationalDiagnosticSignal::McpRestarting => (
                    "mcp-restarting",
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    "mcp",
                    "MCP lifecycle authority is applying bounded restart recovery",
                    None,
                ),
                OperationalDiagnosticSignal::WorkspaceRecovery => (
                    "workspace-recovery",
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    "workspace",
                    "Workspace authority retained a validated recovered working copy",
                    None,
                ),
                OperationalDiagnosticSignal::BrowserClearPending => (
                    "browser-clear-pending",
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    "browser",
                    "Browser profile data clear remains pending native completion",
                    None,
                ),
                OperationalDiagnosticSignal::TerminalDormantRecovery => (
                    "terminal-dormant-recovery",
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    "terminal",
                    "Terminal authority retained a dormant recovery session",
                    None,
                ),
                OperationalDiagnosticSignal::CredentialFallback => (
                    "credential-fallback",
                    DiagnosticCategory::Security,
                    DiagnosticSeverity::Warning,
                    "credential",
                    "Credential protection requires an explicit fallback decision",
                    None,
                ),
            };
        Ok(Self {
            state_key: format!("{signal_key}:{authority_key}"),
            category,
            severity,
            component_boundary: component_boundary.into(),
            message: message.into(),
            recovery_action,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateCoordinatorSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub channels: Vec<UpdateComponentSnapshot>,
    pub candidates: Vec<UpdateCandidateSnapshot>,
    pub pending_operations: Vec<UpdatePendingOperation>,
    pub recovery_notices: Vec<UpdateRecoveryNotice>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticsSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub records: Vec<UpdateDiagnosticRecord>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticsExportSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub export_id: String,
    pub created_at_ms: u64,
    pub sha256: String,
    pub records: Vec<UpdateDiagnosticRecord>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalUpdateStageInput {
    pub expected_generation: u64,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub candidate_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateActionInput {
    pub expected_generation: u64,
    pub channel: UpdateChannel,
    pub component_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRevocationInput {
    pub expected_generation: u64,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRecoveryInput {
    pub expected_generation: u64,
    pub channel: UpdateChannel,
    pub component_id: String,
    pub recovery_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticsExportInput {
    pub expected_generation: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateCoordinatorDocument {
    schema_version: u16,
    generation: u64,
    channels: Vec<UpdateComponentSnapshot>,
    candidates: Vec<UpdateCandidateSnapshot>,
    pending_operations: Vec<UpdatePendingOperation>,
    recovery_notices: Vec<UpdateRecoveryNotice>,
    diagnostics: Vec<UpdateDiagnosticRecord>,
    next_sequence: u64,
    #[serde(default, skip_serializing_if = "is_false")]
    cleanup_residue: bool,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("update input is invalid")]
    InvalidInput,
    #[error("update state is invalid")]
    InvalidState,
    #[error("update component was not found")]
    NotFound,
    #[error("update generation is stale")]
    Conflict,
    #[error("update component is revoked")]
    Revoked,
    #[error("update operation exceeded a bounded limit")]
    BoundExceeded,
    #[error("update persistence failed")]
    Persistence,
    #[error("update fault was injected at {0:?}")]
    InjectedFault(UpdateFaultPoint),
}

impl From<DatabaseError> for UpdateError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::Conflict(_) => Self::Conflict,
            _ => Self::Persistence,
        }
    }
}

pub struct UpdateCoordinator {
    database: Arc<DatabaseActor>,
    document: UpdateCoordinatorDocument,
    store_authority_root: PathBuf,
    store_root: PathBuf,
    diagnostic_retention_days: u16,
    diagnostic_redaction_patterns: Vec<String>,
    authoritative_compatibility_bindings: BTreeMap<(UpdateChannel, String), String>,
    injected_faults: BTreeSet<UpdateFaultPoint>,
}

struct TemporaryCandidateFile {
    path: PathBuf,
    committed: bool,
}

impl Drop for TemporaryCandidateFile {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UpdateFaultPoint {
    JournalPrepare,
    CandidateCopy,
    Verification,
    Compatibility,
    SelectorPublication,
    MigrationSnapshot,
    Health,
    DurableCompletion,
    RollbackPublication,
    Cleanup,
}

impl UpdateCoordinator {
    pub fn restore(database: Arc<DatabaseActor>, now_ms: u64) -> Result<Self, UpdateError> {
        let now_ms = now_ms.max(1);
        let c4os_home = database
            .descriptor()
            .path
            .parent()
            .and_then(Path::parent)
            .ok_or(UpdateError::InvalidState)?;
        let (store_authority_root, store_root) = prepare_store_root(c4os_home)?;
        let record = database.runtime_state_document(DOCUMENT_KIND, DOCUMENT_ID)?;
        let mut coordinator = match record {
            Some(record) => {
                let mut document: UpdateCoordinatorDocument =
                    serde_json::from_str(&record.canonical_document)
                        .map_err(|_| UpdateError::InvalidState)?;
                rehydrate_staged_compatibility(&mut document)?;
                if document.generation != record.generation
                    || serde_json::to_string(&document).map_err(|_| UpdateError::InvalidState)?
                        != record.canonical_document
                {
                    return Err(UpdateError::InvalidState);
                }
                let correlations_normalized = normalize_persisted_correlations(&mut document);
                if correlations_normalized {
                    document.generation = document
                        .generation
                        .checked_add(1)
                        .ok_or(UpdateError::BoundExceeded)?;
                }
                validate_document(&document)?;
                let coordinator = Self {
                    database,
                    document,
                    store_authority_root,
                    store_root,
                    diagnostic_retention_days: 30,
                    diagnostic_redaction_patterns: Vec::new(),
                    authoritative_compatibility_bindings: BTreeMap::new(),
                    injected_faults: BTreeSet::new(),
                };
                if correlations_normalized {
                    coordinator.persist_document(
                        &coordinator.document,
                        Some(record.generation),
                        now_ms,
                    )?;
                }
                coordinator
            }
            None => {
                let document = UpdateCoordinatorDocument {
                    schema_version: UPDATE_COORDINATOR_SCHEMA_VERSION,
                    generation: 1,
                    channels: Vec::new(),
                    candidates: Vec::new(),
                    pending_operations: Vec::new(),
                    recovery_notices: Vec::new(),
                    diagnostics: Vec::new(),
                    next_sequence: 1,
                    cleanup_residue: false,
                };
                let coordinator = Self {
                    database,
                    document,
                    store_authority_root,
                    store_root,
                    diagnostic_retention_days: 30,
                    diagnostic_redaction_patterns: Vec::new(),
                    authoritative_compatibility_bindings: BTreeMap::new(),
                    injected_faults: BTreeSet::new(),
                };
                coordinator.persist_document(&coordinator.document, None, now_ms)?;
                coordinator
            }
        };
        coordinator.recover_interrupted_on_startup(now_ms)?;
        Ok(coordinator)
    }

    pub fn snapshot(&self) -> UpdateCoordinatorSnapshot {
        let mut channels = self.document.channels.clone();
        channels.sort_by(|left, right| {
            (left.channel, left.component_id.as_str())
                .cmp(&(right.channel, right.component_id.as_str()))
        });
        let mut candidates = self.document.candidates.clone();
        candidates.sort_by(|left, right| {
            (left.channel, left.component_id.as_str())
                .cmp(&(right.channel, right.component_id.as_str()))
                .then_with(|| {
                    Version::parse(&right.version)
                        .ok()
                        .cmp(&Version::parse(&left.version).ok())
                })
                .then_with(|| left.candidate_id.cmp(&right.candidate_id))
        });
        UpdateCoordinatorSnapshot {
            schema_version: self.document.schema_version,
            generation: self.document.generation,
            channels,
            candidates,
            pending_operations: self.document.pending_operations.clone(),
            recovery_notices: self.document.recovery_notices.clone(),
        }
    }

    /// Applies app-config-owned preferences and atomically re-sanitizes the
    /// durable diagnostic projection. The editable source remains app
    /// `config.toml`, never this journal.
    pub fn apply_diagnostic_preferences(
        &mut self,
        retention_days: Option<u16>,
        redaction_patterns: &[String],
        now_ms: u64,
    ) -> Result<(), UpdateError> {
        let retention_days = retention_days.unwrap_or(30);
        if now_ms == 0
            || retention_days == 0
            || redaction_patterns.len() > 128
            || redaction_patterns.iter().any(|pattern| {
                pattern.is_empty()
                    || pattern.len() > MAX_SUMMARY_BYTES
                    || pattern.chars().any(char::is_control)
            })
        {
            return Err(UpdateError::InvalidInput);
        }
        if self.diagnostic_retention_days == retention_days
            && self.diagnostic_redaction_patterns == redaction_patterns
        {
            return Ok(());
        }
        let previous_retention_days = self.diagnostic_retention_days;
        let previous_redaction_patterns = std::mem::replace(
            &mut self.diagnostic_redaction_patterns,
            redaction_patterns.to_vec(),
        );
        self.diagnostic_retention_days = retention_days;
        let mut next = self.document.clone();
        let transition = (|| {
            sanitize_diagnostics(&mut next, &self.diagnostic_redaction_patterns);
            bounded_cleanup(&mut next, now_ms, self.diagnostic_retention_days);
            next.generation = next
                .generation
                .checked_add(1)
                .ok_or(UpdateError::BoundExceeded)?;
            validate_document(&next)?;
            self.persist_document(&next, Some(self.document.generation), now_ms)
        })();
        if let Err(error) = transition {
            self.diagnostic_retention_days = previous_retention_days;
            self.diagnostic_redaction_patterns = previous_redaction_patterns;
            return Err(error);
        }
        self.document = next;
        Ok(())
    }

    /// Registers the compiled compatibility authority used to stage one
    /// application or runtime candidate. Bindings are deliberately transient:
    /// restart restores fail closed until bootstrap registers current truth.
    #[doc(hidden)]
    pub fn register_authoritative_compatibility_binding(
        &mut self,
        channel: UpdateChannel,
        component_id: &str,
        rust_compatibility_binding: &[u8],
    ) -> Result<(), UpdateError> {
        if channel == UpdateChannel::Plugin
            || rust_compatibility_binding.is_empty()
            || rust_compatibility_binding.len() > 64 * 1024
        {
            return Err(UpdateError::InvalidInput);
        }
        validate_identifier(component_id)?;
        self.authoritative_compatibility_bindings.insert(
            (channel, component_id.to_owned()),
            sha256_bytes(rust_compatibility_binding),
        );
        Ok(())
    }

    /// Replaces only current Rust-owned operational records. Stable records
    /// retain their original correlation and timestamp, so repeated reads do
    /// not grow history or advance the durable generation.
    #[doc(hidden)]
    pub fn synchronize_operational_diagnostics(
        &mut self,
        observations: &[OperationalDiagnosticObservation],
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<DiagnosticsSnapshot, UpdateError> {
        validate_identifier(correlation_id)?;
        if now_ms == 0 || observations.len() > 128 {
            return Err(UpdateError::InvalidInput);
        }
        let existing = self
            .document
            .diagnostics
            .iter()
            .filter(|record| record.diagnostic_id.starts_with("operational-"))
            .map(|record| (record.diagnostic_id.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut desired = Vec::with_capacity(observations.len());
        let mut identities = BTreeSet::new();
        for observation in observations {
            validate_identifier(&observation.state_key)?;
            validate_identifier(&observation.component_boundary)?;
            validate_summary(&observation.message)?;
            let diagnostic_id = format!(
                "operational-{}",
                &sha256_bytes(observation.state_key.as_bytes()).trim_start_matches("sha256:")[..24]
            );
            if !identities.insert(diagnostic_id.clone()) {
                return Err(UpdateError::InvalidInput);
            }
            let current = UpdateDiagnosticRecord {
                diagnostic_id: diagnostic_id.clone(),
                correlation_id: persisted_correlation_id(correlation_id),
                category: observation.category,
                severity: observation.severity,
                component_boundary: observation.component_boundary.clone(),
                message: observation.message.clone(),
                recovery_action: observation.recovery_action,
                created_at_ms: now_ms,
            };
            desired.push(
                existing
                    .get(&diagnostic_id)
                    .filter(|record| {
                        record.category == current.category
                            && record.severity == current.severity
                            && record.component_boundary == current.component_boundary
                            && record.message == current.message
                            && record.recovery_action == current.recovery_action
                    })
                    .map(|record| (**record).clone())
                    .unwrap_or(current),
            );
        }
        desired.sort_by(|left, right| left.diagnostic_id.cmp(&right.diagnostic_id));
        let mut current = existing
            .into_values()
            .cloned()
            .collect::<Vec<UpdateDiagnosticRecord>>();
        current.sort_by(|left, right| left.diagnostic_id.cmp(&right.diagnostic_id));
        if desired == current {
            return Ok(self.diagnostics_snapshot());
        }
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            next.diagnostics
                .retain(|record| !record.diagnostic_id.starts_with("operational-"));
            next.diagnostics.extend(desired);
            Ok(())
        })?;
        Ok(self.diagnostics_snapshot())
    }

    /// Rust-only deterministic fault seam. No Tauri command exposes it.
    #[doc(hidden)]
    pub fn inject_fault_for_test(&mut self, point: UpdateFaultPoint) {
        self.injected_faults.insert(point);
    }

    /// Copies a Rust-discovered local candidate into an immutable
    /// content-addressed store and registers only computed verification data.
    /// Callers cannot provide a claimed artifact digest or candidate identity.
    #[doc(hidden)]
    pub fn discover_file_candidate(
        &mut self,
        channel: UpdateChannel,
        component_id: &str,
        version: &str,
        source: &Path,
        rust_compatibility_binding: &[u8],
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_identity(channel, component_id)?;
        validate_version(version)?;
        validate_identifier(correlation_id)?;
        if rust_compatibility_binding.is_empty() || rust_compatibility_binding.len() > 64 * 1024 {
            return Err(UpdateError::InvalidInput);
        }
        self.check_fault(UpdateFaultPoint::CandidateCopy)?;
        let source_metadata =
            fs::symlink_metadata(source).map_err(|_| UpdateError::InvalidInput)?;
        if !source_metadata.file_type().is_file()
            || source_metadata.len() == 0
            || source_metadata.len() > MAX_UPDATE_ARTIFACT_BYTES
        {
            return Err(UpdateError::InvalidInput);
        }
        let source = source
            .canonicalize()
            .map_err(|_| UpdateError::InvalidInput)?;
        self.check_fault(UpdateFaultPoint::Verification)?;
        let artifact_sha256 = sha256_file(&source)?;
        let digest_hex = artifact_sha256
            .strip_prefix("sha256:")
            .ok_or(UpdateError::InvalidState)?;
        let store_root = self.validated_store_root()?;
        let destination = store_root.join(digest_hex);
        match fs::symlink_metadata(&destination) {
            Ok(metadata) => {
                if !metadata.file_type().is_file() || sha256_file(&destination)? != artifact_sha256
                {
                    return Err(UpdateError::InvalidState);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let temporary = self
                    .validated_store_root()?
                    .join(format!(".candidate-{digest_hex}-{}", std::process::id()));
                let mut input = File::open(&source).map_err(|_| UpdateError::Persistence)?;
                let mut output = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&temporary)
                    .map_err(|_| UpdateError::Persistence)?;
                let mut temporary_guard = TemporaryCandidateFile {
                    path: temporary.clone(),
                    committed: false,
                };
                std::io::copy(&mut input, &mut output).map_err(|_| UpdateError::Persistence)?;
                output.sync_all().map_err(|_| UpdateError::Persistence)?;
                drop(output);
                if sha256_file(&temporary)? != artifact_sha256 {
                    return Err(UpdateError::InvalidState);
                }
                fs::rename(&temporary, &destination).map_err(|_| UpdateError::Persistence)?;
                temporary_guard.committed = true;
                File::open(&store_root)
                    .and_then(|directory| directory.sync_all())
                    .map_err(|_| UpdateError::Persistence)?;
            }
            Err(_) => return Err(UpdateError::Persistence),
        }
        let mut permissions = fs::metadata(&destination)
            .map_err(|_| UpdateError::Persistence)?
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&destination, permissions).map_err(|_| UpdateError::Persistence)?;
        File::open(&destination)
            .and_then(|file| file.sync_all())
            .map_err(|_| UpdateError::Persistence)?;
        self.check_fault(UpdateFaultPoint::Compatibility)?;
        let compatibility_sha256 = sha256_bytes(rust_compatibility_binding);
        let candidate_id = format!(
            "candidate-{}",
            &sha256_bytes(
                format!(
                    "{channel:?}:{component_id}:{version}:{artifact_sha256}:{compatibility_sha256}"
                )
                .as_bytes()
            )
            .trim_start_matches("sha256:")[..24]
        );
        self.register_discovered_candidate(
            UpdateCandidateSnapshot {
                candidate_id,
                channel,
                component_id: component_id.to_owned(),
                version: version.to_owned(),
                artifact_sha256,
                compatibility_sha256,
                discovered_at_ms: now_ms,
            },
            correlation_id,
            now_ms,
        )
    }

    pub fn diagnostics_snapshot(&self) -> DiagnosticsSnapshot {
        DiagnosticsSnapshot {
            schema_version: self.document.schema_version,
            generation: self.document.generation,
            records: self.document.diagnostics.clone(),
            truncated: self.document.diagnostics.len() == MAX_DIAGNOSTIC_RECORDS,
        }
    }

    #[doc(hidden)]
    pub fn ingest_diagnostic(
        &mut self,
        category: DiagnosticCategory,
        severity: DiagnosticSeverity,
        component_boundary: &str,
        message: &str,
        recovery_action: Option<UpdateRecoveryAction>,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<DiagnosticsSnapshot, UpdateError> {
        validate_identifier(component_boundary)?;
        validate_summary(message)?;
        validate_identifier(correlation_id)?;
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            record_diagnostic(
                next,
                correlation_id,
                category,
                severity,
                component_boundary,
                message,
                recovery_action,
                now_ms,
            )
        })?;
        Ok(self.diagnostics_snapshot())
    }

    pub fn diagnostics_export(
        &self,
        expected_generation: u64,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<DiagnosticsExportSnapshot, UpdateError> {
        self.require_generation(expected_generation)?;
        validate_identifier(correlation_id)?;
        let records = self.document.diagnostics.clone();
        let canonical = serde_json::to_vec(&records).map_err(|_| UpdateError::InvalidState)?;
        let sha256 = sha256_bytes(&canonical);
        let export_id = format!(
            "diagnostics-{}",
            &sha256.trim_start_matches("sha256:")[..24]
        );
        Ok(DiagnosticsExportSnapshot {
            schema_version: self.document.schema_version,
            generation: self.document.generation,
            export_id,
            created_at_ms: now_ms,
            sha256,
            records,
            truncated: self.document.diagnostics.len() == MAX_DIAGNOSTIC_RECORDS,
        })
    }

    /// Reconciles application/runtime truth supplied by compiled Rust code.
    /// A candidate becomes active only after a rebuilt process reports that
    /// exact version; a renderer request cannot claim binary activation.
    pub fn reconcile_authoritative_component(
        &mut self,
        channel: UpdateChannel,
        component_id: &str,
        authoritative_version: &str,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        if channel == UpdateChannel::Plugin {
            return Err(UpdateError::InvalidInput);
        }
        validate_identifier(component_id)?;
        validate_identifier(correlation_id)?;
        validate_version(authoritative_version)?;
        if self.document.channels.iter().any(|component| {
            component.channel == channel
                && component.component_id == component_id
                && component.current_version == authoritative_version
        }) {
            return Ok(self.snapshot());
        }
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            match component_mut(next, channel, component_id) {
                Ok(component) if component.current_version == authoritative_version => {}
                Ok(component)
                    if component.candidate_version.as_deref() == Some(authoritative_version) =>
                {
                    let prior = component.current_version.clone();
                    component.current_version = authoritative_version.to_owned();
                    component.last_known_good_version = Some(prior);
                    component.candidate_version = None;
                    component.staged_artifact_sha256 = None;
                    component.compatibility_sha256 = None;
                    component.state = UpdateLifecycleState::Activated;
                    component.failure_code = None;
                    component.recovery_action = None;
                    component.updated_at_ms = now_ms;
                    record_diagnostic(
                        next,
                        correlation_id,
                        channel.diagnostic_category(),
                        DiagnosticSeverity::Info,
                        component_id,
                        "Compiled component version reconciled after restart",
                        None,
                        now_ms,
                    )?;
                }
                Ok(component) => {
                    let prior = component.current_version.clone();
                    component.current_version = authoritative_version.to_owned();
                    component.last_known_good_version = Some(prior);
                    component.candidate_version = None;
                    component.staged_artifact_sha256 = None;
                    component.compatibility_sha256 = None;
                    component.state = if component.revoked {
                        UpdateLifecycleState::Revoked
                    } else {
                        UpdateLifecycleState::Current
                    };
                    component.failure_code = None;
                    component.recovery_action = None;
                    component.updated_at_ms = now_ms;
                }
                Err(UpdateError::NotFound) => {
                    if next.channels.len() >= MAX_UPDATE_COMPONENTS {
                        return Err(UpdateError::BoundExceeded);
                    }
                    next.channels.push(UpdateComponentSnapshot {
                        channel,
                        component_id: component_id.to_owned(),
                        state: UpdateLifecycleState::Current,
                        current_version: authoritative_version.to_owned(),
                        candidate_version: None,
                        last_known_good_version: None,
                        staged_artifact_sha256: None,
                        revoked: false,
                        recovery_action: None,
                        failure_code: None,
                        updated_at_ms: now_ms,
                        compatibility_sha256: None,
                    });
                }
                Err(error) => return Err(error),
            }
            Ok(())
        })
    }

    /// Mirrors ExtensionService truth without becoming a second Plugin
    /// authority. The immutable package store and selector remain authoritative.
    #[allow(clippy::too_many_arguments)]
    pub fn reconcile_plugin_component(
        &mut self,
        component_id: &str,
        current_version: &str,
        candidate_version: Option<&str>,
        staged_artifact_sha256: Option<&str>,
        last_known_good_version: Option<&str>,
        revoked: bool,
        failure_code: Option<&str>,
        authoritative_state: UpdateLifecycleState,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_identifier(component_id)?;
        validate_identifier(correlation_id)?;
        validate_version(current_version)?;
        if let Some(version) = candidate_version {
            validate_version(version)?;
        }
        if let Some(digest) = staged_artifact_sha256 {
            validate_digest(digest)?;
        }
        if let Some(version) = last_known_good_version {
            validate_version(version)?;
        }
        if let Some(code) = failure_code {
            validate_identifier(code)?;
        }
        if matches!(
            authoritative_state,
            UpdateLifecycleState::Activating | UpdateLifecycleState::Staged
        ) {
            return Err(UpdateError::InvalidInput);
        }
        let desired_state = if revoked {
            UpdateLifecycleState::Revoked
        } else if staged_artifact_sha256.is_some() {
            UpdateLifecycleState::Staged
        } else if authoritative_state != UpdateLifecycleState::Current {
            authoritative_state
        } else if failure_code.is_some() {
            UpdateLifecycleState::Failed
        } else {
            authoritative_state
        };
        if self.document.channels.iter().any(|component| {
            component.channel == UpdateChannel::Plugin
                && component.component_id == component_id
                && component.current_version == current_version
                && component.candidate_version.as_deref() == candidate_version
                && component.last_known_good_version.as_deref() == last_known_good_version
                && component.staged_artifact_sha256.as_deref() == staged_artifact_sha256
                && component.compatibility_sha256.as_deref() == staged_artifact_sha256
                && component.revoked == revoked
                && component.failure_code.as_deref() == failure_code
                && component.state == desired_state
        }) {
            return Ok(self.snapshot());
        }
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            let staged = !revoked && staged_artifact_sha256.is_some();
            match component_mut(next, UpdateChannel::Plugin, component_id) {
                Ok(component) => {
                    let changed_version = component.current_version != current_version;
                    if changed_version {
                        component.last_known_good_version = last_known_good_version
                            .map(str::to_owned)
                            .or_else(|| Some(component.current_version.clone()));
                    } else {
                        component.last_known_good_version =
                            last_known_good_version.map(str::to_owned);
                    }
                    component.current_version = current_version.to_owned();
                    component.candidate_version = candidate_version.map(str::to_owned);
                    component.staged_artifact_sha256 = staged_artifact_sha256.map(str::to_owned);
                    component.compatibility_sha256 = staged_artifact_sha256.map(str::to_owned);
                    component.revoked = revoked;
                    if revoked {
                        component.candidate_version = None;
                        component.staged_artifact_sha256 = None;
                        component.compatibility_sha256 = None;
                    }
                    component.failure_code = failure_code.map(str::to_owned);
                    component.recovery_action = if revoked {
                        None
                    } else if staged {
                        Some(UpdateRecoveryAction::ActivateStaged)
                    } else {
                        None
                    };
                    component.state = if revoked {
                        UpdateLifecycleState::Revoked
                    } else if staged {
                        UpdateLifecycleState::Staged
                    } else if authoritative_state != UpdateLifecycleState::Current {
                        authoritative_state
                    } else if changed_version
                        && authoritative_state == UpdateLifecycleState::Current
                    {
                        UpdateLifecycleState::Activated
                    } else if failure_code.is_some() {
                        UpdateLifecycleState::Failed
                    } else {
                        authoritative_state
                    };
                    component.updated_at_ms = now_ms;
                }
                Err(UpdateError::NotFound) => {
                    if next.channels.len() >= MAX_UPDATE_COMPONENTS {
                        return Err(UpdateError::BoundExceeded);
                    }
                    next.channels.push(UpdateComponentSnapshot {
                        channel: UpdateChannel::Plugin,
                        component_id: component_id.to_owned(),
                        state: if revoked {
                            UpdateLifecycleState::Revoked
                        } else if staged {
                            UpdateLifecycleState::Staged
                        } else if authoritative_state != UpdateLifecycleState::Current {
                            authoritative_state
                        } else if failure_code.is_some() {
                            UpdateLifecycleState::Failed
                        } else {
                            authoritative_state
                        },
                        current_version: current_version.to_owned(),
                        candidate_version: (!revoked)
                            .then(|| candidate_version.map(str::to_owned))
                            .flatten(),
                        last_known_good_version: last_known_good_version.map(str::to_owned),
                        staged_artifact_sha256: (!revoked)
                            .then(|| staged_artifact_sha256.map(str::to_owned))
                            .flatten(),
                        revoked,
                        recovery_action: if revoked {
                            None
                        } else if staged {
                            Some(UpdateRecoveryAction::ActivateStaged)
                        } else {
                            None
                        },
                        failure_code: failure_code.map(str::to_owned),
                        updated_at_ms: now_ms,
                        compatibility_sha256: (!revoked)
                            .then(|| staged_artifact_sha256.map(str::to_owned))
                            .flatten(),
                    });
                }
                Err(error) => return Err(error),
            }
            Ok(())
        })
    }

    pub fn stage(
        &mut self,
        input: &LocalUpdateStageInput,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_stage_input(input)?;
        validate_identifier(correlation_id)?;
        self.preflight_stage(input)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            let candidate = next
                .candidates
                .iter()
                .find(|candidate| {
                    candidate.candidate_id == input.candidate_id
                        && candidate.channel == input.channel
                        && candidate.component_id == input.component_id
                })
                .cloned()
                .ok_or(UpdateError::NotFound)?;
            let component = component_mut(next, input.channel, &input.component_id)?;
            if component.revoked {
                return Err(UpdateError::Revoked);
            }
            if Version::parse(&candidate.version).map_err(|_| UpdateError::InvalidState)?
                <= Version::parse(&component.current_version)
                    .map_err(|_| UpdateError::InvalidState)?
            {
                return Err(UpdateError::Conflict);
            }
            component.state = UpdateLifecycleState::Staged;
            component.candidate_version = Some(candidate.version);
            component.staged_artifact_sha256 = Some(candidate.artifact_sha256);
            component.compatibility_sha256 = Some(candidate.compatibility_sha256);
            component.failure_code = None;
            component.recovery_action = Some(UpdateRecoveryAction::ActivateStaged);
            component.updated_at_ms = now_ms;
            record_diagnostic(
                next,
                correlation_id,
                input.channel.diagnostic_category(),
                DiagnosticSeverity::Info,
                &input.component_id,
                "Update candidate staged after local verification",
                Some(UpdateRecoveryAction::ActivateStaged),
                now_ms,
            )
        })
    }

    pub(crate) fn preflight_stage(&self, input: &LocalUpdateStageInput) -> Result<(), UpdateError> {
        validate_stage_input(input)?;
        self.require_generation(input.expected_generation)?;
        let candidate = self
            .document
            .candidates
            .iter()
            .find(|candidate| {
                candidate.candidate_id == input.candidate_id
                    && candidate.channel == input.channel
                    && candidate.component_id == input.component_id
            })
            .ok_or(UpdateError::NotFound)?;
        let component = self
            .document
            .channels
            .iter()
            .find(|component| {
                component.channel == input.channel && component.component_id == input.component_id
            })
            .ok_or(UpdateError::NotFound)?;
        if component.revoked {
            return Err(UpdateError::Revoked);
        }
        if Version::parse(&candidate.version).map_err(|_| UpdateError::InvalidState)?
            <= Version::parse(&component.current_version).map_err(|_| UpdateError::InvalidState)?
        {
            return Err(UpdateError::Conflict);
        }
        if input.channel != UpdateChannel::Plugin {
            let current_binding = self
                .authoritative_compatibility_bindings
                .get(&(input.channel, input.component_id.clone()))
                .ok_or(UpdateError::InvalidState)?;
            if current_binding != &candidate.compatibility_sha256 {
                return Err(UpdateError::InvalidState);
            }
            self.verify_store_candidate(candidate)?;
        }
        Ok(())
    }

    pub(crate) fn stage_candidate(
        &self,
        input: &LocalUpdateStageInput,
    ) -> Result<UpdateCandidateSnapshot, UpdateError> {
        self.preflight_stage(input)?;
        self.document
            .candidates
            .iter()
            .find(|candidate| {
                candidate.candidate_id == input.candidate_id
                    && candidate.channel == input.channel
                    && candidate.component_id == input.component_id
            })
            .cloned()
            .ok_or(UpdateError::NotFound)
    }

    pub(crate) fn require_action(&self, input: &UpdateActionInput) -> Result<(), UpdateError> {
        validate_action_input(input)?;
        self.require_generation(input.expected_generation)?;
        let component = self
            .document
            .channels
            .iter()
            .find(|component| {
                component.channel == input.channel && component.component_id == input.component_id
            })
            .ok_or(UpdateError::NotFound)?;
        if component.revoked {
            return Err(UpdateError::Revoked);
        }
        if component.last_known_good_version.is_none() {
            return Err(UpdateError::InvalidState);
        }
        Ok(())
    }

    pub(crate) fn require_revocation(
        &self,
        input: &UpdateRevocationInput,
    ) -> Result<(), UpdateError> {
        validate_action_identity(input.channel, &input.component_id)?;
        validate_identifier(&input.reason_code)?;
        self.require_generation(input.expected_generation)?;
        if self.document.channels.iter().any(|component| {
            component.channel == input.channel && component.component_id == input.component_id
        }) {
            Ok(())
        } else {
            Err(UpdateError::NotFound)
        }
    }

    /// Registers a candidate discovered and verified by a Rust-owned source.
    /// This method is intentionally not a Tauri input surface.
    pub(crate) fn register_discovered_candidate(
        &mut self,
        candidate: UpdateCandidateSnapshot,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_candidate(&candidate)?;
        validate_identifier(correlation_id)?;
        if !self.document.channels.iter().any(|component| {
            component.channel == candidate.channel
                && component.component_id == candidate.component_id
        }) {
            return Err(UpdateError::NotFound);
        }
        if let Some(existing) = self
            .document
            .candidates
            .iter()
            .find(|existing| existing.candidate_id == candidate.candidate_id)
        {
            return if existing.channel == candidate.channel
                && existing.component_id == candidate.component_id
                && existing.version == candidate.version
                && existing.artifact_sha256 == candidate.artifact_sha256
                && existing.compatibility_sha256 == candidate.compatibility_sha256
            {
                Ok(self.snapshot())
            } else {
                Err(UpdateError::Conflict)
            };
        }
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            if next.candidates.len() >= MAX_UPDATE_CANDIDATES {
                return Err(UpdateError::BoundExceeded);
            }
            next.candidates.push(candidate);
            next.candidates.sort_by(|left, right| {
                (
                    left.channel,
                    left.component_id.as_str(),
                    left.version.as_str(),
                )
                    .cmp(&(
                        right.channel,
                        right.component_id.as_str(),
                        right.version.as_str(),
                    ))
            });
            record_diagnostic(
                next,
                correlation_id,
                DiagnosticCategory::Update,
                DiagnosticSeverity::Info,
                "update-discovery",
                "Rust-owned update candidate was discovered and verified",
                None,
                now_ms,
            )
        })
    }

    /// Registers only ExtensionService-owned candidate truth. Package bytes,
    /// verification, compatibility, and selector publication remain delegated
    /// to ExtensionService and its immutable store.
    #[doc(hidden)]
    pub fn register_extension_candidate(
        &mut self,
        component_id: &str,
        version: &str,
        artifact_sha256: &str,
        compatibility_binding: &str,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_identifier(component_id)?;
        validate_version(version)?;
        validate_digest(artifact_sha256)?;
        validate_summary(compatibility_binding)?;
        let compatibility_sha256 = sha256_bytes(compatibility_binding.as_bytes());
        let candidate_id = format!(
            "candidate-{}",
            &sha256_bytes(
                format!("Plugin:{component_id}:{version}:{artifact_sha256}:{compatibility_sha256}")
                    .as_bytes()
            )
            .trim_start_matches("sha256:")[..24]
        );
        self.register_discovered_candidate(
            UpdateCandidateSnapshot {
                candidate_id,
                channel: UpdateChannel::Plugin,
                component_id: component_id.to_owned(),
                version: version.to_owned(),
                artifact_sha256: artifact_sha256.to_owned(),
                compatibility_sha256,
                discovered_at_ms: now_ms,
            },
            correlation_id,
            now_ms,
        )
    }

    pub fn begin_activation(
        &mut self,
        input: &UpdateActionInput,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_input(input)?;
        validate_identifier(correlation_id)?;
        self.check_fault(UpdateFaultPoint::MigrationSnapshot)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            if next.pending_operations.len() >= MAX_UPDATE_PENDING_OPERATIONS {
                return Err(UpdateError::BoundExceeded);
            }
            let sequence = next.next_sequence;
            let (from_version, to_version) = {
                let component = component_mut(next, input.channel, &input.component_id)?;
                if component.revoked {
                    return Err(UpdateError::Revoked);
                }
                if component.state != UpdateLifecycleState::Staged
                    || component.staged_artifact_sha256.is_none()
                    || component.compatibility_sha256.is_none()
                {
                    return Err(UpdateError::InvalidState);
                }
                let to_version = component
                    .candidate_version
                    .clone()
                    .ok_or(UpdateError::InvalidState)?;
                component.state = UpdateLifecycleState::Activating;
                component.recovery_action = Some(UpdateRecoveryAction::RecoverInterrupted);
                component.updated_at_ms = now_ms;
                (component.current_version.clone(), to_version)
            };
            let operation_id = operation_id(
                input.channel,
                &input.component_id,
                &from_version,
                &to_version,
                sequence,
            );
            next.next_sequence = next
                .next_sequence
                .checked_add(1)
                .ok_or(UpdateError::BoundExceeded)?;
            next.pending_operations.push(UpdatePendingOperation {
                operation_id,
                correlation_id: persisted_correlation_id(correlation_id),
                channel: input.channel,
                component_id: input.component_id.clone(),
                from_version,
                to_version,
                state: UpdateOperationState::Activating,
                started_at_ms: now_ms,
            });
            Ok(())
        })
    }

    #[doc(hidden)]
    pub fn complete_activation(
        &mut self,
        input: &UpdateActionInput,
        correlation_id: &str,
        succeeded: bool,
        failure_code: Option<&str>,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_input(input)?;
        validate_identifier(correlation_id)?;
        if input.channel != UpdateChannel::Plugin {
            return Err(UpdateError::InvalidInput);
        }
        if let Some(code) = failure_code {
            validate_identifier(code)?;
        }
        if succeeded {
            self.check_fault(UpdateFaultPoint::SelectorPublication)?;
            self.check_fault(UpdateFaultPoint::Health)?;
            self.check_fault(UpdateFaultPoint::DurableCompletion)?;
        }
        self.mutate(input.expected_generation, now_ms, |next| {
            let pending_index = next
                .pending_operations
                .iter()
                .position(|operation| {
                    operation.channel == input.channel
                        && operation.component_id == input.component_id
                })
                .ok_or(UpdateError::InvalidState)?;
            let pending = next.pending_operations.remove(pending_index);
            let component = component_mut(next, input.channel, &input.component_id)?;
            if succeeded {
                component.last_known_good_version = Some(pending.from_version);
                component.current_version = pending.to_version;
                component.candidate_version = None;
                component.staged_artifact_sha256 = None;
                component.compatibility_sha256 = None;
                component.state = UpdateLifecycleState::Activated;
                component.failure_code = None;
                component.recovery_action = None;
            } else {
                component.current_version = pending.from_version;
                component.state = UpdateLifecycleState::RolledBack;
                component.failure_code = Some(failure_code.unwrap_or("activation_failed").into());
                component.recovery_action = Some(UpdateRecoveryAction::ActivateStaged);
            }
            component.updated_at_ms = now_ms;
            record_diagnostic(
                next,
                correlation_id,
                if succeeded {
                    input.channel.diagnostic_category()
                } else {
                    DiagnosticCategory::Recovery
                },
                if succeeded {
                    DiagnosticSeverity::Info
                } else {
                    DiagnosticSeverity::Error
                },
                &input.component_id,
                if succeeded {
                    "Update activation completed"
                } else {
                    "Update activation failed and last-known-good state was retained"
                },
                (!succeeded).then_some(UpdateRecoveryAction::ActivateStaged),
                now_ms,
            )
        })
    }

    /// Records a safe deferral without claiming activation. Used for application
    /// rebuild gates and runtime active-run/adapter compatibility gates.
    pub fn defer_activation(
        &mut self,
        input: &UpdateActionInput,
        correlation_id: &str,
        failure_code: &str,
        recovery_action: UpdateRecoveryAction,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_input(input)?;
        validate_identifier(correlation_id)?;
        validate_identifier(failure_code)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            let component = component_mut(next, input.channel, &input.component_id)?;
            if component.state != UpdateLifecycleState::Staged {
                return Err(UpdateError::InvalidState);
            }
            component.failure_code = Some(failure_code.into());
            component.recovery_action = Some(recovery_action);
            component.updated_at_ms = now_ms;
            record_diagnostic(
                next,
                correlation_id,
                DiagnosticCategory::Update,
                DiagnosticSeverity::Warning,
                &input.component_id,
                "Update activation was safely deferred",
                Some(recovery_action),
                now_ms,
            )
        })
    }

    pub fn request_rollback(
        &mut self,
        input: &UpdateActionInput,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_input(input)?;
        validate_identifier(correlation_id)?;
        self.check_fault(UpdateFaultPoint::RollbackPublication)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            let component = component_mut(next, input.channel, &input.component_id)?;
            let last_known_good = component
                .last_known_good_version
                .clone()
                .ok_or(UpdateError::InvalidState)?;
            component.candidate_version = Some(last_known_good);
            component.staged_artifact_sha256 = None;
            component.compatibility_sha256 = None;
            component.state = UpdateLifecycleState::Failed;
            component.failure_code = Some("rollback_requested".into());
            component.recovery_action = Some(match input.channel {
                UpdateChannel::Plugin => UpdateRecoveryAction::ReviewPluginUpdate,
                _ => UpdateRecoveryAction::RebuildLastKnownGood,
            });
            component.updated_at_ms = now_ms;
            let recovery_action = component.recovery_action;
            record_diagnostic(
                next,
                correlation_id,
                DiagnosticCategory::Recovery,
                DiagnosticSeverity::Warning,
                &input.component_id,
                "Last-known-good rollback was requested",
                recovery_action,
                now_ms,
            )
        })
    }

    pub fn revoke(
        &mut self,
        input: &UpdateRevocationInput,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_identity(input.channel, &input.component_id)?;
        validate_identifier(&input.reason_code)?;
        validate_identifier(correlation_id)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            next.pending_operations.retain(|operation| {
                operation.channel != input.channel || operation.component_id != input.component_id
            });
            let component = component_mut(next, input.channel, &input.component_id)?;
            component.revoked = true;
            component.state = UpdateLifecycleState::Revoked;
            component.candidate_version = None;
            component.staged_artifact_sha256 = None;
            component.compatibility_sha256 = None;
            component.failure_code = Some(input.reason_code.clone());
            component.recovery_action = None;
            component.updated_at_ms = now_ms;
            record_diagnostic(
                next,
                correlation_id,
                DiagnosticCategory::Security,
                DiagnosticSeverity::Warning,
                &input.component_id,
                "Update authority was revoked",
                None,
                now_ms,
            )
        })
    }

    pub fn recover(
        &mut self,
        input: &UpdateRecoveryInput,
        correlation_id: &str,
        now_ms: u64,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        validate_action_identity(input.channel, &input.component_id)?;
        validate_identifier(&input.recovery_id)?;
        validate_identifier(correlation_id)?;
        self.mutate(input.expected_generation, now_ms, |next| {
            let notice_index = next
                .recovery_notices
                .iter()
                .position(|notice| {
                    notice.recovery_id == input.recovery_id
                        && notice.channel == input.channel
                        && notice.component_id == input.component_id
                })
                .ok_or(UpdateError::NotFound)?;
            next.recovery_notices.remove(notice_index);
            let component = component_mut(next, input.channel, &input.component_id)?;
            if component.candidate_version.is_some() && component.staged_artifact_sha256.is_some() {
                component.state = UpdateLifecycleState::Staged;
                component.failure_code = None;
                component.recovery_action = Some(UpdateRecoveryAction::ActivateStaged);
            } else {
                component.state = if component.revoked {
                    UpdateLifecycleState::Revoked
                } else {
                    UpdateLifecycleState::Current
                };
                component.failure_code = None;
                component.recovery_action = None;
            }
            component.updated_at_ms = now_ms;
            let recovery_action = component.recovery_action;
            record_diagnostic(
                next,
                correlation_id,
                DiagnosticCategory::Recovery,
                DiagnosticSeverity::Info,
                &input.component_id,
                "Interrupted update recovery was reviewed",
                recovery_action,
                now_ms,
            )
        })
    }

    fn recover_interrupted_on_startup(&mut self, now_ms: u64) -> Result<(), UpdateError> {
        if self.document.pending_operations.is_empty() {
            return Ok(());
        }
        let expected = self.document.generation;
        self.mutate(expected, now_ms, |next| {
            let pending = std::mem::take(&mut next.pending_operations);
            for operation in pending {
                if next.recovery_notices.len() >= MAX_UPDATE_RECOVERY_NOTICES {
                    next.recovery_notices.remove(0);
                }
                if let Ok(component) =
                    component_mut(next, operation.channel, &operation.component_id)
                {
                    component.state = UpdateLifecycleState::Failed;
                    component.failure_code = Some("interrupted_activation_unverified".into());
                    component.recovery_action = Some(UpdateRecoveryAction::RecoverInterrupted);
                    component.updated_at_ms = now_ms;
                }
                let recovery_id = format!("recovery-{}", operation.operation_id);
                next.recovery_notices.push(UpdateRecoveryNotice {
                    recovery_id,
                    correlation_id: operation.correlation_id.clone(),
                    channel: operation.channel,
                    component_id: operation.component_id.clone(),
                    summary: "Interrupted activation requires authoritative reconciliation".into(),
                    action: UpdateRecoveryAction::RecoverInterrupted,
                    created_at_ms: now_ms,
                });
                record_diagnostic(
                    next,
                    &operation.correlation_id,
                    DiagnosticCategory::Recovery,
                    DiagnosticSeverity::Warning,
                    &operation.component_id,
                    "Interrupted activation was blocked pending authoritative reconciliation",
                    Some(UpdateRecoveryAction::RecoverInterrupted),
                    now_ms,
                )?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn require_generation(&self, expected_generation: u64) -> Result<(), UpdateError> {
        if expected_generation == self.document.generation {
            Ok(())
        } else {
            Err(UpdateError::Conflict)
        }
    }

    fn mutate(
        &mut self,
        expected_generation: u64,
        now_ms: u64,
        mutation: impl FnOnce(&mut UpdateCoordinatorDocument) -> Result<(), UpdateError>,
    ) -> Result<UpdateCoordinatorSnapshot, UpdateError> {
        self.require_generation(expected_generation)?;
        self.check_fault(UpdateFaultPoint::JournalPrepare)?;
        let mut next = self.document.clone();
        mutation(&mut next)?;
        next.generation = next
            .generation
            .checked_add(1)
            .ok_or(UpdateError::BoundExceeded)?;
        sanitize_diagnostics(&mut next, &self.diagnostic_redaction_patterns);
        let cleanup_faulted = self.injected_faults.remove(&UpdateFaultPoint::Cleanup);
        if cleanup_faulted {
            set_cleanup_residue(&mut next, true, now_ms);
        }
        normalize_persisted_correlations(&mut next);
        bounded_cleanup(&mut next, now_ms, self.diagnostic_retention_days);
        validate_document(&next)?;
        self.persist_document(&next, Some(self.document.generation), now_ms.max(1))?;
        self.document = next;
        if !cleanup_faulted {
            let cleanup_failed = self.cleanup_unreferenced_store_bytes().is_err();
            let _ = self.persist_cleanup_residue_state(cleanup_failed, now_ms);
        }
        Ok(self.snapshot())
    }

    fn check_fault(&mut self, point: UpdateFaultPoint) -> Result<(), UpdateError> {
        if self.injected_faults.remove(&point) {
            Err(UpdateError::InjectedFault(point))
        } else {
            Ok(())
        }
    }

    fn verify_store_candidate(
        &self,
        candidate: &UpdateCandidateSnapshot,
    ) -> Result<(), UpdateError> {
        let digest = candidate
            .artifact_sha256
            .strip_prefix("sha256:")
            .ok_or(UpdateError::InvalidState)?;
        let stored = self.validated_store_root()?.join(digest);
        let metadata = fs::symlink_metadata(&stored).map_err(|_| UpdateError::InvalidState)?;
        if !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_UPDATE_ARTIFACT_BYTES
            || sha256_file(&stored).map_err(|_| UpdateError::InvalidState)?
                != candidate.artifact_sha256
        {
            return Err(UpdateError::InvalidState);
        }
        Ok(())
    }

    fn cleanup_unreferenced_store_bytes(&self) -> Result<(), UpdateError> {
        let store_root = self.validated_store_root()?;
        let retained = self
            .document
            .candidates
            .iter()
            .map(|candidate| candidate.artifact_sha256.as_str())
            .chain(
                self.document
                    .channels
                    .iter()
                    .filter_map(|component| component.staged_artifact_sha256.as_deref()),
            )
            .filter_map(|digest| digest.strip_prefix("sha256:"))
            .collect::<BTreeSet<_>>();
        let entries = match fs::read_dir(&store_root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err(UpdateError::Persistence),
        };
        let mut failed = false;
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    failed = true;
                    continue;
                }
            };
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let metadata = match fs::symlink_metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(_) => {
                    failed = true;
                    continue;
                }
            };
            if file_name.len() == 64
                && file_name
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
            {
                if !metadata.file_type().is_file() {
                    failed = true;
                } else if !retained.contains(file_name) {
                    failed |= fs::remove_file(entry.path()).is_err();
                }
            } else if file_name.starts_with(".candidate-") {
                if metadata.file_type().is_file() {
                    failed |= fs::remove_file(entry.path()).is_err();
                } else {
                    failed = true;
                }
            }
        }
        if failed {
            Err(UpdateError::Persistence)
        } else {
            Ok(())
        }
    }

    fn validated_store_root(&self) -> Result<PathBuf, UpdateError> {
        let metadata =
            fs::symlink_metadata(&self.store_root).map_err(|_| UpdateError::Persistence)?;
        if !metadata.file_type().is_dir() {
            return Err(UpdateError::InvalidState);
        }
        let canonical = self
            .store_root
            .canonicalize()
            .map_err(|_| UpdateError::Persistence)?;
        if canonical != self.store_root || !canonical.starts_with(&self.store_authority_root) {
            return Err(UpdateError::InvalidState);
        }
        Ok(canonical)
    }

    fn persist_cleanup_residue_state(
        &mut self,
        failed: bool,
        now_ms: u64,
    ) -> Result<(), UpdateError> {
        if self.document.cleanup_residue == failed {
            return Ok(());
        }
        let mut next = self.document.clone();
        set_cleanup_residue(&mut next, failed, now_ms);
        next.generation = next
            .generation
            .checked_add(1)
            .ok_or(UpdateError::BoundExceeded)?;
        sanitize_diagnostics(&mut next, &self.diagnostic_redaction_patterns);
        bounded_cleanup(&mut next, now_ms, self.diagnostic_retention_days);
        validate_document(&next)?;
        self.persist_document(&next, Some(self.document.generation), now_ms.max(1))?;
        self.document = next;
        Ok(())
    }

    fn persist_document(
        &self,
        document: &UpdateCoordinatorDocument,
        expected_generation: Option<u64>,
        now_ms: u64,
    ) -> Result<(), UpdateError> {
        let canonical_document =
            serde_json::to_string(document).map_err(|_| UpdateError::InvalidState)?;
        let diagnostics = document
            .diagnostics
            .iter()
            .map(|diagnostic| DiagnosticRecord {
                diagnostic_id: diagnostic.diagnostic_id.clone(),
                category: format!("{:?}.{:?}", diagnostic.category, diagnostic.severity)
                    .to_ascii_lowercase(),
                message: diagnostic.message.clone(),
                created_at: i64::try_from(diagnostic.created_at_ms).unwrap_or(i64::MAX),
            })
            .collect::<Vec<_>>();
        let diagnostic_ids_to_replace = self
            .document
            .diagnostics
            .iter()
            .chain(document.diagnostics.iter())
            .map(|diagnostic| diagnostic.diagnostic_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let retention_ms = u64::from(self.diagnostic_retention_days).saturating_mul(86_400_000);
        let retain_after = i64::try_from(now_ms.saturating_sub(retention_ms)).unwrap_or(i64::MAX);
        self.database.save_runtime_state_with_diagnostics(
            RuntimeStateDocumentRecord {
                document_kind: DOCUMENT_KIND.into(),
                document_id: DOCUMENT_ID.into(),
                generation: document.generation,
                canonical_document,
                updated_at_ms: now_ms,
            },
            expected_generation,
            diagnostics,
            diagnostic_ids_to_replace,
            retain_after,
            MAX_DIAGNOSTIC_RECORDS,
        )?;
        Ok(())
    }
}

fn prepare_store_root(c4os_home: &Path) -> Result<(PathBuf, PathBuf), UpdateError> {
    let authority_root = c4os_home
        .canonicalize()
        .map_err(|_| UpdateError::InvalidState)?;
    let authority_metadata =
        fs::symlink_metadata(&authority_root).map_err(|_| UpdateError::InvalidState)?;
    if !authority_metadata.file_type().is_dir() {
        return Err(UpdateError::InvalidState);
    }
    let mut current = authority_root.clone();
    for component in ["updates", "store", "sha256"] {
        let child = current.join(component);
        match fs::symlink_metadata(&child) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => return Err(UpdateError::InvalidState),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&child).map_err(|_| UpdateError::Persistence)?;
            }
            Err(_) => return Err(UpdateError::Persistence),
        }
        let canonical = child.canonicalize().map_err(|_| UpdateError::Persistence)?;
        if !canonical.starts_with(&authority_root) {
            return Err(UpdateError::InvalidState);
        }
        current = canonical;
    }
    Ok((authority_root, current))
}

fn component_mut<'a>(
    document: &'a mut UpdateCoordinatorDocument,
    channel: UpdateChannel,
    component_id: &str,
) -> Result<&'a mut UpdateComponentSnapshot, UpdateError> {
    document
        .channels
        .iter_mut()
        .find(|component| component.channel == channel && component.component_id == component_id)
        .ok_or(UpdateError::NotFound)
}

fn record_diagnostic(
    document: &mut UpdateCoordinatorDocument,
    correlation_id: &str,
    category: DiagnosticCategory,
    severity: DiagnosticSeverity,
    component_boundary: &str,
    message: &str,
    recovery_action: Option<UpdateRecoveryAction>,
    now_ms: u64,
) -> Result<(), UpdateError> {
    validate_identifier(correlation_id)?;
    validate_identifier(component_boundary)?;
    validate_summary(message)?;
    let sequence = document.next_sequence;
    document.next_sequence = document
        .next_sequence
        .checked_add(1)
        .ok_or(UpdateError::BoundExceeded)?;
    let diagnostic_id = format!(
        "diagnostic-{}",
        &sha256_bytes(format!("{correlation_id}:{component_boundary}:{sequence}").as_bytes())
            .trim_start_matches("sha256:")[..24]
    );
    document.diagnostics.push(UpdateDiagnosticRecord {
        diagnostic_id,
        correlation_id: persisted_correlation_id(correlation_id),
        category,
        severity,
        component_boundary: component_boundary.to_owned(),
        message: message.to_owned(),
        recovery_action,
        created_at_ms: now_ms,
    });
    Ok(())
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn set_cleanup_residue(document: &mut UpdateCoordinatorDocument, retained: bool, now_ms: u64) {
    const CLEANUP_DIAGNOSTIC_ID: &str = "diagnostic-update-cleanup-residue";
    document.cleanup_residue = retained;
    document
        .diagnostics
        .retain(|record| record.diagnostic_id != CLEANUP_DIAGNOSTIC_ID);
    if retained {
        document.diagnostics.push(UpdateDiagnosticRecord {
            diagnostic_id: CLEANUP_DIAGNOSTIC_ID.into(),
            correlation_id: persisted_correlation_id("update:cleanup"),
            category: DiagnosticCategory::Persistence,
            severity: DiagnosticSeverity::Warning,
            component_boundary: "update-coordinator".into(),
            message: "Update cleanup residue was retained for a later bounded retry".into(),
            recovery_action: Some(UpdateRecoveryAction::RetryStage),
            created_at_ms: now_ms.max(1),
        });
    }
}

fn persisted_correlation_id(correlation_id: &str) -> String {
    if is_persisted_correlation_id(correlation_id) {
        return correlation_id.to_owned();
    }
    format!(
        "correlation-{}",
        &sha256_bytes(correlation_id.as_bytes()).trim_start_matches("sha256:")[..24]
    )
}

fn is_persisted_correlation_id(correlation_id: &str) -> bool {
    correlation_id
        .strip_prefix("correlation-")
        .is_some_and(|digest| {
            digest.len() == 24
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        })
}

fn normalize_persisted_correlations(document: &mut UpdateCoordinatorDocument) -> bool {
    let mut changed = false;
    for correlation_id in document
        .pending_operations
        .iter_mut()
        .map(|operation| &mut operation.correlation_id)
        .chain(
            document
                .recovery_notices
                .iter_mut()
                .map(|notice| &mut notice.correlation_id),
        )
        .chain(
            document
                .diagnostics
                .iter_mut()
                .map(|diagnostic| &mut diagnostic.correlation_id),
        )
    {
        let persisted = persisted_correlation_id(correlation_id);
        if *correlation_id != persisted {
            *correlation_id = persisted;
            changed = true;
        }
    }
    changed
}

fn sanitize_diagnostics(document: &mut UpdateCoordinatorDocument, configured_patterns: &[String]) {
    for diagnostic in &mut document.diagnostics {
        diagnostic.message = redact_diagnostic_text(&diagnostic.message, configured_patterns);
        diagnostic.component_boundary =
            redact_diagnostic_identifier(&diagnostic.component_boundary, configured_patterns);
    }
    for notice in &mut document.recovery_notices {
        notice.summary = redact_diagnostic_text(&notice.summary, configured_patterns);
    }
}

fn redact_diagnostic_identifier(value: &str, configured_patterns: &[String]) -> String {
    if configured_patterns
        .iter()
        .any(|pattern| value.contains(pattern))
        || looks_sensitive(value)
    {
        "redacted".into()
    } else {
        value.to_owned()
    }
}

fn redact_diagnostic_text(value: &str, configured_patterns: &[String]) -> String {
    if looks_sensitive(value)
        || configured_patterns
            .iter()
            .any(|pattern| value.contains(pattern))
    {
        return "[redacted diagnostic]".into();
    }
    value.to_owned()
}

fn bounded_cleanup(document: &mut UpdateCoordinatorDocument, now_ms: u64, retention_days: u16) {
    let retention_ms = u64::from(retention_days).saturating_mul(86_400_000);
    let retain_after = now_ms.saturating_sub(retention_ms);
    let staged_digests = document
        .channels
        .iter()
        .filter_map(|component| component.staged_artifact_sha256.as_deref())
        .collect::<BTreeSet<_>>();
    document.candidates.retain(|candidate| {
        candidate.discovered_at_ms >= retain_after
            || staged_digests.contains(candidate.artifact_sha256.as_str())
    });
    if document.candidates.len() > MAX_UPDATE_CANDIDATES {
        document
            .candidates
            .sort_by_key(|candidate| candidate.discovered_at_ms);
        let excess = document.candidates.len() - MAX_UPDATE_CANDIDATES;
        document.candidates.drain(..excess);
    }
    document
        .diagnostics
        .retain(|record| record.created_at_ms >= retain_after);
    if document.diagnostics.len() > MAX_DIAGNOSTIC_RECORDS {
        let excess = document.diagnostics.len() - MAX_DIAGNOSTIC_RECORDS;
        document.diagnostics.drain(..excess);
    }
    if document.recovery_notices.len() > MAX_UPDATE_RECOVERY_NOTICES {
        let excess = document.recovery_notices.len() - MAX_UPDATE_RECOVERY_NOTICES;
        document.recovery_notices.drain(..excess);
    }
}

fn validate_document(document: &UpdateCoordinatorDocument) -> Result<(), UpdateError> {
    if document.schema_version != UPDATE_COORDINATOR_SCHEMA_VERSION
        || document.generation == 0
        || document.next_sequence == 0
        || document.channels.len() > MAX_UPDATE_COMPONENTS
        || document.candidates.len() > MAX_UPDATE_CANDIDATES
        || document.pending_operations.len() > MAX_UPDATE_PENDING_OPERATIONS
        || document.recovery_notices.len() > MAX_UPDATE_RECOVERY_NOTICES
        || document.diagnostics.len() > MAX_DIAGNOSTIC_RECORDS
    {
        return Err(UpdateError::InvalidState);
    }
    let mut identities = BTreeSet::new();
    for component in &document.channels {
        validate_identifier(&component.component_id)?;
        validate_version(&component.current_version)?;
        if component.updated_at_ms == 0
            || !identities.insert((component.channel, component.component_id.as_str()))
            || component.revoked != (component.state == UpdateLifecycleState::Revoked)
        {
            return Err(UpdateError::InvalidState);
        }
        if let Some(version) = &component.candidate_version {
            validate_version(version)?;
        }
        if let Some(version) = &component.last_known_good_version {
            validate_version(version)?;
        }
        if let Some(digest) = &component.staged_artifact_sha256 {
            validate_digest(digest)?;
        }
        if let Some(digest) = &component.compatibility_sha256 {
            validate_digest(digest)?;
        }
        if let Some(code) = &component.failure_code {
            validate_identifier(code)?;
        }
        let stage_field_count = usize::from(component.candidate_version.is_some())
            + usize::from(component.staged_artifact_sha256.is_some())
            + usize::from(component.compatibility_sha256.is_some());
        let deferred_rollback = component.state == UpdateLifecycleState::Failed
            && component.failure_code.as_deref() == Some("rollback_requested")
            && component.candidate_version.is_some()
            && component.staged_artifact_sha256.is_none()
            && component.compatibility_sha256.is_none();
        if !matches!(stage_field_count, 0 | 3) && !deferred_rollback {
            return Err(UpdateError::InvalidState);
        }
        match component.state {
            UpdateLifecycleState::Staged | UpdateLifecycleState::Activating
                if stage_field_count != 3 =>
            {
                return Err(UpdateError::InvalidState);
            }
            UpdateLifecycleState::Current
            | UpdateLifecycleState::Activated
            | UpdateLifecycleState::Revoked
                if stage_field_count != 0 =>
            {
                return Err(UpdateError::InvalidState);
            }
            _ => {}
        }
    }
    let mut candidate_ids = BTreeSet::new();
    for candidate in &document.candidates {
        validate_candidate(candidate)?;
        if !candidate_ids.insert(candidate.candidate_id.as_str())
            || !identities.contains(&(candidate.channel, candidate.component_id.as_str()))
        {
            return Err(UpdateError::InvalidState);
        }
    }
    for component in &document.channels {
        if matches!(
            component.state,
            UpdateLifecycleState::Staged | UpdateLifecycleState::Activating
        ) {
            let exact_matches = document
                .candidates
                .iter()
                .filter(|candidate| {
                    candidate.channel == component.channel
                        && candidate.component_id == component.component_id
                        && Some(candidate.version.as_str())
                            == component.candidate_version.as_deref()
                        && Some(candidate.artifact_sha256.as_str())
                            == component.staged_artifact_sha256.as_deref()
                        && Some(candidate.compatibility_sha256.as_str())
                            == component.compatibility_sha256.as_deref()
                })
                .count();
            if exact_matches != 1 {
                return Err(UpdateError::InvalidState);
            }
        }
    }
    let mut pending_ids = BTreeSet::new();
    let mut pending_components = BTreeMap::new();
    for pending in &document.pending_operations {
        validate_identifier(&pending.operation_id)?;
        if !is_persisted_correlation_id(&pending.correlation_id) {
            return Err(UpdateError::InvalidState);
        }
        validate_identifier(&pending.component_id)?;
        validate_version(&pending.from_version)?;
        validate_version(&pending.to_version)?;
        let identity = (pending.channel, pending.component_id.as_str());
        if pending.started_at_ms == 0
            || !pending_ids.insert(pending.operation_id.as_str())
            || pending_components.insert(identity, pending).is_some()
            || !identities.contains(&identity)
        {
            return Err(UpdateError::InvalidState);
        }
        let component = document
            .channels
            .iter()
            .find(|component| {
                component.channel == pending.channel
                    && component.component_id == pending.component_id
            })
            .ok_or(UpdateError::InvalidState)?;
        if component.state != UpdateLifecycleState::Activating
            || component.current_version != pending.from_version
            || component.candidate_version.as_deref() != Some(pending.to_version.as_str())
        {
            return Err(UpdateError::InvalidState);
        }
    }
    for component in &document.channels {
        let has_pending =
            pending_components.contains_key(&(component.channel, component.component_id.as_str()));
        if has_pending != (component.state == UpdateLifecycleState::Activating) {
            return Err(UpdateError::InvalidState);
        }
    }
    let mut recovery_ids = BTreeSet::new();
    let mut recovery_components = BTreeSet::new();
    for notice in &document.recovery_notices {
        validate_identifier(&notice.recovery_id)?;
        if !is_persisted_correlation_id(&notice.correlation_id) {
            return Err(UpdateError::InvalidState);
        }
        validate_identifier(&notice.component_id)?;
        validate_summary(&notice.summary)?;
        let identity = (notice.channel, notice.component_id.as_str());
        if notice.created_at_ms == 0
            || !recovery_ids.insert(notice.recovery_id.as_str())
            || !recovery_components.insert(identity)
            || !identities.contains(&identity)
        {
            return Err(UpdateError::InvalidState);
        }
    }
    let mut diagnostic_ids = BTreeSet::new();
    for diagnostic in &document.diagnostics {
        validate_identifier(&diagnostic.diagnostic_id)?;
        if !is_persisted_correlation_id(&diagnostic.correlation_id) {
            return Err(UpdateError::InvalidState);
        }
        validate_identifier(&diagnostic.component_boundary)?;
        validate_summary(&diagnostic.message)?;
        if diagnostic.created_at_ms == 0
            || !diagnostic_ids.insert(diagnostic.diagnostic_id.as_str())
        {
            return Err(UpdateError::InvalidState);
        }
    }
    if document.cleanup_residue != diagnostic_ids.contains("diagnostic-update-cleanup-residue") {
        return Err(UpdateError::InvalidState);
    }
    Ok(())
}

fn rehydrate_staged_compatibility(
    document: &mut UpdateCoordinatorDocument,
) -> Result<(), UpdateError> {
    for component in &mut document.channels {
        if component.compatibility_sha256.is_some()
            || component.candidate_version.is_none()
            || component.staged_artifact_sha256.is_none()
        {
            continue;
        }
        let matches = document
            .candidates
            .iter()
            .filter(|candidate| {
                candidate.channel == component.channel
                    && candidate.component_id == component.component_id
                    && Some(candidate.version.as_str()) == component.candidate_version.as_deref()
                    && Some(candidate.artifact_sha256.as_str())
                        == component.staged_artifact_sha256.as_deref()
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(UpdateError::InvalidState);
        }
        component.compatibility_sha256 = Some(matches[0].compatibility_sha256.clone());
    }
    Ok(())
}

fn validate_stage_input(input: &LocalUpdateStageInput) -> Result<(), UpdateError> {
    validate_action_identity(input.channel, &input.component_id)?;
    validate_identifier(&input.candidate_id)
}

fn validate_candidate(candidate: &UpdateCandidateSnapshot) -> Result<(), UpdateError> {
    validate_identifier(&candidate.candidate_id)?;
    validate_identifier(&candidate.component_id)?;
    validate_version(&candidate.version)?;
    validate_digest(&candidate.artifact_sha256)?;
    validate_digest(&candidate.compatibility_sha256)?;
    if candidate.discovered_at_ms == 0 {
        return Err(UpdateError::InvalidInput);
    }
    Ok(())
}

fn validate_action_input(input: &UpdateActionInput) -> Result<(), UpdateError> {
    validate_action_identity(input.channel, &input.component_id)
}

fn validate_action_identity(
    _channel: UpdateChannel,
    component_id: &str,
) -> Result<(), UpdateError> {
    validate_identifier(component_id)
}

fn validate_identifier(value: &str) -> Result<(), UpdateError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'@')
        });
    if valid && !looks_sensitive(value) {
        Ok(())
    } else {
        Err(UpdateError::InvalidInput)
    }
}

fn validate_summary(value: &str) -> Result<(), UpdateError> {
    if !value.is_empty() && value.len() <= MAX_SUMMARY_BYTES && !value.chars().any(char::is_control)
    {
        Ok(())
    } else {
        Err(UpdateError::InvalidInput)
    }
}

fn looks_sensitive(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("bearer ")
        || normalized.contains("authorization:")
        || normalized.contains("basic ")
        || normalized.contains("cookie=")
        || normalized.contains("set-cookie:")
        || normalized.contains("password=")
        || normalized.contains("token=")
        || normalized.contains("secret=")
        || normalized.contains("api_key=")
        || normalized.contains("apikey=")
        || normalized.contains("-----begin ")
        || normalized.contains("file://")
        || normalized.contains("/users/")
        || normalized.contains("/private/")
        || normalized.contains("/var/")
        || normalized.contains("/tmp/")
        || normalized.contains("\\users\\")
        || normalized.contains("home=")
        || normalized.contains("path=")
}

fn validate_version(value: &str) -> Result<(), UpdateError> {
    if value.len() > MAX_IDENTIFIER_BYTES || Version::parse(value).is_err() {
        Err(UpdateError::InvalidInput)
    } else {
        Ok(())
    }
}

fn validate_digest(value: &str) -> Result<(), UpdateError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(UpdateError::InvalidInput);
    };
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(UpdateError::InvalidInput)
    }
}

fn operation_id(
    channel: UpdateChannel,
    component_id: &str,
    from_version: &str,
    to_version: &str,
    sequence: u64,
) -> String {
    let canonical = format!("{channel:?}:{component_id}:{from_version}:{to_version}:{sequence}");
    format!(
        "operation-{}",
        &sha256_bytes(canonical.as_bytes()).trim_start_matches("sha256:")[..24]
    )
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut output = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn sha256_file(path: &Path) -> Result<String, UpdateError> {
    let mut file = File::open(path).map_err(|_| UpdateError::Persistence)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| UpdateError::Persistence)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let mut output = String::from("sha256:");
    for byte in digest.finalize() {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    Ok(output)
}
