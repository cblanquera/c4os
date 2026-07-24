//! Descriptor-rooted File and Folder operations for one trusted Project.
//!
//! Every path is Project-relative and every component is opened relative to an
//! owned directory descriptor with `O_NOFOLLOW`. The renderer never receives
//! the root descriptor or a native absolute path. Policy and Action Gateway
//! integration deliberately remain outside this facility.

use std::collections::BTreeSet;
use std::ffi::{CStr, CString};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use super::{FilesystemObjectIdentity, FilesystemObjectKind, TrustedProjectRoot};

pub const MAX_PROJECT_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PROJECT_FOLDER_ENTRIES: usize = 4_096;
pub const MAX_PROJECT_FOLDER_NAME_BYTES: usize = 1024 * 1024;

const MAX_RELATIVE_PATH_BYTES: usize = 4_096;
const MAX_RELATIVE_PATH_COMPONENTS: usize = 128;
const MAX_PATH_COMPONENT_BYTES: usize = 255;
const TEMPORARY_PREFIX: &str = ".c4os-write-";
const TEMPORARY_ATTEMPTS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectFilesystemLimits {
    max_file_bytes: u64,
    max_folder_entries: usize,
    max_folder_name_bytes: usize,
}

impl ProjectFilesystemLimits {
    pub fn new(
        max_file_bytes: u64,
        max_folder_entries: usize,
        max_folder_name_bytes: usize,
    ) -> Result<Self, ProjectFilesystemError> {
        if max_file_bytes == 0
            || max_file_bytes > MAX_PROJECT_FILE_BYTES
            || max_folder_entries == 0
            || max_folder_entries > MAX_PROJECT_FOLDER_ENTRIES
            || max_folder_name_bytes == 0
            || max_folder_name_bytes > MAX_PROJECT_FOLDER_NAME_BYTES
        {
            return Err(ProjectFilesystemError::InvalidLimits);
        }
        Ok(Self {
            max_file_bytes,
            max_folder_entries,
            max_folder_name_bytes,
        })
    }

    pub fn max_file_bytes(self) -> u64 {
        self.max_file_bytes
    }

    pub fn max_folder_entries(self) -> usize {
        self.max_folder_entries
    }

    pub fn max_folder_name_bytes(self) -> usize {
        self.max_folder_name_bytes
    }
}

impl Default for ProjectFilesystemLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: MAX_PROJECT_FILE_BYTES,
            max_folder_entries: MAX_PROJECT_FOLDER_ENTRIES,
            max_folder_name_bytes: MAX_PROJECT_FOLDER_NAME_BYTES,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileVersion {
    content_sha256: String,
    identity: FilesystemObjectIdentity,
    byte_length: u64,
    modified_at_seconds: i64,
    modified_at_nanoseconds: i64,
    changed_at_seconds: i64,
    changed_at_nanoseconds: i64,
}

impl FileVersion {
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    pub fn identity(&self) -> &FilesystemObjectIdentity {
        &self.identity
    }

    pub fn byte_length(&self) -> u64 {
        self.byte_length
    }

    /// Exact opaque value suitable for `CanonicalAction.target_version`.
    pub fn target_version(&self) -> String {
        let mut digest = Sha256::new();
        digest.update(b"c4os.project-file-version.v1\0");
        digest.update(self.content_sha256.as_bytes());
        digest.update(self.identity.device.to_le_bytes());
        digest.update(self.identity.inode.to_le_bytes());
        digest.update(self.byte_length.to_le_bytes());
        digest.update(self.modified_at_seconds.to_le_bytes());
        digest.update(self.modified_at_nanoseconds.to_le_bytes());
        digest.update(self.changed_at_seconds.to_le_bytes());
        digest.update(self.changed_at_nanoseconds.to_le_bytes());
        prefixed_digest(digest)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileRead {
    relative_path: PathBuf,
    content: String,
    version: FileVersion,
}

impl FileRead {
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> &FileVersion {
        &self.version
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FolderEntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderEntry {
    name: String,
    kind: FolderEntryKind,
    byte_length: Option<u64>,
}

impl FolderEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> FolderEntryKind {
        self.kind
    }

    pub fn byte_length(&self) -> Option<u64> {
        self.byte_length
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FolderListing {
    relative_path: PathBuf,
    entries: Vec<FolderEntry>,
    version: String,
}

impl FolderListing {
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn entries(&self) -> &[FolderEntry] {
        &self.entries
    }

    pub fn target_version(&self) -> &str {
        &self.version
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", content = "version", rename_all = "camelCase")]
pub enum ExpectedFileState {
    Absent,
    Existing(FileVersion),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileWriteOutcome {
    relative_path: PathBuf,
    created: bool,
    version: FileVersion,
}

impl FileWriteOutcome {
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn created(&self) -> bool {
        self.created
    }

    pub fn version(&self) -> &FileVersion {
        &self.version
    }
}

#[derive(Debug, Error)]
pub enum ProjectFilesystemError {
    #[error("Project filesystem limits are invalid")]
    InvalidLimits,
    #[error("Project-relative path is empty or not normalized")]
    InvalidPath,
    #[error("trusted Project root changed or is unavailable")]
    RootChanged,
    #[error("target is unavailable")]
    TargetUnavailable,
    #[error("symbolic links are not permitted in Project filesystem paths")]
    SymlinkRejected,
    #[error("path component is not a directory")]
    NotDirectory,
    #[error("target is not a regular file")]
    NotRegularFile,
    #[error("file exceeds the configured {max_bytes}-byte limit")]
    FileLimitExceeded { max_bytes: u64 },
    #[error("folder exceeds the configured {max_entries}-entry limit")]
    FolderEntryLimitExceeded { max_entries: usize },
    #[error("folder names exceed the configured {max_bytes}-byte aggregate limit")]
    FolderNameLimitExceeded { max_bytes: usize },
    #[error("file or folder name is not valid UTF-8")]
    InvalidUtf8,
    #[error("target changed while it was being inspected")]
    TargetChanged,
    #[error("expected File version does not match the live target")]
    Conflict,
    #[error("atomic filesystem operation is unavailable")]
    AtomicOperationUnavailable,
    #[error("atomic write recovery could not establish a safe final state")]
    RecoveryFailed,
    #[error("Project filesystem write lock is unavailable")]
    WriteLockUnavailable,
    #[error("Project filesystem {operation} failed")]
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
}

#[derive(Clone)]
pub struct ProjectFilesystem {
    trusted_root: TrustedProjectRoot,
    root_directory: Arc<File>,
    limits: ProjectFilesystemLimits,
    write_lock: Arc<Mutex<()>>,
}

impl fmt::Debug for ProjectFilesystem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProjectFilesystem")
            .field("root_identity", self.trusted_root.identity())
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl ProjectFilesystem {
    pub fn bind(trusted_root: TrustedProjectRoot) -> Result<Self, ProjectFilesystemError> {
        Self::bind_with_limits(trusted_root, ProjectFilesystemLimits::default())
    }

    pub fn bind_with_limits(
        trusted_root: TrustedProjectRoot,
        limits: ProjectFilesystemLimits,
    ) -> Result<Self, ProjectFilesystemError> {
        trusted_root
            .revalidate()
            .map_err(|_| ProjectFilesystemError::RootChanged)?;
        let root_directory =
            open_absolute_directory(trusted_root.canonical_root()).map_err(|source| {
                ProjectFilesystemError::Io {
                    operation: "root bind",
                    source,
                }
            })?;
        let metadata = root_directory
            .metadata()
            .map_err(|source| ProjectFilesystemError::Io {
                operation: "root identity read",
                source,
            })?;
        if !metadata.is_dir()
            || metadata.dev() != trusted_root.identity().device
            || metadata.ino() != trusted_root.identity().inode
        {
            return Err(ProjectFilesystemError::RootChanged);
        }
        Ok(Self {
            trusted_root,
            root_directory: Arc::new(root_directory),
            limits,
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn limits(&self) -> ProjectFilesystemLimits {
        self.limits
    }

    pub fn read_utf8(&self, relative_path: &Path) -> Result<FileRead, ProjectFilesystemError> {
        let path = NormalizedRelativePath::file(relative_path)?;
        self.ensure_root_current()?;
        let (parent, name) = self.open_parent(&path)?;
        let parent_identity = metadata_snapshot(&parent, "read parent identity")?;
        let (content, version, target_metadata) =
            read_utf8_at(&parent, name, self.limits.max_file_bytes)?;
        self.ensure_root_current()?;
        self.revalidate_opened_target(&path, &parent_identity, &target_metadata)?;
        Ok(FileRead {
            relative_path: path.path,
            content,
            version,
        })
    }

    pub fn list_folder(
        &self,
        relative_path: &Path,
    ) -> Result<FolderListing, ProjectFilesystemError> {
        let path = NormalizedRelativePath::directory(relative_path)?;
        self.ensure_root_current()?;
        let directory = self.open_directory(&path)?;
        let before = metadata_snapshot(&directory, "folder identity read")?;
        if !before.is_directory() {
            return Err(ProjectFilesystemError::NotDirectory);
        }
        let scanned = scan_directory(
            &directory,
            self.limits.max_folder_entries,
            self.limits.max_folder_name_bytes,
        )?;
        let after = metadata_snapshot(&directory, "folder identity recheck")?;
        if before != after {
            return Err(ProjectFilesystemError::TargetChanged);
        }
        self.ensure_root_current()?;
        let current = self.open_directory(&path)?;
        if metadata_snapshot(&current, "folder path recheck")? != before {
            return Err(ProjectFilesystemError::TargetChanged);
        }

        let mut digest = Sha256::new();
        digest.update(b"c4os.project-folder-version.v1\0");
        before.update_digest(&mut digest);
        for entry in &scanned {
            digest.update((entry.public.name.len() as u64).to_le_bytes());
            digest.update(entry.public.name.as_bytes());
            entry.metadata.update_digest(&mut digest);
        }
        Ok(FolderListing {
            relative_path: path.path,
            entries: scanned.into_iter().map(|entry| entry.public).collect(),
            version: prefixed_digest(digest),
        })
    }

    pub fn write_utf8(
        &self,
        relative_path: &Path,
        content: &str,
        expected: &ExpectedFileState,
    ) -> Result<FileWriteOutcome, ProjectFilesystemError> {
        if content.len() as u64 > self.limits.max_file_bytes {
            return Err(ProjectFilesystemError::FileLimitExceeded {
                max_bytes: self.limits.max_file_bytes,
            });
        }
        let path = NormalizedRelativePath::file(relative_path)?;
        let _write_guard = self
            .write_lock
            .lock()
            .map_err(|_| ProjectFilesystemError::WriteLockUnavailable)?;
        self.ensure_root_current()?;
        let (parent, name) = self.open_parent(&path)?;
        let parent_identity = metadata_snapshot(&parent, "write parent identity")?;
        let (mut temporary, mut cleanup) = create_temporary(&parent)?;
        temporary
            .write_all(content.as_bytes())
            .map_err(|source| ProjectFilesystemError::Io {
                operation: "temporary File write",
                source,
            })?;

        // Reopen the path from the bound root immediately before publication.
        // This catches an ancestor directory replacement while the temporary
        // File was written; publication itself remains relative to `parent`.
        self.ensure_root_current()?;
        let (current_parent, current_name) = self.open_parent(&path)?;
        if current_name.to_bytes() != name.to_bytes()
            || !same_directory_identity(
                &metadata_snapshot(&current_parent, "write parent recheck")?,
                &parent_identity,
            )
        {
            return Err(ProjectFilesystemError::TargetChanged);
        }

        // Validate the expected destination before publishing. Existing files
        // keep their exact POSIX permission bits; replacing a Project script
        // must not silently strip executability or sharing permissions. The
        // content is written before `fchmod` so set-id bits are not cleared by
        // the write itself.
        match expected {
            ExpectedFileState::Absent => match stat_at(&parent, name) {
                Ok(_) => return Err(ProjectFilesystemError::Conflict),
                Err(error) if is_missing(&error) => {}
                Err(error) => return Err(map_path_error(error)),
            },
            ExpectedFileState::Existing(expected_version) => {
                let (_, current_version, current_metadata) =
                    match read_utf8_at(&parent, name, self.limits.max_file_bytes) {
                        Ok(current) => current,
                        Err(
                            ProjectFilesystemError::TargetUnavailable
                            | ProjectFilesystemError::SymlinkRejected
                            | ProjectFilesystemError::NotDirectory
                            | ProjectFilesystemError::NotRegularFile,
                        ) => return Err(ProjectFilesystemError::Conflict),
                        Err(error) => return Err(error),
                    };
                if !same_expected_file(&current_version, expected_version) {
                    return Err(ProjectFilesystemError::Conflict);
                }
                set_file_mode(&temporary, current_metadata.mode & 0o7777)?;
            }
        }
        temporary
            .sync_all()
            .map_err(|source| ProjectFilesystemError::Io {
                operation: "temporary File sync",
                source,
            })?;
        let temporary_metadata = metadata_snapshot(&temporary, "temporary File identity")?;
        let written_version = file_version(&temporary_metadata, content.as_bytes());
        drop(temporary);

        let created = match expected {
            ExpectedFileState::Absent => {
                match atomic_create(&parent, cleanup.name(), name) {
                    Ok(()) => cleanup.disarm(),
                    Err(error) if is_exists(&error) => {
                        return Err(ProjectFilesystemError::Conflict);
                    }
                    Err(error) if atomic_unsupported(&error) => {
                        return Err(ProjectFilesystemError::AtomicOperationUnavailable);
                    }
                    Err(source) => {
                        return Err(ProjectFilesystemError::Io {
                            operation: "atomic File create",
                            source,
                        });
                    }
                }
                true
            }
            ExpectedFileState::Existing(expected_version) => {
                match atomic_exchange(&parent, cleanup.name(), name) {
                    Ok(()) => {}
                    Err(error) if atomic_unsupported(&error) => {
                        return Err(ProjectFilesystemError::AtomicOperationUnavailable);
                    }
                    Err(error) if is_missing(&error) => {
                        return Err(ProjectFilesystemError::Conflict);
                    }
                    Err(source) => {
                        return Err(ProjectFilesystemError::Io {
                            operation: "atomic File exchange",
                            source,
                        });
                    }
                }

                // The old destination now occupies the temporary name. This
                // post-exchange check validates the exact object displaced by
                // the atomic operation, closing the destination-name race in
                // the pre-exchange check. A mismatch is exchanged back.
                let displaced =
                    read_version_at(&parent, cleanup.name(), self.limits.max_file_bytes);
                if !displaced
                    .as_ref()
                    .is_ok_and(|version| same_expected_file(version, expected_version))
                {
                    if atomic_exchange(&parent, cleanup.name(), name).is_err() {
                        cleanup.disarm();
                        return Err(ProjectFilesystemError::RecoveryFailed);
                    }
                    let restored = read_version_at(&parent, name, self.limits.max_file_bytes);
                    let staged =
                        read_version_at(&parent, cleanup.name(), self.limits.max_file_bytes);
                    if !restored.as_ref().is_ok_and(|version| {
                        displaced
                            .as_ref()
                            .is_ok_and(|displaced| same_expected_file(version, displaced))
                    }) || !staged
                        .as_ref()
                        .is_ok_and(|version| same_expected_file(version, &written_version))
                    {
                        cleanup.disarm();
                        return Err(ProjectFilesystemError::RecoveryFailed);
                    }
                    cleanup.remove()?;
                    sync_directory(&parent)?;
                    return Err(ProjectFilesystemError::Conflict);
                }
                cleanup.remove()?;
                false
            }
        };
        sync_directory(&parent)?;
        let live_version = read_version_at(&parent, name, self.limits.max_file_bytes)?;
        if live_version.identity != written_version.identity
            || live_version.content_sha256 != written_version.content_sha256
        {
            return Err(ProjectFilesystemError::TargetChanged);
        }
        Ok(FileWriteOutcome {
            relative_path: path.path,
            created,
            version: live_version,
        })
    }

    fn ensure_root_current(&self) -> Result<(), ProjectFilesystemError> {
        self.trusted_root
            .revalidate()
            .map_err(|_| ProjectFilesystemError::RootChanged)?;
        let metadata = self
            .root_directory
            .metadata()
            .map_err(|_| ProjectFilesystemError::RootChanged)?;
        if !metadata.is_dir()
            || metadata.dev() != self.trusted_root.identity().device
            || metadata.ino() != self.trusted_root.identity().inode
        {
            return Err(ProjectFilesystemError::RootChanged);
        }
        Ok(())
    }

    fn open_parent<'a>(
        &self,
        path: &'a NormalizedRelativePath,
    ) -> Result<(File, &'a CStr), ProjectFilesystemError> {
        let (name, ancestors) = path
            .components
            .split_last()
            .ok_or(ProjectFilesystemError::InvalidPath)?;
        let mut directory =
            self.root_directory
                .try_clone()
                .map_err(|source| ProjectFilesystemError::Io {
                    operation: "root descriptor clone",
                    source,
                })?;
        for component in ancestors {
            directory = open_directory_at(&directory, component)?;
        }
        Ok((directory, name))
    }

    fn open_directory(
        &self,
        path: &NormalizedRelativePath,
    ) -> Result<File, ProjectFilesystemError> {
        let mut directory =
            self.root_directory
                .try_clone()
                .map_err(|source| ProjectFilesystemError::Io {
                    operation: "root descriptor clone",
                    source,
                })?;
        for component in &path.components {
            directory = open_directory_at(&directory, component)?;
        }
        Ok(directory)
    }

    fn revalidate_opened_target(
        &self,
        path: &NormalizedRelativePath,
        expected_parent: &MetadataSnapshot,
        expected_target: &MetadataSnapshot,
    ) -> Result<(), ProjectFilesystemError> {
        let (parent, name) = self.open_parent(path)?;
        if metadata_snapshot(&parent, "read parent recheck")? != *expected_parent {
            return Err(ProjectFilesystemError::TargetChanged);
        }
        let file = open_regular_file_at(&parent, name)?;
        if metadata_snapshot(&file, "read target recheck")? != *expected_target {
            return Err(ProjectFilesystemError::TargetChanged);
        }
        Ok(())
    }
}

#[derive(Debug)]
struct NormalizedRelativePath {
    path: PathBuf,
    components: Vec<CString>,
}

impl NormalizedRelativePath {
    fn file(path: &Path) -> Result<Self, ProjectFilesystemError> {
        Self::new(path, false)
    }

    fn directory(path: &Path) -> Result<Self, ProjectFilesystemError> {
        Self::new(path, true)
    }

    fn new(path: &Path, allow_empty: bool) -> Result<Self, ProjectFilesystemError> {
        let mut component_names = Vec::new();
        let mut normalized = PathBuf::new();
        let mut total_bytes = 0_usize;
        for component in path.components() {
            let Component::Normal(name) = component else {
                return Err(ProjectFilesystemError::InvalidPath);
            };
            let bytes = name.as_bytes();
            if bytes.is_empty() || bytes.len() > MAX_PATH_COMPONENT_BYTES {
                return Err(ProjectFilesystemError::InvalidPath);
            }
            std::str::from_utf8(bytes).map_err(|_| ProjectFilesystemError::InvalidUtf8)?;
            total_bytes = total_bytes
                .checked_add(bytes.len())
                .and_then(|total| total.checked_add(usize::from(!component_names.is_empty())))
                .ok_or(ProjectFilesystemError::InvalidPath)?;
            if total_bytes > MAX_RELATIVE_PATH_BYTES
                || component_names.len() >= MAX_RELATIVE_PATH_COMPONENTS
            {
                return Err(ProjectFilesystemError::InvalidPath);
            }
            normalized.push(name);
            component_names
                .push(CString::new(bytes).map_err(|_| ProjectFilesystemError::InvalidPath)?);
        }
        if (!allow_empty && component_names.is_empty())
            || normalized.as_os_str().as_bytes() != path.as_os_str().as_bytes()
        {
            return Err(ProjectFilesystemError::InvalidPath);
        }
        Ok(Self {
            path: normalized,
            components: component_names,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetadataSnapshot {
    device: u64,
    inode: u64,
    mode: u32,
    byte_length: u64,
    modified_at_seconds: i64,
    modified_at_nanoseconds: i64,
    changed_at_seconds: i64,
    changed_at_nanoseconds: i64,
}

impl MetadataSnapshot {
    fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            mode: metadata.mode(),
            byte_length: metadata.len(),
            modified_at_seconds: metadata.mtime(),
            modified_at_nanoseconds: metadata.mtime_nsec(),
            changed_at_seconds: metadata.ctime(),
            changed_at_nanoseconds: metadata.ctime_nsec(),
        }
    }

    fn from_stat(stat: &libc::stat) -> Self {
        Self {
            device: stat.st_dev as u64,
            inode: stat.st_ino,
            mode: stat.st_mode as u32,
            byte_length: u64::try_from(stat.st_size).unwrap_or(0),
            modified_at_seconds: stat.st_mtime,
            modified_at_nanoseconds: stat.st_mtime_nsec,
            changed_at_seconds: stat.st_ctime,
            changed_at_nanoseconds: stat.st_ctime_nsec,
        }
    }

    fn file_type(&self) -> libc::mode_t {
        self.mode as libc::mode_t & libc::S_IFMT
    }

    fn is_directory(&self) -> bool {
        self.file_type() == libc::S_IFDIR
    }

    fn is_regular_file(&self) -> bool {
        self.file_type() == libc::S_IFREG
    }

    fn is_symlink(&self) -> bool {
        self.file_type() == libc::S_IFLNK
    }

    fn update_digest(&self, digest: &mut Sha256) {
        digest.update(self.device.to_le_bytes());
        digest.update(self.inode.to_le_bytes());
        digest.update(self.mode.to_le_bytes());
        digest.update(self.byte_length.to_le_bytes());
        digest.update(self.modified_at_seconds.to_le_bytes());
        digest.update(self.modified_at_nanoseconds.to_le_bytes());
        digest.update(self.changed_at_seconds.to_le_bytes());
        digest.update(self.changed_at_nanoseconds.to_le_bytes());
    }
}

#[derive(Debug)]
struct ScannedFolderEntry {
    public: FolderEntry,
    metadata: MetadataSnapshot,
}

fn open_absolute_directory(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    options.open(path)
}

fn open_directory_at(parent: &File, name: &CStr) -> Result<File, ProjectFilesystemError> {
    let metadata = stat_at(parent, name).map_err(map_path_error)?;
    if metadata.is_symlink() {
        return Err(ProjectFilesystemError::SymlinkRejected);
    }
    if !metadata.is_directory() {
        return Err(ProjectFilesystemError::NotDirectory);
    }
    open_at(parent, name, libc::O_RDONLY | libc::O_DIRECTORY).map_err(map_path_error)
}

fn open_regular_file_at(parent: &File, name: &CStr) -> Result<File, ProjectFilesystemError> {
    let metadata = stat_at(parent, name).map_err(map_path_error)?;
    if metadata.is_symlink() {
        return Err(ProjectFilesystemError::SymlinkRejected);
    }
    if !metadata.is_regular_file() {
        return Err(ProjectFilesystemError::NotRegularFile);
    }
    open_at(parent, name, libc::O_RDONLY).map_err(map_path_error)
}

fn open_at(parent: &File, name: &CStr, flags: libc::c_int) -> io::Result<File> {
    // SAFETY: `parent` is an owned live directory descriptor, `name` is one
    // NUL-terminated normalized component, and ownership of a successful new
    // descriptor is transferred exactly once into `File`.
    let descriptor = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor owned by this function.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

fn stat_at(parent: &File, name: &CStr) -> io::Result<MetadataSnapshot> {
    // SAFETY: zero is a valid initialization for `stat`; `fstatat` initializes
    // the structure before it is inspected on its successful return.
    let mut stat = unsafe { std::mem::zeroed::<libc::stat>() };
    // SAFETY: `parent` and `name` are live/valid for the call, and
    // `AT_SYMLINK_NOFOLLOW` ensures the directory entry itself is inspected.
    let result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            name.as_ptr(),
            &mut stat,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(MetadataSnapshot::from_stat(&stat))
}

fn metadata_snapshot(
    file: &File,
    operation: &'static str,
) -> Result<MetadataSnapshot, ProjectFilesystemError> {
    file.metadata()
        .map(|metadata| MetadataSnapshot::from_metadata(&metadata))
        .map_err(|source| ProjectFilesystemError::Io { operation, source })
}

fn read_utf8_at(
    parent: &File,
    name: &CStr,
    max_bytes: u64,
) -> Result<(String, FileVersion, MetadataSnapshot), ProjectFilesystemError> {
    let mut file = open_regular_file_at(parent, name)?;
    let before = metadata_snapshot(&file, "File identity read")?;
    if before.byte_length > max_bytes {
        return Err(ProjectFilesystemError::FileLimitExceeded { max_bytes });
    }
    let capacity = usize::try_from(before.byte_length)
        .map_err(|_| ProjectFilesystemError::FileLimitExceeded { max_bytes })?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| ProjectFilesystemError::Io {
            operation: "File read",
            source,
        })?;
    let after = metadata_snapshot(&file, "File identity recheck")?;
    if before != after || bytes.len() as u64 != before.byte_length {
        return Err(ProjectFilesystemError::TargetChanged);
    }
    let version = file_version(&before, &bytes);
    let content = String::from_utf8(bytes).map_err(|_| ProjectFilesystemError::InvalidUtf8)?;
    Ok((content, version, before))
}

fn read_version_at(
    parent: &File,
    name: &CStr,
    max_bytes: u64,
) -> Result<FileVersion, ProjectFilesystemError> {
    read_utf8_at(parent, name, max_bytes).map(|(_, version, _)| version)
}

fn file_version(metadata: &MetadataSnapshot, bytes: &[u8]) -> FileVersion {
    FileVersion {
        content_sha256: prefixed_sha256(bytes),
        identity: FilesystemObjectIdentity {
            device: metadata.device,
            inode: metadata.inode,
            kind: FilesystemObjectKind::File,
        },
        byte_length: metadata.byte_length,
        modified_at_seconds: metadata.modified_at_seconds,
        modified_at_nanoseconds: metadata.modified_at_nanoseconds,
        changed_at_seconds: metadata.changed_at_seconds,
        changed_at_nanoseconds: metadata.changed_at_nanoseconds,
    }
}

fn same_expected_file(left: &FileVersion, right: &FileVersion) -> bool {
    left.identity == right.identity
        && left.byte_length == right.byte_length
        && left.content_sha256 == right.content_sha256
}

fn same_directory_identity(left: &MetadataSnapshot, right: &MetadataSnapshot) -> bool {
    left.is_directory()
        && right.is_directory()
        && left.device == right.device
        && left.inode == right.inode
}

fn scan_directory(
    directory: &File,
    max_entries: usize,
    max_name_bytes: usize,
) -> Result<Vec<ScannedFolderEntry>, ProjectFilesystemError> {
    let mut names = BTreeSet::new();
    let mut entries = Vec::new();
    let mut aggregate_name_bytes = 0_usize;
    let mut stream = DirectoryStream::open(directory)?;
    while let Some(name) = stream.next_name()? {
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        if entries.len() >= max_entries {
            return Err(ProjectFilesystemError::FolderEntryLimitExceeded { max_entries });
        }
        aggregate_name_bytes = aggregate_name_bytes
            .checked_add(name.to_bytes().len())
            .filter(|total| *total <= max_name_bytes)
            .ok_or(ProjectFilesystemError::FolderNameLimitExceeded {
                max_bytes: max_name_bytes,
            })?;
        let name_string = name
            .to_str()
            .map(str::to_owned)
            .map_err(|_| ProjectFilesystemError::InvalidUtf8)?;
        if !names.insert(name_string.clone()) {
            return Err(ProjectFilesystemError::TargetChanged);
        }
        let metadata = stat_at(directory, &name).map_err(|error| {
            if is_missing(&error) {
                ProjectFilesystemError::TargetChanged
            } else {
                map_path_error(error)
            }
        })?;
        let (kind, byte_length) = if metadata.is_directory() {
            (FolderEntryKind::Directory, None)
        } else if metadata.is_regular_file() {
            (FolderEntryKind::File, Some(metadata.byte_length))
        } else if metadata.is_symlink() {
            (FolderEntryKind::Symlink, None)
        } else {
            (FolderEntryKind::Other, None)
        };
        entries.push(ScannedFolderEntry {
            public: FolderEntry {
                name: name_string,
                kind,
                byte_length,
            },
            metadata,
        });
    }
    entries.sort_by(|left, right| left.public.name.cmp(&right.public.name));
    Ok(entries)
}

struct DirectoryStream {
    raw: *mut libc::DIR,
}

impl DirectoryStream {
    fn open(directory: &File) -> Result<Self, ProjectFilesystemError> {
        let current_directory = c".";
        // `dup` would share the directory cursor and make later listings start
        // at EOF. Opening `.` relative to the descriptor creates an independent
        // open-file description while preserving the descriptor-rooted bound.
        let descriptor = open_at(
            directory,
            current_directory,
            libc::O_RDONLY | libc::O_DIRECTORY,
        )
        .map_err(|source| ProjectFilesystemError::Io {
            operation: "folder descriptor clone",
            source,
        })?
        .into_raw_fd();
        // SAFETY: ownership of the duplicate descriptor transfers to DIR on
        // success. On failure this function closes the descriptor itself.
        let raw = unsafe { libc::fdopendir(descriptor) };
        if raw.is_null() {
            let source = io::Error::last_os_error();
            // SAFETY: `fdopendir` failed, so it did not consume `descriptor`.
            unsafe { libc::close(descriptor) };
            return Err(ProjectFilesystemError::Io {
                operation: "folder stream open",
                source,
            });
        }
        Ok(Self { raw })
    }

    fn next_name(&mut self) -> Result<Option<CString>, ProjectFilesystemError> {
        set_errno(0);
        // SAFETY: `self.raw` is an owned live DIR until Drop. The returned
        // entry is copied before the next `readdir` call.
        let entry = unsafe { libc::readdir(self.raw) };
        if entry.is_null() {
            let error = errno();
            return if error == 0 {
                Ok(None)
            } else {
                Err(ProjectFilesystemError::Io {
                    operation: "folder read",
                    source: io::Error::from_raw_os_error(error),
                })
            };
        }
        // SAFETY: POSIX guarantees `d_name` is NUL-terminated for the live
        // directory entry returned by `readdir`.
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
        CString::new(name.to_bytes())
            .map(Some)
            .map_err(|_| ProjectFilesystemError::InvalidPath)
    }
}

impl Drop for DirectoryStream {
    fn drop(&mut self) {
        // SAFETY: `raw` is owned by this wrapper and closed exactly once.
        unsafe { libc::closedir(self.raw) };
    }
}

struct TemporaryCleanup {
    parent_fd: RawFd,
    name: CString,
    armed: bool,
}

impl TemporaryCleanup {
    fn name(&self) -> &CStr {
        &self.name
    }

    fn disarm(&mut self) {
        self.armed = false;
    }

    fn remove(&mut self) -> Result<(), ProjectFilesystemError> {
        // SAFETY: `parent_fd` remains live for this function and `name` is one
        // NUL-terminated component owned by this guard.
        let result = unsafe { libc::unlinkat(self.parent_fd, self.name.as_ptr(), 0) };
        if result != 0 {
            self.disarm();
            return Err(ProjectFilesystemError::RecoveryFailed);
        }
        self.disarm();
        Ok(())
    }
}

impl Drop for TemporaryCleanup {
    fn drop(&mut self) {
        if self.armed {
            // SAFETY: best-effort cleanup while the parent descriptor remains
            // live in the declaring write scope. Errors preserve the original
            // operation result.
            unsafe { libc::unlinkat(self.parent_fd, self.name.as_ptr(), 0) };
        }
    }
}

fn create_temporary(parent: &File) -> Result<(File, TemporaryCleanup), ProjectFilesystemError> {
    for _ in 0..TEMPORARY_ATTEMPTS {
        let name = CString::new(format!("{TEMPORARY_PREFIX}{}", Uuid::new_v4()))
            .expect("UUID temporary names contain no NUL");
        // SAFETY: `parent` and `name` are live, the mode is explicit, and a
        // successful descriptor is transferred once into `File`.
        let descriptor = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600 as libc::c_uint,
            )
        };
        if descriptor >= 0 {
            // SAFETY: `openat` returned a new owned descriptor.
            let file = unsafe { File::from_raw_fd(descriptor) };
            return Ok((
                file,
                TemporaryCleanup {
                    parent_fd: parent.as_raw_fd(),
                    name,
                    armed: true,
                },
            ));
        }
        let error = io::Error::last_os_error();
        if !is_exists(&error) {
            return Err(ProjectFilesystemError::Io {
                operation: "temporary File create",
                source: error,
            });
        }
    }
    Err(ProjectFilesystemError::Io {
        operation: "temporary File create",
        source: io::Error::new(io::ErrorKind::AlreadyExists, "temporary name collision"),
    })
}

fn set_file_mode(file: &File, mode: u32) -> Result<(), ProjectFilesystemError> {
    // SAFETY: `file` owns a live descriptor and `mode` contains only the
    // existing target's POSIX permission and special bits.
    let result = unsafe { libc::fchmod(file.as_raw_fd(), mode as libc::mode_t) };
    if result == 0 {
        Ok(())
    } else {
        Err(ProjectFilesystemError::Io {
            operation: "temporary File mode preservation",
            source: io::Error::last_os_error(),
        })
    }
}

#[cfg(target_os = "macos")]
fn atomic_create(parent: &File, temporary: &CStr, target: &CStr) -> io::Result<()> {
    // SAFETY: both names are normalized single components in the same owned
    // directory. `RENAME_EXCL` guarantees create-if-absent publication.
    let result = unsafe {
        libc::renameatx_np(
            parent.as_raw_fd(),
            temporary.as_ptr(),
            parent.as_raw_fd(),
            target.as_ptr(),
            libc::RENAME_EXCL,
        )
    };
    syscall_result(result)
}

#[cfg(target_os = "macos")]
fn atomic_exchange(parent: &File, temporary: &CStr, target: &CStr) -> io::Result<()> {
    // SAFETY: both names are normalized single components in the same owned
    // directory. `RENAME_SWAP` atomically exchanges their directory entries.
    let result = unsafe {
        libc::renameatx_np(
            parent.as_raw_fd(),
            temporary.as_ptr(),
            parent.as_raw_fd(),
            target.as_ptr(),
            libc::RENAME_SWAP,
        )
    };
    syscall_result(result)
}

#[cfg(target_os = "linux")]
fn atomic_create(parent: &File, temporary: &CStr, target: &CStr) -> io::Result<()> {
    renameat2(parent, temporary, target, libc::RENAME_NOREPLACE)
}

#[cfg(target_os = "linux")]
fn atomic_exchange(parent: &File, temporary: &CStr, target: &CStr) -> io::Result<()> {
    renameat2(parent, temporary, target, libc::RENAME_EXCHANGE)
}

#[cfg(target_os = "linux")]
fn renameat2(parent: &File, source: &CStr, target: &CStr, flags: libc::c_uint) -> io::Result<()> {
    // SAFETY: names and directory descriptors are valid for the syscall.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            parent.as_raw_fd(),
            source.as_ptr(),
            parent.as_raw_fd(),
            target.as_ptr(),
            flags,
        )
    };
    syscall_result(result as libc::c_int)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn atomic_create(_parent: &File, _temporary: &CStr, _target: &CStr) -> io::Result<()> {
    Err(io::Error::from_raw_os_error(libc::ENOTSUP))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn atomic_exchange(_parent: &File, _temporary: &CStr, _target: &CStr) -> io::Result<()> {
    Err(io::Error::from_raw_os_error(libc::ENOTSUP))
}

fn syscall_result(result: libc::c_int) -> io::Result<()> {
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn sync_directory(directory: &File) -> Result<(), ProjectFilesystemError> {
    directory
        .sync_all()
        .map_err(|source| ProjectFilesystemError::Io {
            operation: "parent directory sync",
            source,
        })
}

fn map_path_error(error: io::Error) -> ProjectFilesystemError {
    match error.raw_os_error() {
        Some(libc::ELOOP) => ProjectFilesystemError::SymlinkRejected,
        Some(libc::ENOENT) => ProjectFilesystemError::TargetUnavailable,
        Some(libc::ENOTDIR) => ProjectFilesystemError::NotDirectory,
        _ => ProjectFilesystemError::Io {
            operation: "descriptor-relative path open",
            source: error,
        },
    }
}

fn is_missing(error: &io::Error) -> bool {
    error.raw_os_error() == Some(libc::ENOENT)
}

fn is_exists(error: &io::Error) -> bool {
    error.raw_os_error() == Some(libc::EEXIST)
}

fn atomic_unsupported(error: &io::Error) -> bool {
    error.raw_os_error().is_some_and(|code| {
        code == libc::ENOSYS || code == libc::ENOTSUP || code == libc::EOPNOTSUPP
    })
}

fn prefixed_sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    prefixed_digest(digest)
}

fn prefixed_digest(digest: Sha256) -> String {
    let bytes = digest.finalize();
    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in bytes {
        use fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("String writes are infallible");
    }
    encoded
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn set_errno(value: libc::c_int) {
    // SAFETY: `__error` returns the calling thread's errno storage.
    unsafe { *libc::__error() = value };
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn errno() -> libc::c_int {
    // SAFETY: `__error` returns the calling thread's errno storage.
    unsafe { *libc::__error() }
}

#[cfg(target_os = "linux")]
fn set_errno(value: libc::c_int) {
    // SAFETY: `__errno_location` returns the calling thread's errno storage.
    unsafe { *libc::__errno_location() = value };
}

#[cfg(target_os = "linux")]
fn errno() -> libc::c_int {
    // SAFETY: `__errno_location` returns the calling thread's errno storage.
    unsafe { *libc::__errno_location() }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux")))]
fn set_errno(_value: libc::c_int) {}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "linux")))]
fn errno() -> libc::c_int {
    0
}
