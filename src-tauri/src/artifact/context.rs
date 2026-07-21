use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

use super::file::{FileArtifactState, FileStateError};
use super::folder::{FolderArtifactState, FolderEntryKind, FolderStateError};
use super::record::{
    ARTIFACT_PROVIDER_SCHEMA_VERSION, ArtifactRecordError, ArtifactResourceVersion,
    validate_identifier,
};

pub const ARTIFACT_CONTEXT_SCHEMA_VERSION: u16 = 1;
pub const MAX_CONTEXT_INPUT_BYTES: usize = 8 * 1_024 * 1_024;
pub const MAX_CONTEXT_BUDGET_BYTES: usize = 4 * 1_024 * 1_024;
pub const MAX_CONTEXT_SEGMENTS: usize = 32;
pub const MAX_CONTEXT_REDACTIONS: usize = 64;
pub const MAX_CONTEXT_CAPABILITIES: usize = 128;
pub const MAX_STABLE_REFERENCE_BYTES: usize = 512;

/// Text that has already crossed its provider's redaction/normalization
/// boundary. The type deliberately exposes no raw browser storage, credential,
/// terminal environment, password, or unrelated-history fields.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SafeContextText(String);

impl SafeContextText {
    pub fn new(value: impl Into<String>) -> Result<Self, ArtifactContextError> {
        let value = value.into();
        if value.len() > MAX_CONTEXT_INPUT_BYTES
            || value.contains('\0')
            || value
                .chars()
                .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
        {
            return Err(ArtifactContextError::UnsafeContextText);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self) -> Result<(), ArtifactContextError> {
        Self::new(self.0.clone()).map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextPriority {
    Selection,
    VisibleOrCurrent,
    Recent,
    Metadata,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContextSegment {
    pub priority: ContextPriority,
    pub source: String,
    pub text: String,
    pub original_bytes: u64,
    pub omitted_bytes: u64,
}

impl ContextSegment {
    fn validate(&self) -> Result<(), ArtifactContextError> {
        validate_identifier("artifact context segment source", &self.source)?;
        if self.text.contains('\0')
            || self
                .text
                .chars()
                .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            || self.original_bytes != self.text.len() as u64 + self.omitted_bytes
        {
            return Err(ArtifactContextError::InvalidSegment);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContextBudget {
    pub maximum_bytes: u64,
    pub used_bytes: u64,
    pub omitted_bytes: u64,
    pub omitted_segments: u32,
    pub truncated: bool,
}

impl ContextBudget {
    fn validate(&self, segments: &[ContextSegment]) -> Result<(), ArtifactContextError> {
        if self.maximum_bytes == 0
            || self.maximum_bytes > MAX_CONTEXT_BUDGET_BYTES as u64
            || self.used_bytes > self.maximum_bytes
            || self.used_bytes
                != segments
                    .iter()
                    .map(|segment| segment.text.len() as u64)
                    .sum::<u64>()
            || self.omitted_bytes
                != segments
                    .iter()
                    .map(|segment| segment.omitted_bytes)
                    .sum::<u64>()
            || self.omitted_segments
                != segments
                    .iter()
                    .filter(|segment| segment.text.is_empty() && segment.original_bytes > 0)
                    .count() as u32
            || self.truncated != (self.omitted_bytes > 0)
        {
            return Err(ArtifactContextError::InvalidBudget);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextRedaction {
    Secrets,
    BrowserCredentials,
    BrowserStorage,
    BrowserUnrelatedHistory,
    TerminalRawEnvironment,
    TerminalPasswords,
    TerminalUnrelatedHistory,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityAccess {
    Readable,
    ApprovalRequired,
    Denied,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CapabilitySummaryEntry {
    pub capability_id: String,
    pub access: CapabilityAccess,
    pub reason_code: Option<String>,
}

impl CapabilitySummaryEntry {
    fn validate(&self) -> Result<(), ArtifactContextError> {
        validate_identifier("artifact context capability id", &self.capability_id)?;
        if let Some(reason) = &self.reason_code {
            validate_identifier("artifact context capability reason", reason)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextCaptureIdentity {
    pub snapshot_id: String,
    pub artifact_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub provider_type: String,
    pub provider_schema_version: u16,
    pub artifact_record_revision: u64,
    pub live_resource_version: ArtifactResourceVersion,
}

impl ContextCaptureIdentity {
    fn validate(&self) -> Result<(), ArtifactContextError> {
        for (label, value) in [
            ("artifact context snapshot id", self.snapshot_id.as_str()),
            ("artifact context artifact id", self.artifact_id.as_str()),
            ("artifact context Workspace id", self.workspace_id.as_str()),
            ("artifact context Project id", self.project_id.as_str()),
            ("artifact context session id", self.session_id.as_str()),
            (
                "artifact context provider type",
                self.provider_type.as_str(),
            ),
        ] {
            validate_identifier(label, value)?;
        }
        if self.provider_schema_version == 0 || self.artifact_record_revision == 0 {
            return Err(ArtifactContextError::InvalidIdentity);
        }
        self.live_resource_version.validate()?;
        Ok(())
    }

    fn stable_reference(&self) -> String {
        format!(
            "artifact:{}:record:{}:resource:{}:{}",
            self.artifact_id,
            self.artifact_record_revision,
            self.live_resource_version.sequence,
            self.live_resource_version.sha256
        )
    }
}

#[derive(Clone, Debug)]
pub struct FileContextInput<'a> {
    pub state: &'a FileArtifactState,
    pub selected_text: Option<SafeContextText>,
    pub visible_text: Option<SafeContextText>,
    pub recent_text: Option<SafeContextText>,
    pub redactions: Vec<ContextRedaction>,
    pub capabilities: Vec<CapabilitySummaryEntry>,
}

#[derive(Clone, Debug)]
pub struct FolderContextInput<'a> {
    pub state: &'a FolderArtifactState,
    pub recent_text: Option<SafeContextText>,
    pub redactions: Vec<ContextRedaction>,
    pub capabilities: Vec<CapabilitySummaryEntry>,
}

/// Safe Browser input intentionally cannot carry cookies, credentials,
/// storage, screenshots, or unrelated history.
#[derive(Clone, Debug)]
pub struct BrowserContextInput {
    pub current_url: SafeContextText,
    pub title: SafeContextText,
    pub navigation_state: SafeContextText,
    pub selected_text: Option<SafeContextText>,
    pub visible_text: Option<SafeContextText>,
    pub extracted_content: Option<SafeContextText>,
    pub recent_activity: Option<SafeContextText>,
    pub capabilities: Vec<CapabilitySummaryEntry>,
}

/// Safe Terminal input intentionally cannot carry raw environment values,
/// passwords, credential prompts, or unrelated shell history.
#[derive(Clone, Debug)]
pub struct TerminalContextInput {
    pub command: SafeContextText,
    pub working_directory_display: SafeContextText,
    pub environment_id: String,
    pub process_state: String,
    pub exit_code: Option<i32>,
    pub selected_output: Option<SafeContextText>,
    pub visible_output: Option<SafeContextText>,
    pub recent_output_tail: Option<SafeContextText>,
    pub capabilities: Vec<CapabilitySummaryEntry>,
}

#[derive(Clone, Debug)]
pub enum ContextCaptureInput<'a> {
    File(FileContextInput<'a>),
    Folder(FolderContextInput<'a>),
    Browser(BrowserContextInput),
    Terminal(TerminalContextInput),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase",
    tag = "type",
    content = "value"
)]
pub enum ContextPayload {
    File {
        had_selection: bool,
        captured_unsaved_draft: bool,
        segments: Vec<ContextSegment>,
    },
    Folder {
        selected_entry_id: Option<String>,
        listed_entry_count: u32,
        segments: Vec<ContextSegment>,
    },
    Browser {
        had_selection: bool,
        segments: Vec<ContextSegment>,
    },
    Terminal {
        had_selection: bool,
        exit_code: Option<i32>,
        segments: Vec<ContextSegment>,
    },
}

impl ContextPayload {
    fn segments(&self) -> &[ContextSegment] {
        match self {
            Self::File { segments, .. }
            | Self::Folder { segments, .. }
            | Self::Browser { segments, .. }
            | Self::Terminal { segments, .. } => segments,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactContextSnapshot {
    pub schema_version: u16,
    pub snapshot_id: String,
    pub stable_reference: String,
    pub artifact_id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub session_id: String,
    pub provider_type: String,
    pub provider_schema_version: u16,
    pub artifact_record_revision: u64,
    pub captured_live_version: ArtifactResourceVersion,
    pub payload: ContextPayload,
    pub budget: ContextBudget,
    pub unsaved: bool,
    pub redactions: Vec<ContextRedaction>,
    pub capabilities: Vec<CapabilitySummaryEntry>,
    pub captured_at_ms: u64,
}

impl ArtifactContextSnapshot {
    pub fn validate(&self) -> Result<(), ArtifactContextError> {
        if self.schema_version != ARTIFACT_CONTEXT_SCHEMA_VERSION {
            return Err(ArtifactContextError::UnsupportedSchema);
        }
        let identity = ContextCaptureIdentity {
            snapshot_id: self.snapshot_id.clone(),
            artifact_id: self.artifact_id.clone(),
            workspace_id: self.workspace_id.clone(),
            project_id: self.project_id.clone(),
            session_id: self.session_id.clone(),
            provider_type: self.provider_type.clone(),
            provider_schema_version: self.provider_schema_version,
            artifact_record_revision: self.artifact_record_revision,
            live_resource_version: self.captured_live_version.clone(),
        };
        identity.validate()?;
        if self.captured_at_ms < self.captured_live_version.observed_at_ms
            || self.stable_reference != identity.stable_reference()
            || self.stable_reference.len() > MAX_STABLE_REFERENCE_BYTES
            || self.payload.segments().len() > MAX_CONTEXT_SEGMENTS
        {
            return Err(ArtifactContextError::InvalidSnapshot);
        }
        let expected_provider = match &self.payload {
            ContextPayload::File { .. } => "file",
            ContextPayload::Folder { .. } => "folder",
            ContextPayload::Browser { .. } => "browser",
            ContextPayload::Terminal { .. } => "terminal",
        };
        if self.provider_type != expected_provider
            || self.unsaved
                != matches!(
                    &self.payload,
                    ContextPayload::File {
                        captured_unsaved_draft: true,
                        ..
                    }
                )
        {
            return Err(ArtifactContextError::InvalidSnapshot);
        }
        validate_segments(self.payload.segments())?;
        self.budget.validate(self.payload.segments())?;
        validate_redactions(&self.redactions)?;
        validate_capabilities(&self.capabilities)
    }
}

pub fn capture_artifact_context(
    identity: ContextCaptureIdentity,
    input: ContextCaptureInput<'_>,
    maximum_bytes: usize,
    captured_at_ms: u64,
) -> Result<ArtifactContextSnapshot, ArtifactContextError> {
    identity.validate()?;
    if maximum_bytes == 0
        || maximum_bytes > MAX_CONTEXT_BUDGET_BYTES
        || captured_at_ms < identity.live_resource_version.observed_at_ms
        || identity.stable_reference().len() > MAX_STABLE_REFERENCE_BYTES
    {
        return Err(ArtifactContextError::InvalidBudget);
    }
    validate_capture_binding(&identity, &input)?;

    let captured = match input {
        ContextCaptureInput::File(input) => capture_file(input, maximum_bytes)?,
        ContextCaptureInput::Folder(input) => capture_folder(input, maximum_bytes)?,
        ContextCaptureInput::Browser(input) => capture_browser(input, maximum_bytes)?,
        ContextCaptureInput::Terminal(input) => capture_terminal(input, maximum_bytes)?,
    };
    let stable_reference = identity.stable_reference();
    let snapshot = ArtifactContextSnapshot {
        schema_version: ARTIFACT_CONTEXT_SCHEMA_VERSION,
        snapshot_id: identity.snapshot_id,
        stable_reference,
        artifact_id: identity.artifact_id,
        workspace_id: identity.workspace_id,
        project_id: identity.project_id,
        session_id: identity.session_id,
        provider_type: identity.provider_type,
        provider_schema_version: identity.provider_schema_version,
        artifact_record_revision: identity.artifact_record_revision,
        captured_live_version: identity.live_resource_version,
        payload: captured.payload,
        budget: captured.budget,
        unsaved: captured.unsaved,
        redactions: captured.redactions,
        capabilities: captured.capabilities,
        captured_at_ms,
    };
    snapshot.validate()?;
    Ok(snapshot)
}

fn validate_capture_binding(
    identity: &ContextCaptureIdentity,
    input: &ContextCaptureInput<'_>,
) -> Result<(), ArtifactContextError> {
    if identity.provider_schema_version != ARTIFACT_PROVIDER_SCHEMA_VERSION {
        return Err(ArtifactContextError::CaptureBindingMismatch);
    }
    match input {
        ContextCaptureInput::File(input) => {
            if identity.provider_type != "file"
                || identity.live_resource_version != input.state.live_version.as_resource_version()
            {
                return Err(ArtifactContextError::CaptureBindingMismatch);
            }
            validate_optional_safe(&input.selected_text)?;
            validate_optional_safe(&input.visible_text)?;
            validate_optional_safe(&input.recent_text)
        }
        ContextCaptureInput::Folder(input) => {
            if identity.provider_type != "folder"
                || identity.live_resource_version
                    != input.state.listing_version.as_resource_version()
            {
                return Err(ArtifactContextError::CaptureBindingMismatch);
            }
            validate_optional_safe(&input.recent_text)
        }
        ContextCaptureInput::Browser(input) => {
            if identity.provider_type != "browser" {
                return Err(ArtifactContextError::CaptureBindingMismatch);
            }
            input.current_url.validate()?;
            input.title.validate()?;
            input.navigation_state.validate()?;
            validate_optional_safe(&input.selected_text)?;
            validate_optional_safe(&input.visible_text)?;
            validate_optional_safe(&input.extracted_content)?;
            validate_optional_safe(&input.recent_activity)
        }
        ContextCaptureInput::Terminal(input) => {
            if identity.provider_type != "terminal" {
                return Err(ArtifactContextError::CaptureBindingMismatch);
            }
            input.command.validate()?;
            input.working_directory_display.validate()?;
            validate_optional_safe(&input.selected_output)?;
            validate_optional_safe(&input.visible_output)?;
            validate_optional_safe(&input.recent_output_tail)
        }
    }
}

fn validate_optional_safe(value: &Option<SafeContextText>) -> Result<(), ArtifactContextError> {
    value.as_ref().map_or(Ok(()), SafeContextText::validate)
}

struct CapturedPayload {
    payload: ContextPayload,
    budget: ContextBudget,
    unsaved: bool,
    redactions: Vec<ContextRedaction>,
    capabilities: Vec<CapabilitySummaryEntry>,
}

fn capture_file(
    input: FileContextInput<'_>,
    maximum_bytes: usize,
) -> Result<CapturedPayload, ArtifactContextError> {
    input.state.validate()?;
    let had_selection = input.selected_text.is_some();
    let unsaved = input.state.draft.as_ref().is_some_and(|draft| draft.dirty);
    let current = input
        .state
        .draft
        .as_ref()
        .filter(|draft| draft.dirty)
        .map_or(input.state.content.as_str(), |draft| draft.content.as_str());
    let current = SafeContextText::new(current)?;
    let metadata = SafeContextText::new(format!(
        "path={}\nname={}\nmediaType={}\nresourceVersion={}\nunsaved={unsaved}",
        input.state.resource.project_relative_path,
        input.state.display_name,
        input.state.media_type,
        input.state.live_version.sequence,
    ))?;
    let candidates = vec![
        candidate(
            ContextPriority::Selection,
            "selected-text",
            input.selected_text,
        ),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "visible-text",
            input.visible_text,
        ),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "current-document",
            Some(current),
        ),
        candidate(
            ContextPriority::Recent,
            "recent-file-state",
            input.recent_text,
        ),
        candidate(ContextPriority::Metadata, "file-metadata", Some(metadata)),
    ];
    let (segments, budget) = allocate_segments(candidates, maximum_bytes);
    Ok(CapturedPayload {
        payload: ContextPayload::File {
            had_selection,
            captured_unsaved_draft: unsaved,
            segments,
        },
        budget,
        unsaved,
        redactions: normalize_redactions(input.redactions)?,
        capabilities: normalize_capabilities(input.capabilities)?,
    })
}

fn capture_folder(
    input: FolderContextInput<'_>,
    maximum_bytes: usize,
) -> Result<CapturedPayload, ArtifactContextError> {
    input.state.validate()?;
    let selected = input.state.selection.as_ref().and_then(|selection| {
        input
            .state
            .entries
            .iter()
            .find(|entry| entry.entry_id == selection.entry_id)
    });
    let selected_text = selected
        .map(|entry| {
            SafeContextText::new(format!(
                "{}\t{}\t{}",
                match entry.kind {
                    FolderEntryKind::Folder => "folder",
                    FolderEntryKind::File => "file",
                },
                entry.name,
                entry.project_relative_path
            ))
        })
        .transpose()?;
    let listing = SafeContextText::new(
        input
            .state
            .entries
            .iter()
            .map(|entry| {
                format!(
                    "{}\t{}\t{}",
                    match entry.kind {
                        FolderEntryKind::Folder => "folder",
                        FolderEntryKind::File => "file",
                    },
                    entry.name,
                    entry.project_relative_path
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )?;
    let breadcrumbs = input
        .state
        .breadcrumbs
        .iter()
        .map(|breadcrumb| breadcrumb.label.as_str())
        .collect::<Vec<_>>()
        .join(" / ");
    let metadata = SafeContextText::new(format!(
        "path={}\nbreadcrumbs={}\nlistingVersion={}\nentryCount={}",
        input.state.resource.project_relative_path,
        breadcrumbs,
        input.state.listing_version.sequence,
        input.state.entries.len()
    ))?;
    let candidates = vec![
        candidate(ContextPriority::Selection, "selected-entry", selected_text),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "current-listing",
            Some(listing),
        ),
        candidate(
            ContextPriority::Recent,
            "recent-folder-state",
            input.recent_text,
        ),
        candidate(ContextPriority::Metadata, "folder-metadata", Some(metadata)),
    ];
    let (segments, budget) = allocate_segments(candidates, maximum_bytes);
    Ok(CapturedPayload {
        payload: ContextPayload::Folder {
            selected_entry_id: selected.map(|entry| entry.entry_id.clone()),
            listed_entry_count: input.state.entries.len() as u32,
            segments,
        },
        budget,
        unsaved: false,
        redactions: normalize_redactions(input.redactions)?,
        capabilities: normalize_capabilities(input.capabilities)?,
    })
}

fn capture_browser(
    input: BrowserContextInput,
    maximum_bytes: usize,
) -> Result<CapturedPayload, ArtifactContextError> {
    let had_selection = input.selected_text.is_some();
    let metadata = SafeContextText::new(format!(
        "url={}\ntitle={}\nnavigationState={}",
        input.current_url.as_str(),
        input.title.as_str(),
        input.navigation_state.as_str()
    ))?;
    let candidates = vec![
        candidate(
            ContextPriority::Selection,
            "selected-text",
            input.selected_text,
        ),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "visible-text",
            input.visible_text,
        ),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "extracted-content",
            input.extracted_content,
        ),
        candidate(
            ContextPriority::Recent,
            "recent-navigation",
            input.recent_activity,
        ),
        candidate(
            ContextPriority::Metadata,
            "browser-metadata",
            Some(metadata),
        ),
    ];
    let (segments, budget) = allocate_segments(candidates, maximum_bytes);
    Ok(CapturedPayload {
        payload: ContextPayload::Browser {
            had_selection,
            segments,
        },
        budget,
        unsaved: false,
        redactions: normalize_redactions(vec![
            ContextRedaction::Secrets,
            ContextRedaction::BrowserCredentials,
            ContextRedaction::BrowserStorage,
            ContextRedaction::BrowserUnrelatedHistory,
        ])?,
        capabilities: normalize_capabilities(input.capabilities)?,
    })
}

fn capture_terminal(
    input: TerminalContextInput,
    maximum_bytes: usize,
) -> Result<CapturedPayload, ArtifactContextError> {
    validate_identifier("terminal context environment id", &input.environment_id)?;
    validate_identifier("terminal context process state", &input.process_state)?;
    let had_selection = input.selected_output.is_some();
    let metadata = SafeContextText::new(format!(
        "command={}\nworkingDirectory={}\nenvironment={}\nprocessState={}\nexitCode={}",
        input.command.as_str(),
        input.working_directory_display.as_str(),
        input.environment_id,
        input.process_state,
        input
            .exit_code
            .map_or_else(|| "none".into(), |code| code.to_string())
    ))?;
    let candidates = vec![
        candidate(
            ContextPriority::Selection,
            "selected-output",
            input.selected_output,
        ),
        candidate(
            ContextPriority::VisibleOrCurrent,
            "visible-output",
            input.visible_output,
        ),
        candidate(
            ContextPriority::Recent,
            "recent-output-tail",
            input.recent_output_tail,
        ),
        candidate(
            ContextPriority::Metadata,
            "terminal-metadata",
            Some(metadata),
        ),
    ];
    let (segments, budget) = allocate_segments(candidates, maximum_bytes);
    Ok(CapturedPayload {
        payload: ContextPayload::Terminal {
            had_selection,
            exit_code: input.exit_code,
            segments,
        },
        budget,
        unsaved: false,
        redactions: normalize_redactions(vec![
            ContextRedaction::Secrets,
            ContextRedaction::TerminalRawEnvironment,
            ContextRedaction::TerminalPasswords,
            ContextRedaction::TerminalUnrelatedHistory,
        ])?,
        capabilities: normalize_capabilities(input.capabilities)?,
    })
}

struct SegmentCandidate {
    priority: ContextPriority,
    source: &'static str,
    text: Option<SafeContextText>,
}

fn candidate(
    priority: ContextPriority,
    source: &'static str,
    text: Option<SafeContextText>,
) -> SegmentCandidate {
    SegmentCandidate {
        priority,
        source,
        text,
    }
}

fn allocate_segments(
    candidates: Vec<SegmentCandidate>,
    maximum_bytes: usize,
) -> (Vec<ContextSegment>, ContextBudget) {
    let mut remaining = maximum_bytes;
    let mut segments = Vec::new();
    for candidate in candidates {
        let Some(text) = candidate.text else {
            continue;
        };
        let original = text.as_str();
        let supplied = truncate_utf8(original, remaining);
        remaining = remaining.saturating_sub(supplied.len());
        segments.push(ContextSegment {
            priority: candidate.priority,
            source: candidate.source.into(),
            text: supplied.to_owned(),
            original_bytes: original.len() as u64,
            omitted_bytes: (original.len() - supplied.len()) as u64,
        });
    }
    let used_bytes = segments
        .iter()
        .map(|segment| segment.text.len() as u64)
        .sum::<u64>();
    let omitted_bytes = segments
        .iter()
        .map(|segment| segment.omitted_bytes)
        .sum::<u64>();
    let omitted_segments = segments
        .iter()
        .filter(|segment| segment.text.is_empty() && segment.original_bytes > 0)
        .count() as u32;
    (
        segments,
        ContextBudget {
            maximum_bytes: maximum_bytes as u64,
            used_bytes,
            omitted_bytes,
            omitted_segments,
            truncated: omitted_bytes > 0,
        },
    )
}

fn truncate_utf8(value: &str, maximum_bytes: usize) -> &str {
    if value.len() <= maximum_bytes {
        return value;
    }
    let mut boundary = 0;
    for (index, character) in value.char_indices() {
        let next = index + character.len_utf8();
        if next > maximum_bytes {
            break;
        }
        boundary = next;
    }
    &value[..boundary]
}

fn normalize_redactions(
    mut redactions: Vec<ContextRedaction>,
) -> Result<Vec<ContextRedaction>, ArtifactContextError> {
    if redactions.len() > MAX_CONTEXT_REDACTIONS {
        return Err(ArtifactContextError::InvalidRedactions);
    }
    redactions.sort();
    redactions.dedup();
    Ok(redactions)
}

fn validate_redactions(redactions: &[ContextRedaction]) -> Result<(), ArtifactContextError> {
    if redactions.len() > MAX_CONTEXT_REDACTIONS
        || redactions.windows(2).any(|pair| pair[0] >= pair[1])
    {
        Err(ArtifactContextError::InvalidRedactions)
    } else {
        Ok(())
    }
}

fn normalize_capabilities(
    mut capabilities: Vec<CapabilitySummaryEntry>,
) -> Result<Vec<CapabilitySummaryEntry>, ArtifactContextError> {
    if capabilities.len() > MAX_CONTEXT_CAPABILITIES {
        return Err(ArtifactContextError::CapabilityBoundExceeded);
    }
    for capability in &capabilities {
        capability.validate()?;
    }
    capabilities.sort_by(|left, right| left.capability_id.cmp(&right.capability_id));
    if capabilities
        .windows(2)
        .any(|pair| pair[0].capability_id == pair[1].capability_id)
    {
        return Err(ArtifactContextError::DuplicateCapability);
    }
    Ok(capabilities)
}

fn validate_capabilities(
    capabilities: &[CapabilitySummaryEntry],
) -> Result<(), ArtifactContextError> {
    if capabilities.len() > MAX_CONTEXT_CAPABILITIES {
        return Err(ArtifactContextError::CapabilityBoundExceeded);
    }
    let mut ids = BTreeSet::new();
    for capability in capabilities {
        capability.validate()?;
        if !ids.insert(capability.capability_id.as_str()) {
            return Err(ArtifactContextError::DuplicateCapability);
        }
    }
    if capabilities
        .windows(2)
        .any(|pair| pair[0].capability_id >= pair[1].capability_id)
    {
        return Err(ArtifactContextError::DuplicateCapability);
    }
    Ok(())
}

fn validate_segments(segments: &[ContextSegment]) -> Result<(), ArtifactContextError> {
    if segments.len() > MAX_CONTEXT_SEGMENTS {
        return Err(ArtifactContextError::SegmentBoundExceeded);
    }
    let mut sources = BTreeSet::new();
    let mut previous_priority = None;
    for segment in segments {
        segment.validate()?;
        if !sources.insert(segment.source.as_str())
            || previous_priority.is_some_and(|priority| priority > segment.priority)
        {
            return Err(ArtifactContextError::InvalidSegment);
        }
        previous_priority = Some(segment.priority);
    }
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ArtifactContextError {
    #[error("unsupported Artifact Context Snapshot schema")]
    UnsupportedSchema,
    #[error(transparent)]
    Artifact(#[from] ArtifactRecordError),
    #[error(transparent)]
    File(#[from] FileStateError),
    #[error(transparent)]
    Folder(#[from] FolderStateError),
    #[error("unsafe or unbounded context text")]
    UnsafeContextText,
    #[error("invalid Artifact Context Snapshot identity")]
    InvalidIdentity,
    #[error("Artifact Context Snapshot identity does not match its capture input")]
    CaptureBindingMismatch,
    #[error("invalid Artifact Context Snapshot")]
    InvalidSnapshot,
    #[error("invalid context segment")]
    InvalidSegment,
    #[error("context segments exceed their bound")]
    SegmentBoundExceeded,
    #[error("invalid context budget")]
    InvalidBudget,
    #[error("invalid context redaction set")]
    InvalidRedactions,
    #[error("context capabilities exceed their bound")]
    CapabilityBoundExceeded,
    #[error("context capability identity is repeated or unordered")]
    DuplicateCapability,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::file::{FileLiveVersion, FileResourceReference, sha256_text};
    use crate::artifact::folder::{
        FolderBreadcrumb, FolderEntry, FolderListingVersion, FolderResourceReference,
    };

    fn identity(provider_type: &str, version: ArtifactResourceVersion) -> ContextCaptureIdentity {
        ContextCaptureIdentity {
            snapshot_id: "context-1".into(),
            artifact_id: "artifact-1".into(),
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            session_id: "session-1".into(),
            provider_type: provider_type.into(),
            provider_schema_version: 1,
            artifact_record_revision: 4,
            live_resource_version: version,
        }
    }

    fn file_state() -> FileArtifactState {
        FileArtifactState::new(
            FileResourceReference::new("docs/plan.md", "docs/plan.md").unwrap(),
            "plan.md",
            "text/markdown",
            "current document",
            FileLiveVersion::new(
                2,
                sha256_text("current document"),
                "current document".len() as u64,
                10,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn capability(id: &str) -> CapabilitySummaryEntry {
        CapabilitySummaryEntry {
            capability_id: id.into(),
            access: CapabilityAccess::Readable,
            reason_code: None,
        }
    }

    #[test]
    fn file_capture_prioritizes_selection_and_marks_unsaved_truncation() {
        let mut file = file_state();
        file.begin_draft(20).unwrap();
        file.update_draft("unsaved document", 21).unwrap();
        let resource = file.live_version.as_resource_version();
        let snapshot = capture_artifact_context(
            identity("file", resource),
            ContextCaptureInput::File(FileContextInput {
                state: &file,
                selected_text: Some(SafeContextText::new("selected").unwrap()),
                visible_text: Some(SafeContextText::new("visible").unwrap()),
                recent_text: Some(SafeContextText::new("recent").unwrap()),
                redactions: vec![ContextRedaction::Secrets],
                capabilities: vec![capability("artifact-read")],
            }),
            12,
            30,
        )
        .unwrap();

        let ContextPayload::File {
            segments,
            captured_unsaved_draft,
            ..
        } = &snapshot.payload
        else {
            panic!("expected File context");
        };
        assert_eq!(segments[0].source, "selected-text");
        assert_eq!(segments[0].text, "selected");
        assert_eq!(segments[1].text, "visi");
        assert!(captured_unsaved_draft);
        assert!(snapshot.unsaved);
        assert!(snapshot.budget.truncated);
        assert_eq!(
            snapshot.stable_reference,
            format!(
                "artifact:artifact-1:record:4:resource:2:{}",
                sha256_text("current document")
            )
        );
        snapshot.validate().unwrap();
    }

    #[test]
    fn captured_file_context_is_immutable_after_the_draft_changes() {
        let mut file = file_state();
        let snapshot = capture_artifact_context(
            identity("file", file.live_version.as_resource_version()),
            ContextCaptureInput::File(FileContextInput {
                state: &file,
                selected_text: None,
                visible_text: None,
                recent_text: None,
                redactions: Vec::new(),
                capabilities: Vec::new(),
            }),
            1_024,
            20,
        )
        .unwrap();
        file.begin_draft(21).unwrap();
        file.update_draft("later edit", 22).unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();
        assert!(serialized.contains("current document"));
        assert!(!serialized.contains("later edit"));
    }

    #[test]
    fn capture_identity_must_match_the_file_live_version() {
        let file = file_state();
        let mut wrong = file.live_version.as_resource_version();
        wrong.sequence += 1;
        assert_eq!(
            capture_artifact_context(
                identity("file", wrong),
                ContextCaptureInput::File(FileContextInput {
                    state: &file,
                    selected_text: None,
                    visible_text: None,
                    recent_text: None,
                    redactions: Vec::new(),
                    capabilities: Vec::new(),
                }),
                1_024,
                20,
            ),
            Err(ArtifactContextError::CaptureBindingMismatch)
        );
    }

    #[test]
    fn folder_capture_is_bounded_and_non_recursive() {
        let entries = vec![FolderEntry {
            entry_id: "entry-plan".into(),
            name: "plan.md".into(),
            kind: FolderEntryKind::File,
            project_relative_path: "docs/plan.md".into(),
            display_metadata: None,
            byte_length: Some(4),
            modified_at_ms: Some(10),
            content_sha256: Some(sha256_text("plan")),
        }];
        let folder = FolderArtifactState::new(
            FolderResourceReference::new("docs", "docs").unwrap(),
            vec![
                FolderBreadcrumb {
                    label: "Project".into(),
                    project_relative_path: String::new(),
                },
                FolderBreadcrumb {
                    label: "docs".into(),
                    project_relative_path: "docs".into(),
                },
            ],
            entries.clone(),
            FolderListingVersion::from_entries(3, &entries, 10).unwrap(),
        )
        .unwrap();
        let snapshot = capture_artifact_context(
            identity("folder", folder.listing_version.as_resource_version()),
            ContextCaptureInput::Folder(FolderContextInput {
                state: &folder,
                recent_text: None,
                redactions: Vec::new(),
                capabilities: Vec::new(),
            }),
            1_024,
            20,
        )
        .unwrap();
        let ContextPayload::Folder {
            listed_entry_count,
            segments,
            ..
        } = snapshot.payload
        else {
            panic!("expected Folder context");
        };
        assert_eq!(listed_entry_count, 1);
        assert!(
            segments
                .iter()
                .any(|segment| segment.text.contains("docs/plan.md"))
        );
    }

    #[test]
    fn browser_input_has_no_sensitive_storage_or_credential_fields() {
        let snapshot = capture_artifact_context(
            identity(
                "browser",
                ArtifactResourceVersion {
                    sequence: 1,
                    sha256: sha256_text("browser"),
                    observed_at_ms: 10,
                },
            ),
            ContextCaptureInput::Browser(BrowserContextInput {
                current_url: SafeContextText::new("https://example.test").unwrap(),
                title: SafeContextText::new("Example").unwrap(),
                navigation_state: SafeContextText::new("loaded").unwrap(),
                selected_text: None,
                visible_text: Some(SafeContextText::new("visible page").unwrap()),
                extracted_content: None,
                recent_activity: None,
                capabilities: Vec::new(),
            }),
            1_024,
            20,
        )
        .unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();
        assert!(!serialized.contains("cookie"));
        assert!(!serialized.contains("localStorage"));
        assert!(
            snapshot
                .redactions
                .contains(&ContextRedaction::BrowserStorage)
        );
        assert!(
            snapshot
                .redactions
                .contains(&ContextRedaction::BrowserCredentials)
        );
    }

    #[test]
    fn terminal_input_excludes_raw_environment_passwords_and_history() {
        let snapshot = capture_artifact_context(
            identity(
                "terminal",
                ArtifactResourceVersion {
                    sequence: 1,
                    sha256: sha256_text("terminal"),
                    observed_at_ms: 10,
                },
            ),
            ContextCaptureInput::Terminal(TerminalContextInput {
                command: SafeContextText::new("npm test").unwrap(),
                working_directory_display: SafeContextText::new("Project/docs").unwrap(),
                environment_id: "local".into(),
                process_state: "completed".into(),
                exit_code: Some(0),
                selected_output: None,
                visible_output: Some(SafeContextText::new("32 tests passed").unwrap()),
                recent_output_tail: None,
                capabilities: Vec::new(),
            }),
            1_024,
            20,
        )
        .unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();
        assert!(!serialized.contains("rawEnvironment"));
        assert!(!serialized.contains("password"));
        assert!(
            snapshot
                .redactions
                .contains(&ContextRedaction::TerminalRawEnvironment)
        );
        assert!(
            snapshot
                .redactions
                .contains(&ContextRedaction::TerminalUnrelatedHistory)
        );
    }

    #[test]
    fn budget_truncation_never_splits_utf8() {
        let text = SafeContextText::new("ééé").unwrap();
        let (segments, budget) = allocate_segments(
            vec![candidate(
                ContextPriority::Selection,
                "selected-text",
                Some(text),
            )],
            3,
        );
        assert_eq!(segments[0].text, "é");
        assert_eq!(segments[0].omitted_bytes, 4);
        assert_eq!(budget.used_bytes, 2);
        assert!(budget.truncated);
    }
}
