//! Resolution and verification of production OpenCode assets.
//!
//! The resolver accepts a Tauri resource root (or the project root in local
//! development), pins every project-owned manifest and dependency artifact,
//! and resolves the provenance-pinned C4OS build based on the exact OpenCode
//! tag. The official package remains the installation layout and SDK source,
//! while the executable is rebuilt with the descriptor-only server-auth patch.

use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::runtime::opencode::OPENCODE_NATIVE_VERSION;
use crate::runtime::opencode_native::{
    NativeBoundaryError, OpenCodeNativeCommandDriver, VaultCredentialResolver,
};
use crate::runtime::opencode_sdk::{OpenCodeSdkIntegrity, OpenCodeSdkIntegrityReceipt};
use crate::runtime::supervisor::sha256_file;

const LAUNCHER_SHA256: &str =
    "sha256:c9376b8d4e05f87c277313f57bff259258180cedd84d133c8dae20dcd4db6a46";
const NATIVE_PACKAGE_SHA256: &str =
    "sha256:8c9ae08080f9f9a3354a7eabf348b516201a83d08ebfdf81e5cba98078145bbd";
const NATIVE_LOCK_SHA256: &str =
    "sha256:2db45b42b9c266daf6856eb6b5e275b2afcce66e66b4e0f3a588807dfa191ba8";
const C4OS_BUILD_SCRIPT_SHA256: &str =
    "sha256:376de503312ec9440b6c8e8d9ec861fdd2405c3041cd6adcb753419bdcc9d7ca";
const C4OS_BUILD_PATH_SAFETY_SHA256: &str =
    "sha256:a92ab20759dc8fe8efa36bcaea9eed1c10630a856457ce3bc192d4b36c11aa8c";
const C4OS_BUILD_MANIFEST_SHA256: &str =
    "sha256:d08f96814b24a3da0bbdb85794adeae8fd24c0f3791866813a1da47fbe7f7114";
const C4OS_AUTH_PATCH_SHA256: &str =
    "sha256:3b07e427f8c7f85b4ec1ec7d51c1474f396643d12b44df01926591acb500b33f";
const C4OS_MODELS_SNAPSHOT_SHA256: &str =
    "sha256:bc9565e9e805f3a5496e674dccbce5ed0338dabf0e2d93c6ddbf41d00533216d";
const C4OS_BUILD_FLAVOR: &str = "c4os-auth-fd.1";
const NATIVE_EXECUTABLE_SHA256: &str =
    "sha256:4d8e086228e3b8effe720284b5ed5c4e1f0a98dd9e677bb16e5074eb415f27b1";
const NATIVE_TREE_SHA256: &str =
    "sha256:aa6356261a3511f7c5b082bca5952f3206002b9c11639b904750b25b8c786df7";
const OPENCODE_PACKAGE_INTEGRITY: &str = "sha512-HnItl/+uhSpj7JV9x6ITiE0XFq4b/PKF5OM03TIyiFoFiLw3MQoJOAXZFTEzC7IOgAIYcysRQBBmCmlXILkxww==";
const OPENCODE_DARWIN_ARM64_INTEGRITY: &str = "sha512-BkfJf4E502OVwpDXgaVz8eqTDOy2B9xXPefYebT+xv8TXYvEo1wtby4eu4MoN51o4o1qpzcvCCXDqUDXu1pBGQ==";
const MAX_NATIVE_TREE_FILES: usize = 64;
const MAX_NATIVE_TREE_BYTES: u64 = 384 * 1024 * 1024;
const MAX_NATIVE_FILE_BYTES: u64 = 192 * 1024 * 1024;
// The 137 MB native binary can require a cold macOS code/provenance scan when
// first executed from a newly assembled app bundle. Keep the check bounded,
// but align it with the production startup budget instead of treating that
// first-run scan as a version substitution. This verification precedes the
// separately bounded 30-second server startup phase.
const VERSION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedOpenCodeAssets {
    resource_root: PathBuf,
    launcher: PathBuf,
    sdk_root: PathBuf,
    native_root: PathBuf,
    native_executable: PathBuf,
    sdk_integrity: OpenCodeSdkIntegrityReceipt,
    native_tree_sha256: String,
}

/// Factory-minted identity for the only OpenCode native executable this
/// installation may launch. The private fields keep launch callers from
/// supplying their own path, digest, tree receipt, or version claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PinnedOpenCodeLaunchIdentity {
    executable: PathBuf,
    executable_sha256: String,
    native_dependency_root: PathBuf,
    native_tree_sha256: String,
    native_version: String,
    build_flavor: String,
}

impl PinnedOpenCodeLaunchIdentity {
    pub(crate) fn accepts(&self, executable: &Path, expected_sha256: &str) -> bool {
        executable == self.executable
            && expected_sha256 == self.executable_sha256
            && self.native_version == OPENCODE_NATIVE_VERSION
            && self.build_flavor == C4OS_BUILD_FLAVOR
            && sha256_file(&self.executable).ok().as_deref()
                == Some(self.executable_sha256.as_str())
            && native_dependency_tree_sha256(&self.native_dependency_root)
                .ok()
                .as_deref()
                == Some(self.native_tree_sha256.as_str())
            && verify_exact_version(&self.executable).unwrap_or(false)
    }
}

impl ResolvedOpenCodeAssets {
    pub fn resource_root(&self) -> &Path {
        &self.resource_root
    }

    pub fn launcher(&self) -> &Path {
        &self.launcher
    }

    pub fn sdk_root(&self) -> &Path {
        &self.sdk_root
    }

    pub fn native_root(&self) -> &Path {
        &self.native_root
    }

    pub fn native_executable(&self) -> &Path {
        &self.native_executable
    }

    pub fn native_executable_sha256(&self) -> &'static str {
        NATIVE_EXECUTABLE_SHA256
    }

    pub fn native_build_flavor(&self) -> &'static str {
        C4OS_BUILD_FLAVOR
    }

    pub fn launcher_sha256(&self) -> &'static str {
        LAUNCHER_SHA256
    }

    pub fn native_tree_sha256(&self) -> &str {
        &self.native_tree_sha256
    }

    pub fn sdk_dependency_tree_sha256(&self) -> &str {
        self.sdk_integrity.dependency_tree_sha256()
    }

    fn pinned_launch_identity(&self) -> PinnedOpenCodeLaunchIdentity {
        PinnedOpenCodeLaunchIdentity {
            executable: self.native_executable.clone(),
            executable_sha256: NATIVE_EXECUTABLE_SHA256.into(),
            native_dependency_root: self.native_root.join("node_modules"),
            native_tree_sha256: self.native_tree_sha256.clone(),
            native_version: OPENCODE_NATIVE_VERSION.into(),
            build_flavor: C4OS_BUILD_FLAVOR.into(),
        }
    }
}

#[derive(Debug)]
pub struct PreparedOpenCodeNativeRuntime {
    assets: ResolvedOpenCodeAssets,
    driver: OpenCodeNativeCommandDriver,
}

impl PreparedOpenCodeNativeRuntime {
    pub fn assets(&self) -> &ResolvedOpenCodeAssets {
        &self.assets
    }

    pub fn driver_mut(&mut self) -> &mut OpenCodeNativeCommandDriver {
        &mut self.driver
    }

    pub fn into_parts(self) -> (ResolvedOpenCodeAssets, OpenCodeNativeCommandDriver) {
        (self.assets, self.driver)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeProductionAssetFactory {
    resource_root: PathBuf,
}

impl OpenCodeProductionAssetFactory {
    pub fn new(resource_root: impl Into<PathBuf>) -> Result<Self, OpenCodeAssetError> {
        let resource_root = canonical_directory(&resource_root.into())?;
        Ok(Self { resource_root })
    }

    pub fn resolve(&self) -> Result<ResolvedOpenCodeAssets, OpenCodeAssetError> {
        if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            return Err(OpenCodeAssetError::UnsupportedPlatform);
        }
        let launcher = verified_file(
            &self.resource_root,
            "sidecars/opencode-launcher/main.mjs",
            LAUNCHER_SHA256,
            256 * 1024,
        )?;
        let sdk_root = self.resource_root.join("sidecars/opencode-sdk");
        let sdk_integrity = OpenCodeSdkIntegrity::verify(&sdk_root)
            .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
        let sdk_root = sdk_integrity.sidecar_root().to_path_buf();
        if !sdk_root.starts_with(&self.resource_root) {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }

        let native_root =
            canonical_directory(&self.resource_root.join("sidecars/opencode-native"))?;
        if !native_root.starts_with(&self.resource_root) {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }
        verified_file(
            &native_root,
            "package.json",
            NATIVE_PACKAGE_SHA256,
            64 * 1024,
        )?;
        verified_file(
            &native_root,
            "package-lock.json",
            NATIVE_LOCK_SHA256,
            1024 * 1024,
        )?;
        verified_file(
            &native_root,
            "build-c4os.mjs",
            C4OS_BUILD_SCRIPT_SHA256,
            256 * 1024,
        )?;
        verified_file(
            &native_root,
            "build-path-safety.mjs",
            C4OS_BUILD_PATH_SAFETY_SHA256,
            64 * 1024,
        )?;
        verified_file(
            &native_root,
            "c4os-build.json",
            C4OS_BUILD_MANIFEST_SHA256,
            64 * 1024,
        )?;
        verified_file(
            &native_root,
            "patches/c4os-server-auth-fd.patch",
            C4OS_AUTH_PATCH_SHA256,
            64 * 1024,
        )?;
        verified_file(
            &native_root,
            "build-inputs/models-dev-api-20260719.json.gz",
            C4OS_MODELS_SNAPSHOT_SHA256,
            4 * 1024 * 1024,
        )?;
        verify_c4os_build_manifest(&native_root)?;
        verify_native_lock(&native_root)?;
        let native_tree_sha256 = native_dependency_tree_sha256(&native_root.join("node_modules"))?;
        if native_tree_sha256 != NATIVE_TREE_SHA256 {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }
        let native_executable = verified_file(
            &native_root,
            "node_modules/opencode-darwin-arm64/bin/opencode",
            NATIVE_EXECUTABLE_SHA256,
            MAX_NATIVE_FILE_BYTES,
        )?;
        if native_executable
            .metadata()
            .map_err(OpenCodeAssetError::Io)?
            .permissions()
            .mode()
            & 0o111
            == 0
            || !verify_exact_version(&native_executable)?
        {
            return Err(OpenCodeAssetError::InvalidVersion);
        }
        Ok(ResolvedOpenCodeAssets {
            resource_root: self.resource_root.clone(),
            launcher,
            sdk_root,
            native_root,
            native_executable,
            sdk_integrity,
            native_tree_sha256,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn construct_driver(
        &self,
        node_executable: &Path,
        expected_node_sha256: &str,
        credentials: VaultCredentialResolver,
        process_generation: u64,
        startup_timeout: Duration,
        shutdown_timeout: Duration,
    ) -> Result<PreparedOpenCodeNativeRuntime, OpenCodeAssetError> {
        let provider_credential_vault = credentials.credential_vault();
        self.construct_driver_with_provider_vault(
            node_executable,
            expected_node_sha256,
            credentials,
            provider_credential_vault,
            process_generation,
            startup_timeout,
            shutdown_timeout,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn construct_driver_with_provider_vault(
        &self,
        node_executable: &Path,
        expected_node_sha256: &str,
        credentials: VaultCredentialResolver,
        provider_credential_vault: crate::security::credentials::CredentialVault,
        process_generation: u64,
        startup_timeout: Duration,
        shutdown_timeout: Duration,
    ) -> Result<PreparedOpenCodeNativeRuntime, OpenCodeAssetError> {
        let assets = self.resolve()?;
        if sha256_file(node_executable).map_err(|_| OpenCodeAssetError::InvalidIntegrity)?
            != expected_node_sha256
        {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }
        let pinned_native = assets.pinned_launch_identity();
        let driver = OpenCodeNativeCommandDriver::new(
            node_executable,
            expected_node_sha256,
            assets.launcher(),
            assets.launcher_sha256(),
            assets.sdk_root(),
            pinned_native,
            credentials,
            provider_credential_vault,
            process_generation,
            startup_timeout,
            shutdown_timeout,
        )
        .map_err(OpenCodeAssetError::Driver)?;
        Ok(PreparedOpenCodeNativeRuntime { assets, driver })
    }
}

#[derive(Debug, Error)]
pub enum OpenCodeAssetError {
    #[error("OpenCode production assets are unsupported on this platform")]
    UnsupportedPlatform,
    #[error("OpenCode production asset integrity failed")]
    InvalidIntegrity,
    #[error("OpenCode production native version failed")]
    InvalidVersion,
    #[error("OpenCode native driver construction failed")]
    Driver(NativeBoundaryError),
    #[error("OpenCode production asset I/O failed")]
    Io(#[source] std::io::Error),
}

fn verify_c4os_build_manifest(native_root: &Path) -> Result<(), OpenCodeAssetError> {
    let manifest: Value = serde_json::from_reader(
        File::open(native_root.join("c4os-build.json")).map_err(OpenCodeAssetError::Io)?,
    )
    .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    let expected = [
        ("/buildFlavor", C4OS_BUILD_FLAVOR),
        ("/upstream/tag", "v1.18.3"),
        (
            "/upstream/commit",
            "127bdb30784d508cc556c71a0f32b508a3061517",
        ),
        ("/upstream/nativeVersion", OPENCODE_NATIVE_VERSION),
        ("/toolchain/nodeVersion", "26.3.0"),
        (
            "/toolchain/nodeDarwinArm64Sha256",
            "sha256:cdf556966c52b321abb07cd565edb5fb2ca61c467f84a434bb28e0d33a3c9580",
        ),
        ("/toolchain/bunVersion", "1.3.14"),
        (
            "/toolchain/bunDarwinArm64Sha256",
            "sha256:e0c90ec15d33363e6b70713d56bc3b2c7585c17f40a0fe0f8fd9305901d4e233",
        ),
        ("/patch/sha256", C4OS_AUTH_PATCH_SHA256),
        ("/modelsSnapshot/gzipSha256", C4OS_MODELS_SNAPSHOT_SHA256),
        (
            "/modelsSnapshot/jsonSha256",
            "sha256:754d36153239e618691b364980dda1ab478004cf734dc16079479d7d946b1789",
        ),
        ("/artifact/sha256", NATIVE_EXECUTABLE_SHA256),
    ];
    if manifest.get("schemaVersion").and_then(Value::as_u64) != Some(1)
        || manifest
            .pointer("/artifact/sizeBytes")
            .and_then(Value::as_u64)
            != Some(137_518_946)
        || expected.into_iter().any(|(pointer, value)| {
            manifest.pointer(pointer).and_then(Value::as_str) != Some(value)
        })
    {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    Ok(())
}

fn verify_native_lock(native_root: &Path) -> Result<(), OpenCodeAssetError> {
    let package: Value = serde_json::from_reader(
        File::open(native_root.join("package.json")).map_err(OpenCodeAssetError::Io)?,
    )
    .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    let lock: Value = serde_json::from_reader(
        File::open(native_root.join("package-lock.json")).map_err(OpenCodeAssetError::Io)?,
    )
    .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    let dependencies = package
        .get("dependencies")
        .and_then(Value::as_object)
        .ok_or(OpenCodeAssetError::InvalidIntegrity)?;
    if dependencies.len() != 1
        || dependencies.get("opencode-ai").and_then(Value::as_str) != Some(OPENCODE_NATIVE_VERSION)
        || lock.get("lockfileVersion").and_then(Value::as_u64) != Some(3)
    {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    verify_lock_package(
        &lock,
        "node_modules/opencode-ai",
        OPENCODE_PACKAGE_INTEGRITY,
    )?;
    verify_lock_package(
        &lock,
        "node_modules/opencode-darwin-arm64",
        OPENCODE_DARWIN_ARM64_INTEGRITY,
    )
}

fn verify_lock_package(
    lock: &Value,
    path: &str,
    integrity: &str,
) -> Result<(), OpenCodeAssetError> {
    let escaped = path.replace('~', "~0").replace('/', "~1");
    let package = lock
        .pointer(&format!("/packages/{escaped}"))
        .and_then(Value::as_object)
        .ok_or(OpenCodeAssetError::InvalidIntegrity)?;
    if package.get("version").and_then(Value::as_str) != Some(OPENCODE_NATIVE_VERSION)
        || package.get("integrity").and_then(Value::as_str) != Some(integrity)
    {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    Ok(())
}

fn verified_file(
    root: &Path,
    relative: &str,
    expected_sha256: &str,
    maximum_bytes: u64,
) -> Result<PathBuf, OpenCodeAssetError> {
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    let metadata = path
        .metadata()
        .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    if !path.starts_with(root)
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum_bytes
        || sha256_file(&path).map_err(|_| OpenCodeAssetError::InvalidIntegrity)? != expected_sha256
    {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    Ok(path)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, OpenCodeAssetError> {
    let path = path
        .canonicalize()
        .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
    if !path.metadata().map_err(OpenCodeAssetError::Io)?.is_dir() {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    Ok(path)
}

fn verify_exact_version(executable: &Path) -> Result<bool, OpenCodeAssetError> {
    let mut child = Command::new(executable)
        .arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(OpenCodeAssetError::Io)?;
    let process_group_id = child.id();
    let deadline = Instant::now() + VERSION_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().map_err(OpenCodeAssetError::Io)? {
            if !status.success() {
                return Ok(false);
            }
            let mut output = Vec::new();
            child
                .stdout
                .take()
                .ok_or(OpenCodeAssetError::InvalidVersion)?
                .take(129)
                .read_to_end(&mut output)
                .map_err(OpenCodeAssetError::Io)?;
            return Ok(output.len() <= 128
                && String::from_utf8(output)
                    .ok()
                    .is_some_and(|version| version.trim() == OPENCODE_NATIVE_VERSION));
        }
        if Instant::now() >= deadline {
            if let Ok(process_group_id) = i32::try_from(process_group_id) {
                // SAFETY: the child was started in its own process group and
                // the constant signal does not dereference memory.
                unsafe { libc::kill(-process_group_id, libc::SIGKILL) };
            }
            let _ = child.wait();
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn native_dependency_tree_sha256(node_modules: &Path) -> Result<String, OpenCodeAssetError> {
    let root = canonical_directory(node_modules)?;
    let mut entries = Vec::new();
    collect_entries(&root, &root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.len() > MAX_NATIVE_TREE_FILES {
        return Err(OpenCodeAssetError::InvalidIntegrity);
    }
    let mut total_bytes = 0_u64;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    for (relative, path, kind) in entries {
        digest.update([kind]);
        digest.update(relative.as_bytes());
        digest.update([0]);
        if kind == b'F' {
            let metadata = path.metadata().map_err(OpenCodeAssetError::Io)?;
            if metadata.len() > MAX_NATIVE_FILE_BYTES {
                return Err(OpenCodeAssetError::InvalidIntegrity);
            }
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or(OpenCodeAssetError::InvalidIntegrity)?;
            if total_bytes > MAX_NATIVE_TREE_BYTES {
                return Err(OpenCodeAssetError::InvalidIntegrity);
            }
            digest.update(metadata.len().to_be_bytes());
            let mut file = File::open(path).map_err(OpenCodeAssetError::Io)?;
            loop {
                let read = file.read(&mut buffer).map_err(OpenCodeAssetError::Io)?;
                if read == 0 {
                    break;
                }
                digest.update(&buffer[..read]);
            }
        } else {
            digest.update(
                fs::read_link(path)
                    .map_err(OpenCodeAssetError::Io)?
                    .as_os_str()
                    .as_encoded_bytes(),
            );
        }
        digest.update([0xff]);
    }
    let mut encoded = String::from("sha256:");
    for byte in digest.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing a String is infallible");
    }
    Ok(encoded)
}

fn collect_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(String, PathBuf, u8)>,
) -> Result<(), OpenCodeAssetError> {
    for entry in fs::read_dir(directory).map_err(OpenCodeAssetError::Io)? {
        let entry = entry.map_err(OpenCodeAssetError::Io)?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(OpenCodeAssetError::Io)?;
        if metadata.is_dir() {
            collect_entries(root, &path, entries)?;
        } else if metadata.is_file() {
            entries.push((relative_utf8(root, &path)?, path, b'F'));
        } else if metadata.file_type().is_symlink() {
            let canonical = path
                .canonicalize()
                .map_err(|_| OpenCodeAssetError::InvalidIntegrity)?;
            if !canonical.starts_with(root) || !canonical.is_file() {
                return Err(OpenCodeAssetError::InvalidIntegrity);
            }
            entries.push((relative_utf8(root, &path)?, path, b'L'));
        } else {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }
        if entries.len() > MAX_NATIVE_TREE_FILES {
            return Err(OpenCodeAssetError::InvalidIntegrity);
        }
    }
    Ok(())
}

fn relative_utf8(root: &Path, path: &Path) -> Result<String, OpenCodeAssetError> {
    path.strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(OpenCodeAssetError::InvalidIntegrity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn pinned_launch_identity_rechecks_native_file_and_tree_at_launch() {
        let temporary = TempDir::new().expect("temporary native root");
        let native_dependency_root = temporary.path().join("node_modules");
        let package_root = native_dependency_root.join("opencode-test/bin");
        fs::create_dir_all(&package_root).expect("synthetic package root");
        let executable = package_root.join("opencode");
        fs::write(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 1.18.3; exit 0; fi\nexit 1\n",
        )
        .expect("synthetic native executable");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
            .expect("synthetic native permissions");
        let executable = executable.canonicalize().expect("canonical executable");
        let executable_sha256 = sha256_file(&executable).expect("executable digest");
        let native_tree_sha256 =
            native_dependency_tree_sha256(&native_dependency_root).expect("native tree digest");
        let identity = PinnedOpenCodeLaunchIdentity {
            executable: executable.clone(),
            executable_sha256: executable_sha256.clone(),
            native_dependency_root: native_dependency_root.clone(),
            native_tree_sha256,
            native_version: OPENCODE_NATIVE_VERSION.into(),
            build_flavor: C4OS_BUILD_FLAVOR.into(),
        };

        assert!(identity.accepts(&executable, &executable_sha256));

        let injected = native_dependency_root.join("opencode-test/injected.txt");
        fs::write(&injected, "post-verification mutation").expect("inject native tree file");
        assert!(
            !identity.accepts(&executable, &executable_sha256),
            "the complete verified native tree must be rechecked before spawn"
        );
        fs::remove_file(injected).expect("remove injected fixture");

        fs::write(
            &executable,
            "#!/bin/sh\n# changed after factory verification\nif [ \"$1\" = \"--version\" ]; then echo 1.18.3; exit 0; fi\nexit 1\n",
        )
        .expect("mutate native executable");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
            .expect("mutated native permissions");
        assert!(
            !identity.accepts(&executable, &executable_sha256),
            "an executable that still claims 1.18.3 must fail after its bytes change"
        );
    }

    #[test]
    fn pinned_launch_identity_rejects_path_digest_and_version_substitution() {
        let temporary = TempDir::new().expect("temporary native root");
        let native_dependency_root = temporary.path().join("node_modules");
        fs::create_dir(&native_dependency_root).expect("native dependency root");
        let executable = native_dependency_root.join("opencode");
        fs::write(&executable, "#!/bin/sh\necho 1.18.3\n").expect("synthetic executable");
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))
            .expect("synthetic executable permissions");
        let executable = executable.canonicalize().expect("canonical executable");
        let executable_sha256 = sha256_file(&executable).expect("executable digest");
        let identity = PinnedOpenCodeLaunchIdentity {
            executable: executable.clone(),
            executable_sha256: executable_sha256.clone(),
            native_dependency_root: native_dependency_root.clone(),
            native_tree_sha256: native_dependency_tree_sha256(&native_dependency_root)
                .expect("native tree digest"),
            native_version: OPENCODE_NATIVE_VERSION.into(),
            build_flavor: C4OS_BUILD_FLAVOR.into(),
        };
        let substitute = temporary.path().join("same-binary-different-path");
        std::os::unix::fs::symlink(&executable, &substitute).expect("substitute symlink");

        assert!(!identity.accepts(&substitute, &executable_sha256));
        assert!(!identity.accepts(
            &executable,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
        ));

        let mut wrong_version = identity;
        wrong_version.native_version = "1.18.4".into();
        assert!(!wrong_version.accepts(&executable, &executable_sha256));
    }
}
