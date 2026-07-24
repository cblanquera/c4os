use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use thiserror::Error;

use super::record::{self, ArtifactResourceVersion, MAX_ARTIFACT_DISPLAY_BYTES};

pub const MAX_FILE_CONTENT_BYTES: usize = 1_048_576;
pub const MAX_FILE_DIFF_BYTES: usize = 1_048_576;
pub const MAX_FILE_HISTORY_ENTRIES: usize = 128;
pub const MAX_PROJECT_RELATIVE_PATH_BYTES: usize = 4_096;
pub const FILE_REPLY_PROPOSAL_BEGIN: &str = "<c4os-file-proposal>";
pub const FILE_REPLY_PROPOSAL_END: &str = "</c4os-file-proposal>";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileResourceReference {
    /// Slash-delimited path below the authoritative Project root. It is a
    /// display/identity reference, never ambient filesystem authority.
    pub project_relative_path: String,
    pub display_path: String,
}

impl FileResourceReference {
    pub fn new(
        project_relative_path: impl Into<String>,
        display_path: impl Into<String>,
    ) -> Result<Self, FileStateError> {
        let reference = Self {
            project_relative_path: project_relative_path.into(),
            display_path: display_path.into(),
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn validate(&self) -> Result<(), FileStateError> {
        validate_project_relative_path(&self.project_relative_path)?;
        validate_bounded_text(
            "file display path",
            &self.display_path,
            MAX_PROJECT_RELATIVE_PATH_BYTES,
            false,
        )?;
        if self.display_path.chars().any(char::is_control) {
            return Err(FileStateError::InvalidDisplayPath);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileLiveVersion {
    pub sequence: u64,
    /// Exact opaque descriptor identity returned by
    /// `ProjectFilesystem::FileVersion::target_version()`. This remains
    /// distinct from the content digest so same-content replacements still
    /// invalidate stale saves.
    pub target_version: String,
    pub content_sha256: String,
    pub byte_length: u64,
    pub observed_at_ms: u64,
}

impl FileLiveVersion {
    pub fn new(
        sequence: u64,
        content_sha256: impl Into<String>,
        byte_length: u64,
        observed_at_ms: u64,
    ) -> Result<Self, FileStateError> {
        let content_sha256 = content_sha256.into();
        Self::new_with_target_version(
            sequence,
            content_sha256.clone(),
            content_sha256,
            byte_length,
            observed_at_ms,
        )
    }

    /// Production constructor that preserves the exact filesystem descriptor
    /// identity independently of the file's content digest.
    pub fn new_with_target_version(
        sequence: u64,
        target_version: impl Into<String>,
        content_sha256: impl Into<String>,
        byte_length: u64,
        observed_at_ms: u64,
    ) -> Result<Self, FileStateError> {
        let version = Self {
            sequence,
            target_version: target_version.into(),
            content_sha256: content_sha256.into(),
            byte_length,
            observed_at_ms,
        };
        version.validate()?;
        Ok(version)
    }

    pub fn validate(&self) -> Result<(), FileStateError> {
        if self.sequence == 0 || self.observed_at_ms == 0 {
            return Err(FileStateError::InvalidLiveVersion);
        }
        validate_sha256(&self.target_version)?;
        validate_sha256(&self.content_sha256)?;
        Ok(())
    }

    pub fn as_resource_version(&self) -> ArtifactResourceVersion {
        ArtifactResourceVersion {
            sequence: self.sequence,
            sha256: self.target_version.clone(),
            observed_at_ms: self.observed_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileDraft {
    pub content: String,
    pub base_live_version: FileLiveVersion,
    pub dirty: bool,
    pub updated_at_ms: u64,
}

impl FileDraft {
    fn validate(&self, current_content: &str) -> Result<(), FileStateError> {
        validate_file_content(&self.content)?;
        self.base_live_version.validate()?;
        if self.updated_at_ms == 0 || self.dirty != (self.content != current_content) {
            return Err(FileStateError::InvalidDraft);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileProposalStatus {
    Pending,
    Rejected { rejected_at_ms: u64 },
    Applied { applied_at_ms: u64 },
    Conflict { observed_at_ms: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileProposal {
    pub proposal_id: String,
    pub base_live_version: FileLiveVersion,
    pub proposed_content: String,
    pub unified_diff: String,
    pub status: FileProposalStatus,
    pub created_at_ms: u64,
}

impl FileProposal {
    pub fn validate(&self) -> Result<(), FileStateError> {
        validate_identifier("file proposal id", &self.proposal_id)?;
        self.base_live_version.validate()?;
        validate_file_content(&self.proposed_content)?;
        validate_bounded_text(
            "file proposal diff",
            &self.unified_diff,
            MAX_FILE_DIFF_BYTES,
            true,
        )?;
        if self.created_at_ms == 0 {
            return Err(FileStateError::InvalidProposal);
        }
        match &self.status {
            FileProposalStatus::Rejected { rejected_at_ms }
                if *rejected_at_ms < self.created_at_ms =>
            {
                Err(FileStateError::InvalidProposal)
            }
            FileProposalStatus::Applied { applied_at_ms }
                if *applied_at_ms < self.created_at_ms =>
            {
                Err(FileStateError::InvalidProposal)
            }
            FileProposalStatus::Conflict { observed_at_ms }
                if *observed_at_ms < self.created_at_ms =>
            {
                Err(FileStateError::InvalidProposal)
            }
            _ => Ok(()),
        }
    }
}

/// Extracts the one machine-readable proposal envelope emitted for a File
/// Reply. Ordinary assistant prose is never interpreted as file content, and
/// duplicate or malformed envelopes fail closed without mutating the
/// artifact. JSON preserves exact leading/trailing whitespace in the file.
pub fn file_reply_proposed_content(markdown: &str) -> Option<String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ProposalEnvelope {
        content: String,
    }

    let start = markdown.find(FILE_REPLY_PROPOSAL_BEGIN)?;
    if markdown[start + FILE_REPLY_PROPOSAL_BEGIN.len()..].contains(FILE_REPLY_PROPOSAL_BEGIN) {
        return None;
    }
    let payload_start = start + FILE_REPLY_PROPOSAL_BEGIN.len();
    let relative_end = markdown[payload_start..].find(FILE_REPLY_PROPOSAL_END)?;
    let payload_end = payload_start + relative_end;
    if markdown[payload_end + FILE_REPLY_PROPOSAL_END.len()..].contains(FILE_REPLY_PROPOSAL_END) {
        return None;
    }
    let envelope =
        serde_json::from_str::<ProposalEnvelope>(markdown[payload_start..payload_end].trim())
            .ok()?;
    validate_file_content(&envelope.content).ok()?;
    Some(envelope.content)
}

/// Produces a bounded unified-style diff without invoking Git or accepting a
/// renderer-authored patch. Common leading/trailing lines are collapsed so
/// the durable proposal remains useful for review even for larger files.
pub fn file_reply_unified_diff(path: &str, current: &str, proposed: &str) -> String {
    if current == proposed {
        return String::new();
    }
    let current_lines = current.split_inclusive('\n').collect::<Vec<_>>();
    let proposed_lines = proposed.split_inclusive('\n').collect::<Vec<_>>();
    let mut prefix = 0;
    while prefix < current_lines.len()
        && prefix < proposed_lines.len()
        && current_lines[prefix] == proposed_lines[prefix]
    {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < current_lines.len().saturating_sub(prefix)
        && suffix < proposed_lines.len().saturating_sub(prefix)
        && current_lines[current_lines.len() - 1 - suffix]
            == proposed_lines[proposed_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let current_end = current_lines.len().saturating_sub(suffix);
    let proposed_end = proposed_lines.len().saturating_sub(suffix);
    let mut diff = format!(
        "--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n",
        prefix + 1,
        current_end.saturating_sub(prefix),
        prefix + 1,
        proposed_end.saturating_sub(prefix),
    );
    for line in &current_lines[prefix..current_end] {
        diff.push('-');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
    for line in &proposed_lines[prefix..proposed_end] {
        diff.push('+');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
    if diff.len() > MAX_FILE_DIFF_BYTES {
        let suffix = "\n[diff truncated by C4OS]\n";
        let mut boundary = MAX_FILE_DIFF_BYTES.saturating_sub(suffix.len());
        while !diff.is_char_boundary(boundary) {
            boundary = boundary.saturating_sub(1);
        }
        diff.truncate(boundary);
        diff.push_str(suffix);
    }
    diff
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileConflict {
    pub expected_base: FileLiveVersion,
    pub observed_live: FileLiveVersion,
    pub attempted_content_sha256: String,
    pub message: String,
    pub observed_at_ms: u64,
}

impl FileConflict {
    fn validate(&self) -> Result<(), FileStateError> {
        self.expected_base.validate()?;
        self.observed_live.validate()?;
        validate_sha256(&self.attempted_content_sha256)?;
        validate_bounded_text("file conflict message", &self.message, 8 * 1_024, false)?;
        if self.observed_at_ms < self.observed_live.observed_at_ms
            || self.observed_live.sequence <= self.expected_base.sequence
        {
            return Err(FileStateError::InvalidConflict);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "state")]
pub enum FileRecoveryState {
    SaveFailed {
        code: String,
        message: String,
        retryable: bool,
        failed_at_ms: u64,
    },
    Restored {
        from_live_version: FileLiveVersion,
        restored_at_ms: u64,
    },
}

impl FileRecoveryState {
    fn validate(&self) -> Result<(), FileStateError> {
        match self {
            Self::SaveFailed {
                code,
                message,
                failed_at_ms,
                ..
            } => {
                validate_identifier("file recovery code", code)?;
                validate_bounded_text("file recovery message", message, 8 * 1_024, false)?;
                if *failed_at_ms == 0 {
                    return Err(FileStateError::InvalidRecovery);
                }
                Ok(())
            }
            Self::Restored {
                from_live_version,
                restored_at_ms,
            } => {
                from_live_version.validate()?;
                if *restored_at_ms == 0 {
                    return Err(FileStateError::InvalidRecovery);
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileVersionOrigin {
    Opened,
    Refreshed,
    DirectSave,
    ProposalSave,
    Recovery,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileVersionHistoryEntry {
    pub live_version: FileLiveVersion,
    pub origin: FileVersionOrigin,
    pub recorded_at_ms: u64,
}

impl FileVersionHistoryEntry {
    fn validate(&self) -> Result<(), FileStateError> {
        self.live_version.validate()?;
        if self.recorded_at_ms == 0 {
            return Err(FileStateError::InvalidHistory);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileArtifactState {
    pub resource: FileResourceReference,
    pub display_name: String,
    pub media_type: String,
    /// Bounded UTF-8 snapshot. It is not a renderer filesystem capability.
    pub content: String,
    pub live_version: FileLiveVersion,
    pub draft: Option<FileDraft>,
    pub proposal: Option<FileProposal>,
    pub conflict: Option<FileConflict>,
    pub recovery: Option<FileRecoveryState>,
    /// Durable marker for a save request that has crossed into the security
    /// gateway but has not reached a terminal result. The action and approval
    /// remain intentionally non-durable; restart recovery revalidates this
    /// retained candidate before issuing a fresh approval prompt.
    #[serde(default)]
    pub pending_save_requested_at_ms: Option<u64>,
    pub version_history: Vec<FileVersionHistoryEntry>,
}

impl FileArtifactState {
    pub fn new(
        resource: FileResourceReference,
        display_name: impl Into<String>,
        media_type: impl Into<String>,
        content: impl Into<String>,
        live_version: FileLiveVersion,
    ) -> Result<Self, FileStateError> {
        let state = Self {
            resource,
            display_name: display_name.into(),
            media_type: media_type.into(),
            content: content.into(),
            live_version,
            draft: None,
            proposal: None,
            conflict: None,
            recovery: None,
            pending_save_requested_at_ms: None,
            version_history: Vec::new(),
        };
        state.validate()?;
        Ok(state)
    }

    pub fn validate(&self) -> Result<(), FileStateError> {
        self.resource.validate()?;
        validate_bounded_text(
            "file display name",
            &self.display_name,
            MAX_ARTIFACT_DISPLAY_BYTES,
            false,
        )?;
        if self.display_name.chars().any(char::is_control)
            || self.display_name.contains('/')
            || self.display_name.contains('\\')
        {
            return Err(FileStateError::InvalidDisplayName);
        }
        validate_media_type(&self.media_type)?;
        validate_file_content(&self.content)?;
        self.live_version.validate()?;
        if self.live_version.byte_length != self.content.len() as u64
            || self.live_version.content_sha256 != sha256_text(&self.content)
        {
            return Err(FileStateError::ContentVersionMismatch);
        }
        if let Some(draft) = &self.draft {
            draft.validate(&self.content)?;
            if draft.base_live_version != self.live_version {
                return Err(FileStateError::InvalidDraft);
            }
        }
        if let Some(proposal) = &self.proposal {
            proposal.validate()?;
            if matches!(&proposal.status, FileProposalStatus::Pending)
                && proposal.base_live_version != self.live_version
            {
                return Err(FileStateError::StaleProposal);
            }
        }
        if let Some(conflict) = &self.conflict {
            conflict.validate()?;
        }
        if let Some(recovery) = &self.recovery {
            recovery.validate()?;
        }
        if let Some(requested_at_ms) = self.pending_save_requested_at_ms {
            let (_, _, _, candidate_at_ms) = self.save_candidate()?;
            if requested_at_ms < candidate_at_ms {
                return Err(FileStateError::InvalidSaveRequest);
            }
        }
        self.validate_history()
    }

    pub fn begin_draft(&mut self, now_ms: u64) -> Result<(), FileStateError> {
        if now_ms < self.live_version.observed_at_ms {
            return Err(FileStateError::InvalidTimestamp);
        }
        self.draft = Some(FileDraft {
            content: self.content.clone(),
            base_live_version: self.live_version.clone(),
            dirty: false,
            updated_at_ms: now_ms,
        });
        self.conflict = None;
        self.recovery = None;
        self.pending_save_requested_at_ms = None;
        Ok(())
    }

    pub fn update_draft(
        &mut self,
        content: impl Into<String>,
        now_ms: u64,
    ) -> Result<(), FileStateError> {
        let content = content.into();
        validate_file_content(&content)?;
        let draft = self.draft.as_mut().ok_or(FileStateError::DraftNotStarted)?;
        if now_ms < draft.updated_at_ms {
            return Err(FileStateError::InvalidTimestamp);
        }
        draft.content = content;
        draft.dirty = draft.content != self.content;
        draft.updated_at_ms = now_ms;
        self.conflict = None;
        self.recovery = None;
        self.pending_save_requested_at_ms = None;
        Ok(())
    }

    pub fn discard_draft(&mut self) {
        self.draft = None;
        self.conflict = None;
        self.recovery = None;
        self.pending_save_requested_at_ms = None;
    }

    pub fn install_proposal(&mut self, proposal: FileProposal) -> Result<(), FileStateError> {
        proposal.validate()?;
        if proposal.base_live_version != self.live_version
            || !matches!(&proposal.status, FileProposalStatus::Pending)
            || proposal.created_at_ms < self.live_version.observed_at_ms
        {
            return Err(FileStateError::StaleProposal);
        }
        self.proposal = Some(proposal);
        self.conflict = None;
        self.recovery = None;
        self.pending_save_requested_at_ms = None;
        Ok(())
    }

    pub fn reject_proposal(&mut self, rejected_at_ms: u64) -> Result<(), FileStateError> {
        let proposal = self
            .proposal
            .as_mut()
            .ok_or(FileStateError::ProposalNotPending)?;
        if !matches!(&proposal.status, FileProposalStatus::Pending) {
            return Err(FileStateError::ProposalNotPending);
        }
        if rejected_at_ms < proposal.created_at_ms {
            return Err(FileStateError::InvalidTimestamp);
        }
        proposal.status = FileProposalStatus::Rejected { rejected_at_ms };
        self.pending_save_requested_at_ms = None;
        Ok(())
    }

    pub fn mark_save_requested(&mut self, requested_at_ms: u64) -> Result<(), FileStateError> {
        let (_, _, _, candidate_at_ms) = self.save_candidate()?;
        if requested_at_ms < candidate_at_ms {
            return Err(FileStateError::InvalidTimestamp);
        }
        self.pending_save_requested_at_ms = Some(requested_at_ms);
        Ok(())
    }

    pub fn apply_save_result(&mut self, result: FileSaveResult) -> Result<(), FileStateError> {
        let (expected_base, attempted_content, proposal_save, candidate_at_ms) =
            self.save_candidate()?;
        match result {
            FileSaveResult::Saved {
                content,
                live_version,
                saved_at_ms,
            } => {
                if content != attempted_content
                    || expected_base != self.live_version
                    || live_version.sequence <= self.live_version.sequence
                    || live_version.observed_at_ms > saved_at_ms
                    || saved_at_ms < candidate_at_ms
                {
                    return Err(FileStateError::InvalidSaveResult);
                }
                validate_file_content(&content)?;
                live_version.validate()?;
                if live_version.byte_length != content.len() as u64
                    || live_version.content_sha256 != sha256_text(&content)
                {
                    return Err(FileStateError::ContentVersionMismatch);
                }
                self.push_history(FileVersionHistoryEntry {
                    live_version: self.live_version.clone(),
                    origin: if proposal_save {
                        FileVersionOrigin::ProposalSave
                    } else {
                        FileVersionOrigin::DirectSave
                    },
                    recorded_at_ms: saved_at_ms,
                });
                if proposal_save && let Some(proposal) = &mut self.proposal {
                    proposal.status = FileProposalStatus::Applied {
                        applied_at_ms: saved_at_ms,
                    };
                } else {
                    // A direct draft save supersedes any proposal based on the
                    // prior live snapshot; retaining it as pending would make
                    // the serialized state stale and ambiguous.
                    self.proposal = None;
                }
                self.content = content;
                self.live_version = live_version;
                self.draft = None;
                self.conflict = None;
                self.recovery = None;
                self.pending_save_requested_at_ms = None;
                Ok(())
            }
            FileSaveResult::Conflict {
                observed_live,
                message,
                observed_at_ms,
            } => {
                observed_live.validate()?;
                if observed_at_ms < observed_live.observed_at_ms
                    || observed_at_ms < candidate_at_ms
                    || observed_live.sequence <= expected_base.sequence
                {
                    return Err(FileStateError::InvalidSaveResult);
                }
                let conflict = FileConflict {
                    expected_base,
                    observed_live,
                    attempted_content_sha256: sha256_text(&attempted_content),
                    message,
                    observed_at_ms,
                };
                conflict.validate()?;
                if proposal_save && let Some(proposal) = &mut self.proposal {
                    proposal.status = FileProposalStatus::Conflict { observed_at_ms };
                }
                self.conflict = Some(conflict);
                self.pending_save_requested_at_ms = None;
                Ok(())
            }
            FileSaveResult::Failed {
                code,
                message,
                retryable,
                failed_at_ms,
            } => {
                if failed_at_ms < candidate_at_ms {
                    return Err(FileStateError::InvalidSaveResult);
                }
                let recovery = FileRecoveryState::SaveFailed {
                    code,
                    message,
                    retryable,
                    failed_at_ms,
                };
                recovery.validate()?;
                self.recovery = Some(recovery);
                self.pending_save_requested_at_ms = None;
                Ok(())
            }
        }
    }

    pub fn restore_live_snapshot(
        &mut self,
        content: impl Into<String>,
        live_version: FileLiveVersion,
        restored_at_ms: u64,
    ) -> Result<(), FileStateError> {
        let content = content.into();
        validate_file_content(&content)?;
        live_version.validate()?;
        if live_version.sequence <= self.live_version.sequence
            || live_version.observed_at_ms > restored_at_ms
            || live_version.byte_length != content.len() as u64
            || live_version.content_sha256 != sha256_text(&content)
        {
            return Err(FileStateError::InvalidSaveResult);
        }
        let prior = self.live_version.clone();
        self.push_history(FileVersionHistoryEntry {
            live_version: prior.clone(),
            origin: FileVersionOrigin::Recovery,
            recorded_at_ms: restored_at_ms,
        });
        self.content = content;
        self.live_version = live_version;
        self.draft = None;
        self.proposal = None;
        self.conflict = None;
        self.recovery = Some(FileRecoveryState::Restored {
            from_live_version: prior,
            restored_at_ms,
        });
        self.pending_save_requested_at_ms = None;
        Ok(())
    }

    fn save_candidate(&self) -> Result<(FileLiveVersion, String, bool, u64), FileStateError> {
        if let Some(proposal) = &self.proposal
            && matches!(&proposal.status, FileProposalStatus::Pending)
        {
            return Ok((
                proposal.base_live_version.clone(),
                proposal.proposed_content.clone(),
                true,
                proposal.created_at_ms,
            ));
        }
        if let Some(draft) = &self.draft
            && draft.dirty
        {
            return Ok((
                draft.base_live_version.clone(),
                draft.content.clone(),
                false,
                draft.updated_at_ms,
            ));
        }
        Err(FileStateError::NoSaveCandidate)
    }

    fn push_history(&mut self, entry: FileVersionHistoryEntry) {
        if self.version_history.len() == MAX_FILE_HISTORY_ENTRIES {
            self.version_history.remove(0);
        }
        self.version_history.push(entry);
    }

    fn validate_history(&self) -> Result<(), FileStateError> {
        if self.version_history.len() > MAX_FILE_HISTORY_ENTRIES {
            return Err(FileStateError::HistoryBoundExceeded);
        }
        let mut previous_sequence = 0;
        for entry in &self.version_history {
            entry.validate()?;
            if entry.live_version.sequence <= previous_sequence
                || entry.live_version.sequence >= self.live_version.sequence
            {
                return Err(FileStateError::InvalidHistory);
            }
            previous_sequence = entry.live_version.sequence;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileSaveResult {
    Saved {
        content: String,
        live_version: FileLiveVersion,
        saved_at_ms: u64,
    },
    Conflict {
        observed_live: FileLiveVersion,
        message: String,
        observed_at_ms: u64,
    },
    Failed {
        code: String,
        message: String,
        retryable: bool,
        failed_at_ms: u64,
    },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum FileStateError {
    #[error("invalid File artifact identifier: {0}")]
    InvalidIdentifier(&'static str),
    #[error("invalid bounded File artifact text: {0}")]
    InvalidText(&'static str),
    #[error("invalid File artifact SHA-256 value")]
    InvalidSha256,
    #[error("invalid Project-relative file path")]
    InvalidProjectRelativePath,
    #[error("invalid file display path")]
    InvalidDisplayPath,
    #[error("invalid file display name")]
    InvalidDisplayName,
    #[error("invalid bounded UTF-8 file content")]
    InvalidContent,
    #[error("invalid file media type")]
    InvalidMediaType,
    #[error("invalid file live version")]
    InvalidLiveVersion,
    #[error("file content and live version disagree")]
    ContentVersionMismatch,
    #[error("invalid pending File save request")]
    InvalidSaveRequest,
    #[error("invalid file draft")]
    InvalidDraft,
    #[error("file draft has not started")]
    DraftNotStarted,
    #[error("invalid file proposal")]
    InvalidProposal,
    #[error("file proposal is stale")]
    StaleProposal,
    #[error("file proposal is not pending")]
    ProposalNotPending,
    #[error("file has no unsaved save candidate")]
    NoSaveCandidate,
    #[error("invalid file save result")]
    InvalidSaveResult,
    #[error("invalid file conflict")]
    InvalidConflict,
    #[error("invalid file recovery state")]
    InvalidRecovery,
    #[error("file version history exceeds its bound")]
    HistoryBoundExceeded,
    #[error("invalid file version history")]
    InvalidHistory,
    #[error("invalid file transition timestamp")]
    InvalidTimestamp,
}

fn validate_identifier(label: &'static str, value: &str) -> Result<(), FileStateError> {
    record::validate_identifier(label, value).map_err(|_| FileStateError::InvalidIdentifier(label))
}

fn validate_bounded_text(
    label: &'static str,
    value: &str,
    maximum_bytes: usize,
    allow_empty: bool,
) -> Result<(), FileStateError> {
    record::validate_bounded_text(label, value, maximum_bytes, allow_empty)
        .map_err(|_| FileStateError::InvalidText(label))
}

fn validate_sha256(value: &str) -> Result<(), FileStateError> {
    record::validate_sha256(value).map_err(|_| FileStateError::InvalidSha256)
}

pub(crate) fn validate_project_relative_path(path: &str) -> Result<(), FileStateError> {
    if path.is_empty()
        || path.len() > MAX_PROJECT_RELATIVE_PATH_BYTES
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains('\0')
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || segment.chars().any(char::is_control)
        })
    {
        Err(FileStateError::InvalidProjectRelativePath)
    } else {
        Ok(())
    }
}

pub(crate) fn validate_file_content(content: &str) -> Result<(), FileStateError> {
    if content.len() > MAX_FILE_CONTENT_BYTES || content.contains('\0') {
        Err(FileStateError::InvalidContent)
    } else {
        Ok(())
    }
}

fn validate_media_type(media_type: &str) -> Result<(), FileStateError> {
    let valid = !media_type.is_empty()
        && media_type.len() <= 256
        && media_type.split_once('/').is_some_and(|(kind, subtype)| {
            !kind.is_empty()
                && !subtype.is_empty()
                && kind
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'+' | b'.'))
                && subtype
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'+' | b'.'))
        });
    if valid {
        Ok(())
    } else {
        Err(FileStateError::InvalidMediaType)
    }
}

pub(crate) fn sha256_text(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in hasher.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(sequence: u64, content: &str, observed_at_ms: u64) -> FileLiveVersion {
        FileLiveVersion::new(
            sequence,
            sha256_text(content),
            content.len() as u64,
            observed_at_ms,
        )
        .unwrap()
    }

    fn state() -> FileArtifactState {
        FileArtifactState::new(
            FileResourceReference::new("docs/plan.md", "docs/plan.md").unwrap(),
            "plan.md",
            "text/markdown",
            "original",
            version(1, "original", 10),
        )
        .unwrap()
    }

    #[test]
    fn production_live_version_keeps_target_identity_separate_from_content() {
        let content_sha256 = sha256_text("same content");
        let target_version = format!("sha256:{}", "f".repeat(64));
        let version = FileLiveVersion::new_with_target_version(
            3,
            target_version.clone(),
            content_sha256.clone(),
            "same content".len() as u64,
            20,
        )
        .unwrap();

        assert_eq!(version.content_sha256, content_sha256);
        assert_eq!(version.target_version, target_version);
        assert_eq!(version.as_resource_version().sha256, target_version);
    }

    #[test]
    fn rejects_ambient_or_traversing_resource_references() {
        for path in ["/tmp/secret", "../secret", "docs/../secret", "docs\\secret"] {
            assert_eq!(
                FileResourceReference::new(path, path),
                Err(FileStateError::InvalidProjectRelativePath)
            );
        }
    }

    #[test]
    fn draft_discard_restores_the_authoritative_snapshot() {
        let mut file = state();
        file.begin_draft(20).unwrap();
        file.update_draft("edited", 21).unwrap();
        assert!(file.draft.as_ref().unwrap().dirty);
        assert_eq!(file.content, "original");
        file.discard_draft();
        assert_eq!(file.draft, None);
        assert_eq!(file.content, "original");
    }

    #[test]
    fn successful_save_advances_live_version_without_conflating_record_revision() {
        let mut file = state();
        file.begin_draft(20).unwrap();
        file.update_draft("edited", 21).unwrap();
        file.apply_save_result(FileSaveResult::Saved {
            content: "edited".into(),
            live_version: version(2, "edited", 22),
            saved_at_ms: 22,
        })
        .unwrap();

        assert_eq!(file.content, "edited");
        assert_eq!(file.live_version.sequence, 2);
        assert_eq!(file.version_history[0].live_version.sequence, 1);
        assert_eq!(file.draft, None);
        file.validate().unwrap();
    }

    #[test]
    fn pending_save_request_survives_serialization_and_clears_on_terminal_result() {
        let mut file = state();
        file.begin_draft(20).unwrap();
        file.update_draft("edited", 21).unwrap();
        file.mark_save_requested(22).unwrap();

        let document = serde_json::to_string(&file).unwrap();
        let mut restored: FileArtifactState = serde_json::from_str(&document).unwrap();
        assert_eq!(restored.pending_save_requested_at_ms, Some(22));
        restored
            .apply_save_result(FileSaveResult::Failed {
                code: "approval-denied".into(),
                message: "cancelled".into(),
                retryable: false,
                failed_at_ms: 23,
            })
            .unwrap();
        assert_eq!(restored.pending_save_requested_at_ms, None);
        assert_eq!(restored.draft.as_ref().unwrap().content, "edited");
        restored.validate().unwrap();
    }

    #[test]
    fn conflict_retains_unsaved_draft_and_records_both_versions() {
        let mut file = state();
        file.begin_draft(20).unwrap();
        file.update_draft("edited", 21).unwrap();
        file.apply_save_result(FileSaveResult::Conflict {
            observed_live: version(2, "external", 22),
            message: "The file changed on disk".into(),
            observed_at_ms: 22,
        })
        .unwrap();

        assert_eq!(file.draft.as_ref().unwrap().content, "edited");
        let conflict = file.conflict.as_ref().unwrap();
        assert_eq!(conflict.expected_base.sequence, 1);
        assert_eq!(conflict.observed_live.sequence, 2);
        assert_eq!(file.content, "original");
    }

    #[test]
    fn proposal_requires_the_current_live_version() {
        let mut file = state();
        let stale = FileProposal {
            proposal_id: "proposal-1".into(),
            base_live_version: version(2, "other", 20),
            proposed_content: "replacement".into(),
            unified_diff: "-original\n+replacement".into(),
            status: FileProposalStatus::Pending,
            created_at_ms: 20,
        };
        assert_eq!(
            file.install_proposal(stale),
            Err(FileStateError::StaleProposal)
        );
    }

    #[test]
    fn reply_proposal_envelope_preserves_exact_content_and_rejects_ambiguity() {
        let markdown = concat!(
            "I prepared the change.\n",
            "<c4os-file-proposal>{\"content\":\"  first\\nsecond\\n\"}</c4os-file-proposal>"
        );
        assert_eq!(
            file_reply_proposed_content(markdown).as_deref(),
            Some("  first\nsecond\n")
        );
        assert!(
            file_reply_proposed_content(
                "<c4os-file-proposal>{\"content\":\"one\"}</c4os-file-proposal>\
                 <c4os-file-proposal>{\"content\":\"two\"}</c4os-file-proposal>"
            )
            .is_none()
        );
        assert!(file_reply_proposed_content("ordinary assistant prose").is_none());
    }

    #[test]
    fn reply_diff_is_bounded_and_proposal_wins_over_a_retained_draft() {
        let mut file = state();
        file.begin_draft(20).unwrap();
        file.update_draft("user draft", 21).unwrap();
        let proposed = "agent proposal";
        let diff = file_reply_unified_diff("docs/plan.md", &file.content, proposed);
        assert!(diff.contains("--- a/docs/plan.md"));
        assert!(diff.contains("-original"));
        assert!(diff.contains("+agent proposal"));
        assert!(diff.len() <= MAX_FILE_DIFF_BYTES);
        file.install_proposal(FileProposal {
            proposal_id: "proposal-current".into(),
            base_live_version: file.live_version.clone(),
            proposed_content: proposed.into(),
            unified_diff: diff,
            status: FileProposalStatus::Pending,
            created_at_ms: 22,
        })
        .unwrap();
        file.apply_save_result(FileSaveResult::Saved {
            content: proposed.into(),
            live_version: version(2, proposed, 23),
            saved_at_ms: 23,
        })
        .unwrap();
        assert_eq!(file.content, proposed);
        assert!(matches!(
            file.proposal.as_ref().map(|proposal| &proposal.status),
            Some(FileProposalStatus::Applied { .. })
        ));
        assert_eq!(file.draft, None);
    }
}
