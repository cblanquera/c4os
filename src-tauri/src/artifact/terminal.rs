use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use thiserror::Error;

use super::record::ArtifactResourceVersion;

pub const MAX_TERMINAL_COMMAND_BYTES: usize = 16 * 1_024;
pub const MAX_TERMINAL_OUTPUT_BYTES: usize = 256 * 1_024;
pub const MAX_TERMINAL_STATUS_MESSAGE_BYTES: usize = 8 * 1_024;
pub const MIN_TERMINAL_COLUMNS: u16 = 20;
pub const MAX_TERMINAL_COLUMNS: u16 = 500;
pub const MIN_TERMINAL_ROWS: u16 = 4;
pub const MAX_TERMINAL_ROWS: u16 = 300;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerminalCommandIdentity {
    pub terminal_session_id: String,
    pub command_id: String,
    pub command_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerminalProcessProvenance {
    pub shell_path: String,
    pub environment_id: String,
    pub environment_generation: u64,
    pub process_generation: u64,
    pub shell_process_id: Option<u32>,
    pub foreground_process_group_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerminalDimensions {
    pub columns: u16,
    pub rows: u16,
}

impl TerminalDimensions {
    pub fn new(columns: u16, rows: u16) -> Result<Self, TerminalStateError> {
        let dimensions = Self { columns, rows };
        dimensions.validate()?;
        Ok(dimensions)
    }

    fn validate(&self) -> Result<(), TerminalStateError> {
        if !(MIN_TERMINAL_COLUMNS..=MAX_TERMINAL_COLUMNS).contains(&self.columns)
            || !(MIN_TERMINAL_ROWS..=MAX_TERMINAL_ROWS).contains(&self.rows)
        {
            return Err(TerminalStateError::InvalidDimensions);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerminalOutputBuffer {
    pub sequence: u64,
    pub retained_base64: String,
    pub retained_bytes: u64,
    pub dropped_bytes: u64,
    pub retained_sha256: String,
    pub observed_at_ms: u64,
}

impl TerminalOutputBuffer {
    fn empty(observed_at_ms: u64) -> Result<Self, TerminalStateError> {
        if observed_at_ms == 0 {
            return Err(TerminalStateError::InvalidTimestamp);
        }
        Ok(Self {
            sequence: 1,
            retained_base64: String::new(),
            retained_bytes: 0,
            dropped_bytes: 0,
            retained_sha256: sha256_bytes(&[]),
            observed_at_ms,
        })
    }

    pub fn retained_bytes(&self) -> Result<Vec<u8>, TerminalStateError> {
        BASE64_STANDARD
            .decode(&self.retained_base64)
            .map_err(|_| TerminalStateError::InvalidOutput)
    }

    /// Produces bounded context/copy text without terminal control sequences.
    /// Raw bytes remain authoritative for xterm replay; this projection is not
    /// written back into the terminal stream.
    pub fn safe_text(&self) -> Result<String, TerminalStateError> {
        let safe = strip_terminal_controls(&String::from_utf8_lossy(&self.retained_bytes()?));
        Ok(bounded_utf8_tail(safe, MAX_TERMINAL_OUTPUT_BYTES))
    }

    pub fn is_safe_for_automatic_context(&self) -> Result<bool, TerminalStateError> {
        Ok(self
            .retained_bytes()?
            .iter()
            .all(|byte| !byte.is_ascii_control() || matches!(*byte, b'\n' | b'\r' | b'\t')))
    }

    fn append(
        &mut self,
        bytes: &[u8],
        dropped_bytes_before: u64,
        observed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        if (bytes.is_empty() && dropped_bytes_before == 0) || observed_at_ms < self.observed_at_ms {
            return Err(TerminalStateError::InvalidOutput);
        }
        self.sequence
            .checked_add(1)
            .ok_or(TerminalStateError::SequenceOverflow)?;
        self.dropped_bytes
            .checked_add(dropped_bytes_before)
            .ok_or(TerminalStateError::SequenceOverflow)?;
        let mut retained = self.retained_bytes()?;
        retained.extend_from_slice(bytes);
        if retained.len() > MAX_TERMINAL_OUTPUT_BYTES {
            let dropped = retained.len() - MAX_TERMINAL_OUTPUT_BYTES;
            self.dropped_bytes
                .checked_add(dropped as u64)
                .ok_or(TerminalStateError::SequenceOverflow)?;
            retained.drain(..dropped);
            self.dropped_bytes = self
                .dropped_bytes
                .checked_add(dropped as u64)
                .ok_or(TerminalStateError::SequenceOverflow)?;
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or(TerminalStateError::SequenceOverflow)?;
        self.dropped_bytes = self
            .dropped_bytes
            .checked_add(dropped_bytes_before)
            .ok_or(TerminalStateError::SequenceOverflow)?;
        self.retained_bytes = retained.len() as u64;
        self.retained_sha256 = sha256_bytes(&retained);
        self.retained_base64 = BASE64_STANDARD.encode(retained);
        self.observed_at_ms = observed_at_ms;
        Ok(())
    }

    fn validate(&self) -> Result<(), TerminalStateError> {
        if self.sequence == 0 || self.observed_at_ms == 0 {
            return Err(TerminalStateError::InvalidOutput);
        }
        let retained = self.retained_bytes()?;
        if retained.len() > MAX_TERMINAL_OUTPUT_BYTES
            || self.retained_bytes != retained.len() as u64
            || self.retained_sha256 != sha256_bytes(&retained)
        {
            return Err(TerminalStateError::InvalidOutput);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase", tag = "phase")]
pub enum TerminalCommandStatus {
    Queued {
        queued_at_ms: u64,
    },
    Running {
        started_at_ms: u64,
        stdin_ready: bool,
        stop_requested_at_ms: Option<u64>,
    },
    Completed {
        exit_code: i32,
        completed_at_ms: u64,
    },
    Interrupted {
        exit_code: i32,
        interrupted_at_ms: u64,
        shell_replaced: bool,
    },
    Failed {
        code: String,
        message: String,
        retryable: bool,
        failed_at_ms: u64,
    },
    Recovery {
        code: String,
        message: String,
        recovered_at_ms: u64,
    },
}

impl TerminalCommandStatus {
    pub fn phase(&self) -> &'static str {
        match self {
            Self::Queued { .. } => "queued",
            Self::Running {
                stop_requested_at_ms: Some(_),
                ..
            } => "stopping",
            Self::Running {
                stdin_ready: true, ..
            } => "stdinReady",
            Self::Running {
                stdin_ready: false, ..
            } => "running",
            Self::Completed { .. } => "completed",
            Self::Interrupted { .. } => "interrupted",
            Self::Failed { .. } => "failed",
            Self::Recovery { .. } => "recovery",
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        match self {
            Self::Completed { exit_code, .. } | Self::Interrupted { exit_code, .. } => {
                Some(*exit_code)
            }
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed { .. }
                | Self::Interrupted { .. }
                | Self::Failed { .. }
                | Self::Recovery { .. }
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TerminalArtifactState {
    pub identity: TerminalCommandIdentity,
    pub command: String,
    pub working_directory_display: String,
    pub dimensions: TerminalDimensions,
    pub process: TerminalProcessProvenance,
    pub status: TerminalCommandStatus,
    pub output: TerminalOutputBuffer,
    pub version: ArtifactResourceVersion,
}

impl TerminalArtifactState {
    #[allow(clippy::too_many_arguments)]
    pub fn new_queued(
        identity: TerminalCommandIdentity,
        command: impl Into<String>,
        working_directory_display: impl Into<String>,
        dimensions: TerminalDimensions,
        process: TerminalProcessProvenance,
        queued_at_ms: u64,
    ) -> Result<Self, TerminalStateError> {
        let mut state = Self {
            identity,
            command: command.into(),
            working_directory_display: working_directory_display.into(),
            dimensions,
            process,
            status: TerminalCommandStatus::Queued { queued_at_ms },
            output: TerminalOutputBuffer::empty(queued_at_ms)?,
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

    pub fn mark_running(
        &mut self,
        shell_process_id: u32,
        foreground_process_group_id: Option<u32>,
        stdin_ready: bool,
        started_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(started_at_ms)?;
        let TerminalCommandStatus::Queued { queued_at_ms } = &self.status else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if shell_process_id == 0 || started_at_ms < *queued_at_ms {
            return Err(TerminalStateError::InvalidTransition);
        }
        if foreground_process_group_id == Some(0) {
            return Err(TerminalStateError::InvalidProcess);
        }
        self.process.shell_process_id = Some(shell_process_id);
        self.process.foreground_process_group_id = foreground_process_group_id;
        self.status = TerminalCommandStatus::Running {
            started_at_ms,
            stdin_ready,
            stop_requested_at_ms: None,
        };
        self.refresh_version(started_at_ms, true)
    }

    pub fn set_stdin_ready(
        &mut self,
        stdin_ready: bool,
        observed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(observed_at_ms)?;
        let TerminalCommandStatus::Running {
            started_at_ms,
            stdin_ready: current,
            stop_requested_at_ms,
        } = &self.status
        else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if observed_at_ms < *started_at_ms || *current == stdin_ready {
            return Err(TerminalStateError::InvalidTransition);
        }
        self.status = TerminalCommandStatus::Running {
            started_at_ms: *started_at_ms,
            stdin_ready,
            stop_requested_at_ms: *stop_requested_at_ms,
        };
        self.refresh_version(observed_at_ms, true)
    }

    pub fn note_input_submitted(&mut self, observed_at_ms: u64) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(observed_at_ms)?;
        let TerminalCommandStatus::Running {
            started_at_ms,
            stdin_ready: true,
            stop_requested_at_ms: None,
        } = &self.status
        else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if observed_at_ms < *started_at_ms {
            return Err(TerminalStateError::InvalidTransition);
        }
        self.refresh_version(observed_at_ms, true)
    }

    pub fn request_stop(&mut self, requested_at_ms: u64) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(requested_at_ms)?;
        let TerminalCommandStatus::Running {
            started_at_ms,
            stdin_ready,
            stop_requested_at_ms: None,
        } = &self.status
        else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if requested_at_ms < *started_at_ms {
            return Err(TerminalStateError::InvalidTransition);
        }
        self.status = TerminalCommandStatus::Running {
            started_at_ms: *started_at_ms,
            stdin_ready: *stdin_ready,
            stop_requested_at_ms: Some(requested_at_ms),
        };
        self.refresh_version(requested_at_ms, true)
    }

    pub fn append_output(
        &mut self,
        bytes: &[u8],
        observed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.append_output_with_dropped(bytes, 0, observed_at_ms)
    }

    pub fn append_output_with_dropped(
        &mut self,
        bytes: &[u8],
        dropped_bytes_before: u64,
        observed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(observed_at_ms)?;
        if !matches!(&self.status, TerminalCommandStatus::Running { .. }) {
            return Err(TerminalStateError::InvalidTransition);
        }
        self.output
            .append(bytes, dropped_bytes_before, observed_at_ms)?;
        self.refresh_version(observed_at_ms, true)
    }

    pub fn resize(
        &mut self,
        dimensions: TerminalDimensions,
        observed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(observed_at_ms)?;
        if !matches!(&self.status, TerminalCommandStatus::Running { .. })
            || dimensions == self.dimensions
        {
            return Err(TerminalStateError::InvalidTransition);
        }
        dimensions.validate()?;
        self.dimensions = dimensions;
        self.refresh_version(observed_at_ms, true)
    }

    pub fn complete(
        &mut self,
        exit_code: i32,
        working_directory_display: impl Into<String>,
        completed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(completed_at_ms)?;
        let TerminalCommandStatus::Running {
            started_at_ms,
            stop_requested_at_ms: None,
            ..
        } = &self.status
        else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if completed_at_ms < *started_at_ms {
            return Err(TerminalStateError::InvalidTransition);
        }
        let working_directory_display = working_directory_display.into();
        validate_display_path(&working_directory_display)?;
        self.working_directory_display = working_directory_display;
        self.process.foreground_process_group_id = None;
        self.status = TerminalCommandStatus::Completed {
            exit_code,
            completed_at_ms,
        };
        self.refresh_version(completed_at_ms, true)
    }

    pub fn interrupt(
        &mut self,
        shell_replaced: bool,
        interrupted_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(interrupted_at_ms)?;
        let TerminalCommandStatus::Running {
            started_at_ms,
            stop_requested_at_ms: Some(_),
            ..
        } = &self.status
        else {
            return Err(TerminalStateError::InvalidTransition);
        };
        if interrupted_at_ms < *started_at_ms {
            return Err(TerminalStateError::InvalidTransition);
        }
        self.process.foreground_process_group_id = None;
        if shell_replaced {
            self.process.shell_process_id = None;
        }
        self.status = TerminalCommandStatus::Interrupted {
            exit_code: 130,
            interrupted_at_ms,
            shell_replaced,
        };
        self.refresh_version(interrupted_at_ms, true)
    }

    pub fn fail(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
        failed_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(failed_at_ms)?;
        if self.status.is_terminal() {
            return Err(TerminalStateError::InvalidTransition);
        }
        let code = code.into();
        let message = message.into();
        validate_identifier(&code)?;
        validate_message(&message)?;
        self.process.foreground_process_group_id = None;
        self.process.shell_process_id = None;
        self.status = TerminalCommandStatus::Failed {
            code,
            message,
            retryable,
            failed_at_ms,
        };
        self.refresh_version(failed_at_ms, true)
    }

    pub fn recover(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        recovered_at_ms: u64,
    ) -> Result<(), TerminalStateError> {
        self.require_transition_timestamp(recovered_at_ms)?;
        if self.status.is_terminal() {
            return Err(TerminalStateError::InvalidTransition);
        }
        let code = code.into();
        let message = message.into();
        validate_identifier(&code)?;
        validate_message(&message)?;
        self.process.shell_process_id = None;
        self.process.foreground_process_group_id = None;
        self.status = TerminalCommandStatus::Recovery {
            code,
            message,
            recovered_at_ms,
        };
        self.refresh_version(recovered_at_ms, true)
    }

    pub fn validate(&self) -> Result<(), TerminalStateError> {
        validate_identifier(&self.identity.terminal_session_id)?;
        validate_identifier(&self.identity.command_id)?;
        if self.identity.command_sequence == 0 {
            return Err(TerminalStateError::InvalidIdentity);
        }
        validate_command(&self.command)?;
        validate_display_path(&self.working_directory_display)?;
        self.dimensions.validate()?;
        self.process.validate()?;
        self.output.validate()?;
        self.version
            .validate()
            .map_err(|_| TerminalStateError::InvalidVersion)?;
        validate_status(&self.status)?;
        match &self.status {
            TerminalCommandStatus::Queued { .. }
                if self.process.shell_process_id.is_some()
                    || self.process.foreground_process_group_id.is_some() =>
            {
                return Err(TerminalStateError::InvalidProcess);
            }
            TerminalCommandStatus::Running { .. }
                if self.process.shell_process_id.is_none()
                    || self.process.foreground_process_group_id.is_none() =>
            {
                return Err(TerminalStateError::InvalidProcess);
            }
            TerminalCommandStatus::Completed { .. }
                if self.process.shell_process_id.is_none()
                    || self.process.foreground_process_group_id.is_some() =>
            {
                return Err(TerminalStateError::InvalidProcess);
            }
            TerminalCommandStatus::Interrupted {
                shell_replaced: false,
                ..
            } if self.process.shell_process_id.is_none()
                || self.process.foreground_process_group_id.is_some() =>
            {
                return Err(TerminalStateError::InvalidProcess);
            }
            TerminalCommandStatus::Interrupted {
                shell_replaced: true,
                ..
            }
            | TerminalCommandStatus::Failed { .. }
            | TerminalCommandStatus::Recovery { .. }
                if self.process.shell_process_id.is_some()
                    || self.process.foreground_process_group_id.is_some() =>
            {
                return Err(TerminalStateError::InvalidProcess);
            }
            _ => {}
        }
        let expected = self.state_digest()?;
        if expected != self.version.sha256 {
            return Err(TerminalStateError::InvalidVersion);
        }
        Ok(())
    }

    pub fn as_resource_version(&self) -> ArtifactResourceVersion {
        self.version.clone()
    }

    fn refresh_version(
        &mut self,
        observed_at_ms: u64,
        increment: bool,
    ) -> Result<(), TerminalStateError> {
        if observed_at_ms == 0 || observed_at_ms < self.version.observed_at_ms {
            return Err(TerminalStateError::InvalidTimestamp);
        }
        if increment {
            self.version.sequence = self
                .version
                .sequence
                .checked_add(1)
                .ok_or(TerminalStateError::SequenceOverflow)?;
        }
        self.version.observed_at_ms = observed_at_ms;
        self.version.sha256 = self.state_digest()?;
        Ok(())
    }

    fn require_transition_timestamp(&self, observed_at_ms: u64) -> Result<(), TerminalStateError> {
        if observed_at_ms == 0 || observed_at_ms < self.version.observed_at_ms {
            return Err(TerminalStateError::InvalidTimestamp);
        }
        self.version
            .sequence
            .checked_add(1)
            .ok_or(TerminalStateError::SequenceOverflow)?;
        Ok(())
    }

    fn state_digest(&self) -> Result<String, TerminalStateError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DigestInput<'a> {
            identity: &'a TerminalCommandIdentity,
            command: &'a str,
            working_directory_display: &'a str,
            dimensions: TerminalDimensions,
            process: &'a TerminalProcessProvenance,
            status: &'a TerminalCommandStatus,
            output: &'a TerminalOutputBuffer,
        }
        let bytes = serde_json::to_vec(&DigestInput {
            identity: &self.identity,
            command: &self.command,
            working_directory_display: &self.working_directory_display,
            dimensions: self.dimensions,
            process: &self.process,
            status: &self.status,
            output: &self.output,
        })
        .map_err(|_| TerminalStateError::Serialization)?;
        Ok(sha256_bytes(&bytes))
    }
}

impl TerminalProcessProvenance {
    fn validate(&self) -> Result<(), TerminalStateError> {
        if !self.shell_path.starts_with('/')
            || self.shell_path.len() > 4_096
            || self.shell_path.chars().any(char::is_control)
        {
            return Err(TerminalStateError::InvalidProcess);
        }
        validate_identifier(&self.environment_id)?;
        if self.environment_generation == 0
            || self.process_generation == 0
            || self.shell_process_id == Some(0)
            || self.foreground_process_group_id == Some(0)
        {
            return Err(TerminalStateError::InvalidProcess);
        }
        Ok(())
    }
}

fn validate_status(status: &TerminalCommandStatus) -> Result<(), TerminalStateError> {
    let timestamp = match status {
        TerminalCommandStatus::Queued { queued_at_ms } => *queued_at_ms,
        TerminalCommandStatus::Running {
            started_at_ms,
            stop_requested_at_ms,
            ..
        } => {
            if stop_requested_at_ms.is_some_and(|requested| requested < *started_at_ms) {
                return Err(TerminalStateError::InvalidStatus);
            }
            *started_at_ms
        }
        TerminalCommandStatus::Completed {
            completed_at_ms, ..
        } => *completed_at_ms,
        TerminalCommandStatus::Interrupted {
            exit_code,
            interrupted_at_ms,
            ..
        } => {
            if *exit_code != 130 {
                return Err(TerminalStateError::InvalidStatus);
            }
            *interrupted_at_ms
        }
        TerminalCommandStatus::Failed {
            code,
            message,
            failed_at_ms,
            ..
        } => {
            validate_identifier(code)?;
            validate_message(message)?;
            *failed_at_ms
        }
        TerminalCommandStatus::Recovery {
            code,
            message,
            recovered_at_ms,
        } => {
            validate_identifier(code)?;
            validate_message(message)?;
            *recovered_at_ms
        }
    };
    if timestamp == 0 {
        return Err(TerminalStateError::InvalidTimestamp);
    }
    Ok(())
}

fn validate_command(command: &str) -> Result<(), TerminalStateError> {
    if command.trim().is_empty()
        || command.len() > MAX_TERMINAL_COMMAND_BYTES
        || command.chars().any(char::is_control)
    {
        return Err(TerminalStateError::InvalidCommand);
    }
    Ok(())
}

fn validate_display_path(path: &str) -> Result<(), TerminalStateError> {
    if path.is_empty() || path.len() > 4_096 || path.chars().any(char::is_control) {
        return Err(TerminalStateError::InvalidWorkingDirectory);
    }
    Ok(())
}

fn validate_identifier(value: &str) -> Result<(), TerminalStateError> {
    if value.is_empty()
        || value.len() > 160
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.:@".contains(character))
    {
        return Err(TerminalStateError::InvalidIdentity);
    }
    Ok(())
}

fn validate_message(value: &str) -> Result<(), TerminalStateError> {
    if value.is_empty()
        || value.len() > MAX_TERMINAL_STATUS_MESSAGE_BYTES
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(TerminalStateError::InvalidStatus);
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

fn strip_terminal_controls(value: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Text,
        Escape,
        Csi,
        String,
        StringEscape,
    }

    let mut state = State::Text;
    let mut safe = String::with_capacity(value.len());
    for character in value.chars() {
        state = match state {
            State::Text => match character {
                '\u{1b}' => State::Escape,
                '\n' | '\r' | '\t' => {
                    safe.push(character);
                    State::Text
                }
                character if character.is_control() => State::Text,
                character => {
                    safe.push(character);
                    State::Text
                }
            },
            State::Escape => match character {
                '[' => State::Csi,
                ']' | 'P' | '_' | '^' => State::String,
                _ => State::Text,
            },
            State::Csi => {
                if ('@'..='~').contains(&character) {
                    State::Text
                } else {
                    State::Csi
                }
            }
            State::String => match character {
                '\u{7}' => State::Text,
                '\u{1b}' => State::StringEscape,
                _ => State::String,
            },
            State::StringEscape => {
                if character == '\\' {
                    State::Text
                } else {
                    State::String
                }
            }
        };
    }
    safe
}

fn bounded_utf8_tail(mut value: String, maximum_bytes: usize) -> String {
    if value.len() <= maximum_bytes {
        return value;
    }
    let mut start = value.len() - maximum_bytes;
    while !value.is_char_boundary(start) {
        start += 1;
    }
    value.drain(..start);
    value
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TerminalStateError {
    #[error("terminal command identity is invalid")]
    InvalidIdentity,
    #[error("terminal command is invalid")]
    InvalidCommand,
    #[error("terminal dimensions are invalid")]
    InvalidDimensions,
    #[error("terminal process provenance is invalid")]
    InvalidProcess,
    #[error("terminal working directory display is invalid")]
    InvalidWorkingDirectory,
    #[error("terminal output is invalid")]
    InvalidOutput,
    #[error("terminal status is invalid")]
    InvalidStatus,
    #[error("terminal state transition is invalid")]
    InvalidTransition,
    #[error("terminal timestamp is invalid")]
    InvalidTimestamp,
    #[error("terminal state version is invalid")]
    InvalidVersion,
    #[error("terminal sequence overflowed")]
    SequenceOverflow,
    #[error("terminal state serialization failed")]
    Serialization,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queued() -> TerminalArtifactState {
        TerminalArtifactState::new_queued(
            TerminalCommandIdentity {
                terminal_session_id: "terminal-session-1".into(),
                command_id: "command-1".into(),
                command_sequence: 1,
            },
            "printf hello",
            "/project",
            TerminalDimensions::new(100, 30).unwrap(),
            TerminalProcessProvenance {
                shell_path: "/bin/zsh".into(),
                environment_id: "environment-1".into(),
                environment_generation: 1,
                process_generation: 1,
                shell_process_id: None,
                foreground_process_group_id: None,
            },
            10,
        )
        .unwrap()
    }

    #[test]
    fn command_lifecycle_preserves_immutable_identity_and_exit() {
        let mut state = queued();
        let identity = state.identity.clone();
        let command = state.command.clone();
        state.mark_running(42, Some(43), true, 11).unwrap();
        state.append_output(b"hello\r\n", 12).unwrap();
        state.complete(0, "/project/subdir", 13).unwrap();

        assert_eq!(state.identity, identity);
        assert_eq!(state.command, command);
        assert_eq!(state.status.exit_code(), Some(0));
        assert_eq!(state.output.retained_bytes().unwrap(), b"hello\r\n");
        assert!(state.validate().is_ok());
    }

    #[test]
    fn stop_is_one_way_and_normalizes_to_130() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        state.request_stop(12).unwrap();
        assert_eq!(
            state.request_stop(13),
            Err(TerminalStateError::InvalidTransition)
        );
        state.append_output(b"^C\r\n", 14).unwrap();
        state.interrupt(false, 15).unwrap();
        assert_eq!(state.status.exit_code(), Some(130));
        assert!(state.status.is_terminal());
        assert!(state.validate().is_ok());
    }

    #[test]
    fn output_retains_a_bounded_tail_with_drop_accounting() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        let bytes = vec![b'x'; MAX_TERMINAL_OUTPUT_BYTES + 11];
        state.append_output(&bytes, 12).unwrap();
        assert_eq!(
            state.output.retained_bytes,
            MAX_TERMINAL_OUTPUT_BYTES as u64
        );
        assert_eq!(state.output.dropped_bytes, 11);
        assert!(state.validate().is_ok());
    }

    #[test]
    fn running_state_recovers_without_trusting_persisted_process_ids() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        state
            .recover(
                "application-restarted",
                "The prior shell could not be reattached.",
                12,
            )
            .unwrap();
        assert_eq!(state.process.shell_process_id, None);
        assert_eq!(state.process.foreground_process_group_id, None);
        assert_eq!(state.status.phase(), "recovery");
        assert!(state.validate().is_ok());
    }

    #[test]
    fn serialized_state_detects_output_tampering() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        state.append_output(b"safe", 12).unwrap();
        state.output.retained_base64 = BASE64_STANDARD.encode(b"changed");
        assert_eq!(state.validate(), Err(TerminalStateError::InvalidOutput));
    }

    #[test]
    fn safe_text_removes_ansi_and_osc_controls() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        state
            .append_output(b"plain\x1b[31mred\x1b[0m\x1b]52;c;secret\x07\n", 12)
            .unwrap();
        assert_eq!(state.output.safe_text().unwrap(), "plainred\n");
    }

    #[test]
    fn safe_text_independently_bounds_invalid_utf8_expansion() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 11).unwrap();
        state
            .append_output(&vec![0xff; MAX_TERMINAL_OUTPUT_BYTES], 12)
            .unwrap();

        let safe = state.output.safe_text().unwrap();
        assert!(safe.len() <= MAX_TERMINAL_OUTPUT_BYTES);
        assert!(safe.is_char_boundary(0));
        assert!(safe.chars().all(|character| !character.is_control()));
    }

    #[test]
    fn safe_text_tail_bound_never_splits_utf8() {
        let value = format!("{}tail", "é".repeat(MAX_TERMINAL_OUTPUT_BYTES));
        let bounded = bounded_utf8_tail(value, MAX_TERMINAL_OUTPUT_BYTES);
        assert!(bounded.len() <= MAX_TERMINAL_OUTPUT_BYTES);
        assert!(bounded.ends_with("tail"));
    }

    #[test]
    fn command_rejects_terminal_editing_controls() {
        for command in ["echo\tvalue", "echo\u{1b}[D", "echo\u{7f}", "echo\u{4}"] {
            assert_eq!(
                TerminalArtifactState::new_queued(
                    TerminalCommandIdentity {
                        terminal_session_id: "terminal-session-1".into(),
                        command_id: "command-1".into(),
                        command_sequence: 1,
                    },
                    command,
                    "/project",
                    TerminalDimensions::new(100, 30).unwrap(),
                    TerminalProcessProvenance {
                        shell_path: "/bin/zsh".into(),
                        environment_id: "environment-1".into(),
                        environment_generation: 1,
                        process_generation: 1,
                        shell_process_id: None,
                        foreground_process_group_id: None,
                    },
                    10,
                ),
                Err(TerminalStateError::InvalidCommand)
            );
        }
    }

    #[test]
    fn stale_event_failure_leaves_state_unchanged() {
        let mut state = queued();
        state.mark_running(42, Some(43), false, 20).unwrap();
        state
            .resize(TerminalDimensions::new(120, 40).unwrap(), 30)
            .unwrap();
        let before = state.clone();
        assert_eq!(
            state.append_output(b"late", 25),
            Err(TerminalStateError::InvalidTimestamp)
        );
        assert_eq!(state, before);
    }
}
