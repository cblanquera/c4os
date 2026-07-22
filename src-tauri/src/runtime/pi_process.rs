//! Spawned Node process boundary for the Pi SDK sidecar.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::{Builder as TempDirBuilder, TempDir};
use thiserror::Error;

use crate::runtime::pi::{PI_MAX_LINE_BYTES, PiLaunchSpec, PiSidecarManifest, PiSidecarRunner};
use crate::runtime::supervisor::sha256_file;
use crate::security::credentials::{CredentialVaultError, OperationCredentialLease};

const MAX_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_CREDENTIAL_SECRET_BYTES: usize = 64 * 1024;
const MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES: usize = 256 * 1024;
const CREDENTIAL_FRAME_MAGIC: &[u8; 8] = b"C4OSCRED";
const CREDENTIAL_FRAME_SCHEMA_VERSION: u8 = 1;
const PI_DEPENDENCY_TREE_SCHEMA: &[u8] = b"c4os.pi.dependency-tree.v1\0";
const PI_MAX_DEPENDENCY_ENTRIES: usize = 50_000;
const PI_MAX_DEPENDENCY_BYTES: u64 = 384 * 1024 * 1024;
const PI_MAX_DEPENDENCY_FILE_BYTES: u64 = 64 * 1024 * 1024;
const PI_DEPENDENCY_TREE_SHA256: &str =
    "sha256:f42d94fbecf198d004e91a456c8f0f9e5339eef2cfdf49c880110b310d219a06";

const PI_LOCKED_PACKAGES: [(&str, &str, &str); 4] = [
    (
        "@earendil-works/pi-coding-agent",
        "0.80.10",
        "sha512-aL4apbupCHiVLSXASXvRzH4Q2vmtfrDa+0s909CJuVu/GgGylbDzr7oyF1mPmip5E+VxYYxKWmph4hV04wUcQg==",
    ),
    (
        "@earendil-works/pi-agent-core",
        "0.80.10",
        "sha512-nwnOR3SuLYGRFfyQm8ri4Nj5VGVAvAM9GuqQd3u7BUQj0d6hmD2F8w7OHAAjThE3CuySIdM+v8E22QJG6/RfCg==",
    ),
    (
        "@earendil-works/pi-ai",
        "0.80.10",
        "sha512-Moe/H8c87yacDGK9dPbWphZNjVsrb3nTrIHycOQJAkFEnY9PYxOOd74+ny44kATfPU9Dm7aTHefar3pZF+UKUA==",
    ),
    (
        "typebox",
        "1.1.38",
        "sha512-pZ0aQPmMmXoUvSbeuWf/Hzsc+avNw/Zd6VeE8CFgkVGWyuHPJvqeJJDeJqLve+K70LvjYIoleGcoJHPT17cWoA==",
    ),
];

/// Compiled trust pins for every local source file in the Pi sidecar graph.
///
/// `package-lock.json` preserves the registry integrity records for the installed
/// dependency graph. The source pins prevent a mutable local sidecar file from
/// being substituted after the product manifest was loaded but before spawn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PiSidecarIntegrity;

impl PiSidecarIntegrity {
    const PINNED_FILES: [(&'static str, &'static str, u64); 7] = [
        (
            "main.mjs",
            "sha256:3ad040a7879c3a5b91e827948f67b7b7d98cc41d76f8a83edfd45f91d24951e0",
            256 * 1024,
        ),
        (
            "adapter.mjs",
            "sha256:f4baf2dddfac67187746514e9bb9f95534bb069910556cff0b7afa0c498e1774",
            1024 * 1024,
        ),
        (
            "pi-sdk-driver.mjs",
            "sha256:276518a383d4963bbe9bd0422cf10559314327ca22b369056ef55cebeaf18455",
            512 * 1024,
        ),
        (
            "protocol.mjs",
            "sha256:43aba77c9cdb7b2d264086ec78713ca2bdfa706f1799664bdc00f09178b026f7",
            512 * 1024,
        ),
        (
            "sidecar-manifest.json",
            "sha256:e99e7f47c0a2a4f26a36a654294e10cb26fd7287359bd57207ec68edea8d241a",
            64 * 1024,
        ),
        (
            "package.json",
            "sha256:23beb9838883a4d53a965f23bd1010ffee4724aa52b50ed3e946d24bdbfa2645",
            64 * 1024,
        ),
        (
            "package-lock.json",
            "sha256:d4597e4afdfb75bd073d7ee7871f81afafa025e6ec7bb05d982e58c16b37e59f",
            8 * 1024 * 1024,
        ),
    ];

    /// Verifies the immutable sidecar source and lock graph before any process
    /// or credential descriptor is created.
    pub fn verify(sidecar_root: &Path) -> Result<(), PiProcessError> {
        let root = sidecar_root
            .canonicalize()
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if !root.is_dir() {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        for (relative, expected_sha256, max_bytes) in Self::PINNED_FILES {
            let unresolved = sidecar_root.join(relative);
            let unresolved_metadata = fs::symlink_metadata(&unresolved)
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            if !unresolved_metadata.file_type().is_file()
                || unresolved_metadata.file_type().is_symlink()
            {
                return Err(PiProcessError::InvalidSidecarIntegrity);
            }
            let candidate = unresolved
                .canonicalize()
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            let metadata = candidate
                .metadata()
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            if !candidate.starts_with(&root)
                || !metadata.is_file()
                || metadata.len() == 0
                || metadata.len() > max_bytes
                || sha256_file(&candidate).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?
                    != expected_sha256
            {
                return Err(PiProcessError::InvalidSidecarIntegrity);
            }
        }
        verify_package_lock(sidecar_root)?;
        if Self::dependency_tree_sha256(sidecar_root)? != PI_DEPENDENCY_TREE_SHA256 {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        Ok(())
    }

    /// Returns the deterministic digest of the complete installed dependency
    /// tree after enforcing its production traversal bounds and path rules.
    pub fn dependency_tree_sha256(sidecar_root: &Path) -> Result<String, PiProcessError> {
        let tree = sidecar_root.join("node_modules");
        let tree_metadata =
            fs::symlink_metadata(&tree).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if !tree_metadata.file_type().is_dir() || tree_metadata.file_type().is_symlink() {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        let tree = tree
            .canonicalize()
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        let root = sidecar_root
            .canonicalize()
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if !tree.starts_with(&root) {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }

        let mut paths = collect_dependency_entries(&tree)?;
        paths.sort_by(|left, right| {
            left.as_os_str()
                .as_bytes()
                .cmp(right.as_os_str().as_bytes())
        });
        let mut digest = Sha256::new();
        digest.update(PI_DEPENDENCY_TREE_SCHEMA);
        let mut total_bytes = 0_u64;
        for relative in paths {
            let candidate = tree.join(&relative);
            let metadata = fs::symlink_metadata(&candidate)
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            hash_path(&mut digest, &relative);
            if metadata.file_type().is_symlink() {
                let target = candidate
                    .canonicalize()
                    .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                if !target.starts_with(&tree) || !target.is_file() {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                let link = fs::read_link(&candidate)
                    .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                let link_bytes = link.as_os_str().as_bytes();
                total_bytes = total_bytes
                    .checked_add(link_bytes.len() as u64)
                    .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
                if total_bytes > PI_MAX_DEPENDENCY_BYTES {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                digest.update(b"L");
                digest.update((link_bytes.len() as u64).to_be_bytes());
                digest.update(link_bytes);
            } else if metadata.is_dir() {
                let canonical = candidate
                    .canonicalize()
                    .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                if !canonical.starts_with(&tree) {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                digest.update(b"D");
            } else if metadata.is_file() {
                if metadata.len() > PI_MAX_DEPENDENCY_FILE_BYTES {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                total_bytes = total_bytes
                    .checked_add(metadata.len())
                    .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
                if total_bytes > PI_MAX_DEPENDENCY_BYTES {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                let canonical = candidate
                    .canonicalize()
                    .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                if !canonical.starts_with(&tree) || !canonical.is_file() {
                    return Err(PiProcessError::InvalidSidecarIntegrity);
                }
                digest.update(b"F");
                digest.update(metadata.len().to_be_bytes());
                let mut file =
                    File::open(&canonical).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                let mut buffer = [0_u8; 64 * 1024];
                loop {
                    let read = file
                        .read(&mut buffer)
                        .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
                    if read == 0 {
                        break;
                    }
                    digest.update(&buffer[..read]);
                }
            } else {
                return Err(PiProcessError::InvalidSidecarIntegrity);
            }
        }
        let mut encoded = String::from("sha256:");
        for byte in digest.finalize() {
            write!(&mut encoded, "{byte:02x}")
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        }
        Ok(encoded)
    }
}

fn collect_dependency_entries(tree: &Path) -> Result<Vec<std::path::PathBuf>, PiProcessError> {
    let mut entries = Vec::new();
    let mut directories = vec![tree.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in
            fs::read_dir(&directory).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?
        {
            let entry = entry.map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            let path = entry.path();
            let relative = path
                .strip_prefix(tree)
                .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?
                .to_path_buf();
            entries.push(relative);
            if entries.len() > PI_MAX_DEPENDENCY_ENTRIES {
                return Err(PiProcessError::InvalidSidecarIntegrity);
            }
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                directories.push(path);
            }
        }
    }
    Ok(entries)
}

fn hash_path(digest: &mut Sha256, relative: &Path) {
    let bytes = relative.as_os_str().as_bytes();
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

fn verify_package_lock(sidecar_root: &Path) -> Result<(), PiProcessError> {
    let bytes = fs::read(sidecar_root.join("package-lock.json"))
        .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    let lock: Value =
        serde_json::from_slice(&bytes).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    if lock.get("name").and_then(Value::as_str) != Some("@c4os/pi-sidecar")
        || lock.get("version").and_then(Value::as_str) != Some("0.1.0")
        || lock.get("lockfileVersion").and_then(Value::as_u64) != Some(3)
        || lock.get("requires").and_then(Value::as_bool) != Some(true)
    {
        return Err(PiProcessError::InvalidSidecarIntegrity);
    }
    let packages = lock
        .get("packages")
        .and_then(Value::as_object)
        .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
    let root_dependencies = packages
        .get("")
        .and_then(|package| package.get("dependencies"))
        .and_then(Value::as_object)
        .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
    for (name, version, integrity) in PI_LOCKED_PACKAGES {
        if root_dependencies.get(name).and_then(Value::as_str) != Some(version) {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        let path = format!("node_modules/{name}");
        let locked = packages
            .get(&path)
            .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
        if locked.get("version").and_then(Value::as_str) != Some(version)
            || locked.get("integrity").and_then(Value::as_str) != Some(integrity)
        {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        let installed = fs::read(sidecar_root.join(&path).join("package.json"))
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if installed.len() > 1024 * 1024 {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        let installed: Value = serde_json::from_slice(&installed)
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if installed.get("name").and_then(Value::as_str) != Some(name)
            || installed.get("version").and_then(Value::as_str) != Some(version)
        {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
    }
    let coding_agent =
        fs::read(sidecar_root.join("node_modules/@earendil-works/pi-coding-agent/package.json"))
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    let coding_agent: Value = serde_json::from_slice(&coding_agent)
        .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    let coding_dependencies = coding_agent
        .get("dependencies")
        .and_then(Value::as_object)
        .ok_or(PiProcessError::InvalidSidecarIntegrity)?;
    if coding_dependencies
        .get("@earendil-works/pi-agent-core")
        .and_then(Value::as_str)
        != Some("^0.80.10")
        || coding_dependencies
            .get("@earendil-works/pi-ai")
            .and_then(Value::as_str)
            != Some("^0.80.10")
    {
        return Err(PiProcessError::InvalidSidecarIntegrity);
    }
    Ok(())
}

pub struct SpawnedPiRunner {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Result<String, String>>,
    reader: Option<JoinHandle<()>>,
    pending: VecDeque<String>,
    credential_channel: Option<UnixStream>,
    process_group_id: u32,
    exchange_timeout: Duration,
    _sidecar_snapshot: Option<TempDir>,
}

struct VerifiedPiSidecarSnapshot {
    temporary: TempDir,
    root: PathBuf,
}

impl VerifiedPiSidecarSnapshot {
    fn root(&self) -> &Path {
        &self.root
    }
}

/// Copies the verified graph into a process-private directory and verifies the
/// copy again. The launched Node process never reads the mutable installation
/// path, closing the verify-to-import substitution window for both source and
/// lazy dependency imports.
fn snapshot_verified_sidecar(
    sidecar_root: &Path,
) -> Result<VerifiedPiSidecarSnapshot, PiProcessError> {
    PiSidecarIntegrity::verify(sidecar_root)?;
    let temporary = TempDirBuilder::new()
        .prefix("c4os-pi-sidecar-")
        .tempdir()
        .map_err(PiProcessError::Io)?;
    let root = temporary.path().join("sidecar");
    fs::create_dir(&root).map_err(PiProcessError::Io)?;
    let root = root
        .canonicalize()
        .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    for (relative, _, max_bytes) in PiSidecarIntegrity::PINNED_FILES {
        copy_bounded_regular_file(
            &sidecar_root.join(relative),
            &root.join(relative),
            max_bytes,
            true,
        )?;
    }
    copy_dependency_snapshot(
        &sidecar_root.join("node_modules"),
        &root.join("node_modules"),
    )?;
    PiSidecarIntegrity::verify(&root)?;
    Ok(VerifiedPiSidecarSnapshot { temporary, root })
}

fn copy_bounded_regular_file(
    source: &Path,
    destination: &Path,
    max_bytes: u64,
    require_nonempty: bool,
) -> Result<(), PiProcessError> {
    let metadata = fs::symlink_metadata(source).map_err(PiProcessError::Io)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || (require_nonempty && metadata.len() == 0)
        || metadata.len() > max_bytes
    {
        return Err(PiProcessError::InvalidSidecarIntegrity);
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(PiProcessError::Io)?;
    }
    let input = File::open(source).map_err(PiProcessError::Io)?;
    let mut output = File::create(destination).map_err(PiProcessError::Io)?;
    let mut bounded = input.take(max_bytes.saturating_add(1));
    let copied = std::io::copy(&mut bounded, &mut output).map_err(PiProcessError::Io)?;
    if copied != metadata.len() || copied > max_bytes {
        return Err(PiProcessError::InvalidSidecarIntegrity);
    }
    Ok(())
}

fn copy_dependency_snapshot(source: &Path, destination: &Path) -> Result<(), PiProcessError> {
    let source = source
        .canonicalize()
        .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
    fs::create_dir(destination).map_err(PiProcessError::Io)?;
    let mut entries = collect_dependency_entries(&source)?;
    entries.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| {
                left.as_os_str()
                    .as_bytes()
                    .cmp(right.as_os_str().as_bytes())
            })
    });
    for relative in entries {
        let source_path = source.join(&relative);
        let destination_path = destination.join(&relative);
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if metadata.file_type().is_symlink() {
            let target =
                fs::read_link(&source_path).map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
            if target.is_absolute() {
                return Err(PiProcessError::InvalidSidecarIntegrity);
            }
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent).map_err(PiProcessError::Io)?;
            }
            symlink(target, destination_path).map_err(PiProcessError::Io)?;
        } else if metadata.is_dir() {
            fs::create_dir(&destination_path).map_err(PiProcessError::Io)?;
        } else if metadata.is_file() {
            copy_bounded_regular_file(
                &source_path,
                &destination_path,
                PI_MAX_DEPENDENCY_FILE_BYTES,
                false,
            )?;
        } else {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
    }
    Ok(())
}

/// Owns a freshly spawned sidecar until every parent-side pipe and reader
/// resource has been initialized. Any early return while this guard is armed
/// terminates the complete process group instead of orphaning a worker.
struct PiSpawnCleanupGuard {
    child: Option<Child>,
    process_group_id: u32,
}

impl PiSpawnCleanupGuard {
    fn new(child: Child) -> Self {
        let process_group_id = child.id();
        Self {
            child: Some(child),
            process_group_id,
        }
    }

    fn child_mut(&mut self) -> &mut Child {
        self.child
            .as_mut()
            .expect("armed Pi spawn guard must own its child")
    }

    fn into_child(mut self) -> Child {
        self.child
            .take()
            .expect("armed Pi spawn guard must own its child")
    }
}

impl Drop for PiSpawnCleanupGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = terminate_process_group(child, self.process_group_id);
        }
    }
}

impl SpawnedPiRunner {
    pub(crate) fn process_id(&self) -> u32 {
        self.process_group_id
    }

    pub fn spawn(
        node_executable: &Path,
        expected_node_sha256: &str,
        sidecar_root: &Path,
        manifest: &PiSidecarManifest,
        process_generation: u64,
        exchange_timeout: Duration,
    ) -> Result<Self, PiProcessError> {
        Self::spawn_inner(
            node_executable,
            expected_node_sha256,
            sidecar_root,
            manifest,
            process_generation,
            exchange_timeout,
            None,
        )
    }

    #[doc(hidden)]
    pub fn spawn_with_test_tls_trust(
        node_executable: &Path,
        expected_node_sha256: &str,
        sidecar_root: &Path,
        manifest: &PiSidecarManifest,
        process_generation: u64,
        exchange_timeout: Duration,
        test_tls_trust_descriptor: Vec<u8>,
    ) -> Result<Self, PiProcessError> {
        Self::spawn_inner(
            node_executable,
            expected_node_sha256,
            sidecar_root,
            manifest,
            process_generation,
            exchange_timeout,
            Some(test_tls_trust_descriptor),
        )
    }

    fn spawn_inner(
        node_executable: &Path,
        expected_node_sha256: &str,
        sidecar_root: &Path,
        manifest: &PiSidecarManifest,
        process_generation: u64,
        exchange_timeout: Duration,
        test_tls_trust_descriptor: Option<Vec<u8>>,
    ) -> Result<Self, PiProcessError> {
        if test_tls_trust_descriptor
            .as_ref()
            .is_some_and(|descriptor| {
                descriptor.is_empty() || descriptor.len() > MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES
            })
        {
            return Err(PiProcessError::TestTlsTrustChannel);
        }
        if !node_executable.is_absolute()
            || exchange_timeout.is_zero()
            || exchange_timeout > MAX_EXCHANGE_TIMEOUT
            || sha256_file(node_executable).map_err(|_| PiProcessError::InvalidExecutable)?
                != expected_node_sha256
        {
            return Err(PiProcessError::InvalidExecutable);
        }
        manifest
            .validate()
            .map_err(|_| PiProcessError::InvalidManifest)?;
        let sidecar_snapshot = snapshot_verified_sidecar(sidecar_root)?;
        let snapshot_root = sidecar_snapshot.root();
        let pinned_manifest = PiSidecarManifest::load(snapshot_root)
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;
        if &pinned_manifest != manifest {
            return Err(PiProcessError::InvalidSidecarIntegrity);
        }
        let (credential_channel, child_credential_channel) =
            UnixStream::pair().map_err(PiProcessError::Io)?;
        let credential_fd = child_credential_channel.as_raw_fd();
        clear_close_on_exec(credential_fd)?;
        let credential_fd =
            u32::try_from(credential_fd).map_err(|_| PiProcessError::CredentialChannel)?;
        let test_tls_trust_channel = if test_tls_trust_descriptor.is_some() {
            let (channel, child_channel) =
                UnixStream::pair().map_err(|_| PiProcessError::TestTlsTrustChannel)?;
            let fd = child_channel.as_raw_fd();
            clear_close_on_exec(fd).map_err(|_| PiProcessError::TestTlsTrustChannel)?;
            let fd = u32::try_from(fd).map_err(|_| PiProcessError::TestTlsTrustChannel)?;
            if fd == credential_fd {
                return Err(PiProcessError::TestTlsTrustChannel);
            }
            Some((channel, child_channel, fd))
        } else {
            None
        };
        let launch = PiLaunchSpec::new(
            node_executable.to_path_buf(),
            snapshot_root,
            manifest,
            process_generation,
            Some(credential_fd),
        )
        .map_err(|_| PiProcessError::InvalidManifest)?;
        let allowed_sidecar_root = snapshot_root
            .canonicalize()
            .map_err(|_| PiProcessError::InvalidSidecarIntegrity)?;

        let mut command = Command::new(&launch.executable);
        configure_node_permissions(&mut command, &allowed_sidecar_root);
        command
            .args(&launch.arguments)
            .current_dir(snapshot_root)
            .env_clear()
            .envs(&launch.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0);
        if let Some((_, _, fd)) = test_tls_trust_channel.as_ref() {
            command.arg(format!("--tls-trust-fd={fd}"));
        }
        let child = command.spawn().map_err(PiProcessError::Io)?;
        let mut cleanup = PiSpawnCleanupGuard::new(child);
        drop(child_credential_channel);
        if let (Some(mut descriptor), Some((mut channel, child_channel, _))) =
            (test_tls_trust_descriptor, test_tls_trust_channel)
        {
            drop(child_channel);
            let delivery = channel
                .write_all(&descriptor)
                .and_then(|()| channel.shutdown(Shutdown::Write));
            descriptor.fill(0);
            delivery.map_err(|_| PiProcessError::TestTlsTrustChannel)?;
        }
        let process_group_id = cleanup.process_group_id;
        let stdin = cleanup
            .child_mut()
            .stdin
            .take()
            .ok_or(PiProcessError::MissingPipe)?;
        let stdout = cleanup
            .child_mut()
            .stdout
            .take()
            .ok_or(PiProcessError::MissingPipe)?;
        let (sender, lines) = mpsc::sync_channel(1_024);
        let reader = thread::Builder::new()
            .name("c4os-pi-sidecar-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) if line.len() <= PI_MAX_LINE_BYTES + 1 => {
                            if sender.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {
                            let _ = sender.send(Err("Pi sidecar line exceeded its bound".into()));
                            break;
                        }
                        Err(_) => {
                            let _ = sender.send(Err("Pi sidecar output became unavailable".into()));
                            break;
                        }
                    }
                }
            })
            .map_err(PiProcessError::Io)?;
        let child = cleanup.into_child();
        Ok(Self {
            child,
            stdin: Some(stdin),
            lines,
            reader: Some(reader),
            pending: VecDeque::new(),
            credential_channel: Some(credential_channel),
            process_group_id,
            exchange_timeout,
            _sidecar_snapshot: Some(sidecar_snapshot.temporary),
        })
    }

    pub fn deliver_credential_lease(
        &mut self,
        metadata: &PiCredentialLeaseMetadata,
        lease: &OperationCredentialLease,
    ) -> Result<(), PiProcessError> {
        metadata.validate()?;
        if !lease.is_valid() {
            return Err(PiProcessError::CredentialChannel);
        }
        let credential_channel = self
            .credential_channel
            .as_mut()
            .ok_or(PiProcessError::CredentialChannel)?;
        let mut frame = CredentialFrameWriter::new(credential_channel, metadata);
        lease
            .deliver_to(&mut frame)
            .map_err(PiProcessError::Credential)?;
        frame.finish().map_err(PiProcessError::Io)
    }

    fn receive_line(&mut self, deadline: Instant) -> Result<String, String> {
        if let Some(line) = self.pending.pop_front() {
            return Ok(line);
        }
        let timeout = deadline.saturating_duration_since(Instant::now());
        self.lines
            .recv_timeout(timeout)
            .map_err(|_| "Pi sidecar response timed out".to_owned())?
    }
}

/// Applies the minimum Node permission-model grants required by the pinned Pi
/// SDK: its verified dependency graph is read-only and provider I/O is networked.
fn configure_node_permissions(command: &mut Command, allowed_sidecar_root: &Path) {
    command.arg("--permission").arg("--allow-net").arg(format!(
        "--allow-fs-read={}",
        allowed_sidecar_root.display()
    ));
}

impl PiSidecarRunner for SpawnedPiRunner {
    fn exchange(&mut self, request_line: &str) -> Result<Vec<String>, String> {
        if request_line.is_empty()
            || request_line.len() > PI_MAX_LINE_BYTES + 1
            || !request_line.ends_with('\n')
        {
            return Err("Pi request framing is invalid".into());
        }
        let request: Value = serde_json::from_str(request_line)
            .map_err(|_| "Pi request is not valid JSON".to_owned())?;
        let request_id = request
            .get("requestId")
            .and_then(Value::as_str)
            .ok_or_else(|| "Pi request has no request identity".to_owned())?;
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| "Pi sidecar input is closed".to_owned())?;
        stdin
            .write_all(request_line.as_bytes())
            .and_then(|_| stdin.flush())
            .map_err(|_| "Pi sidecar input became unavailable".to_owned())?;
        let deadline = Instant::now() + self.exchange_timeout;
        let mut observed = Vec::new();
        loop {
            let line = self.receive_line(deadline)?;
            let value: Value = serde_json::from_str(line.trim_end())
                .map_err(|_| "Pi sidecar returned malformed JSON".to_owned())?;
            let is_matching_response = value.get("kind").and_then(Value::as_str)
                == Some("response")
                && value.get("requestId").and_then(Value::as_str) == Some(request_id);
            let is_other_response = value.get("kind").and_then(Value::as_str) == Some("response")
                && !is_matching_response;
            if is_other_response {
                return Err("Pi sidecar response identity is out of order".into());
            }
            observed.push(line);
            if is_matching_response {
                return Ok(observed);
            }
        }
    }

    fn poll(&mut self) -> Result<Vec<String>, String> {
        let mut available = self.pending.drain(..).collect::<Vec<_>>();
        loop {
            match self.lines.try_recv() {
                Ok(Ok(line)) => available.push(line),
                Ok(Err(error)) => return Err(error),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("Pi sidecar output disconnected unexpectedly".into());
                }
            }
        }
        if self
            .child
            .try_wait()
            .map_err(|_| "Pi sidecar process state became unavailable".to_owned())?
            .is_some()
        {
            return Err("Pi sidecar exited unexpectedly".into());
        }
        Ok(available)
    }

    fn terminate(&mut self) -> Result<(), String> {
        self.stdin.take();
        self.credential_channel.take();
        terminate_process_group(&mut self.child, self.process_group_id)
            .map_err(|_| "Pi process-group cleanup failed".to_owned())?;
        if let Some(reader) = self.reader.take() {
            reader
                .join()
                .map_err(|_| "Pi reader thread failed".to_owned())?;
        }
        Ok(())
    }
}

impl Drop for SpawnedPiRunner {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PiCredentialLeaseMetadata {
    pub lease_id: String,
    pub provider: String,
    pub expires_at_ms: u64,
}

impl PiCredentialLeaseMetadata {
    fn validate(&self) -> Result<(), PiProcessError> {
        if !bounded_id(&self.lease_id) || !bounded_id(&self.provider) || self.expires_at_ms == 0 {
            return Err(PiProcessError::CredentialChannel);
        }
        Ok(())
    }
}

struct CredentialFrameWriter<'a> {
    inner: &'a mut UnixStream,
    metadata: &'a PiCredentialLeaseMetadata,
    header_written: bool,
    secret_bytes: usize,
}

impl<'a> CredentialFrameWriter<'a> {
    fn new(inner: &'a mut UnixStream, metadata: &'a PiCredentialLeaseMetadata) -> Self {
        Self {
            inner,
            metadata,
            header_written: false,
            secret_bytes: 0,
        }
    }

    fn write_header(&mut self) -> std::io::Result<()> {
        if self.header_written {
            return Ok(());
        }
        let lease_id = self.metadata.lease_id.as_bytes();
        let provider = self.metadata.provider.as_bytes();
        let lease_id_len = u16::try_from(lease_id.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "lease identity is too long",
            )
        })?;
        let provider_len = u16::try_from(provider.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "provider identity is too long",
            )
        })?;
        self.inner.write_all(CREDENTIAL_FRAME_MAGIC)?;
        self.inner.write_all(&[CREDENTIAL_FRAME_SCHEMA_VERSION])?;
        self.inner.write_all(&lease_id_len.to_be_bytes())?;
        self.inner.write_all(&provider_len.to_be_bytes())?;
        self.inner
            .write_all(&self.metadata.expires_at_ms.to_be_bytes())?;
        self.inner.write_all(lease_id)?;
        self.inner.write_all(provider)?;
        self.header_written = true;
        Ok(())
    }

    fn finish(mut self) -> std::io::Result<()> {
        self.write_header()?;
        if self.secret_bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "credential is empty",
            ));
        }
        self.inner.write_all(&0_u32.to_be_bytes())?;
        self.inner.flush()
    }
}

impl Write for CredentialFrameWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let next_secret_bytes = self.secret_bytes.checked_add(buffer.len()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "credential is too large")
        })?;
        if next_secret_bytes > MAX_CREDENTIAL_SECRET_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "credential is too large",
            ));
        }
        let chunk_len = u32::try_from(buffer.len()).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "credential is too large")
        })?;
        self.write_header()?;
        self.inner.write_all(&chunk_len.to_be_bytes())?;
        // The lease writes directly into the anonymous descriptor. There is no
        // Rust-side encoded or intermediate secret buffer to retain or wipe.
        self.inner.write_all(buffer)?;
        self.secret_bytes = next_secret_bytes;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

fn clear_close_on_exec(fd: RawFd) -> Result<(), PiProcessError> {
    // SAFETY: `fd` is owned by the live UnixStream above; fcntl does not retain
    // the pointer or outlive the call, and both commands operate on integer
    // descriptor flags only.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) } < 0 {
        return Err(PiProcessError::CredentialChannel);
    }
    Ok(())
}

fn terminate_process_group(child: &mut Child, process_group_id: u32) -> std::io::Result<()> {
    let _ = child.try_wait()?;
    if process_group_exists(process_group_id)? {
        signal_group(process_group_id, libc::SIGTERM)?;
        if !wait_for_process_group_exit(child, process_group_id, Duration::from_millis(500))? {
            signal_group(process_group_id, libc::SIGKILL)?;
            if !wait_for_process_group_exit(child, process_group_id, Duration::from_millis(500))? {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Pi process group survived forced termination",
                ));
            }
        }
    }
    if child.try_wait()?.is_none() {
        child.kill()?;
        child.wait()?;
    }
    Ok(())
}

fn wait_for_process_group_exit(
    child: &mut Child,
    process_group_id: u32,
    timeout: Duration,
) -> std::io::Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        let _ = child.try_wait()?;
        if !process_group_exists(process_group_id)? {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn process_group_exists(process_group_id: u32) -> std::io::Result<bool> {
    let process_group_id = process_group_signal_target(process_group_id)?;
    // SAFETY: signal zero performs a liveness/permission check only and does
    // not retain the process-group identifier after this call.
    if unsafe { libc::kill(process_group_id, 0) } == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(false),
        Some(libc::EPERM) => Ok(true),
        _ => Err(error),
    }
}

fn signal_group(process_group_id: u32, signal: libc::c_int) -> std::io::Result<()> {
    let process_group_id = process_group_signal_target(process_group_id)?;
    // SAFETY: kill receives a validated negative process-group identifier and
    // a fixed POSIX signal; it does not retain either argument.
    if unsafe { libc::kill(process_group_id, signal) } == 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

fn process_group_signal_target(process_group_id: u32) -> std::io::Result<libc::pid_t> {
    let process_group_id = libc::pid_t::try_from(process_group_id).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Pi process-group identifier is out of range",
        )
    })?;
    if process_group_id <= 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Pi process-group identifier is invalid",
        ));
    }
    Ok(-process_group_id)
}

fn bounded_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

#[derive(Debug, Error)]
pub enum PiProcessError {
    #[error("Pi Node executable is invalid or changed")]
    InvalidExecutable,
    #[error("Pi sidecar manifest is invalid")]
    InvalidManifest,
    #[error("Pi sidecar source, package lock, or installed dependency tree changed")]
    InvalidSidecarIntegrity,
    #[error("Pi sidecar pipes are unavailable")]
    MissingPipe,
    #[error("Pi credential channel is unavailable")]
    CredentialChannel,
    #[error("Pi test TLS trust channel is unavailable")]
    TestTlsTrustChannel,
    #[error("Pi process I/O failed: {0}")]
    Io(#[source] std::io::Error),
    #[error("Pi credential delivery failed: {0}")]
    Credential(#[source] CredentialVaultError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_permission_contract_allows_provider_network_after_enabling_permissions() {
        let mut command = Command::new("/usr/bin/true");
        configure_node_permissions(&mut command, Path::new("/verified/pi-sidecar"));
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            arguments,
            [
                "--permission",
                "--allow-net",
                "--allow-fs-read=/verified/pi-sidecar",
            ]
        );
    }

    #[test]
    fn initialization_failure_after_spawn_terminates_the_process_group() {
        let child = Command::new("/bin/sh")
            .arg("-c")
            .arg("trap 'exit 0' TERM; while :; do /bin/sleep 0.05; done")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .expect("spawn initialization-failure fixture");
        let process_group_id = child.id();
        assert!(process_group_exists(process_group_id).unwrap());

        let result = (|| -> Result<(), PiProcessError> {
            let mut cleanup = PiSpawnCleanupGuard::new(child);
            let _stdin = cleanup
                .child_mut()
                .stdin
                .take()
                .ok_or(PiProcessError::MissingPipe)?;
            Err(PiProcessError::MissingPipe)
        })();

        assert!(matches!(result, Err(PiProcessError::MissingPipe)));
        assert!(!process_group_exists(process_group_id).unwrap());
    }

    #[test]
    fn termination_signals_descendants_after_the_group_leader_has_exited() {
        let mut leader = Command::new("/bin/sh")
            .arg("-c")
            .arg(
                "/bin/sh -c 'trap \"\" HUP; trap \"exit 0\" TERM; while :; do /bin/sleep 0.05; done' & exit 0",
            )
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .expect("spawn process-group leader");
        let process_group_id = leader.id();
        leader.wait().expect("reap process-group leader");
        let descendant_was_alive = process_group_exists(process_group_id).unwrap();

        let cleanup = terminate_process_group(&mut leader, process_group_id);
        if cleanup.is_err() {
            let _ = signal_group(process_group_id, libc::SIGKILL);
        }

        assert!(descendant_was_alive);
        cleanup.expect("terminate descendant process group");
        assert!(!process_group_exists(process_group_id).unwrap());
    }

    #[test]
    fn polling_surfaces_an_exited_process_instead_of_returning_an_empty_batch() {
        let mut child = Command::new("/usr/bin/true")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .expect("spawn short-lived process");
        let process_group_id = child.id();
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        drop(stdout);
        child.wait().expect("reap short-lived process");
        let (sender, lines) = mpsc::sync_channel(1);
        drop(sender);
        let mut runner = SpawnedPiRunner {
            child,
            stdin,
            lines,
            reader: None,
            pending: VecDeque::new(),
            credential_channel: None,
            process_group_id,
            exchange_timeout: Duration::from_millis(10),
            _sidecar_snapshot: None,
        };

        assert!(runner.poll().is_err());
    }

    #[test]
    fn process_private_snapshot_is_unchanged_when_its_verified_source_is_mutated() {
        let sidecar_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecars/pi");
        let mutable_source = snapshot_verified_sidecar(&sidecar_root).unwrap();
        let launch_snapshot = snapshot_verified_sidecar(mutable_source.root()).unwrap();

        fs::write(
            mutable_source.root().join("pi-sdk-driver.mjs"),
            b"mutated after launch snapshot verification",
        )
        .unwrap();

        assert!(matches!(
            PiSidecarIntegrity::verify(mutable_source.root()),
            Err(PiProcessError::InvalidSidecarIntegrity)
        ));
        PiSidecarIntegrity::verify(launch_snapshot.root()).unwrap();
    }
}
