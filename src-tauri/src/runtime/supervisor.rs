//! Exact-version runtime installation and process supervision.
//!
//! Runtime workers are disposable least-authority processes. C4OS owns their
//! compatibility state, isolated namespace, process generation, health, and
//! process-group cleanup; a worker cannot make itself authoritative by merely
//! emitting a ready event.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub const RUNTIME_PROTOCOL_VERSION: u16 = 1;
pub const OPENCODE_NATIVE_VERSION: &str = "1.18.3";
pub const PI_NATIVE_VERSION: &str = "0.80.10";
const MAX_ID_BYTES: usize = 160;
const MAX_ARGUMENTS: usize = 128;
const MAX_ARGUMENT_BYTES: usize = 4_096;
const MAX_ENVIRONMENT_FIELDS: usize = 32;
const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RUNTIME_INSTALLATIONS: usize = 128;
const MAX_TRACE_EVENTS: usize = 4_096;
const MAX_AUTOMATIC_RESTART_ATTEMPTS: u16 = 5;
const MAX_RESTART_BACKOFF_MS: u64 = 300_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeKind {
    OpenCode,
    Pi,
}

impl RuntimeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenCode => "opencode",
            Self::Pi => "pi",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityRequirement {
    pub runtime_kind: RuntimeKind,
    pub native_version: String,
    pub adapter_version: String,
    pub protocol_version: u16,
}

impl CompatibilityRequirement {
    fn validate(&self) -> Result<(), SupervisorError> {
        validate_version(&self.native_version)?;
        validate_version(&self.adapter_version)?;
        if self.protocol_version != RUNTIME_PROTOCOL_VERSION {
            return Err(SupervisorError::UnsupportedProtocol);
        }
        Ok(())
    }
}

pub fn pinned_compatibility() -> [CompatibilityRequirement; 2] {
    [
        CompatibilityRequirement {
            runtime_kind: RuntimeKind::OpenCode,
            native_version: OPENCODE_NATIVE_VERSION.into(),
            adapter_version: "1.0.0".into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
        },
        CompatibilityRequirement {
            runtime_kind: RuntimeKind::Pi,
            native_version: PI_NATIVE_VERSION.into(),
            adapter_version: "1.0.0".into(),
            protocol_version: RUNTIME_PROTOCOL_VERSION,
        },
    ]
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInstallation {
    pub runtime_id: String,
    pub workspace_id: String,
    pub runtime_kind: RuntimeKind,
    pub native_version: String,
    pub adapter_version: String,
    pub protocol_version: u16,
    /// Immutable, verified native asset tree. The launch executable and
    /// writable state namespace are separate authorities and need not be
    /// descendants of this tree (Pi launches a verified bundled Node host).
    pub install_root: PathBuf,
    pub asset_tree_sha256: String,
    pub executable: PathBuf,
    pub executable_sha256: String,
    pub state_namespace: PathBuf,
    pub arguments: Vec<String>,
    pub sanitized_environment: BTreeMap<String, String>,
}

impl RuntimeInstallation {
    pub fn validate(&self) -> Result<(), SupervisorError> {
        validate_id(&self.runtime_id)?;
        validate_id(&self.workspace_id)?;
        validate_version(&self.native_version)?;
        validate_version(&self.adapter_version)?;
        validate_digest(&self.asset_tree_sha256)?;
        validate_digest(&self.executable_sha256)?;
        if self.protocol_version != RUNTIME_PROTOCOL_VERSION
            || !self.install_root.is_absolute()
            || !self.executable.is_absolute()
            || !self.state_namespace.is_absolute()
            || self.arguments.len() > MAX_ARGUMENTS
            || self.arguments.iter().any(|argument| {
                !bounded_value(argument, MAX_ARGUMENT_BYTES) || looks_like_secret(argument)
            })
            || self.sanitized_environment.len() > MAX_ENVIRONMENT_FIELDS
            || self.sanitized_environment.iter().any(|(key, value)| {
                !valid_environment_key(key)
                    || contains_secret_name(key)
                    || !bounded_value(value, MAX_ARGUMENT_BYTES)
                    || looks_like_secret(value)
            })
        {
            return Err(SupervisorError::InvalidInstallation);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityState {
    Compatible,
    Incompatible,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeLifecycle {
    Stopped,
    Starting,
    Ready,
    Degraded,
    Incompatible,
    Stopping,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealthState {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRecoveryAction {
    ReviewRuntimeCrashLoop,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRecord {
    pub installation: RuntimeInstallation,
    pub compatibility: CompatibilityState,
    pub lifecycle: RuntimeLifecycle,
    pub health: HealthState,
    pub process_generation: u64,
    pub process_id: Option<u32>,
    pub checked_at_ms: u64,
    pub last_exit_code: Option<i32>,
    #[serde(default)]
    pub restart_attempts: u16,
    #[serde(default)]
    pub next_restart_at_ms: Option<u64>,
    #[serde(default)]
    pub recovery_action: Option<RuntimeRecoveryAction>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupervisorSnapshot {
    pub state_generation: u64,
    pub records: Vec<RuntimeRecord>,
    pub events: Vec<RuntimeTraceEvent>,
    pub trace_events_dropped: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeTraceEvent {
    pub runtime_id: String,
    pub process_generation: u64,
    pub at_ms: u64,
    pub kind: RuntimeTraceKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeTraceKind {
    InstallationRegistered,
    ProcessStarted,
    Ready,
    Degraded,
    Unhealthy,
    StaleEventRejected,
    TerminationRequested,
    ForcedTermination,
    Stopped,
    Exited,
    RecoveredInterrupted,
    RecoveryReviewed,
}

struct ManagedProcess {
    child: Child,
    process_group_id: u32,
}

pub struct RuntimeSupervisor {
    compatibility: BTreeMap<RuntimeKind, CompatibilityRequirement>,
    records: BTreeMap<String, RuntimeRecord>,
    processes: HashMap<String, ManagedProcess>,
    events: Vec<RuntimeTraceEvent>,
    state_generation: u64,
    trace_events_dropped: u64,
}

impl RuntimeSupervisor {
    pub fn pinned() -> Self {
        Self::new(pinned_compatibility()).expect("pinned compatibility records are valid")
    }

    pub fn new(
        requirements: impl IntoIterator<Item = CompatibilityRequirement>,
    ) -> Result<Self, SupervisorError> {
        let mut compatibility = BTreeMap::new();
        for requirement in requirements {
            requirement.validate()?;
            if compatibility
                .insert(requirement.runtime_kind, requirement)
                .is_some()
            {
                return Err(SupervisorError::DuplicateCompatibility);
            }
        }
        Ok(Self {
            compatibility,
            records: BTreeMap::new(),
            processes: HashMap::new(),
            events: Vec::new(),
            state_generation: 0,
            trace_events_dropped: 0,
        })
    }

    /// Restores durable installation/version/generation truth but never trusts
    /// a persisted PID or ready state. Any previously active worker becomes a
    /// visible interrupted recovery record and must be started and health-
    /// checked again under a fresh process generation.
    pub fn restore(
        requirements: impl IntoIterator<Item = CompatibilityRequirement>,
        snapshot: SupervisorSnapshot,
    ) -> Result<Self, SupervisorError> {
        if snapshot.records.len() > MAX_RUNTIME_INSTALLATIONS
            || snapshot.events.len() > MAX_TRACE_EVENTS
        {
            return Err(SupervisorError::InvalidSnapshot);
        }
        let mut supervisor = Self::new(requirements)?;
        supervisor.state_generation = snapshot.state_generation;
        supervisor.trace_events_dropped = snapshot.trace_events_dropped;
        let mut recovered = Vec::new();
        for mut record in snapshot.records {
            record.installation.validate()?;
            if supervisor
                .records
                .contains_key(&record.installation.runtime_id)
            {
                return Err(SupervisorError::InvalidSnapshot);
            }
            let requirement = supervisor
                .compatibility
                .get(&record.installation.runtime_kind)
                .ok_or(SupervisorError::NoCompatibilityRecord)?;
            let compatibility = if requirement.native_version == record.installation.native_version
                && requirement.adapter_version == record.installation.adapter_version
                && requirement.protocol_version == record.installation.protocol_version
            {
                CompatibilityState::Compatible
            } else {
                CompatibilityState::Incompatible
            };
            let was_active = record.process_id.is_some()
                || matches!(
                    record.lifecycle,
                    RuntimeLifecycle::Starting
                        | RuntimeLifecycle::Ready
                        | RuntimeLifecycle::Degraded
                        | RuntimeLifecycle::Stopping
                );
            record.compatibility = compatibility;
            record.lifecycle = if compatibility == CompatibilityState::Compatible {
                RuntimeLifecycle::Stopped
            } else {
                RuntimeLifecycle::Incompatible
            };
            record.health = HealthState::Unknown;
            record.process_id = None;
            if was_active {
                let recovered_at_ms = record.checked_at_ms.max(1);
                apply_crash_backoff(&mut record, recovered_at_ms);
            }
            validate_restart_state(&record)?;
            let runtime_id = record.installation.runtime_id.clone();
            let generation = record.process_generation;
            let checked_at_ms = record.checked_at_ms;
            supervisor.records.insert(runtime_id.clone(), record);
            if was_active {
                recovered.push(RuntimeTraceEvent {
                    runtime_id,
                    process_generation: generation,
                    at_ms: checked_at_ms.max(1),
                    kind: RuntimeTraceKind::RecoveredInterrupted,
                });
            }
        }
        for event in snapshot.events {
            validate_id(&event.runtime_id)?;
            if event.at_ms == 0
                || !supervisor.records.contains_key(&event.runtime_id)
                || event.process_generation
                    > supervisor.records[&event.runtime_id].process_generation
            {
                return Err(SupervisorError::InvalidSnapshot);
            }
            supervisor.push_trace(event);
        }
        let recovered_any = !recovered.is_empty();
        for event in recovered {
            supervisor.push_trace(event);
        }
        if recovered_any {
            supervisor.bump_state_generation()?;
        }
        Ok(supervisor)
    }

    pub fn register_installation(
        &mut self,
        installation: RuntimeInstallation,
        checked_at_ms: u64,
    ) -> Result<CompatibilityState, SupervisorError> {
        installation.validate()?;
        if checked_at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        if self.processes.contains_key(&installation.runtime_id) {
            return Err(SupervisorError::AlreadyRunning);
        }
        if !self.records.contains_key(&installation.runtime_id)
            && self.records.len() >= MAX_RUNTIME_INSTALLATIONS
        {
            return Err(SupervisorError::CapacityExceeded);
        }
        let requirement = self
            .compatibility
            .get(&installation.runtime_kind)
            .ok_or(SupervisorError::NoCompatibilityRecord)?;
        let compatibility = if requirement.native_version == installation.native_version
            && requirement.adapter_version == installation.adapter_version
            && requirement.protocol_version == installation.protocol_version
        {
            CompatibilityState::Compatible
        } else {
            CompatibilityState::Incompatible
        };
        self.bump_state_generation()?;
        let process_generation = self
            .records
            .get(&installation.runtime_id)
            .map_or(0, |record| record.process_generation);
        let runtime_id = installation.runtime_id.clone();
        self.records.insert(
            runtime_id.clone(),
            RuntimeRecord {
                installation,
                compatibility,
                lifecycle: if compatibility == CompatibilityState::Compatible {
                    RuntimeLifecycle::Stopped
                } else {
                    RuntimeLifecycle::Incompatible
                },
                health: HealthState::Unknown,
                process_generation,
                process_id: None,
                checked_at_ms,
                last_exit_code: None,
                restart_attempts: 0,
                next_restart_at_ms: None,
                recovery_action: None,
            },
        );
        self.trace(
            &runtime_id,
            process_generation,
            checked_at_ms,
            RuntimeTraceKind::InstallationRegistered,
        );
        Ok(compatibility)
    }

    /// Replaces the complete installation set for one authoritative
    /// Workspace in a single supervisor generation. The candidate is fully
    /// validated before any record or trace becomes visible, and an active
    /// native generation can never be displaced by a Workspace switch.
    pub fn replace_workspace_installations(
        &mut self,
        workspace_id: &str,
        installations: Vec<RuntimeInstallation>,
        checked_at_ms: u64,
    ) -> Result<BTreeMap<String, CompatibilityState>, SupervisorError> {
        validate_id(workspace_id)?;
        if checked_at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        if installations.is_empty() || installations.len() > MAX_RUNTIME_INSTALLATIONS {
            return Err(SupervisorError::CapacityExceeded);
        }
        if !self.processes.is_empty()
            || self.records.values().any(|record| {
                record.process_id.is_some()
                    || matches!(
                        record.lifecycle,
                        RuntimeLifecycle::Starting
                            | RuntimeLifecycle::Ready
                            | RuntimeLifecycle::Degraded
                            | RuntimeLifecycle::Stopping
                    )
            })
        {
            return Err(SupervisorError::AlreadyRunning);
        }

        let mut records = BTreeMap::new();
        let mut compatibility_by_runtime = BTreeMap::new();
        for installation in installations {
            installation.validate()?;
            if installation.workspace_id != workspace_id {
                return Err(SupervisorError::InvalidInstallation);
            }
            let requirement = self
                .compatibility
                .get(&installation.runtime_kind)
                .ok_or(SupervisorError::NoCompatibilityRecord)?;
            let compatibility = if requirement.native_version == installation.native_version
                && requirement.adapter_version == installation.adapter_version
                && requirement.protocol_version == installation.protocol_version
            {
                CompatibilityState::Compatible
            } else {
                CompatibilityState::Incompatible
            };
            let runtime_id = installation.runtime_id.clone();
            let previous_record = self.records.get(&runtime_id);
            let matching_record = previous_record.filter(|record| {
                record.installation == installation && record.compatibility == compatibility
            });
            let process_generation = previous_record.map_or(0, |record| record.process_generation);
            let record = RuntimeRecord {
                installation,
                compatibility,
                lifecycle: if compatibility == CompatibilityState::Compatible {
                    RuntimeLifecycle::Stopped
                } else {
                    RuntimeLifecycle::Incompatible
                },
                health: HealthState::Unknown,
                process_generation,
                process_id: None,
                checked_at_ms,
                last_exit_code: None,
                restart_attempts: matching_record.map_or(0, |record| record.restart_attempts),
                next_restart_at_ms: matching_record.and_then(|record| record.next_restart_at_ms),
                recovery_action: matching_record.and_then(|record| record.recovery_action),
            };
            if records.insert(runtime_id.clone(), record).is_some()
                || compatibility_by_runtime
                    .insert(runtime_id, compatibility)
                    .is_some()
            {
                return Err(SupervisorError::InvalidInstallation);
            }
        }

        self.bump_state_generation()?;
        self.events.retain(|event| {
            records
                .get(&event.runtime_id)
                .is_some_and(|record| event.process_generation <= record.process_generation)
        });
        self.records = records;
        for (runtime_id, record) in self.records.clone() {
            self.trace(
                &runtime_id,
                record.process_generation,
                checked_at_ms,
                RuntimeTraceKind::InstallationRegistered,
            );
        }
        Ok(compatibility_by_runtime)
    }

    pub fn clear_installations(&mut self) -> Result<(), SupervisorError> {
        if !self.processes.is_empty()
            || self.records.values().any(|record| {
                record.process_id.is_some()
                    || matches!(
                        record.lifecycle,
                        RuntimeLifecycle::Starting
                            | RuntimeLifecycle::Ready
                            | RuntimeLifecycle::Degraded
                            | RuntimeLifecycle::Stopping
                    )
            })
        {
            return Err(SupervisorError::AlreadyRunning);
        }
        if self.records.is_empty() {
            return Ok(());
        }
        self.bump_state_generation()?;
        self.records.clear();
        self.events.clear();
        Ok(())
    }

    pub fn start(&mut self, runtime_id: &str, at_ms: u64) -> Result<u64, SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        if self.processes.contains_key(runtime_id) {
            return Err(SupervisorError::AlreadyRunning);
        }
        let (installation, prior_process_generation) = {
            let record = self
                .records
                .get(runtime_id)
                .ok_or(SupervisorError::NotFound)?;
            if record.compatibility != CompatibilityState::Compatible {
                return Err(SupervisorError::Incompatible);
            }
            require_restart_budget(record, at_ms)?;
            (record.installation.clone(), record.process_generation)
        };
        verify_executable(&installation)?;

        let generation = prior_process_generation
            .checked_add(1)
            .ok_or(SupervisorError::GenerationExhausted)?;
        let state_namespace = prepare_state_namespace(&installation, generation)?;
        let mut command = Command::new(&installation.executable);
        command
            .args(&installation.arguments)
            .current_dir(&state_namespace)
            .env_clear()
            .envs(&installation.sanitized_environment)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(SupervisorError::ProcessIo)?;
        if let Some(status) = child.try_wait().map_err(SupervisorError::ProcessIo)? {
            return Err(SupervisorError::EarlyExit(exit_code(status)));
        }
        if let Err(error) = self.bump_state_generation() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let process_id = child.id();
        self.processes.insert(
            runtime_id.into(),
            ManagedProcess {
                child,
                process_group_id: process_id,
            },
        );
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.process_generation = generation;
        record.process_id = Some(process_id);
        record.lifecycle = RuntimeLifecycle::Starting;
        record.health = HealthState::Unknown;
        record.checked_at_ms = at_ms;
        record.last_exit_code = None;
        self.trace(
            runtime_id,
            generation,
            at_ms,
            RuntimeTraceKind::ProcessStarted,
        );
        Ok(generation)
    }

    /// Reserves the next exact process generation for a production adapter
    /// that owns the native child handle itself. The caller must either attach
    /// the verified process with [`Self::attach_managed_process`] or abort the
    /// reservation with [`Self::abort_managed_start`]. No ready state is
    /// published while only a reservation exists.
    pub fn reserve_managed_start(
        &mut self,
        runtime_id: &str,
        at_ms: u64,
    ) -> Result<u64, SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        if self.processes.contains_key(runtime_id) {
            return Err(SupervisorError::AlreadyRunning);
        }
        let record = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if record.compatibility != CompatibilityState::Compatible {
            return Err(SupervisorError::Incompatible);
        }
        require_restart_budget(record, at_ms)?;
        if record.process_id.is_some()
            || matches!(
                record.lifecycle,
                RuntimeLifecycle::Starting
                    | RuntimeLifecycle::Ready
                    | RuntimeLifecycle::Degraded
                    | RuntimeLifecycle::Stopping
            )
        {
            return Err(SupervisorError::AlreadyRunning);
        }
        verify_executable(&record.installation)?;
        let generation = record
            .process_generation
            .checked_add(1)
            .ok_or(SupervisorError::GenerationExhausted)?;
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.process_generation = generation;
        record.process_id = None;
        record.lifecycle = RuntimeLifecycle::Starting;
        record.health = HealthState::Unknown;
        record.checked_at_ms = at_ms;
        record.last_exit_code = None;
        self.trace(
            runtime_id,
            generation,
            at_ms,
            RuntimeTraceKind::ProcessStarted,
        );
        Ok(generation)
    }

    /// Publishes a production-adapter-owned child only after that adapter has
    /// completed its exact-version health check. The supervisor keeps the
    /// authoritative identity and lifecycle while the production peer keeps
    /// the process handle needed for protocol-aware shutdown.
    pub fn attach_managed_process(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        process_id: u32,
        health: HealthState,
        at_ms: u64,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 || process_id == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        if !matches!(health, HealthState::Healthy | HealthState::Degraded) {
            return Err(SupervisorError::InvalidHealth);
        }
        if self.processes.contains_key(runtime_id) {
            return Err(SupervisorError::AlreadyRunning);
        }
        let record = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if record.process_generation != process_generation
            || record.lifecycle != RuntimeLifecycle::Starting
            || record.process_id.is_some()
        {
            return Err(SupervisorError::StaleGeneration);
        }
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.process_id = Some(process_id);
        record.health = health;
        record.lifecycle = match health {
            HealthState::Healthy => RuntimeLifecycle::Ready,
            HealthState::Degraded => RuntimeLifecycle::Degraded,
            HealthState::Unknown | HealthState::Unhealthy => {
                unreachable!("non-ready health was rejected")
            }
        };
        record.checked_at_ms = at_ms;
        if health == HealthState::Healthy {
            record.restart_attempts = 0;
            record.next_restart_at_ms = None;
            record.recovery_action = None;
        }
        self.trace(
            runtime_id,
            process_generation,
            at_ms,
            match health {
                HealthState::Healthy => RuntimeTraceKind::Ready,
                HealthState::Degraded => RuntimeTraceKind::Degraded,
                HealthState::Unhealthy | HealthState::Unknown => RuntimeTraceKind::Unhealthy,
            },
        );
        Ok(())
    }

    /// Rolls back a generation reservation when production peer preparation
    /// fails before a native child becomes active.
    pub fn abort_managed_start(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        let record = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if self.processes.contains_key(runtime_id)
            || record.process_generation != process_generation
            || record.lifecycle != RuntimeLifecycle::Starting
            || record.process_id.is_some()
        {
            return Err(SupervisorError::StaleGeneration);
        }
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.lifecycle = RuntimeLifecycle::Stopped;
        record.health = HealthState::Unknown;
        record.checked_at_ms = at_ms;
        apply_crash_backoff(record, at_ms);
        self.trace(
            runtime_id,
            process_generation,
            at_ms,
            RuntimeTraceKind::Stopped,
        );
        Ok(())
    }

    /// Records the completed protocol-aware shutdown of an externally managed
    /// production peer. Calling the generic `shutdown` path for such a process
    /// is rejected so process ownership cannot be silently lost.
    pub fn finish_managed_shutdown(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        let record = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if self.processes.contains_key(runtime_id)
            || record.process_generation != process_generation
            || record.process_id.is_none()
        {
            return Err(SupervisorError::StaleGeneration);
        }
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.lifecycle = RuntimeLifecycle::Stopped;
        record.health = HealthState::Unknown;
        record.process_id = None;
        record.checked_at_ms = at_ms;
        self.trace(
            runtime_id,
            process_generation,
            at_ms,
            RuntimeTraceKind::Stopped,
        );
        Ok(())
    }

    pub fn record_health(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        health: HealthState,
        at_ms: u64,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        self.refresh_exit(runtime_id, at_ms)?;
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if record.process_generation != process_generation || record.process_id.is_none() {
            self.trace(
                runtime_id,
                process_generation,
                at_ms,
                RuntimeTraceKind::StaleEventRejected,
            );
            return Err(SupervisorError::StaleGeneration);
        }
        record.health = health;
        record.lifecycle = match health {
            HealthState::Healthy => RuntimeLifecycle::Ready,
            HealthState::Degraded => RuntimeLifecycle::Degraded,
            HealthState::Unknown => RuntimeLifecycle::Starting,
            HealthState::Unhealthy => RuntimeLifecycle::Failed,
        };
        record.checked_at_ms = at_ms;
        match health {
            HealthState::Healthy => {
                record.restart_attempts = 0;
                record.next_restart_at_ms = None;
                record.recovery_action = None;
            }
            HealthState::Unhealthy => apply_crash_backoff(record, at_ms),
            HealthState::Unknown | HealthState::Degraded => {}
        }
        let kind = match health {
            HealthState::Healthy => RuntimeTraceKind::Ready,
            HealthState::Degraded => RuntimeTraceKind::Degraded,
            HealthState::Unknown | HealthState::Unhealthy => RuntimeTraceKind::Unhealthy,
        };
        self.trace(runtime_id, process_generation, at_ms, kind);
        Ok(())
    }

    pub fn restart(
        &mut self,
        runtime_id: &str,
        at_ms: u64,
        shutdown_timeout: Duration,
    ) -> Result<u64, SupervisorError> {
        self.shutdown(runtime_id, at_ms, shutdown_timeout)?;
        self.start(
            runtime_id,
            at_ms
                .checked_add(1)
                .ok_or(SupervisorError::InvalidTimestamp)?,
        )
    }

    pub fn review_crash_loop(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 {
            return Err(SupervisorError::InvalidTimestamp);
        }
        let record = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        if record.process_generation != process_generation
            || record.process_id.is_some()
            || record.recovery_action != Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop)
        {
            return Err(SupervisorError::RecoveryUnavailable);
        }
        self.bump_state_generation()?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.restart_attempts = 0;
        record.next_restart_at_ms = None;
        record.recovery_action = None;
        record.lifecycle = RuntimeLifecycle::Stopped;
        record.health = HealthState::Unknown;
        record.checked_at_ms = at_ms;
        self.trace(
            runtime_id,
            process_generation,
            at_ms,
            RuntimeTraceKind::RecoveryReviewed,
        );
        Ok(())
    }

    pub fn shutdown(
        &mut self,
        runtime_id: &str,
        at_ms: u64,
        timeout: Duration,
    ) -> Result<(), SupervisorError> {
        if at_ms == 0 || timeout > MAX_SHUTDOWN_TIMEOUT {
            return Err(SupervisorError::InvalidTimestamp);
        }
        let generation = self
            .records
            .get(runtime_id)
            .ok_or(SupervisorError::NotFound)?
            .process_generation;
        if !self.processes.contains_key(runtime_id)
            && self
                .records
                .get(runtime_id)
                .is_some_and(|record| record.process_id.is_some())
        {
            return Err(SupervisorError::ExternallyManaged);
        }
        self.bump_state_generation()?;
        let Some(mut process) = self.processes.remove(runtime_id) else {
            let record = self
                .records
                .get_mut(runtime_id)
                .ok_or(SupervisorError::NotFound)?;
            record.lifecycle = RuntimeLifecycle::Stopped;
            record.health = HealthState::Unknown;
            record.process_id = None;
            record.checked_at_ms = at_ms;
            return Ok(());
        };

        if let Some(record) = self.records.get_mut(runtime_id) {
            record.lifecycle = RuntimeLifecycle::Stopping;
        }
        self.trace(
            runtime_id,
            generation,
            at_ms,
            RuntimeTraceKind::TerminationRequested,
        );
        terminate_group(&mut process, timeout, || {
            self.trace(
                runtime_id,
                generation,
                at_ms,
                RuntimeTraceKind::ForcedTermination,
            );
        })?;
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.lifecycle = RuntimeLifecycle::Stopped;
        record.health = HealthState::Unknown;
        record.process_id = None;
        record.checked_at_ms = at_ms;
        record.last_exit_code = process
            .child
            .try_wait()
            .map_err(SupervisorError::ProcessIo)?
            .and_then(exit_code);
        self.trace(runtime_id, generation, at_ms, RuntimeTraceKind::Stopped);
        Ok(())
    }

    pub fn snapshot(&self) -> SupervisorSnapshot {
        SupervisorSnapshot {
            state_generation: self.state_generation,
            records: self.records.values().cloned().collect(),
            events: self.events.clone(),
            trace_events_dropped: self.trace_events_dropped,
        }
    }

    fn refresh_exit(&mut self, runtime_id: &str, at_ms: u64) -> Result<(), SupervisorError> {
        let Some(process) = self.processes.get_mut(runtime_id) else {
            return Ok(());
        };
        let Some(status) = process
            .child
            .try_wait()
            .map_err(SupervisorError::ProcessIo)?
        else {
            return Ok(());
        };
        self.processes.remove(runtime_id);
        let record = self
            .records
            .get_mut(runtime_id)
            .ok_or(SupervisorError::NotFound)?;
        record.lifecycle = RuntimeLifecycle::Failed;
        record.health = HealthState::Unhealthy;
        record.process_id = None;
        record.checked_at_ms = at_ms;
        record.last_exit_code = exit_code(status);
        apply_crash_backoff(record, at_ms);
        let generation = record.process_generation;
        self.trace(runtime_id, generation, at_ms, RuntimeTraceKind::Exited);
        Ok(())
    }

    fn trace(
        &mut self,
        runtime_id: &str,
        process_generation: u64,
        at_ms: u64,
        kind: RuntimeTraceKind,
    ) {
        self.push_trace(RuntimeTraceEvent {
            runtime_id: runtime_id.into(),
            process_generation,
            at_ms,
            kind,
        });
    }

    fn push_trace(&mut self, event: RuntimeTraceEvent) {
        if self.events.len() == MAX_TRACE_EVENTS {
            self.events.remove(0);
            self.trace_events_dropped = self.trace_events_dropped.saturating_add(1);
        }
        self.events.push(event);
    }

    fn bump_state_generation(&mut self) -> Result<(), SupervisorError> {
        self.state_generation = self
            .state_generation
            .checked_add(1)
            .ok_or(SupervisorError::GenerationExhausted)?;
        Ok(())
    }
}

fn require_restart_budget(record: &RuntimeRecord, at_ms: u64) -> Result<(), SupervisorError> {
    if record.recovery_action == Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop) {
        return Err(SupervisorError::CrashLoopBudgetExceeded);
    }
    if record
        .next_restart_at_ms
        .is_some_and(|next_restart| at_ms < next_restart)
    {
        return Err(SupervisorError::RestartBackoff);
    }
    Ok(())
}

fn validate_restart_state(record: &RuntimeRecord) -> Result<(), SupervisorError> {
    if record.restart_attempts > MAX_AUTOMATIC_RESTART_ATTEMPTS
        || (record.restart_attempts >= MAX_AUTOMATIC_RESTART_ATTEMPTS
            && (record.next_restart_at_ms.is_some()
                || record.recovery_action != Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop)))
        || (record.restart_attempts < MAX_AUTOMATIC_RESTART_ATTEMPTS
            && record.recovery_action.is_some())
    {
        return Err(SupervisorError::InvalidSnapshot);
    }
    Ok(())
}

fn apply_crash_backoff(record: &mut RuntimeRecord, at_ms: u64) {
    record.restart_attempts = record
        .restart_attempts
        .saturating_add(1)
        .min(MAX_AUTOMATIC_RESTART_ATTEMPTS);
    if record.restart_attempts >= MAX_AUTOMATIC_RESTART_ATTEMPTS {
        record.next_restart_at_ms = None;
        record.recovery_action = Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop);
        return;
    }
    let exponent = u32::from(record.restart_attempts.saturating_sub(1));
    let delay_ms = 1_000_u64
        .saturating_mul(2_u64.saturating_pow(exponent))
        .min(MAX_RESTART_BACKOFF_MS);
    record.next_restart_at_ms = Some(at_ms.saturating_add(delay_ms));
    record.recovery_action = None;
}

impl Drop for RuntimeSupervisor {
    fn drop(&mut self) {
        for (_, mut process) in self.processes.drain() {
            let _ = terminate_group(&mut process, Duration::from_millis(100), || {});
        }
    }
}

fn verify_executable(installation: &RuntimeInstallation) -> Result<(), SupervisorError> {
    let metadata =
        fs::symlink_metadata(&installation.executable).map_err(SupervisorError::ProcessIo)?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(SupervisorError::InvalidExecutable);
    }
    let actual = sha256_file(&installation.executable)?;
    if actual != installation.executable_sha256 {
        return Err(SupervisorError::DigestMismatch);
    }
    Ok(())
}

fn prepare_state_namespace(
    installation: &RuntimeInstallation,
    process_generation: u64,
) -> Result<PathBuf, SupervisorError> {
    fs::create_dir_all(&installation.state_namespace).map_err(SupervisorError::ProcessIo)?;
    if fs::symlink_metadata(&installation.state_namespace)
        .map_err(SupervisorError::ProcessIo)?
        .file_type()
        .is_symlink()
    {
        return Err(SupervisorError::InvalidInstallation);
    }
    let canonical_base = installation
        .state_namespace
        .canonicalize()
        .map_err(SupervisorError::ProcessIo)?;
    let namespace = canonical_base
        .join(installation.runtime_kind.as_str())
        .join(&installation.native_version)
        .join(&installation.workspace_id)
        .join(format!("generation-{process_generation}"));
    fs::create_dir_all(&namespace).map_err(SupervisorError::ProcessIo)?;
    let canonical_namespace = namespace
        .canonicalize()
        .map_err(SupervisorError::ProcessIo)?;
    if !canonical_namespace.starts_with(&canonical_base) {
        return Err(SupervisorError::InvalidInstallation);
    }
    Ok(canonical_namespace)
}

pub fn sha256_file(path: &Path) -> Result<String, SupervisorError> {
    let mut file = File::open(path).map_err(SupervisorError::ProcessIo)?;
    let metadata = file.metadata().map_err(SupervisorError::ProcessIo)?;
    if metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(SupervisorError::InvalidExecutable);
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(SupervisorError::ProcessIo)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let digest = digest.finalize();
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    Ok(encoded)
}

fn terminate_group(
    process: &mut ManagedProcess,
    timeout: Duration,
    mut on_force: impl FnMut(),
) -> Result<(), SupervisorError> {
    signal_process_group(process, "TERM")?;
    let deadline = Instant::now() + timeout;
    loop {
        if process
            .child
            .try_wait()
            .map_err(SupervisorError::ProcessIo)?
            .is_some()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    on_force();
    signal_process_group(process, "KILL")?;
    process.child.wait().map_err(SupervisorError::ProcessIo)?;
    Ok(())
}

#[cfg(unix)]
fn signal_process_group(
    process: &mut ManagedProcess,
    signal: &'static str,
) -> Result<(), SupervisorError> {
    let status = Command::new("/bin/kill")
        .arg(format!("-{signal}"))
        .arg(format!("-{}", process.process_group_id))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(SupervisorError::ProcessIo)?;
    if status.success() {
        Ok(())
    } else {
        // A process may have exited between the prior poll and the signal.
        process
            .child
            .try_wait()
            .map_err(SupervisorError::ProcessIo)?
            .map(|_| ())
            .ok_or(SupervisorError::SignalFailed)
    }
}

#[cfg(not(unix))]
fn signal_process_group(
    process: &mut ManagedProcess,
    _signal: &'static str,
) -> Result<(), SupervisorError> {
    process.child.kill().map_err(SupervisorError::ProcessIo)
}

fn exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

fn validate_id(value: &str) -> Result<(), SupervisorError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
    {
        return Err(SupervisorError::InvalidInstallation);
    }
    Ok(())
}

fn validate_version(value: &str) -> Result<(), SupervisorError> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    {
        return Err(SupervisorError::InvalidInstallation);
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), SupervisorError> {
    if value.strip_prefix("sha256:").is_none_or(|digest| {
        digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return Err(SupervisorError::InvalidInstallation);
    }
    Ok(())
}

fn bounded_value(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.contains('\0')
}

fn valid_environment_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn contains_secret_name(value: &str) -> bool {
    ["AUTH", "CREDENTIAL", "KEY", "PASSWORD", "SECRET", "TOKEN"]
        .iter()
        .any(|needle| value.contains(needle))
}

fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.contains("-----begin private key-----")
        || lower.starts_with("bearer ")
        || lower.starts_with("basic ")
        || lower.starts_with("sk-")
        || lower.contains("password=")
        || lower.contains("secret=")
        || lower.contains("token=")
        || lower.contains("api_key=")
        || lower.contains("apikey=")
        || lower.contains("authorization:")
    {
        return true;
    }
    if let Some(authority) = lower
        .find("://")
        .map(|scheme| &lower[scheme.saturating_add(3)..])
    {
        let authority = authority.split('/').next().unwrap_or(authority);
        if authority.contains('@') && authority.contains(':') {
            return true;
        }
    }
    let jwt_parts = value.split('.').collect::<Vec<_>>();
    jwt_parts.len() == 3
        && jwt_parts.iter().all(|part| {
            part.len() >= 8
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
}

#[derive(Debug, Error)]
pub enum SupervisorError {
    #[error("runtime installation is invalid")]
    InvalidInstallation,
    #[error("runtime protocol is unsupported")]
    UnsupportedProtocol,
    #[error("runtime compatibility record is duplicated")]
    DuplicateCompatibility,
    #[error("runtime compatibility record is absent")]
    NoCompatibilityRecord,
    #[error("runtime installation is incompatible")]
    Incompatible,
    #[error("runtime installation was not found")]
    NotFound,
    #[error("runtime supervisor snapshot is invalid")]
    InvalidSnapshot,
    #[error("runtime installation capacity was reached")]
    CapacityExceeded,
    #[error("runtime process is already running")]
    AlreadyRunning,
    #[error("runtime restart is waiting for bounded backoff")]
    RestartBackoff,
    #[error("runtime crash-loop restart budget is exhausted")]
    CrashLoopBudgetExceeded,
    #[error("runtime crash-loop recovery action is unavailable")]
    RecoveryUnavailable,
    #[error("runtime process is owned by the production adapter")]
    ExternallyManaged,
    #[error("runtime process event is stale")]
    StaleGeneration,
    #[error("runtime timestamp or timeout is invalid")]
    InvalidTimestamp,
    #[error("runtime managed-process health is not ready")]
    InvalidHealth,
    #[error("runtime process generation was exhausted")]
    GenerationExhausted,
    #[error("runtime executable is invalid")]
    InvalidExecutable,
    #[error("runtime executable digest does not match the installation record")]
    DigestMismatch,
    #[error("runtime process exited before supervision with code {0:?}")]
    EarlyExit(Option<i32>),
    #[error("runtime process-group signal failed")]
    SignalFailed,
    #[error("runtime process I/O failed: {0}")]
    ProcessIo(#[source] std::io::Error),
}
