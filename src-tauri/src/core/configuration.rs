//! Rust-owned, strict, scope-aware C4OS configuration.
//!
//! Editable TOML is an input to this service, never an authority of its own.
//! The service validates a complete scope, keeps a canonical last-known-good
//! document, and publishes immutable effective snapshots after applying
//! precedence and non-bypassable policy.

use notify::{Config, Event, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

pub const CONFIGURATION_SCHEMA_VERSION: u16 = 1;
pub const MAX_CONFIGURATION_BYTES: usize = 1_048_576;
const DEFAULT_STABLE_READ_ATTEMPTS: usize = 4;
const DEFAULT_STABLE_READ_DELAY: Duration = Duration::from_millis(8);
const DEFAULT_WATCH_DEBOUNCE: Duration = Duration::from_millis(75);
const WATCH_POLL_FALLBACK_INTERVAL: Duration = Duration::from_millis(250);
const MAX_DEBOUNCE_WINDOWS: u32 = 4;

fn is_empty_map<K, V>(value: &BTreeMap<K, V>) -> bool {
    value.is_empty()
}

/// The four editable levels in increasing-precedence order.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationScope {
    App,
    Workspace,
    Project,
    Chat,
}

const APP_CONFIGURATION_KEYS: &[&str] = &[
    "schema_version",
    "restore_last_workspace",
    "model_route",
    "default_runtime",
    "default_environment",
    "default_approval_preset",
    "inherit_shell_environment",
    "shell_environment_allowlist",
    "browser_environment",
    "requested_limits",
    "approval_overrides",
    "updates",
    "diagnostics",
];
// Workspace owns portable defaults, including the runtime and inherited shell
// boundary used by its Projects.
const WORKSPACE_CONFIGURATION_KEYS: &[&str] = &[
    "schema_version",
    "model_route",
    "default_runtime",
    "default_environment",
    "default_approval_preset",
    "inherit_shell_environment",
    "shell_environment_allowlist",
    "browser_environment",
    "requested_limits",
    "approval_overrides",
];
// Project may narrow environment and approvals but cannot replace the
// Workspace-selected runtime default.
const PROJECT_CONFIGURATION_KEYS: &[&str] = &[
    "schema_version",
    "model_route",
    "default_environment",
    "default_approval_preset",
    "inherit_shell_environment",
    "shell_environment_allowlist",
    "browser_environment",
    "requested_limits",
    "approval_overrides",
];
// Chat may narrow per-session routing, environment, approvals, Browser scope,
// and resource requests; process-level runtime/shell inheritance stays above it.
const CHAT_CONFIGURATION_KEYS: &[&str] = &[
    "schema_version",
    "model_route",
    "default_environment",
    "default_approval_preset",
    "browser_environment",
    "requested_limits",
    "approval_overrides",
];

impl ConfigurationScope {
    pub const PRECEDENCE: [Self; 4] = [Self::App, Self::Workspace, Self::Project, Self::Chat];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Workspace => "workspace",
            Self::Project => "project",
            Self::Chat => "chat",
        }
    }

    pub const fn allowed_keys(self) -> &'static [&'static str] {
        match self {
            Self::App => APP_CONFIGURATION_KEYS,
            Self::Workspace => WORKSPACE_CONFIGURATION_KEYS,
            Self::Project => PROJECT_CONFIGURATION_KEYS,
            Self::Chat => CHAT_CONFIGURATION_KEYS,
        }
    }

    fn allows_key(self, key: &str) -> bool {
        self.allowed_keys().contains(&key)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalPreset {
    AskForApproval,
    #[default]
    ApproveSafeActions,
    ApproveForMe,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserEnvironment {
    AppWide,
    WorkspaceProject,
    Chat,
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    Automatic,
    Staged,
    Manual,
    Disabled,
}

/// User-requested resource limits. Managed ceilings are deliberately a
/// separate Rust-owned input and are not deserializable from TOML.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel_actions: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsPreferences {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_days: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redaction_patterns: Option<Vec<String>>,
}

/// Strict TOML document shared by the four scopes. Scope validation runs after
/// deserialization so a known app-only key can produce a precise scope error.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationDocument {
    pub schema_version: u16,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_last_workspace: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_route: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_approval_preset: Option<ApprovalPreset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherit_shell_environment: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell_environment_allowlist: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_environment: Option<BrowserEnvironment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_limits: Option<RequestedLimits>,

    /// A declared keyed table. Entries merge by key across scopes.
    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub approval_overrides: BTreeMap<String, ApprovalPreset>,

    /// App-only independent update-channel preferences.
    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub updates: BTreeMap<String, UpdatePolicy>,
    /// App-only diagnostic retention and extra redaction preferences.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<DiagnosticsPreferences>,
}

impl Default for ConfigurationDocument {
    fn default() -> Self {
        Self {
            schema_version: CONFIGURATION_SCHEMA_VERSION,
            restore_last_workspace: None,
            model_route: None,
            default_runtime: None,
            default_environment: None,
            default_approval_preset: None,
            inherit_shell_environment: None,
            shell_environment_allowlist: None,
            browser_environment: None,
            requested_limits: None,
            approval_overrides: BTreeMap::new(),
            updates: BTreeMap::new(),
            diagnostics: None,
        }
    }
}

/// Non-editable managed limits applied after all four user scopes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedCeilings {
    pub parallel_actions: u16,
    pub output_bytes: u64,
}

impl Default for ManagedCeilings {
    fn default() -> Self {
        Self {
            parallel_actions: u16::MAX,
            output_bytes: u64::MAX,
        }
    }
}

/// Non-bypassable constraints applied after managed ceilings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityConstraints {
    pub forced_approval_preset: Option<ApprovalPreset>,
    pub shell_environment_allowed: bool,
    pub permitted_shell_environment_names: Option<BTreeSet<String>>,
    pub forced_approval_overrides: BTreeMap<String, ApprovalPreset>,
}

impl Default for SecurityConstraints {
    fn default() -> Self {
        Self {
            forced_approval_preset: None,
            shell_environment_allowed: true,
            permitted_shell_environment_names: None,
            forced_approval_overrides: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EffectiveConfiguration {
    pub restore_last_workspace: bool,
    pub model_route: Option<String>,
    pub default_runtime: Option<String>,
    pub default_environment: Option<String>,
    pub default_approval_preset: ApprovalPreset,
    pub inherit_shell_environment: bool,
    pub shell_environment_allowlist: Vec<String>,
    pub browser_environment: BrowserEnvironment,
    pub parallel_actions: Option<u16>,
    pub output_bytes: Option<u64>,
    pub approval_overrides: BTreeMap<String, ApprovalPreset>,
    pub updates: BTreeMap<String, UpdatePolicy>,
    pub diagnostics_retention_days: Option<u16>,
    pub diagnostic_redaction_patterns: Vec<String>,
}

impl Default for EffectiveConfiguration {
    fn default() -> Self {
        Self {
            restore_last_workspace: false,
            model_route: None,
            default_runtime: None,
            default_environment: None,
            default_approval_preset: ApprovalPreset::ApproveSafeActions,
            inherit_shell_environment: false,
            shell_environment_allowlist: Vec::new(),
            browser_environment: BrowserEnvironment::None,
            parallel_actions: None,
            output_bytes: None,
            approval_overrides: BTreeMap::new(),
            updates: BTreeMap::new(),
            diagnostics_retention_days: None,
            diagnostic_redaction_patterns: Vec::new(),
        }
    }
}

/// The service retains the same `Arc` it returns, so callers cannot obtain a
/// unique mutable reference to the active configuration.
#[derive(Clone, Debug)]
pub struct EffectiveConfigurationSnapshot {
    pub generation: u64,
    pub configuration: Arc<EffectiveConfiguration>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationDiagnosticCode {
    UnstableRead,
    Io,
    InvalidToml,
    UnsupportedSchema,
    ForbiddenContent,
    InvalidScopeKey,
    InvalidValue,
    SizeLimitExceeded,
    GenerationOverflow,
    Watcher,
}

/// Safe to persist: it identifies a path/key and a fixed diagnostic category,
/// but never embeds the rejected TOML value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConfigurationDiagnostic {
    pub scope: ConfigurationScope,
    pub path: PathBuf,
    pub key: Option<String>,
    pub code: ConfigurationDiagnosticCode,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LastKnownGoodDocument {
    pub scope: ConfigurationScope,
    pub path: PathBuf,
    pub canonical_toml: String,
    pub activated_generation: u64,
    pub changed_keys: BTreeSet<String>,
}

/// A complete scope document that passed the same strict checks used for live
/// activation. Archive inspection can use this without mutating service state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedConfigurationDocument {
    pub scope: ConfigurationScope,
    pub path: PathBuf,
    pub canonical_toml: String,
    document: ConfigurationDocument,
}

impl ValidatedConfigurationDocument {
    pub fn document(&self) -> &ConfigurationDocument {
        &self.document
    }

    fn into_parts(self) -> (ConfigurationDocument, String) {
        (self.document, self.canonical_toml)
    }
}

#[derive(Clone, Debug)]
struct ActiveScope {
    document: ConfigurationDocument,
    record: LastKnownGoodDocument,
    source_fingerprint: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct StaleGenerationConflict {
    pub base_generation: u64,
    pub active_generation: u64,
    pub changed_keys: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct ExternalConfigurationConflict {
    pub base_generation: u64,
    pub active_generation: u64,
    pub changed_keys: BTreeSet<String>,
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("configuration generation is stale")]
    StaleGeneration(StaleGenerationConflict),
    #[error("configuration file changed outside the active generation")]
    ExternalReplacement(ExternalConfigurationConflict),
    #[error("configuration generation cannot advance beyond u64::MAX")]
    GenerationOverflow,
    #[error("configuration {operation} failed at {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("configuration watcher setup failed: {0}")]
    Watcher(#[from] notify::Error),
}

#[derive(Clone, Debug)]
pub enum ConfigurationUpdate {
    Activated {
        snapshot: EffectiveConfigurationSnapshot,
        record: LastKnownGoodDocument,
    },
    Unchanged {
        snapshot: EffectiveConfigurationSnapshot,
    },
    DeduplicatedSelfWrite {
        snapshot: EffectiveConfigurationSnapshot,
    },
    Rejected {
        diagnostic: ConfigurationDiagnostic,
        snapshot: EffectiveConfigurationSnapshot,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct StableReadPolicy {
    pub max_attempts: usize,
    pub retry_delay: Duration,
}

impl Default for StableReadPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_STABLE_READ_ATTEMPTS,
            retry_delay: DEFAULT_STABLE_READ_DELAY,
        }
    }
}

#[derive(Clone)]
pub struct ConfigurationService {
    scopes: BTreeMap<ConfigurationScope, ActiveScope>,
    managed_ceilings: ManagedCeilings,
    security_constraints: SecurityConstraints,
    snapshot: EffectiveConfigurationSnapshot,
    changes_by_generation: BTreeMap<u64, BTreeSet<String>>,
    diagnostics: Vec<ConfigurationDiagnostic>,
    self_write_fingerprints: BTreeMap<PathBuf, [u8; 32]>,
    stable_read_policy: StableReadPolicy,
}

impl ConfigurationService {
    pub fn new(
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Self {
        let effective = apply_policies(
            EffectiveConfiguration::default(),
            &managed_ceilings,
            &security_constraints,
        );
        Self {
            scopes: BTreeMap::new(),
            managed_ceilings,
            security_constraints,
            snapshot: EffectiveConfigurationSnapshot {
                generation: 0,
                configuration: Arc::new(effective),
            },
            changes_by_generation: BTreeMap::new(),
            diagnostics: Vec::new(),
            self_write_fingerprints: BTreeMap::new(),
            stable_read_policy: StableReadPolicy::default(),
        }
    }

    /// Reconstruct active state from persisted canonical last-known-good
    /// records. The complete restore fails closed on any corrupt record.
    pub fn from_last_known_good(
        records: impl IntoIterator<Item = LastKnownGoodDocument>,
        managed_ceilings: ManagedCeilings,
        security_constraints: SecurityConstraints,
    ) -> Result<Self, ConfigurationDiagnostic> {
        let mut scopes = BTreeMap::new();
        let mut changes_by_generation = BTreeMap::new();
        let mut generations = BTreeSet::new();
        let mut maximum_generation = 0;

        let mut records: Vec<_> = records.into_iter().collect();
        records.sort_by(|left, right| {
            (left.scope, left.activated_generation, &left.path).cmp(&(
                right.scope,
                right.activated_generation,
                &right.path,
            ))
        });

        for persisted in records {
            if persisted.activated_generation == 0 {
                return Err(ConfigurationDiagnostic {
                    scope: persisted.scope,
                    path: persisted.path,
                    key: Some("activated_generation".to_owned()),
                    code: ConfigurationDiagnosticCode::InvalidValue,
                    message: "persisted configuration generation must be positive".to_owned(),
                });
            }
            if scopes.contains_key(&persisted.scope) {
                return Err(ConfigurationDiagnostic {
                    scope: persisted.scope,
                    path: persisted.path,
                    key: None,
                    code: ConfigurationDiagnosticCode::InvalidValue,
                    message: "persisted configuration contains a duplicate scope".to_owned(),
                });
            }
            if !generations.insert(persisted.activated_generation) {
                return Err(ConfigurationDiagnostic {
                    scope: persisted.scope,
                    path: persisted.path,
                    key: Some("activated_generation".to_owned()),
                    code: ConfigurationDiagnosticCode::InvalidValue,
                    message: "persisted configuration generation is duplicated".to_owned(),
                });
            }

            let validated = validate_scope_document(
                persisted.scope,
                persisted.path.clone(),
                &persisted.canonical_toml,
            )?;
            if validated.canonical_toml != persisted.canonical_toml {
                return Err(ConfigurationDiagnostic {
                    scope: persisted.scope,
                    path: persisted.path,
                    key: None,
                    code: ConfigurationDiagnosticCode::InvalidValue,
                    message: "persisted configuration is not canonical".to_owned(),
                });
            }

            let changed_keys = changed_document_keys(persisted.scope, None, validated.document());
            let source_fingerprint = fingerprint(persisted.canonical_toml.as_bytes());
            let record = LastKnownGoodDocument {
                scope: persisted.scope,
                path: persisted.path,
                canonical_toml: persisted.canonical_toml,
                activated_generation: persisted.activated_generation,
                changed_keys: changed_keys.clone(),
            };
            maximum_generation = maximum_generation.max(record.activated_generation);
            changes_by_generation.insert(record.activated_generation, changed_keys);
            scopes.insert(
                record.scope,
                ActiveScope {
                    document: validated.document,
                    record,
                    source_fingerprint,
                },
            );
        }

        let effective = resolve_effective(&scopes, &managed_ceilings, &security_constraints);
        Ok(Self {
            scopes,
            managed_ceilings,
            security_constraints,
            snapshot: EffectiveConfigurationSnapshot {
                generation: maximum_generation,
                configuration: Arc::new(effective),
            },
            changes_by_generation,
            diagnostics: Vec::new(),
            self_write_fingerprints: BTreeMap::new(),
            stable_read_policy: StableReadPolicy::default(),
        })
    }

    pub fn with_stable_read_policy(mut self, policy: StableReadPolicy) -> Self {
        self.stable_read_policy = policy;
        self
    }

    pub fn snapshot(&self) -> EffectiveConfigurationSnapshot {
        self.snapshot.clone()
    }

    /// Raises the process-local generation floor before a coordinator allocates
    /// the next globally serialized scope generation. This does not activate a
    /// document or alter the current effective configuration.
    pub(crate) fn raise_generation_floor(&mut self, floor: u64) {
        if floor > self.snapshot.generation {
            self.snapshot = EffectiveConfigurationSnapshot {
                generation: floor,
                configuration: Arc::clone(&self.snapshot.configuration),
            };
        }
    }

    pub fn diagnostics(&self) -> &[ConfigurationDiagnostic] {
        &self.diagnostics
    }

    pub fn take_diagnostics(&mut self) -> Vec<ConfigurationDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Records a fixed, value-free persistence failure while retaining the
    /// current last-known-good snapshot. Cohesive service coordinators use
    /// this when durable publication fails after a valid external edit.
    pub fn record_persistence_failure(
        &mut self,
        scope: ConfigurationScope,
        path: impl Into<PathBuf>,
    ) {
        self.diagnostics.push(ConfigurationDiagnostic {
            scope,
            path: path.into(),
            key: None,
            code: ConfigurationDiagnosticCode::Io,
            message: "validated configuration could not be durably activated".to_owned(),
        });
    }

    pub(crate) fn record_diagnostic(&mut self, diagnostic: ConfigurationDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn last_known_good(&self, scope: ConfigurationScope) -> Option<&LastKnownGoodDocument> {
        self.scopes.get(&scope).map(|active| &active.record)
    }

    pub fn last_known_good_records(&self) -> Vec<LastKnownGoodDocument> {
        self.scopes
            .values()
            .map(|active| active.record.clone())
            .collect()
    }

    /// Reconcile a complete external document after the caller's debounce.
    /// Invalid input is recorded and leaves every active scope untouched.
    pub fn reconcile_external_text(
        &mut self,
        scope: ConfigurationScope,
        path: impl Into<PathBuf>,
        text: &str,
    ) -> ConfigurationUpdate {
        let path = path.into();
        let fingerprint = fingerprint(text.as_bytes());
        if self.self_write_fingerprints.get(&path) == Some(&fingerprint) {
            self.self_write_fingerprints.remove(&path);
            return ConfigurationUpdate::DeduplicatedSelfWrite {
                snapshot: self.snapshot(),
            };
        }

        match validate_scope_document(scope, &path, text) {
            Ok(validated) => {
                let (document, canonical_toml) = validated.into_parts();
                self.activate(scope, path, document, canonical_toml, fingerprint)
            }
            Err(diagnostic) => self.reject(diagnostic),
        }
    }

    /// Read twice (or more) until file bytes and metadata stabilize, then
    /// reconcile the full scope. Watchers should call this after debouncing.
    pub fn reload_scope(
        &mut self,
        scope: ConfigurationScope,
        path: impl Into<PathBuf>,
    ) -> ConfigurationUpdate {
        let path = path.into();
        match stable_read(&path, self.stable_read_policy) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => self.reconcile_external_text(scope, path, &text),
                Err(_) => self.reject(ConfigurationDiagnostic {
                    scope,
                    path,
                    key: None,
                    code: ConfigurationDiagnosticCode::InvalidToml,
                    message: "configuration must be valid UTF-8 TOML".to_owned(),
                }),
            },
            Err(StableReadFailure::Unstable) => self.reject(ConfigurationDiagnostic {
                scope,
                path,
                key: None,
                code: ConfigurationDiagnosticCode::UnstableRead,
                message: "configuration did not become stable before the read bound".to_owned(),
            }),
            Err(StableReadFailure::Io) => self.reject(ConfigurationDiagnostic {
                scope,
                path,
                key: None,
                code: ConfigurationDiagnosticCode::Io,
                message: "configuration could not be read".to_owned(),
            }),
            Err(StableReadFailure::TooLarge) => self.reject(ConfigurationDiagnostic {
                scope,
                path,
                key: None,
                code: ConfigurationDiagnosticCode::SizeLimitExceeded,
                message: "configuration exceeds the stable-read byte limit".to_owned(),
            }),
        }
    }

    /// Validate and atomically write a UI edit. The base generation is checked
    /// before parsing or touching the filesystem.
    pub fn save_scope_text(
        &mut self,
        scope: ConfigurationScope,
        path: impl Into<PathBuf>,
        text: &str,
        base_generation: u64,
    ) -> Result<ConfigurationUpdate, ConfigurationError> {
        let path = path.into();
        if base_generation != self.snapshot.generation {
            return Err(ConfigurationError::StaleGeneration(
                self.stale_conflict(base_generation),
            ));
        }

        let (document, canonical_toml) = match validate_scope_document(scope, &path, text) {
            Ok(validated) => validated.into_parts(),
            Err(diagnostic) => return Ok(self.reject(diagnostic)),
        };

        let previous = self
            .scopes
            .get(&scope)
            .map(|active| active.document.clone());
        let expected_disk = self
            .scopes
            .get(&scope)
            .filter(|active| active.record.path == path)
            .map(|active| ExpectedDiskState::Fingerprint(active.source_fingerprint))
            .unwrap_or(ExpectedDiskState::Missing);
        if previous.as_ref() != Some(&document) && self.snapshot.generation.checked_add(1).is_none()
        {
            return Err(ConfigurationError::GenerationOverflow);
        }

        atomic_write(
            &path,
            canonical_toml.as_bytes(),
            AtomicWriteGuard {
                expected_disk,
                stable_read_policy: self.stable_read_policy,
                scope,
                previous: previous.as_ref(),
                base_generation,
                active_generation: self.snapshot.generation,
            },
        )?;
        let source_fingerprint = fingerprint(canonical_toml.as_bytes());
        self.self_write_fingerprints
            .insert(path.clone(), source_fingerprint);
        Ok(self.activate(scope, path, document, canonical_toml, source_fingerprint))
    }

    pub fn changed_keys_since(&self, base_generation: u64) -> BTreeSet<String> {
        let Some(first_generation) = base_generation.checked_add(1) else {
            return BTreeSet::new();
        };
        self.changes_by_generation
            .range(first_generation..)
            .flat_map(|(_, keys)| keys.iter().cloned())
            .collect()
    }

    fn stale_conflict(&self, base_generation: u64) -> StaleGenerationConflict {
        let mut changed_keys = self.changed_keys_since(base_generation);
        if changed_keys.is_empty() && base_generation != self.snapshot.generation {
            changed_keys.insert("generation".to_owned());
        }
        StaleGenerationConflict {
            base_generation,
            active_generation: self.snapshot.generation,
            changed_keys,
        }
    }

    fn reject(&mut self, diagnostic: ConfigurationDiagnostic) -> ConfigurationUpdate {
        self.diagnostics.push(diagnostic.clone());
        ConfigurationUpdate::Rejected {
            diagnostic,
            snapshot: self.snapshot(),
        }
    }

    fn activate(
        &mut self,
        scope: ConfigurationScope,
        path: PathBuf,
        document: ConfigurationDocument,
        canonical_toml: String,
        source_fingerprint: [u8; 32],
    ) -> ConfigurationUpdate {
        if let Some(active) = self
            .scopes
            .get_mut(&scope)
            .filter(|active| active.document == document)
        {
            active.record.path = path;
            active.record.canonical_toml = canonical_toml;
            active.source_fingerprint = source_fingerprint;
            return ConfigurationUpdate::Unchanged {
                snapshot: self.snapshot(),
            };
        }

        let changed_keys = changed_document_keys(
            scope,
            self.scopes.get(&scope).map(|active| &active.document),
            &document,
        );
        let Some(next_generation) = self.snapshot.generation.checked_add(1) else {
            return self.reject(ConfigurationDiagnostic {
                scope,
                path,
                key: Some("generation".to_owned()),
                code: ConfigurationDiagnosticCode::GenerationOverflow,
                message: "configuration generation cannot advance beyond u64::MAX".to_owned(),
            });
        };
        let record = LastKnownGoodDocument {
            scope,
            path,
            canonical_toml,
            activated_generation: next_generation,
            changed_keys: changed_keys.clone(),
        };
        self.scopes.insert(
            scope,
            ActiveScope {
                document,
                record: record.clone(),
                source_fingerprint,
            },
        );

        let effective = resolve_effective(
            &self.scopes,
            &self.managed_ceilings,
            &self.security_constraints,
        );
        self.snapshot = EffectiveConfigurationSnapshot {
            generation: next_generation,
            configuration: Arc::new(effective),
        };
        self.changes_by_generation
            .insert(next_generation, changed_keys);
        ConfigurationUpdate::Activated {
            snapshot: self.snapshot(),
            record,
        }
    }
}

/// Compensates a UI file replacement after its SQLite LKG publication fails.
/// The replacement is conditional on the failed write still owning the path;
/// a newer external edit is preserved and reported as a conflict.
pub(crate) fn compensate_scope_write(
    scope: ConfigurationScope,
    path: &Path,
    failed_bytes: &[u8],
    previous_bytes: Option<&[u8]>,
    base_generation: u64,
    active_generation: u64,
) -> Result<(), ConfigurationError> {
    let expected_disk = ExpectedDiskState::Fingerprint(fingerprint(failed_bytes));
    match previous_bytes {
        Some(previous_bytes) => atomic_write(
            path,
            previous_bytes,
            AtomicWriteGuard {
                expected_disk,
                stable_read_policy: StableReadPolicy::default(),
                scope,
                previous: None,
                base_generation,
                active_generation,
            },
        ),
        None => {
            if let Err(observed) =
                verify_disk_base(path, expected_disk, StableReadPolicy::default())
            {
                return Err(external_replacement_error(
                    scope,
                    path,
                    None,
                    observed,
                    base_generation,
                    active_generation,
                ));
            }
            fs::remove_file(path).map_err(|source| ConfigurationError::Io {
                operation: "remove failed configuration",
                path: path.to_path_buf(),
                source,
            })?;
            let parent = path.parent().ok_or_else(|| ConfigurationError::Io {
                operation: "resolve parent",
                path: path.to_path_buf(),
                source: io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory"),
            })?;
            sync_parent(parent)
        }
    }
}

/// Rehydrates a missing editable file from its validated SQLite LKG copy. The
/// create remains conditional so a concurrent external editor is never
/// overwritten during restart recovery.
pub(crate) fn recover_missing_scope_file(
    scope: ConfigurationScope,
    path: &Path,
    canonical_toml: &str,
    generation: u64,
) -> Result<(), ConfigurationError> {
    atomic_write(
        path,
        canonical_toml.as_bytes(),
        AtomicWriteGuard {
            expected_disk: ExpectedDiskState::Missing,
            stable_read_policy: StableReadPolicy::default(),
            scope,
            previous: None,
            base_generation: generation,
            active_generation: generation,
        },
    )
}

/// Performs the same stable, bounded read used by watcher reloads without
/// activating the candidate. Coordinators can therefore publish a canonical
/// LKG transaction before changing in-memory effective state.
pub(crate) fn stable_scope_text(
    scope: ConfigurationScope,
    path: &Path,
) -> Result<String, ConfigurationDiagnostic> {
    match stable_read(path, StableReadPolicy::default()) {
        Ok(bytes) => String::from_utf8(bytes).map_err(|_| ConfigurationDiagnostic {
            scope,
            path: path.to_path_buf(),
            key: None,
            code: ConfigurationDiagnosticCode::InvalidToml,
            message: "configuration must be valid UTF-8 TOML".to_owned(),
        }),
        Err(StableReadFailure::Unstable) => Err(ConfigurationDiagnostic {
            scope,
            path: path.to_path_buf(),
            key: None,
            code: ConfigurationDiagnosticCode::UnstableRead,
            message: "configuration did not become stable before the read bound".to_owned(),
        }),
        Err(StableReadFailure::Io) => Err(ConfigurationDiagnostic {
            scope,
            path: path.to_path_buf(),
            key: None,
            code: ConfigurationDiagnosticCode::Io,
            message: "configuration could not be read".to_owned(),
        }),
        Err(StableReadFailure::TooLarge) => Err(ConfigurationDiagnostic {
            scope,
            path: path.to_path_buf(),
            key: None,
            code: ConfigurationDiagnosticCode::SizeLimitExceeded,
            message: "configuration exceeds the stable-read byte limit".to_owned(),
        }),
    }
}

/// Strict, bounded, and side-effect-free validation for archive inspection,
/// restart recovery, and other pre-activation gates.
pub fn validate_scope_document(
    scope: ConfigurationScope,
    path: impl Into<PathBuf>,
    text: &str,
) -> Result<ValidatedConfigurationDocument, ConfigurationDiagnostic> {
    let path = path.into();
    let (document, canonical_toml) = parse_and_validate(scope, &path, text)?;
    Ok(ValidatedConfigurationDocument {
        scope,
        path,
        canonical_toml,
        document,
    })
}

fn parse_and_validate(
    scope: ConfigurationScope,
    path: &Path,
    text: &str,
) -> Result<(ConfigurationDocument, String), ConfigurationDiagnostic> {
    let diagnostic = |code, key: Option<String>, message: &str| ConfigurationDiagnostic {
        scope,
        path: path.to_path_buf(),
        key,
        code,
        message: message.to_owned(),
    };

    if text.len() > MAX_CONFIGURATION_BYTES {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::SizeLimitExceeded,
            None,
            "configuration exceeds the document byte limit",
        ));
    }

    let first_key = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .and_then(|line| line.split_once('=').map(|(key, _)| key.trim()));
    if first_key != Some("schema_version") {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::UnsupportedSchema,
            Some("schema_version".to_owned()),
            "schema_version must be the first configuration key",
        ));
    }

    let value: toml::Value = toml::from_str(text).map_err(|_| {
        diagnostic(
            ConfigurationDiagnosticCode::InvalidToml,
            None,
            "configuration failed strict TOML syntax validation",
        )
    })?;
    if let Some(forbidden_key) = find_forbidden_key(&value, "") {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::ForbiddenContent,
            Some(forbidden_key),
            "configuration contains content owned by a protected subsystem",
        ));
    }

    let document: ConfigurationDocument = toml::from_str(text).map_err(|_| {
        diagnostic(
            ConfigurationDiagnosticCode::InvalidToml,
            None,
            "configuration failed strict TOML schema validation",
        )
    })?;

    if document.schema_version != CONFIGURATION_SCHEMA_VERSION {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::UnsupportedSchema,
            Some("schema_version".to_owned()),
            "configuration schema version is unsupported",
        ));
    }

    if let Some(key) = value
        .as_table()
        .and_then(|table| table.keys().find(|key| !scope.allows_key(key)))
    {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::InvalidScopeKey,
            Some(key.to_owned()),
            "configuration key is not valid for this scope",
        ));
    }

    validate_document(scope, path, &document)?;
    let canonical_toml = toml::to_string_pretty(&document).map_err(|_| {
        diagnostic(
            ConfigurationDiagnosticCode::InvalidToml,
            None,
            "configuration could not be canonicalized",
        )
    })?;
    if canonical_toml.len() > MAX_CONFIGURATION_BYTES {
        return Err(diagnostic(
            ConfigurationDiagnosticCode::SizeLimitExceeded,
            None,
            "canonical configuration exceeds the document byte limit",
        ));
    }
    Ok((document, canonical_toml))
}

fn validate_document(
    scope: ConfigurationScope,
    path: &Path,
    document: &ConfigurationDocument,
) -> Result<(), ConfigurationDiagnostic> {
    let invalid = |key: &str, message: &str| ConfigurationDiagnostic {
        scope,
        path: path.to_path_buf(),
        key: Some(key.to_owned()),
        code: ConfigurationDiagnosticCode::InvalidValue,
        message: message.to_owned(),
    };

    for (key, value) in [
        ("model_route", document.model_route.as_deref()),
        ("default_runtime", document.default_runtime.as_deref()),
        (
            "default_environment",
            document.default_environment.as_deref(),
        ),
    ] {
        if value.is_some_and(|value| !is_safe_reference_identifier(value)) {
            return Err(invalid(
                key,
                "configuration reference must use non-secret identifier syntax",
            ));
        }
    }

    if let Some(limits) = &document.requested_limits {
        if limits.parallel_actions == Some(0) {
            return Err(invalid(
                "requested_limits.parallel_actions",
                "requested parallel actions must be positive",
            ));
        }
        if limits.output_bytes == Some(0) {
            return Err(invalid(
                "requested_limits.output_bytes",
                "requested output bytes must be positive",
            ));
        }
    }

    for name in document.shell_environment_allowlist.iter().flatten() {
        let valid = !name.is_empty()
            && name.len() <= 128
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && !name.as_bytes()[0].is_ascii_digit();
        if !valid {
            return Err(invalid(
                "shell_environment_allowlist",
                "shell environment names must use portable identifier syntax",
            ));
        }
    }

    for key in document.approval_overrides.keys() {
        if key.trim().is_empty() || key.len() > 160 {
            return Err(invalid(
                "approval_overrides",
                "approval override keys must be non-empty and bounded",
            ));
        }
    }

    const UPDATE_CHANNELS: [&str; 5] = ["application", "runtimes", "plugins", "skills", "mcp"];
    if let Some(key) = document
        .updates
        .keys()
        .find(|key| !UPDATE_CHANNELS.contains(&key.as_str()))
    {
        return Err(invalid(
            &format!("updates.{key}"),
            "update channel is unsupported",
        ));
    }

    if document
        .diagnostics
        .as_ref()
        .and_then(|preferences| preferences.retention_days)
        == Some(0)
    {
        return Err(invalid(
            "diagnostics.retention_days",
            "diagnostic retention must be positive",
        ));
    }

    Ok(())
}

fn is_safe_reference_identifier(value: &str) -> bool {
    let syntax_is_valid = !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'@')
        });
    syntax_is_valid && !looks_credential_like(value)
}

fn looks_credential_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    let credential_segment = normalized
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|segment| {
            matches!(
                segment,
                "secret" | "secrets" | "password" | "token" | "credential" | "credentials"
            )
        });
    credential_segment
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("private_key")
        || normalized.contains("authorization")
        || normalized.contains("sk_live")
        || normalized.contains("sk-live")
        || normalized.contains("sk_test")
        || normalized.contains("sk-test")
        || normalized.contains("ghp_")
        || normalized.contains("github_pat_")
        || normalized.contains("xoxb-")
        || normalized.contains("xoxp-")
        || normalized.starts_with("eyj") && normalized.matches('.').count() == 2
}

fn find_forbidden_key(value: &toml::Value, prefix: &str) -> Option<String> {
    match value {
        toml::Value::Table(table) => {
            for (key, child) in table {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                let normalized = key.to_ascii_lowercase().replace('-', "_");
                if is_forbidden_key(&normalized) {
                    return Some(path);
                }
                if let Some(found) = find_forbidden_key(child, &path) {
                    return Some(found);
                }
            }
            None
        }
        toml::Value::Array(values) => values
            .iter()
            .find_map(|child| find_forbidden_key(child, prefix)),
        _ => None,
    }
}

fn is_forbidden_key(normalized: &str) -> bool {
    matches!(
        normalized,
        "secret"
            | "secrets"
            | "provider_secret"
            | "provider_secrets"
            | "api_key"
            | "client_secret"
            | "access_token"
            | "refresh_token"
            | "bearer_token"
            | "password"
            | "authorization"
            | "private_key"
            | "credential"
            | "credentials"
            | "cookie"
            | "cookies"
            | "chat_history"
            | "history"
            | "package_manifest"
            | "package_manifests"
            | "managed_ceiling"
            | "managed_ceilings"
    ) || normalized.ends_with("_secret")
        || normalized.ends_with("_password")
        || normalized.ends_with("_api_key")
        || normalized.ends_with("_access_token")
        || normalized.ends_with("_refresh_token")
        || normalized.ends_with("_private_key")
}

fn resolve_effective(
    scopes: &BTreeMap<ConfigurationScope, ActiveScope>,
    ceilings: &ManagedCeilings,
    constraints: &SecurityConstraints,
) -> EffectiveConfiguration {
    let mut effective = EffectiveConfiguration::default();
    for scope in ConfigurationScope::PRECEDENCE {
        if let Some(active) = scopes.get(&scope) {
            merge_document(&mut effective, &active.document);
        }
    }
    apply_policies(effective, ceilings, constraints)
}

fn merge_document(effective: &mut EffectiveConfiguration, document: &ConfigurationDocument) {
    if let Some(value) = document.restore_last_workspace {
        effective.restore_last_workspace = value;
    }
    if let Some(value) = &document.model_route {
        effective.model_route = Some(value.clone());
    }
    if let Some(value) = &document.default_runtime {
        effective.default_runtime = Some(value.clone());
    }
    if let Some(value) = &document.default_environment {
        effective.default_environment = Some(value.clone());
    }
    if let Some(value) = document.default_approval_preset {
        effective.default_approval_preset = value;
    }
    if let Some(value) = document.inherit_shell_environment {
        effective.inherit_shell_environment = value;
    }
    if let Some(value) = &document.shell_environment_allowlist {
        effective.shell_environment_allowlist = value.clone();
    }
    if let Some(value) = document.browser_environment {
        effective.browser_environment = value;
    }
    if let Some(limits) = &document.requested_limits {
        if let Some(value) = limits.parallel_actions {
            effective.parallel_actions = Some(value);
        }
        if let Some(value) = limits.output_bytes {
            effective.output_bytes = Some(value);
        }
    }
    effective
        .approval_overrides
        .extend(document.approval_overrides.clone());
    effective.updates.extend(document.updates.clone());
    if let Some(preferences) = &document.diagnostics {
        if let Some(value) = preferences.retention_days {
            effective.diagnostics_retention_days = Some(value);
        }
        if let Some(value) = &preferences.redaction_patterns {
            effective.diagnostic_redaction_patterns = value.clone();
        }
    }
}

fn apply_policies(
    mut effective: EffectiveConfiguration,
    ceilings: &ManagedCeilings,
    constraints: &SecurityConstraints,
) -> EffectiveConfiguration {
    effective.parallel_actions = effective
        .parallel_actions
        .map(|value| value.min(ceilings.parallel_actions));
    effective.output_bytes = effective
        .output_bytes
        .map(|value| value.min(ceilings.output_bytes));

    if let Some(forced) = constraints.forced_approval_preset {
        effective.default_approval_preset = forced;
        for preset in effective.approval_overrides.values_mut() {
            *preset = forced;
        }
    }
    for (key, preset) in &constraints.forced_approval_overrides {
        effective
            .approval_overrides
            .entry(key.clone())
            .and_modify(|current| *current = *preset)
            .or_insert(*preset);
    }

    if !constraints.shell_environment_allowed {
        effective.inherit_shell_environment = false;
        effective.shell_environment_allowlist.clear();
    } else if let Some(permitted) = &constraints.permitted_shell_environment_names {
        effective
            .shell_environment_allowlist
            .retain(|name| permitted.contains(name));
    }
    effective
}

fn changed_document_keys(
    scope: ConfigurationScope,
    previous: Option<&ConfigurationDocument>,
    current: &ConfigurationDocument,
) -> BTreeSet<String> {
    let previous = previous
        .and_then(|document| toml::Value::try_from(document).ok())
        .unwrap_or_else(|| toml::Value::Table(toml::Table::new()));
    let current =
        toml::Value::try_from(current).unwrap_or_else(|_| toml::Value::Table(toml::Table::new()));
    let mut keys = BTreeSet::new();
    collect_changed_keys(&previous, &current, "", &mut keys);
    keys.into_iter()
        .filter(|key| key != "schema_version")
        .map(|key| format!("{}.{key}", scope.as_str()))
        .collect()
}

fn collect_changed_keys(
    previous: &toml::Value,
    current: &toml::Value,
    prefix: &str,
    keys: &mut BTreeSet<String>,
) {
    match (previous, current) {
        (toml::Value::Table(previous), toml::Value::Table(current)) => {
            let names: BTreeSet<_> = previous.keys().chain(current.keys()).collect();
            for name in names {
                let path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}.{name}")
                };
                match (previous.get(name), current.get(name)) {
                    (Some(left), Some(right)) => collect_changed_keys(left, right, &path, keys),
                    _ => {
                        keys.insert(path);
                    }
                }
            }
        }
        // Lists replace as a unit, so conflicts identify the list key rather
        // than unstable numeric indices.
        (toml::Value::Array(_), toml::Value::Array(_)) if previous != current => {
            keys.insert(prefix.to_owned());
        }
        _ if previous != current => {
            keys.insert(prefix.to_owned());
        }
        _ => {}
    }
}

fn fingerprint(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

enum StableReadFailure {
    Io,
    Unstable,
    TooLarge,
}

fn stable_read(path: &Path, policy: StableReadPolicy) -> Result<Vec<u8>, StableReadFailure> {
    let attempts = policy.max_attempts.max(2);
    let mut previous: Option<(Vec<u8>, u64, Option<std::time::SystemTime>)> = None;
    for attempt in 0..attempts {
        let observation = read_bounded_observation(path)?;
        if previous.as_ref() == Some(&observation) {
            return Ok(observation.0);
        }
        previous = Some(observation);
        if attempt + 1 < attempts {
            thread::sleep(policy.retry_delay);
        }
    }
    Err(StableReadFailure::Unstable)
}

fn read_bounded_observation(
    path: &Path,
) -> Result<(Vec<u8>, u64, Option<std::time::SystemTime>), StableReadFailure> {
    let mut file = File::open(path).map_err(|_| StableReadFailure::Io)?;
    let initial = file.metadata().map_err(|_| StableReadFailure::Io)?;
    if initial.len() > MAX_CONFIGURATION_BYTES as u64 {
        return Err(StableReadFailure::TooLarge);
    }

    let mut bytes = Vec::with_capacity(initial.len() as usize);
    (&mut file)
        .take((MAX_CONFIGURATION_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| StableReadFailure::Io)?;
    if bytes.len() > MAX_CONFIGURATION_BYTES {
        return Err(StableReadFailure::TooLarge);
    }
    let final_metadata = file.metadata().map_err(|_| StableReadFailure::Io)?;
    if final_metadata.len() > MAX_CONFIGURATION_BYTES as u64 {
        return Err(StableReadFailure::TooLarge);
    }
    Ok((bytes, final_metadata.len(), final_metadata.modified().ok()))
}

#[derive(Clone, Copy)]
enum ExpectedDiskState {
    Missing,
    Fingerprint([u8; 32]),
}

enum DiskBaseMismatch {
    Missing,
    Bytes(Vec<u8>),
    Unreadable,
}

struct AtomicWriteGuard<'a> {
    expected_disk: ExpectedDiskState,
    stable_read_policy: StableReadPolicy,
    scope: ConfigurationScope,
    previous: Option<&'a ConfigurationDocument>,
    base_generation: u64,
    active_generation: u64,
}

fn atomic_write(
    path: &Path,
    bytes: &[u8],
    guard: AtomicWriteGuard<'_>,
) -> Result<(), ConfigurationError> {
    let parent = path.parent().ok_or_else(|| ConfigurationError::Io {
        operation: "resolve parent",
        path: path.to_path_buf(),
        source: io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory"),
    })?;
    fs::create_dir_all(parent).map_err(|source| ConfigurationError::Io {
        operation: "create parent",
        path: parent.to_path_buf(),
        source,
    })?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ConfigurationError::Io {
            operation: "resolve file name",
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::InvalidInput, "path has no UTF-8 file name"),
        })?;
    let temporary = parent.join(format!(".{file_name}.c4os-{}.tmp", Uuid::new_v4()));

    let write_result = (|| -> Result<(), ConfigurationError> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|source| ConfigurationError::Io {
                operation: "create temporary file",
                path: temporary.clone(),
                source,
            })?;
        file.write_all(bytes)
            .map_err(|source| ConfigurationError::Io {
                operation: "write temporary file",
                path: temporary.clone(),
                source,
            })?;
        file.sync_all().map_err(|source| ConfigurationError::Io {
            operation: "sync temporary file",
            path: temporary.clone(),
            source,
        })?;
        drop(file);

        if let Err(observed) = verify_disk_base(path, guard.expected_disk, guard.stable_read_policy)
        {
            return Err(external_replacement_error(
                guard.scope,
                path,
                guard.previous,
                observed,
                guard.base_generation,
                guard.active_generation,
            ));
        }

        fs::rename(&temporary, path).map_err(|source| ConfigurationError::Io {
            operation: "replace configuration",
            path: path.to_path_buf(),
            source,
        })?;
        sync_parent(parent)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn verify_disk_base(
    path: &Path,
    expected: ExpectedDiskState,
    stable_read_policy: StableReadPolicy,
) -> Result<(), DiskBaseMismatch> {
    match expected {
        ExpectedDiskState::Missing => match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err(DiskBaseMismatch::Unreadable),
            Ok(_) => match stable_read(path, stable_read_policy) {
                Ok(bytes) => Err(DiskBaseMismatch::Bytes(bytes)),
                Err(_) => Err(DiskBaseMismatch::Unreadable),
            },
        },
        ExpectedDiskState::Fingerprint(expected_fingerprint) => {
            match stable_read(path, stable_read_policy) {
                Ok(bytes) if fingerprint(&bytes) == expected_fingerprint => Ok(()),
                Ok(bytes) => Err(DiskBaseMismatch::Bytes(bytes)),
                Err(StableReadFailure::Io)
                    if fs::symlink_metadata(path)
                        .is_err_and(|error| error.kind() == io::ErrorKind::NotFound) =>
                {
                    Err(DiskBaseMismatch::Missing)
                }
                Err(_) => Err(DiskBaseMismatch::Unreadable),
            }
        }
    }
}

fn external_replacement_error(
    scope: ConfigurationScope,
    path: &Path,
    previous: Option<&ConfigurationDocument>,
    observed: DiskBaseMismatch,
    base_generation: u64,
    active_generation: u64,
) -> ConfigurationError {
    let mut changed_keys = match observed {
        DiskBaseMismatch::Bytes(bytes) => String::from_utf8(bytes)
            .ok()
            .and_then(|text| validate_scope_document(scope, path, &text).ok())
            .map(|validated| changed_document_keys(scope, previous, validated.document()))
            .unwrap_or_default(),
        DiskBaseMismatch::Missing | DiskBaseMismatch::Unreadable => BTreeSet::new(),
    };
    if changed_keys.is_empty() {
        changed_keys.insert(format!("{}.external_document", scope.as_str()));
    }
    ConfigurationError::ExternalReplacement(ExternalConfigurationConflict {
        base_generation,
        active_generation,
        changed_keys,
    })
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), ConfigurationError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| ConfigurationError::Io {
            operation: "sync parent directory",
            path: parent.to_path_buf(),
            source,
        })
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<(), ConfigurationError> {
    Ok(())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WatchedConfiguration {
    pub scope: ConfigurationScope,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationWatcherPlan {
    pub targets: Vec<WatchedConfiguration>,
    pub parent_directories: BTreeSet<PathBuf>,
}

impl ConfigurationWatcherPlan {
    pub fn new(targets: impl IntoIterator<Item = WatchedConfiguration>) -> io::Result<Self> {
        let targets: Vec<_> = targets.into_iter().collect();
        let mut parent_directories = BTreeSet::new();
        for target in &targets {
            let parent = target.path.parent().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "watch target has no parent")
            })?;
            parent_directories.insert(parent.to_path_buf());
        }
        Ok(Self {
            targets,
            parent_directories,
        })
    }
}

#[derive(Clone, Debug)]
pub enum ConfigurationWatcherNotice {
    Changed(Vec<WatchedConfiguration>),
    Error(String),
}

enum DebounceCommand {
    Changed(Vec<WatchedConfiguration>),
    Error(String),
    Shutdown,
}

/// Trailing-edge coalescer shared by the notify-backed watcher and deterministic
/// tests. A continuous event stream is still bounded to four debounce windows,
/// preventing replacement storms from starving configuration activation.
pub struct ConfigurationEventDebouncer {
    sender: mpsc::Sender<DebounceCommand>,
    worker: Option<thread::JoinHandle<()>>,
}

impl ConfigurationEventDebouncer {
    pub fn start<F>(debounce_window: Duration, callback: F) -> Self
    where
        F: FnMut(ConfigurationWatcherNotice) + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            debounce_configuration_events(receiver, debounce_window, callback)
        });
        Self {
            sender,
            worker: Some(worker),
        }
    }

    pub fn push_changed(&self, targets: impl IntoIterator<Item = WatchedConfiguration>) -> bool {
        self.sender
            .send(DebounceCommand::Changed(targets.into_iter().collect()))
            .is_ok()
    }

    pub fn push_error(&self, message: impl Into<String>) -> bool {
        self.sender
            .send(DebounceCommand::Error(message.into()))
            .is_ok()
    }

    fn command_sender(&self) -> mpsc::Sender<DebounceCommand> {
        self.sender.clone()
    }
}

impl Drop for ConfigurationEventDebouncer {
    fn drop(&mut self) {
        let _ = self.sender.send(DebounceCommand::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn debounce_configuration_events<F>(
    receiver: mpsc::Receiver<DebounceCommand>,
    debounce_window: Duration,
    mut callback: F,
) where
    F: FnMut(ConfigurationWatcherNotice),
{
    loop {
        match receiver.recv() {
            Ok(DebounceCommand::Changed(targets)) => {
                let mut pending: BTreeSet<_> = targets.into_iter().collect();
                let started = Instant::now();
                let maximum_deadline =
                    started + debounce_window.saturating_mul(MAX_DEBOUNCE_WINDOWS);
                let mut quiet_deadline = started + debounce_window;
                loop {
                    if Instant::now() >= maximum_deadline {
                        publish_pending(&mut pending, &mut callback);
                        break;
                    }
                    let deadline = quiet_deadline.min(maximum_deadline);
                    let timeout = deadline.saturating_duration_since(Instant::now());
                    match receiver.recv_timeout(timeout) {
                        Ok(DebounceCommand::Changed(targets)) => {
                            pending.extend(targets);
                            quiet_deadline = Instant::now() + debounce_window;
                        }
                        Ok(DebounceCommand::Error(message)) => {
                            callback(ConfigurationWatcherNotice::Error(message));
                        }
                        Ok(DebounceCommand::Shutdown)
                        | Err(mpsc::RecvTimeoutError::Disconnected) => {
                            publish_pending(&mut pending, &mut callback);
                            return;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            publish_pending(&mut pending, &mut callback);
                            break;
                        }
                    }
                }
            }
            Ok(DebounceCommand::Error(message)) => {
                callback(ConfigurationWatcherNotice::Error(message));
            }
            Ok(DebounceCommand::Shutdown) | Err(_) => return,
        }
    }
}

fn publish_pending<F>(pending: &mut BTreeSet<WatchedConfiguration>, callback: &mut F)
where
    F: FnMut(ConfigurationWatcherNotice),
{
    if !pending.is_empty() {
        callback(ConfigurationWatcherNotice::Changed(
            std::mem::take(pending).into_iter().collect(),
        ));
    }
}

/// Keeps `notify` alive while parent directories are monitored. Replacement
/// bursts are bounded, coalesced, and target-deduplicated before the callback;
/// the callback can therefore perform one stable full-scope reconciliation.
pub struct ParentDirectoryConfigurationWatcher {
    _native_watcher: RecommendedWatcher,
    _poll_watcher: PollWatcher,
    _debouncer: ConfigurationEventDebouncer,
    plan: ConfigurationWatcherPlan,
}

impl ParentDirectoryConfigurationWatcher {
    pub fn start<F>(plan: ConfigurationWatcherPlan, callback: F) -> Result<Self, ConfigurationError>
    where
        F: FnMut(ConfigurationWatcherNotice) + Send + 'static,
    {
        Self::start_debounced(plan, DEFAULT_WATCH_DEBOUNCE, callback)
    }

    pub fn start_debounced<F>(
        plan: ConfigurationWatcherPlan,
        debounce_window: Duration,
        callback: F,
    ) -> Result<Self, ConfigurationError>
    where
        F: FnMut(ConfigurationWatcherNotice) + Send + 'static,
    {
        let debouncer = ConfigurationEventDebouncer::start(debounce_window, callback);
        let event_sender = debouncer.command_sender();
        let native_plan = plan.clone();
        let native_sender = event_sender.clone();
        let mut native_watcher = notify::recommended_watcher(move |event| {
            forward_configuration_event(event, &native_plan, &native_sender);
        })?;
        // Some supported filesystems and sandboxed macOS execution contexts do
        // not deliver native events reliably. A bounded parent-directory poll
        // closes that observability gap and shares the same deduplication path.
        let poll_plan = plan.clone();
        let mut poll_watcher = PollWatcher::new(
            move |event| forward_configuration_event(event, &poll_plan, &event_sender),
            // PollWatcher rounds mtimes to whole seconds. Configuration files
            // are bounded and live alone at these parent-directory roots, so
            // content comparison is required to observe rapid atomic replaces
            // whose old and new metadata land in the same second.
            Config::default()
                .with_poll_interval(WATCH_POLL_FALLBACK_INTERVAL)
                .with_compare_contents(true),
        )?;

        for parent in &plan.parent_directories {
            native_watcher.watch(parent, RecursiveMode::NonRecursive)?;
            poll_watcher.watch(parent, RecursiveMode::NonRecursive)?;
        }
        Ok(Self {
            _native_watcher: native_watcher,
            _poll_watcher: poll_watcher,
            _debouncer: debouncer,
            plan,
        })
    }

    pub fn plan(&self) -> &ConfigurationWatcherPlan {
        &self.plan
    }
}

fn forward_configuration_event(
    event: notify::Result<Event>,
    plan: &ConfigurationWatcherPlan,
    sender: &mpsc::Sender<DebounceCommand>,
) {
    let event = match event {
        Ok(event) => event,
        Err(error) => {
            let _ = sender.send(DebounceCommand::Error(error.to_string()));
            return;
        }
    };
    let mut affected = Vec::new();
    for target in &plan.targets {
        let Some(parent) = target.path.parent() else {
            continue;
        };
        let matches = event.paths.iter().any(|changed| {
            watch_paths_equivalent(changed, &target.path)
                || watch_paths_equivalent(changed, parent)
                || changed
                    .parent()
                    .is_some_and(|changed_parent| watch_paths_equivalent(changed_parent, parent))
        });
        if matches {
            affected.push(target.clone());
        }
    }
    if !affected.is_empty() {
        let _ = sender.send(DebounceCommand::Changed(affected));
    }
}

/// Filesystem event backends may report the physical path even when a caller
/// registered a lexical alias (macOS commonly reports `/private/var/...` for a
/// `/var/...` target). Retain the caller-owned target path, but match against
/// both lexical and resolved identities so those events are not silently lost.
fn watch_paths_equivalent(left: &Path, right: &Path) -> bool {
    let left_identities = watch_path_identities(left);
    let right_identities = watch_path_identities(right);
    left_identities
        .iter()
        .any(|identity| right_identities.contains(identity))
}

fn watch_path_identities(path: &Path) -> BTreeSet<PathBuf> {
    let mut identities = BTreeSet::from([path.to_path_buf()]);
    if let Ok(resolved) = fs::canonicalize(path) {
        identities.insert(resolved);
    }
    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name())
        && let Ok(resolved_parent) = fs::canonicalize(parent)
    {
        identities.insert(resolved_parent.join(file_name));
    }
    identities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn watcher_matching_accepts_lexical_and_physical_path_aliases() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("temporary watcher root");
        let physical_parent = temp.path().join("physical");
        let alias_parent = temp.path().join("alias");
        fs::create_dir_all(&physical_parent).expect("physical watcher parent");
        symlink(&physical_parent, &alias_parent).expect("watcher parent alias");

        let lexical_target = alias_parent.join("config.toml");
        let physical_event = physical_parent.join("config.toml");
        assert!(watch_paths_equivalent(&lexical_target, &physical_event));
    }
}
