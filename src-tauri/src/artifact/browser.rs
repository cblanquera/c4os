use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use thiserror::Error;
use url::Url;

use super::record::ArtifactResourceVersion;

pub const MAX_BROWSER_NAVIGATION_URL_BYTES: usize = 8 * 1_024;
pub const MAX_BROWSER_DISPLAY_URL_BYTES: usize = 4 * 1_024;
pub const MAX_BROWSER_TITLE_BYTES: usize = 512;
pub const MAX_BROWSER_HISTORY_ENTRIES: usize = 256;
pub const MAX_BROWSER_REFERENCE_ID_BYTES: usize = 160;
pub const MAX_BROWSER_REPLY_ATTEMPTS: usize = 64;
pub const BROWSER_REPLY_NAVIGATION_BEGIN: &str = "<c4os-browser-navigation>";
pub const BROWSER_REPLY_NAVIGATION_END: &str = "</c4os-browser-navigation>";

/// One normalized navigation target held only for the immediate native action.
///
/// This type deliberately does not implement `Serialize` or `Deserialize`.
/// Query strings and fragments may be required for navigation, but must not be
/// copied into the durable Browser artifact state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserNavigationTarget {
    navigation_url: Url,
    display_url: String,
}

impl BrowserNavigationTarget {
    pub fn parse(address: &str) -> Result<Self, BrowserStateError> {
        normalize_browser_address(address)
    }

    pub fn navigation_url(&self) -> &Url {
        &self.navigation_url
    }

    pub fn navigation_url_str(&self) -> &str {
        self.navigation_url.as_str()
    }

    pub fn display_url(&self) -> &str {
        &self.display_url
    }

    pub fn navigation_sha256(&self) -> String {
        sha256_bytes(self.navigation_url.as_str().as_bytes())
    }

    pub fn into_navigation_url(self) -> Url {
        self.navigation_url
    }
}

/// Extracts one explicit machine-readable Browser Reply navigation request.
/// Ordinary assistant prose and ambiguous/malformed envelopes never navigate.
pub fn browser_reply_navigation_target(markdown: &str) -> Option<BrowserNavigationTarget> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct NavigationEnvelope {
        address: String,
    }

    let start = markdown.find(BROWSER_REPLY_NAVIGATION_BEGIN)?;
    if markdown[start + BROWSER_REPLY_NAVIGATION_BEGIN.len()..]
        .contains(BROWSER_REPLY_NAVIGATION_BEGIN)
    {
        return None;
    }
    let payload_start = start + BROWSER_REPLY_NAVIGATION_BEGIN.len();
    let relative_end = markdown[payload_start..].find(BROWSER_REPLY_NAVIGATION_END)?;
    let payload_end = payload_start + relative_end;
    if markdown[payload_end + BROWSER_REPLY_NAVIGATION_END.len()..]
        .contains(BROWSER_REPLY_NAVIGATION_END)
    {
        return None;
    }
    let envelope =
        serde_json::from_str::<NavigationEnvelope>(markdown[payload_start..payload_end].trim())
            .ok()?;
    BrowserNavigationTarget::parse(&envelope.address).ok()
}

pub fn normalize_browser_address(
    address: &str,
) -> Result<BrowserNavigationTarget, BrowserStateError> {
    if address.is_empty()
        || address.len() > MAX_BROWSER_NAVIGATION_URL_BYTES
        || address.trim() != address
        || address.chars().any(char::is_control)
    {
        return Err(BrowserStateError::InvalidAddress);
    }

    let parsed = match Url::parse(address) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => parsed,
        Ok(_) if looks_like_host_and_port(address) => parse_with_https(address)?,
        Ok(_) => return Err(BrowserStateError::UnsupportedScheme),
        Err(_) if address.starts_with("//") => Url::parse(&format!("https:{address}"))
            .map_err(|_| BrowserStateError::InvalidAddress)?,
        Err(_) if has_explicit_non_port_scheme(address) => {
            return Err(BrowserStateError::UnsupportedScheme);
        }
        Err(_) => parse_with_https(address)?,
    };

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(BrowserStateError::UnsupportedScheme);
    }
    if parsed.host_str().is_none() || parsed.as_str().len() > MAX_BROWSER_NAVIGATION_URL_BYTES {
        return Err(BrowserStateError::InvalidAddress);
    }
    if raw_authority_contains_userinfo(address)
        || authority_contains_userinfo(&parsed)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(BrowserStateError::AddressContainsCredentials);
    }

    let mut display = parsed.clone();
    display
        .set_username("")
        .map_err(|_| BrowserStateError::InvalidAddress)?;
    display
        .set_password(None)
        .map_err(|_| BrowserStateError::InvalidAddress)?;
    display.set_query(None);
    display.set_fragment(None);
    let display_url = display.to_string();
    validate_display_url(&display_url)?;

    Ok(BrowserNavigationTarget {
        navigation_url: parsed,
        display_url,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserEnvironmentScope {
    AppWide,
    WorkspaceProject,
    Chat,
    None,
}

/// Durable scope reference for resolving a profile in the Rust-owned registry.
///
/// It intentionally contains no WebKit data-store UUID or website data.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BrowserEnvironmentReference {
    pub scope: BrowserEnvironmentScope,
    pub workspace_id: Option<String>,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub artifact_id: Option<String>,
    pub generation: u64,
}

impl BrowserEnvironmentReference {
    pub fn app_wide(generation: u64) -> Result<Self, BrowserStateError> {
        Self::new(
            BrowserEnvironmentScope::AppWide,
            None,
            None,
            None,
            None,
            generation,
        )
    }

    pub fn workspace_project(
        workspace_id: impl Into<String>,
        project_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, BrowserStateError> {
        Self::new(
            BrowserEnvironmentScope::WorkspaceProject,
            Some(workspace_id.into()),
            Some(project_id.into()),
            None,
            None,
            generation,
        )
    }

    pub fn chat(
        workspace_id: impl Into<String>,
        project_id: impl Into<String>,
        session_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, BrowserStateError> {
        Self::new(
            BrowserEnvironmentScope::Chat,
            Some(workspace_id.into()),
            Some(project_id.into()),
            Some(session_id.into()),
            None,
            generation,
        )
    }

    pub fn ephemeral(
        artifact_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, BrowserStateError> {
        Self::new(
            BrowserEnvironmentScope::None,
            None,
            None,
            None,
            Some(artifact_id.into()),
            generation,
        )
    }

    fn new(
        scope: BrowserEnvironmentScope,
        workspace_id: Option<String>,
        project_id: Option<String>,
        session_id: Option<String>,
        artifact_id: Option<String>,
        generation: u64,
    ) -> Result<Self, BrowserStateError> {
        let reference = Self {
            scope,
            workspace_id,
            project_id,
            session_id,
            artifact_id,
            generation,
        };
        reference.validate()?;
        Ok(reference)
    }

    pub fn validate(&self) -> Result<(), BrowserStateError> {
        if self.generation == 0 {
            return Err(BrowserStateError::InvalidEnvironmentReference);
        }
        let valid_shape = match self.scope {
            BrowserEnvironmentScope::AppWide => {
                self.workspace_id.is_none()
                    && self.project_id.is_none()
                    && self.session_id.is_none()
                    && self.artifact_id.is_none()
            }
            BrowserEnvironmentScope::WorkspaceProject => {
                self.workspace_id.is_some()
                    && self.project_id.is_some()
                    && self.session_id.is_none()
                    && self.artifact_id.is_none()
            }
            BrowserEnvironmentScope::Chat => {
                self.workspace_id.is_some()
                    && self.project_id.is_some()
                    && self.session_id.is_some()
                    && self.artifact_id.is_none()
            }
            BrowserEnvironmentScope::None => {
                self.workspace_id.is_none()
                    && self.project_id.is_none()
                    && self.session_id.is_none()
                    && self.artifact_id.is_some()
            }
        };
        if !valid_shape {
            return Err(BrowserStateError::InvalidEnvironmentReference);
        }
        for identifier in [
            self.workspace_id.as_deref(),
            self.project_id.as_deref(),
            self.session_id.as_deref(),
            self.artifact_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            validate_reference_identifier(identifier)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserErrorCode {
    PolicyDenied,
    InvalidAddress,
    NavigationFailed,
    PermissionDenied,
    PopupDenied,
    DownloadDenied,
    ControllerUnavailable,
    ProfileUnavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserRecoveryCode {
    WebContentProcessTerminated,
    ControllerRecreated,
    ProfileCleared,
    ApplicationRelaunch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "phase")]
pub enum BrowserPhase {
    Queued {
        queued_at_ms: u64,
    },
    Loading {
        started_at_ms: u64,
    },
    Ready {
        ready_at_ms: u64,
    },
    Error {
        code: BrowserErrorCode,
        retryable: bool,
        failed_at_ms: u64,
    },
    Recovery {
        code: BrowserRecoveryCode,
        recovered_at_ms: u64,
    },
}

impl BrowserPhase {
    pub fn phase(&self) -> &'static str {
        match self {
            Self::Queued { .. } => "queued",
            Self::Loading { .. } => "loading",
            Self::Ready { .. } => "ready",
            Self::Error { .. } => "error",
            Self::Recovery { .. } => "recovery",
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    fn timestamp(&self) -> u64 {
        match self {
            Self::Queued { queued_at_ms } => *queued_at_ms,
            Self::Loading { started_at_ms } => *started_at_ms,
            Self::Ready { ready_at_ms } => *ready_at_ms,
            Self::Error { failed_at_ms, .. } => *failed_at_ms,
            Self::Recovery {
                recovered_at_ms, ..
            } => *recovered_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BrowserHistoryEntry {
    pub display_url: String,
    /// `sha256:` digest of the exact normalized navigation URL, including any
    /// query or fragment that must stay out of durable and renderer-visible
    /// state.
    pub navigation_sha256: String,
    pub title: Option<String>,
}

impl BrowserHistoryEntry {
    fn from_target(target: &BrowserNavigationTarget) -> Self {
        Self {
            display_url: target.display_url.clone(),
            navigation_sha256: target.navigation_sha256(),
            title: None,
        }
    }

    fn from_bound_target(target: &BrowserNavigationTarget, navigation_sha256: &str) -> Self {
        Self {
            display_url: target.display_url.clone(),
            navigation_sha256: navigation_sha256.to_owned(),
            title: None,
        }
    }

    fn validate(&self) -> Result<(), BrowserStateError> {
        validate_display_url(&self.display_url)?;
        validate_navigation_sha256(&self.navigation_sha256)?;
        if let Some(title) = &self.title {
            validate_title(title)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserNavigationIntent {
    Back,
    Forward,
    Refresh,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserNavigationKind {
    Initial,
    New,
    Back,
    Forward,
    Refresh,
    Recovery,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BrowserControllerEventMeta {
    pub controller_generation: u64,
    pub event_sequence: u64,
    pub observed_at_ms: u64,
}

impl BrowserControllerEventMeta {
    pub fn new(
        controller_generation: u64,
        event_sequence: u64,
        observed_at_ms: u64,
    ) -> Result<Self, BrowserStateError> {
        let metadata = Self {
            controller_generation,
            event_sequence,
            observed_at_ms,
        };
        metadata.validate()?;
        Ok(metadata)
    }

    fn validate(&self) -> Result<(), BrowserStateError> {
        if self.controller_generation == 0 || self.event_sequence == 0 || self.observed_at_ms == 0 {
            return Err(BrowserStateError::InvalidControllerEvent);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BrowserArtifactState {
    pub environment: BrowserEnvironmentReference,
    pub phase: BrowserPhase,
    pub controller_generation: u64,
    pub controller_event_sequence: u64,
    pub history: Vec<BrowserHistoryEntry>,
    pub current_history_index: u32,
    pub processed_reply_attempts: Vec<String>,
    pub version: ArtifactResourceVersion,
}

impl BrowserArtifactState {
    pub fn new_queued(
        environment: BrowserEnvironmentReference,
        target: &BrowserNavigationTarget,
        controller_generation: u64,
        queued_at_ms: u64,
    ) -> Result<Self, BrowserStateError> {
        environment.validate()?;
        if controller_generation == 0 {
            return Err(BrowserStateError::InvalidControllerGeneration);
        }
        if queued_at_ms == 0 {
            return Err(BrowserStateError::InvalidTimestamp);
        }
        let mut state = Self {
            environment,
            phase: BrowserPhase::Queued { queued_at_ms },
            controller_generation,
            controller_event_sequence: 0,
            history: vec![BrowserHistoryEntry::from_target(target)],
            current_history_index: 0,
            processed_reply_attempts: Vec::new(),
            version: ArtifactResourceVersion {
                sequence: 1,
                sha256: sha256_bytes(&[]),
                observed_at_ms: queued_at_ms,
            },
        };
        state.refresh_version(queued_at_ms, false)?;
        state.validate()?;
        Ok(state)
    }

    pub fn current_entry(&self) -> &BrowserHistoryEntry {
        &self.history[self.current_history_index as usize]
    }

    pub fn current_display_url(&self) -> &str {
        &self.current_entry().display_url
    }

    pub fn current_title(&self) -> Option<&str> {
        self.current_entry().title.as_deref()
    }

    pub fn current_navigation_sha256(&self) -> &str {
        &self.current_entry().navigation_sha256
    }

    pub fn can_go_back(&self) -> bool {
        self.current_history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        (self.current_history_index as usize) + 1 < self.history.len()
    }

    pub fn validate_navigation_intent(
        &self,
        intent: BrowserNavigationIntent,
    ) -> Result<(), BrowserStateError> {
        if !matches!(
            self.phase,
            BrowserPhase::Ready { .. } | BrowserPhase::Error { .. } | BrowserPhase::Recovery { .. }
        ) {
            return Err(BrowserStateError::InvalidTransition);
        }
        if matches!(self.phase, BrowserPhase::Recovery { .. })
            && !matches!(intent, BrowserNavigationIntent::Refresh)
        {
            return Err(BrowserStateError::InvalidTransition);
        }
        match intent {
            BrowserNavigationIntent::Back if !self.can_go_back() => {
                Err(BrowserStateError::HistoryBoundary)
            }
            BrowserNavigationIntent::Forward if !self.can_go_forward() => {
                Err(BrowserStateError::HistoryBoundary)
            }
            BrowserNavigationIntent::Back
            | BrowserNavigationIntent::Forward
            | BrowserNavigationIntent::Refresh => Ok(()),
        }
    }

    pub fn start_navigation(
        &mut self,
        metadata: BrowserControllerEventMeta,
        target: &BrowserNavigationTarget,
        kind: BrowserNavigationKind,
    ) -> Result<(), BrowserStateError> {
        self.start_navigation_bound(metadata, target, kind, &target.navigation_sha256())
    }

    pub fn start_navigation_bound(
        &mut self,
        metadata: BrowserControllerEventMeta,
        target: &BrowserNavigationTarget,
        kind: BrowserNavigationKind,
        navigation_sha256: &str,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        self.validate_navigation_start(target, kind)?;
        validate_navigation_sha256(navigation_sha256)?;
        self.phase = BrowserPhase::Loading {
            started_at_ms: metadata.observed_at_ms,
        };
        self.commit_controller_event(metadata)
    }

    pub fn mark_ready(
        &mut self,
        metadata: BrowserControllerEventMeta,
        target: &BrowserNavigationTarget,
        kind: BrowserNavigationKind,
        title: Option<&str>,
    ) -> Result<(), BrowserStateError> {
        self.mark_ready_bound(metadata, target, kind, &target.navigation_sha256(), title)
    }

    pub fn mark_ready_bound(
        &mut self,
        metadata: BrowserControllerEventMeta,
        target: &BrowserNavigationTarget,
        kind: BrowserNavigationKind,
        navigation_sha256: &str,
        title: Option<&str>,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        if !matches!(self.phase, BrowserPhase::Loading { .. }) {
            return Err(BrowserStateError::InvalidTransition);
        }
        let title = normalize_title(title)?;
        validate_navigation_sha256(navigation_sha256)?;
        match kind {
            BrowserNavigationKind::Initial => {}
            BrowserNavigationKind::New => {
                let retained = self.current_history_index as usize + 1;
                self.history.truncate(retained);
                if self.history.len() == MAX_BROWSER_HISTORY_ENTRIES {
                    self.history.remove(0);
                }
                self.history.push(BrowserHistoryEntry::from_bound_target(
                    target,
                    navigation_sha256,
                ));
                self.current_history_index = (self.history.len() - 1) as u32;
            }
            BrowserNavigationKind::Back => {
                if !self.can_go_back() {
                    return Err(BrowserStateError::HistoryBoundary);
                }
                self.current_history_index -= 1;
            }
            BrowserNavigationKind::Forward => {
                if !self.can_go_forward() {
                    return Err(BrowserStateError::HistoryBoundary);
                }
                self.current_history_index += 1;
            }
            BrowserNavigationKind::Refresh | BrowserNavigationKind::Recovery => {}
        }
        let current = &mut self.history[self.current_history_index as usize];
        current.display_url = target.display_url.clone();
        current.navigation_sha256 = navigation_sha256.to_owned();
        current.title = title;
        self.phase = BrowserPhase::Ready {
            ready_at_ms: metadata.observed_at_ms,
        };
        self.commit_controller_event(metadata)
    }

    pub fn cancel_navigation(
        &mut self,
        metadata: BrowserControllerEventMeta,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        if !matches!(self.phase, BrowserPhase::Loading { .. }) {
            return Err(BrowserStateError::InvalidTransition);
        }
        self.phase = BrowserPhase::Ready {
            ready_at_ms: metadata.observed_at_ms,
        };
        self.commit_controller_event(metadata)
    }

    pub fn update_title(
        &mut self,
        metadata: BrowserControllerEventMeta,
        title: Option<&str>,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        if !matches!(
            self.phase,
            BrowserPhase::Loading { .. } | BrowserPhase::Ready { .. }
        ) {
            return Err(BrowserStateError::InvalidTransition);
        }
        self.history[self.current_history_index as usize].title = normalize_title(title)?;
        self.commit_controller_event(metadata)
    }

    pub fn fail_from_controller(
        &mut self,
        metadata: BrowserControllerEventMeta,
        code: BrowserErrorCode,
        retryable: bool,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        if !matches!(
            self.phase,
            BrowserPhase::Loading { .. } | BrowserPhase::Ready { .. }
        ) {
            return Err(BrowserStateError::InvalidTransition);
        }
        self.phase = BrowserPhase::Error {
            code,
            retryable,
            failed_at_ms: metadata.observed_at_ms,
        };
        self.commit_controller_event(metadata)
    }

    pub fn fail_before_controller(
        &mut self,
        code: BrowserErrorCode,
        retryable: bool,
        failed_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        self.require_core_transition_timestamp(failed_at_ms)?;
        if !matches!(self.phase, BrowserPhase::Queued { .. }) {
            return Err(BrowserStateError::InvalidTransition);
        }
        self.phase = BrowserPhase::Error {
            code,
            retryable,
            failed_at_ms,
        };
        self.refresh_version(failed_at_ms, true)
    }

    pub fn recover_from_controller(
        &mut self,
        metadata: BrowserControllerEventMeta,
        code: BrowserRecoveryCode,
    ) -> Result<(), BrowserStateError> {
        self.validate_controller_event(&metadata)?;
        if matches!(self.phase, BrowserPhase::Queued { .. }) {
            return Err(BrowserStateError::InvalidTransition);
        }
        self.phase = BrowserPhase::Recovery {
            code,
            recovered_at_ms: metadata.observed_at_ms,
        };
        self.commit_controller_event(metadata)
    }

    pub fn begin_recovery(
        &mut self,
        code: BrowserRecoveryCode,
        recovered_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        self.require_core_transition_timestamp(recovered_at_ms)?;
        self.phase = BrowserPhase::Recovery {
            code,
            recovered_at_ms,
        };
        self.refresh_version(recovered_at_ms, true)
    }

    pub fn install_controller_generation(
        &mut self,
        controller_generation: u64,
        observed_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        self.require_core_transition_timestamp(observed_at_ms)?;
        if controller_generation <= self.controller_generation || self.phase.is_loading() {
            return Err(BrowserStateError::InvalidControllerGeneration);
        }
        self.controller_generation = controller_generation;
        self.controller_event_sequence = 0;
        self.refresh_version(observed_at_ms, true)
    }

    pub fn rebind_cleared_environment(
        &mut self,
        data_generation: u64,
        observed_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        self.require_core_transition_timestamp(observed_at_ms)?;
        if matches!(self.environment.scope, BrowserEnvironmentScope::None)
            || data_generation <= self.environment.generation
        {
            return Err(BrowserStateError::InvalidEnvironmentReference);
        }
        self.environment.generation = data_generation;
        self.phase = BrowserPhase::Recovery {
            code: BrowserRecoveryCode::ProfileCleared,
            recovered_at_ms: observed_at_ms,
        };
        self.controller_generation = self
            .controller_generation
            .checked_add(1)
            .ok_or(BrowserStateError::SequenceOverflow)?;
        self.controller_event_sequence = 0;
        self.refresh_version(observed_at_ms, true)
    }

    pub fn clear_ephemeral_environment(
        &mut self,
        observed_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        self.require_core_transition_timestamp(observed_at_ms)?;
        if !matches!(self.environment.scope, BrowserEnvironmentScope::None) {
            return Err(BrowserStateError::InvalidEnvironmentReference);
        }
        self.environment.generation = self
            .environment
            .generation
            .checked_add(1)
            .ok_or(BrowserStateError::InvalidEnvironmentReference)?;
        self.phase = BrowserPhase::Recovery {
            code: BrowserRecoveryCode::ProfileCleared,
            recovered_at_ms: observed_at_ms,
        };
        self.controller_generation = self
            .controller_generation
            .checked_add(1)
            .ok_or(BrowserStateError::SequenceOverflow)?;
        self.controller_event_sequence = 0;
        self.refresh_version(observed_at_ms, true)
    }

    pub fn has_processed_reply_attempt(&self, attempt_id: &str) -> bool {
        self.processed_reply_attempts
            .iter()
            .any(|processed| processed == attempt_id)
    }

    pub fn record_reply_attempt(
        &mut self,
        attempt_id: &str,
        observed_at_ms: u64,
    ) -> Result<bool, BrowserStateError> {
        validate_reference_identifier(attempt_id)?;
        if self.has_processed_reply_attempt(attempt_id) {
            return Ok(false);
        }
        self.require_core_transition_timestamp(observed_at_ms)?;
        if self.processed_reply_attempts.len() == MAX_BROWSER_REPLY_ATTEMPTS {
            self.processed_reply_attempts.remove(0);
        }
        self.processed_reply_attempts.push(attempt_id.to_owned());
        self.refresh_version(observed_at_ms, true)?;
        Ok(true)
    }

    pub fn validate(&self) -> Result<(), BrowserStateError> {
        self.environment.validate()?;
        if self.controller_generation == 0 {
            return Err(BrowserStateError::InvalidControllerGeneration);
        }
        if self.history.is_empty() || self.history.len() > MAX_BROWSER_HISTORY_ENTRIES {
            return Err(BrowserStateError::InvalidHistory);
        }
        if self.current_history_index as usize >= self.history.len() {
            return Err(BrowserStateError::InvalidHistory);
        }
        for entry in &self.history {
            entry.validate()?;
        }
        if self.processed_reply_attempts.len() > MAX_BROWSER_REPLY_ATTEMPTS {
            return Err(BrowserStateError::InvalidReplyAttempt);
        }
        let mut reply_attempts = BTreeSet::new();
        for attempt_id in &self.processed_reply_attempts {
            validate_reference_identifier(attempt_id)
                .map_err(|_| BrowserStateError::InvalidReplyAttempt)?;
            if !reply_attempts.insert(attempt_id) {
                return Err(BrowserStateError::InvalidReplyAttempt);
            }
        }
        if self.phase.timestamp() == 0 || self.phase.timestamp() > self.version.observed_at_ms {
            return Err(BrowserStateError::InvalidTimestamp);
        }
        self.version
            .validate()
            .map_err(|_| BrowserStateError::InvalidVersion)?;
        if self.state_digest()? != self.version.sha256 {
            return Err(BrowserStateError::InvalidVersion);
        }
        Ok(())
    }

    pub fn as_resource_version(&self) -> ArtifactResourceVersion {
        self.version.clone()
    }

    fn validate_navigation_start(
        &self,
        target: &BrowserNavigationTarget,
        kind: BrowserNavigationKind,
    ) -> Result<(), BrowserStateError> {
        if self.phase.is_loading() {
            return Err(BrowserStateError::InvalidTransition);
        }
        let expected = match kind {
            BrowserNavigationKind::Initial => {
                if !matches!(self.phase, BrowserPhase::Queued { .. }) {
                    return Err(BrowserStateError::InvalidTransition);
                }
                Some(self.current_display_url())
            }
            BrowserNavigationKind::New => {
                if matches!(self.phase, BrowserPhase::Queued { .. }) {
                    return Err(BrowserStateError::InvalidTransition);
                }
                None
            }
            BrowserNavigationKind::Back => {
                self.validate_navigation_intent(BrowserNavigationIntent::Back)?;
                Some(
                    self.history[self.current_history_index as usize - 1]
                        .display_url
                        .as_str(),
                )
            }
            BrowserNavigationKind::Forward => {
                self.validate_navigation_intent(BrowserNavigationIntent::Forward)?;
                Some(
                    self.history[self.current_history_index as usize + 1]
                        .display_url
                        .as_str(),
                )
            }
            BrowserNavigationKind::Refresh => {
                self.validate_navigation_intent(BrowserNavigationIntent::Refresh)?;
                Some(self.current_display_url())
            }
            BrowserNavigationKind::Recovery => {
                if !matches!(self.phase, BrowserPhase::Recovery { .. }) {
                    return Err(BrowserStateError::InvalidTransition);
                }
                Some(self.current_display_url())
            }
        };
        if expected.is_some_and(|expected| expected != target.display_url()) {
            return Err(BrowserStateError::HistoryTargetMismatch);
        }
        Ok(())
    }

    fn validate_controller_event(
        &self,
        metadata: &BrowserControllerEventMeta,
    ) -> Result<(), BrowserStateError> {
        metadata.validate()?;
        if metadata.controller_generation != self.controller_generation {
            return Err(BrowserStateError::StaleControllerGeneration);
        }
        if metadata.event_sequence <= self.controller_event_sequence {
            return Err(BrowserStateError::StaleEventSequence);
        }
        let expected_sequence = self
            .controller_event_sequence
            .checked_add(1)
            .ok_or(BrowserStateError::SequenceOverflow)?;
        if metadata.event_sequence != expected_sequence {
            return Err(BrowserStateError::ControllerEventSequenceGap);
        }
        if metadata.observed_at_ms < self.version.observed_at_ms {
            return Err(BrowserStateError::InvalidTimestamp);
        }
        Ok(())
    }

    fn commit_controller_event(
        &mut self,
        metadata: BrowserControllerEventMeta,
    ) -> Result<(), BrowserStateError> {
        self.controller_event_sequence = metadata.event_sequence;
        self.refresh_version(metadata.observed_at_ms, true)
    }

    fn require_core_transition_timestamp(
        &self,
        observed_at_ms: u64,
    ) -> Result<(), BrowserStateError> {
        if observed_at_ms == 0 || observed_at_ms < self.version.observed_at_ms {
            return Err(BrowserStateError::InvalidTimestamp);
        }
        self.version
            .sequence
            .checked_add(1)
            .ok_or(BrowserStateError::SequenceOverflow)?;
        Ok(())
    }

    fn refresh_version(
        &mut self,
        observed_at_ms: u64,
        increment: bool,
    ) -> Result<(), BrowserStateError> {
        if observed_at_ms == 0 || observed_at_ms < self.version.observed_at_ms {
            return Err(BrowserStateError::InvalidTimestamp);
        }
        if increment {
            self.version.sequence = self
                .version
                .sequence
                .checked_add(1)
                .ok_or(BrowserStateError::SequenceOverflow)?;
        }
        self.version.observed_at_ms = observed_at_ms;
        self.version.sha256 = self.state_digest()?;
        Ok(())
    }

    fn state_digest(&self) -> Result<String, BrowserStateError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DigestInput<'a> {
            environment: &'a BrowserEnvironmentReference,
            phase: &'a BrowserPhase,
            controller_generation: u64,
            controller_event_sequence: u64,
            history: &'a [BrowserHistoryEntry],
            current_history_index: u32,
            processed_reply_attempts: &'a [String],
        }

        let bytes = serde_json::to_vec(&DigestInput {
            environment: &self.environment,
            phase: &self.phase,
            controller_generation: self.controller_generation,
            controller_event_sequence: self.controller_event_sequence,
            history: &self.history,
            current_history_index: self.current_history_index,
            processed_reply_attempts: &self.processed_reply_attempts,
        })
        .map_err(|_| BrowserStateError::Serialization)?;
        Ok(sha256_bytes(&bytes))
    }
}

fn parse_with_https(address: &str) -> Result<Url, BrowserStateError> {
    Url::parse(&format!("https://{address}")).map_err(|_| BrowserStateError::InvalidAddress)
}

fn looks_like_host_and_port(address: &str) -> bool {
    let authority = address.split(['/', '?', '#']).next().unwrap_or_default();
    let Some((host, port)) = authority.rsplit_once(':') else {
        return false;
    };
    !host.is_empty() && !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
}

fn has_explicit_non_port_scheme(address: &str) -> bool {
    let prefix = address.split(['/', '?', '#']).next().unwrap_or_default();
    let Some((scheme, remainder)) = prefix.split_once(':') else {
        return false;
    };
    if !remainder.is_empty() && remainder.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

fn authority_contains_userinfo(url: &Url) -> bool {
    let Some((_, after_scheme)) = url.as_str().split_once("://") else {
        return false;
    };
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|authority| authority.contains('@'))
}

fn raw_authority_contains_userinfo(address: &str) -> bool {
    let authority_and_path = address
        .split_once("://")
        .map(|(_, remainder)| remainder)
        .or_else(|| address.strip_prefix("//"))
        .unwrap_or(address);
    authority_and_path
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|authority| authority.contains('@'))
}

fn validate_display_url(value: &str) -> Result<(), BrowserStateError> {
    if value.is_empty()
        || value.len() > MAX_BROWSER_DISPLAY_URL_BYTES
        || value.chars().any(char::is_control)
        || value.contains('?')
        || value.contains('#')
    {
        return Err(BrowserStateError::InvalidDisplayUrl);
    }
    let parsed = Url::parse(value).map_err(|_| BrowserStateError::InvalidDisplayUrl)?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || authority_contains_userinfo(&parsed)
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(BrowserStateError::InvalidDisplayUrl);
    }
    Ok(())
}

fn validate_navigation_sha256(value: &str) -> Result<(), BrowserStateError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(BrowserStateError::InvalidAddress);
    };
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(BrowserStateError::InvalidAddress);
    }
    Ok(())
}

fn normalize_title(title: Option<&str>) -> Result<Option<String>, BrowserStateError> {
    let Some(title) = title else {
        return Ok(None);
    };
    let title = title.trim();
    if title.is_empty() {
        return Ok(None);
    }
    validate_title(title)?;
    Ok(Some(title.to_string()))
}

fn validate_title(title: &str) -> Result<(), BrowserStateError> {
    if title.is_empty()
        || title.len() > MAX_BROWSER_TITLE_BYTES
        || title.chars().any(char::is_control)
    {
        return Err(BrowserStateError::InvalidTitle);
    }
    Ok(())
}

fn validate_reference_identifier(value: &str) -> Result<(), BrowserStateError> {
    if value.is_empty()
        || value.len() > MAX_BROWSER_REFERENCE_ID_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
    {
        return Err(BrowserStateError::InvalidEnvironmentReference);
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(7 + digest.len() * 2);
    encoded.push_str("sha256:");
    for byte in digest {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BrowserStateError {
    #[error("browser address is invalid")]
    InvalidAddress,
    #[error("browser address scheme is unsupported")]
    UnsupportedScheme,
    #[error("browser address contains credentials")]
    AddressContainsCredentials,
    #[error("browser display URL is invalid")]
    InvalidDisplayUrl,
    #[error("browser environment reference is invalid")]
    InvalidEnvironmentReference,
    #[error("browser title is invalid")]
    InvalidTitle,
    #[error("browser history is invalid")]
    InvalidHistory,
    #[error("browser reply attempt history is invalid")]
    InvalidReplyAttempt,
    #[error("browser history has reached its boundary")]
    HistoryBoundary,
    #[error("browser navigation target does not match history")]
    HistoryTargetMismatch,
    #[error("browser state transition is invalid")]
    InvalidTransition,
    #[error("browser timestamp is invalid")]
    InvalidTimestamp,
    #[error("browser controller generation is invalid")]
    InvalidControllerGeneration,
    #[error("browser controller event is invalid")]
    InvalidControllerEvent,
    #[error("browser controller generation is stale")]
    StaleControllerGeneration,
    #[error("browser controller event sequence is stale")]
    StaleEventSequence,
    #[error("browser controller event sequence has a gap")]
    ControllerEventSequenceGap,
    #[error("browser state version is invalid")]
    InvalidVersion,
    #[error("browser state sequence overflowed")]
    SequenceOverflow,
    #[error("browser state serialization failed")]
    Serialization,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn environment() -> BrowserEnvironmentReference {
        BrowserEnvironmentReference::ephemeral("artifact-browser-1", 1).unwrap()
    }

    fn target(value: &str) -> BrowserNavigationTarget {
        BrowserNavigationTarget::parse(value).unwrap()
    }

    fn queued() -> BrowserArtifactState {
        BrowserArtifactState::new_queued(
            environment(),
            &target("example.com/start?token=secret#private"),
            7,
            10,
        )
        .unwrap()
    }

    fn metadata(sequence: u64, observed_at_ms: u64) -> BrowserControllerEventMeta {
        BrowserControllerEventMeta::new(7, sequence, observed_at_ms).unwrap()
    }

    #[test]
    fn normalization_retains_navigation_only_transiently() {
        let normalized = target("Example.COM/path?q=secret#fragment");
        assert_eq!(
            normalized.navigation_url_str(),
            "https://example.com/path?q=secret#fragment"
        );
        assert_eq!(normalized.display_url(), "https://example.com/path");

        let path_at = target("https://example.com/@owner?q=secret");
        assert_eq!(path_at.display_url(), "https://example.com/@owner");

        let state = BrowserArtifactState::new_queued(environment(), &normalized, 1, 10).unwrap();
        let serialized = serde_json::to_string(&state).unwrap();
        assert!(serialized.contains("https://example.com/path"));
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("fragment"));
    }

    #[test]
    fn normalization_rejects_hostile_schemes_credentials_controls_and_bounds() {
        for address in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,hostile",
            "ftp://example.com/file",
        ] {
            assert_eq!(
                BrowserNavigationTarget::parse(address),
                Err(BrowserStateError::UnsupportedScheme)
            );
        }
        for address in [
            "https://user:password@example.com/private",
            "https://@example.com/private",
        ] {
            assert_eq!(
                BrowserNavigationTarget::parse(address),
                Err(BrowserStateError::AddressContainsCredentials)
            );
        }
        assert_eq!(
            BrowserNavigationTarget::parse(" https://example.com"),
            Err(BrowserStateError::InvalidAddress)
        );
        assert_eq!(
            BrowserNavigationTarget::parse("https://example.com/\nnext"),
            Err(BrowserStateError::InvalidAddress)
        );
        assert_eq!(
            BrowserNavigationTarget::parse(&"a".repeat(MAX_BROWSER_NAVIGATION_URL_BYTES + 1)),
            Err(BrowserStateError::InvalidAddress)
        );
    }

    #[test]
    fn reply_navigation_requires_one_strict_envelope_and_records_attempt_once() {
        let target = browser_reply_navigation_target(
            "I can update it. <c4os-browser-navigation>{\"address\":\"https://example.com/next?private=value#section\"}</c4os-browser-navigation>",
        )
        .unwrap();
        assert_eq!(
            target.navigation_url_str(),
            "https://example.com/next?private=value#section"
        );
        assert_eq!(target.display_url(), "https://example.com/next");
        assert!(browser_reply_navigation_target("ordinary assistant prose").is_none());
        assert!(browser_reply_navigation_target(
            "<c4os-browser-navigation>{\"address\":\"https://example.com/one\"}</c4os-browser-navigation>\
             <c4os-browser-navigation>{\"address\":\"https://example.com/two\"}</c4os-browser-navigation>"
        )
        .is_none());

        let mut state = queued();
        assert!(state.record_reply_attempt("attempt-browser-1", 11).unwrap());
        assert!(!state.record_reply_attempt("attempt-browser-1", 12).unwrap());
        assert_eq!(state.processed_reply_attempts, ["attempt-browser-1"]);
        assert!(state.validate().is_ok());
    }

    #[test]
    fn environment_references_are_bounded_and_scope_exact() {
        BrowserEnvironmentReference::app_wide(1).unwrap();
        BrowserEnvironmentReference::workspace_project("workspace-1", "project-1", 1).unwrap();
        BrowserEnvironmentReference::chat("workspace-1", "project-1", "session-1", 1).unwrap();
        BrowserEnvironmentReference::ephemeral("artifact-1", 1).unwrap();

        let invalid = BrowserEnvironmentReference {
            scope: BrowserEnvironmentScope::AppWide,
            workspace_id: Some("workspace-1".into()),
            project_id: None,
            session_id: None,
            artifact_id: None,
            generation: 1,
        };
        assert_eq!(
            invalid.validate(),
            Err(BrowserStateError::InvalidEnvironmentReference)
        );
        assert_eq!(
            BrowserEnvironmentReference::ephemeral(
                "a".repeat(MAX_BROWSER_REFERENCE_ID_BYTES + 1),
                1,
            ),
            Err(BrowserStateError::InvalidEnvironmentReference)
        );
    }

    #[test]
    fn navigation_history_and_intents_follow_controller_events() {
        let mut state = queued();
        let first = target("https://example.com/start?token=other");
        state
            .start_navigation(metadata(1, 11), &first, BrowserNavigationKind::Initial)
            .unwrap();
        state
            .mark_ready(
                metadata(2, 12),
                &first,
                BrowserNavigationKind::Initial,
                Some(" First page "),
            )
            .unwrap();
        assert_eq!(state.phase.phase(), "ready");
        assert_eq!(state.current_title(), Some("First page"));
        assert_eq!(
            state.validate_navigation_intent(BrowserNavigationIntent::Back),
            Err(BrowserStateError::HistoryBoundary)
        );

        let second = target("https://example.com/second?q=private");
        state
            .start_navigation(metadata(3, 13), &second, BrowserNavigationKind::New)
            .unwrap();
        state
            .mark_ready(
                metadata(4, 14),
                &second,
                BrowserNavigationKind::New,
                Some("Second page"),
            )
            .unwrap();
        assert!(state.can_go_back());
        state
            .validate_navigation_intent(BrowserNavigationIntent::Back)
            .unwrap();

        state
            .start_navigation(metadata(5, 15), &first, BrowserNavigationKind::Back)
            .unwrap();
        state
            .mark_ready(
                metadata(6, 16),
                &first,
                BrowserNavigationKind::Back,
                Some("First page"),
            )
            .unwrap();
        assert_eq!(state.current_history_index, 0);
        assert!(state.can_go_forward());
        state
            .validate_navigation_intent(BrowserNavigationIntent::Forward)
            .unwrap();
        state
            .validate_navigation_intent(BrowserNavigationIntent::Refresh)
            .unwrap();
        assert!(state.validate().is_ok());
    }

    #[test]
    fn provisional_navigation_commits_only_after_display_and_can_be_cancelled() {
        let mut state = queued();
        let first = target("https://example.com/start?token=private");
        state
            .start_navigation(metadata(1, 11), &first, BrowserNavigationKind::Initial)
            .unwrap();
        state
            .mark_ready(
                metadata(2, 12),
                &first,
                BrowserNavigationKind::Initial,
                Some("First page"),
            )
            .unwrap();
        let committed_history = state.history.clone();

        let blocked = target("https://example.com/binary?secret=private");
        state
            .start_navigation(metadata(3, 13), &blocked, BrowserNavigationKind::New)
            .unwrap();
        assert_eq!(state.history, committed_history);
        assert_eq!(state.current_display_url(), "https://example.com/start");

        state.cancel_navigation(metadata(4, 14)).unwrap();
        assert_eq!(state.history, committed_history);
        assert_eq!(state.phase.phase(), "ready");
        assert!(!state.can_go_back());
        assert!(state.validate().is_ok());
    }

    #[test]
    fn exact_navigation_digest_distinguishes_private_targets_without_persisting_them() {
        let first_exact = target("https://example.com/start?token=one#first");
        let second_exact = target("https://example.com/start?token=two#second");
        assert_eq!(first_exact.display_url(), second_exact.display_url());
        assert_ne!(
            first_exact.navigation_sha256(),
            second_exact.navigation_sha256()
        );

        let sanitized = target(first_exact.display_url());
        let mut state = queued();
        state
            .start_navigation_bound(
                metadata(1, 11),
                &sanitized,
                BrowserNavigationKind::Initial,
                &second_exact.navigation_sha256(),
            )
            .unwrap();
        state
            .mark_ready_bound(
                metadata(2, 12),
                &sanitized,
                BrowserNavigationKind::Initial,
                &second_exact.navigation_sha256(),
                Some("Bound target"),
            )
            .unwrap();

        assert_eq!(
            state.current_navigation_sha256(),
            second_exact.navigation_sha256()
        );
        let serialized = serde_json::to_string(&state).unwrap();
        assert!(!serialized.contains("token=one"));
        assert!(!serialized.contains("token=two"));
        assert!(!serialized.contains("#first"));
        assert!(!serialized.contains("#second"));
        assert!(state.validate().is_ok());
    }

    #[test]
    fn profile_and_ephemeral_clears_rebind_environment_and_controller_generations() {
        let persistent_environment =
            BrowserEnvironmentReference::chat("workspace-1", "project-1", "session-1", 3).unwrap();
        let initial = target("https://example.com/");
        let mut persistent =
            BrowserArtifactState::new_queued(persistent_environment, &initial, 7, 10).unwrap();
        persistent.rebind_cleared_environment(4, 11).unwrap();
        assert_eq!(persistent.environment.generation, 4);
        assert_eq!(persistent.controller_generation, 8);
        assert_eq!(persistent.controller_event_sequence, 0);
        assert!(matches!(
            persistent.phase,
            BrowserPhase::Recovery {
                code: BrowserRecoveryCode::ProfileCleared,
                ..
            }
        ));
        assert_eq!(
            persistent.rebind_cleared_environment(4, 12),
            Err(BrowserStateError::InvalidEnvironmentReference)
        );
        assert!(persistent.validate().is_ok());

        let mut ephemeral = queued();
        ephemeral.clear_ephemeral_environment(11).unwrap();
        assert_eq!(ephemeral.environment.generation, 2);
        assert_eq!(ephemeral.controller_generation, 8);
        assert_eq!(ephemeral.controller_event_sequence, 0);
        assert!(matches!(
            ephemeral.phase,
            BrowserPhase::Recovery {
                code: BrowserRecoveryCode::ProfileCleared,
                ..
            }
        ));
        assert_eq!(
            persistent.clear_ephemeral_environment(13),
            Err(BrowserStateError::InvalidEnvironmentReference)
        );
        assert!(ephemeral.validate().is_ok());
    }

    #[test]
    fn stale_generations_sequences_and_gaps_fail_closed() {
        let mut state = queued();
        let first = target("https://example.com/start");
        assert_eq!(
            state.start_navigation(
                BrowserControllerEventMeta::new(6, 1, 11).unwrap(),
                &first,
                BrowserNavigationKind::Initial,
            ),
            Err(BrowserStateError::StaleControllerGeneration)
        );
        assert_eq!(
            state.start_navigation(metadata(2, 11), &first, BrowserNavigationKind::Initial),
            Err(BrowserStateError::ControllerEventSequenceGap)
        );
        state
            .start_navigation(metadata(1, 11), &first, BrowserNavigationKind::Initial)
            .unwrap();
        assert_eq!(
            state.mark_ready(
                metadata(1, 12),
                &first,
                BrowserNavigationKind::Initial,
                None,
            ),
            Err(BrowserStateError::StaleEventSequence)
        );
        assert_eq!(state.controller_event_sequence, 1);
    }

    #[test]
    fn failure_recovery_and_new_controller_are_generation_bound() {
        let mut state = queued();
        let first = target("https://example.com/start");
        state
            .start_navigation(metadata(1, 11), &first, BrowserNavigationKind::Initial)
            .unwrap();
        state
            .fail_from_controller(metadata(2, 12), BrowserErrorCode::NavigationFailed, true)
            .unwrap();
        state
            .recover_from_controller(
                metadata(3, 13),
                BrowserRecoveryCode::WebContentProcessTerminated,
            )
            .unwrap();
        state.install_controller_generation(8, 14).unwrap();
        assert_eq!(state.controller_event_sequence, 0);
        assert_eq!(state.phase.phase(), "recovery");
        assert_eq!(
            state.start_navigation(metadata(1, 15), &first, BrowserNavigationKind::Recovery),
            Err(BrowserStateError::StaleControllerGeneration)
        );
        let replacement = BrowserControllerEventMeta::new(8, 1, 15).unwrap();
        state
            .start_navigation(replacement, &first, BrowserNavigationKind::Recovery)
            .unwrap();
        assert!(state.validate().is_ok());
    }

    #[test]
    fn navigation_intents_are_phase_and_recovery_bound() {
        let mut state = queued();
        for intent in [
            BrowserNavigationIntent::Back,
            BrowserNavigationIntent::Forward,
            BrowserNavigationIntent::Refresh,
        ] {
            assert_eq!(
                state.validate_navigation_intent(intent),
                Err(BrowserStateError::InvalidTransition)
            );
        }

        state
            .begin_recovery(BrowserRecoveryCode::ApplicationRelaunch, 11)
            .unwrap();
        assert_eq!(
            state.validate_navigation_intent(BrowserNavigationIntent::Refresh),
            Ok(())
        );
        assert_eq!(
            state.validate_navigation_intent(BrowserNavigationIntent::Back),
            Err(BrowserStateError::InvalidTransition)
        );
        assert_eq!(
            state.validate_navigation_intent(BrowserNavigationIntent::Forward),
            Err(BrowserStateError::InvalidTransition)
        );
    }

    #[test]
    fn title_and_history_bounds_are_enforced() {
        let mut state = queued();
        let first = target("https://example.com/start");
        state
            .start_navigation(metadata(1, 11), &first, BrowserNavigationKind::Initial)
            .unwrap();
        assert_eq!(
            state.mark_ready(
                metadata(2, 12),
                &first,
                BrowserNavigationKind::Initial,
                Some(&"t".repeat(MAX_BROWSER_TITLE_BYTES + 1)),
            ),
            Err(BrowserStateError::InvalidTitle)
        );

        let mut oversized = queued();
        oversized.history = vec![
            BrowserHistoryEntry {
                display_url: "https://example.com/".into(),
                navigation_sha256: sha256_bytes(b"https://example.com/"),
                title: None,
            };
            MAX_BROWSER_HISTORY_ENTRIES + 1
        ];
        assert_eq!(oversized.validate(), Err(BrowserStateError::InvalidHistory));
    }

    #[test]
    fn serde_rejects_unknown_fields_at_every_durable_boundary() {
        let state = queued();
        let mut serialized = serde_json::to_value(&state).unwrap();
        serialized
            .as_object_mut()
            .unwrap()
            .insert("cookies".into(), json!(["secret"]));
        assert!(serde_json::from_value::<BrowserArtifactState>(serialized).is_err());

        let mut environment = serde_json::to_value(environment()).unwrap();
        environment
            .as_object_mut()
            .unwrap()
            .insert("profileUuid".into(), json!("must-not-persist"));
        assert!(serde_json::from_value::<BrowserEnvironmentReference>(environment).is_err());

        let mut history = serde_json::to_value(state.history[0].clone()).unwrap();
        history
            .as_object_mut()
            .unwrap()
            .insert("pageHtml".into(), json!("<html>secret</html>"));
        assert!(serde_json::from_value::<BrowserHistoryEntry>(history).is_err());
    }
}
