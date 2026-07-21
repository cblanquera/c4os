//! Chat-scoped, authorized pseudo-terminal supervision.
//!
//! This module deliberately owns no policy decision, renderer protocol, or
//! durable artifact schema. The coordinator calls the `*_authorized` methods
//! only from inside a consumed Action Gateway effect. Constructing or
//! inspecting a supervisor never spawns a process or writes PTY bytes.

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use thiserror::Error;
use uuid::Uuid;

use super::environment::{
    ExecutionEnvironmentIdentity, ExecutionEnvironmentKind, TrustedProjectRoot,
};

pub const MAX_TERMINAL_IDENTIFIER_BYTES: usize = 160;
pub const MAX_TERMINAL_COMMAND_BYTES: usize = 64 * 1_024;
pub const MAX_TERMINAL_STDIN_BYTES: usize = 64 * 1_024;
pub const MAX_TERMINAL_EVENT_OUTPUT_BYTES: usize = 8 * 1_024;
pub const MAX_TERMINAL_DRAIN_EVENTS: usize = 256;
pub const MAX_TERMINAL_DRAIN_BYTES: usize = 1024 * 1024;
pub const MAX_TERMINAL_DIMENSION: u16 = 1_000;
pub const MAX_TERMINAL_RETAINED_SESSIONS: usize = 256;

const READER_FRAME_BYTES: usize = 8 * 1_024;
const DEFAULT_READER_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_READER_FRAMES_PER_DRAIN: usize = 256;
const DEFAULT_PENDING_EVENT_BYTES: usize = 512 * 1_024;
const DEFAULT_PENDING_EVENTS: usize = 1_024;
const DEFAULT_RETAINED_SESSIONS: usize = 8;
const MAX_SENTINEL_BYTES: usize = 32 * 1_024;
const DEFAULT_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TerminalSessionKey {
    pub workspace_id: String,
    pub chat_id: String,
}

impl TerminalSessionKey {
    pub fn new(
        workspace_id: impl Into<String>,
        chat_id: impl Into<String>,
    ) -> Result<Self, TerminalError> {
        let key = Self {
            workspace_id: workspace_id.into(),
            chat_id: chat_id.into(),
        };
        key.validate()?;
        Ok(key)
    }

    fn validate(&self) -> Result<(), TerminalError> {
        validate_identifier("Terminal Workspace id", &self.workspace_id)?;
        validate_identifier("Terminal Chat id", &self.chat_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalCommandIdentity {
    pub terminal_session_id: String,
    pub command_id: String,
    pub command_sequence: u64,
}

impl TerminalCommandIdentity {
    pub fn new(
        terminal_session_id: impl Into<String>,
        command_id: impl Into<String>,
        command_sequence: u64,
    ) -> Result<Self, TerminalError> {
        let identity = Self {
            terminal_session_id: terminal_session_id.into(),
            command_id: command_id.into(),
            command_sequence,
        };
        identity.validate()?;
        Ok(identity)
    }

    fn validate(&self) -> Result<(), TerminalError> {
        validate_identifier("Terminal session id", &self.terminal_session_id)?;
        validate_identifier("Terminal command id", &self.command_id)?;
        if self.command_sequence == 0 {
            return Err(TerminalError::InvalidRequest(
                "Terminal command sequence must be non-zero",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalDimensions {
    pub cols: u16,
    pub rows: u16,
}

impl TerminalDimensions {
    pub fn new(cols: u16, rows: u16) -> Result<Self, TerminalError> {
        let dimensions = Self { cols, rows };
        dimensions.validate()?;
        Ok(dimensions)
    }

    fn validate(self) -> Result<(), TerminalError> {
        if self.cols == 0
            || self.rows == 0
            || self.cols > MAX_TERMINAL_DIMENSION
            || self.rows > MAX_TERMINAL_DIMENSION
        {
            return Err(TerminalError::InvalidRequest(
                "Terminal dimensions are outside the supported bound",
            ));
        }
        Ok(())
    }

    fn pty_size(self) -> PtySize {
        PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TerminalExecuteRequest {
    pub key: TerminalSessionKey,
    pub command: TerminalCommandIdentity,
    pub process_generation: u64,
    pub environment: ExecutionEnvironmentIdentity,
    pub dimensions: TerminalDimensions,
    pub trusted_project_root: TrustedProjectRoot,
    pub shell_path: PathBuf,
    pub command_line: String,
}

impl TerminalExecuteRequest {
    fn validate(&self) -> Result<PathBuf, TerminalError> {
        self.key.validate()?;
        self.command.validate()?;
        validate_environment(&self.environment)?;
        self.dimensions.validate()?;
        self.trusted_project_root
            .revalidate()
            .map_err(|_| TerminalError::StaleBinding("The physical Project root changed".into()))?;
        if self.process_generation == 0 {
            return Err(TerminalError::InvalidRequest(
                "Terminal process generation must be non-zero",
            ));
        }
        validate_command_line(&self.command_line)?;
        validate_shell_path(&self.shell_path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalStdinRequest {
    pub key: TerminalSessionKey,
    pub command: TerminalCommandIdentity,
    pub process_generation: u64,
    /// Exact bytes already authorized by the coordinator. A caller that wants
    /// line submission includes `\n`; this module never fabricates it.
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalResizeRequest {
    pub key: TerminalSessionKey,
    pub terminal_session_id: String,
    pub process_generation: u64,
    pub dimensions: TerminalDimensions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalStopRequest {
    pub key: TerminalSessionKey,
    pub command: TerminalCommandIdentity,
    pub process_generation: u64,
}

#[derive(Clone, Debug)]
pub struct TerminalRestartRecord {
    pub key: TerminalSessionKey,
    pub command: TerminalCommandIdentity,
    pub process_generation: u64,
    pub environment: ExecutionEnvironmentIdentity,
    pub dimensions: TerminalDimensions,
    pub trusted_project_root: TrustedProjectRoot,
    pub shell_path: PathBuf,
    pub working_directory: PathBuf,
    /// Deliberately informational. Recovery emits that this value was ignored
    /// and never probes or signals it.
    pub persisted_process_id: Option<u32>,
}

/// Durable idle-shell state restored after application restart. Unlike a
/// running-command recovery record, this never emits an interruption because
/// the referenced command already reached a terminal state before shutdown.
#[derive(Clone, Debug)]
pub struct TerminalCompletedSessionRecord {
    pub key: TerminalSessionKey,
    pub command: TerminalCommandIdentity,
    pub process_generation: u64,
    pub environment: ExecutionEnvironmentIdentity,
    pub dimensions: TerminalDimensions,
    pub trusted_project_root: TrustedProjectRoot,
    pub shell_path: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalDrainRequest {
    pub key: TerminalSessionKey,
    pub terminal_session_id: String,
    pub process_generation: u64,
    pub after_chunk_sequence: u64,
    pub maximum_events: usize,
    pub maximum_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalAcknowledgeRequest {
    pub key: TerminalSessionKey,
    pub terminal_session_id: String,
    pub process_generation: u64,
    pub through_chunk_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalLifecycle {
    Live,
    DormantAfterRecovery,
    DormantAfterRestart,
    DormantAfterReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalLiveSessionSnapshot {
    pub key: TerminalSessionKey,
    pub terminal_session_id: String,
    pub lifecycle: TerminalLifecycle,
    pub process_generation: u64,
    pub process_id: Option<u32>,
    pub foreground_process_group_id: Option<i32>,
    pub active_command: Option<TerminalCommandIdentity>,
    pub next_command_sequence: u64,
    pub environment: ExecutionEnvironmentIdentity,
    pub dimensions: TerminalDimensions,
    pub shell_path: PathBuf,
    pub working_directory: PathBuf,
    pub pending_event_count: usize,
    pub pending_output_bytes: usize,
    pub output_bytes_dropped: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalEvent {
    pub key: TerminalSessionKey,
    pub terminal_session_id: String,
    pub process_generation: u64,
    pub command_id: String,
    pub command_sequence: u64,
    pub chunk_sequence: u64,
    pub kind: TerminalEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminalEventKind {
    Started {
        process_id: u32,
        foreground_process_group_id: i32,
        working_directory: PathBuf,
    },
    Output {
        bytes: Vec<u8>,
        dropped_bytes_before: u64,
    },
    Completed {
        exit_code: i32,
        working_directory: PathBuf,
    },
    Interrupted130 {
        working_directory: PathBuf,
        shell_replaced: bool,
    },
    RecoveredInterrupted {
        persisted_process_id_ignored: bool,
        working_directory: PathBuf,
    },
    Failed {
        code: String,
        message: String,
        shell_replaced: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalStopDisposition {
    pub foreground_process_group_id: i32,
    pub used_pty_etx: bool,
}

#[derive(Clone, Debug)]
pub struct TerminalSupervisorConfiguration {
    /// Hard process/resource bound across live shells and dormant recovery
    /// records. Existing Chat keys may still reuse or replace their slot.
    pub maximum_retained_sessions: usize,
    pub reader_channel_capacity: usize,
    pub reader_frames_per_drain: usize,
    pub maximum_pending_events: usize,
    pub maximum_pending_output_bytes: usize,
    pub stop_timeout: Duration,
    pub shutdown_timeout: Duration,
}

impl Default for TerminalSupervisorConfiguration {
    fn default() -> Self {
        Self {
            maximum_retained_sessions: DEFAULT_RETAINED_SESSIONS,
            reader_channel_capacity: DEFAULT_READER_CHANNEL_CAPACITY,
            reader_frames_per_drain: DEFAULT_READER_FRAMES_PER_DRAIN,
            maximum_pending_events: DEFAULT_PENDING_EVENTS,
            maximum_pending_output_bytes: DEFAULT_PENDING_EVENT_BYTES,
            stop_timeout: DEFAULT_STOP_TIMEOUT,
            shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
        }
    }
}

impl TerminalSupervisorConfiguration {
    fn validate(&self) -> Result<(), TerminalError> {
        if self.maximum_retained_sessions == 0
            || self.maximum_retained_sessions > MAX_TERMINAL_RETAINED_SESSIONS
            || self.reader_channel_capacity == 0
            || self.reader_channel_capacity > 4_096
            || self.reader_frames_per_drain == 0
            || self.reader_frames_per_drain > 4_096
            || self.maximum_pending_events < 8
            || self.maximum_pending_events > 65_536
            || self.maximum_pending_output_bytes < MAX_TERMINAL_EVENT_OUTPUT_BYTES
            || self.maximum_pending_output_bytes > 64 * 1_024 * 1_024
            || self.stop_timeout.is_zero()
            || self.stop_timeout > Duration::from_secs(30)
            || self.shutdown_timeout.is_zero()
            || self.shutdown_timeout > Duration::from_secs(30)
        {
            return Err(TerminalError::InvalidRequest(
                "Terminal supervisor configuration is invalid",
            ));
        }
        Ok(())
    }
}

pub struct TerminalSupervisor {
    configuration: TerminalSupervisorConfiguration,
    sessions: HashMap<TerminalSessionKey, TerminalSlot>,
}

impl TerminalSupervisor {
    pub fn new() -> Self {
        Self::with_configuration(TerminalSupervisorConfiguration::default())
            .expect("the default Terminal supervisor configuration is valid")
    }

    pub fn with_configuration(
        configuration: TerminalSupervisorConfiguration,
    ) -> Result<Self, TerminalError> {
        configuration.validate()?;
        Ok(Self {
            configuration,
            sessions: HashMap::new(),
        })
    }

    pub fn live_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|slot| matches!(slot, TerminalSlot::Live(_)))
            .count()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn session_snapshots(&self) -> Vec<TerminalLiveSessionSnapshot> {
        let mut snapshots = self
            .sessions
            .iter()
            .map(|(key, slot)| match slot {
                TerminalSlot::Live(live) => snapshot_live(key, live),
                TerminalSlot::Dormant(dormant) => snapshot_dormant(key, dormant),
            })
            .collect::<Vec<_>>();
        snapshots.sort_by(|left, right| {
            left.key
                .workspace_id
                .cmp(&right.key.workspace_id)
                .then_with(|| left.key.chat_id.cmp(&right.key.chat_id))
        });
        snapshots
    }

    /// Lazily spawns or reuses the one shell bound to the request Chat. The
    /// method name is an authority boundary: callers must hold a consumed
    /// gateway permit before entering it.
    pub fn execute_authorized(
        &mut self,
        request: TerminalExecuteRequest,
    ) -> Result<TerminalLiveSessionSnapshot, TerminalError> {
        let canonical_shell = request.validate()?;
        let key = request.key.clone();
        self.require_capacity_for_key(&key)?;

        if let Some(TerminalSlot::Live(live)) = self.sessions.get_mut(&key) {
            validate_live_execution(live, &request, &canonical_shell)?;
            write_command(live, &request.command, &request.command_line)?;
            return Ok(snapshot_live(&key, live));
        }

        match self.sessions.get(&key) {
            Some(TerminalSlot::Dormant(dormant)) => {
                // Validate before removal so a stale request cannot destroy the
                // retained recovery record or its reconnectable event ledger.
                validate_dormant_execution(dormant, &request, &canonical_shell)?;
            }
            Some(TerminalSlot::Live(_)) => unreachable!("live session was handled above"),
            None => {
                if request.process_generation != 1 || request.command.command_sequence != 1 {
                    return Err(TerminalError::StaleGeneration);
                }
            }
        }
        let prior = self.sessions.remove(&key);
        let ledger = match &prior {
            Some(TerminalSlot::Dormant(dormant)) => dormant.ledger.clone(),
            Some(TerminalSlot::Live(_)) => unreachable!("live session was handled above"),
            None => EventLedger::new(&self.configuration),
        };
        let restart_cwd = match &prior {
            Some(TerminalSlot::Dormant(dormant)) => Some(dormant.working_directory.clone()),
            _ => None,
        };

        match spawn_live_session(
            &request,
            canonical_shell,
            restart_cwd,
            &self.configuration,
            &ledger,
        ) {
            Ok(live) => {
                let snapshot = snapshot_live(&key, &live);
                self.sessions
                    .insert(key, TerminalSlot::Live(Box::new(live)));
                Ok(snapshot)
            }
            Err(error) => {
                if let Some(TerminalSlot::Dormant(mut dormant)) = prior {
                    dormant.ledger = ledger;
                    self.sessions.insert(key, TerminalSlot::Dormant(dormant));
                }
                Err(error)
            }
        }
    }

    pub fn submit_stdin_authorized(
        &mut self,
        request: TerminalStdinRequest,
    ) -> Result<(), TerminalError> {
        request.key.validate()?;
        request.command.validate()?;
        if request.bytes.is_empty() || request.bytes.len() > MAX_TERMINAL_STDIN_BYTES {
            return Err(TerminalError::InvalidRequest(
                "Terminal stdin is empty or exceeds its byte bound",
            ));
        }
        let live = self.require_live_mut(
            &request.key,
            &request.command.terminal_session_id,
            request.process_generation,
        )?;
        if live
            .active_command
            .as_ref()
            .map(|command| &command.identity)
            != Some(&request.command)
        {
            return Err(TerminalError::NoActiveCommand);
        }
        if !live
            .active_command
            .as_ref()
            .is_some_and(|command| command.started)
        {
            return Err(TerminalError::CommandNotReady);
        }
        // Interactive shells restore their preferred termios during startup,
        // after the PTY is opened. Reassert and verify echo-off immediately at
        // the authorized write boundary so stdin (including secrets) cannot be
        // reflected into the durable Output stream.
        disable_pty_echo(live.master.as_ref())?;
        live.writer
            .write_all(&request.bytes)
            .and_then(|_| live.writer.flush())
            .map_err(TerminalError::PtyIo)
    }

    pub fn resize_authorized(
        &mut self,
        request: TerminalResizeRequest,
    ) -> Result<(), TerminalError> {
        request.key.validate()?;
        validate_identifier("Terminal session id", &request.terminal_session_id)?;
        request.dimensions.validate()?;
        let live = self.require_live_mut(
            &request.key,
            &request.terminal_session_id,
            request.process_generation,
        )?;
        live.master
            .resize(request.dimensions.pty_size())
            .map_err(|error| TerminalError::Pty(error.to_string()))?;
        live.dimensions = request.dimensions;
        Ok(())
    }

    pub fn stop_authorized(
        &mut self,
        request: TerminalStopRequest,
    ) -> Result<TerminalStopDisposition, TerminalError> {
        request.key.validate()?;
        request.command.validate()?;
        let stop_timeout = self.configuration.stop_timeout;
        let live = self.require_live_mut(
            &request.key,
            &request.command.terminal_session_id,
            request.process_generation,
        )?;
        let active = live
            .active_command
            .as_mut()
            .filter(|active| active.identity == request.command)
            .ok_or(TerminalError::NoActiveCommand)?;
        if active.stop_deadline.is_some() {
            return Err(TerminalError::AlreadyStopping);
        }
        let foreground = live
            .master
            .process_group_leader()
            .filter(|process_group| *process_group > 0)
            .ok_or(TerminalError::ForegroundProcessUnavailable)?;
        let shell_pid = i32::try_from(live.process_id)
            .map_err(|_| TerminalError::ForegroundProcessUnavailable)?;
        let used_pty_etx = foreground == shell_pid;
        if used_pty_etx {
            live.writer
                .write_all(&[0x03])
                .and_then(|_| live.writer.flush())
                .map_err(TerminalError::PtyIo)?;
        } else {
            signal_process_group(foreground, libc::SIGINT)?;
        }
        active.stop_deadline = Some(Instant::now() + stop_timeout);
        Ok(TerminalStopDisposition {
            foreground_process_group_id: foreground,
            used_pty_etx,
        })
    }

    /// Drains bounded reader work into a reconnectable event ledger, then
    /// returns events newer than the caller's cursor. The reader channel is
    /// bounded and blocking, so an unresponsive coordinator applies PTY
    /// backpressure instead of allocating unbounded memory.
    pub fn drain_events(
        &mut self,
        request: TerminalDrainRequest,
    ) -> Result<Vec<TerminalEvent>, TerminalError> {
        validate_drain_request(&request)?;
        let replacement = match self.sessions.get_mut(&request.key) {
            Some(TerminalSlot::Live(live)) => {
                validate_session_binding(
                    &live.terminal_session_id,
                    live.process_generation,
                    &request.terminal_session_id,
                    request.process_generation,
                )?;
                pump_reader(live, &self.configuration)?
            }
            Some(TerminalSlot::Dormant(dormant)) => {
                validate_session_binding(
                    &dormant.terminal_session_id,
                    dormant.process_generation,
                    &request.terminal_session_id,
                    request.process_generation,
                )?;
                None
            }
            None => return Err(TerminalError::SessionNotFound),
        };
        if let Some(reason) = replacement {
            self.replace_live_with_dormant(&request.key, reason)?;
        }

        let ledger = match self.sessions.get(&request.key) {
            Some(TerminalSlot::Live(live)) => &live.ledger,
            Some(TerminalSlot::Dormant(dormant)) => &dormant.ledger,
            None => return Err(TerminalError::SessionNotFound),
        };
        Ok(ledger.drain_after(
            request.process_generation,
            request.after_chunk_sequence,
            request.maximum_events,
            request.maximum_bytes,
        ))
    }

    pub fn acknowledge_output(
        &mut self,
        request: TerminalAcknowledgeRequest,
    ) -> Result<(), TerminalError> {
        request.key.validate()?;
        validate_identifier("Terminal session id", &request.terminal_session_id)?;
        let ledger = match self.sessions.get_mut(&request.key) {
            Some(TerminalSlot::Live(live)) => {
                validate_session_binding(
                    &live.terminal_session_id,
                    live.process_generation,
                    &request.terminal_session_id,
                    request.process_generation,
                )?;
                &mut live.ledger
            }
            Some(TerminalSlot::Dormant(dormant)) => {
                validate_session_binding(
                    &dormant.terminal_session_id,
                    dormant.process_generation,
                    &request.terminal_session_id,
                    request.process_generation,
                )?;
                &mut dormant.ledger
            }
            None => return Err(TerminalError::SessionNotFound),
        };
        ledger.acknowledge(request.through_chunk_sequence);
        Ok(())
    }

    pub fn live_session(&self, key: &TerminalSessionKey) -> Option<TerminalLiveSessionSnapshot> {
        match self.sessions.get(key) {
            Some(TerminalSlot::Live(live)) => Some(snapshot_live(key, live)),
            Some(TerminalSlot::Dormant(dormant)) => Some(snapshot_dormant(key, dormant)),
            None => None,
        }
    }

    /// Reconciles a durable running command after process restart. The
    /// persisted PID is never read as process authority, probed, or signalled.
    pub fn reconcile_restart(
        &mut self,
        record: TerminalRestartRecord,
    ) -> Result<TerminalLiveSessionSnapshot, TerminalError> {
        validate_restart_record(&record)?;
        let next_command_sequence = record
            .command
            .command_sequence
            .checked_add(1)
            .ok_or(TerminalError::GenerationExhausted)?;
        if self.sessions.contains_key(&record.key) {
            return Err(TerminalError::SessionAlreadyExists);
        }
        self.require_capacity_for_key(&record.key)?;
        let shell_path = validate_shell_path(&record.shell_path)?;
        let working_directory = retained_restart_directory(
            &record.trusted_project_root,
            Some(&record.working_directory),
        );
        let mut ledger = EventLedger::new(&self.configuration);
        ledger.push(
            &record.key,
            &record.command,
            record.process_generation,
            TerminalEventKind::RecoveredInterrupted {
                persisted_process_id_ignored: record.persisted_process_id.is_some(),
                working_directory: working_directory.clone(),
            },
        );
        let dormant = DormantSession {
            terminal_session_id: record.command.terminal_session_id.clone(),
            process_generation: record.process_generation,
            next_command_sequence,
            environment: record.environment,
            dimensions: record.dimensions,
            trusted_project_root: record.trusted_project_root,
            shell_path,
            working_directory,
            lifecycle: TerminalLifecycle::DormantAfterRecovery,
            ledger,
        };
        let snapshot = snapshot_dormant(&record.key, &dormant);
        self.sessions
            .insert(record.key, TerminalSlot::Dormant(Box::new(dormant)));
        Ok(snapshot)
    }

    /// Restores the retained identity and safe working directory of a shell
    /// whose last durable command had already completed. No persisted PID is
    /// trusted and no synthetic recovery event is created.
    pub fn restore_completed_session(
        &mut self,
        record: TerminalCompletedSessionRecord,
    ) -> Result<TerminalLiveSessionSnapshot, TerminalError> {
        validate_completed_session_record(&record)?;
        let next_command_sequence = record
            .command
            .command_sequence
            .checked_add(1)
            .ok_or(TerminalError::GenerationExhausted)?;
        if self.sessions.contains_key(&record.key) {
            return Err(TerminalError::SessionAlreadyExists);
        }
        self.require_capacity_for_key(&record.key)?;
        let shell_path = validate_shell_path(&record.shell_path)?;
        let working_directory = retained_restart_directory(
            &record.trusted_project_root,
            Some(&record.working_directory),
        );
        let dormant = DormantSession {
            terminal_session_id: record.command.terminal_session_id,
            process_generation: record.process_generation,
            next_command_sequence,
            environment: record.environment,
            dimensions: record.dimensions,
            trusted_project_root: record.trusted_project_root,
            shell_path,
            working_directory,
            lifecycle: TerminalLifecycle::DormantAfterRestart,
            ledger: EventLedger::new(&self.configuration),
        };
        let snapshot = snapshot_dormant(&record.key, &dormant);
        self.sessions
            .insert(record.key, TerminalSlot::Dormant(Box::new(dormant)));
        Ok(snapshot)
    }

    pub fn shutdown_session(&mut self, key: &TerminalSessionKey) -> Result<bool, TerminalError> {
        key.validate()?;
        let Some(slot) = self.sessions.remove(key) else {
            return Ok(false);
        };
        if let TerminalSlot::Live(live) = slot {
            terminate_live_session(*live, self.configuration.shutdown_timeout)?;
        }
        Ok(true)
    }

    pub fn shutdown_workspace(&mut self, workspace_id: &str) -> Result<usize, TerminalError> {
        validate_identifier("Terminal Workspace id", workspace_id)?;
        let keys = self
            .sessions
            .keys()
            .filter(|key| key.workspace_id == workspace_id)
            .cloned()
            .collect::<Vec<_>>();
        for key in &keys {
            self.shutdown_session(key)?;
        }
        Ok(keys.len())
    }

    pub fn shutdown_all(&mut self) -> Result<usize, TerminalError> {
        let keys = self.sessions.keys().cloned().collect::<Vec<_>>();
        let mut first_error = None;
        for key in &keys {
            if let Err(error) = self.shutdown_session(key)
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        }
        first_error.map_or(Ok(keys.len()), Err)
    }

    fn require_live_mut(
        &mut self,
        key: &TerminalSessionKey,
        terminal_session_id: &str,
        process_generation: u64,
    ) -> Result<&mut LiveSession, TerminalError> {
        let live = match self.sessions.get_mut(key) {
            Some(TerminalSlot::Live(live)) => live,
            Some(TerminalSlot::Dormant(_)) => return Err(TerminalError::ShellUnavailable),
            None => return Err(TerminalError::SessionNotFound),
        };
        validate_session_binding(
            &live.terminal_session_id,
            live.process_generation,
            terminal_session_id,
            process_generation,
        )?;
        Ok(live)
    }

    fn require_capacity_for_key(&self, key: &TerminalSessionKey) -> Result<(), TerminalError> {
        if !self.sessions.contains_key(key)
            && self.sessions.len() >= self.configuration.maximum_retained_sessions
        {
            return Err(TerminalError::SessionCapacityExceeded {
                maximum: self.configuration.maximum_retained_sessions,
            });
        }
        Ok(())
    }

    fn replace_live_with_dormant(
        &mut self,
        key: &TerminalSessionKey,
        reason: ReplacementReason,
    ) -> Result<(), TerminalError> {
        let slot = self
            .sessions
            .remove(key)
            .ok_or(TerminalError::SessionNotFound)?;
        let TerminalSlot::Live(mut live) = slot else {
            self.sessions.insert(key.clone(), slot);
            return Ok(());
        };
        let command = live
            .active_command
            .take()
            .map(|active| active.identity)
            .unwrap_or_else(|| live.last_command.clone());
        match reason {
            ReplacementReason::StopTimeout => live.ledger.push(
                key,
                &command,
                live.process_generation,
                TerminalEventKind::Interrupted130 {
                    working_directory: live.working_directory.clone(),
                    shell_replaced: true,
                },
            ),
            ReplacementReason::ReaderFailure(message) => live.ledger.push(
                key,
                &command,
                live.process_generation,
                TerminalEventKind::Failed {
                    code: "terminal-shell-closed".into(),
                    message,
                    shell_replaced: true,
                },
            ),
            ReplacementReason::InvalidSentinel(message) => live.ledger.push(
                key,
                &command,
                live.process_generation,
                TerminalEventKind::Failed {
                    code: "terminal-sentinel-invalid".into(),
                    message,
                    shell_replaced: true,
                },
            ),
        }
        let dormant = DormantSession {
            terminal_session_id: live.terminal_session_id.clone(),
            process_generation: live.process_generation,
            next_command_sequence: live.next_command_sequence,
            environment: live.environment.clone(),
            dimensions: live.dimensions,
            trusted_project_root: live.trusted_project_root.clone(),
            shell_path: live.shell_path.clone(),
            working_directory: live.working_directory.clone(),
            lifecycle: TerminalLifecycle::DormantAfterReplacement,
            ledger: live.ledger.clone(),
        };
        let termination = terminate_live_session(*live, self.configuration.shutdown_timeout);
        self.sessions
            .insert(key.clone(), TerminalSlot::Dormant(Box::new(dormant)));
        termination
    }
}

impl Default for TerminalSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalSupervisor {
    fn drop(&mut self) {
        let keys = self.sessions.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            if let Some(TerminalSlot::Live(live)) = self.sessions.remove(&key) {
                let _ = terminate_live_session(*live, Duration::from_millis(250));
            }
        }
    }
}

enum TerminalSlot {
    Live(Box<LiveSession>),
    Dormant(Box<DormantSession>),
}

struct LiveSession {
    key: TerminalSessionKey,
    terminal_session_id: String,
    process_generation: u64,
    process_id: u32,
    environment: ExecutionEnvironmentIdentity,
    dimensions: TerminalDimensions,
    trusted_project_root: TrustedProjectRoot,
    shell_path: PathBuf,
    working_directory: PathBuf,
    next_command_sequence: u64,
    last_command: TerminalCommandIdentity,
    active_command: Option<ActiveCommand>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    reader_rx: Option<Receiver<ReaderFrame>>,
    reader_thread: Option<JoinHandle<()>>,
    reader_closed: bool,
    ledger: EventLedger,
}

struct DormantSession {
    terminal_session_id: String,
    process_generation: u64,
    next_command_sequence: u64,
    environment: ExecutionEnvironmentIdentity,
    dimensions: TerminalDimensions,
    trusted_project_root: TrustedProjectRoot,
    shell_path: PathBuf,
    working_directory: PathBuf,
    lifecycle: TerminalLifecycle,
    ledger: EventLedger,
}

struct ActiveCommand {
    identity: TerminalCommandIdentity,
    parser: SentinelParser,
    started: bool,
    stop_deadline: Option<Instant>,
}

#[derive(Clone)]
struct EventLedger {
    events: VecDeque<TerminalEvent>,
    next_chunk_sequence: u64,
    pending_output_bytes: usize,
    output_bytes_dropped: u64,
    maximum_events: usize,
    maximum_output_bytes: usize,
}

impl EventLedger {
    fn new(configuration: &TerminalSupervisorConfiguration) -> Self {
        Self {
            events: VecDeque::new(),
            next_chunk_sequence: 1,
            pending_output_bytes: 0,
            output_bytes_dropped: 0,
            maximum_events: configuration.maximum_pending_events,
            maximum_output_bytes: configuration.maximum_pending_output_bytes,
        }
    }

    fn push(
        &mut self,
        key: &TerminalSessionKey,
        command: &TerminalCommandIdentity,
        process_generation: u64,
        mut kind: TerminalEventKind,
    ) {
        let output_length = output_length(&kind);
        if output_length > 0 {
            self.make_output_room(output_length);
            if let TerminalEventKind::Output {
                dropped_bytes_before,
                ..
            } = &mut kind
            {
                *dropped_bytes_before =
                    dropped_bytes_before.saturating_add(self.output_bytes_dropped);
                self.output_bytes_dropped = 0;
            }
        }
        while self.events.len() >= self.maximum_events {
            self.drop_oldest_event();
        }
        let chunk_sequence = self.next_chunk_sequence;
        self.next_chunk_sequence = self.next_chunk_sequence.saturating_add(1);
        self.pending_output_bytes = self.pending_output_bytes.saturating_add(output_length);
        self.events.push_back(TerminalEvent {
            key: key.clone(),
            terminal_session_id: command.terminal_session_id.clone(),
            process_generation,
            command_id: command.command_id.clone(),
            command_sequence: command.command_sequence,
            chunk_sequence,
            kind,
        });
    }

    fn make_output_room(&mut self, needed: usize) {
        while self.pending_output_bytes.saturating_add(needed) > self.maximum_output_bytes {
            let Some(position) = self
                .events
                .iter()
                .position(|event| matches!(event.kind, TerminalEventKind::Output { .. }))
            else {
                break;
            };
            let Some(event) = self.events.remove(position) else {
                break;
            };
            let dropped = output_length(&event.kind);
            self.pending_output_bytes = self.pending_output_bytes.saturating_sub(dropped);
            self.output_bytes_dropped = self
                .output_bytes_dropped
                .saturating_add(u64::try_from(dropped).unwrap_or(u64::MAX));
        }
    }

    fn drop_oldest_event(&mut self) {
        if let Some(event) = self.events.pop_front() {
            let dropped = output_length(&event.kind);
            self.pending_output_bytes = self.pending_output_bytes.saturating_sub(dropped);
            self.output_bytes_dropped = self
                .output_bytes_dropped
                .saturating_add(u64::try_from(dropped).unwrap_or(u64::MAX));
        }
    }

    fn acknowledge(&mut self, through: u64) {
        while self
            .events
            .front()
            .is_some_and(|event| event.chunk_sequence <= through)
        {
            if let Some(event) = self.events.pop_front() {
                self.pending_output_bytes = self
                    .pending_output_bytes
                    .saturating_sub(output_length(&event.kind));
            }
        }
    }

    fn drain_after(
        &self,
        process_generation: u64,
        after: u64,
        maximum_events: usize,
        maximum_bytes: usize,
    ) -> Vec<TerminalEvent> {
        let mut events = Vec::new();
        let mut bytes = 0usize;
        for event in self.events.iter().filter(|event| {
            event.process_generation == process_generation && event.chunk_sequence > after
        }) {
            let event_bytes = output_length(&event.kind).saturating_add(256);
            if events.len() == maximum_events
                || (!events.is_empty() && bytes.saturating_add(event_bytes) > maximum_bytes)
            {
                break;
            }
            bytes = bytes.saturating_add(event_bytes);
            events.push(event.clone());
        }
        events
    }
}

enum ReaderFrame {
    Data(Vec<u8>),
    Eof,
    Error(String),
}

enum ReplacementReason {
    StopTimeout,
    ReaderFailure(String),
    InvalidSentinel(String),
}

enum ParsedFrame {
    Started,
    Output(Vec<u8>),
    Completed { exit_code: i32, cwd: PathBuf },
}

struct SentinelParser {
    prefix: Vec<u8>,
    command_sequence: u64,
    buffer: Vec<u8>,
    started: bool,
}

impl SentinelParser {
    fn new(token: &str, command_sequence: u64) -> Self {
        Self {
            prefix: format!("\x1eC4OS:{token}:").into_bytes(),
            command_sequence,
            buffer: Vec::new(),
            started: false,
        }
    }

    fn feed(&mut self, bytes: &[u8]) -> Result<Vec<ParsedFrame>, TerminalError> {
        self.buffer.extend_from_slice(bytes);
        let mut parsed = Vec::new();
        loop {
            let Some(position) = find_subslice(&self.buffer, &self.prefix) else {
                let retained = terminal_sentinel_prefix_suffix(&self.buffer, &self.prefix);
                let emitted = self.buffer.len().saturating_sub(retained);
                if emitted > 0 {
                    let bytes = self.buffer.drain(..emitted).collect::<Vec<_>>();
                    if self.started {
                        push_bounded_output(&mut parsed, bytes);
                    }
                }
                break;
            };
            if position > 0 {
                let bytes = self.buffer.drain(..position).collect::<Vec<_>>();
                if self.started {
                    push_bounded_output(&mut parsed, bytes);
                }
            }
            let Some(end) = self.buffer.iter().position(|byte| *byte == 0x1f) else {
                if self.buffer.len() > MAX_SENTINEL_BYTES {
                    let byte = self.buffer.remove(0);
                    if self.started {
                        push_bounded_output(&mut parsed, vec![byte]);
                    }
                    continue;
                }
                break;
            };
            let payload = self.buffer[self.prefix.len()..end].to_vec();
            self.buffer.drain(..=end);
            let payload = std::str::from_utf8(&payload).map_err(|_| {
                TerminalError::InvalidSentinel("sentinel payload is not UTF-8".into())
            })?;
            let start = format!("START:{}", self.command_sequence);
            if payload == start {
                if self.started {
                    return Err(TerminalError::InvalidSentinel(
                        "duplicate Terminal start sentinel".into(),
                    ));
                }
                self.started = true;
                parsed.push(ParsedFrame::Started);
                continue;
            }
            let expected = format!("END:{}:", self.command_sequence);
            let Some(rest) = payload.strip_prefix(&expected) else {
                return Err(TerminalError::InvalidSentinel(
                    "Terminal sentinel identity is mismatched".into(),
                ));
            };
            if !self.started {
                return Err(TerminalError::InvalidSentinel(
                    "Terminal completion preceded its start".into(),
                ));
            }
            let Some((exit_code, cwd)) = rest.split_once(':') else {
                return Err(TerminalError::InvalidSentinel(
                    "Terminal completion sentinel is incomplete".into(),
                ));
            };
            let exit_code = exit_code.parse::<i32>().map_err(|_| {
                TerminalError::InvalidSentinel("Terminal exit code is invalid".into())
            })?;
            if !(0..=255).contains(&exit_code) {
                return Err(TerminalError::InvalidSentinel(
                    "Terminal exit code is outside the shell range".into(),
                ));
            }
            let cwd = BASE64_STANDARD.decode(cwd).map_err(|_| {
                TerminalError::InvalidSentinel("Terminal cwd encoding is invalid".into())
            })?;
            let cwd = String::from_utf8(cwd)
                .map_err(|_| TerminalError::InvalidSentinel("Terminal cwd is not UTF-8".into()))?;
            let cwd = PathBuf::from(cwd);
            if !cwd.is_absolute() || cwd.as_os_str().is_empty() {
                return Err(TerminalError::InvalidSentinel(
                    "Terminal cwd is not absolute".into(),
                ));
            }
            self.started = false;
            parsed.push(ParsedFrame::Completed { exit_code, cwd });
        }
        Ok(parsed)
    }
}

fn terminal_sentinel_prefix_suffix(bytes: &[u8], prefix: &[u8]) -> usize {
    let maximum = prefix.len().saturating_sub(1).min(bytes.len());
    (1..=maximum)
        .rev()
        .find(|length| bytes.ends_with(&prefix[..*length]))
        .unwrap_or(0)
}

fn spawn_live_session(
    request: &TerminalExecuteRequest,
    canonical_shell: PathBuf,
    restart_cwd: Option<PathBuf>,
    configuration: &TerminalSupervisorConfiguration,
    ledger: &EventLedger,
) -> Result<LiveSession, TerminalError> {
    let working_directory =
        retained_restart_directory(&request.trusted_project_root, restart_cwd.as_deref());
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(request.dimensions.pty_size())
        .map_err(|error| TerminalError::Pty(error.to_string()))?;
    disable_pty_echo(pair.master.as_ref())?;
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| TerminalError::Pty(error.to_string()))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| TerminalError::Pty(error.to_string()))?;

    let mut command = CommandBuilder::new(&canonical_shell);
    command.arg("-f");
    command.arg("-i");
    command.cwd(&working_directory);
    command.env_clear();
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("LANG", "C");
    command.env("LC_ALL", "C");
    command.env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin");
    command.env("PS1", "");
    command.env("PS2", "");
    command.env("PS3", "");
    command.env("PS4", "");
    let mut child = pair
        .slave
        .spawn_command(command)
        .map_err(|error| TerminalError::Pty(error.to_string()))?;
    drop(pair.slave);
    let Some(process_id) = child.process_id() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(TerminalError::ProcessIdentityUnavailable);
    };
    let Some(_foreground_process_group_id) = pair
        .master
        .process_group_leader()
        .filter(|process_group| *process_group > 0)
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(TerminalError::ForegroundProcessUnavailable);
    };
    let (reader_tx, reader_rx) = mpsc::sync_channel(configuration.reader_channel_capacity);
    let reader_thread = match thread::Builder::new()
        .name(format!(
            "c4os-terminal-reader-{}-{}",
            request.key.chat_id, request.process_generation
        ))
        .spawn(move || blocking_reader(&mut reader, reader_tx))
    {
        Ok(reader_thread) => reader_thread,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TerminalError::PtyIo(error));
        }
    };
    let mut live = LiveSession {
        key: request.key.clone(),
        terminal_session_id: request.command.terminal_session_id.clone(),
        process_generation: request.process_generation,
        process_id,
        environment: request.environment.clone(),
        dimensions: request.dimensions,
        trusted_project_root: request.trusted_project_root.clone(),
        shell_path: canonical_shell,
        working_directory,
        next_command_sequence: request.command.command_sequence,
        last_command: request.command.clone(),
        active_command: None,
        master: pair.master,
        writer,
        child,
        reader_rx: Some(reader_rx),
        reader_thread: Some(reader_thread),
        reader_closed: false,
        ledger: ledger.clone(),
    };
    if let Err(error) = write_command(&mut live, &request.command, &request.command_line) {
        let _ = terminate_live_session(live, configuration.shutdown_timeout);
        return Err(error);
    }
    Ok(live)
}

fn write_command(
    live: &mut LiveSession,
    identity: &TerminalCommandIdentity,
    command_line: &str,
) -> Result<(), TerminalError> {
    if live.active_command.is_some() {
        return Err(TerminalError::CommandAlreadyRunning);
    }
    if identity.terminal_session_id != live.terminal_session_id
        || identity.command_sequence != live.next_command_sequence
    {
        return Err(TerminalError::StaleGeneration);
    }
    let next_command_sequence = identity
        .command_sequence
        .checked_add(1)
        .ok_or(TerminalError::GenerationExhausted)?;
    let token = Uuid::new_v4().as_simple().to_string();
    let status_variable = format!("__c4os_status_{token}");
    let cwd_variable = format!("__c4os_cwd_{token}");
    let quoted_command = shell_single_quote(command_line);
    // The wrapper is one shell input line. A command that reads stdin cannot
    // consume private completion framing because the retained shell parses the
    // complete wrapper before `eval` transfers the foreground PTY to a child.
    // `eval` runs in this shell, so successful `cd` changes remain persistent.
    let wrapper = format!(
        "stty -echo || exit 126; printf '\\036C4OS:{token}:START:{}\\037'; trap ':' INT; eval {quoted_command}; {status_variable}=$?; trap - INT; {cwd_variable}=$(/bin/pwd -P); printf '\\036C4OS:{token}:END:{}:%s:' \"${status_variable}\"; printf '%s' \"${cwd_variable}\" | /usr/bin/base64 | /usr/bin/tr -d '\\n'; printf '\\037'\n",
        identity.command_sequence, identity.command_sequence
    );
    live.writer
        .write_all(wrapper.as_bytes())
        .and_then(|_| live.writer.flush())
        .map_err(TerminalError::PtyIo)?;
    live.last_command = identity.clone();
    live.next_command_sequence = next_command_sequence;
    live.active_command = Some(ActiveCommand {
        identity: identity.clone(),
        parser: SentinelParser::new(&token, identity.command_sequence),
        started: false,
        stop_deadline: None,
    });
    Ok(())
}

fn pump_reader(
    live: &mut LiveSession,
    configuration: &TerminalSupervisorConfiguration,
) -> Result<Option<ReplacementReason>, TerminalError> {
    let key = live.key.clone();
    let Some(receiver) = live.reader_rx.as_ref() else {
        return Ok(Some(ReplacementReason::ReaderFailure(
            "Terminal reader is unavailable".into(),
        )));
    };
    let mut frames = Vec::new();
    for _ in 0..configuration.reader_frames_per_drain {
        match receiver.try_recv() {
            Ok(frame) => frames.push(frame),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                live.reader_closed = true;
                break;
            }
        }
    }
    for frame in frames {
        match frame {
            ReaderFrame::Data(bytes) => {
                let (parsed, identity, interrupted) = {
                    let Some(active) = live.active_command.as_mut() else {
                        continue;
                    };
                    let parsed = match active.parser.feed(&bytes) {
                        Ok(parsed) => parsed,
                        Err(TerminalError::InvalidSentinel(message)) => {
                            return Ok(Some(ReplacementReason::InvalidSentinel(message)));
                        }
                        Err(error) => return Err(error),
                    };
                    (
                        parsed,
                        active.identity.clone(),
                        active.stop_deadline.is_some(),
                    )
                };
                let mut completed = false;
                for parsed in parsed {
                    match parsed {
                        ParsedFrame::Started => {
                            if let Some(active) = live.active_command.as_mut() {
                                active.started = true;
                            }
                            let foreground_process_group_id = live
                                .master
                                .process_group_leader()
                                .filter(|process_group| *process_group > 0)
                                .unwrap_or_else(|| {
                                    i32::try_from(live.process_id).unwrap_or(i32::MAX)
                                });
                            live.ledger.push(
                                &key,
                                &identity,
                                live.process_generation,
                                TerminalEventKind::Started {
                                    process_id: live.process_id,
                                    foreground_process_group_id,
                                    working_directory: live.working_directory.clone(),
                                },
                            );
                        }
                        ParsedFrame::Output(bytes) => live.ledger.push(
                            &key,
                            &identity,
                            live.process_generation,
                            TerminalEventKind::Output {
                                bytes,
                                dropped_bytes_before: 0,
                            },
                        ),
                        ParsedFrame::Completed { exit_code, cwd } => {
                            live.working_directory = cwd.clone();
                            live.ledger.push(
                                &key,
                                &identity,
                                live.process_generation,
                                if interrupted {
                                    TerminalEventKind::Interrupted130 {
                                        working_directory: cwd,
                                        shell_replaced: false,
                                    }
                                } else {
                                    TerminalEventKind::Completed {
                                        exit_code,
                                        working_directory: cwd,
                                    }
                                },
                            );
                            completed = true;
                            break;
                        }
                    }
                }
                if completed {
                    live.active_command = None;
                }
            }
            ReaderFrame::Eof => live.reader_closed = true,
            ReaderFrame::Error(message) => {
                return Ok(Some(ReplacementReason::ReaderFailure(message)));
            }
        }
    }

    if live
        .active_command
        .as_ref()
        .and_then(|active| active.stop_deadline)
        .is_some_and(|deadline| Instant::now() >= deadline)
    {
        return Ok(Some(ReplacementReason::StopTimeout));
    }
    let exited = live.child.try_wait().map_err(TerminalError::PtyIo)?;
    if live.reader_closed || exited.is_some() {
        let message = exited.map_or_else(
            || "The Terminal reader closed before a command boundary".into(),
            |status| format!("The Terminal shell exited with code {}", status.exit_code()),
        );
        return Ok(Some(ReplacementReason::ReaderFailure(message)));
    }
    Ok(None)
}

fn blocking_reader(reader: &mut dyn Read, sender: SyncSender<ReaderFrame>) {
    let mut buffer = vec![0u8; READER_FRAME_BYTES];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                let _ = sender.send(ReaderFrame::Eof);
                return;
            }
            Ok(read) => {
                if sender
                    .send(ReaderFrame::Data(buffer[..read].to_vec()))
                    .is_err()
                {
                    return;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                let _ = sender.send(ReaderFrame::Error(error.to_string()));
                return;
            }
        }
    }
}

fn validate_live_execution(
    live: &LiveSession,
    request: &TerminalExecuteRequest,
    canonical_shell: &Path,
) -> Result<(), TerminalError> {
    validate_session_binding(
        &live.terminal_session_id,
        live.process_generation,
        &request.command.terminal_session_id,
        request.process_generation,
    )?;
    if live.active_command.is_some() {
        return Err(TerminalError::CommandAlreadyRunning);
    }
    if request.command.command_sequence != live.next_command_sequence
        || request.environment != live.environment
        || request.trusted_project_root != live.trusted_project_root
        || canonical_shell != live.shell_path
    {
        return Err(TerminalError::StaleBinding(
            "Terminal shell binding changed".into(),
        ));
    }
    live.trusted_project_root
        .revalidate()
        .map_err(|_| TerminalError::StaleBinding("The physical Project root changed".into()))
}

fn validate_dormant_execution(
    dormant: &DormantSession,
    request: &TerminalExecuteRequest,
    canonical_shell: &Path,
) -> Result<(), TerminalError> {
    let expected_generation = dormant
        .process_generation
        .checked_add(1)
        .ok_or(TerminalError::GenerationExhausted)?;
    if request.command.terminal_session_id != dormant.terminal_session_id
        || request.process_generation != expected_generation
        || request.command.command_sequence != dormant.next_command_sequence
        || request.environment != dormant.environment
        || request.trusted_project_root != dormant.trusted_project_root
        || canonical_shell != dormant.shell_path
    {
        return Err(TerminalError::StaleGeneration);
    }
    Ok(())
}

fn validate_restart_record(record: &TerminalRestartRecord) -> Result<(), TerminalError> {
    record.key.validate()?;
    record.command.validate()?;
    validate_environment(&record.environment)?;
    record.dimensions.validate()?;
    record
        .trusted_project_root
        .revalidate()
        .map_err(|_| TerminalError::StaleBinding("The recovered Project root changed".into()))?;
    if record.process_generation == 0 {
        return Err(TerminalError::InvalidRequest(
            "Recovered process generation must be non-zero",
        ));
    }
    Ok(())
}

fn validate_completed_session_record(
    record: &TerminalCompletedSessionRecord,
) -> Result<(), TerminalError> {
    record.key.validate()?;
    record.command.validate()?;
    validate_environment(&record.environment)?;
    record.dimensions.validate()?;
    record
        .trusted_project_root
        .revalidate()
        .map_err(|_| TerminalError::StaleBinding("The restored Project root changed".into()))?;
    if record.process_generation == 0 {
        return Err(TerminalError::InvalidRequest(
            "Restored process generation must be non-zero",
        ));
    }
    Ok(())
}

fn validate_environment(environment: &ExecutionEnvironmentIdentity) -> Result<(), TerminalError> {
    if environment.kind != ExecutionEnvironmentKind::Local {
        return Err(TerminalError::UnsupportedEnvironment);
    }
    validate_identifier("Terminal environment id", &environment.environment_id)?;
    if environment.generation == 0 {
        return Err(TerminalError::InvalidRequest(
            "Terminal environment generation must be non-zero",
        ));
    }
    Ok(())
}

fn validate_command_line(command: &str) -> Result<(), TerminalError> {
    if command.trim().is_empty()
        || command.len() > MAX_TERMINAL_COMMAND_BYTES
        || command
            .chars()
            .any(|character| matches!(character, '\0' | '\n' | '\r'))
    {
        return Err(TerminalError::InvalidRequest(
            "Terminal command must be one non-empty bounded line",
        ));
    }
    Ok(())
}

fn validate_shell_path(path: &Path) -> Result<PathBuf, TerminalError> {
    if !path.is_absolute() {
        return Err(TerminalError::InvalidRequest(
            "Terminal shell path must be absolute",
        ));
    }
    let canonical = fs::canonicalize(path).map_err(TerminalError::PtyIo)?;
    let metadata = fs::metadata(&canonical).map_err(TerminalError::PtyIo)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(TerminalError::InvalidRequest(
            "Terminal shell must be an executable regular file",
        ));
    }
    Ok(canonical)
}

fn validate_session_binding(
    actual_session_id: &str,
    actual_generation: u64,
    requested_session_id: &str,
    requested_generation: u64,
) -> Result<(), TerminalError> {
    if actual_session_id != requested_session_id || actual_generation != requested_generation {
        Err(TerminalError::StaleGeneration)
    } else {
        Ok(())
    }
}

fn validate_drain_request(request: &TerminalDrainRequest) -> Result<(), TerminalError> {
    request.key.validate()?;
    validate_identifier("Terminal session id", &request.terminal_session_id)?;
    if request.process_generation == 0
        || request.maximum_events == 0
        || request.maximum_events > MAX_TERMINAL_DRAIN_EVENTS
        || request.maximum_bytes == 0
        || request.maximum_bytes > MAX_TERMINAL_DRAIN_BYTES
    {
        return Err(TerminalError::InvalidRequest(
            "Terminal event drain bound is invalid",
        ));
    }
    Ok(())
}

fn validate_identifier(label: &'static str, value: &str) -> Result<(), TerminalError> {
    if value.is_empty()
        || value.len() > MAX_TERMINAL_IDENTIFIER_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
    {
        Err(TerminalError::InvalidIdentifier(label))
    } else {
        Ok(())
    }
}

fn retained_restart_directory(root: &TrustedProjectRoot, candidate: Option<&Path>) -> PathBuf {
    candidate
        .and_then(|candidate| fs::canonicalize(candidate).ok())
        .filter(|candidate| candidate.is_dir() && candidate.starts_with(root.canonical_root()))
        .unwrap_or_else(|| root.canonical_root().to_path_buf())
}

fn snapshot_live(key: &TerminalSessionKey, live: &LiveSession) -> TerminalLiveSessionSnapshot {
    TerminalLiveSessionSnapshot {
        key: key.clone(),
        terminal_session_id: live.terminal_session_id.clone(),
        lifecycle: TerminalLifecycle::Live,
        process_generation: live.process_generation,
        process_id: Some(live.process_id),
        foreground_process_group_id: live.master.process_group_leader(),
        active_command: live
            .active_command
            .as_ref()
            .map(|command| command.identity.clone()),
        next_command_sequence: live.next_command_sequence,
        environment: live.environment.clone(),
        dimensions: live.dimensions,
        shell_path: live.shell_path.clone(),
        working_directory: live.working_directory.clone(),
        pending_event_count: live.ledger.events.len(),
        pending_output_bytes: live.ledger.pending_output_bytes,
        output_bytes_dropped: live.ledger.output_bytes_dropped,
    }
}

fn snapshot_dormant(
    key: &TerminalSessionKey,
    dormant: &DormantSession,
) -> TerminalLiveSessionSnapshot {
    TerminalLiveSessionSnapshot {
        key: key.clone(),
        terminal_session_id: dormant.terminal_session_id.clone(),
        lifecycle: dormant.lifecycle,
        process_generation: dormant.process_generation,
        process_id: None,
        foreground_process_group_id: None,
        active_command: None,
        next_command_sequence: dormant.next_command_sequence,
        environment: dormant.environment.clone(),
        dimensions: dormant.dimensions,
        shell_path: dormant.shell_path.clone(),
        working_directory: dormant.working_directory.clone(),
        pending_event_count: dormant.ledger.events.len(),
        pending_output_bytes: dormant.ledger.pending_output_bytes,
        output_bytes_dropped: dormant.ledger.output_bytes_dropped,
    }
}

fn terminate_live_session(mut live: LiveSession, timeout: Duration) -> Result<(), TerminalError> {
    let shell_pid =
        i32::try_from(live.process_id).map_err(|_| TerminalError::ProcessIdentityUnavailable)?;
    let foreground = live.master.process_group_leader();
    if let Some(foreground) = foreground.filter(|foreground| *foreground > 0) {
        let _ = signal_process_group(foreground, libc::SIGTERM);
    }
    let _ = signal_process_group(shell_pid, libc::SIGTERM);
    drop(live.writer);
    drop(live.master);
    drop(live.reader_rx.take());
    let deadline = Instant::now() + timeout;
    loop {
        if live
            .child
            .try_wait()
            .map_err(TerminalError::PtyIo)?
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            if let Some(foreground) = foreground.filter(|foreground| *foreground > 0) {
                let _ = signal_process_group(foreground, libc::SIGKILL);
            }
            let _ = signal_process_group(shell_pid, libc::SIGKILL);
            let _ = live.child.kill();
            let _ = live.child.wait();
            break;
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
    if let Some(reader) = live.reader_thread.take() {
        reader.join().map_err(|_| TerminalError::ReaderPanicked)?;
    }
    Ok(())
}

fn disable_pty_echo(master: &dyn MasterPty) -> Result<(), TerminalError> {
    let file_descriptor = master.as_raw_fd().ok_or_else(|| {
        TerminalError::Pty("the PTY configuration interface is unavailable".into())
    })?;
    let mut attributes = std::mem::MaybeUninit::<libc::termios>::uninit();
    // SAFETY: `file_descriptor` is the live master PTY descriptor supplied by
    // portable-pty, and `attributes` points to writable termios storage.
    if unsafe { libc::tcgetattr(file_descriptor, attributes.as_mut_ptr()) } != 0 {
        return Err(TerminalError::PtyIo(io::Error::last_os_error()));
    }
    // SAFETY: successful `tcgetattr` initialized the complete termios value.
    let mut attributes = unsafe { attributes.assume_init() };
    attributes.c_lflag &= !(libc::ECHO | libc::ECHONL);
    // SAFETY: the descriptor and initialized termios value remain valid for
    // this immediate update, before either spawn or an authorized stdin write.
    if unsafe { libc::tcsetattr(file_descriptor, libc::TCSANOW, &attributes) } != 0 {
        return Err(TerminalError::PtyIo(io::Error::last_os_error()));
    }
    let mut verified = std::mem::MaybeUninit::<libc::termios>::uninit();
    // SAFETY: this is the same live PTY descriptor and writable termios
    // storage used above, read back synchronously before any stdin write.
    if unsafe { libc::tcgetattr(file_descriptor, verified.as_mut_ptr()) } != 0 {
        return Err(TerminalError::PtyIo(io::Error::last_os_error()));
    }
    // SAFETY: successful `tcgetattr` initialized the complete termios value.
    let verified = unsafe { verified.assume_init() };
    if verified.c_lflag & (libc::ECHO | libc::ECHONL) != 0 {
        return Err(TerminalError::Pty(
            "the PTY refused to disable input echo".into(),
        ));
    }
    Ok(())
}

fn signal_process_group(process_group_id: i32, signal: i32) -> Result<(), TerminalError> {
    if process_group_id <= 0 {
        return Err(TerminalError::ForegroundProcessUnavailable);
    }
    // SAFETY: a negative, validated process-group id targets only that Unix
    // process group. No pointer or borrowed memory crosses the FFI boundary.
    let result = unsafe { libc::kill(-process_group_id, signal) };
    if result == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(TerminalError::PtyIo(error))
    }
}

fn push_bounded_output(parsed: &mut Vec<ParsedFrame>, bytes: Vec<u8>) {
    for chunk in bytes.chunks(MAX_TERMINAL_EVENT_OUTPUT_BYTES) {
        if !chunk.is_empty() {
            parsed.push(ParsedFrame::Output(chunk.to_vec()));
        }
    }
}

fn output_length(kind: &TerminalEventKind) -> usize {
    match kind {
        TerminalEventKind::Output { bytes, .. } => bytes.len(),
        _ => 0,
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("invalid Terminal identifier: {0}")]
    InvalidIdentifier(&'static str),
    #[error("invalid Terminal request: {0}")]
    InvalidRequest(&'static str),
    #[error("Terminal session was not found")]
    SessionNotFound,
    #[error("Terminal session already exists")]
    SessionAlreadyExists,
    #[error("Terminal retained-session capacity {maximum} is exhausted")]
    SessionCapacityExceeded { maximum: usize },
    #[error("Terminal shell is unavailable and must be replaced")]
    ShellUnavailable,
    #[error("a Terminal command is already running")]
    CommandAlreadyRunning,
    #[error("there is no matching active Terminal command")]
    NoActiveCommand,
    #[error("the Terminal command has not reached its private start boundary")]
    CommandNotReady,
    #[error("the Terminal command is already stopping")]
    AlreadyStopping,
    #[error("the Terminal process generation or command sequence is stale")]
    StaleGeneration,
    #[error("the Terminal binding changed: {0}")]
    StaleBinding(String),
    #[error("Terminal process generation is exhausted")]
    GenerationExhausted,
    #[error("only the local execution environment has a production PTY implementation")]
    UnsupportedEnvironment,
    #[error("the Terminal process identity is unavailable")]
    ProcessIdentityUnavailable,
    #[error("the Terminal foreground process group is unavailable")]
    ForegroundProcessUnavailable,
    #[error("the Terminal sentinel is invalid: {0}")]
    InvalidSentinel(String),
    #[error("the Terminal PTY failed: {0}")]
    Pty(String),
    #[error("the Terminal PTY I/O failed: {0}")]
    PtyIo(#[source] io::Error),
    #[error("the Terminal reader thread panicked")]
    ReaderPanicked,
}
