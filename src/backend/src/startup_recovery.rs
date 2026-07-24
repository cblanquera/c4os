//! Fail-closed startup recovery authority.
//!
//! This module owns only bounded, renderer-safe recovery state. Native backup
//! paths and recovery locations remain in the production bootstrap layer. A
//! prepared action identifies what that layer may do without carrying a path,
//! secret, process handle, or other ambient authority.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const STARTUP_RECOVERY_SCHEMA_VERSION: u16 = 1;
pub const MAX_STARTUP_RECOVERY_HISTORY: usize = 64;
pub const MAX_STARTUP_RECOVERY_IDENTIFIER_BYTES: usize = 256;
pub const MAX_STARTUP_RECOVERY_MESSAGE_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupRecoveryBoundary {
    Database,
    Configuration,
    BrowserRegistry,
    Extension,
    Workspace,
    Runtime,
    Mcp,
    Update,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupRecoveryLifecycle {
    Healthy,
    Degraded,
    Retrying,
    Recovered,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupRecoveryAction {
    Retry,
    RestoreValidatedBackup,
    OpenRecoveryLocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartupRecoveryHistoryKind {
    FailureReported,
    ActionStarted,
    ActionFailed,
    ActionSucceeded,
    RecoveryLocationOpened,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartupRecoveryFailureInput {
    pub boundary: StartupRecoveryBoundary,
    pub correlation_id: String,
    pub diagnostic_code: String,
    pub message: String,
    pub failed_at_ms: u64,
    pub validated_backup_available: bool,
    pub recovery_location_available: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartupRecoveryActionInput {
    pub expected_generation: u64,
    pub action: StartupRecoveryAction,
    pub requested_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupRecoveryFailureSnapshot {
    pub boundary: StartupRecoveryBoundary,
    pub correlation_id: String,
    pub diagnostic_code: String,
    pub message: String,
    pub failed_at_ms: u64,
    pub validated_backup_available: bool,
    pub recovery_location_available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupRecoveryActiveActionSnapshot {
    pub boundary: StartupRecoveryBoundary,
    pub action: StartupRecoveryAction,
    pub started_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupRecoveryHistorySnapshot {
    pub generation: u64,
    pub boundary: StartupRecoveryBoundary,
    pub lifecycle: StartupRecoveryLifecycle,
    pub kind: StartupRecoveryHistoryKind,
    pub action: Option<StartupRecoveryAction>,
    pub correlation_id: String,
    pub diagnostic_code: String,
    pub message: String,
    pub occurred_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupRecoverySnapshot {
    pub schema_version: u16,
    pub authority: &'static str,
    pub generation: u64,
    pub lifecycle: StartupRecoveryLifecycle,
    pub normal_work_authorized: bool,
    pub failure: Option<StartupRecoveryFailureSnapshot>,
    pub available_actions: Vec<StartupRecoveryAction>,
    pub active_action: Option<StartupRecoveryActiveActionSnapshot>,
    pub history: Vec<StartupRecoveryHistorySnapshot>,
    pub history_truncated: u64,
}

/// Opaque permission for the production bootstrap layer to perform exactly one
/// recovery operation. It deliberately contains no native path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartupRecoveryPreparedAction {
    attempt_id: u64,
    generation: u64,
    boundary: StartupRecoveryBoundary,
    action: StartupRecoveryAction,
}

impl StartupRecoveryPreparedAction {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn boundary(&self) -> StartupRecoveryBoundary {
        self.boundary
    }

    pub fn action(&self) -> StartupRecoveryAction {
        self.action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartupRecoveryActionOutcome {
    Succeeded {
        completed_at_ms: u64,
    },
    Failed {
        diagnostic_code: String,
        message: String,
        completed_at_ms: u64,
    },
}

#[derive(Clone, Debug)]
struct PendingAction {
    attempt_id: u64,
    generation: u64,
    boundary: StartupRecoveryBoundary,
    action: StartupRecoveryAction,
    started_at_ms: u64,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum StartupRecoveryError {
    #[error("startup recovery input is invalid")]
    InvalidInput,
    #[error(
        "startup recovery generation is stale: expected {expected_generation}, current {actual_generation}"
    )]
    StaleGeneration {
        expected_generation: u64,
        actual_generation: u64,
    },
    #[error("startup recovery action is unavailable")]
    UnavailableAction,
    #[error("startup recovery state does not allow this operation")]
    InvalidState,
    #[error("startup recovery action completion does not match the active attempt")]
    AttemptMismatch,
    #[error("startup recovery generation is exhausted")]
    GenerationExhausted,
}

#[derive(Clone, Debug)]
pub struct StartupRecoveryController {
    generation: u64,
    lifecycle: StartupRecoveryLifecycle,
    failure: Option<StartupRecoveryFailureSnapshot>,
    pending_action: Option<PendingAction>,
    history: Vec<StartupRecoveryHistorySnapshot>,
    history_truncated: u64,
    next_attempt_id: u64,
}

impl Default for StartupRecoveryController {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupRecoveryController {
    pub fn new() -> Self {
        Self {
            generation: 0,
            lifecycle: StartupRecoveryLifecycle::Healthy,
            failure: None,
            pending_action: None,
            history: Vec::new(),
            history_truncated: 0,
            next_attempt_id: 1,
        }
    }

    pub fn snapshot(&self) -> StartupRecoverySnapshot {
        StartupRecoverySnapshot {
            schema_version: STARTUP_RECOVERY_SCHEMA_VERSION,
            authority: "rust-startup-recovery",
            generation: self.generation,
            lifecycle: self.lifecycle,
            normal_work_authorized: self.normal_work_authorized(),
            failure: self.failure.clone(),
            available_actions: self.available_actions(),
            active_action: self.pending_action.as_ref().map(|pending| {
                StartupRecoveryActiveActionSnapshot {
                    boundary: pending.boundary,
                    action: pending.action,
                    started_at_ms: pending.started_at_ms,
                }
            }),
            history: self.history.clone(),
            history_truncated: self.history_truncated,
        }
    }

    /// This is the production bootstrap gate. Any internally inconsistent
    /// state fails closed because authority requires every healthy invariant.
    pub fn normal_work_authorized(&self) -> bool {
        matches!(
            self.lifecycle,
            StartupRecoveryLifecycle::Healthy | StartupRecoveryLifecycle::Recovered
        ) && self.failure.is_none()
            && self.pending_action.is_none()
    }

    /// Returns true for every state that must block normal product work,
    /// including an in-progress retry or restore.
    pub fn is_degraded(&self) -> bool {
        !self.normal_work_authorized()
    }

    pub fn report_failure(
        &mut self,
        input: StartupRecoveryFailureInput,
    ) -> Result<StartupRecoverySnapshot, StartupRecoveryError> {
        validate_failure_input(&input)?;
        if self.pending_action.is_some() {
            return Err(StartupRecoveryError::InvalidState);
        }
        let generation = self.next_generation()?;
        let failure = StartupRecoveryFailureSnapshot {
            boundary: input.boundary,
            correlation_id: input.correlation_id,
            diagnostic_code: input.diagnostic_code,
            message: input.message,
            failed_at_ms: input.failed_at_ms,
            validated_backup_available: input.validated_backup_available,
            recovery_location_available: input.recovery_location_available,
        };
        self.generation = generation;
        self.lifecycle = StartupRecoveryLifecycle::Degraded;
        self.failure = Some(failure.clone());
        self.push_history(StartupRecoveryHistorySnapshot {
            generation,
            boundary: failure.boundary,
            lifecycle: self.lifecycle,
            kind: StartupRecoveryHistoryKind::FailureReported,
            action: None,
            correlation_id: failure.correlation_id.clone(),
            diagnostic_code: failure.diagnostic_code.clone(),
            message: failure.message.clone(),
            occurred_at_ms: failure.failed_at_ms,
        });
        Ok(self.snapshot())
    }

    pub fn prepare_action(
        &mut self,
        input: StartupRecoveryActionInput,
    ) -> Result<StartupRecoveryPreparedAction, StartupRecoveryError> {
        validate_timestamp(input.requested_at_ms)?;
        self.require_generation(input.expected_generation)?;
        if self.pending_action.is_some() || self.lifecycle != StartupRecoveryLifecycle::Degraded {
            return Err(StartupRecoveryError::InvalidState);
        }
        let failure = self
            .failure
            .as_ref()
            .ok_or(StartupRecoveryError::InvalidState)?;
        if !action_available(failure, input.action) {
            return Err(StartupRecoveryError::UnavailableAction);
        }
        let generation = self.next_generation()?;
        let attempt_id = self.next_attempt_id;
        let next_attempt_id = attempt_id
            .checked_add(1)
            .ok_or(StartupRecoveryError::GenerationExhausted)?;
        let boundary = failure.boundary;
        let correlation_id = failure.correlation_id.clone();
        self.generation = generation;
        self.next_attempt_id = next_attempt_id;
        self.lifecycle = match input.action {
            StartupRecoveryAction::Retry | StartupRecoveryAction::RestoreValidatedBackup => {
                StartupRecoveryLifecycle::Retrying
            }
            StartupRecoveryAction::OpenRecoveryLocation => StartupRecoveryLifecycle::Degraded,
        };
        self.pending_action = Some(PendingAction {
            attempt_id,
            generation,
            boundary,
            action: input.action,
            started_at_ms: input.requested_at_ms,
        });
        self.push_history(StartupRecoveryHistorySnapshot {
            generation,
            boundary,
            lifecycle: self.lifecycle,
            kind: StartupRecoveryHistoryKind::ActionStarted,
            action: Some(input.action),
            correlation_id,
            diagnostic_code: "recovery-action-started".into(),
            message: safe_action_started_message(input.action).into(),
            occurred_at_ms: input.requested_at_ms,
        });
        Ok(StartupRecoveryPreparedAction {
            attempt_id,
            generation,
            boundary,
            action: input.action,
        })
    }

    pub fn complete_action(
        &mut self,
        prepared: &StartupRecoveryPreparedAction,
        outcome: StartupRecoveryActionOutcome,
    ) -> Result<StartupRecoverySnapshot, StartupRecoveryError> {
        validate_outcome(&outcome)?;
        let pending = self
            .pending_action
            .as_ref()
            .ok_or(StartupRecoveryError::InvalidState)?;
        if pending.attempt_id != prepared.attempt_id
            || pending.generation != prepared.generation
            || pending.boundary != prepared.boundary
            || pending.action != prepared.action
            || self.generation != prepared.generation
        {
            return Err(StartupRecoveryError::AttemptMismatch);
        }
        let generation = self.next_generation()?;
        let failure = self
            .failure
            .clone()
            .ok_or(StartupRecoveryError::InvalidState)?;
        let action = pending.action;
        let boundary = pending.boundary;
        self.generation = generation;
        self.pending_action = None;
        match outcome {
            StartupRecoveryActionOutcome::Succeeded { completed_at_ms } => match action {
                StartupRecoveryAction::Retry | StartupRecoveryAction::RestoreValidatedBackup => {
                    self.lifecycle = StartupRecoveryLifecycle::Recovered;
                    self.failure = None;
                    self.push_history(StartupRecoveryHistorySnapshot {
                        generation,
                        boundary,
                        lifecycle: self.lifecycle,
                        kind: StartupRecoveryHistoryKind::ActionSucceeded,
                        action: Some(action),
                        correlation_id: failure.correlation_id,
                        diagnostic_code: "startup-recovered".into(),
                        message: "Startup recovery completed successfully.".into(),
                        occurred_at_ms: completed_at_ms,
                    });
                }
                StartupRecoveryAction::OpenRecoveryLocation => {
                    self.lifecycle = StartupRecoveryLifecycle::Degraded;
                    self.push_history(StartupRecoveryHistorySnapshot {
                        generation,
                        boundary,
                        lifecycle: self.lifecycle,
                        kind: StartupRecoveryHistoryKind::RecoveryLocationOpened,
                        action: Some(action),
                        correlation_id: failure.correlation_id,
                        diagnostic_code: "recovery-location-opened".into(),
                        message: "The native recovery location was opened.".into(),
                        occurred_at_ms: completed_at_ms,
                    });
                }
            },
            StartupRecoveryActionOutcome::Failed {
                diagnostic_code,
                message,
                completed_at_ms,
            } => {
                self.lifecycle = StartupRecoveryLifecycle::Degraded;
                let replacement = StartupRecoveryFailureSnapshot {
                    boundary,
                    correlation_id: failure.correlation_id.clone(),
                    diagnostic_code: diagnostic_code.clone(),
                    message: message.clone(),
                    failed_at_ms: completed_at_ms,
                    validated_backup_available: failure.validated_backup_available,
                    recovery_location_available: failure.recovery_location_available,
                };
                self.failure = Some(replacement);
                self.push_history(StartupRecoveryHistorySnapshot {
                    generation,
                    boundary,
                    lifecycle: self.lifecycle,
                    kind: StartupRecoveryHistoryKind::ActionFailed,
                    action: Some(action),
                    correlation_id: failure.correlation_id,
                    diagnostic_code,
                    message,
                    occurred_at_ms: completed_at_ms,
                });
            }
        }
        Ok(self.snapshot())
    }

    fn available_actions(&self) -> Vec<StartupRecoveryAction> {
        let Some(failure) = self.failure.as_ref() else {
            return Vec::new();
        };
        if self.lifecycle != StartupRecoveryLifecycle::Degraded || self.pending_action.is_some() {
            return Vec::new();
        }
        let mut actions = vec![StartupRecoveryAction::Retry];
        if failure.validated_backup_available {
            actions.push(StartupRecoveryAction::RestoreValidatedBackup);
        }
        if failure.recovery_location_available {
            actions.push(StartupRecoveryAction::OpenRecoveryLocation);
        }
        actions
    }

    fn require_generation(&self, expected_generation: u64) -> Result<(), StartupRecoveryError> {
        if expected_generation != self.generation {
            return Err(StartupRecoveryError::StaleGeneration {
                expected_generation,
                actual_generation: self.generation,
            });
        }
        Ok(())
    }

    fn next_generation(&self) -> Result<u64, StartupRecoveryError> {
        self.generation
            .checked_add(1)
            .ok_or(StartupRecoveryError::GenerationExhausted)
    }

    fn push_history(&mut self, record: StartupRecoveryHistorySnapshot) {
        if self.history.len() == MAX_STARTUP_RECOVERY_HISTORY {
            self.history.remove(0);
            self.history_truncated = self.history_truncated.saturating_add(1);
        }
        self.history.push(record);
    }
}

fn action_available(
    failure: &StartupRecoveryFailureSnapshot,
    action: StartupRecoveryAction,
) -> bool {
    match action {
        StartupRecoveryAction::Retry => true,
        StartupRecoveryAction::RestoreValidatedBackup => failure.validated_backup_available,
        StartupRecoveryAction::OpenRecoveryLocation => failure.recovery_location_available,
    }
}

fn validate_failure_input(input: &StartupRecoveryFailureInput) -> Result<(), StartupRecoveryError> {
    validate_identifier(&input.correlation_id)?;
    validate_identifier(&input.diagnostic_code)?;
    validate_message(&input.message)?;
    validate_timestamp(input.failed_at_ms)
}

fn validate_outcome(outcome: &StartupRecoveryActionOutcome) -> Result<(), StartupRecoveryError> {
    match outcome {
        StartupRecoveryActionOutcome::Succeeded { completed_at_ms } => {
            validate_timestamp(*completed_at_ms)
        }
        StartupRecoveryActionOutcome::Failed {
            diagnostic_code,
            message,
            completed_at_ms,
        } => {
            validate_identifier(diagnostic_code)?;
            validate_message(message)?;
            validate_timestamp(*completed_at_ms)
        }
    }
}

fn validate_identifier(value: &str) -> Result<(), StartupRecoveryError> {
    if value.is_empty()
        || value.len() > MAX_STARTUP_RECOVERY_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
    {
        return Err(StartupRecoveryError::InvalidInput);
    }
    Ok(())
}

fn validate_message(value: &str) -> Result<(), StartupRecoveryError> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > MAX_STARTUP_RECOVERY_MESSAGE_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(StartupRecoveryError::InvalidInput);
    }
    let lower = value.to_ascii_lowercase();
    const FORBIDDEN_MARKERS: &[&str] = &[
        "authorization:",
        "bearer ",
        "basic ",
        "cookie=",
        "set-cookie:",
        "password=",
        "passwd=",
        "secret=",
        "token=",
        "api_key=",
        "apikey=",
        "x-api-key",
        "file://",
        "~/.",
        "/users/",
        "/private/",
        "/var/",
        "/tmp/",
        "\\users\\",
    ];
    if FORBIDDEN_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(StartupRecoveryError::InvalidInput);
    }
    Ok(())
}

fn validate_timestamp(value: u64) -> Result<(), StartupRecoveryError> {
    if value == 0 {
        return Err(StartupRecoveryError::InvalidInput);
    }
    Ok(())
}

fn safe_action_started_message(action: StartupRecoveryAction) -> &'static str {
    match action {
        StartupRecoveryAction::Retry => "Retrying the failed startup boundary.",
        StartupRecoveryAction::RestoreValidatedBackup => "Restoring the validated startup backup.",
        StartupRecoveryAction::OpenRecoveryLocation => "Opening the native recovery location.",
    }
}
