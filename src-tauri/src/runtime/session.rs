//! Rust-authoritative Chat Session, immutable User Turn, and Run Attempt state.
//!
//! The service deliberately depends on a small compare-and-swap repository
//! contract. A repository must commit one complete [`SessionRecord`] per
//! transition so first-submit promotion, immutable bindings, and attempt
//! lifecycle changes cannot be partially persisted.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::Mutex;
use thiserror::Error;

use crate::mcp::McpTurnSnapshot;

use crate::runtime::capability::{
    CapabilityDescriptor, CapabilityEvidence, CapabilityKey, CapabilityLayer, CapabilityState,
    NumericCapabilityKey,
};

pub const SESSION_SCHEMA_VERSION: u16 = 3;
pub const MAX_SESSION_IDENTIFIER_BYTES: usize = 160;
pub const MAX_SESSION_TEXT_BYTES: usize = 1_048_576;
pub const MAX_REPLY_SOURCE_EXCERPT_BYTES: usize = 256 * 1_024;
/// Accepted Chat titles occupy at most 48 visible characters. A truncated
/// title reserves the final character for the ellipsis so the complete value
/// remains inside that bound.
pub const MAX_SESSION_TITLE_CHARS: usize = 48;
pub const MAX_ATTACHMENTS_PER_TURN: usize = 64;
pub const MAX_SKILL_CONTEXTS_PER_TURN: usize = 16;
pub const MAX_SKILL_CONTEXT_BYTES: usize = 512 * 1024;
pub const MAX_SKILL_CONTEXT_REFERENCES: usize = 128;
pub const MAX_RESOURCES_PER_SNAPSHOT: usize = 512;
pub const MAX_CAPABILITIES_PER_SNAPSHOT: usize = 512;
pub const MAX_RUN_EVENTS_PER_ATTEMPT: usize = 4_096;
pub const MAX_RUN_EVENT_BYTES: usize = 64 * 1_024;
pub const MAX_SIDE_EFFECTS_PER_ATTEMPT: usize = 256;
const MAX_CAPABILITY_CANONICAL_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeKind {
    OpenCode,
    Pi,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AdapterBinding {
    pub adapter_id: String,
    pub adapter_version: String,
    pub native_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExecutionEnvironmentBinding {
    pub environment_id: String,
    pub environment_kind: String,
    pub host_alias: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ModelRouteSnapshot {
    pub route_id: String,
    pub provider_id: String,
    pub endpoint_id: String,
    pub model_id: String,
    pub model_revision: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConfigurationSnapshot {
    pub snapshot_id: String,
    pub version: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResourceSnapshot {
    pub snapshot_id: String,
    pub version: u64,
    pub sha256: String,
    pub resource_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
    Unknown,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CapabilityStateSnapshot {
    pub capability_id: String,
    pub support: CapabilitySupport,
    pub source_id: String,
    pub constraint_id: Option<String>,
    pub reason_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CapabilitySnapshot {
    pub snapshot_id: String,
    pub version: u64,
    pub sha256: String,
    pub capabilities: Vec<CapabilityStateSnapshot>,
    /// Exact canonical effective descriptor for every current run snapshot.
    /// The option preserves strict parsing of older records so validation can
    /// reject them as unsupported instead of silently inventing authority.
    pub effective_descriptor: Option<CapabilityDescriptor>,
}

/// Immutable identity captured when a provisional Chat is first promoted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SessionBinding {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub runtime_id: String,
    pub runtime_kind: RuntimeKind,
    pub adapter: AdapterBinding,
    pub environment: ExecutionEnvironmentBinding,
    pub initial_model_route: ModelRouteSnapshot,
    pub initial_configuration: ConfigurationSnapshot,
    pub initial_resources: ResourceSnapshot,
    pub initial_capabilities: CapabilitySnapshot,
    pub bound_at_ms: u64,
}

/// Exact run-scoped context. Retry may capture newer approved model,
/// configuration, resource, and capability snapshots but may not change the
/// Chat's workspace, Project, runtime, adapter/native version, or environment.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AttemptContextSnapshot {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub runtime_id: String,
    pub runtime_kind: RuntimeKind,
    pub adapter: AdapterBinding,
    pub environment: ExecutionEnvironmentBinding,
    pub model_route: ModelRouteSnapshot,
    pub configuration: ConfigurationSnapshot,
    pub resources: ResourceSnapshot,
    pub capabilities: CapabilitySnapshot,
}

impl AttemptContextSnapshot {
    pub fn from_binding(binding: &SessionBinding) -> Self {
        Self {
            workspace_id: binding.workspace_id.clone(),
            project_id: binding.project_id.clone(),
            runtime_id: binding.runtime_id.clone(),
            runtime_kind: binding.runtime_kind,
            adapter: binding.adapter.clone(),
            environment: binding.environment.clone(),
            model_route: binding.initial_model_route.clone(),
            configuration: binding.initial_configuration.clone(),
            resources: binding.initial_resources.clone(),
            capabilities: binding.initial_capabilities.clone(),
        }
    }
}

/// Metadata-only attachment snapshot. Durable content is addressed by a
/// stable opaque reference and digest; raw bytes never enter this record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AttachmentSnapshot {
    pub attachment_id: String,
    pub stable_reference: String,
    pub display_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub content_sha256: String,
    pub snapshot_version: u64,
    /// Immutable one-based display reference assigned by the owning Chat draft.
    #[serde(default)]
    pub original_reference: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UserTurnRecord {
    pub turn_id: String,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_context: Vec<SkillContextSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_context: Option<MessageReplyContextSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_turn: Option<McpTurnSnapshot>,
    pub submitted_at_ms: u64,
}

/// Immutable, turn-start Skill material. The qualified identity and entrypoint
/// digest make the exact instructions auditable without granting the native
/// runtime access to extension storage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SkillContextSnapshot {
    pub identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
    pub entrypoint_sha256: String,
    pub instructions: String,
    pub referenced_resources: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MessageReplyContextSnapshot {
    pub target_id: String,
    pub target_kind: String,
    pub source_sha256: String,
    pub source_excerpt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_context: Option<crate::artifact::ArtifactContextSnapshot>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunEventKind {
    Status,
    TextDelta,
    ReasoningDelta,
    ReasoningSummary,
    WorkActivity,
    ActionIntent,
    ActionProgress,
    ActionResult,
    Media,
    Usage,
    Error,
    Completion,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RunEventRecord {
    pub sequence: u64,
    pub session_id: String,
    pub turn_id: String,
    pub attempt_id: String,
    pub runtime_id: String,
    pub environment_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub kind: RunEventKind,
    pub payload: String,
    pub recorded_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum RunAttemptStatus {
    Dispatching,
    Streaming {
        started_at_ms: u64,
    },
    CancellationRequested {
        requested_at_ms: u64,
    },
    Completed {
        completed_at_ms: u64,
    },
    Failed {
        failed_at_ms: u64,
        error_code: String,
    },
    Interrupted {
        interrupted_at_ms: u64,
        reason_code: String,
    },
    Cancelled {
        cancelled_at_ms: u64,
    },
}

impl RunAttemptStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. }
                | Self::Failed { .. }
                | Self::Interrupted { .. }
                | Self::Cancelled { .. }
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum SideEffectState {
    None,
    Started { action_id: String, idempotent: bool },
    Completed { action_id: String },
    ProvenNotCompleted { action_id: String, idempotent: bool },
    Unknown { action_id: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RunAttemptRecord {
    pub attempt_id: String,
    pub turn_id: String,
    pub parent_attempt_id: Option<String>,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub context: AttemptContextSnapshot,
    pub status: RunAttemptStatus,
    pub side_effects: Vec<SideEffectState>,
    pub reviewed_parent_unknown_effect: bool,
    pub events: Vec<RunEventRecord>,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum SessionLifecycle {
    Provisional,
    Bound { binding: Box<SessionBinding> },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SessionRecord {
    pub schema_version: u16,
    pub session_id: String,
    pub revision: u64,
    pub lifecycle: SessionLifecycle,
    pub title: Option<String>,
    pub turns: Vec<UserTurnRecord>,
    pub attempts: Vec<RunAttemptRecord>,
    pub active_attempt_id: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl SessionRecord {
    pub fn binding(&self) -> Option<&SessionBinding> {
        match &self.lifecycle {
            SessionLifecycle::Provisional => None,
            SessionLifecycle::Bound { binding } => Some(binding.as_ref()),
        }
    }

    pub fn attempt(&self, attempt_id: &str) -> Option<&RunAttemptRecord> {
        self.attempts
            .iter()
            .find(|attempt| attempt.attempt_id == attempt_id)
    }

    pub fn turn(&self, turn_id: &str) -> Option<&UserTurnRecord> {
        self.turns.iter().find(|turn| turn.turn_id == turn_id)
    }

    pub fn validate(&self) -> Result<(), SessionError> {
        if self.schema_version != SESSION_SCHEMA_VERSION {
            return Err(SessionError::UnsupportedSchema);
        }
        validate_identifier("session id", &self.session_id)?;
        if self.revision == 0 {
            return Err(SessionError::InvalidRecord("revision must be non-zero"));
        }
        if self.updated_at_ms < self.created_at_ms {
            return Err(SessionError::InvalidRecord("invalid session timestamps"));
        }
        if let Some(title) = &self.title
            && (title.trim().is_empty() || title.chars().count() > MAX_SESSION_TITLE_CHARS)
        {
            return Err(SessionError::InvalidRecord("invalid session title"));
        }

        match (
            &self.lifecycle,
            self.turns.is_empty(),
            self.attempts.is_empty(),
        ) {
            (SessionLifecycle::Provisional, true, true) => {
                if self.title.is_some() || self.active_attempt_id.is_some() {
                    return Err(SessionError::InvalidRecord(
                        "provisional session carries promoted state",
                    ));
                }
            }
            (SessionLifecycle::Bound { binding }, false, false) => {
                binding.validate()?;
                if binding.bound_at_ms != self.turns[0].submitted_at_ms
                    || binding.bound_at_ms < self.created_at_ms
                {
                    return Err(SessionError::InvalidRecord(
                        "first-submit binding timestamp mismatch",
                    ));
                }
            }
            _ => {
                return Err(SessionError::InvalidRecord(
                    "session lifecycle and child records disagree",
                ));
            }
        }

        let mut turn_ids = BTreeSet::new();
        let mut attachment_ids = BTreeSet::new();
        for turn in &self.turns {
            turn.validate()?;
            if !turn_ids.insert(turn.turn_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("turn id"));
            }
            for attachment in &turn.attachments {
                if !attachment_ids.insert(attachment.attachment_id.as_str()) {
                    return Err(SessionError::DuplicateIdentifier("attachment id"));
                }
            }
        }

        let mut attempt_ids = BTreeSet::new();
        let mut correlation_ids = BTreeSet::new();
        let mut authorization_scope_ids = BTreeSet::new();
        let binding = self.binding();
        for attempt in &self.attempts {
            attempt.validate()?;
            if !turn_ids.contains(attempt.turn_id.as_str()) {
                return Err(SessionError::InvalidRecord(
                    "attempt references unknown turn",
                ));
            }
            if !attempt_ids.insert(attempt.attempt_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("attempt id"));
            }
            if !correlation_ids.insert(attempt.correlation_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("correlation id"));
            }
            if !authorization_scope_ids.insert(attempt.authorization_scope_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("authorization scope id"));
            }
            if let Some(binding) = binding {
                attempt.context.validate_against(binding)?;
            }
            for event in &attempt.events {
                if event.session_id != self.session_id
                    || event.turn_id != attempt.turn_id
                    || event.attempt_id != attempt.attempt_id
                    || event.runtime_id != attempt.context.runtime_id
                    || event.environment_id != attempt.context.environment.environment_id
                {
                    return Err(SessionError::InvalidRecord(
                        "run event identity does not match its attempt",
                    ));
                }
            }
        }
        for (attempt_index, attempt) in self.attempts.iter().enumerate() {
            if let Some(parent_attempt_id) = &attempt.parent_attempt_id {
                let Some(parent_index) = self
                    .attempts
                    .iter()
                    .position(|candidate| candidate.attempt_id == *parent_attempt_id)
                else {
                    return Err(SessionError::InvalidRecord("retry parent does not exist"));
                };
                if parent_index >= attempt_index {
                    return Err(SessionError::InvalidRecord(
                        "retry parent must precede child",
                    ));
                }
                let parent = &self.attempts[parent_index];
                if parent.turn_id != attempt.turn_id || !parent.status.is_terminal() {
                    return Err(SessionError::InvalidRecord("invalid retry ancestry"));
                }
                if has_unknown_effect(parent) != attempt.reviewed_parent_unknown_effect {
                    return Err(SessionError::InvalidRecord(
                        "unknown-effect review provenance mismatch",
                    ));
                }
            } else if attempt.reviewed_parent_unknown_effect {
                return Err(SessionError::InvalidRecord(
                    "root attempt cannot carry retry review provenance",
                ));
            }
        }

        for turn in &self.turns {
            if self
                .attempts
                .iter()
                .filter(|attempt| {
                    attempt.turn_id == turn.turn_id && attempt.parent_attempt_id.is_none()
                })
                .count()
                != 1
            {
                return Err(SessionError::InvalidRecord(
                    "turn must own exactly one root attempt",
                ));
            }
        }

        let nonterminal = self
            .attempts
            .iter()
            .filter(|attempt| !attempt.status.is_terminal())
            .collect::<Vec<_>>();
        match (&self.active_attempt_id, nonterminal.as_slice()) {
            (None, []) => {}
            (Some(active), [attempt]) if active == &attempt.attempt_id => {}
            _ => {
                return Err(SessionError::InvalidRecord(
                    "active attempt identity is inconsistent",
                ));
            }
        }
        Ok(())
    }
}

impl SessionBinding {
    pub(crate) fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("workspace id", &self.workspace_id)?;
        validate_optional_identifier("project id", self.project_id.as_deref())?;
        validate_identifier("runtime id", &self.runtime_id)?;
        self.adapter.validate()?;
        self.environment.validate()?;
        self.initial_model_route.validate()?;
        self.initial_configuration.validate()?;
        self.initial_resources.validate()?;
        self.initial_capabilities.validate_against_runtime(
            &self.initial_model_route,
            self.runtime_kind,
            &self.adapter,
            &self.initial_configuration,
        )?;
        Ok(())
    }
}

impl AttemptContextSnapshot {
    pub(crate) fn validate_against(&self, binding: &SessionBinding) -> Result<(), SessionError> {
        validate_identifier("workspace id", &self.workspace_id)?;
        validate_optional_identifier("project id", self.project_id.as_deref())?;
        validate_identifier("runtime id", &self.runtime_id)?;
        self.adapter.validate()?;
        self.environment.validate()?;
        self.model_route.validate()?;
        self.configuration.validate()?;
        self.resources.validate()?;
        self.capabilities.validate_against_runtime(
            &self.model_route,
            self.runtime_kind,
            &self.adapter,
            &self.configuration,
        )?;
        if self.workspace_id != binding.workspace_id
            || self.project_id != binding.project_id
            || self.runtime_id != binding.runtime_id
            || self.runtime_kind != binding.runtime_kind
            || self.adapter != binding.adapter
            || self.environment != binding.environment
        {
            return Err(SessionError::CrossRuntimeMigration);
        }
        Ok(())
    }
}

impl AdapterBinding {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("adapter id", &self.adapter_id)?;
        validate_identifier("adapter version", &self.adapter_version)?;
        validate_identifier("native version", &self.native_version)
    }
}

impl ExecutionEnvironmentBinding {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("environment id", &self.environment_id)?;
        validate_identifier("environment kind", &self.environment_kind)?;
        validate_optional_identifier("host alias", self.host_alias.as_deref())
    }
}

impl ModelRouteSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        for (kind, value) in [
            ("model route id", &self.route_id),
            ("provider id", &self.provider_id),
            ("endpoint id", &self.endpoint_id),
            ("model id", &self.model_id),
            ("model revision", &self.model_revision),
        ] {
            validate_identifier(kind, value)?;
        }
        Ok(())
    }
}

impl ConfigurationSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("configuration snapshot id", &self.snapshot_id)?;
        validate_version(self.version)?;
        validate_sha256(&self.sha256)
    }
}

impl ResourceSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("resource snapshot id", &self.snapshot_id)?;
        validate_version(self.version)?;
        validate_sha256(&self.sha256)?;
        validate_unique_identifiers(
            "resource id",
            &self.resource_ids,
            MAX_RESOURCES_PER_SNAPSHOT,
        )
    }
}

impl CapabilitySnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("capability snapshot id", &self.snapshot_id)?;
        validate_version(self.version)?;
        validate_sha256(&self.sha256)?;
        if self.capabilities.len() > MAX_CAPABILITIES_PER_SNAPSHOT {
            return Err(SessionError::BoundExceeded("capability snapshot"));
        }
        let mut ids = BTreeSet::new();
        for capability in &self.capabilities {
            capability.validate()?;
            if !ids.insert(capability.capability_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("capability id"));
            }
        }
        let descriptor = self
            .effective_descriptor
            .as_ref()
            .ok_or(SessionError::InvalidRecord("capability descriptor"))?;
        descriptor
            .validate()
            .map_err(|_| SessionError::InvalidRecord("capability descriptor"))?;
        if descriptor.layer != CapabilityLayer::Effective {
            return Err(SessionError::InvalidRecord("capability descriptor"));
        }
        let canonical = canonical_capability_descriptor(descriptor);
        if descriptor != &canonical {
            return Err(SessionError::InvalidRecord("capability descriptor"));
        }
        if self.capabilities != capability_states_from_descriptor(&canonical)? {
            return Err(SessionError::InvalidRecord("capability snapshot"));
        }
        let expected_sha256 = capability_snapshot_sha256(&canonical, self.version)?;
        if self.sha256 != expected_sha256
            || self.snapshot_id
                != format!(
                    "capabilities:{}",
                    expected_sha256
                        .strip_prefix("sha256:")
                        .ok_or(SessionError::InvalidRecord("capability snapshot"))?
                )
        {
            return Err(SessionError::InvalidRecord("capability snapshot"));
        }
        Ok(())
    }

    fn validate_against_runtime(
        &self,
        model_route: &ModelRouteSnapshot,
        runtime_kind: RuntimeKind,
        adapter: &AdapterBinding,
        configuration: &ConfigurationSnapshot,
    ) -> Result<(), SessionError> {
        self.validate()?;
        let descriptor = self
            .effective_descriptor
            .as_ref()
            .ok_or(SessionError::InvalidRecord("capability descriptor"))?;
        let expected_runtime_kind = match runtime_kind {
            RuntimeKind::OpenCode => "opencode",
            RuntimeKind::Pi => "pi",
        };
        if descriptor.route.provider_id != model_route.provider_id
            || descriptor.route.endpoint_id != model_route.endpoint_id
            || descriptor.route.provider_model_id != model_route.model_id
            || descriptor.route.model_revision != model_route.model_revision
            || descriptor.route.adapter_kind != expected_runtime_kind
            || descriptor.route.adapter_version != adapter.adapter_version
            || descriptor.route.runtime_kind != expected_runtime_kind
            || descriptor.route.native_runtime_version != adapter.native_version
            || descriptor.route.session_configuration_sha256 != configuration.sha256
        {
            return Err(SessionError::InvalidRecord("capability route"));
        }
        Ok(())
    }
}

pub fn capability_snapshot_from_effective_descriptor(
    descriptor: CapabilityDescriptor,
    version: u64,
) -> Result<CapabilitySnapshot, SessionError> {
    validate_version(version)?;
    let descriptor = canonical_capability_descriptor(&descriptor);
    descriptor
        .validate()
        .map_err(|_| SessionError::InvalidRecord("capability descriptor"))?;
    if descriptor.layer != CapabilityLayer::Effective {
        return Err(SessionError::InvalidRecord("capability descriptor"));
    }
    let sha256 = capability_snapshot_sha256(&descriptor, version)?;
    let snapshot_id = format!(
        "capabilities:{}",
        sha256
            .strip_prefix("sha256:")
            .ok_or(SessionError::InvalidRecord("capability snapshot"))?
    );
    let snapshot = CapabilitySnapshot {
        snapshot_id,
        version,
        sha256,
        capabilities: capability_states_from_descriptor(&descriptor)?,
        effective_descriptor: Some(descriptor),
    };
    snapshot.validate()?;
    Ok(snapshot)
}

pub(crate) fn canonical_capability_descriptor(
    descriptor: &CapabilityDescriptor,
) -> CapabilityDescriptor {
    let mut canonical = descriptor.clone();
    for evidence in canonical.features.values_mut() {
        evidence.constraints.sort();
        evidence.allowed_values.sort();
    }
    for numeric in canonical.numeric_limits.values_mut() {
        numeric.evidence.constraints.sort();
        numeric.evidence.allowed_values.sort();
    }
    canonical
}

pub(crate) fn capability_snapshot_sha256(
    descriptor: &CapabilityDescriptor,
    generation: u64,
) -> Result<String, SessionError> {
    let canonical = canonical_capability_descriptor(descriptor);
    digest_capability_value(&("c4os.capabilities.v1", generation, canonical))
}

pub(crate) fn capability_states_from_descriptor(
    descriptor: &CapabilityDescriptor,
) -> Result<Vec<CapabilityStateSnapshot>, SessionError> {
    let canonical = canonical_capability_descriptor(descriptor);
    let mut states = Vec::with_capacity(canonical.features.len() + canonical.numeric_limits.len());
    for (key, evidence) in &canonical.features {
        states.push(capability_state_from_evidence(
            feature_capability_id(*key),
            evidence,
            (!evidence.constraints.is_empty() || !evidence.allowed_values.is_empty())
                .then(|| digest_capability_value(&(key, evidence)))
                .transpose()?,
        ));
    }
    for (key, numeric) in &canonical.numeric_limits {
        states.push(capability_state_from_evidence(
            numeric_capability_id(*key),
            &numeric.evidence,
            Some(digest_capability_value(&(key, numeric))?),
        ));
    }
    Ok(states)
}

fn digest_capability_value(value: &impl Serialize) -> Result<String, SessionError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| SessionError::InvalidRecord("capability snapshot"))?;
    if bytes.len() > MAX_CAPABILITY_CANONICAL_BYTES {
        return Err(SessionError::BoundExceeded("capability snapshot"));
    }
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    Ok(encoded)
}

fn capability_state_from_evidence(
    capability_id: &'static str,
    evidence: &CapabilityEvidence,
    constraint_sha256: Option<String>,
) -> CapabilityStateSnapshot {
    CapabilityStateSnapshot {
        capability_id: capability_id.into(),
        support: match evidence.state {
            CapabilityState::Supported => CapabilitySupport::Supported,
            CapabilityState::Unsupported => CapabilitySupport::Unsupported,
            CapabilityState::Unknown => CapabilitySupport::Unknown,
            CapabilityState::Degraded => CapabilitySupport::Degraded,
        },
        source_id: evidence.source.clone(),
        constraint_id: constraint_sha256.map(|digest| {
            format!(
                "constraint:{}",
                digest.strip_prefix("sha256:").unwrap_or(&digest)
            )
        }),
        reason_code: evidence.reason.as_ref().map(|reason| {
            let digest = Sha256::digest(reason.as_bytes());
            let mut encoded = String::with_capacity(71);
            encoded.push_str("reason:");
            for byte in digest {
                write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
            }
            encoded
        }),
    }
}

fn feature_capability_id(key: CapabilityKey) -> &'static str {
    match key {
        CapabilityKey::InputText => "feature:input-text",
        CapabilityKey::InputImage => "feature:input-image",
        CapabilityKey::InputAudio => "feature:input-audio",
        CapabilityKey::InputVideo => "feature:input-video",
        CapabilityKey::InputPdf => "feature:input-pdf",
        CapabilityKey::OutputText => "feature:output-text",
        CapabilityKey::OutputImage => "feature:output-image",
        CapabilityKey::OutputAudio => "feature:output-audio",
        CapabilityKey::OutputVideo => "feature:output-video",
        CapabilityKey::Streaming => "feature:streaming",
        CapabilityKey::Reasoning => "feature:reasoning",
        CapabilityKey::ReasoningSummary => "feature:reasoning-summary",
        CapabilityKey::ToolCalling => "feature:tool-calling",
        CapabilityKey::ParallelToolCalling => "feature:parallel-tool-calling",
        CapabilityKey::StrictToolSchema => "feature:strict-tool-schema",
        CapabilityKey::StreamedToolArguments => "feature:streamed-tool-arguments",
        CapabilityKey::StructuredJson => "feature:structured-json",
        CapabilityKey::StructuredJsonSchema => "feature:structured-json-schema",
        CapabilityKey::Temperature => "feature:temperature",
        CapabilityKey::TopP => "feature:top-p",
        CapabilityKey::TopK => "feature:top-k",
        CapabilityKey::StopSequences => "feature:stop-sequences",
        CapabilityKey::Seed => "feature:seed",
        CapabilityKey::Verbosity => "feature:verbosity",
        CapabilityKey::PromptCaching => "feature:prompt-caching",
        CapabilityKey::SessionAffinity => "feature:session-affinity",
    }
}

fn numeric_capability_id(key: NumericCapabilityKey) -> &'static str {
    match key {
        NumericCapabilityKey::ContextTokens => "limit:context-tokens",
        NumericCapabilityKey::InputTokens => "limit:input-tokens",
        NumericCapabilityKey::OutputTokens => "limit:output-tokens",
        NumericCapabilityKey::AttachmentBytes => "limit:attachment-bytes",
        NumericCapabilityKey::AttachmentCount => "limit:attachment-count",
    }
}

impl CapabilityStateSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("capability id", &self.capability_id)?;
        validate_identifier("capability source id", &self.source_id)?;
        validate_optional_identifier("capability constraint id", self.constraint_id.as_deref())?;
        validate_optional_identifier("capability reason code", self.reason_code.as_deref())
    }
}

impl AttachmentSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("attachment id", &self.attachment_id)?;
        validate_identifier("attachment reference", &self.stable_reference)?;
        validate_bounded_text("attachment display name", &self.display_name, 512, false)?;
        validate_identifier("attachment media type", &self.media_type)?;
        validate_sha256(&self.content_sha256)?;
        validate_version(self.snapshot_version)
    }
}

impl UserTurnRecord {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("turn id", &self.turn_id)?;
        validate_submission(self.prompt.as_deref(), &self.attachments)?;
        if self.attachments.len() > MAX_ATTACHMENTS_PER_TURN {
            return Err(SessionError::BoundExceeded("turn attachments"));
        }
        if self.skill_context.len() > MAX_SKILL_CONTEXTS_PER_TURN {
            return Err(SessionError::BoundExceeded("turn Skill context"));
        }
        let mut ids = BTreeSet::new();
        for attachment in &self.attachments {
            attachment.validate()?;
            if !ids.insert(attachment.attachment_id.as_str()) {
                return Err(SessionError::DuplicateIdentifier("attachment id"));
            }
        }
        if let Some(reply) = &self.reply_context {
            reply.validate()?;
        }
        if let Some(mcp_turn) = &self.mcp_turn {
            mcp_turn
                .validate()
                .map_err(|_| SessionError::InvalidRecord("turn MCP capability snapshot"))?;
        }
        let mut skill_identities = BTreeSet::new();
        let mut skill_bytes = 0usize;
        for skill in &self.skill_context {
            skill.validate()?;
            if !skill_identities.insert(skill.identity.as_str()) {
                return Err(SessionError::DuplicateIdentifier("Skill context identity"));
            }
            skill_bytes = skill_bytes
                .checked_add(skill.instructions.len())
                .ok_or(SessionError::BoundExceeded("turn Skill context"))?;
        }
        if skill_bytes > MAX_SKILL_CONTEXT_BYTES {
            return Err(SessionError::BoundExceeded("turn Skill context"));
        }
        Ok(())
    }
}

impl SkillContextSnapshot {
    pub fn validate(&self) -> Result<(), SessionError> {
        validate_bounded_text("Skill context identity", &self.identity, 512, false)?;
        if !self.identity.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        }) {
            return Err(SessionError::InvalidIdentifier("Skill context identity"));
        }
        validate_optional_identifier("Skill package id", self.package_id.as_deref())?;
        validate_sha256(&self.entrypoint_sha256)?;
        validate_bounded_text(
            "Skill instructions",
            &self.instructions,
            MAX_SKILL_CONTEXT_BYTES,
            false,
        )?;
        validate_unique_identifiers(
            "Skill referenced resource",
            &self.referenced_resources,
            MAX_SKILL_CONTEXT_REFERENCES,
        )
    }
}

impl MessageReplyContextSnapshot {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("Reply target id", &self.target_id)?;
        let message_target = matches!(
            self.target_kind.as_str(),
            "user-message" | "assistant-message"
        );
        let artifact_target = matches!(
            self.target_kind.as_str(),
            "file" | "folder" | "browser" | "terminal"
        );
        if (!message_target && !artifact_target)
            || message_target != self.artifact_context.is_none()
        {
            return Err(SessionError::InvalidRecord("Reply target kind"));
        }
        if let Some(context) = &self.artifact_context {
            context
                .validate()
                .map_err(|_| SessionError::InvalidRecord("Artifact Reply context"))?;
            if context.artifact_id != self.target_id || context.provider_type != self.target_kind {
                return Err(SessionError::InvalidRecord("Artifact Reply identity"));
            }
        }
        validate_sha256(&self.source_sha256)?;
        validate_bounded_text(
            "Reply source excerpt",
            &self.source_excerpt,
            MAX_REPLY_SOURCE_EXCERPT_BYTES,
            false,
        )
    }
}

impl RunAttemptRecord {
    fn validate(&self) -> Result<(), SessionError> {
        validate_identifier("attempt id", &self.attempt_id)?;
        validate_identifier("turn id", &self.turn_id)?;
        validate_optional_identifier("parent attempt id", self.parent_attempt_id.as_deref())?;
        validate_identifier("authorization scope id", &self.authorization_scope_id)?;
        validate_identifier("correlation id", &self.correlation_id)?;
        if self.process_generation == 0 {
            return Err(SessionError::InvalidGeneration);
        }
        match &self.status {
            RunAttemptStatus::Failed { error_code, .. } => {
                validate_identifier("attempt error code", error_code)?
            }
            RunAttemptStatus::Interrupted { reason_code, .. } => {
                validate_identifier("interruption reason code", reason_code)?
            }
            _ => {}
        }
        if self.side_effects.len() > MAX_SIDE_EFFECTS_PER_ATTEMPT {
            return Err(SessionError::BoundExceeded("side effects"));
        }
        let mut action_ids = BTreeSet::new();
        for effect in &self.side_effects {
            effect.validate()?;
            let action_id = effect
                .action_id()
                .ok_or(SessionError::InvalidRecord("empty side-effect entry"))?;
            if !action_ids.insert(action_id) {
                return Err(SessionError::DuplicateIdentifier("side-effect action id"));
            }
        }
        let terminal_at = match &self.status {
            RunAttemptStatus::Dispatching => None,
            RunAttemptStatus::Streaming { started_at_ms } => Some(*started_at_ms),
            RunAttemptStatus::CancellationRequested { requested_at_ms } => Some(*requested_at_ms),
            RunAttemptStatus::Completed { completed_at_ms } => Some(*completed_at_ms),
            RunAttemptStatus::Failed { failed_at_ms, .. } => Some(*failed_at_ms),
            RunAttemptStatus::Interrupted {
                interrupted_at_ms, ..
            } => Some(*interrupted_at_ms),
            RunAttemptStatus::Cancelled { cancelled_at_ms } => Some(*cancelled_at_ms),
        };
        if terminal_at.is_some_and(|timestamp| timestamp < self.created_at_ms) {
            return Err(SessionError::InvalidRecord("invalid attempt timestamp"));
        }
        if self.events.len() > MAX_RUN_EVENTS_PER_ATTEMPT {
            return Err(SessionError::BoundExceeded("run events"));
        }
        for (expected_sequence, event) in (1_u64..).zip(self.events.iter()) {
            event.validate()?;
            if event.sequence != expected_sequence
                || event.correlation_id != self.correlation_id
                || event.process_generation != self.process_generation
                || event.recorded_at_ms < self.created_at_ms
            {
                return Err(SessionError::InvalidRecord("invalid run event sequence"));
            }
        }
        Ok(())
    }
}

impl SideEffectState {
    fn action_id(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Started { action_id, .. }
            | Self::Completed { action_id }
            | Self::ProvenNotCompleted { action_id, .. }
            | Self::Unknown { action_id } => Some(action_id),
        }
    }

    fn validate(&self) -> Result<(), SessionError> {
        match self {
            Self::None => Ok(()),
            Self::Started { action_id, .. }
            | Self::Completed { action_id }
            | Self::ProvenNotCompleted { action_id, .. }
            | Self::Unknown { action_id } => validate_identifier("action id", action_id),
        }
    }
}

impl RunEventRecord {
    fn validate(&self) -> Result<(), SessionError> {
        if self.sequence == 0 || self.process_generation == 0 {
            return Err(SessionError::InvalidGeneration);
        }
        validate_identifier("correlation id", &self.correlation_id)?;
        validate_identifier("event session id", &self.session_id)?;
        validate_identifier("event turn id", &self.turn_id)?;
        validate_identifier("event attempt id", &self.attempt_id)?;
        validate_identifier("event runtime id", &self.runtime_id)?;
        validate_identifier("event environment id", &self.environment_id)?;
        validate_bounded_text(
            "run event payload",
            &self.payload,
            MAX_RUN_EVENT_BYTES,
            true,
        )
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SessionRepositoryError {
    #[error("session repository compare-and-swap conflict")]
    Conflict,
    #[error("session repository unavailable")]
    Unavailable,
}

/// Storage ownership remains outside the domain service. Implementations must
/// make `create` and `compare_and_swap` atomic and durable before returning.
pub trait SessionRepository: Send + Sync {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError>;

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError>;

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError>;

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError>;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SessionError {
    #[error("unsupported session schema")]
    UnsupportedSchema,
    #[error("invalid {0}")]
    InvalidIdentifier(&'static str),
    #[error("invalid persisted session record: {0}")]
    InvalidRecord(&'static str),
    #[error("{0} exceeds its persistence bound")]
    BoundExceeded(&'static str),
    #[error("duplicate {0}")]
    DuplicateIdentifier(&'static str),
    #[error("session does not exist")]
    NotFound,
    #[error("submission must contain non-empty text or an attachment")]
    EmptySubmission,
    #[error("provisional session was already promoted")]
    AlreadyBound,
    #[error("session is still provisional")]
    NotBound,
    #[error("another attempt is already active")]
    AttemptAlreadyActive,
    #[error("attempt does not exist")]
    AttemptNotFound,
    #[error("attempt is not the active attempt")]
    NotActiveAttempt,
    #[error("attempt is not terminal")]
    AttemptNotTerminal,
    #[error("attempt is already terminal")]
    AttemptAlreadyTerminal,
    #[error("runtime or execution environment migration requires a new Chat")]
    CrossRuntimeMigration,
    #[error("process generation must be non-zero")]
    InvalidGeneration,
    #[error("stale or mismatched correlation identity")]
    StaleCorrelation,
    #[error("stale process generation")]
    StaleGeneration,
    #[error("run event sequence is stale or out of order")]
    StaleEventSequence,
    #[error("run event carries stale or mismatched run identity")]
    StaleRunIdentity,
    #[error("unknown side-effect completion requires explicit user review")]
    UnknownEffectReviewRequired,
    #[error("automatic retry is unsafe after a possible or completed side effect")]
    UnsafeAutomaticRetry,
    #[error("invalid side-effect state transition")]
    InvalidEffectTransition,
    #[error("session revision overflow")]
    RevisionOverflow,
    #[error(transparent)]
    Repository(#[from] SessionRepositoryError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptIdentity {
    pub attempt_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstSubmission {
    pub session_id: String,
    pub turn_id: String,
    pub attempt_id: String,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    pub skill_context: Vec<SkillContextSnapshot>,
    pub mcp_turn: Option<McpTurnSnapshot>,
    pub binding: SessionBinding,
    pub submitted_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnSubmission {
    pub session_id: String,
    pub turn_id: String,
    pub attempt_id: String,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub prompt: Option<String>,
    pub attachments: Vec<AttachmentSnapshot>,
    pub skill_context: Vec<SkillContextSnapshot>,
    pub reply_context: Option<MessageReplyContextSnapshot>,
    pub mcp_turn: Option<McpTurnSnapshot>,
    pub context: AttemptContextSnapshot,
    pub submitted_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryRequest {
    pub session_id: String,
    pub parent_attempt_id: String,
    pub attempt_id: String,
    pub authorization_scope_id: String,
    pub correlation_id: String,
    pub process_generation: u64,
    pub context: AttemptContextSnapshot,
    pub automatic: bool,
    pub reviewed_unknown_effect: bool,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalAttemptOutcome {
    Completed,
    Failed { error_code: String },
    Interrupted { reason_code: String },
    Cancelled,
}

pub struct SessionService<R> {
    repository: R,
    /// A blank Chat is a renderer/core draft, not a durable Chat record. The
    /// first valid text or attachment submission creates the complete bound
    /// record atomically; dropping the process discards every pending blank.
    provisionals: Mutex<BTreeMap<String, SessionRecord>>,
}

impl<R: SessionRepository> SessionService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            provisionals: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn create_provisional(
        &self,
        session_id: impl Into<String>,
        created_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        let record = SessionRecord {
            schema_version: SESSION_SCHEMA_VERSION,
            session_id: session_id.into(),
            revision: 1,
            lifecycle: SessionLifecycle::Provisional,
            title: None,
            turns: Vec::new(),
            attempts: Vec::new(),
            active_attempt_id: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        };
        record.validate()?;
        if self.repository.load(&record.session_id)?.is_some() {
            return Err(SessionError::AlreadyBound);
        }
        let mut provisionals = self
            .provisionals
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        if provisionals
            .insert(record.session_id.clone(), record.clone())
            .is_some()
        {
            return Err(SessionError::AlreadyBound);
        }
        Ok(record)
    }

    pub fn session(&self, session_id: &str) -> Result<SessionRecord, SessionError> {
        if let Some(record) = self
            .provisionals
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .get(session_id)
            .cloned()
        {
            record.validate()?;
            return Ok(record);
        }
        let record = self
            .repository
            .load(session_id)?
            .ok_or(SessionError::NotFound)?;
        record.validate()?;
        Ok(record)
    }

    /// Lists only durable bound sessions from the Workspace repository.
    /// Memory-only provisionals are intentionally excluded so a blank Chat
    /// cannot leak into navigation before its first valid submission.
    pub fn durable_sessions(&self) -> Result<Vec<SessionRecord>, SessionError> {
        let records = self.repository.list()?;
        for record in &records {
            record.validate()?;
            if matches!(record.lifecycle, SessionLifecycle::Provisional) {
                return Err(SessionError::InvalidRecord(
                    "durable repository contains a provisional session",
                ));
            }
        }
        Ok(records)
    }

    pub fn discard_provisional(&self, session_id: &str) -> Result<SessionRecord, SessionError> {
        validate_identifier("session id", session_id)?;
        self.provisionals
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .remove(session_id)
            .ok_or(SessionError::NotFound)
    }

    /// Invalidates every memory-only Chat before a Workspace authority change.
    /// Durable sessions remain in the database that owns them; provisionals
    /// cannot cross into a newly bound Workspace.
    pub fn discard_all_provisionals(&self) -> Result<usize, SessionError> {
        let mut provisionals = self
            .provisionals
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let discarded = provisionals.len();
        provisionals.clear();
        Ok(discarded)
    }

    /// Promotes one memory-only provisional session and durably creates its
    /// immutable binding, first Turn, and first Run Attempt in one insert.
    pub fn submit_first(
        &self,
        mut submission: FirstSubmission,
    ) -> Result<SessionRecord, SessionError> {
        validate_identifier("session id", &submission.session_id)?;
        validate_submission(submission.prompt.as_deref(), &submission.attachments)?;
        submission.binding.bound_at_ms = submission.submitted_at_ms;
        submission.binding.validate()?;
        let mut provisionals = self
            .provisionals
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let mut record = provisionals
            .get(&submission.session_id)
            .cloned()
            .ok_or_else(|| {
                if self
                    .repository
                    .load(&submission.session_id)
                    .ok()
                    .flatten()
                    .is_some()
                {
                    SessionError::AlreadyBound
                } else {
                    SessionError::NotFound
                }
            })?;
        if !matches!(record.lifecycle, SessionLifecycle::Provisional) {
            return Err(SessionError::AlreadyBound);
        }
        if submission.process_generation == 0 {
            return Err(SessionError::InvalidGeneration);
        }
        validate_new_attempt_ids(
            &record,
            &submission.turn_id,
            &submission.attempt_id,
            &submission.authorization_scope_id,
            &submission.correlation_id,
        )?;
        let turn = UserTurnRecord {
            turn_id: submission.turn_id,
            prompt: submission.prompt,
            attachments: submission.attachments,
            skill_context: submission.skill_context,
            reply_context: None,
            mcp_turn: submission.mcp_turn,
            submitted_at_ms: submission.submitted_at_ms,
        };
        turn.validate()?;
        let attempt = RunAttemptRecord {
            attempt_id: submission.attempt_id.clone(),
            turn_id: turn.turn_id.clone(),
            parent_attempt_id: None,
            authorization_scope_id: submission.authorization_scope_id,
            correlation_id: submission.correlation_id,
            process_generation: submission.process_generation,
            context: AttemptContextSnapshot::from_binding(&submission.binding),
            status: RunAttemptStatus::Dispatching,
            side_effects: Vec::new(),
            reviewed_parent_unknown_effect: false,
            events: Vec::new(),
            created_at_ms: submission.submitted_at_ms,
        };
        record.lifecycle = SessionLifecycle::Bound {
            binding: Box::new(submission.binding),
        };
        record.title = Some(derive_title(&turn));
        record.turns.push(turn);
        record.attempts.push(attempt);
        record.active_attempt_id = Some(submission.attempt_id);
        record.updated_at_ms = submission.submitted_at_ms;
        record.validate()?;
        self.repository.create(&record)?;
        provisionals.remove(&record.session_id);
        Ok(record)
    }

    pub fn submit_turn(&self, submission: TurnSubmission) -> Result<SessionRecord, SessionError> {
        validate_submission(submission.prompt.as_deref(), &submission.attachments)?;
        let mut record = self.session(&submission.session_id)?;
        let binding = record.binding().ok_or(SessionError::NotBound)?;
        submission.context.validate_against(binding)?;
        ensure_no_active_attempt(&record)?;
        if submission.process_generation == 0 {
            return Err(SessionError::InvalidGeneration);
        }
        validate_new_attempt_ids(
            &record,
            &submission.turn_id,
            &submission.attempt_id,
            &submission.authorization_scope_id,
            &submission.correlation_id,
        )?;
        let turn = UserTurnRecord {
            turn_id: submission.turn_id,
            prompt: submission.prompt,
            attachments: submission.attachments,
            skill_context: submission.skill_context,
            reply_context: submission.reply_context,
            mcp_turn: submission.mcp_turn,
            submitted_at_ms: submission.submitted_at_ms,
        };
        turn.validate()?;
        let attempt = RunAttemptRecord {
            attempt_id: submission.attempt_id.clone(),
            turn_id: turn.turn_id.clone(),
            parent_attempt_id: None,
            authorization_scope_id: submission.authorization_scope_id,
            correlation_id: submission.correlation_id,
            process_generation: submission.process_generation,
            context: submission.context,
            status: RunAttemptStatus::Dispatching,
            side_effects: Vec::new(),
            reviewed_parent_unknown_effect: false,
            events: Vec::new(),
            created_at_ms: submission.submitted_at_ms,
        };
        record.turns.push(turn);
        record.attempts.push(attempt);
        record.active_attempt_id = Some(submission.attempt_id);
        self.persist_replacement(record, submission.submitted_at_ms)
    }

    pub fn append_event(
        &self,
        session_id: &str,
        identity: &AttemptIdentity,
        event: RunEventRecord,
    ) -> Result<SessionRecord, SessionError> {
        let mut record = self.session(session_id)?;
        ensure_active_identity(&record, identity)?;
        let attempt = find_attempt_mut(&mut record, &identity.attempt_id)?;
        if event.session_id != session_id
            || event.turn_id != attempt.turn_id
            || event.attempt_id != attempt.attempt_id
            || event.runtime_id != attempt.context.runtime_id
            || event.environment_id != attempt.context.environment.environment_id
        {
            return Err(SessionError::StaleRunIdentity);
        }
        if event.correlation_id != identity.correlation_id {
            return Err(SessionError::StaleCorrelation);
        }
        if event.process_generation != identity.process_generation {
            return Err(SessionError::StaleGeneration);
        }
        let expected_sequence = attempt.events.len() as u64 + 1;
        if event.sequence != expected_sequence {
            return Err(SessionError::StaleEventSequence);
        }
        if attempt.events.len() >= MAX_RUN_EVENTS_PER_ATTEMPT {
            return Err(SessionError::BoundExceeded("run events"));
        }
        event.validate()?;
        if matches!(attempt.status, RunAttemptStatus::Dispatching) {
            attempt.status = RunAttemptStatus::Streaming {
                started_at_ms: event.recorded_at_ms,
            };
        }
        attempt.events.push(event.clone());
        self.persist_replacement(record, event.recorded_at_ms)
    }

    pub fn request_cancellation(
        &self,
        session_id: &str,
        identity: &AttemptIdentity,
        requested_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        let mut record = self.session(session_id)?;
        ensure_active_identity(&record, identity)?;
        let attempt = find_attempt_mut(&mut record, &identity.attempt_id)?;
        if matches!(
            attempt.status,
            RunAttemptStatus::CancellationRequested { .. }
        ) {
            return Ok(record);
        }
        attempt.status = RunAttemptStatus::CancellationRequested { requested_at_ms };
        self.persist_replacement(record, requested_at_ms)
    }

    pub fn finish_attempt(
        &self,
        session_id: &str,
        identity: &AttemptIdentity,
        outcome: TerminalAttemptOutcome,
        finished_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        let mut record = self.session(session_id)?;
        ensure_active_identity(&record, identity)?;
        let attempt = find_attempt_mut(&mut record, &identity.attempt_id)?;
        // A terminal run state never proves that an in-flight effect completed.
        // Preserve each ambiguous action independently for explicit review.
        mark_started_effects_unknown(attempt);
        attempt.status = match outcome {
            TerminalAttemptOutcome::Completed => RunAttemptStatus::Completed {
                completed_at_ms: finished_at_ms,
            },
            TerminalAttemptOutcome::Failed { error_code } => {
                validate_identifier("attempt error code", &error_code)?;
                RunAttemptStatus::Failed {
                    failed_at_ms: finished_at_ms,
                    error_code,
                }
            }
            TerminalAttemptOutcome::Interrupted { reason_code } => {
                validate_identifier("interruption reason code", &reason_code)?;
                RunAttemptStatus::Interrupted {
                    interrupted_at_ms: finished_at_ms,
                    reason_code,
                }
            }
            TerminalAttemptOutcome::Cancelled => RunAttemptStatus::Cancelled {
                cancelled_at_ms: finished_at_ms,
            },
        };
        record.active_attempt_id = None;
        self.persist_replacement(record, finished_at_ms)
    }

    pub fn set_side_effect_state(
        &self,
        session_id: &str,
        identity: &AttemptIdentity,
        next: SideEffectState,
        changed_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        next.validate()?;
        let mut record = self.session(session_id)?;
        ensure_active_identity(&record, identity)?;
        let attempt = find_attempt_mut(&mut record, &identity.attempt_id)?;
        let action_id = next
            .action_id()
            .ok_or(SessionError::InvalidEffectTransition)?
            .to_owned();
        if let Some(index) = attempt
            .side_effects
            .iter()
            .position(|effect| effect.action_id() == Some(action_id.as_str()))
        {
            if !valid_effect_transition(&attempt.side_effects[index], &next) {
                return Err(SessionError::InvalidEffectTransition);
            }
            attempt.side_effects[index] = next;
        } else if matches!(next, SideEffectState::Started { .. })
            && attempt.side_effects.len() < MAX_SIDE_EFFECTS_PER_ATTEMPT
        {
            attempt.side_effects.push(next);
        } else if attempt.side_effects.len() >= MAX_SIDE_EFFECTS_PER_ATTEMPT {
            return Err(SessionError::BoundExceeded("side effects"));
        } else {
            return Err(SessionError::InvalidEffectTransition);
        }
        self.persist_replacement(record, changed_at_ms)
    }

    pub fn retry(&self, request: RetryRequest) -> Result<SessionRecord, SessionError> {
        let mut record = self.session(&request.session_id)?;
        ensure_no_active_attempt(&record)?;
        let binding = record.binding().ok_or(SessionError::NotBound)?;
        request.context.validate_against(binding)?;
        if request.process_generation == 0 {
            return Err(SessionError::InvalidGeneration);
        }
        let parent = record
            .attempt(&request.parent_attempt_id)
            .ok_or(SessionError::AttemptNotFound)?;
        if !parent.status.is_terminal() {
            return Err(SessionError::AttemptNotTerminal);
        }
        if has_unknown_effect(parent) && !request.reviewed_unknown_effect {
            return Err(SessionError::UnknownEffectReviewRequired);
        }
        if request.automatic && !automatic_retry_safe(parent) {
            return Err(SessionError::UnsafeAutomaticRetry);
        }
        let turn_id = parent.turn_id.clone();
        validate_new_attempt_ids(
            &record,
            &turn_id,
            &request.attempt_id,
            &request.authorization_scope_id,
            &request.correlation_id,
        )?;
        let attempt = RunAttemptRecord {
            attempt_id: request.attempt_id.clone(),
            turn_id,
            parent_attempt_id: Some(request.parent_attempt_id),
            authorization_scope_id: request.authorization_scope_id,
            correlation_id: request.correlation_id,
            process_generation: request.process_generation,
            context: request.context,
            status: RunAttemptStatus::Dispatching,
            side_effects: Vec::new(),
            reviewed_parent_unknown_effect: has_unknown_effect(parent)
                && request.reviewed_unknown_effect,
            events: Vec::new(),
            created_at_ms: request.created_at_ms,
        };
        record.attempts.push(attempt);
        record.active_attempt_id = Some(request.attempt_id);
        self.persist_replacement(record, request.created_at_ms)
    }

    /// Restart recovery preserves every prior record and turns the one active
    /// attempt into an interruption. A started effect becomes unknown so Retry
    /// is review-gated.
    pub fn recover_interrupted(
        &self,
        session_id: &str,
        recovered_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        let mut record = self.session(session_id)?;
        let Some(active_attempt_id) = record.active_attempt_id.clone() else {
            return Ok(record);
        };
        let attempt = find_attempt_mut(&mut record, &active_attempt_id)?;
        mark_started_effects_unknown(attempt);
        attempt.status = RunAttemptStatus::Interrupted {
            interrupted_at_ms: recovered_at_ms,
            reason_code: "application-restart".into(),
        };
        record.active_attempt_id = None;
        self.persist_replacement(record, recovered_at_ms)
    }

    /// Enumerates every durable Chat so app startup can recover all active
    /// attempts rather than only sessions already known to the renderer.
    pub fn recover_all_interrupted(
        &self,
        recovered_at_ms: u64,
    ) -> Result<Vec<SessionRecord>, SessionError> {
        let active_ids = self
            .repository
            .list()?
            .into_iter()
            .filter(|record| record.active_attempt_id.is_some())
            .map(|record| record.session_id)
            .collect::<Vec<_>>();
        active_ids
            .into_iter()
            .map(|session_id| self.recover_interrupted(&session_id, recovered_at_ms))
            .collect()
    }

    fn persist_replacement(
        &self,
        mut record: SessionRecord,
        updated_at_ms: u64,
    ) -> Result<SessionRecord, SessionError> {
        let expected_revision = record.revision;
        record.revision = record
            .revision
            .checked_add(1)
            .ok_or(SessionError::RevisionOverflow)?;
        record.updated_at_ms = updated_at_ms;
        record.validate()?;
        self.repository
            .compare_and_swap(&record.session_id, expected_revision, &record)?;
        Ok(record)
    }
}

fn find_attempt_mut<'a>(
    record: &'a mut SessionRecord,
    attempt_id: &str,
) -> Result<&'a mut RunAttemptRecord, SessionError> {
    record
        .attempts
        .iter_mut()
        .find(|attempt| attempt.attempt_id == attempt_id)
        .ok_or(SessionError::AttemptNotFound)
}

fn ensure_no_active_attempt(record: &SessionRecord) -> Result<(), SessionError> {
    if record.active_attempt_id.is_some() {
        Err(SessionError::AttemptAlreadyActive)
    } else {
        Ok(())
    }
}

fn ensure_active_identity(
    record: &SessionRecord,
    identity: &AttemptIdentity,
) -> Result<(), SessionError> {
    if record.active_attempt_id.as_deref() != Some(identity.attempt_id.as_str()) {
        return Err(SessionError::NotActiveAttempt);
    }
    let attempt = record
        .attempt(&identity.attempt_id)
        .ok_or(SessionError::AttemptNotFound)?;
    if attempt.status.is_terminal() {
        return Err(SessionError::AttemptAlreadyTerminal);
    }
    if attempt.correlation_id != identity.correlation_id {
        return Err(SessionError::StaleCorrelation);
    }
    if attempt.process_generation != identity.process_generation {
        return Err(SessionError::StaleGeneration);
    }
    Ok(())
}

fn validate_new_attempt_ids(
    record: &SessionRecord,
    turn_id: &str,
    attempt_id: &str,
    authorization_scope_id: &str,
    correlation_id: &str,
) -> Result<(), SessionError> {
    for (kind, value) in [
        ("turn id", turn_id),
        ("attempt id", attempt_id),
        ("authorization scope id", authorization_scope_id),
        ("correlation id", correlation_id),
    ] {
        validate_identifier(kind, value)?;
    }
    if record.turn(turn_id).is_some() && !record.attempts.iter().any(|a| a.turn_id == turn_id) {
        return Err(SessionError::DuplicateIdentifier("turn id"));
    }
    if record.attempt(attempt_id).is_some() {
        return Err(SessionError::DuplicateIdentifier("attempt id"));
    }
    if record
        .attempts
        .iter()
        .any(|attempt| attempt.authorization_scope_id == authorization_scope_id)
    {
        return Err(SessionError::DuplicateIdentifier("authorization scope id"));
    }
    if record
        .attempts
        .iter()
        .any(|attempt| attempt.correlation_id == correlation_id)
    {
        return Err(SessionError::DuplicateIdentifier("correlation id"));
    }
    Ok(())
}

fn valid_effect_transition(current: &SideEffectState, next: &SideEffectState) -> bool {
    match (current, next) {
        (SideEffectState::None, SideEffectState::Started { .. }) => true,
        (
            SideEffectState::Started {
                action_id: current, ..
            },
            SideEffectState::Completed { action_id: next }
            | SideEffectState::ProvenNotCompleted {
                action_id: next, ..
            }
            | SideEffectState::Unknown { action_id: next },
        ) => current == next,
        _ => false,
    }
}

fn has_unknown_effect(attempt: &RunAttemptRecord) -> bool {
    attempt
        .side_effects
        .iter()
        .any(|effect| matches!(effect, SideEffectState::Unknown { .. }))
}

fn automatic_retry_safe(attempt: &RunAttemptRecord) -> bool {
    attempt.side_effects.iter().all(|effect| {
        matches!(
            effect,
            SideEffectState::ProvenNotCompleted {
                idempotent: true,
                ..
            }
        )
    })
}

fn mark_started_effects_unknown(attempt: &mut RunAttemptRecord) {
    for effect in &mut attempt.side_effects {
        if let SideEffectState::Started { action_id, .. } = effect {
            *effect = SideEffectState::Unknown {
                action_id: action_id.clone(),
            };
        }
    }
}

fn validate_submission(
    prompt: Option<&str>,
    attachments: &[AttachmentSnapshot],
) -> Result<(), SessionError> {
    let has_text = prompt.is_some_and(|value| !value.trim().is_empty());
    if !has_text && attachments.is_empty() {
        return Err(SessionError::EmptySubmission);
    }
    if let Some(prompt) = prompt {
        validate_bounded_text("turn prompt", prompt, MAX_SESSION_TEXT_BYTES, true)?;
    }
    if attachments.len() > MAX_ATTACHMENTS_PER_TURN {
        return Err(SessionError::BoundExceeded("turn attachments"));
    }
    Ok(())
}

fn derive_title(turn: &UserTurnRecord) -> String {
    let candidate = turn
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            turn.attachments
                .first()
                .map(|attachment| attachment.display_name.trim())
        })
        .unwrap_or("New Chat");
    let compact = candidate.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_SESSION_TITLE_CHARS {
        compact
    } else {
        compact
            .chars()
            .take(MAX_SESSION_TITLE_CHARS.saturating_sub(1))
            .chain(std::iter::once('…'))
            .collect()
    }
}

fn validate_version(version: u64) -> Result<(), SessionError> {
    if version == 0 {
        Err(SessionError::InvalidGeneration)
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), SessionError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(SessionError::InvalidRecord("invalid SHA-256 digest"));
    };
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(SessionError::InvalidRecord("invalid SHA-256 digest"))
    }
}

fn validate_unique_identifiers(
    kind: &'static str,
    values: &[String],
    limit: usize,
) -> Result<(), SessionError> {
    if values.len() > limit {
        return Err(SessionError::BoundExceeded(kind));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        validate_identifier(kind, value)?;
        if !unique.insert(value.as_str()) {
            return Err(SessionError::DuplicateIdentifier(kind));
        }
    }
    Ok(())
}

fn validate_optional_identifier(
    kind: &'static str,
    value: Option<&str>,
) -> Result<(), SessionError> {
    if let Some(value) = value {
        validate_identifier(kind, value)?;
    }
    Ok(())
}

fn validate_identifier(kind: &'static str, value: &str) -> Result<(), SessionError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_SESSION_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        });
    if valid {
        Ok(())
    } else {
        Err(SessionError::InvalidIdentifier(kind))
    }
}

fn validate_bounded_text(
    kind: &'static str,
    value: &str,
    max_bytes: usize,
    allow_empty: bool,
) -> Result<(), SessionError> {
    if value.len() > max_bytes {
        return Err(SessionError::BoundExceeded(kind));
    }
    if (!allow_empty && value.trim().is_empty()) || value.contains('\0') {
        return Err(SessionError::InvalidRecord(kind));
    }
    Ok(())
}
