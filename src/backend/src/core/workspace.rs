//! Rust-owned Workspace layout, archive, locking, and Project-reference lifecycle.
//!
//! The unpacked working copy is authoritative while open. A Workspace zip is a
//! validated portable snapshot only; Project folders remain external trusted
//! roots and are never written by this module.

use fs4::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const WORKSPACE_SCHEMA_VERSION: u16 = 1;
pub const MANIFEST_FILE: &str = "manifest.toml";
pub const WORKSPACE_DATABASE_FILE: &str = "state/workspace.sqlite3";
pub const WORKSPACE_CONFIGURATION_FILE: &str = "config/workspace.toml";

const MAX_ENTRY_NAME_BYTES: usize = 1_024;
const SHA256_HEX_BYTES: usize = 64;
const MAX_CENTRAL_DIRECTORY_BYTES: u64 = 16 * 1024 * 1024;
const END_OF_CENTRAL_DIRECTORY_BYTES: usize = 22;
const MAX_ZIP_COMMENT_BYTES: usize = u16::MAX as usize;
const ZIP_PROBE_TAIL_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveLimits {
    pub max_entries: usize,
    pub max_manifest_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_expanded_bytes: u64,
    pub max_compression_ratio: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceArchiveSourceFile {
    path: String,
    source: PathBuf,
    expanded_size: u64,
}

/// Side-effect-free, bounded validation result for the authoritative source
/// tree used by a production Workspace archive save.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceArchiveSourcePreflight {
    pub entry_count: usize,
    pub manifest_bytes: u64,
    pub expanded_bytes: u64,
    files: Vec<WorkspaceArchiveSourceFile>,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_manifest_bytes: 1_048_576,
            max_entry_bytes: 268_435_456,
            max_expanded_bytes: 2_147_483_648,
            max_compression_ratio: 200,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArchiveViolation {
    MissingManifest,
    DuplicateEntry(String),
    InvalidEntryName(String),
    DisallowedRoot(String),
    UnsupportedEntryType(String),
    EncryptedEntry(String),
    UnsupportedCompression(String),
    EntryCountExceeded,
    EntrySizeExceeded(String),
    ExpandedSizeExceeded,
    CompressionRatioExceeded(String),
    UndeclaredEntry(String),
    MissingDeclaredEntry(String),
    DeclaredSizeMismatch(String),
    DigestMismatch(String),
    BlobDigestMismatch(String),
    InvalidManifest(String),
    UnsupportedSchema(u16),
    MinimumVersionNotMet(String),
}

impl std::fmt::Display for ArchiveViolation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifest => formatter.write_str("Workspace manifest is missing"),
            Self::DuplicateEntry(path) => write!(formatter, "duplicate archive entry: {path}"),
            Self::InvalidEntryName(path) => write!(formatter, "invalid archive entry name: {path}"),
            Self::DisallowedRoot(path) => {
                write!(formatter, "entry is outside the archive allowlist: {path}")
            }
            Self::UnsupportedEntryType(path) => {
                write!(formatter, "unsupported archive entry type: {path}")
            }
            Self::EncryptedEntry(path) => write!(formatter, "encrypted archive entry: {path}"),
            Self::UnsupportedCompression(path) => {
                write!(formatter, "unsupported compression method: {path}")
            }
            Self::EntryCountExceeded => formatter.write_str("archive entry-count limit exceeded"),
            Self::EntrySizeExceeded(path) => {
                write!(formatter, "archive entry size limit exceeded: {path}")
            }
            Self::ExpandedSizeExceeded => {
                formatter.write_str("archive expanded-size limit exceeded")
            }
            Self::CompressionRatioExceeded(path) => write!(
                formatter,
                "archive compression-ratio limit exceeded: {path}"
            ),
            Self::UndeclaredEntry(path) => {
                write!(formatter, "archive entry is not declared: {path}")
            }
            Self::MissingDeclaredEntry(path) => {
                write!(formatter, "declared archive entry is missing: {path}")
            }
            Self::DeclaredSizeMismatch(path) => write!(
                formatter,
                "declared archive entry size is incorrect: {path}"
            ),
            Self::DigestMismatch(path) => {
                write!(formatter, "archive entry digest is incorrect: {path}")
            }
            Self::BlobDigestMismatch(path) => {
                write!(
                    formatter,
                    "content-addressed blob path is incorrect: {path}"
                )
            }
            Self::InvalidManifest(message) => {
                write!(formatter, "invalid Workspace manifest: {message}")
            }
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported Workspace schema version: {version}")
            }
            Self::MinimumVersionNotMet(version) => {
                write!(formatter, "Workspace requires C4OS {version} or newer")
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("zip operation failed: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("TOML decode failed: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("TOML encode failed: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("hostile or incompatible Workspace archive: {0}")]
    Archive(ArchiveViolation),
    #[error("Workspace lifecycle conflict: {0}")]
    Conflict(String),
    #[error(
        "Workspace {operation} rollback failed; retained recovery state at {recovery_path}: {message}"
    )]
    CommitRecovery {
        operation: &'static str,
        recovery_path: PathBuf,
        message: String,
    },
    #[error("invalid Project reference: {0}")]
    InvalidProject(String),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct C4osHomeLayout {
    root: PathBuf,
}

impl C4osHomeLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn app_configuration(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn app_database(&self) -> PathBuf {
        self.root.join("state/app.sqlite3")
    }

    pub fn credential_vault(&self) -> PathBuf {
        self.root.join("vault/credentials.vault")
    }

    pub fn active_workspace(&self) -> PathBuf {
        self.root.join("workspace/active")
    }

    pub fn workspace_recovery_root(&self) -> PathBuf {
        self.root.join("workspace/recovery")
    }

    pub fn workspace_lock(&self) -> PathBuf {
        self.root.join("workspace/active.lock")
    }

    pub fn browser_profile_registry(&self) -> PathBuf {
        self.root.join("browser/profiles.toml")
    }

    pub fn ensure_roots(&self) -> WorkspaceResult<()> {
        for relative in [
            "state",
            "vault",
            "workspace",
            "workspace/recovery",
            "mcp",
            "skills/user",
            "plugins",
            "extensions/packages/sha256",
            "extensions/staging",
            "runtimes",
            "browser",
            "cache",
            "logs",
            "tmp",
        ] {
            fs::create_dir_all(self.root.join(relative))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceLayout {
    root: PathBuf,
}

impl WorkspaceLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> PathBuf {
        self.root.join(MANIFEST_FILE)
    }

    pub fn database(&self) -> PathBuf {
        self.root.join(WORKSPACE_DATABASE_FILE)
    }

    pub fn workspace_configuration(&self) -> PathBuf {
        self.root.join(WORKSPACE_CONFIGURATION_FILE)
    }

    pub fn project_configuration(&self, project_id: Uuid) -> PathBuf {
        self.root.join(format!("projects/{project_id}/config.toml"))
    }

    pub fn chat_configuration(&self, chat_id: Uuid) -> PathBuf {
        self.root.join(format!("chats/{chat_id}/config.toml"))
    }

    pub fn blob(&self, digest: &str) -> PathBuf {
        self.root.join(format!("blobs/sha256/{digest}"))
    }

    pub fn ensure_roots(&self) -> WorkspaceResult<()> {
        for relative in [
            "state",
            "config",
            "projects",
            "chats",
            "blobs/sha256",
            "skills",
        ] {
            fs::create_dir_all(self.root.join(relative))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    pub schema_version: u16,
    pub workspace_id: Uuid,
    pub generation: u64,
    pub minimum_app_version: String,
    #[serde(default)]
    pub projects: Vec<ProjectReference>,
    #[serde(default)]
    pub files: Vec<ManifestFile>,
}

impl WorkspaceManifest {
    pub fn new(minimum_app_version: impl Into<String>) -> Self {
        Self {
            schema_version: WORKSPACE_SCHEMA_VERSION,
            workspace_id: Uuid::new_v4(),
            generation: 0,
            minimum_app_version: minimum_app_version.into(),
            projects: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn add_project(&mut self, project: ProjectReference) -> WorkspaceResult<()> {
        project.validate()?;
        if self
            .projects
            .iter()
            .any(|existing| existing.project_id == project.project_id)
        {
            return Err(WorkspaceError::Conflict(
                "Project is already referenced".into(),
            ));
        }
        self.projects.push(project);
        self.advance_generation()?;
        Ok(())
    }

    pub fn relocate_project(&mut self, project_id: Uuid, new_path: &Path) -> WorkspaceResult<()> {
        let normalized = external_project_path(new_path)?;
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| WorkspaceError::Conflict("Project reference was not found".into()))?;
        project.last_known_path = normalized;
        // Trust is local runtime authority and never follows a relocated path.
        project.trusted_root = false;
        self.advance_generation()?;
        Ok(())
    }

    /// Applies a grant resolved from local trusted-root authority. This flag is
    /// runtime-only, is omitted from Workspace manifests, and does not advance
    /// the portable Workspace generation.
    pub fn resolve_project_trust(
        &mut self,
        project_id: Uuid,
        trusted_root_granted: bool,
    ) -> WorkspaceResult<()> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| WorkspaceError::Conflict("Project reference was not found".into()))?;
        project.trusted_root = trusted_root_granted;
        Ok(())
    }

    pub fn reorder_projects(&mut self, ordered_ids: &[Uuid]) -> WorkspaceResult<()> {
        if ordered_ids.len() != self.projects.len()
            || ordered_ids.iter().copied().collect::<BTreeSet<_>>().len() != ordered_ids.len()
        {
            return Err(WorkspaceError::Conflict(
                "Project reorder must contain every Project exactly once".into(),
            ));
        }
        let mut by_id: BTreeMap<Uuid, ProjectReference> = self
            .projects
            .drain(..)
            .map(|project| (project.project_id, project))
            .collect();
        let reordered = ordered_ids
            .iter()
            .map(|project_id| {
                by_id.remove(project_id).ok_or_else(|| {
                    WorkspaceError::Conflict("Project reorder contains an unknown Project".into())
                })
            })
            .collect::<WorkspaceResult<Vec<_>>>()?;
        self.projects = reordered;
        self.advance_generation()?;
        Ok(())
    }

    pub fn project_status(&self, project_id: Uuid) -> WorkspaceResult<ProjectPathStatus> {
        let project = self
            .projects
            .iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| WorkspaceError::Conflict("Project reference was not found".into()))?;
        let path = Path::new(&project.last_known_path);
        if !path.is_dir() {
            Ok(ProjectPathStatus::Missing)
        } else if project.trusted_root {
            Ok(ProjectPathStatus::Trusted)
        } else {
            Ok(ProjectPathStatus::NeedsTrust)
        }
    }

    pub fn advance_generation(&mut self) -> WorkspaceResult<u64> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| WorkspaceError::Conflict("Workspace generation overflow".into()))?;
        Ok(self.generation)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectReference {
    pub project_id: Uuid,
    pub display_name: String,
    pub last_known_path: String,
    /// Local trust resolution is never portable or deserializable.
    #[serde(skip)]
    pub trusted_root: bool,
}

impl ProjectReference {
    pub fn from_folder(
        project_folder: &Path,
        display_name: impl Into<String>,
    ) -> WorkspaceResult<Self> {
        let reference = Self {
            project_id: Uuid::new_v4(),
            display_name: display_name.into(),
            last_known_path: external_project_path(project_folder)?,
            trusted_root: false,
        };
        reference.validate()?;
        Ok(reference)
    }

    fn validate(&self) -> WorkspaceResult<()> {
        let name = self.display_name.trim();
        if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
            return Err(WorkspaceError::InvalidProject(
                "invalid display name".into(),
            ));
        }
        external_project_path(Path::new(&self.last_known_path))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectPathStatus {
    Trusted,
    NeedsTrust,
    Missing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
    pub expanded_size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedArchive {
    pub manifest: WorkspaceManifest,
    pub expanded_bytes: u64,
    pub entry_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceLockOwner {
    pub process_id: u32,
    pub app_instance_id: Uuid,
    pub acquired_unix_ms: u64,
    pub label: String,
}

pub struct WorkspaceWriterLock {
    file: File,
    path: PathBuf,
    owner: WorkspaceLockOwner,
}

impl std::fmt::Debug for WorkspaceWriterLock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkspaceWriterLock")
            .field("path", &self.path)
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

impl WorkspaceWriterLock {
    pub fn owner(&self) -> &WorkspaceLockOwner {
        &self.owner
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for WorkspaceWriterLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

#[derive(Debug)]
pub enum WriterAccess {
    Writable(WorkspaceWriterLock),
    ReadOnly { owner: Option<WorkspaceLockOwner> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDisposition {
    Current,
    RecoverWorkingCopy {
        working_generation: u64,
        archive_generation: u64,
    },
    ArchiveAhead {
        working_generation: u64,
        archive_generation: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryNotice {
    pub workspace_id: Uuid,
    pub working_generation: u64,
    pub archive_generation: u64,
    pub must_notify_before_next_save: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecentWorkspaceHandoff {
    pub workspace_id: Uuid,
    pub archive_path: Option<PathBuf>,
    pub last_known_generation: u64,
    pub project_count: usize,
}

#[derive(Debug)]
pub struct WritableWorkspace {
    pub manifest: WorkspaceManifest,
    pub working_root: PathBuf,
    pub archive_path: PathBuf,
    pub recovery_notice: Option<RecoveryNotice>,
    pub writer_lock: WorkspaceWriterLock,
}

#[derive(Debug)]
enum OpenWorkspaceRollback {
    Promoted {
        active_root: PathBuf,
        previous_active: Option<PathBuf>,
        recovery_root: PathBuf,
        workspace_parent: PathBuf,
    },
}

/// A validated and promoted Workspace whose prior active state remains armed
/// for rollback until the caller finishes database activation and app-owned
/// recent-state persistence.
///
/// Callers must drop any database actor opened against `workspace()` before
/// calling `abort` or allowing this guard to drop.
#[derive(Debug)]
pub struct PendingWorkspaceOpen {
    workspace: Option<WritableWorkspace>,
    rollback: Option<OpenWorkspaceRollback>,
}

impl PendingWorkspaceOpen {
    pub fn workspace(&self) -> &WritableWorkspace {
        self.workspace
            .as_ref()
            .expect("pending Workspace open always owns its Workspace")
    }

    pub fn commit(mut self) -> WritableWorkspace {
        self.rollback = None;
        self.workspace
            .take()
            .expect("pending Workspace open always owns its Workspace")
    }

    pub fn abort(mut self) -> WorkspaceResult<()> {
        self.rollback_inner()
    }

    fn rollback_inner(&mut self) -> WorkspaceResult<()> {
        let Some(OpenWorkspaceRollback::Promoted {
            active_root,
            previous_active,
            recovery_root,
            workspace_parent,
        }) = self.rollback.as_ref()
        else {
            return Ok(());
        };
        let active_root = active_root.clone();
        let previous_active = previous_active.clone();
        let recovery_root = recovery_root.clone();
        let workspace_parent = workspace_parent.clone();
        let rejected_candidate =
            recovery_root.join(format!("aborted-candidate-{}", Uuid::new_v4()));

        if let Err(error) = commit_rename(&active_root, &rejected_candidate) {
            return Err(commit_recovery_error(
                "open",
                previous_active.as_deref().unwrap_or(&active_root),
                error,
            ));
        }
        if let Some(previous_active) = &previous_active
            && let Err(error) = commit_rename(previous_active, &active_root)
        {
            let _ = commit_rename(&rejected_candidate, &active_root);
            return Err(commit_recovery_error("open", previous_active, error));
        }
        self.rollback = None;
        if let Err(error) = commit_sync_directory(&workspace_parent) {
            return Err(commit_recovery_error(
                "open",
                previous_active
                    .as_ref()
                    .map_or(rejected_candidate.as_path(), |_| active_root.as_path()),
                error,
            ));
        }

        let _ = fs::remove_dir_all(&rejected_candidate);
        let _ = sync_directory(&recovery_root);
        Ok(())
    }
}

impl Drop for PendingWorkspaceOpen {
    fn drop(&mut self) {
        let _ = self.rollback_inner();
    }
}

impl WritableWorkspace {
    pub fn recent_handoff(&self) -> RecentWorkspaceHandoff {
        RecentWorkspaceHandoff {
            workspace_id: self.manifest.workspace_id,
            archive_path: Some(self.archive_path.clone()),
            last_known_generation: self.manifest.generation,
            project_count: self.manifest.projects.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadOnlyWorkspace {
    pub manifest: WorkspaceManifest,
    pub archive_path: PathBuf,
    pub owner: Option<WorkspaceLockOwner>,
}

#[derive(Debug)]
pub enum OpenWorkspaceOutcome {
    Writable(WritableWorkspace),
    ReadOnly(ReadOnlyWorkspace),
}

#[derive(Debug)]
pub enum PreparedOpenWorkspaceOutcome {
    Writable(PendingWorkspaceOpen),
    ReadOnly(ReadOnlyWorkspace),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedWorkspace {
    pub manifest: WorkspaceManifest,
    pub archive_path: PathBuf,
}

#[derive(Debug)]
enum ArchiveSaveRollback {
    RestorePrevious(PathBuf),
    RemoveNew,
}

/// A fully validated and durably renamed archive whose prior destination is
/// retained until the caller commits app-owned save bookkeeping.
#[derive(Debug)]
pub struct PendingWorkspaceSave {
    saved: Option<SavedWorkspace>,
    rollback: Option<ArchiveSaveRollback>,
    archive_parent: PathBuf,
}

impl PendingWorkspaceSave {
    pub fn saved(&self) -> &SavedWorkspace {
        self.saved
            .as_ref()
            .expect("pending Workspace save always owns its result")
    }

    pub fn commit(mut self) -> SavedWorkspace {
        // The coordinated caller invokes commit only after app-owned recent
        // persistence succeeds. From this point the new archive is the
        // committed result; retained-backup cleanup cannot reverse that
        // decision or turn the completed save into an error.
        let rollback = self.rollback.take();
        if let Some(ArchiveSaveRollback::RestorePrevious(previous)) = rollback {
            let _ = fs::remove_file(previous);
            let _ = sync_directory(&self.archive_parent);
        }
        self.saved
            .take()
            .expect("pending Workspace save always owns its result")
    }

    pub fn abort(mut self) -> WorkspaceResult<()> {
        self.rollback_inner()
    }

    fn rollback_inner(&mut self) -> WorkspaceResult<()> {
        let Some(saved) = self.saved.as_ref() else {
            return Ok(());
        };
        let archive_path = saved.archive_path.clone();
        match self.rollback.as_ref() {
            Some(ArchiveSaveRollback::RestorePrevious(previous)) => {
                if let Err(error) = commit_rename(previous, &archive_path) {
                    return Err(commit_recovery_error("save", previous, error));
                }
            }
            Some(ArchiveSaveRollback::RemoveNew) => {
                if let Err(error) = fs::remove_file(&archive_path)
                    && error.kind() != io::ErrorKind::NotFound
                {
                    return Err(commit_recovery_error("save", &archive_path, error));
                }
            }
            None => return Ok(()),
        }
        self.rollback = None;
        if let Err(error) = commit_sync_directory(&self.archive_parent) {
            return Err(commit_recovery_error("save", &archive_path, error));
        }
        Ok(())
    }
}

impl Drop for PendingWorkspaceSave {
    fn drop(&mut self) {
        let _ = self.rollback_inner();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceSemanticValidationTarget {
    ActiveRecovery,
    ArchiveCandidate,
}

pub fn acquire_workspace_writer_lock(
    path: &Path,
    owner: WorkspaceLockOwner,
) -> WorkspaceResult<WriterAccess> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    match FileExt::try_lock(&file) {
        Ok(()) => {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            serde_json::to_writer(&mut file, &owner)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            file.flush()?;
            file.sync_all()?;
            Ok(WriterAccess::Writable(WorkspaceWriterLock {
                file,
                path: path.to_path_buf(),
                owner,
            }))
        }
        Err(fs4::TryLockError::WouldBlock) => {
            file.seek(SeekFrom::Start(0))?;
            let mut bytes = Vec::new();
            file.take(64 * 1024).read_to_end(&mut bytes)?;
            let owner = serde_json::from_slice(&bytes).ok();
            Ok(WriterAccess::ReadOnly { owner })
        }
        Err(fs4::TryLockError::Error(error)) => Err(error.into()),
    }
}

pub fn create_untitled_working_copy(
    _writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    project_folder: &Path,
    display_name: impl Into<String>,
    minimum_app_version: &str,
) -> WorkspaceResult<WorkspaceManifest> {
    if working_root.exists() && fs::read_dir(working_root)?.next().is_some() {
        return Err(WorkspaceError::Conflict(
            "untitled working-copy destination is not empty".into(),
        ));
    }
    let layout = WorkspaceLayout::new(working_root);
    layout.ensure_roots()?;
    let mut manifest = WorkspaceManifest::new(minimum_app_version);
    manifest.add_project(ProjectReference::from_folder(project_folder, display_name)?)?;
    let project_id = manifest.projects[0].project_id;
    manifest.resolve_project_trust(project_id, true)?;
    persist_working_manifest(working_root, &manifest)?;
    Ok(manifest)
}

pub fn persist_committed_generation(
    _writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    manifest: &mut WorkspaceManifest,
) -> WorkspaceResult<u64> {
    let mut candidate = manifest.clone();
    let generation = candidate.advance_generation()?;
    let refreshed = manifest_for_snapshot(working_root, candidate)?;
    persist_working_manifest(working_root, &refreshed)?;
    *manifest = refreshed;
    Ok(generation)
}

/// Persists a caller-derived canonical manifest while holding the Workspace
/// writer lock. File declarations are rebuilt from the authoritative working
/// tree, but the supplied generation and semantic records are not advanced or
/// otherwise rewritten by this primitive.
pub fn persist_canonical_working_manifest(
    _writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    canonical_manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<WorkspaceManifest> {
    validate_manifest(canonical_manifest, current_app_version)?;
    let refreshed = manifest_for_snapshot(working_root, canonical_manifest.clone())?;
    validate_manifest(&refreshed, current_app_version)?;
    validate_manifest_limits(&refreshed, limits)?;
    persist_working_manifest(working_root, &refreshed)?;
    Ok(refreshed)
}

pub fn detect_recovery(
    working: &WorkspaceManifest,
    archive: &WorkspaceManifest,
) -> WorkspaceResult<RecoveryDisposition> {
    if working.workspace_id != archive.workspace_id {
        return Err(WorkspaceError::Conflict(
            "recovery manifests belong to different Workspaces".into(),
        ));
    }
    Ok(match working.generation.cmp(&archive.generation) {
        std::cmp::Ordering::Greater => RecoveryDisposition::RecoverWorkingCopy {
            working_generation: working.generation,
            archive_generation: archive.generation,
        },
        std::cmp::Ordering::Less => RecoveryDisposition::ArchiveAhead {
            working_generation: working.generation,
            archive_generation: archive.generation,
        },
        std::cmp::Ordering::Equal => RecoveryDisposition::Current,
    })
}

pub fn validate_archive(
    archive_path: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<ValidatedArchive> {
    inspect_archive(archive_path, current_app_version, limits, None)
}

pub fn validate_working_copy(
    working_root: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<WorkspaceManifest> {
    let manifest_size = fs::metadata(working_root.join(MANIFEST_FILE))?.len();
    if manifest_size > limits.max_manifest_bytes || manifest_size > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(MANIFEST_FILE.into()));
    }
    let manifest = load_working_manifest(working_root)?;
    validate_manifest(&manifest, current_app_version)?;
    let refreshed = manifest_for_snapshot(working_root, manifest.clone())?;
    if refreshed.files != manifest.files {
        let expected: BTreeMap<_, _> = manifest
            .files
            .iter()
            .map(|file| (file.path.as_str(), (&file.sha256, file.expanded_size)))
            .collect();
        let actual: BTreeMap<_, _> = refreshed
            .files
            .iter()
            .map(|file| (file.path.as_str(), (&file.sha256, file.expanded_size)))
            .collect();
        let path = expected
            .keys()
            .chain(actual.keys())
            .find(|path| expected.get(**path) != actual.get(**path))
            .copied()
            .unwrap_or("unknown");
        return Err(WorkspaceError::Archive(ArchiveViolation::DigestMismatch(
            path.to_string(),
        )));
    }
    let expanded = manifest
        .files
        .iter()
        .try_fold(manifest_size, |total, file| {
            if file.expanded_size > limits.max_entry_bytes {
                return archive_error(ArchiveViolation::EntrySizeExceeded(file.path.clone()));
            }
            total
                .checked_add(file.expanded_size)
                .ok_or(WorkspaceError::Archive(
                    ArchiveViolation::ExpandedSizeExceeded,
                ))
        })?;
    let entry_count = manifest
        .files
        .len()
        .checked_add(1)
        .ok_or(WorkspaceError::Archive(
            ArchiveViolation::EntryCountExceeded,
        ))?;
    if entry_count > limits.max_entries {
        return archive_error(ArchiveViolation::EntryCountExceeded);
    }
    if expanded > limits.max_expanded_bytes {
        return Err(WorkspaceError::Archive(
            ArchiveViolation::ExpandedSizeExceeded,
        ));
    }
    Ok(manifest)
}

/// Performs the complete production archive-source preflight without creating
/// directories, temporary files, SQLite backups, or staging copies.
pub fn preflight_workspace_archive_source(
    working_root: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<WorkspaceArchiveSourcePreflight> {
    validate_manifest(manifest, current_app_version)?;
    if limits.max_entries == 0 {
        return archive_error(ArchiveViolation::EntryCountExceeded);
    }

    let mut files = Vec::new();
    let mut entry_count = 1_usize;
    let mut source_expanded_bytes = 0_u64;
    collect_source_paths(
        working_root,
        limits,
        &mut entry_count,
        &mut source_expanded_bytes,
        &mut files,
    )?;
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let mut preflight_manifest = manifest.clone();
    preflight_manifest.files = files
        .iter()
        .map(|file| ManifestFile {
            path: file.path.clone(),
            // SHA-256 hex has a fixed serialized width; placeholder content
            // gives the exact manifest byte budget without hashing or staging.
            sha256: "0".repeat(SHA256_HEX_BYTES),
            expanded_size: file.expanded_size,
        })
        .collect();
    let manifest_bytes = toml::to_string(&preflight_manifest)?.len() as u64;
    if manifest_bytes > limits.max_manifest_bytes || manifest_bytes > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(MANIFEST_FILE.into()));
    }
    let expanded_bytes =
        source_expanded_bytes
            .checked_add(manifest_bytes)
            .ok_or(WorkspaceError::Archive(
                ArchiveViolation::ExpandedSizeExceeded,
            ))?;
    if expanded_bytes > limits.max_expanded_bytes {
        return archive_error(ArchiveViolation::ExpandedSizeExceeded);
    }

    for file in &files {
        validate_file_compression_ratio(&file.source, &file.path, limits)?;
    }

    Ok(WorkspaceArchiveSourcePreflight {
        entry_count,
        manifest_bytes,
        expanded_bytes,
        files,
    })
}

pub fn save_workspace_archive_by_copy(
    writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<SavedWorkspace> {
    save_workspace_archive(
        writer_lock,
        working_root,
        archive_path,
        manifest,
        current_app_version,
        limits,
        |source, destination| {
            copy_regular_file(source, destination)?;
            Ok(())
        },
    )
}

/// Saves a portable snapshot. `backup_workspace_database` must create a
/// consistent SQLite online backup at `destination`; this module intentionally
/// has no database ownership or connection dependency.
pub fn save_workspace_archive<F>(
    writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
    backup_workspace_database: F,
) -> WorkspaceResult<SavedWorkspace>
where
    F: FnOnce(&Path, &Path) -> WorkspaceResult<()>,
{
    Ok(prepare_workspace_archive_save(
        writer_lock,
        working_root,
        archive_path,
        manifest,
        current_app_version,
        limits,
        backup_workspace_database,
    )?
    .commit())
}

/// Prepares a portable snapshot and atomically promotes it while retaining the
/// prior archive until the caller commits app-owned save bookkeeping.
/// Dropping or aborting the returned guard restores the prior archive (or
/// removes the newly created archive when there was no prior destination).
pub fn prepare_workspace_archive_save<F>(
    _writer_lock: &WorkspaceWriterLock,
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
    backup_workspace_database: F,
) -> WorkspaceResult<PendingWorkspaceSave>
where
    F: FnOnce(&Path, &Path) -> WorkspaceResult<()>,
{
    let source_preflight =
        preflight_workspace_archive_source(working_root, manifest, current_app_version, limits)?;
    let archive_parent = archive_path
        .parent()
        .ok_or_else(|| WorkspaceError::Conflict("archive destination has no parent".into()))?;
    fs::create_dir_all(archive_parent)?;

    let staging = tempfile::Builder::new()
        .prefix(".c4os-snapshot-")
        .tempdir_in(archive_parent)?;
    stage_working_copy(
        staging.path(),
        source_preflight,
        limits,
        backup_workspace_database,
    )?;
    let snapshot_manifest = manifest_for_snapshot(staging.path(), manifest.clone())?;
    validate_manifest(&snapshot_manifest, current_app_version)?;
    validate_manifest_limits(&snapshot_manifest, limits)?;
    persist_working_manifest(staging.path(), &snapshot_manifest)?;

    let temporary_archive = archive_parent.join(format!(
        ".{}.{}.tmp",
        archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace"),
        Uuid::new_v4()
    ));
    let result = (|| {
        write_snapshot_archive(staging.path(), &temporary_archive, &snapshot_manifest)?;
        validate_archive(&temporary_archive, current_app_version, limits)?;
        // Canonical working-copy metadata is a pre-commit operation. A
        // failure here leaves the prior portable archive untouched.
        persist_working_manifest(working_root, &snapshot_manifest)?;
        sync_directory(archive_parent)?;
        let rollback = retain_prior_archive(archive_path)?;
        if matches!(rollback, ArchiveSaveRollback::RestorePrevious(_)) {
            commit_sync_directory(archive_parent)?;
        }
        if let Err(error) = commit_rename(&temporary_archive, archive_path) {
            if let ArchiveSaveRollback::RestorePrevious(previous) = &rollback {
                let _ = fs::remove_file(previous);
                let _ = sync_directory(archive_parent);
            }
            return Err(error.into());
        }
        let mut pending = PendingWorkspaceSave {
            saved: Some(SavedWorkspace {
                manifest: snapshot_manifest,
                archive_path: archive_path.to_path_buf(),
            }),
            rollback: Some(rollback),
            archive_parent: archive_parent.to_path_buf(),
        };
        if let Err(error) = commit_sync_directory(archive_parent) {
            return match pending.rollback_inner() {
                Ok(()) => Err(error.into()),
                Err(rollback_error) => Err(rollback_error),
            };
        }
        Ok(pending)
    })();
    if temporary_archive.exists() {
        let _ = fs::remove_file(&temporary_archive);
    }
    result
}

pub fn open_workspace_archive<F>(
    home: &C4osHomeLayout,
    archive_path: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
    lock_owner: WorkspaceLockOwner,
    validate_extracted_workspace: F,
) -> WorkspaceResult<OpenWorkspaceOutcome>
where
    F: Fn(
        &Path,
        &WorkspaceManifest,
        WorkspaceSemanticValidationTarget,
    ) -> WorkspaceResult<WorkspaceManifest>,
{
    match prepare_open_workspace_archive(
        home,
        archive_path,
        current_app_version,
        limits,
        lock_owner,
        validate_extracted_workspace,
    )? {
        PreparedOpenWorkspaceOutcome::Writable(pending) => {
            Ok(OpenWorkspaceOutcome::Writable(pending.commit()))
        }
        PreparedOpenWorkspaceOutcome::ReadOnly(workspace) => {
            Ok(OpenWorkspaceOutcome::ReadOnly(workspace))
        }
    }
}

/// Validates and promotes an archive while retaining the prior active working
/// copy until the caller finishes database activation and app-owned recent
/// persistence. Dropping or aborting a writable result restores the prior
/// active state.
pub fn prepare_open_workspace_archive<F>(
    home: &C4osHomeLayout,
    archive_path: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
    lock_owner: WorkspaceLockOwner,
    validate_extracted_workspace: F,
) -> WorkspaceResult<PreparedOpenWorkspaceOutcome>
where
    F: Fn(
        &Path,
        &WorkspaceManifest,
        WorkspaceSemanticValidationTarget,
    ) -> WorkspaceResult<WorkspaceManifest>,
{
    home.ensure_roots()?;
    let validated = validate_archive(archive_path, current_app_version, limits)?;
    let access = acquire_workspace_writer_lock(&home.workspace_lock(), lock_owner)?;
    let writer_lock = match access {
        WriterAccess::ReadOnly { owner } => {
            let active_root = home.active_workspace();
            let workspace_parent = active_root
                .parent()
                .ok_or_else(|| WorkspaceError::Conflict("active Workspace has no parent".into()))?;
            let extraction = tempfile::Builder::new()
                .prefix(".c4os-open-read-only-")
                .tempdir_in(workspace_parent)?;
            let extracted = inspect_archive(
                archive_path,
                current_app_version,
                limits,
                Some(extraction.path()),
            )?;
            // Lock contention removes promotion authority, not the requirement
            // to prove that the extracted database and configuration are a
            // semantically valid Workspace. The temporary tree is discarded
            // on every return path and active state is never consulted or
            // mutated by this read-only branch.
            let candidate_canonical = validate_extracted_workspace(
                extraction.path(),
                &extracted.manifest,
                WorkspaceSemanticValidationTarget::ArchiveCandidate,
            )?;
            validate_manifest(&candidate_canonical, current_app_version)?;
            if !portable_manifests_equal(&candidate_canonical, &extracted.manifest)? {
                return Err(WorkspaceError::Conflict(
                    "archive semantic state does not match its portable manifest".into(),
                ));
            }
            return Ok(PreparedOpenWorkspaceOutcome::ReadOnly(ReadOnlyWorkspace {
                manifest: extracted.manifest,
                archive_path: archive_path.to_path_buf(),
                owner,
            }));
        }
        WriterAccess::Writable(lock) => lock,
    };

    let active_root = home.active_workspace();
    let mut previous_manifest = None;
    if active_root.exists() {
        match load_working_manifest(&active_root) {
            Ok(active_manifest)
                if validate_manifest(&active_manifest, current_app_version).is_ok() =>
            {
                previous_manifest = Some(active_manifest.clone());
                if active_manifest.workspace_id == validated.manifest.workspace_id {
                    let canonical = validate_extracted_workspace(
                        &active_root,
                        &active_manifest,
                        WorkspaceSemanticValidationTarget::ActiveRecovery,
                    )
                    .and_then(|canonical| {
                        validate_manifest(&canonical, current_app_version)?;
                        if canonical.workspace_id != active_manifest.workspace_id {
                            return Err(WorkspaceError::Conflict(
                                "active semantic Workspace identity mismatch".into(),
                            ));
                        }
                        Ok(canonical)
                    });
                    match canonical {
                        Ok(canonical) if canonical.generation > validated.manifest.generation => {
                            match persist_canonical_working_manifest(
                                &writer_lock,
                                &active_root,
                                &canonical,
                                current_app_version,
                                limits,
                            ) {
                                Ok(canonical) => {
                                    return Ok(PreparedOpenWorkspaceOutcome::Writable(
                                        PendingWorkspaceOpen {
                                            workspace: Some(WritableWorkspace {
                                                recovery_notice: Some(RecoveryNotice {
                                                    workspace_id: canonical.workspace_id,
                                                    working_generation: canonical.generation,
                                                    archive_generation: validated
                                                        .manifest
                                                        .generation,
                                                    must_notify_before_next_save: true,
                                                }),
                                                manifest: canonical,
                                                working_root: active_root,
                                                archive_path: archive_path.to_path_buf(),
                                                writer_lock,
                                            }),
                                            rollback: None,
                                        },
                                    ));
                                }
                                Err(_) => previous_manifest = None,
                            }
                        }
                        Ok(canonical) => previous_manifest = Some(canonical),
                        Err(_) => previous_manifest = None,
                    }
                }
            }
            _ => {
                // A malformed or semantically unusable active copy is retained
                // until the fully validated archive candidate is ready.
                previous_manifest = None;
            }
        }
    }

    let workspace_parent = active_root
        .parent()
        .ok_or_else(|| WorkspaceError::Conflict("active Workspace has no parent".into()))?
        .to_path_buf();
    let extraction = tempfile::Builder::new()
        .prefix(".c4os-open-")
        .tempdir_in(&workspace_parent)?;
    let extracted = inspect_archive(
        archive_path,
        current_app_version,
        limits,
        Some(extraction.path()),
    )?;
    // Structural validation is necessary but cannot establish database,
    // migration, or scoped-configuration semantics. The caller-owned service
    // must validate the fully extracted tree before any existing active copy
    // is moved or the candidate is promoted.
    let candidate_canonical = validate_extracted_workspace(
        extraction.path(),
        &extracted.manifest,
        WorkspaceSemanticValidationTarget::ArchiveCandidate,
    )?;
    validate_manifest(&candidate_canonical, current_app_version)?;
    if !portable_manifests_equal(&candidate_canonical, &extracted.manifest)? {
        return Err(WorkspaceError::Conflict(
            "archive semantic state does not match its portable manifest".into(),
        ));
    }
    sync_tree(extraction.path())?;

    let previous = if active_root.exists() {
        let recovery_name = previous_manifest.map_or_else(
            || format!("corrupt-active-{}", Uuid::new_v4()),
            |previous| {
                format!(
                    "{}-g{}-{}",
                    previous.workspace_id,
                    previous.generation,
                    Uuid::new_v4()
                )
            },
        );
        let recovery = home.workspace_recovery_root().join(recovery_name);
        commit_rename(&active_root, &recovery)?;
        Some(recovery)
    } else {
        None
    };

    if previous.is_some()
        && let Err(error) = commit_sync_directory(&workspace_parent)
    {
        if let Some(previous) = &previous {
            if let Err(rollback_error) = commit_rename(previous, &active_root) {
                return Err(commit_recovery_error("open", previous, rollback_error));
            }
            let _ = sync_directory(&workspace_parent);
        }
        return Err(error.into());
    }

    let promoted = extraction.keep();
    if let Err(error) = commit_rename(&promoted, &active_root) {
        if let Some(previous) = &previous {
            if let Err(rollback_error) = commit_rename(previous, &active_root) {
                return Err(commit_recovery_error("open", previous, rollback_error));
            }
            if let Err(rollback_error) = commit_sync_directory(&workspace_parent) {
                return Err(commit_recovery_error("open", &active_root, rollback_error));
            }
        }
        let _ = fs::remove_dir_all(&promoted);
        return Err(error.into());
    }
    let mut pending = PendingWorkspaceOpen {
        workspace: Some(WritableWorkspace {
            manifest: extracted.manifest,
            working_root: active_root.clone(),
            archive_path: archive_path.to_path_buf(),
            recovery_notice: None,
            writer_lock,
        }),
        rollback: Some(OpenWorkspaceRollback::Promoted {
            active_root,
            previous_active: previous,
            recovery_root: home.workspace_recovery_root(),
            workspace_parent: workspace_parent.clone(),
        }),
    };
    if let Err(error) = commit_sync_directory(&workspace_parent) {
        return match pending.rollback_inner() {
            Ok(()) => Err(error.into()),
            Err(rollback_error) => Err(rollback_error),
        };
    }

    Ok(PreparedOpenWorkspaceOutcome::Writable(pending))
}

fn inspect_archive(
    archive_path: &Path,
    current_app_version: &str,
    limits: ArchiveLimits,
    extraction_root: Option<&Path>,
) -> WorkspaceResult<ValidatedArchive> {
    let mut file = File::open(archive_path)?;
    preflight_central_directory(&mut file, limits)?;
    file.seek(SeekFrom::Start(0))?;
    let mut archive = ZipArchive::new(file).map_err(map_archive_open_error)?;
    if archive.len() > limits.max_entries {
        return archive_error(ArchiveViolation::EntryCountExceeded);
    }

    let mut names = BTreeSet::new();
    let mut folded_names = BTreeSet::new();
    let mut metadata = BTreeMap::new();
    let mut expanded_bytes = 0_u64;
    for index in 0..archive.len() {
        // Raw access exposes encryption and type metadata without requiring a
        // password or beginning decompression of hostile content.
        let entry = archive.by_index_raw(index)?;
        let name = validate_entry_metadata(&entry, limits)?;
        if !names.insert(name.clone()) || !folded_names.insert(name.to_ascii_lowercase()) {
            return archive_error(ArchiveViolation::DuplicateEntry(name));
        }
        expanded_bytes =
            expanded_bytes
                .checked_add(entry.size())
                .ok_or(WorkspaceError::Archive(
                    ArchiveViolation::ExpandedSizeExceeded,
                ))?;
        if expanded_bytes > limits.max_expanded_bytes {
            return archive_error(ArchiveViolation::ExpandedSizeExceeded);
        }
        metadata.insert(name, (entry.size(), entry.compressed_size()));
    }
    let manifest_size = metadata
        .get(MANIFEST_FILE)
        .map(|(size, _)| *size)
        .ok_or(WorkspaceError::Archive(ArchiveViolation::MissingManifest))?;
    if manifest_size > limits.max_manifest_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(MANIFEST_FILE.into()));
    }

    let manifest_bytes =
        read_archive_entry(&mut archive, MANIFEST_FILE, limits.max_manifest_bytes)?;
    let manifest_text = std::str::from_utf8(&manifest_bytes).map_err(|_| {
        WorkspaceError::Archive(ArchiveViolation::InvalidManifest(
            "manifest is not UTF-8".into(),
        ))
    })?;
    let manifest: WorkspaceManifest = toml::from_str(manifest_text).map_err(|error| {
        WorkspaceError::Archive(ArchiveViolation::InvalidManifest(error.to_string()))
    })?;
    validate_manifest(&manifest, current_app_version)?;

    let declared: BTreeMap<&str, &ManifestFile> = manifest
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    for name in names.iter().filter(|name| name.as_str() != MANIFEST_FILE) {
        if !declared.contains_key(name.as_str()) {
            return archive_error(ArchiveViolation::UndeclaredEntry(name.clone()));
        }
    }
    for declaration in &manifest.files {
        let Some((actual_size, _)) = metadata.get(&declaration.path) else {
            return archive_error(ArchiveViolation::MissingDeclaredEntry(
                declaration.path.clone(),
            ));
        };
        if *actual_size != declaration.expanded_size {
            return archive_error(ArchiveViolation::DeclaredSizeMismatch(
                declaration.path.clone(),
            ));
        }
    }

    if let Some(root) = extraction_root {
        fs::create_dir_all(root)?;
        write_extracted_file(root, MANIFEST_FILE, &manifest_bytes)?;
    }
    for declaration in &manifest.files {
        let bytes = read_archive_entry(&mut archive, &declaration.path, declaration.expanded_size)?;
        let digest = sha256_bytes(&bytes);
        if digest != declaration.sha256 {
            return archive_error(ArchiveViolation::DigestMismatch(declaration.path.clone()));
        }
        if let Some(root) = extraction_root {
            write_extracted_file(root, &declaration.path, &bytes)?;
        }
    }

    Ok(ValidatedArchive {
        manifest,
        expanded_bytes,
        entry_count: archive.len(),
    })
}

fn validate_entry_metadata<R: Read>(
    entry: &zip::read::ZipFile<'_, R>,
    limits: ArchiveLimits,
) -> WorkspaceResult<String> {
    let name = entry.name().to_string();
    validate_archive_path(&name)?;
    if entry.enclosed_name().is_none() {
        return archive_error(ArchiveViolation::InvalidEntryName(name));
    }
    if entry.encrypted() {
        return archive_error(ArchiveViolation::EncryptedEntry(name));
    }
    let mode_type = entry.unix_mode().unwrap_or(0) & 0o170000;
    if !entry.is_file() || !matches!(mode_type, 0 | 0o100000) {
        return archive_error(ArchiveViolation::UnsupportedEntryType(name));
    }
    if entry.compression() != CompressionMethod::Deflated {
        return archive_error(ArchiveViolation::UnsupportedCompression(name));
    }
    if entry.size() > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(name));
    }
    if entry.size() > 0
        && (entry.compressed_size() == 0
            || u128::from(entry.size())
                > u128::from(entry.compressed_size()) * u128::from(limits.max_compression_ratio))
    {
        return archive_error(ArchiveViolation::CompressionRatioExceeded(name));
    }
    Ok(name)
}

fn validate_manifest(
    manifest: &WorkspaceManifest,
    current_app_version: &str,
) -> WorkspaceResult<()> {
    if manifest.schema_version != WORKSPACE_SCHEMA_VERSION {
        return archive_error(ArchiveViolation::UnsupportedSchema(manifest.schema_version));
    }
    if manifest.workspace_id.is_nil() {
        return archive_error(ArchiveViolation::InvalidManifest(
            "Workspace UUID cannot be nil".into(),
        ));
    }
    if parse_version(&manifest.minimum_app_version)? > parse_version(current_app_version)? {
        return archive_error(ArchiveViolation::MinimumVersionNotMet(
            manifest.minimum_app_version.clone(),
        ));
    }
    let mut project_ids = BTreeSet::new();
    for project in &manifest.projects {
        project.validate().map_err(|_| {
            WorkspaceError::Archive(ArchiveViolation::InvalidManifest(
                "invalid Project reference".into(),
            ))
        })?;
        if !project_ids.insert(project.project_id) {
            return archive_error(ArchiveViolation::InvalidManifest(
                "duplicate Project reference".into(),
            ));
        }
    }
    let mut file_paths = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for file in &manifest.files {
        validate_archive_path(&file.path)?;
        if file.path == MANIFEST_FILE {
            return archive_error(ArchiveViolation::InvalidManifest(
                "manifest cannot declare itself".into(),
            ));
        }
        if !file_paths.insert(file.path.as_str()) {
            return archive_error(ArchiveViolation::InvalidManifest(
                "duplicate file declaration".into(),
            ));
        }
        if previous.is_some_and(|previous| previous >= file.path.as_str()) {
            return archive_error(ArchiveViolation::InvalidManifest(
                "file declarations must be in canonical path order".into(),
            ));
        }
        previous = Some(&file.path);
        if !is_sha256_hex(&file.sha256) {
            return archive_error(ArchiveViolation::InvalidManifest(
                "invalid SHA-256 declaration".into(),
            ));
        }
        if let Some(path_digest) = blob_digest_from_path(&file.path)
            && path_digest != file.sha256
        {
            return archive_error(ArchiveViolation::BlobDigestMismatch(file.path.clone()));
        }
    }
    Ok(())
}

fn validate_manifest_limits(
    manifest: &WorkspaceManifest,
    limits: ArchiveLimits,
) -> WorkspaceResult<()> {
    let manifest_size = toml::to_string(manifest)?.len() as u64;
    if manifest_size > limits.max_manifest_bytes || manifest_size > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(MANIFEST_FILE.into()));
    }
    let entry_count = manifest
        .files
        .len()
        .checked_add(1)
        .ok_or(WorkspaceError::Archive(
            ArchiveViolation::EntryCountExceeded,
        ))?;
    if entry_count > limits.max_entries {
        return archive_error(ArchiveViolation::EntryCountExceeded);
    }
    let mut expanded = manifest_size;
    for file in &manifest.files {
        if file.expanded_size > limits.max_entry_bytes {
            return archive_error(ArchiveViolation::EntrySizeExceeded(file.path.clone()));
        }
        expanded = expanded
            .checked_add(file.expanded_size)
            .ok_or(WorkspaceError::Archive(
                ArchiveViolation::ExpandedSizeExceeded,
            ))?;
        if expanded > limits.max_expanded_bytes {
            return archive_error(ArchiveViolation::ExpandedSizeExceeded);
        }
    }
    Ok(())
}

fn portable_manifests_equal(
    left: &WorkspaceManifest,
    right: &WorkspaceManifest,
) -> WorkspaceResult<bool> {
    // TOML serialization deliberately omits runtime-only trusted-root grants.
    Ok(toml::to_string(left)? == toml::to_string(right)?)
}

fn validate_archive_path(path: &str) -> WorkspaceResult<()> {
    if path.is_empty()
        || path.len() > MAX_ENTRY_NAME_BYTES
        || path.contains('\0')
        || path.contains('\\')
        || path.starts_with('/')
        || path.ends_with('/')
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || segment.chars().any(char::is_control)
        })
    {
        return archive_error(ArchiveViolation::InvalidEntryName(path.into()));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return archive_error(ArchiveViolation::InvalidEntryName(path.into()));
    }
    if !archive_path_allowed(path) {
        return archive_error(ArchiveViolation::DisallowedRoot(path.into()));
    }
    Ok(())
}

fn archive_path_allowed(path: &str) -> bool {
    if matches!(
        path,
        MANIFEST_FILE | WORKSPACE_DATABASE_FILE | WORKSPACE_CONFIGURATION_FILE
    ) {
        return true;
    }
    let parts: Vec<_> = path.split('/').collect();
    match parts.as_slice() {
        ["projects", id, "config.toml"] => valid_uuid_segment(id),
        ["chats", id, "config.toml"] => valid_uuid_segment(id),
        ["chats", id, area, tail @ ..]
            if valid_uuid_segment(id)
                && matches!(*area, "cache" | "archive")
                && !tail.is_empty() =>
        {
            tail.iter().all(|segment| portable_segment(segment))
        }
        ["skills", skill_directory, tail @ ..] if !tail.is_empty() => {
            portable_segment(skill_directory)
                && tail.iter().all(|segment| portable_segment(segment))
        }
        ["blobs", "sha256", digest] => is_sha256_hex(digest),
        _ => false,
    }
}

fn valid_uuid_segment(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|parsed| parsed.to_string() == value)
}

fn portable_segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == SHA256_HEX_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn blob_digest_from_path(path: &str) -> Option<&str> {
    path.strip_prefix("blobs/sha256/")
        .filter(|digest| !digest.contains('/'))
}

fn external_project_path(path: &Path) -> WorkspaceResult<String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(WorkspaceError::InvalidProject(
            "Project path must be an absolute normalized path".into(),
        ));
    }
    path.to_str()
        .filter(|value| !value.is_empty() && value.len() <= 4_096)
        .map(str::to_string)
        .ok_or_else(|| WorkspaceError::InvalidProject("Project path must be UTF-8".into()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SemanticVersion {
    major: String,
    minor: String,
    patch: String,
    prerelease: Option<Vec<PrereleaseIdentifier>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PrereleaseIdentifier {
    Numeric(String),
    Alphanumeric(String),
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        compare_numeric_identifier(&self.major, &other.major)
            .then_with(|| compare_numeric_identifier(&self.minor, &other.minor))
            .then_with(|| compare_numeric_identifier(&self.patch, &other.patch))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(left), Some(right)) => compare_prerelease(left, right),
            })
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_numeric_identifier(left: &str, right: &str) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn compare_prerelease(
    left: &[PrereleaseIdentifier],
    right: &[PrereleaseIdentifier],
) -> std::cmp::Ordering {
    for (left, right) in left.iter().zip(right) {
        let ordering = match (left, right) {
            (PrereleaseIdentifier::Numeric(left), PrereleaseIdentifier::Numeric(right)) => {
                compare_numeric_identifier(left, right)
            }
            (PrereleaseIdentifier::Numeric(_), PrereleaseIdentifier::Alphanumeric(_)) => {
                std::cmp::Ordering::Less
            }
            (PrereleaseIdentifier::Alphanumeric(_), PrereleaseIdentifier::Numeric(_)) => {
                std::cmp::Ordering::Greater
            }
            (
                PrereleaseIdentifier::Alphanumeric(left),
                PrereleaseIdentifier::Alphanumeric(right),
            ) => left.cmp(right),
        };
        if ordering != std::cmp::Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

fn parse_version(value: &str) -> WorkspaceResult<SemanticVersion> {
    if value.is_empty() || !value.is_ascii() {
        return invalid_semver();
    }
    let (without_build, build) = match value.split_once('+') {
        Some((version, build)) if !build.contains('+') => (version, Some(build)),
        Some(_) => return invalid_semver(),
        None => (value, None),
    };
    if let Some(build) = build {
        validate_semver_identifiers(build, true)?;
    }
    let (core, prerelease) = match without_build.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (without_build, None),
    };
    let mut core_parts = core.split('.');
    let major = parse_core_identifier(core_parts.next())?;
    let minor = parse_core_identifier(core_parts.next())?;
    let patch = parse_core_identifier(core_parts.next())?;
    if core_parts.next().is_some() {
        return invalid_semver();
    }
    let prerelease = prerelease
        .map(|value| {
            validate_semver_identifiers(value, false)?;
            Ok::<Vec<PrereleaseIdentifier>, WorkspaceError>(
                value
                    .split('.')
                    .map(|identifier| {
                        if identifier.bytes().all(|byte| byte.is_ascii_digit()) {
                            PrereleaseIdentifier::Numeric(identifier.to_string())
                        } else {
                            PrereleaseIdentifier::Alphanumeric(identifier.to_string())
                        }
                    })
                    .collect(),
            )
        })
        .transpose()?;
    Ok(SemanticVersion {
        major,
        minor,
        patch,
        prerelease,
    })
}

fn parse_core_identifier(value: Option<&str>) -> WorkspaceResult<String> {
    let Some(value) = value else {
        return invalid_semver();
    };
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return invalid_semver();
    }
    Ok(value.to_string())
}

fn validate_semver_identifiers(value: &str, build: bool) -> WorkspaceResult<()> {
    for identifier in value.split('.') {
        if identifier.is_empty()
            || !identifier
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || (!build
                && identifier.len() > 1
                && identifier.starts_with('0')
                && identifier.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return invalid_semver();
        }
    }
    Ok(())
}

fn invalid_semver<T>() -> WorkspaceResult<T> {
    archive_error(ArchiveViolation::InvalidManifest(
        "application version must be valid SemVer".into(),
    ))
}

fn load_working_manifest(root: &Path) -> WorkspaceResult<WorkspaceManifest> {
    let bytes = fs::read(root.join(MANIFEST_FILE))?;
    if bytes.len() as u64 > ArchiveLimits::default().max_manifest_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(MANIFEST_FILE.into()));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        WorkspaceError::Archive(ArchiveViolation::InvalidManifest(
            "manifest is not UTF-8".into(),
        ))
    })?;
    toml::from_str(text).map_err(|error| {
        WorkspaceError::Archive(ArchiveViolation::InvalidManifest(error.to_string()))
    })
}

fn persist_working_manifest(root: &Path, manifest: &WorkspaceManifest) -> WorkspaceResult<()> {
    fs::create_dir_all(root)?;
    let bytes = toml::to_string(manifest)?.into_bytes();
    let destination = root.join(MANIFEST_FILE);
    let temporary = root.join(format!(".manifest.{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, &destination)?;
        sync_directory(root)?;
        Ok(())
    })();
    if temporary.exists() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn manifest_for_snapshot(
    root: &Path,
    mut manifest: WorkspaceManifest,
) -> WorkspaceResult<WorkspaceManifest> {
    let mut files = Vec::new();
    collect_snapshot_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    manifest.files = files;
    Ok(manifest)
}

fn collect_snapshot_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<ManifestFile>,
) -> WorkspaceResult<()> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            let relative = relative_archive_name(root, &path)?;
            return archive_error(ArchiveViolation::UnsupportedEntryType(relative));
        }
        if metadata.is_dir() {
            collect_snapshot_files(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = relative_archive_name(root, &path)?;
            if relative == MANIFEST_FILE
                || relative.starts_with(".manifest.")
                || is_transient_workspace_database_sidecar(&relative)
            {
                continue;
            }
            validate_archive_path(&relative)?;
            let (sha256, expanded_size) = sha256_file(&path)?;
            files.push(ManifestFile {
                path: relative,
                sha256,
                expanded_size,
            });
        } else {
            let relative = relative_archive_name(root, &path)?;
            return archive_error(ArchiveViolation::UnsupportedEntryType(relative));
        }
    }
    Ok(())
}

fn stage_working_copy<F>(
    destination_root: &Path,
    source_preflight: WorkspaceArchiveSourcePreflight,
    limits: ArchiveLimits,
    backup_workspace_database: F,
) -> WorkspaceResult<()>
where
    F: FnOnce(&Path, &Path) -> WorkspaceResult<()>,
{
    let mut database_callback = Some(backup_workspace_database);
    let mut staged_expanded_bytes = source_preflight.manifest_bytes;
    for file in source_preflight.files {
        let destination = destination_root.join(&file.path);
        if file.path == WORKSPACE_DATABASE_FILE {
            let callback = database_callback.take().ok_or_else(|| {
                WorkspaceError::Conflict("database snapshot callback reused".into())
            })?;
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            callback(&file.source, &destination)?;
            ensure_regular_snapshot_file(&destination, &file.path)?;
            let expanded_size = fs::metadata(&destination)?.len();
            validate_staged_file_size(
                &file.path,
                expanded_size,
                &mut staged_expanded_bytes,
                limits,
            )?;
            validate_file_compression_ratio(&destination, &file.path, limits)?;
        } else {
            let remaining_expanded = limits
                .max_expanded_bytes
                .saturating_sub(staged_expanded_bytes);
            let copy_limit = limits.max_entry_bytes.min(remaining_expanded);
            let expanded_size =
                copy_regular_file_bounded(&file.source, &destination, &file.path, copy_limit)?;
            validate_staged_file_size(
                &file.path,
                expanded_size,
                &mut staged_expanded_bytes,
                limits,
            )?;
        }
    }
    sync_tree(destination_root)?;
    Ok(())
}

fn collect_source_paths(
    root: &Path,
    limits: ArchiveLimits,
    entry_count: &mut usize,
    expanded_bytes: &mut u64,
    files: &mut Vec<WorkspaceArchiveSourceFile>,
) -> WorkspaceResult<()> {
    let scan_limit = limits.max_entries.saturating_mul(4).saturating_add(64);
    let mut scanned_entries = 0_usize;
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory)? {
            scanned_entries = scanned_entries
                .checked_add(1)
                .ok_or(WorkspaceError::Archive(
                    ArchiveViolation::EntryCountExceeded,
                ))?;
            if scanned_entries > scan_limit {
                return archive_error(ArchiveViolation::EntryCountExceeded);
            }

            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() {
                return archive_error(ArchiveViolation::UnsupportedEntryType(
                    relative_archive_name(root, &path)?,
                ));
            }
            if metadata.is_dir() {
                directories.push(path);
            } else if metadata.is_file() {
                let relative = relative_archive_name(root, &path)?;
                if relative == MANIFEST_FILE
                    || relative.starts_with(".manifest.")
                    || is_transient_workspace_database_sidecar(&relative)
                {
                    continue;
                }
                validate_archive_path(&relative)?;
                *entry_count = entry_count.checked_add(1).ok_or(WorkspaceError::Archive(
                    ArchiveViolation::EntryCountExceeded,
                ))?;
                if *entry_count > limits.max_entries {
                    return archive_error(ArchiveViolation::EntryCountExceeded);
                }
                let expanded_size = metadata.len();
                if expanded_size > limits.max_entry_bytes {
                    return archive_error(ArchiveViolation::EntrySizeExceeded(relative));
                }
                *expanded_bytes =
                    expanded_bytes
                        .checked_add(expanded_size)
                        .ok_or(WorkspaceError::Archive(
                            ArchiveViolation::ExpandedSizeExceeded,
                        ))?;
                if *expanded_bytes > limits.max_expanded_bytes {
                    return archive_error(ArchiveViolation::ExpandedSizeExceeded);
                }
                files.push(WorkspaceArchiveSourceFile {
                    path: relative,
                    source: path,
                    expanded_size,
                });
            } else {
                return archive_error(ArchiveViolation::UnsupportedEntryType(
                    relative_archive_name(root, &path)?,
                ));
            }
        }
    }
    Ok(())
}

fn copy_regular_file(source: &Path, destination: &Path) -> WorkspaceResult<()> {
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return archive_error(ArchiveViolation::UnsupportedEntryType(
            source.display().to_string(),
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    io::copy(&mut input, &mut output)?;
    output.sync_all()?;
    Ok(())
}

fn copy_regular_file_bounded(
    source: &Path,
    destination: &Path,
    relative: &str,
    max_bytes: u64,
) -> WorkspaceResult<u64> {
    let metadata = fs::symlink_metadata(source)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return archive_error(ArchiveViolation::UnsupportedEntryType(relative.into()));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let input = File::open(source)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let copied = io::copy(&mut input.take(max_bytes.saturating_add(1)), &mut output)?;
    if copied > max_bytes {
        drop(output);
        let _ = fs::remove_file(destination);
        return archive_error(ArchiveViolation::EntrySizeExceeded(relative.into()));
    }
    output.sync_all()?;
    Ok(copied)
}

fn validate_staged_file_size(
    relative: &str,
    expanded_size: u64,
    staged_expanded_bytes: &mut u64,
    limits: ArchiveLimits,
) -> WorkspaceResult<()> {
    if expanded_size > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(relative.into()));
    }
    *staged_expanded_bytes =
        staged_expanded_bytes
            .checked_add(expanded_size)
            .ok_or(WorkspaceError::Archive(
                ArchiveViolation::ExpandedSizeExceeded,
            ))?;
    if *staged_expanded_bytes > limits.max_expanded_bytes {
        return archive_error(ArchiveViolation::ExpandedSizeExceeded);
    }
    Ok(())
}

#[derive(Debug, Default)]
struct CountingZipSink {
    written: u64,
    tail: Vec<u8>,
}

impl CountingZipSink {
    fn compressed_size(&self) -> WorkspaceResult<u64> {
        let central_offset = self
            .tail
            .windows(4)
            .rposition(|window| window == b"PK\x01\x02")
            .ok_or_else(|| invalid_zip("compression probe central directory is missing"))?;
        Ok(u64::from(little_u32(&self.tail, central_offset + 20)?))
    }
}

impl Write for CountingZipSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.written = self
            .written
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| io::Error::other("compression probe length overflow"))?;
        if bytes.len() >= ZIP_PROBE_TAIL_BYTES {
            self.tail.clear();
            self.tail
                .extend_from_slice(&bytes[bytes.len() - ZIP_PROBE_TAIL_BYTES..]);
        } else {
            let overflow = self
                .tail
                .len()
                .saturating_add(bytes.len())
                .saturating_sub(ZIP_PROBE_TAIL_BYTES);
            if overflow > 0 {
                self.tail.drain(..overflow);
            }
            self.tail.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn validate_file_compression_ratio(
    source: &Path,
    relative: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<()> {
    let mut writer = ZipWriter::new_stream(CountingZipSink::default());
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    writer.start_file("probe", options)?;
    let input = File::open(source)?;
    let copied = io::copy(
        &mut input.take(limits.max_entry_bytes.saturating_add(1)),
        &mut writer,
    )?;
    if copied > limits.max_entry_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(relative.into()));
    }
    let probe = writer.finish()?.into_inner();
    let compressed_size = probe.compressed_size()?;
    if copied > 0
        && (compressed_size == 0
            || u128::from(copied)
                > u128::from(compressed_size) * u128::from(limits.max_compression_ratio))
    {
        return archive_error(ArchiveViolation::CompressionRatioExceeded(relative.into()));
    }
    Ok(())
}

fn ensure_regular_snapshot_file(path: &Path, relative: &str) -> WorkspaceResult<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return archive_error(ArchiveViolation::UnsupportedEntryType(relative.into()));
    }
    File::open(path)?.sync_all()?;
    Ok(())
}

fn write_snapshot_archive(
    snapshot_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
) -> WorkspaceResult<()> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive_path)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    writer.start_file(MANIFEST_FILE, options)?;
    writer.write_all(toml::to_string(manifest)?.as_bytes())?;
    for declaration in &manifest.files {
        writer.start_file(&declaration.path, options)?;
        let mut input = File::open(snapshot_root.join(&declaration.path))?;
        io::copy(&mut input, &mut writer)?;
    }
    let output = writer.finish()?;
    output.sync_all()?;
    Ok(())
}

fn read_archive_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
    max_bytes: u64,
) -> WorkspaceResult<Vec<u8>> {
    let mut entry = archive.by_name(name)?;
    let capacity = usize::try_from(entry.size().min(max_bytes)).unwrap_or(0);
    let mut bytes = Vec::with_capacity(capacity);
    let copied = (&mut entry)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if copied as u64 > max_bytes {
        return archive_error(ArchiveViolation::EntrySizeExceeded(name.into()));
    }
    Ok(bytes)
}

fn write_extracted_file(root: &Path, name: &str, bytes: &[u8]) -> WorkspaceResult<()> {
    validate_archive_path(name)?;
    let destination = root.join(name);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn relative_archive_name(root: &Path, path: &Path) -> WorkspaceResult<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| WorkspaceError::Conflict("path escaped the working copy".into()))?;
    let mut segments = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(segment) => {
                segments.push(segment.to_str().ok_or_else(|| {
                    WorkspaceError::Conflict("Workspace path is not UTF-8".into())
                })?)
            }
            _ => {
                return archive_error(ArchiveViolation::InvalidEntryName(
                    path.display().to_string(),
                ));
            }
        }
    }
    Ok(segments.join("/"))
}

fn is_transient_workspace_database_sidecar(path: &str) -> bool {
    matches!(
        path,
        "state/workspace.sqlite3-wal"
            | "state/workspace.sqlite3-shm"
            | "state/workspace.sqlite3-journal"
    )
}

fn sha256_file(path: &Path) -> WorkspaceResult<(String, u64)> {
    let mut input = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size = size
            .checked_add(read as u64)
            .ok_or(WorkspaceError::Archive(
                ArchiveViolation::ExpandedSizeExceeded,
            ))?;
    }
    Ok((hex_digest(&hasher.finalize()), size))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn retain_prior_archive(archive_path: &Path) -> WorkspaceResult<ArchiveSaveRollback> {
    match fs::symlink_metadata(archive_path) {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(WorkspaceError::Conflict(
                    "archive destination is not a regular file".into(),
                ));
            }
            let archive_parent = archive_path.parent().ok_or_else(|| {
                WorkspaceError::Conflict("archive destination has no parent".into())
            })?;
            let previous = archive_parent.join(format!(
                ".{}.{}.previous",
                archive_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("workspace"),
                Uuid::new_v4()
            ));
            fs::hard_link(archive_path, &previous)?;
            Ok(ArchiveSaveRollback::RestorePrevious(previous))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(ArchiveSaveRollback::RemoveNew),
        Err(error) => Err(error.into()),
    }
}

fn commit_recovery_error(
    operation: &'static str,
    recovery_path: &Path,
    error: impl std::fmt::Display,
) -> WorkspaceError {
    WorkspaceError::CommitRecovery {
        operation,
        recovery_path: recovery_path.to_path_buf(),
        message: error.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitOperation {
    Rename,
    SyncDirectory,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct InjectedCommitFailure {
    operation: CommitOperation,
    matching_operations_to_skip: usize,
}

#[cfg(test)]
thread_local! {
    static INJECTED_COMMIT_FAILURE: std::cell::RefCell<Option<InjectedCommitFailure>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn inject_commit_failure(operation: CommitOperation, matching_operations_to_skip: usize) {
    INJECTED_COMMIT_FAILURE.with(|injection| {
        *injection.borrow_mut() = Some(InjectedCommitFailure {
            operation,
            matching_operations_to_skip,
        });
    });
}

#[cfg(test)]
fn fail_injected_commit_operation(operation: CommitOperation) -> io::Result<()> {
    INJECTED_COMMIT_FAILURE.with(|injection| {
        let mut injection = injection.borrow_mut();
        let Some(failure) = injection.as_mut() else {
            return Ok(());
        };
        if failure.operation != operation {
            return Ok(());
        }
        if failure.matching_operations_to_skip > 0 {
            failure.matching_operations_to_skip -= 1;
            return Ok(());
        }
        *injection = None;
        Err(io::Error::other(format!(
            "injected {operation:?} commit failure"
        )))
    })
}

#[cfg(not(test))]
fn fail_injected_commit_operation(_operation: CommitOperation) -> io::Result<()> {
    Ok(())
}

pub(super) fn commit_rename(source: &Path, destination: &Path) -> io::Result<()> {
    fail_injected_commit_operation(CommitOperation::Rename)?;
    fs::rename(source, destination)
}

pub(super) fn commit_sync_directory(path: &Path) -> io::Result<()> {
    fail_injected_commit_operation(CommitOperation::SyncDirectory)?;
    File::open(path)?.sync_all()
}

fn sync_tree(root: &Path) -> WorkspaceResult<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            sync_tree(&entry.path())?;
        } else if metadata.is_file() {
            File::open(entry.path())?.sync_all()?;
        }
    }
    sync_directory(root)
}

fn sync_directory(path: &Path) -> WorkspaceResult<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

/// `zip` intentionally indexes central-directory records by name. Inspect the
/// bounded raw central directory first so duplicate names cannot be collapsed
/// before C4OS sees them. The portable format does not need ZIP64 because its
/// expanded-size limit is below 4 GiB, so ZIP64 sentinels fail closed.
fn preflight_central_directory(file: &mut File, limits: ArchiveLimits) -> WorkspaceResult<()> {
    let file_len = file.metadata()?.len();
    if file_len < END_OF_CENTRAL_DIRECTORY_BYTES as u64 {
        return Err(invalid_zip("missing end-of-central-directory record"));
    }
    let tail_len = file_len.min((END_OF_CENTRAL_DIRECTORY_BYTES + MAX_ZIP_COMMENT_BYTES) as u64);
    file.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0_u8; tail_len as usize];
    file.read_exact(&mut tail)?;
    let eocd_offset = tail
        .windows(4)
        .rposition(|window| window == b"PK\x05\x06")
        .ok_or_else(|| invalid_zip("missing end-of-central-directory record"))?;
    if eocd_offset + END_OF_CENTRAL_DIRECTORY_BYTES > tail.len() {
        return Err(invalid_zip("truncated end-of-central-directory record"));
    }
    let eocd = &tail[eocd_offset..];
    let disk_number = little_u16(eocd, 4)?;
    let central_disk = little_u16(eocd, 6)?;
    let disk_entries = little_u16(eocd, 8)?;
    let total_entries = little_u16(eocd, 10)?;
    let central_size = little_u32(eocd, 12)? as u64;
    let central_offset = little_u32(eocd, 16)? as u64;
    let comment_len = little_u16(eocd, 20)? as usize;
    if disk_number != 0 || central_disk != 0 || disk_entries != total_entries {
        return Err(invalid_zip("multi-disk archives are unsupported"));
    }
    if total_entries == u16::MAX
        || central_size == u32::MAX as u64
        || central_offset == u32::MAX as u64
    {
        return Err(invalid_zip("ZIP64 archives are outside the bounded format"));
    }
    if total_entries as usize > limits.max_entries {
        return archive_error(ArchiveViolation::EntryCountExceeded);
    }
    if central_size > MAX_CENTRAL_DIRECTORY_BYTES {
        return Err(invalid_zip("central directory exceeds its byte limit"));
    }
    let absolute_eocd = file_len - tail_len + eocd_offset as u64;
    if eocd_offset + END_OF_CENTRAL_DIRECTORY_BYTES + comment_len != tail.len()
        || central_offset.checked_add(central_size) != Some(absolute_eocd)
    {
        return Err(invalid_zip("central-directory bounds are inconsistent"));
    }

    file.seek(SeekFrom::Start(central_offset))?;
    let mut names = BTreeSet::new();
    let mut folded_names = BTreeSet::new();
    let mut consumed = 0_u64;
    for _ in 0..total_entries {
        let mut fixed = [0_u8; 46];
        file.read_exact(&mut fixed)?;
        consumed += fixed.len() as u64;
        if &fixed[..4] != b"PK\x01\x02" {
            return Err(invalid_zip("invalid central-directory record"));
        }
        let name_len = little_u16(&fixed, 28)? as usize;
        let extra_len = little_u16(&fixed, 30)? as usize;
        let entry_comment_len = little_u16(&fixed, 32)? as usize;
        if name_len == 0 || name_len > MAX_ENTRY_NAME_BYTES {
            return archive_error(ArchiveViolation::InvalidEntryName(
                "central-directory-name".into(),
            ));
        }
        if extra_len > 16 * 1024 || entry_comment_len != 0 {
            return Err(invalid_zip(
                "entry extra data or comments exceed the supported format",
            ));
        }
        let variable_len = name_len
            .checked_add(extra_len)
            .and_then(|length| length.checked_add(entry_comment_len))
            .ok_or_else(|| invalid_zip("central-directory length overflow"))?;
        consumed = consumed
            .checked_add(variable_len as u64)
            .ok_or_else(|| invalid_zip("central-directory length overflow"))?;
        if consumed > central_size {
            return Err(invalid_zip("central-directory record exceeds its bounds"));
        }
        let mut variable = vec![0_u8; variable_len];
        file.read_exact(&mut variable)?;
        let name = String::from_utf8(variable[..name_len].to_vec()).map_err(|_| {
            WorkspaceError::Archive(ArchiveViolation::InvalidEntryName("non-UTF-8-entry".into()))
        })?;
        if !names.insert(name.clone()) || !folded_names.insert(name.to_ascii_lowercase()) {
            return archive_error(ArchiveViolation::DuplicateEntry(name));
        }
    }
    if consumed != central_size {
        return Err(invalid_zip("central-directory size is inconsistent"));
    }
    Ok(())
}

fn little_u16(bytes: &[u8], offset: usize) -> WorkspaceResult<u16> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| invalid_zip("truncated zip metadata"))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn little_u32(bytes: &[u8], offset: usize) -> WorkspaceResult<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| invalid_zip("truncated zip metadata"))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn invalid_zip(message: &'static str) -> WorkspaceError {
    WorkspaceError::Zip(zip::result::ZipError::InvalidArchive(message.into()))
}

fn archive_error<T>(violation: ArchiveViolation) -> WorkspaceResult<T> {
    Err(WorkspaceError::Archive(violation))
}

fn map_archive_open_error(error: zip::result::ZipError) -> WorkspaceError {
    if let zip::result::ZipError::InvalidArchive(message) = &error
        && let Some(name) = message.strip_prefix("Duplicate filename: ")
    {
        return WorkspaceError::Archive(ArchiveViolation::DuplicateEntry(name.to_string()));
    }
    WorkspaceError::Zip(error)
}

#[cfg(test)]
mod commit_failure_tests {
    use super::*;
    use tempfile::TempDir;

    const TEST_VERSION: &str = "0.1.0";

    fn owner(label: &str) -> WorkspaceLockOwner {
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: 1,
            label: label.into(),
        }
    }

    fn writer_lock(temp: &TempDir, label: &str) -> WorkspaceWriterLock {
        match acquire_workspace_writer_lock(
            &temp.path().join(format!("{label}.lock")),
            owner(label),
        )
        .expect("writer lock")
        {
            WriterAccess::Writable(lock) => lock,
            WriterAccess::ReadOnly { .. } => panic!("unique fixture lock is writable"),
        }
    }

    fn working_copy(
        temp: &TempDir,
        label: &str,
    ) -> (PathBuf, WorkspaceManifest, WorkspaceWriterLock) {
        let project = temp.path().join(format!("{label}-project"));
        fs::create_dir(&project).expect("project");
        let root = temp.path().join(format!("{label}-working"));
        let lock = writer_lock(temp, label);
        let manifest = create_untitled_working_copy(&lock, &root, &project, label, TEST_VERSION)
            .expect("working copy");
        let layout = WorkspaceLayout::new(&root);
        fs::write(layout.database(), format!("{label} database")).expect("database");
        fs::write(
            layout.workspace_configuration(),
            format!("schema_version = 1\nlabel = \"{label}\"\n"),
        )
        .expect("configuration");
        (root, manifest, lock)
    }

    #[test]
    fn archive_commit_faults_restore_the_prior_validated_destination() {
        let temp = TempDir::new().expect("tempdir");
        let (working, manifest, lock) = working_copy(&temp, "save-fault");
        let archive = temp.path().join("save-fault.zip");
        save_workspace_archive_by_copy(
            &lock,
            &working,
            &archive,
            &manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
        )
        .expect("baseline save");
        let prior = fs::read(&archive).expect("prior archive");
        fs::write(
            WorkspaceLayout::new(&working).workspace_configuration(),
            b"schema_version = 1\nrevision = 2\n",
        )
        .expect("updated configuration");

        inject_commit_failure(CommitOperation::Rename, 0);
        prepare_workspace_archive_save(
            &lock,
            &working,
            &archive,
            &manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
            |source, destination| {
                fs::copy(source, destination)?;
                Ok(())
            },
        )
        .expect_err("archive promotion rename failure");
        assert_eq!(fs::read(&archive).expect("rename rollback"), prior);

        inject_commit_failure(CommitOperation::SyncDirectory, 1);
        prepare_workspace_archive_save(
            &lock,
            &working,
            &archive,
            &manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
            |source, destination| {
                fs::copy(source, destination)?;
                Ok(())
            },
        )
        .expect_err("post-rename parent sync failure");
        assert_eq!(fs::read(&archive).expect("sync rollback"), prior);

        let pending = prepare_workspace_archive_save(
            &lock,
            &working,
            &archive,
            &manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
            |source, destination| {
                fs::copy(source, destination)?;
                Ok(())
            },
        )
        .expect("prepared save");
        inject_commit_failure(CommitOperation::Rename, 0);
        let rollback_error = pending.abort().expect_err("injected rollback failure");
        assert!(matches!(
            rollback_error,
            WorkspaceError::CommitRecovery {
                operation: "save",
                ..
            }
        ));
        assert_eq!(
            fs::read(&archive).expect("Drop retries failed rollback"),
            prior
        );
    }

    #[test]
    fn active_promotion_faults_restore_the_prior_working_copy() {
        let temp = TempDir::new().expect("tempdir");
        let (first_root, first_manifest, first_lock) = working_copy(&temp, "first");
        let (second_root, second_manifest, second_lock) = working_copy(&temp, "second");
        let first_archive = temp.path().join("first.zip");
        let second_archive = temp.path().join("second.zip");
        save_workspace_archive_by_copy(
            &first_lock,
            &first_root,
            &first_archive,
            &first_manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
        )
        .expect("first archive");
        save_workspace_archive_by_copy(
            &second_lock,
            &second_root,
            &second_archive,
            &second_manifest,
            TEST_VERSION,
            ArchiveLimits::default(),
        )
        .expect("second archive");
        let home = C4osHomeLayout::new(temp.path().join("home"));
        let opened = open_workspace_archive(
            &home,
            &first_archive,
            TEST_VERSION,
            ArchiveLimits::default(),
            owner("first-open"),
            |_root, candidate, _target| Ok(candidate.clone()),
        )
        .expect("first open");
        let OpenWorkspaceOutcome::Writable(opened) = opened else {
            panic!("first open is writable");
        };
        drop(opened);
        let prior_active =
            fs::read(home.active_workspace().join(MANIFEST_FILE)).expect("prior active manifest");

        inject_commit_failure(CommitOperation::Rename, 1);
        prepare_open_workspace_archive(
            &home,
            &second_archive,
            TEST_VERSION,
            ArchiveLimits::default(),
            owner("rename-fault"),
            |_root, candidate, _target| Ok(candidate.clone()),
        )
        .expect_err("candidate promotion rename failure");
        assert_eq!(
            fs::read(home.active_workspace().join(MANIFEST_FILE)).expect("rename restored active"),
            prior_active
        );

        inject_commit_failure(CommitOperation::SyncDirectory, 1);
        prepare_open_workspace_archive(
            &home,
            &second_archive,
            TEST_VERSION,
            ArchiveLimits::default(),
            owner("sync-fault"),
            |_root, candidate, _target| Ok(candidate.clone()),
        )
        .expect_err("post-promotion parent sync failure");
        assert_eq!(
            fs::read(home.active_workspace().join(MANIFEST_FILE)).expect("sync restored active"),
            prior_active
        );

        let prepared = prepare_open_workspace_archive(
            &home,
            &second_archive,
            TEST_VERSION,
            ArchiveLimits::default(),
            owner("rollback-fault"),
            |_root, candidate, _target| Ok(candidate.clone()),
        )
        .expect("prepared open");
        let PreparedOpenWorkspaceOutcome::Writable(pending) = prepared else {
            panic!("prepared open is writable");
        };
        inject_commit_failure(CommitOperation::Rename, 0);
        let rollback_error = pending.abort().expect_err("injected open rollback failure");
        assert!(matches!(
            rollback_error,
            WorkspaceError::CommitRecovery {
                operation: "open",
                ..
            }
        ));
        assert_eq!(
            fs::read(home.active_workspace().join(MANIFEST_FILE))
                .expect("Drop retries open rollback"),
            prior_active
        );
    }
}
