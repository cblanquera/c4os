//! Private quarantine and immutable content-addressed package storage.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use serde::{Deserialize, Serialize};

use super::{
    ExtensionError,
    package::{
        PACKAGE_MANIFEST_NAME, PackageVerificationPolicy, VerifiedPackage, verify_package_directory,
    },
    validate_digest, validate_identifier,
};

#[derive(Clone, Debug)]
pub struct ExtensionStore {
    root: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuarantinedPackage {
    pub path: PathBuf,
    pub package_id: String,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledPackage {
    pub path: PathBuf,
    pub package_id: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSelector {
    pub schema_version: u16,
    pub package_id: String,
    pub generation: u64,
    pub active_digest: Option<String>,
    pub last_known_good_digest: Option<String>,
}

impl PackageSelector {
    pub const SCHEMA_VERSION: u16 = 1;

    pub fn validate(&self) -> Result<(), ExtensionError> {
        if self.schema_version != Self::SCHEMA_VERSION || self.generation == 0 {
            return Err(ExtensionError::InvalidState);
        }
        validate_identifier(&self.package_id)?;
        if let Some(digest) = &self.active_digest {
            validate_digest(digest)?;
        }
        if let Some(digest) = &self.last_known_good_digest {
            validate_digest(digest)?;
        }
        Ok(())
    }
}

impl ExtensionStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ExtensionError> {
        let root = root.as_ref().to_path_buf();
        ensure_private_directory(&root)?;
        ensure_private_directory(&root.join("staging"))?;
        ensure_private_directory(&root.join("packages"))?;
        ensure_private_directory(&root.join("packages/sha256"))?;
        ensure_private_directory(&root.join("selectors"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Copies an already-verified package into a private staging directory and
    /// verifies the copy again. Nothing from the package executes here.
    pub fn quarantine_verified(
        &self,
        verified: &VerifiedPackage,
        policy: &PackageVerificationPolicy<'_>,
    ) -> Result<QuarantinedPackage, ExtensionError> {
        let source = verify_package_directory(&verified.root, policy)?;
        if source.digest != verified.digest
            || source.manifest.package.id != verified.manifest.package.id
        {
            return Err(ExtensionError::MutableContent);
        }

        let temporary = tempfile::Builder::new()
            .prefix("package-")
            .tempdir_in(self.root.join("staging"))?;
        set_directory_private(temporary.path())?;
        copy_regular_file(
            &source.root.join(PACKAGE_MANIFEST_NAME),
            &temporary.path().join(PACKAGE_MANIFEST_NAME),
        )?;
        for entry in &source.inventory {
            let target = temporary.path().join(&entry.path);
            if let Some(parent) = target.parent() {
                ensure_private_directory(parent)?;
            }
            copy_regular_file(&source.root.join(&entry.path), &target)?;
        }
        sync_tree(temporary.path())?;
        let copied = verify_package_directory(temporary.path(), policy)?;
        if copied.digest != source.digest
            || copied.manifest.package.id != source.manifest.package.id
        {
            return Err(ExtensionError::MutableContent);
        }
        let path = temporary.keep();
        Ok(QuarantinedPackage {
            path,
            package_id: copied.manifest.package.id,
            digest: copied.digest,
        })
    }

    /// Atomically promotes a verified quarantine to the immutable digest path.
    /// A pre-existing path is accepted only when its full verification matches.
    pub fn install_quarantined(
        &self,
        quarantined: QuarantinedPackage,
        policy: &PackageVerificationPolicy<'_>,
    ) -> Result<InstalledPackage, ExtensionError> {
        ensure_direct_child(&self.root.join("staging"), &quarantined.path)?;
        validate_identifier(&quarantined.package_id)?;
        validate_digest(&quarantined.digest)?;
        let verified = verify_package_directory(&quarantined.path, policy)?;
        if verified.digest != quarantined.digest
            || verified.manifest.package.id != quarantined.package_id
        {
            return Err(ExtensionError::MutableContent);
        }
        let target = self.package_path(&quarantined.digest)?;
        if target.exists() {
            let existing = verify_package_directory(&target, policy)?;
            if existing.digest != quarantined.digest
                || existing.manifest.package.id != quarantined.package_id
            {
                return Err(ExtensionError::MutableContent);
            }
            remove_tree_without_execution(&quarantined.path)?;
        } else {
            // macOS refuses to rename a directory whose own mode is 0500.
            // Seal every descendant before the atomic promotion, retain the
            // private 0700 quarantine root only for the rename, then seal that
            // root immediately at its immutable content-addressed location.
            seal_tree_for_promotion(&quarantined.path)?;
            fs::rename(&quarantined.path, &target)?;
            if let Err(error) = seal_promoted_root(&target) {
                let _ = remove_tree_without_execution(&target);
                return Err(error);
            }
            sync_directory(&target)?;
            sync_directory(target.parent().ok_or(ExtensionError::InvalidState)?)?;
        }
        Ok(InstalledPackage {
            path: target,
            package_id: quarantined.package_id,
            digest: quarantined.digest,
        })
    }

    pub fn verify_installed(
        &self,
        digest: &str,
        policy: &PackageVerificationPolicy<'_>,
    ) -> Result<InstalledPackage, ExtensionError> {
        let path = self.package_path(digest)?;
        let verified = verify_package_directory(&path, policy)?;
        if verified.digest != digest {
            return Err(ExtensionError::MutableContent);
        }
        Ok(InstalledPackage {
            path,
            package_id: verified.manifest.package.id,
            digest: verified.digest,
        })
    }

    pub fn read_selector(
        &self,
        package_id: &str,
    ) -> Result<Option<PackageSelector>, ExtensionError> {
        let path = self.selector_path(package_id)?;
        if !path.exists() {
            return Ok(None);
        }
        reject_symlink_or_non_file(&path)?;
        let bytes = fs::read(&path)?;
        if bytes.len() > 64 * 1024 {
            return Err(ExtensionError::BoundExceeded);
        }
        let selector: PackageSelector = serde_json::from_slice(&bytes)?;
        selector.validate()?;
        if selector.package_id != package_id {
            return Err(ExtensionError::MutableContent);
        }
        Ok(Some(selector))
    }

    /// Compare-and-swap prevents stale UI or worker state from activating a
    /// digest over a newer lifecycle decision.
    pub fn compare_and_swap_selector(
        &self,
        package_id: &str,
        expected_generation: Option<u64>,
        active_digest: Option<&str>,
        last_known_good_digest: Option<&str>,
    ) -> Result<PackageSelector, ExtensionError> {
        validate_identifier(package_id)?;
        if let Some(digest) = active_digest {
            validate_digest(digest)?;
            if !self.package_path(digest)?.exists() {
                return Err(ExtensionError::InvalidState);
            }
        }
        if let Some(digest) = last_known_good_digest {
            validate_digest(digest)?;
            if !self.package_path(digest)?.exists() {
                return Err(ExtensionError::InvalidState);
            }
        }
        let current = self.read_selector(package_id)?;
        if current.as_ref().map(|selector| selector.generation) != expected_generation {
            return Err(ExtensionError::Conflict);
        }
        let generation = expected_generation
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ExtensionError::BoundExceeded)?;
        let selector = PackageSelector {
            schema_version: PackageSelector::SCHEMA_VERSION,
            package_id: package_id.to_owned(),
            generation,
            active_digest: active_digest.map(str::to_owned),
            last_known_good_digest: last_known_good_digest.map(str::to_owned),
        };
        selector.validate()?;
        self.write_selector(&selector)?;
        Ok(selector)
    }

    pub fn rollback_selector(
        &self,
        package_id: &str,
        expected_generation: u64,
    ) -> Result<PackageSelector, ExtensionError> {
        let current = self
            .read_selector(package_id)?
            .ok_or(ExtensionError::InvalidState)?;
        if current.generation != expected_generation {
            return Err(ExtensionError::Conflict);
        }
        let last_known_good = current
            .last_known_good_digest
            .as_deref()
            .ok_or(ExtensionError::InvalidState)?;
        self.compare_and_swap_selector(
            package_id,
            Some(expected_generation),
            Some(last_known_good),
            Some(last_known_good),
        )
    }

    /// Removes bytes directly; no manifest-defined uninstall hook or script is
    /// consulted. Active selectors must first be disabled by the coordinator.
    pub fn uninstall_without_execution(
        &self,
        package_id: &str,
        digest: &str,
    ) -> Result<(), ExtensionError> {
        validate_identifier(package_id)?;
        validate_digest(digest)?;
        if self
            .read_selector(package_id)?
            .is_some_and(|selector| selector.active_digest.as_deref() == Some(digest))
        {
            return Err(ExtensionError::Conflict);
        }
        let path = self.package_path(digest)?;
        if path.exists() {
            // Even if trust has since been revoked, identity is read from the
            // strict manifest before deletion. No package content is executed.
            let manifest_path = path.join(PACKAGE_MANIFEST_NAME);
            reject_symlink_or_non_file(&manifest_path)?;
            let bytes = fs::read_to_string(manifest_path)?;
            let manifest = super::package::parse_manifest(&bytes)?;
            if manifest.package.id != package_id || manifest.package_digest != digest {
                return Err(ExtensionError::MutableContent);
            }
            remove_tree_without_execution(&path)?;
            sync_directory(path.parent().ok_or(ExtensionError::InvalidState)?)?;
        }
        Ok(())
    }

    fn package_path(&self, digest: &str) -> Result<PathBuf, ExtensionError> {
        validate_digest(digest)?;
        Ok(self.root.join("packages/sha256").join(&digest[7..]))
    }

    fn selector_path(&self, package_id: &str) -> Result<PathBuf, ExtensionError> {
        validate_identifier(package_id)?;
        Ok(self
            .root
            .join("selectors")
            .join(format!("{package_id}.json")))
    }

    fn write_selector(&self, selector: &PackageSelector) -> Result<(), ExtensionError> {
        self.write_selector_with_directory_sync(selector, sync_directory)
    }

    fn write_selector_with_directory_sync(
        &self,
        selector: &PackageSelector,
        sync_parent: impl FnOnce(&Path) -> Result<(), ExtensionError>,
    ) -> Result<(), ExtensionError> {
        let path = self.selector_path(&selector.package_id)?;
        let mut temporary = tempfile::NamedTempFile::new_in(self.root.join("selectors"))?;
        #[cfg(unix)]
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
        let bytes = serde_json::to_vec(selector)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&path)
            .map_err(|error| ExtensionError::Io(error.error))?;
        let parent = path.parent().ok_or(ExtensionError::InvalidState)?;
        if let Err(error) = sync_parent(parent) {
            // The fully synced temporary inode has already been atomically
            // renamed into place. Reporting this as a failed CAS would let a
            // coordinator compensate its durable database document after the
            // new selector became visible, creating split authority. The
            // database document is the restart authority and repairs a lost
            // directory entry, so rename is the selector publication commit
            // point. Retain the exact visible selector and surface the
            // best-effort directory-sync degradation only to the native log.
            tracing::warn!(
                package_id = %selector.package_id,
                generation = selector.generation,
                error = %error,
                "selector directory sync failed after atomic publication"
            );
        }
        Ok(())
    }
}

fn copy_regular_file(source: &Path, target: &Path) -> Result<(), ExtensionError> {
    reject_symlink_or_non_file(source)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut input = options.open(source)?;
    if !input.metadata()?.is_file() {
        return Err(ExtensionError::InvalidInput);
    }
    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    #[cfg(unix)]
    output.set_permissions(fs::Permissions::from_mode(0o600))?;
    output.write_all(&bytes)?;
    output.sync_all()?;
    Ok(())
}

fn ensure_direct_child(parent: &Path, child: &Path) -> Result<(), ExtensionError> {
    let parent = parent.canonicalize()?;
    let child = child.canonicalize()?;
    if child.parent() != Some(parent.as_path()) {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), ExtensionError> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ExtensionError::MutableContent);
        }
    } else {
        fs::create_dir(path)?;
    }
    set_directory_private(path)
}

fn set_directory_private(path: &Path) -> Result<(), ExtensionError> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn seal_tree_for_promotion(root: &Path) -> Result<(), ExtensionError> {
    let mut directories = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| {
            ExtensionError::Io(error.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid store tree")
            }))
        })?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
            return Err(ExtensionError::MutableContent);
        }
        if metadata.is_dir() {
            if entry.path() != root {
                directories.push(entry.path().to_path_buf());
            }
        } else {
            #[cfg(unix)]
            fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o400))?;
        }
    }
    for directory in directories.into_iter().rev() {
        #[cfg(unix)]
        fs::set_permissions(directory, fs::Permissions::from_mode(0o500))?;
    }
    Ok(())
}

fn seal_promoted_root(root: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::MutableContent);
    }
    #[cfg(unix)]
    fs::set_permissions(root, fs::Permissions::from_mode(0o500))?;
    Ok(())
}

fn remove_tree_without_execution(root: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::MutableContent);
    }
    #[cfg(unix)]
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .contents_first(true)
    {
        let entry = entry.map_err(|error| {
            ExtensionError::Io(error.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid store tree")
            }))
        })?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(ExtensionError::MutableContent);
        }
        if metadata.is_dir() {
            fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o700))?;
        } else if metadata.is_file() {
            fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o600))?;
        } else {
            return Err(ExtensionError::MutableContent);
        }
    }
    fs::remove_dir_all(root)?;
    Ok(())
}

fn reject_symlink_or_non_file(path: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ExtensionError::MutableContent);
    }
    Ok(())
}

fn sync_tree(root: &Path) -> Result<(), ExtensionError> {
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| {
            ExtensionError::Io(error.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid store tree")
            }))
        })?;
        if entry.file_type().is_dir() {
            sync_directory(entry.path())?;
        }
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), ExtensionError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str =
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn selector_is_strict_and_generation_is_monotonic() {
        let root = tempfile::tempdir().expect("tempdir");
        let store = ExtensionStore::new(root.path()).expect("store");
        fs::create_dir(store.package_path(DIGEST_A).expect("path")).expect("package A");
        fs::create_dir(store.package_path(DIGEST_B).expect("path")).expect("package B");

        let first = store
            .compare_and_swap_selector("sample", None, Some(DIGEST_A), Some(DIGEST_A))
            .expect("first selector");
        assert_eq!(first.generation, 1);
        assert!(matches!(
            store.compare_and_swap_selector("sample", None, Some(DIGEST_B), Some(DIGEST_A)),
            Err(ExtensionError::Conflict)
        ));
        let second = store
            .compare_and_swap_selector(
                "sample",
                Some(first.generation),
                Some(DIGEST_B),
                Some(DIGEST_A),
            )
            .expect("second selector");
        let rolled_back = store
            .rollback_selector("sample", second.generation)
            .expect("rollback");
        assert_eq!(rolled_back.active_digest.as_deref(), Some(DIGEST_A));
        assert_eq!(rolled_back.generation, 3);
    }

    #[test]
    fn active_content_cannot_be_uninstalled() {
        let root = tempfile::tempdir().expect("tempdir");
        let store = ExtensionStore::new(root.path()).expect("store");
        fs::create_dir(store.package_path(DIGEST_A).expect("path")).expect("package");
        store
            .compare_and_swap_selector("sample", None, Some(DIGEST_A), Some(DIGEST_A))
            .expect("selector");
        assert!(matches!(
            store.uninstall_without_execution("sample", DIGEST_A),
            Err(ExtensionError::Conflict)
        ));
    }

    #[test]
    fn post_rename_directory_sync_failure_is_not_reported_as_a_failed_publication() {
        let root = tempfile::tempdir().expect("tempdir");
        let store = ExtensionStore::new(root.path()).expect("store");
        fs::create_dir(store.package_path(DIGEST_A).expect("path")).expect("package");
        let selector = PackageSelector {
            schema_version: PackageSelector::SCHEMA_VERSION,
            package_id: "sample".into(),
            generation: 1,
            active_digest: Some(DIGEST_A.into()),
            last_known_good_digest: Some(DIGEST_A.into()),
        };

        store
            .write_selector_with_directory_sync(&selector, |_| {
                Err(ExtensionError::Io(std::io::Error::other(
                    "injected post-rename directory sync failure",
                )))
            })
            .expect("rename is the unambiguous publication commit point");

        assert_eq!(
            store.read_selector("sample").expect("read selector"),
            Some(selector),
            "a caller must never compensate durable state after a visible selector publication"
        );
    }
}
