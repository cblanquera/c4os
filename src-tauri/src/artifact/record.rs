use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

use super::file::FileArtifactState;
use super::folder::FolderArtifactState;

pub const ARTIFACT_SCHEMA_VERSION: u16 = 1;
pub const ARTIFACT_PROVIDER_SCHEMA_VERSION: u16 = 1;
pub const MAX_ARTIFACT_IDENTIFIER_BYTES: usize = 160;
pub const MAX_ARTIFACT_DISPLAY_BYTES: usize = 512;
pub const MAX_ARTIFACT_MESSAGE_BYTES: usize = 8 * 1_024;
pub const MAX_ARTIFACT_HISTORY_ENTRIES: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactFocusCapability {
    InlineOnly,
    Focusable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactProviderDescriptor {
    pub schema_version: u16,
    pub type_id: String,
    pub label: String,
    pub accessible_name: String,
    pub focus: ArtifactFocusCapability,
}

impl ArtifactProviderDescriptor {
    pub fn file() -> Self {
        Self {
            schema_version: ARTIFACT_PROVIDER_SCHEMA_VERSION,
            type_id: "file".into(),
            label: "File".into(),
            accessible_name: "File artifact".into(),
            focus: ArtifactFocusCapability::Focusable,
        }
    }

    pub fn folder() -> Self {
        Self {
            schema_version: ARTIFACT_PROVIDER_SCHEMA_VERSION,
            type_id: "folder".into(),
            label: "Folder".into(),
            accessible_name: "Folder artifact".into(),
            focus: ArtifactFocusCapability::Focusable,
        }
    }

    pub fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.schema_version == 0 {
            return Err(ArtifactRecordError::InvalidProvider(
                "provider schema version must be non-zero",
            ));
        }
        validate_identifier("provider type id", &self.type_id)?;
        validate_bounded_text(
            "provider label",
            &self.label,
            MAX_ARTIFACT_DISPLAY_BYTES,
            false,
        )?;
        validate_bounded_text(
            "provider accessible name",
            &self.accessible_name,
            MAX_ARTIFACT_DISPLAY_BYTES,
            false,
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "source")]
pub enum ArtifactSource {
    DirectOperation { operation_id: String },
    RunAttempt { run_id: String, attempt_id: String },
}

impl ArtifactSource {
    fn validate(&self) -> Result<(), ArtifactRecordError> {
        match self {
            Self::DirectOperation { operation_id } => {
                validate_identifier("direct operation id", operation_id)
            }
            Self::RunAttempt { run_id, attempt_id } => {
                validate_identifier("source run id", run_id)?;
                validate_identifier("source attempt id", attempt_id)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactResourceVersion {
    pub sequence: u64,
    pub sha256: String,
    pub observed_at_ms: u64,
}

impl ArtifactResourceVersion {
    pub fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.sequence == 0 || self.observed_at_ms == 0 {
            return Err(ArtifactRecordError::InvalidResourceVersion);
        }
        validate_sha256(&self.sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum ArtifactLifecycle {
    Loading {
        started_at_ms: u64,
    },
    Ready,
    Error {
        code: String,
        message: String,
        retryable: bool,
    },
    Degraded {
        code: String,
        message: String,
    },
    UnknownVersion {
        provider_schema_version: u16,
    },
}

impl ArtifactLifecycle {
    fn validate(&self) -> Result<(), ArtifactRecordError> {
        match self {
            Self::Loading { started_at_ms } if *started_at_ms == 0 => Err(
                ArtifactRecordError::InvalidLifecycle("loading timestamp must be non-zero"),
            ),
            Self::Error { code, message, .. } | Self::Degraded { code, message } => {
                validate_identifier("artifact lifecycle code", code)?;
                validate_bounded_text(
                    "artifact lifecycle message",
                    message,
                    MAX_ARTIFACT_MESSAGE_BYTES,
                    false,
                )
            }
            Self::UnknownVersion {
                provider_schema_version,
            } if *provider_schema_version == 0 => Err(ArtifactRecordError::InvalidLifecycle(
                "unknown provider version must be non-zero",
            )),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UnknownArtifactState {
    pub provider_schema_version: u16,
    pub state_sha256: String,
    pub resource_version: ArtifactResourceVersion,
}

impl UnknownArtifactState {
    fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.provider_schema_version == 0 {
            return Err(ArtifactRecordError::InvalidProvider(
                "unknown state provider version must be non-zero",
            ));
        }
        validate_sha256(&self.state_sha256)?;
        self.resource_version.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase",
    tag = "type",
    content = "value"
)]
pub enum ArtifactState {
    File(Box<FileArtifactState>),
    Folder(Box<FolderArtifactState>),
    Unknown(UnknownArtifactState),
}

impl ArtifactState {
    pub fn resource_version(&self) -> ArtifactResourceVersion {
        match self {
            Self::File(file) => file.live_version.as_resource_version(),
            Self::Folder(folder) => folder.listing_version.as_resource_version(),
            Self::Unknown(unknown) => unknown.resource_version.clone(),
        }
    }

    fn validate(&self) -> Result<(), ArtifactRecordError> {
        match self {
            Self::File(file) => file.validate().map_err(ArtifactRecordError::File),
            Self::Folder(folder) => folder.validate().map_err(ArtifactRecordError::Folder),
            Self::Unknown(unknown) => unknown.validate(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactHistoryKind {
    Created,
    ResourceRefreshed,
    DraftChanged,
    DraftDiscarded,
    ProposalChanged,
    SaveRequested,
    SaveCompleted,
    ConflictObserved,
    RecoveryChanged,
    NavigationChanged,
    Converted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactHistoryEntry {
    pub record_revision: u64,
    pub resource_version: ArtifactResourceVersion,
    pub state_sha256: String,
    pub kind: ArtifactHistoryKind,
    pub recorded_at_ms: u64,
}

impl ArtifactHistoryEntry {
    fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.record_revision == 0 || self.recorded_at_ms == 0 {
            return Err(ArtifactRecordError::InvalidHistory);
        }
        self.resource_version.validate()?;
        validate_sha256(&self.state_sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactRecord {
    pub schema_version: u16,
    pub artifact_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub provider: ArtifactProviderDescriptor,
    pub source: ArtifactSource,
    /// CAS identity for the artifact record itself.
    pub record_revision: u64,
    pub lifecycle: ArtifactLifecycle,
    pub state: ArtifactState,
    pub history: Vec<ArtifactHistoryEntry>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

/// One Workspace-owned focus selection. A scalar identity structurally
/// prevents two artifacts from being focused at once. Persistence integration
/// must validate the referenced record with [`Self::validate_against`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactWorkspaceUiState {
    pub schema_version: u16,
    pub workspace_id: String,
    pub revision: u64,
    pub focused_artifact_id: Option<String>,
    pub updated_at_ms: u64,
}

impl ArtifactWorkspaceUiState {
    pub fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.schema_version != ARTIFACT_SCHEMA_VERSION {
            return Err(ArtifactRecordError::UnsupportedSchema);
        }
        validate_identifier("artifact UI Workspace id", &self.workspace_id)?;
        if self.revision == 0 || self.updated_at_ms == 0 {
            return Err(ArtifactRecordError::InvalidUiState);
        }
        if let Some(artifact_id) = &self.focused_artifact_id {
            validate_identifier("focused artifact id", artifact_id)?;
        }
        Ok(())
    }

    pub fn validate_against(&self, artifact: &ArtifactRecord) -> Result<(), ArtifactRecordError> {
        self.validate()?;
        artifact.validate()?;
        if self.workspace_id != artifact.workspace_id
            || self.focused_artifact_id.as_deref() != Some(artifact.artifact_id.as_str())
            || artifact.provider.focus != ArtifactFocusCapability::Focusable
            || matches!(
                &artifact.lifecycle,
                ArtifactLifecycle::Loading { .. } | ArtifactLifecycle::UnknownVersion { .. }
            )
        {
            return Err(ArtifactRecordError::InvalidUiState);
        }
        Ok(())
    }

    pub fn focus(
        &mut self,
        artifact: &ArtifactRecord,
        updated_at_ms: u64,
    ) -> Result<(), ArtifactRecordError> {
        self.validate()?;
        artifact.validate()?;
        if self.workspace_id != artifact.workspace_id
            || artifact.provider.focus != ArtifactFocusCapability::Focusable
            || matches!(
                &artifact.lifecycle,
                ArtifactLifecycle::Loading { .. } | ArtifactLifecycle::UnknownVersion { .. }
            )
            || updated_at_ms < self.updated_at_ms
        {
            return Err(ArtifactRecordError::InvalidUiState);
        }
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(ArtifactRecordError::RevisionOverflow)?;
        self.focused_artifact_id = Some(artifact.artifact_id.clone());
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }

    pub fn clear_focus(&mut self, updated_at_ms: u64) -> Result<(), ArtifactRecordError> {
        self.validate()?;
        if updated_at_ms < self.updated_at_ms {
            return Err(ArtifactRecordError::InvalidUiState);
        }
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(ArtifactRecordError::RevisionOverflow)?;
        self.focused_artifact_id = None;
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }
}

impl ArtifactRecord {
    pub fn validate(&self) -> Result<(), ArtifactRecordError> {
        if self.schema_version != ARTIFACT_SCHEMA_VERSION {
            return Err(ArtifactRecordError::UnsupportedSchema);
        }
        for (label, value) in [
            ("artifact id", self.artifact_id.as_str()),
            ("workspace id", self.workspace_id.as_str()),
            ("project id", self.project_id.as_str()),
            ("session id", self.session_id.as_str()),
        ] {
            validate_identifier(label, value)?;
        }
        if self.record_revision == 0
            || self.created_at_ms == 0
            || self.updated_at_ms < self.created_at_ms
        {
            return Err(ArtifactRecordError::InvalidRecordIdentity);
        }
        self.provider.validate()?;
        self.source.validate()?;
        self.lifecycle.validate()?;
        self.state.validate()?;
        self.validate_provider_state_pair()?;
        self.validate_history()
    }

    pub fn resource_version(&self) -> ArtifactResourceVersion {
        self.state.resource_version()
    }

    pub fn append_history(
        &mut self,
        entry: ArtifactHistoryEntry,
    ) -> Result<(), ArtifactRecordError> {
        entry.validate()?;
        if entry.record_revision != self.record_revision
            || entry.recorded_at_ms < self.created_at_ms
            || self
                .history
                .last()
                .is_some_and(|current| current.record_revision >= entry.record_revision)
        {
            return Err(ArtifactRecordError::InvalidHistory);
        }
        if self.history.len() == MAX_ARTIFACT_HISTORY_ENTRIES {
            self.history.remove(0);
        }
        self.history.push(entry);
        Ok(())
    }

    fn validate_provider_state_pair(&self) -> Result<(), ArtifactRecordError> {
        match (&self.state, self.provider.type_id.as_str()) {
            (ArtifactState::File(_), "file") | (ArtifactState::Folder(_), "folder")
                if self.provider.schema_version == ARTIFACT_PROVIDER_SCHEMA_VERSION
                    && !matches!(&self.lifecycle, ArtifactLifecycle::UnknownVersion { .. }) =>
            {
                Ok(())
            }
            (ArtifactState::Unknown(unknown), _) => {
                if !matches!(
                    &self.lifecycle,
                    ArtifactLifecycle::UnknownVersion {
                        provider_schema_version
                    } if *provider_schema_version == unknown.provider_schema_version
                ) || self.provider.schema_version != unknown.provider_schema_version
                    || self.provider.focus != ArtifactFocusCapability::InlineOnly
                {
                    return Err(ArtifactRecordError::ProviderStateMismatch);
                }
                Ok(())
            }
            _ => Err(ArtifactRecordError::ProviderStateMismatch),
        }
    }

    fn validate_history(&self) -> Result<(), ArtifactRecordError> {
        if self.history.len() > MAX_ARTIFACT_HISTORY_ENTRIES {
            return Err(ArtifactRecordError::HistoryBoundExceeded);
        }
        let mut revisions = BTreeSet::new();
        let mut previous_revision = 0;
        for entry in &self.history {
            entry.validate()?;
            if entry.record_revision > self.record_revision
                || entry.recorded_at_ms < self.created_at_ms
                || entry.recorded_at_ms > self.updated_at_ms
                || entry.record_revision <= previous_revision
                || !revisions.insert(entry.record_revision)
            {
                return Err(ArtifactRecordError::InvalidHistory);
            }
            previous_revision = entry.record_revision;
        }
        Ok(())
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ArtifactRecordError {
    #[error("unsupported artifact schema")]
    UnsupportedSchema,
    #[error("invalid artifact identifier: {0}")]
    InvalidIdentifier(&'static str),
    #[error("invalid bounded artifact text: {0}")]
    InvalidText(&'static str),
    #[error("invalid SHA-256 value")]
    InvalidSha256,
    #[error("invalid provider: {0}")]
    InvalidProvider(&'static str),
    #[error("invalid artifact record identity")]
    InvalidRecordIdentity,
    #[error("invalid live resource version")]
    InvalidResourceVersion,
    #[error("invalid artifact lifecycle: {0}")]
    InvalidLifecycle(&'static str),
    #[error("artifact provider and state disagree")]
    ProviderStateMismatch,
    #[error("artifact history exceeds its bound")]
    HistoryBoundExceeded,
    #[error("invalid artifact history")]
    InvalidHistory,
    #[error("invalid artifact Workspace UI state")]
    InvalidUiState,
    #[error("artifact revision overflow")]
    RevisionOverflow,
    #[error(transparent)]
    File(#[from] super::file::FileStateError),
    #[error(transparent)]
    Folder(#[from] super::folder::FolderStateError),
}

pub(crate) fn validate_identifier(
    label: &'static str,
    value: &str,
) -> Result<(), ArtifactRecordError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_ARTIFACT_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        });
    if valid {
        Ok(())
    } else {
        Err(ArtifactRecordError::InvalidIdentifier(label))
    }
}

pub(crate) fn validate_bounded_text(
    label: &'static str,
    value: &str,
    maximum_bytes: usize,
    allow_empty: bool,
) -> Result<(), ArtifactRecordError> {
    if value.len() > maximum_bytes
        || (!allow_empty && value.trim().is_empty())
        || value.contains('\0')
    {
        Err(ArtifactRecordError::InvalidText(label))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_sha256(value: &str) -> Result<(), ArtifactRecordError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(ArtifactRecordError::InvalidSha256);
    };
    if hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ArtifactRecordError::InvalidSha256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::file::{
        FileArtifactState, FileLiveVersion, FileResourceReference, sha256_text,
    };

    fn digest(character: char) -> String {
        format!("sha256:{}", character.to_string().repeat(64))
    }

    fn file_state() -> FileArtifactState {
        FileArtifactState::new(
            FileResourceReference::new("src/main.rs", "src/main.rs").unwrap(),
            "main.rs",
            "text/rust",
            "fn main() {}",
            FileLiveVersion::new(1, sha256_text("fn main() {}"), 12, 10).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn validates_distinct_record_and_live_resource_revisions() {
        let record = ArtifactRecord {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-1".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: ArtifactProviderDescriptor::file(),
            source: ArtifactSource::DirectOperation {
                operation_id: "operation-1".into(),
            },
            record_revision: 7,
            lifecycle: ArtifactLifecycle::Ready,
            state: ArtifactState::File(Box::new(file_state())),
            history: vec![ArtifactHistoryEntry {
                record_revision: 1,
                resource_version: ArtifactResourceVersion {
                    sequence: 1,
                    sha256: digest('a'),
                    observed_at_ms: 10,
                },
                state_sha256: digest('b'),
                kind: ArtifactHistoryKind::Created,
                recorded_at_ms: 10,
            }],
            created_at_ms: 10,
            updated_at_ms: 20,
        };

        record.validate().unwrap();
        assert_eq!(record.record_revision, 7);
        assert_eq!(record.resource_version().sequence, 1);
    }

    #[test]
    fn rejects_provider_state_mismatch() {
        let mut record = ArtifactRecord {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-1".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: ArtifactProviderDescriptor::folder(),
            source: ArtifactSource::DirectOperation {
                operation_id: "operation-1".into(),
            },
            record_revision: 1,
            lifecycle: ArtifactLifecycle::Ready,
            state: ArtifactState::File(Box::new(file_state())),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
        };
        assert_eq!(
            record.validate(),
            Err(ArtifactRecordError::ProviderStateMismatch)
        );
        record.provider = ArtifactProviderDescriptor::file();
        record.validate().unwrap();
        record.provider.schema_version = ARTIFACT_PROVIDER_SCHEMA_VERSION + 1;
        assert_eq!(
            record.validate(),
            Err(ArtifactRecordError::ProviderStateMismatch)
        );
    }

    #[test]
    fn unknown_versions_are_inline_only_and_fail_closed() {
        let mut provider = ArtifactProviderDescriptor {
            schema_version: 9,
            type_id: "future-provider".into(),
            label: "Future".into(),
            accessible_name: "Future artifact".into(),
            focus: ArtifactFocusCapability::InlineOnly,
        };
        let mut record = ArtifactRecord {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-1".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: provider.clone(),
            source: ArtifactSource::RunAttempt {
                run_id: "run-1".into(),
                attempt_id: "attempt-1".into(),
            },
            record_revision: 1,
            lifecycle: ArtifactLifecycle::UnknownVersion {
                provider_schema_version: 9,
            },
            state: ArtifactState::Unknown(UnknownArtifactState {
                provider_schema_version: 9,
                state_sha256: digest('f'),
                resource_version: ArtifactResourceVersion {
                    sequence: 3,
                    sha256: digest('e'),
                    observed_at_ms: 10,
                },
            }),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
        };
        record.validate().unwrap();
        provider.focus = ArtifactFocusCapability::Focusable;
        record.provider = provider;
        assert_eq!(
            record.validate(),
            Err(ArtifactRecordError::ProviderStateMismatch)
        );
    }

    #[test]
    fn workspace_ui_state_structurally_owns_exactly_one_focus() {
        let record = ArtifactRecord {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            artifact_id: "artifact-1".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider: ArtifactProviderDescriptor::file(),
            source: ArtifactSource::DirectOperation {
                operation_id: "operation-1".into(),
            },
            record_revision: 1,
            lifecycle: ArtifactLifecycle::Ready,
            state: ArtifactState::File(Box::new(file_state())),
            history: Vec::new(),
            created_at_ms: 10,
            updated_at_ms: 10,
        };
        let mut ui = ArtifactWorkspaceUiState {
            schema_version: ARTIFACT_SCHEMA_VERSION,
            workspace_id: "workspace-1".into(),
            revision: 1,
            focused_artifact_id: None,
            updated_at_ms: 10,
        };
        ui.focus(&record, 20).unwrap();
        ui.validate_against(&record).unwrap();
        assert_eq!(ui.focused_artifact_id.as_deref(), Some("artifact-1"));
        ui.clear_focus(21).unwrap();
        assert_eq!(ui.focused_artifact_id, None);
        assert_eq!(
            ui.focus(&record, 20),
            Err(ArtifactRecordError::InvalidUiState)
        );
    }
}
