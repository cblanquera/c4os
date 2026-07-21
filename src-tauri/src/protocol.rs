//! Versioned messages crossing the C4OS core boundary.
//!
//! The Rust core owns these types. Callers must validate an envelope before
//! inspecting its payload so an unknown protocol version or stale generation
//! cannot accidentally degrade into a permissive operation.
//! JSON object fields and enum discriminants use stable `camelCase` names.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use ts_rs::TS;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_IDENTIFIER_BYTES: usize = 160;
pub const MAX_TEXT_BYTES: usize = 1_048_576;
pub const MAX_DIAGNOSTIC_BYTES: usize = 8_192;
pub const MAX_ATTACHMENTS: usize = 64;
pub const MAX_SAFE_DETAILS: usize = 32;
pub const MAX_REDACTION_MARKERS: usize = 256;
pub const MAX_RECENT_WORKSPACES: usize = 3;
pub const MAX_DISPLAY_NAME_BYTES: usize = 512;

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(
            Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, TS,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ProtocolError> {
                let value = value.into();
                validate_identifier(stringify!($name), &value)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            fn validate(&self) -> Result<(), ProtocolError> {
                validate_identifier(stringify!($name), &self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = ProtocolError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ProtocolError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

typed_id!(RequestId);
typed_id!(CommandId);
typed_id!(CorrelationId);
typed_id!(WorkspaceId);
typed_id!(ProjectId);
typed_id!(SessionId);
typed_id!(TurnId);
typed_id!(AttemptId);
typed_id!(RuntimeId);
typed_id!(EnvironmentId);
typed_id!(ArtifactId);
typed_id!(AttachmentId);
typed_id!(ApprovalId);
typed_id!(PickerGrantId);
typed_id!(ConfigurationScopeId);
typed_id!(SnapshotId);

fn validate_identifier(kind: &'static str, value: &str) -> Result<(), ProtocolError> {
    let valid_bytes = !value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES;
    let valid_characters = value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
    });

    if valid_bytes && valid_characters {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorCode::InvalidIdentifier,
            format!("invalid {kind}"),
            false,
        ))
    }
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, TS,
)]
pub struct StateGeneration(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, TS)]
pub struct ProcessGeneration(pub u64);

impl ProcessGeneration {
    pub fn new(value: u64) -> Result<Self, ProtocolError> {
        if value == 0 {
            Err(ProtocolError::new(
                ProtocolErrorCode::InvalidGeneration,
                "process generation must be non-zero",
                false,
            ))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum ProtocolErrorCode {
    UnknownProtocolVersion,
    InvalidIdentifier,
    InvalidGeneration,
    StaleGeneration,
    FutureGeneration,
    CorrelationMismatch,
    InvalidPayload,
    PayloadTooLarge,
    NotFound,
    Conflict,
    Unauthorized,
    Forbidden,
    Unavailable,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum SafeDetailValue {
    Text(String),
    Integer(i64),
    Unsigned(u64),
    Boolean(bool),
    Redacted(RedactionMarker),
}

impl SafeDetailValue {
    fn validate(&self) -> Result<(), ProtocolError> {
        match self {
            Self::Text(value) if value.len() > MAX_DIAGNOSTIC_BYTES => Err(
                ProtocolError::payload_too_large("diagnostic detail exceeds its byte limit"),
            ),
            Self::Redacted(marker) => marker.validate(),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ProtocolError {
    pub code: ProtocolErrorCode,
    pub message: String,
    pub retryable: bool,
    pub correlation_id: Option<CorrelationId>,
    pub details: BTreeMap<String, SafeDetailValue>,
}

impl ProtocolError {
    pub fn new(code: ProtocolErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            correlation_id: None,
            details: BTreeMap::new(),
        }
    }

    pub fn with_correlation(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: SafeDetailValue) -> Self {
        if self.details.len() < MAX_SAFE_DETAILS {
            self.details.insert(key.into(), value);
        }
        self
    }

    fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(ProtocolErrorCode::PayloadTooLarge, message, false)
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.message.len() > MAX_DIAGNOSTIC_BYTES || self.details.len() > MAX_SAFE_DETAILS {
            return Err(Self::payload_too_large(
                "structured error exceeds its diagnostic limits",
            ));
        }
        if let Some(correlation_id) = &self.correlation_id {
            correlation_id.validate()?;
        }
        for (key, value) in &self.details {
            validate_identifier("error detail key", key)?;
            value.validate()?;
        }
        Ok(())
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum RedactionReason {
    Credential,
    Authorization,
    EnvironmentValue,
    WebsiteStorage,
    SensitiveField,
    Policy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RedactionMarker {
    pub field_path: String,
    pub reason: RedactionReason,
}

impl RedactionMarker {
    pub fn new(field_path: impl Into<String>, reason: RedactionReason) -> Self {
        Self {
            field_path: field_path.into(),
            reason,
        }
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        if self.field_path.is_empty() || self.field_path.len() > MAX_IDENTIFIER_BYTES {
            Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "invalid redaction field path",
                false,
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RunScope {
    pub workspace_id: WorkspaceId,
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub attempt_id: AttemptId,
    pub runtime_id: RuntimeId,
    pub environment_id: EnvironmentId,
    pub correlation_id: CorrelationId,
    pub process_generation: ProcessGeneration,
}

impl RunScope {
    fn validate(&self) -> Result<(), ProtocolError> {
        self.workspace_id.validate()?;
        self.session_id.validate()?;
        self.turn_id.validate()?;
        self.attempt_id.validate()?;
        self.runtime_id.validate()?;
        self.environment_id.validate()?;
        self.correlation_id.validate()?;
        ProcessGeneration::new(self.process_generation.0)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(rename_all = "camelCase")]
pub enum Command {
    OpenWorkspace {
        picker_grant_id: PickerGrantId,
    },
    CreatePendingChat {
        project_id: ProjectId,
    },
    SubmitTurn {
        session_id: SessionId,
        text: Option<String>,
        attachment_ids: Vec<AttachmentId>,
    },
    CancelRun {
        attempt_id: AttemptId,
    },
    AnswerApproval {
        approval_id: ApprovalId,
        decision: ApprovalDecision,
    },
    FocusArtifact {
        artifact_id: ArtifactId,
    },
    SaveFile {
        artifact_id: ArtifactId,
        base_generation: StateGeneration,
        content: String,
    },
    UpdateConfiguration {
        scope_id: ConfigurationScopeId,
        base_generation: StateGeneration,
        canonical_toml: String,
    },
}

impl Command {
    fn validate(&self) -> Result<(), ProtocolError> {
        match self {
            Self::OpenWorkspace { picker_grant_id } => picker_grant_id.validate(),
            Self::CreatePendingChat { project_id } => project_id.validate(),
            Self::SubmitTurn {
                session_id,
                text,
                attachment_ids,
            } => {
                session_id.validate()?;
                if text
                    .as_ref()
                    .is_some_and(|value| value.len() > MAX_TEXT_BYTES)
                {
                    return Err(ProtocolError::payload_too_large(
                        "turn text exceeds its byte limit",
                    ));
                }
                if attachment_ids.len() > MAX_ATTACHMENTS {
                    return Err(ProtocolError::payload_too_large(
                        "turn has too many attachments",
                    ));
                }
                if text.as_ref().is_none_or(|value| value.trim().is_empty())
                    && attachment_ids.is_empty()
                {
                    return Err(ProtocolError::new(
                        ProtocolErrorCode::InvalidPayload,
                        "turn requires text or an attachment",
                        false,
                    ));
                }
                for attachment_id in attachment_ids {
                    attachment_id.validate()?;
                }
                Ok(())
            }
            Self::CancelRun { attempt_id } => attempt_id.validate(),
            Self::AnswerApproval { approval_id, .. } => approval_id.validate(),
            Self::FocusArtifact { artifact_id } => artifact_id.validate(),
            Self::SaveFile {
                artifact_id,
                content,
                ..
            } => {
                artifact_id.validate()?;
                validate_bounded_text("file content", content)
            }
            Self::UpdateConfiguration {
                scope_id,
                canonical_toml,
                ..
            } => {
                scope_id.validate()?;
                validate_bounded_text("configuration document", canonical_toml)
            }
        }
    }
}

fn validate_bounded_text(label: &'static str, value: &str) -> Result<(), ProtocolError> {
    if value.len() > MAX_TEXT_BYTES {
        Err(ProtocolError::payload_too_large(format!(
            "{label} exceeds its byte limit"
        )))
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum ApprovalDecision {
    ApproveOnce,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct CommandRequest {
    pub command_id: CommandId,
    pub command: Command,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RequestEnvelope {
    pub protocol_version: u16,
    pub request_id: RequestId,
    pub correlation_id: CorrelationId,
    pub expected_generation: StateGeneration,
    pub request: CommandRequest,
}

impl RequestEnvelope {
    pub fn validate(&self, current_generation: StateGeneration) -> Result<(), ProtocolError> {
        validate_protocol_version(self.protocol_version)?;
        self.request_id.validate()?;
        self.correlation_id.validate()?;
        self.request.command_id.validate()?;
        validate_exact_generation(self.expected_generation, current_generation)?;
        self.request.command.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(rename_all = "camelCase")]
pub enum ResponsePayload {
    Acknowledged,
    StateSnapshot(StateSnapshot),
    ArtifactSaved {
        artifact_id: ArtifactId,
        generation: StateGeneration,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "status", content = "payload", rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum ResponseResult {
    Ok(ResponsePayload),
    Err(ProtocolError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ResponseEnvelope {
    pub protocol_version: u16,
    pub request_id: RequestId,
    pub correlation_id: CorrelationId,
    pub generation: StateGeneration,
    pub result: ResponseResult,
}

impl ResponseEnvelope {
    pub fn validate_for(
        &self,
        request: &RequestEnvelope,
        minimum_generation: StateGeneration,
    ) -> Result<(), ProtocolError> {
        validate_protocol_version(self.protocol_version)?;
        self.request_id.validate()?;
        self.correlation_id.validate()?;
        if self.request_id != request.request_id || self.correlation_id != request.correlation_id {
            return Err(ProtocolError::new(
                ProtocolErrorCode::CorrelationMismatch,
                "response identity does not match its request",
                false,
            ));
        }
        validate_not_stale(self.generation, minimum_generation)?;
        match &self.result {
            ResponseResult::Ok(ResponsePayload::StateSnapshot(snapshot)) => snapshot.validate(),
            ResponseResult::Ok(ResponsePayload::ArtifactSaved { artifact_id, .. }) => {
                artifact_id.validate()
            }
            ResponseResult::Ok(ResponsePayload::Acknowledged) => Ok(()),
            ResponseResult::Err(error) => error.validate(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct StateSnapshot {
    pub snapshot_id: SnapshotId,
    pub generation: StateGeneration,
    pub active_workspace_id: Option<WorkspaceId>,
    pub active_session_id: Option<SessionId>,
    pub redactions: Vec<RedactionMarker>,
}

impl StateSnapshot {
    fn validate(&self) -> Result<(), ProtocolError> {
        self.snapshot_id.validate()?;
        if let Some(workspace_id) = &self.active_workspace_id {
            workspace_id.validate()?;
        }
        if let Some(session_id) = &self.active_session_id {
            session_id.validate()?;
        }
        if self.redactions.len() > MAX_REDACTION_MARKERS {
            return Err(ProtocolError::payload_too_large(
                "snapshot contains too many redaction markers",
            ));
        }
        for marker in &self.redactions {
            marker.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum RunPhase {
    Queued,
    Running,
    AwaitingApproval,
    Cancelling,
    Completed,
    Failed,
    Interrupted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum RuntimeHealth {
    Starting,
    Ready,
    Degraded,
    Unavailable,
    Stopped,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(rename_all = "camelCase")]
pub enum CoreEvent {
    StateSnapshot(StateSnapshot),
    StateChanged { areas: Vec<String> },
    RunChanged { sequence: u64, phase: RunPhase },
    RuntimeHealthChanged { health: RuntimeHealth },
    ApprovalChanged { approval_id: ApprovalId },
    Error(ProtocolError),
}

impl CoreEvent {
    fn requires_run_scope(&self) -> bool {
        matches!(
            self,
            Self::RunChanged { .. } | Self::RuntimeHealthChanged { .. }
        )
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        match self {
            Self::StateSnapshot(snapshot) => snapshot.validate(),
            Self::StateChanged { areas } => {
                if areas.len() > MAX_SAFE_DETAILS {
                    return Err(ProtocolError::payload_too_large(
                        "state event contains too many changed areas",
                    ));
                }
                for area in areas {
                    validate_identifier("state area", area)?;
                }
                Ok(())
            }
            Self::ApprovalChanged { approval_id } => approval_id.validate(),
            Self::Error(error) => error.validate(),
            Self::RunChanged { .. } | Self::RuntimeHealthChanged { .. } => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub protocol_version: u16,
    pub event_id: RequestId,
    pub correlation_id: CorrelationId,
    pub generation: StateGeneration,
    pub run_scope: Option<RunScope>,
    pub event: CoreEvent,
}

/// Temporary Task 00001 snapshot command retained for the minimal launched
/// foundation route. Later service commands should use `RequestEnvelope`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SnapshotRequest {
    pub protocol_version: u16,
    pub request_id: RequestId,
    pub correlation_id: CorrelationId,
    pub expected_generation: StateGeneration,
}

/// Minimal authoritative payload used to prove the first Rust/renderer bridge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FoundationSnapshot {
    pub protocol_version: u16,
    pub generation: StateGeneration,
    pub authority: String,
    pub redactions: Vec<RedactionMarker>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceRecentSnapshot {
    pub workspace_id: WorkspaceId,
    pub display_name: String,
    pub last_opened_at: i64,
    pub is_missing: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WorkspaceStartSnapshot {
    pub protocol_version: u16,
    pub generation: StateGeneration,
    pub authority: String,
    pub recents: Vec<WorkspaceRecentSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationProjectSnapshot {
    pub project_id: ProjectId,
    pub display_name: String,
    pub path_state: String,
    pub position: i64,
    pub git_versioned: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationBranchSnapshot {
    pub name: String,
    pub target_oid: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationBranchControlSnapshot {
    pub current_branch: Option<String>,
    pub branches: Vec<ConversationBranchSnapshot>,
    pub pending_approval_id: Option<String>,
    pub operation_status: Option<String>,
    pub operation_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationSessionSummarySnapshot {
    pub session_id: SessionId,
    pub project_id: ProjectId,
    pub title: String,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationAttachmentSnapshot {
    pub attachment_id: AttachmentId,
    pub display_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub stable_reference: String,
    pub original_reference: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationAttachmentPreviewInput {
    pub attachment_id: AttachmentId,
    pub stable_reference: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationAttachmentPreviewSnapshot {
    pub attachment_id: AttachmentId,
    pub media_type: String,
    pub data_url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationTurnSnapshot {
    pub turn_id: TurnId,
    pub prompt: Option<String>,
    pub attachments: Vec<ConversationAttachmentSnapshot>,
    pub submitted_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationActivitySnapshot {
    pub sequence: u64,
    pub kind: String,
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationAttemptSnapshot {
    pub attempt_id: AttemptId,
    pub turn_id: TurnId,
    pub status: String,
    pub assistant_markdown: String,
    pub activities: Vec<ConversationActivitySnapshot>,
    pub runtime_id: RuntimeId,
    pub runtime_kind: String,
    pub environment_id: EnvironmentId,
    pub provider_id: String,
    pub model_id: String,
    pub adapter_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationSessionSnapshot {
    pub session_id: SessionId,
    pub title: Option<String>,
    pub turns: Vec<ConversationTurnSnapshot>,
    pub attempts: Vec<ConversationAttemptSnapshot>,
    pub active_attempt_id: Option<AttemptId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationModelSnapshot {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub selected: bool,
    pub available: bool,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_reasoning: bool,
    pub supports_audio: bool,
    pub context_tokens: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PendingConversationSnapshot {
    pub session_id: SessionId,
    pub project_id: ProjectId,
    pub title: String,
    pub attachments: Vec<ConversationAttachmentSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationDraftSnapshot {
    pub prompt: String,
    pub attachments: Vec<ConversationAttachmentSnapshot>,
    pub next_attachment_reference: u32,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
    pub mode: String,
    pub reply_target_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ConversationSnapshot {
    pub protocol_version: u16,
    pub generation: StateGeneration,
    pub authority: String,
    pub workspace_id: Option<WorkspaceId>,
    pub workspace_name: Option<String>,
    pub active_project_id: Option<ProjectId>,
    pub active_session_id: Option<SessionId>,
    pub pending: Option<PendingConversationSnapshot>,
    pub draft: ConversationDraftSnapshot,
    pub projects: Vec<ConversationProjectSnapshot>,
    pub sessions: Vec<ConversationSessionSummarySnapshot>,
    pub active_conversation: Option<ConversationSessionSnapshot>,
    pub models: Vec<ConversationModelSnapshot>,
    pub branch_control: Option<ConversationBranchControlSnapshot>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(rename_all = "kebab-case")]
pub enum ConversationBranchOperation {
    Switch,
    Create,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationBranchInput {
    pub operation: ConversationBranchOperation,
    pub branch: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(rename_all = "kebab-case")]
pub enum ConversationBranchApprovalAnswer {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationBranchApprovalInput {
    pub prompt_id: String,
    pub answer: ConversationBranchApprovalAnswer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationSubmitInput {
    pub prompt: Option<String>,
    pub picker_grant_ids: Vec<PickerGrantId>,
    pub retained_attachment_ids: Vec<AttachmentId>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
    pub resume_mode: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationDraftInput {
    pub prompt: String,
    pub picker_grant_ids: Vec<PickerGrantId>,
    pub retained_attachment_ids: Vec<AttachmentId>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
    pub mode: String,
    pub reply_target_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct ConversationRetryInput {
    pub parent_attempt_id: AttemptId,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_mode: Option<String>,
}

/// Generic response wrapper used by the minimal Task 00001 Tauri command.
/// Product commands use the non-generic `ResponseEnvelope` so generated
/// renderer types remain a closed union.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ProtocolEnvelope<T> {
    pub protocol_version: u16,
    pub request_id: RequestId,
    pub correlation_id: CorrelationId,
    pub generation: StateGeneration,
    pub payload: T,
}

pub type StructuredCoreError = ProtocolError;

/// Validates only the versioned request identity. Mutation commands perform
/// their own exact generation comparison against the complete authority they
/// are about to change; a synthetic response generation is not a valid proxy.
pub fn validate_snapshot_request(request: &SnapshotRequest) -> Result<(), StructuredCoreError> {
    validate_protocol_version(request.protocol_version)?;
    request.request_id.validate()?;
    request.correlation_id.validate()
}

pub(crate) fn snapshot_envelope<T>(
    request: SnapshotRequest,
    generation: StateGeneration,
    payload: T,
) -> Result<ProtocolEnvelope<T>, StructuredCoreError> {
    validate_snapshot_request(&request)?;
    if request.expected_generation > generation {
        return Err(generation_error(
            ProtocolErrorCode::FutureGeneration,
            request.expected_generation.0,
            generation.0,
        ));
    }
    Ok(ProtocolEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id,
        correlation_id: request.correlation_id,
        generation,
        payload,
    })
}

pub fn foundation_snapshot(
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<FoundationSnapshot>, StructuredCoreError> {
    validate_protocol_version(request.protocol_version)?;
    request.request_id.validate()?;
    request.correlation_id.validate()?;
    validate_exact_generation(request.expected_generation, StateGeneration::default())?;

    Ok(ProtocolEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id,
        correlation_id: request.correlation_id,
        generation: StateGeneration::default(),
        payload: FoundationSnapshot {
            protocol_version: PROTOCOL_VERSION,
            generation: StateGeneration::default(),
            authority: "rust-core".to_owned(),
            redactions: Vec::new(),
        },
    })
}

pub fn workspace_start_snapshot(
    request: SnapshotRequest,
    payload: WorkspaceStartSnapshot,
) -> Result<ProtocolEnvelope<WorkspaceStartSnapshot>, StructuredCoreError> {
    validate_protocol_version(request.protocol_version)?;
    request.request_id.validate()?;
    request.correlation_id.validate()?;
    validate_protocol_version(payload.protocol_version)?;
    if request.expected_generation > payload.generation {
        return Err(generation_error(
            ProtocolErrorCode::FutureGeneration,
            request.expected_generation.0,
            payload.generation.0,
        ));
    }
    if payload.authority != "rust-core" || payload.recents.len() > MAX_RECENT_WORKSPACES {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Workspace Start snapshot is invalid",
            false,
        ));
    }
    for recent in &payload.recents {
        recent.workspace_id.validate()?;
        let name = recent.display_name.trim();
        if name.is_empty()
            || name.len() > MAX_DISPLAY_NAME_BYTES
            || name.chars().any(char::is_control)
        {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Workspace display name is invalid",
                false,
            ));
        }
    }

    Ok(ProtocolEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id,
        correlation_id: request.correlation_id,
        generation: payload.generation,
        payload,
    })
}

pub fn conversation_snapshot(
    request: SnapshotRequest,
    payload: ConversationSnapshot,
) -> Result<ProtocolEnvelope<ConversationSnapshot>, StructuredCoreError> {
    validate_protocol_version(request.protocol_version)?;
    request.request_id.validate()?;
    request.correlation_id.validate()?;
    validate_protocol_version(payload.protocol_version)?;
    if request.expected_generation > payload.generation {
        return Err(generation_error(
            ProtocolErrorCode::FutureGeneration,
            request.expected_generation.0,
            payload.generation.0,
        ));
    }
    if payload.authority != "rust-core"
        || payload.projects.len() > 250
        || payload.sessions.len() > 250
        || payload.models.len() > 250
    {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Conversation snapshot is invalid",
            false,
        ));
    }
    if let Some(workspace_id) = &payload.workspace_id {
        workspace_id.validate()?;
    }
    if let Some(project_id) = &payload.active_project_id {
        project_id.validate()?;
    }
    if let Some(session_id) = &payload.active_session_id {
        session_id.validate()?;
    }
    for project in &payload.projects {
        project.project_id.validate()?;
        validate_display_text("Project display name", &project.display_name, 512)?;
        if !matches!(
            project.path_state.as_str(),
            "found" | "missing" | "relocated"
        ) {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Project path state is invalid",
                false,
            ));
        }
    }
    for session in &payload.sessions {
        session.session_id.validate()?;
        session.project_id.validate()?;
        validate_display_text("Chat title", &session.title, 512)?;
    }
    if let Some(pending) = &payload.pending {
        pending.session_id.validate()?;
        pending.project_id.validate()?;
        validate_display_text("pending Chat title", &pending.title, 512)?;
        if pending.attachments.len() > MAX_ATTACHMENTS {
            return Err(ProtocolError::payload_too_large(
                "pending attachment list exceeds its bound",
            ));
        }
        for attachment in &pending.attachments {
            validate_conversation_attachment(attachment)?;
        }
    }
    if payload.draft.prompt.len() > MAX_TEXT_BYTES || payload.draft.prompt.contains('\0') {
        return Err(ProtocolError::payload_too_large(
            "composer draft prompt exceeds its bound",
        ));
    }
    if payload.draft.attachments.len() > MAX_ATTACHMENTS {
        return Err(ProtocolError::payload_too_large(
            "composer draft attachment list exceeds its bound",
        ));
    }
    let mut draft_references = BTreeSet::new();
    for attachment in &payload.draft.attachments {
        validate_conversation_attachment(attachment)?;
        if attachment.original_reference >= payload.draft.next_attachment_reference
            || !draft_references.insert(attachment.original_reference)
        {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "composer draft attachment references are invalid",
                false,
            ));
        }
    }
    if payload.draft.next_attachment_reference == 0 {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "composer draft attachment counter is invalid",
            false,
        ));
    }
    for (label, value) in [
        ("draft provider", payload.draft.provider_id.as_deref()),
        ("draft model", payload.draft.model_id.as_deref()),
        (
            "draft Reply target",
            payload.draft.reply_target_id.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            validate_display_text(label, value, 512)?;
        }
    }
    if !matches!(
        payload.draft.mode.as_str(),
        "chat" | "files" | "browser" | "terminal"
    ) || payload
        .draft
        .reasoning_mode
        .as_deref()
        .is_some_and(|value| !matches!(value, "off" | "low" | "medium" | "high"))
    {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "composer draft controls are invalid",
            false,
        ));
    }
    let selected_models = payload.models.iter().filter(|model| model.selected).count();
    if selected_models > 1 {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "Conversation model selection is inconsistent",
            false,
        ));
    }
    for model in &payload.models {
        validate_identifier("Conversation provider", &model.provider_id)?;
        validate_display_text("Conversation provider name", &model.provider_name, 512)?;
        validate_display_text("Conversation model", &model.model_id, 512)?;
    }
    if let Some(branch_control) = &payload.branch_control {
        if branch_control.branches.len() > 4_096
            || branch_control
                .operation_status
                .as_deref()
                .is_some_and(|status| {
                    !matches!(
                        status,
                        "pending" | "switched" | "created" | "blocked" | "denied"
                    )
                })
        {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Git Branch control snapshot is invalid",
                false,
            ));
        }
        if let Some(current) = &branch_control.current_branch {
            validate_display_text("current Git branch", current, 255)?;
        }
        if let Some(message) = &branch_control.operation_message {
            validate_display_text("Git Branch operation message", message, 4_096)?;
        }
        if branch_control
            .pending_approval_id
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 512)
        {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Git Branch approval identity is invalid",
                false,
            ));
        }
        let selected = branch_control
            .branches
            .iter()
            .filter(|branch| branch.selected)
            .count();
        for branch in &branch_control.branches {
            validate_display_text("Git branch", &branch.name, 255)?;
            if !matches!(branch.target_oid.len(), 40 | 64)
                || !branch
                    .target_oid
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(ProtocolError::new(
                    ProtocolErrorCode::InvalidPayload,
                    "Git branch object identity is invalid",
                    false,
                ));
            }
        }
        if selected > 1 || branch_control.current_branch.is_some() != (selected == 1) {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "Git Branch selection is inconsistent",
                false,
            ));
        }
    }
    if let Some(active) = &payload.active_conversation {
        active.session_id.validate()?;
        if active.turns.len() > 4_096 || active.attempts.len() > 4_096 {
            return Err(ProtocolError::payload_too_large(
                "Conversation history exceeds its bound",
            ));
        }
        if let Some(active_attempt_id) = &active.active_attempt_id {
            active_attempt_id.validate()?;
        }
        for turn in &active.turns {
            turn.turn_id.validate()?;
            if turn.attachments.len() > MAX_ATTACHMENTS {
                return Err(ProtocolError::payload_too_large(
                    "Conversation attachment list exceeds its bound",
                ));
            }
            if turn
                .prompt
                .as_ref()
                .is_some_and(|prompt| prompt.len() > MAX_TEXT_BYTES)
            {
                return Err(ProtocolError::payload_too_large(
                    "Conversation prompt exceeds its bound",
                ));
            }
            for attachment in &turn.attachments {
                validate_conversation_attachment(attachment)?;
            }
        }
        for attempt in &active.attempts {
            attempt.attempt_id.validate()?;
            attempt.turn_id.validate()?;
            attempt.runtime_id.validate()?;
            attempt.environment_id.validate()?;
            if attempt.assistant_markdown.len() > MAX_TEXT_BYTES || attempt.activities.len() > 4_096
            {
                return Err(ProtocolError::payload_too_large(
                    "Conversation attempt exceeds its bound",
                ));
            }
        }
    }
    Ok(ProtocolEnvelope {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id,
        correlation_id: request.correlation_id,
        generation: payload.generation,
        payload,
    })
}

fn validate_conversation_attachment(
    attachment: &ConversationAttachmentSnapshot,
) -> Result<(), ProtocolError> {
    attachment.attachment_id.validate()?;
    validate_display_text("attachment display name", &attachment.display_name, 512)?;
    validate_display_text("attachment media type", &attachment.media_type, 128)?;
    validate_identifier("attachment stable reference", &attachment.stable_reference)?;
    if attachment.original_reference == 0
        || attachment.byte_length == 0
        || attachment.byte_length > 64 * 1024 * 1024
    {
        return Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            "attachment length is invalid",
            false,
        ));
    }
    Ok(())
}

fn validate_display_text(
    kind: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || value.len() > max_bytes
        || value.chars().any(|character| character == '\0')
    {
        Err(ProtocolError::new(
            ProtocolErrorCode::InvalidPayload,
            format!("{kind} is invalid"),
            false,
        ))
    } else {
        Ok(())
    }
}

impl EventEnvelope {
    pub fn validate(
        &self,
        minimum_generation: StateGeneration,
        expected_process_generation: Option<ProcessGeneration>,
    ) -> Result<(), ProtocolError> {
        validate_protocol_version(self.protocol_version)?;
        self.event_id.validate()?;
        self.correlation_id.validate()?;
        validate_not_stale(self.generation, minimum_generation)?;
        self.event.validate()?;

        if self.event.requires_run_scope() && self.run_scope.is_none() {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidPayload,
                "run-scoped event is missing its run identity",
                false,
            ));
        }

        if let Some(scope) = &self.run_scope {
            scope.validate()?;
            if scope.correlation_id != self.correlation_id {
                return Err(ProtocolError::new(
                    ProtocolErrorCode::CorrelationMismatch,
                    "event and run-scope correlations differ",
                    false,
                ));
            }
            if let Some(expected) = expected_process_generation {
                validate_exact_process_generation(scope.process_generation, expected)?;
            }
        }

        Ok(())
    }
}

pub fn validate_protocol_version(version: u16) -> Result<(), ProtocolError> {
    if version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorCode::UnknownProtocolVersion,
            format!("unsupported protocol version {version}"),
            false,
        ))
    }
}

fn validate_exact_generation(
    actual: StateGeneration,
    expected: StateGeneration,
) -> Result<(), ProtocolError> {
    match actual.cmp(&expected) {
        std::cmp::Ordering::Less => Err(generation_error(
            ProtocolErrorCode::StaleGeneration,
            actual.0,
            expected.0,
        )),
        std::cmp::Ordering::Greater => Err(generation_error(
            ProtocolErrorCode::FutureGeneration,
            actual.0,
            expected.0,
        )),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

fn validate_not_stale(
    actual: StateGeneration,
    minimum: StateGeneration,
) -> Result<(), ProtocolError> {
    if actual < minimum {
        Err(generation_error(
            ProtocolErrorCode::StaleGeneration,
            actual.0,
            minimum.0,
        ))
    } else {
        Ok(())
    }
}

fn validate_exact_process_generation(
    actual: ProcessGeneration,
    expected: ProcessGeneration,
) -> Result<(), ProtocolError> {
    match actual.cmp(&expected) {
        std::cmp::Ordering::Less => Err(generation_error(
            ProtocolErrorCode::StaleGeneration,
            actual.0,
            expected.0,
        )),
        std::cmp::Ordering::Greater => Err(generation_error(
            ProtocolErrorCode::FutureGeneration,
            actual.0,
            expected.0,
        )),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

fn generation_error(code: ProtocolErrorCode, actual: u64, expected: u64) -> ProtocolError {
    ProtocolError::new(code, "generation check failed", false)
        .with_detail("actual", SafeDetailValue::Unsigned(actual))
        .with_detail("expected", SafeDetailValue::Unsigned(expected))
}
