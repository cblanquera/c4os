//! Reviewed declarative hook contracts and a bounded macOS out-of-process
//! supervisor. Hook output is proposal data only; this module never applies an
//! effect or grants package code ambient C4OS authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use std::os::unix::process::{CommandExt, ExitStatusExt};
#[cfg(target_os = "macos")]
use std::process::Child;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    EXTENSION_HOOK_PROTOCOL_VERSION, ExtensionError, validate_digest, validate_identifier,
    validate_text,
};

pub const MAX_HOOK_INPUT_BYTES: usize = 64 * 1024;
pub const DEFAULT_HOOK_OUTPUT_BYTES: usize = 64 * 1024;
pub const MAX_HOOK_OUTPUT_BYTES: usize = 1024 * 1024;
pub const DEFAULT_HOOK_TIMEOUT_MS: u64 = 5_000;
pub const MAX_HOOK_TIMEOUT_MS: u64 = 30_000;
// One worker invocation may propose at most one independently authorized
// effect. This keeps application atomic and prevents a mixed proposal batch
// from partially applying before a later item fails closed.
pub const MAX_HOOK_PROPOSALS: usize = 1;
pub const MAX_ACTIVE_HOOK_WORKERS: usize = 256;
pub const HOOK_TERMINATION_GRACE_MS: u64 = 250;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedHookContract {
    pub hook_id: String,
    pub event: String,
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    pub review_digest: String,
    pub timeout_ms: u64,
    pub maximum_output_bytes: usize,
}

impl ReviewedHookContract {
    pub fn validate(&self) -> Result<(), ExtensionError> {
        validate_identifier(&self.hook_id)?;
        validate_identifier(&self.event)?;
        let executable = normalize_relative_path(&self.executable)?;
        if !matches!(
            executable
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("js" | "mjs")
        ) {
            return Err(ExtensionError::HookDenied);
        }
        validate_digest(&self.review_digest)?;
        if self.timeout_ms == 0
            || self.timeout_ms > MAX_HOOK_TIMEOUT_MS
            || self.maximum_output_bytes == 0
            || self.maximum_output_bytes > MAX_HOOK_OUTPUT_BYTES
            || self.arguments.len() > 32
            || self.arguments.iter().any(|argument| {
                argument.is_empty()
                    || argument.len() > 1_024
                    || argument.chars().any(char::is_control)
            })
        {
            return Err(ExtensionError::BoundExceeded);
        }
        Ok(())
    }
}

impl Default for ReviewedHookContract {
    fn default() -> Self {
        Self {
            hook_id: String::new(),
            event: String::new(),
            executable: String::new(),
            arguments: Vec::new(),
            review_digest: String::new(),
            timeout_ms: DEFAULT_HOOK_TIMEOUT_MS,
            maximum_output_bytes: DEFAULT_HOOK_OUTPUT_BYTES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookActivation {
    pub package_id: String,
    pub package_digest: String,
    /// ExtensionService generation that authorized this exact package digest.
    pub generation: u64,
    pub enabled: bool,
    pub explicitly_reviewed: bool,
    pub signature_verified: bool,
    pub revoked: bool,
}

impl HookActivation {
    pub fn validate_for_run(&self) -> Result<(), ExtensionError> {
        validate_identifier(&self.package_id)?;
        validate_digest(&self.package_digest)?;
        if self.generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        if self.revoked {
            return Err(ExtensionError::Revoked);
        }
        if !self.enabled || !self.explicitly_reviewed || !self.signature_verified {
            return Err(ExtensionError::HookDenied);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookEventEnvelope {
    pub protocol_version: u16,
    pub operation_id: String,
    pub package_id: String,
    pub package_digest: String,
    pub event: String,
    pub payload: Value,
}

impl HookEventEnvelope {
    pub fn validate(&self) -> Result<(), ExtensionError> {
        if self.protocol_version != EXTENSION_HOOK_PROTOCOL_VERSION {
            return Err(ExtensionError::InvalidState);
        }
        validate_identifier(&self.operation_id)?;
        validate_identifier(&self.package_id)?;
        validate_digest(&self.package_digest)?;
        validate_identifier(&self.event)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookProposalEnvelope {
    pub protocol_version: u16,
    pub proposals: Vec<HookEffectProposal>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HookEffectProposal {
    /// A declarative proposal class interpreted by C4OS Core. This is not a
    /// permission or an execution capability.
    pub kind: String,
    pub summary: String,
    pub payload: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HookRunResult {
    pub proposals: Vec<HookEffectProposal>,
    pub stderr: String,
    pub duration_ms: u64,
    pub exit_code: i32,
}

#[derive(Clone, Debug)]
pub struct HookSupervisorPolicy {
    /// A C4OS-selected trusted interpreter, not a package-selected native
    /// executable. For the production golden path this is the bundled JS
    /// runtime.
    pub trusted_runtime: PathBuf,
    pub runtime_read_roots: Vec<PathBuf>,
    pub scratch_root: PathBuf,
}

/// Serializable active-worker truth intentionally excludes process IDs. The
/// process group remains private, in-memory supervisor authority and is never
/// restored from persisted state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActiveHookWorkerSnapshot {
    pub package_id: String,
    pub package_digest: String,
    pub generation: u64,
    pub hook_id: String,
    pub operation_id: String,
}

#[derive(Clone, Debug)]
struct ActiveHookProcess {
    snapshot: ActiveHookWorkerSnapshot,
    process_group_id: u32,
}

#[derive(Debug, Default)]
struct ActiveHookRegistryState {
    next_worker_id: u64,
    workers: BTreeMap<u64, ActiveHookProcess>,
    /// Highest lifecycle generation invalidated for each package. A strictly
    /// newer generation may launch without clearing history.
    blocked_through_generation: BTreeMap<String, u64>,
}

#[derive(Debug, Default)]
struct ActiveHookRegistry {
    state: Mutex<ActiveHookRegistryState>,
}

impl ActiveHookRegistry {
    /// First half of the launch barrier. It runs immediately before spawn.
    fn prepare_launch(&self, package_id: &str, generation: u64) -> Result<(), ExtensionError> {
        validate_identifier(package_id)?;
        if generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        let state = self
            .state
            .lock()
            .map_err(|_| ExtensionError::InvalidState)?;
        if generation
            <= state
                .blocked_through_generation
                .get(package_id)
                .copied()
                .unwrap_or(0)
        {
            return Err(ExtensionError::Revoked);
        }
        Ok(())
    }

    fn register(
        self: &Arc<Self>,
        process: ActiveHookProcess,
    ) -> Result<ActiveHookGuard, ExtensionError> {
        validate_identifier(&process.snapshot.package_id)?;
        validate_digest(&process.snapshot.package_digest)?;
        validate_identifier(&process.snapshot.hook_id)?;
        validate_identifier(&process.snapshot.operation_id)?;
        if process.snapshot.generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| ExtensionError::InvalidState)?;
        if process.snapshot.generation
            <= state
                .blocked_through_generation
                .get(&process.snapshot.package_id)
                .copied()
                .unwrap_or(0)
        {
            return Err(ExtensionError::Revoked);
        }
        if state.workers.len() >= MAX_ACTIVE_HOOK_WORKERS || process.process_group_id == 0 {
            return Err(ExtensionError::BoundExceeded);
        }
        if state
            .workers
            .values()
            .any(|worker| worker.process_group_id == process.process_group_id)
        {
            return Err(ExtensionError::Conflict);
        }
        state.next_worker_id = state
            .next_worker_id
            .checked_add(1)
            .ok_or(ExtensionError::BoundExceeded)?;
        let worker_id = state.next_worker_id;
        state.workers.insert(worker_id, process);
        Ok(ActiveHookGuard {
            registry: Arc::clone(self),
            worker_id,
        })
    }

    /// Final barrier check, linearized against lifecycle invalidation, before
    /// the event payload is made available to the child.
    fn authorize_input(&self, worker_id: u64) -> Result<(), ExtensionError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ExtensionError::InvalidState)?;
        let worker = state
            .workers
            .get(&worker_id)
            .ok_or(ExtensionError::Conflict)?;
        if worker.snapshot.generation
            <= state
                .blocked_through_generation
                .get(&worker.snapshot.package_id)
                .copied()
                .unwrap_or(0)
        {
            return Err(ExtensionError::Revoked);
        }
        Ok(())
    }

    fn remove(&self, worker_id: u64) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.workers.remove(&worker_id);
    }

    fn snapshots(&self) -> Result<Vec<ActiveHookWorkerSnapshot>, ExtensionError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ExtensionError::InvalidState)?;
        Ok(state
            .workers
            .values()
            .map(|worker| worker.snapshot.clone())
            .collect())
    }

    fn blocked_through_generation(&self, package_id: &str) -> Result<Option<u64>, ExtensionError> {
        validate_identifier(package_id)?;
        let state = self
            .state
            .lock()
            .map_err(|_| ExtensionError::InvalidState)?;
        Ok(state.blocked_through_generation.get(package_id).copied())
    }

    /// Atomically advances the package barrier and selects only now-invalid
    /// live workers. Termination occurs after the lock is released; concurrent
    /// registration either preceded this selection and is included, or sees
    /// the barrier and fails closed.
    fn invalidate_generation(
        &self,
        package_id: &str,
        generation: u64,
        mut terminate: impl FnMut(u32),
    ) -> Result<usize, ExtensionError> {
        validate_identifier(package_id)?;
        if generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        let process_groups = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| ExtensionError::InvalidState)?;
            let blocked_through = state
                .blocked_through_generation
                .entry(package_id.to_owned())
                .or_insert(generation);
            *blocked_through = (*blocked_through).max(generation);
            let blocked_through = *blocked_through;
            state
                .workers
                .values()
                .filter(|worker| {
                    worker.snapshot.package_id == package_id
                        && worker.snapshot.generation <= blocked_through
                })
                .map(|worker| worker.process_group_id)
                .collect::<Vec<_>>()
        };
        for process_group_id in &process_groups {
            terminate(*process_group_id);
        }
        Ok(process_groups.len())
    }

    fn terminate_matching(
        &self,
        package_id: &str,
        mut terminate: impl FnMut(u32),
    ) -> Result<usize, ExtensionError> {
        validate_identifier(package_id)?;
        let process_groups: Vec<_> = {
            let state = self
                .state
                .lock()
                .map_err(|_| ExtensionError::InvalidState)?;
            state
                .workers
                .values()
                .filter(|worker| worker.snapshot.package_id == package_id)
                .map(|worker| worker.process_group_id)
                .collect()
        };
        for process_group_id in &process_groups {
            terminate(*process_group_id);
        }
        Ok(process_groups.len())
    }
}

#[derive(Debug)]
struct ActiveHookGuard {
    registry: Arc<ActiveHookRegistry>,
    worker_id: u64,
}

impl Drop for ActiveHookGuard {
    fn drop(&mut self) {
        self.registry.remove(self.worker_id);
    }
}

impl ActiveHookGuard {
    fn authorize_input(&self) -> Result<(), ExtensionError> {
        self.registry.authorize_input(self.worker_id)
    }
}

#[derive(Clone, Debug)]
pub struct HookSupervisor {
    policy: HookSupervisorPolicy,
    active: Arc<ActiveHookRegistry>,
}

impl HookSupervisor {
    pub fn new(policy: HookSupervisorPolicy) -> Result<Self, ExtensionError> {
        let trusted_runtime = canonical_regular_file(&policy.trusted_runtime)?;
        reject_symlink_or_non_directory(&policy.scratch_root)?;
        let scratch_root = policy.scratch_root.canonicalize()?;
        let mut runtime_read_roots = Vec::with_capacity(policy.runtime_read_roots.len());
        for root in &policy.runtime_read_roots {
            reject_symlink_or_non_directory(root)?;
            let root = root.canonicalize()?;
            runtime_read_roots.push(root);
        }
        Ok(Self {
            policy: HookSupervisorPolicy {
                trusted_runtime,
                runtime_read_roots,
                scratch_root,
            },
            active: Arc::new(ActiveHookRegistry::default()),
        })
    }

    pub fn active_worker_count(&self) -> Result<u16, ExtensionError> {
        self.active
            .snapshots()?
            .len()
            .try_into()
            .map_err(|_| ExtensionError::BoundExceeded)
    }

    pub fn active_worker_snapshot(&self) -> Result<Vec<ActiveHookWorkerSnapshot>, ExtensionError> {
        self.active.snapshots()
    }

    pub fn blocked_through_generation(
        &self,
        package_id: &str,
    ) -> Result<Option<u64>, ExtensionError> {
        self.active.blocked_through_generation(package_id)
    }

    /// Atomically invalidates this package generation and every older one,
    /// then terminates matching live workers. A strictly newer generation is
    /// still eligible for the prepare/spawn/register barrier.
    pub fn invalidate_package_generation(
        &self,
        package_id: &str,
        generation: u64,
    ) -> Result<u16, ExtensionError> {
        let mut process_groups = Vec::new();
        let count =
            self.active
                .invalidate_generation(package_id, generation, |process_group_id| {
                    process_groups.push(process_group_id)
                })?;
        terminate_process_groups_bounded(&process_groups);
        count.try_into().map_err(|_| ExtensionError::BoundExceeded)
    }

    /// Immediately terminates only live in-memory process groups for the
    /// revoked package. No PID or process-group identity is loaded from disk.
    pub fn terminate_package(&self, package_id: &str) -> Result<u16, ExtensionError> {
        let mut process_groups = Vec::new();
        let count = self
            .active
            .terminate_matching(package_id, |process_group_id| {
                process_groups.push(process_group_id)
            })?;
        terminate_process_groups_bounded(&process_groups);
        count.try_into().map_err(|_| ExtensionError::BoundExceeded)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn run(
        &self,
        _activation: &HookActivation,
        _contract: &ReviewedHookContract,
        _package_root: &Path,
        _event: &HookEventEnvelope,
    ) -> Result<HookRunResult, ExtensionError> {
        Err(ExtensionError::UnsupportedHookTarget)
    }

    #[cfg(target_os = "macos")]
    pub fn run(
        &self,
        activation: &HookActivation,
        contract: &ReviewedHookContract,
        package_root: &Path,
        event: &HookEventEnvelope,
    ) -> Result<HookRunResult, ExtensionError> {
        activation.validate_for_run()?;
        contract.validate()?;
        event.validate()?;
        if event.package_id != activation.package_id
            || event.package_digest != activation.package_digest
            || event.event != contract.event
        {
            return Err(ExtensionError::Conflict);
        }

        reject_symlink_or_non_directory(&package_root)?;
        let package_root = package_root.canonicalize()?;
        let executable = package_root.join(normalize_relative_path(&contract.executable)?);
        let executable = canonical_regular_file(&executable)?;
        ensure_within(&package_root, &executable)?;
        if sha256_file(&executable)? != contract.review_digest {
            return Err(ExtensionError::VerificationFailed);
        }

        let input = serde_json::to_vec(event)?;
        if input.len() > MAX_HOOK_INPUT_BYTES {
            return Err(ExtensionError::BoundExceeded);
        }
        let workspace = tempfile::Builder::new()
            .prefix("hook-")
            .tempdir_in(&self.policy.scratch_root)?;
        set_private_directory(workspace.path())?;
        let profile = build_macos_sandbox_profile(
            &self.policy.trusted_runtime,
            &self.policy.runtime_read_roots,
            &package_root,
            workspace.path(),
        )?;

        let mut command = Command::new("/usr/bin/sandbox-exec");
        command
            .arg("-p")
            .arg(profile)
            .arg(&self.policy.trusted_runtime)
            .arg(&executable)
            .args(&contract.arguments)
            .current_dir(workspace.path())
            .env_clear()
            .env("HOME", workspace.path())
            .env("PATH", "/usr/bin:/bin")
            .env("TMPDIR", workspace.path())
            .env(
                "C4OS_HOOK_PROTOCOL",
                EXTENSION_HOOK_PROTOCOL_VERSION.to_string(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command.process_group(0);
        // First launch-barrier check is intentionally adjacent to spawn. An
        // invalidation after this check is caught by `register`, and one after
        // registration is caught by `authorize_input` or terminates the
        // already registered process group.
        self.active
            .prepare_launch(&activation.package_id, activation.generation)?;
        let started = Instant::now();
        let child = command.spawn()?;
        let process_group_id = child.id();
        let mut child = ManagedHookChild::new(child);
        let active_worker_guard = self.active.register(ActiveHookProcess {
            snapshot: ActiveHookWorkerSnapshot {
                package_id: activation.package_id.clone(),
                package_digest: activation.package_digest.clone(),
                generation: activation.generation,
                hook_id: contract.hook_id.clone(),
                operation_id: event.operation_id.clone(),
            },
            process_group_id,
        })?;
        active_worker_guard.authorize_input()?;
        child
            .child
            .stdin
            .take()
            .ok_or(ExtensionError::HookDenied)?
            .write_all(&input)?;
        let stdout = child
            .child
            .stdout
            .take()
            .ok_or(ExtensionError::HookDenied)?;
        let stderr = child
            .child
            .stderr
            .take()
            .ok_or(ExtensionError::HookDenied)?;
        let limit = contract.maximum_output_bytes;
        let stdout_reader = thread::spawn(move || read_bounded(stdout, limit));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, limit));

        let deadline = started + Duration::from_millis(contract.timeout_ms);
        let status = loop {
            if let Some(status) = child.child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                terminate_process_group_bounded(process_group_id);
                let _ = child.child.wait();
                child.completed = true;
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(ExtensionError::HookTimedOut);
            }
            thread::sleep(Duration::from_millis(10));
        };
        child.completed = true;
        drop(active_worker_guard);
        let (stdout, stdout_exceeded) = stdout_reader
            .join()
            .map_err(|_| ExtensionError::HookDenied)??;
        let (stderr, stderr_exceeded) = stderr_reader
            .join()
            .map_err(|_| ExtensionError::HookDenied)??;
        if stdout_exceeded
            || stderr_exceeded
            || stdout.len().saturating_add(stderr.len()) > contract.maximum_output_bytes
        {
            return Err(ExtensionError::HookOutputExceeded);
        }
        let stderr = String::from_utf8(stderr).map_err(|_| ExtensionError::HookDenied)?;
        if !status.success() {
            return Err(ExtensionError::HookDenied);
        }
        let proposals = parse_proposals(&stdout)?;
        Ok(HookRunResult {
            proposals,
            stderr,
            duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
            exit_code: status
                .code()
                .unwrap_or_else(|| status.signal().unwrap_or(-1)),
        })
    }
}

pub fn parse_proposals(bytes: &[u8]) -> Result<Vec<HookEffectProposal>, ExtensionError> {
    if bytes.is_empty() || bytes.len() > MAX_HOOK_OUTPUT_BYTES {
        return Err(ExtensionError::HookDenied);
    }
    let envelope: HookProposalEnvelope = serde_json::from_slice(bytes)?;
    if envelope.protocol_version != EXTENSION_HOOK_PROTOCOL_VERSION
        || envelope.proposals.len() > MAX_HOOK_PROPOSALS
    {
        return Err(ExtensionError::InvalidState);
    }
    for proposal in &envelope.proposals {
        validate_identifier(&proposal.kind)?;
        validate_text(&proposal.summary)?;
    }
    Ok(envelope.proposals)
}

#[cfg(target_os = "macos")]
fn build_macos_sandbox_profile(
    trusted_runtime: &Path,
    runtime_read_roots: &[PathBuf],
    package_root: &Path,
    workspace: &Path,
) -> Result<String, ExtensionError> {
    let mut profile = vec![
        "(version 1)".to_owned(),
        "(import \"system.sb\")".to_owned(),
        "(deny network*)".to_owned(),
        "(deny file-write*)".to_owned(),
        "(deny process-fork)".to_owned(),
        "(deny process-exec)".to_owned(),
        format!(
            "(allow process-exec (literal \"{}\"))",
            escape_profile_path(trusted_runtime)?
        ),
        format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_profile_path(package_root)?
        ),
        format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_profile_path(workspace)?
        ),
        format!(
            "(allow file-write* (subpath \"{}\"))",
            escape_profile_path(workspace)?
        ),
    ];
    for root in runtime_read_roots {
        profile.push(format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_profile_path(root)?
        ));
    }
    let mut ancestor_metadata = BTreeSet::new();
    for path in std::iter::once(trusted_runtime)
        .chain(runtime_read_roots.iter().map(PathBuf::as_path))
        .chain([package_root, workspace])
    {
        ancestor_metadata.extend(path.ancestors().skip(1).map(Path::to_path_buf));
    }
    for ancestor in ancestor_metadata {
        profile.push(format!(
            "(allow file-read-metadata (literal \"{}\"))",
            escape_profile_path(&ancestor)?
        ));
    }
    Ok(profile.join("\n"))
}

#[cfg(target_os = "macos")]
fn escape_profile_path(path: &Path) -> Result<String, ExtensionError> {
    let value = path.to_str().ok_or(ExtensionError::InvalidInput)?;
    if ['\n', '\r', '\0']
        .iter()
        .any(|character| value.contains(*character))
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct ManagedHookChild {
    child: Child,
    process_group_id: u32,
    completed: bool,
}

#[cfg(target_os = "macos")]
impl ManagedHookChild {
    fn new(child: Child) -> Self {
        let process_group_id = child.id();
        Self {
            child,
            process_group_id,
            completed: false,
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for ManagedHookChild {
    fn drop(&mut self) {
        if !self.completed {
            terminate_process_group_bounded(self.process_group_id);
            let _ = self.child.wait();
        }
    }
}

#[cfg(target_os = "macos")]
fn terminate_process_group_bounded(process_group_id: u32) {
    terminate_process_groups_bounded(&[process_group_id]);
}

#[cfg(target_os = "macos")]
fn terminate_process_groups_bounded(process_group_ids: &[u32]) {
    let process_group_ids: Vec<i32> = process_group_ids
        .iter()
        .filter_map(|process_group_id| i32::try_from(*process_group_id).ok())
        .filter(|process_group_id| *process_group_id > 0)
        .collect();
    // Signal every invalidated worker before waiting for any one worker, so a
    // large package generation cannot delay revocation of its later workers.
    for process_group_id in &process_group_ids {
        // SAFETY: every group was registered immediately from a child spawned
        // by this in-memory supervisor; no persisted PID is trusted.
        unsafe {
            libc::kill(-*process_group_id, libc::SIGTERM);
        }
    }
    let deadline = Instant::now() + Duration::from_millis(HOOK_TERMINATION_GRACE_MS);
    while Instant::now() < deadline
        && process_group_ids
            .iter()
            .any(|process_group_id| process_group_exists(*process_group_id))
    {
        thread::sleep(Duration::from_millis(10));
    }
    for process_group_id in process_group_ids {
        if process_group_exists(process_group_id) {
            // SAFETY: this is the same registered in-memory process group and
            // bounded graceful termination did not remove it.
            unsafe {
                libc::kill(-process_group_id, libc::SIGKILL);
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn process_group_exists(process_group_id: i32) -> bool {
    // SAFETY: signal zero performs an existence check without delivering a
    // signal. The group ID is a positive value created by `Command::spawn`.
    if unsafe { libc::kill(-process_group_id, 0) } == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(not(target_os = "macos"))]
fn terminate_process_group_bounded(_process_group_id: u32) {}

#[cfg(not(target_os = "macos"))]
fn terminate_process_groups_bounded(_process_group_ids: &[u32]) {}

fn read_bounded(mut reader: impl Read, maximum: usize) -> Result<(Vec<u8>, bool), ExtensionError> {
    let mut output = Vec::with_capacity(maximum.min(8 * 1024));
    let mut exceeded = false;
    let mut buffer = [0u8; 8 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = maximum.saturating_sub(output.len());
        output.extend_from_slice(&buffer[..read.min(remaining)]);
        if read > remaining {
            exceeded = true;
        }
    }
    Ok((output, exceeded))
}

fn canonical_regular_file(path: &Path) -> Result<PathBuf, ExtensionError> {
    reject_symlink_or_non_file(path)?;
    let path = path.canonicalize()?;
    reject_symlink_or_non_file(&path)?;
    Ok(path)
}

fn sha256_file(path: &Path) -> Result<String, ExtensionError> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > 8 * 1024 * 1024 {
        return Err(ExtensionError::BoundExceeded);
    }
    let bytes = fs::read(path)?;
    Ok(super::sha256_prefixed(&bytes))
}

fn normalize_relative_path(value: &str) -> Result<PathBuf, ExtensionError> {
    if value.is_empty() || value.len() > 1_024 || value.contains('\\') {
        return Err(ExtensionError::InvalidInput);
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(ExtensionError::InvalidInput);
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => normalized.push(segment),
            _ => return Err(ExtensionError::InvalidInput),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(normalized)
}

fn ensure_within(root: &Path, candidate: &Path) -> Result<(), ExtensionError> {
    if !candidate.starts_with(root) {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn reject_symlink_or_non_file(path: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn reject_symlink_or_non_directory(path: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn set_private_directory(path: &Path) -> Result<(), ExtensionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_requires_all_trust_gates() {
        let mut activation = HookActivation {
            package_id: "sample".to_owned(),
            package_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            generation: 1,
            enabled: true,
            explicitly_reviewed: false,
            signature_verified: true,
            revoked: false,
        };
        assert!(matches!(
            activation.validate_for_run(),
            Err(ExtensionError::HookDenied)
        ));
        activation.explicitly_reviewed = true;
        assert!(activation.validate_for_run().is_ok());
        activation.revoked = true;
        assert!(matches!(
            activation.validate_for_run(),
            Err(ExtensionError::Revoked)
        ));
    }

    #[test]
    fn proposals_are_strict_bounded_data() {
        let valid = br#"{
            "protocolVersion": 1,
            "proposals": [{"kind":"write-file","summary":"Update the changelog","payload":{"path":"CHANGELOG.md"}}]
        }"#;
        let proposals = parse_proposals(valid).expect("proposals");
        assert_eq!(proposals.len(), 1);
        let unknown = br#"{"protocolVersion":1,"proposals":[],"authority":true}"#;
        assert!(parse_proposals(unknown).is_err());
    }

    #[test]
    fn reviewed_executable_digest_detects_mutation() {
        let root = tempfile::tempdir().expect("tempdir");
        let hook = root.path().join("hook.mjs");
        fs::write(&hook, "console.log('{}')").expect("hook");
        let digest = sha256_file(&hook).expect("digest");
        fs::write(&hook, "console.log('changed')").expect("mutate");
        assert_ne!(digest, sha256_file(&hook).expect("changed digest"));
    }

    fn active_process(
        package_id: &str,
        digest_byte: char,
        generation: u64,
        process_group_id: u32,
    ) -> ActiveHookProcess {
        ActiveHookProcess {
            snapshot: ActiveHookWorkerSnapshot {
                package_id: package_id.to_owned(),
                package_digest: format!("sha256:{}", digest_byte.to_string().repeat(64)),
                generation,
                hook_id: format!("hook-{generation}"),
                operation_id: format!("operation-{generation}"),
            },
            process_group_id,
        }
    }

    #[test]
    fn active_registry_is_shared_and_raii_removes_exact_worker() {
        let registry = Arc::new(ActiveHookRegistry::default());
        let first = registry
            .register(active_process("package-a", 'a', 1, 101))
            .expect("first");
        let second = registry
            .register(active_process("package-b", 'b', 2, 202))
            .expect("second");
        let shared = Arc::clone(&registry);
        let snapshots = thread::spawn(move || shared.snapshots().expect("snapshot"))
            .join()
            .expect("snapshot thread");
        assert_eq!(snapshots.len(), 2);
        assert_eq!(snapshots[0].package_id, "package-a");

        drop(first);
        let snapshots = registry.snapshots().expect("after drop");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].package_id, "package-b");
        drop(second);
        assert!(registry.snapshots().expect("empty").is_empty());
    }

    #[test]
    fn termination_targets_only_matching_live_package_groups() {
        let registry = Arc::new(ActiveHookRegistry::default());
        let first = registry
            .register(active_process("package-a", 'a', 7, 301))
            .expect("first");
        let second = registry
            .register(active_process("package-a", 'b', 8, 302))
            .expect("second");
        let other = registry
            .register(active_process("package-b", 'c', 9, 401))
            .expect("other");
        let mut terminated = Vec::new();
        let count = registry
            .terminate_matching("package-a", |process_group| terminated.push(process_group))
            .expect("terminate matching");
        assert_eq!(count, 2);
        assert_eq!(terminated, vec![301, 302]);
        assert_eq!(registry.snapshots().expect("still active").len(), 3);
        assert!(registry.terminate_matching("../invalid", |_| {}).is_err());
        drop((first, second, other));
    }

    #[test]
    fn launch_barrier_blocks_invalidated_generation_and_allows_strictly_newer() {
        let registry = Arc::new(ActiveHookRegistry::default());
        registry
            .prepare_launch("package-a", 5)
            .expect("generation five initially allowed");
        assert_eq!(
            registry
                .invalidate_generation("package-a", 5, |_| {})
                .expect("invalidate"),
            0
        );
        assert_eq!(
            registry
                .blocked_through_generation("package-a")
                .expect("barrier"),
            Some(5)
        );
        assert!(matches!(
            registry.prepare_launch("package-a", 5),
            Err(ExtensionError::Revoked)
        ));
        assert!(matches!(
            registry.prepare_launch("package-a", 4),
            Err(ExtensionError::Revoked)
        ));
        registry
            .prepare_launch("package-a", 6)
            .expect("strictly newer generation");
        registry
            .invalidate_generation("package-a", 3, |_| {})
            .expect("older invalidation is monotonic");
        assert_eq!(
            registry
                .blocked_through_generation("package-a")
                .expect("barrier"),
            Some(5)
        );
    }

    #[test]
    fn launch_barrier_closes_spawn_register_and_pre_input_races() {
        let registry = Arc::new(ActiveHookRegistry::default());

        // Models prepare -> spawn -> concurrent invalidation -> register. The
        // registration recheck rejects, so ManagedHookChild kills the child
        // before the caller can authorize or write input.
        registry
            .prepare_launch("package-a", 7)
            .expect("prepare seven");
        registry
            .invalidate_generation("package-a", 7, |_| {})
            .expect("invalidate during spawn");
        assert!(matches!(
            registry.register(active_process("package-a", 'a', 7, 501)),
            Err(ExtensionError::Revoked)
        ));

        // A newer generation may register. If invalidation wins after
        // registration but before input authorization, the worker is selected
        // for termination and authorization fails closed.
        registry
            .prepare_launch("package-a", 8)
            .expect("prepare eight");
        let worker = registry
            .register(active_process("package-a", 'b', 8, 502))
            .expect("register eight");
        let mut terminated = Vec::new();
        assert_eq!(
            registry
                .invalidate_generation("package-a", 8, |process_group| {
                    terminated.push(process_group)
                })
                .expect("invalidate before input"),
            1
        );
        assert_eq!(terminated, vec![502]);
        assert!(matches!(
            worker.authorize_input(),
            Err(ExtensionError::Revoked)
        ));
        drop(worker);

        registry
            .prepare_launch("package-a", 9)
            .expect("generation nine remains allowed");
        registry
            .prepare_launch("package-b", 1)
            .expect("other package remains isolated");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn sandbox_profile_denies_network_and_limits_writes() {
        let runtime = Path::new("/usr/bin/node");
        let package = Path::new("/private/tmp/package");
        let workspace = Path::new("/private/tmp/workspace");
        let profile =
            build_macos_sandbox_profile(runtime, &[], package, workspace).expect("profile");
        assert!(profile.contains("(deny network*)"));
        assert!(profile.contains("(deny file-write*)"));
        assert!(profile.contains("(deny process-fork)"));
        assert!(profile.contains("(deny process-exec)"));
        assert!(profile.contains("/private/tmp/workspace"));
        assert!(profile.contains("file-read-metadata (literal \"/private\")"));
        assert!(!profile.contains("/Users/"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires the project-owned pinned Node runtime asset"]
    fn native_sandbox_denies_network_writes_detached_children_and_bounds_workers() {
        use std::net::TcpListener;

        let temporary = tempfile::tempdir().expect("temporary hook fixture");
        let package_root = temporary.path().join("package");
        let scratch_root = temporary.path().join("scratch");
        fs::create_dir(&package_root).expect("package root");
        fs::create_dir(&scratch_root).expect("scratch root");
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let runtime_root = repository_root.join(".build/app/c4os-runtime-assets");
        let supervisor = HookSupervisor::new(HookSupervisorPolicy {
            trusted_runtime: runtime_root.join("node"),
            runtime_read_roots: vec![runtime_root],
            scratch_root,
        })
        .expect("native supervisor");
        let activation = HookActivation {
            package_id: "hostile-hook".into(),
            package_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            generation: 1,
            enabled: true,
            explicitly_reviewed: true,
            signature_verified: true,
            revoked: false,
        };
        let event = HookEventEnvelope {
            protocol_version: EXTENSION_HOOK_PROTOCOL_VERSION,
            operation_id: "hostile-native-evidence".into(),
            package_id: activation.package_id.clone(),
            package_digest: activation.package_digest.clone(),
            event: "before-turn".into(),
            payload: serde_json::json!({"bounded": true}),
        };
        let write_hook = |name: &str, source: &str, timeout_ms, maximum_output_bytes| {
            let path = package_root.join(name);
            fs::write(&path, source).expect("hook source");
            ReviewedHookContract {
                hook_id: name.trim_end_matches(".mjs").replace('_', "-"),
                event: "before-turn".into(),
                executable: name.into(),
                arguments: Vec::new(),
                review_digest: sha256_file(&path).expect("review digest"),
                timeout_ms,
                maximum_output_bytes,
            }
        };

        let listener = TcpListener::bind("127.0.0.1:0").expect("local listener");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let port = listener.local_addr().unwrap().port();
        let outside_sentinel = temporary.path().join("outside-write-sentinel");
        let network_and_write = write_hook(
            "network-write.mjs",
            &format!(
                r#"const fs=require('node:fs'); const net=require('node:net');
let escaped=false; try {{ fs.writeFileSync({:?}, 'escape'); escaped=true; }} catch {{}}
let done=false; const finish=(network)=>{{ if(done)return; done=true; process.stdout.write(JSON.stringify({{protocolVersion:1,proposals:(escaped||network)?[{{kind:'context.annotation',summary:'sandbox escape',payload:{{text:'escaped'}}}}]:[]}})); }};
const socket=net.connect({{host:'127.0.0.1',port:{port}}}); socket.on('connect',()=>{{socket.destroy();finish(true);}}); socket.on('error',()=>finish(false)); setTimeout(()=>finish(false),1000);"#,
                outside_sentinel
            ),
            3_000,
            16 * 1_024,
        );
        match supervisor.run(&activation, &network_and_write, &package_root, &event) {
            Ok(result) => assert!(result.proposals.is_empty()),
            Err(ExtensionError::HookDenied) => {}
            Err(error) => panic!("unexpected network/write sandbox result: {error}"),
        }
        assert!(!outside_sentinel.exists());
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );

        let detached_sentinel = temporary.path().join("detached-child-sentinel");
        let detached = write_hook(
            "detached-child.mjs",
            &format!(
                r#"const cp=require('node:child_process'); let done=false; const finish=(spawned)=>{{if(done)return;done=true;process.stdout.write(JSON.stringify({{protocolVersion:1,proposals:spawned?[{{kind:'context.annotation',summary:'spawned',payload:{{text:'spawned'}}}}]:[]}}));}};
try {{ const child=cp.spawn('/bin/sh',['-c',{}],{{detached:true,stdio:'ignore'}}); child.on('spawn',()=>finish(true)); child.on('error',()=>finish(false)); child.unref(); }} catch {{ finish(false); }} setTimeout(()=>finish(false),1000);"#,
                serde_json::to_string(&format!("sleep 0.2; touch {}", detached_sentinel.display()))
                    .unwrap()
            ),
            3_000,
            16 * 1_024,
        );
        match supervisor.run(&activation, &detached, &package_root, &event) {
            Ok(result) => assert!(result.proposals.is_empty()),
            Err(ExtensionError::HookDenied) => {}
            Err(error) => panic!("unexpected detached-child sandbox result: {error}"),
        }
        thread::sleep(Duration::from_millis(400));
        assert!(!detached_sentinel.exists());

        let timeout = write_hook("timeout.mjs", "while (true) {}", 100, 4 * 1_024);
        assert!(matches!(
            supervisor.run(&activation, &timeout, &package_root, &event),
            Err(ExtensionError::HookTimedOut)
        ));
        let output = write_hook(
            "output.mjs",
            "process.stdout.write('x'.repeat(8192));",
            2_000,
            1_024,
        );
        assert!(matches!(
            supervisor.run(&activation, &output, &package_root, &event),
            Err(ExtensionError::HookOutputExceeded)
        ));
        assert_eq!(supervisor.active_worker_count().unwrap(), 0);
    }
}
