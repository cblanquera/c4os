//! Protected C4OS Home registry for persistent Browser Environment profiles.
//!
//! The registry stores only stable scope-to-WebKit-identifier mappings. Raw
//! cookies and website data remain owned by WebKit and are never represented by
//! this module. `None`/ephemeral Browser Environments are deliberately absent
//! from the persistent scope type.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::{Uuid, Variant, Version};

use crate::core::configuration::BrowserEnvironment;

pub const BROWSER_PROFILE_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const MAX_BROWSER_PROFILE_REGISTRY_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_BROWSER_PROFILE_ENTRIES: usize = 32_768;
pub const MAX_BROWSER_PROFILE_IDENTIFIER_BYTES: usize = 160;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersistentProfileScope {
    AppWide,
    WorkspaceProject {
        workspace_id: String,
        project_id: String,
    },
    Chat {
        workspace_id: String,
        chat_id: String,
    },
}

pub fn persistent_profile_scope(
    environment: BrowserEnvironment,
    workspace_id: &str,
    project_id: &str,
    chat_id: &str,
) -> Option<PersistentProfileScope> {
    match environment {
        BrowserEnvironment::AppWide => Some(PersistentProfileScope::AppWide),
        BrowserEnvironment::WorkspaceProject => Some(PersistentProfileScope::WorkspaceProject {
            workspace_id: workspace_id.to_owned(),
            project_id: project_id.to_owned(),
        }),
        BrowserEnvironment::Chat => Some(PersistentProfileScope::Chat {
            workspace_id: workspace_id.to_owned(),
            chat_id: chat_id.to_owned(),
        }),
        BrowserEnvironment::None => None,
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersistentProfileLifecycle {
    Ready,
    ClearPending {
        operation_id: String,
        target_data_generation: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistentProfileSnapshot {
    pub scope: PersistentProfileScope,
    pub profile_id: String,
    pub data_generation: u64,
    pub lifecycle: PersistentProfileLifecycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserProfileRegistrySnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub profiles: Vec<PersistentProfileSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPersistentProfile {
    pub profile: PersistentProfileSnapshot,
    pub registry_generation: u64,
    pub created: bool,
}

#[derive(Debug, Error)]
pub enum BrowserProfileRegistryError {
    #[error("Browser profile registry I/O failed while attempting to {operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("Browser profile registry exceeds its byte limit")]
    RegistryTooLarge,
    #[error("Browser profile registry exceeds its profile-entry limit")]
    TooManyProfiles,
    #[error("Browser profile registry is not valid UTF-8")]
    InvalidUtf8,
    #[error("Browser profile registry TOML is invalid: {0}")]
    InvalidToml(#[from] toml::de::Error),
    #[error("Browser profile registry TOML serialization failed: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
    #[error("Browser profile registry schema version {0} is unsupported")]
    UnsupportedSchema(u16),
    #[error("Browser profile registry is not in canonical form")]
    NonCanonicalDocument,
    #[error("Browser profile registry generation must be positive")]
    InvalidRegistryGeneration,
    #[error("Browser profile registry generation overflowed")]
    RegistryGenerationOverflow,
    #[error("Browser profile data generation is invalid")]
    InvalidDataGeneration,
    #[error("Browser profile scope is invalid: {0}")]
    InvalidScope(&'static str),
    #[error("Browser profile identifier is not a canonical RFC 4122 version-4 UUID")]
    InvalidProfileIdentifier,
    #[error(
        "Browser profile clear operation identifier is not a canonical RFC 4122 version-4 UUID"
    )]
    InvalidOperationIdentifier,
    #[error("Browser profile registry repeats a persistent scope")]
    DuplicateScope,
    #[error("Browser profile registry reuses a WebKit profile identifier")]
    DuplicateProfileIdentifier,
    #[error("Browser profile registry entries are not deterministically sorted")]
    UnsortedProfiles,
    #[error("Browser profile registry generation conflict: expected {expected}, active {actual}")]
    GenerationConflict { expected: u64, actual: u64 },
    #[error("Browser profile data generation conflict: expected {expected}, active {actual}")]
    DataGenerationConflict { expected: u64, actual: u64 },
    #[error("Browser profile scope is not registered")]
    ProfileNotFound,
    #[error("Browser profile clear is already pending")]
    ClearAlreadyPending,
    #[error("Browser profile clear operation does not match the pending operation")]
    ClearOperationMismatch,
    #[error("Browser profile registry path is a symbolic link")]
    SymbolicLink,
    #[error("Browser profile registry path is not a regular file")]
    NonRegularFile,
    #[error("Browser profile registry parent is not a real directory")]
    InvalidParent,
    #[error("Browser profile registry permissions must be 0600")]
    UnsafePermissions,
    #[error("Browser profile registry changed outside the active registry service")]
    ExternalReplacement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryDocument {
    schema_version: u16,
    generation: u64,
    profiles: Vec<ProfileRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileRecord {
    profile_id: String,
    data_generation: u64,
    scope: PersistentProfileScope,
    lifecycle: PersistentProfileLifecycle,
}

#[derive(Clone, Copy)]
enum ExpectedDiskState {
    Missing,
    Fingerprint([u8; 32]),
}

pub struct BrowserProfileRegistry {
    path: PathBuf,
    document: RegistryDocument,
    disk_fingerprint: [u8; 32],
}

impl BrowserProfileRegistry {
    /// Loads the protected registry, creating one empty canonical v1 document
    /// when the path does not exist. Any existing malformed or noncanonical
    /// document fails closed and is never replaced with freshly minted IDs.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, BrowserProfileRegistryError> {
        let path = path.into();
        ensure_parent(&path)?;
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                let bytes = read_registry_file(&path)?;
                let document = decode_document(&bytes)?;
                Ok(Self {
                    path,
                    document,
                    disk_fingerprint: fingerprint(&bytes),
                })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let document = RegistryDocument {
                    schema_version: BROWSER_PROFILE_REGISTRY_SCHEMA_VERSION,
                    generation: 1,
                    profiles: Vec::new(),
                };
                let bytes = encode_document(&document)?;
                persist_document(&path, &bytes, ExpectedDiskState::Missing)?;
                Ok(Self {
                    path,
                    document,
                    disk_fingerprint: fingerprint(&bytes),
                })
            }
            Err(source) => Err(io_error("inspect registry", source)),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn snapshot(&self) -> BrowserProfileRegistrySnapshot {
        BrowserProfileRegistrySnapshot {
            schema_version: self.document.schema_version,
            generation: self.document.generation,
            profiles: self
                .document
                .profiles
                .iter()
                .map(ProfileRecord::snapshot)
                .collect(),
        }
    }

    /// Returns an existing stable mapping or durably creates it before exposing
    /// the WebKit identifier. The caller must present the exact registry
    /// generation even when the mapping already exists.
    pub fn resolve(
        &mut self,
        expected_generation: u64,
        scope: PersistentProfileScope,
    ) -> Result<ResolvedPersistentProfile, BrowserProfileRegistryError> {
        self.require_generation(expected_generation)?;
        validate_scope(&scope)?;
        if let Some(profile) = self
            .document
            .profiles
            .iter()
            .find(|profile| profile.scope == scope)
        {
            return Ok(ResolvedPersistentProfile {
                profile: profile.snapshot(),
                registry_generation: self.document.generation,
                created: false,
            });
        }
        if self.document.profiles.len() >= MAX_BROWSER_PROFILE_ENTRIES {
            return Err(BrowserProfileRegistryError::TooManyProfiles);
        }

        let mut next = self.document.clone();
        next.generation = next_generation(next.generation)?;
        next.profiles.push(ProfileRecord {
            profile_id: Uuid::new_v4().to_string(),
            data_generation: 1,
            scope: scope.clone(),
            lifecycle: PersistentProfileLifecycle::Ready,
        });
        next.profiles
            .sort_by(|left, right| left.scope.cmp(&right.scope));
        self.commit(next)?;
        let profile = self
            .document
            .profiles
            .iter()
            .find(|profile| profile.scope == scope)
            .expect("the committed registry contains the resolved scope")
            .snapshot();
        Ok(ResolvedPersistentProfile {
            profile,
            registry_generation: self.document.generation,
            created: true,
        })
    }

    /// Durably records the recovery marker before WebKit data removal begins.
    /// Repeating the same operation against the active generation is idempotent.
    pub fn mark_clear_pending(
        &mut self,
        expected_generation: u64,
        scope: &PersistentProfileScope,
        expected_data_generation: u64,
        operation_id: impl Into<String>,
    ) -> Result<PersistentProfileSnapshot, BrowserProfileRegistryError> {
        self.require_generation(expected_generation)?;
        validate_scope(scope)?;
        let operation_id = operation_id.into();
        validate_v4_uuid(&operation_id)
            .map_err(|_| BrowserProfileRegistryError::InvalidOperationIdentifier)?;

        let current = self
            .document
            .profiles
            .iter()
            .find(|profile| &profile.scope == scope)
            .ok_or(BrowserProfileRegistryError::ProfileNotFound)?;
        if current.data_generation != expected_data_generation {
            return Err(BrowserProfileRegistryError::DataGenerationConflict {
                expected: expected_data_generation,
                actual: current.data_generation,
            });
        }
        if let PersistentProfileLifecycle::ClearPending {
            operation_id: active_operation,
            target_data_generation,
        } = &current.lifecycle
        {
            if active_operation == &operation_id
                && *target_data_generation == next_data_generation(current.data_generation)?
            {
                return Ok(current.snapshot());
            }
            return Err(BrowserProfileRegistryError::ClearAlreadyPending);
        }

        let target_data_generation = next_data_generation(current.data_generation)?;
        let mut next = self.document.clone();
        next.generation = next_generation(next.generation)?;
        let pending = next
            .profiles
            .iter_mut()
            .find(|profile| &profile.scope == scope)
            .expect("the cloned registry retains the selected scope");
        pending.lifecycle = PersistentProfileLifecycle::ClearPending {
            operation_id,
            target_data_generation,
        };
        self.commit(next)?;
        Ok(self
            .document
            .profiles
            .iter()
            .find(|profile| &profile.scope == scope)
            .expect("the committed registry retains the selected scope")
            .snapshot())
    }

    /// Completes the exact pending clear only after the caller has removed all
    /// public WebKit data types and released the cleared data-store handle.
    pub fn complete_clear(
        &mut self,
        expected_generation: u64,
        scope: &PersistentProfileScope,
        operation_id: &str,
        target_data_generation: u64,
    ) -> Result<PersistentProfileSnapshot, BrowserProfileRegistryError> {
        self.require_generation(expected_generation)?;
        validate_scope(scope)?;
        validate_v4_uuid(operation_id)
            .map_err(|_| BrowserProfileRegistryError::InvalidOperationIdentifier)?;
        let current = self
            .document
            .profiles
            .iter()
            .find(|profile| &profile.scope == scope)
            .ok_or(BrowserProfileRegistryError::ProfileNotFound)?;
        match &current.lifecycle {
            PersistentProfileLifecycle::ClearPending {
                operation_id: active_operation,
                target_data_generation: active_target,
            } if active_operation == operation_id && *active_target == target_data_generation => {}
            _ => return Err(BrowserProfileRegistryError::ClearOperationMismatch),
        }
        if target_data_generation != next_data_generation(current.data_generation)? {
            return Err(BrowserProfileRegistryError::InvalidDataGeneration);
        }

        let mut next = self.document.clone();
        next.generation = next_generation(next.generation)?;
        let completed = next
            .profiles
            .iter_mut()
            .find(|profile| &profile.scope == scope)
            .expect("the cloned registry retains the selected scope");
        completed.data_generation = target_data_generation;
        completed.lifecycle = PersistentProfileLifecycle::Ready;
        self.commit(next)?;
        Ok(self
            .document
            .profiles
            .iter()
            .find(|profile| &profile.scope == scope)
            .expect("the committed registry retains the selected scope")
            .snapshot())
    }

    fn require_generation(&self, expected: u64) -> Result<(), BrowserProfileRegistryError> {
        if expected != self.document.generation {
            return Err(BrowserProfileRegistryError::GenerationConflict {
                expected,
                actual: self.document.generation,
            });
        }
        Ok(())
    }

    fn commit(&mut self, document: RegistryDocument) -> Result<(), BrowserProfileRegistryError> {
        validate_document(&document)?;
        let bytes = encode_document(&document)?;
        persist_document(
            &self.path,
            &bytes,
            ExpectedDiskState::Fingerprint(self.disk_fingerprint),
        )?;
        self.document = document;
        self.disk_fingerprint = fingerprint(&bytes);
        Ok(())
    }
}

impl ProfileRecord {
    fn snapshot(&self) -> PersistentProfileSnapshot {
        PersistentProfileSnapshot {
            scope: self.scope.clone(),
            profile_id: self.profile_id.clone(),
            data_generation: self.data_generation,
            lifecycle: self.lifecycle.clone(),
        }
    }
}

fn decode_document(bytes: &[u8]) -> Result<RegistryDocument, BrowserProfileRegistryError> {
    if bytes.len() > MAX_BROWSER_PROFILE_REGISTRY_BYTES {
        return Err(BrowserProfileRegistryError::RegistryTooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| BrowserProfileRegistryError::InvalidUtf8)?;
    let document: RegistryDocument = toml::from_str(text)?;
    validate_document(&document)?;
    if encode_document(&document)? != bytes {
        return Err(BrowserProfileRegistryError::NonCanonicalDocument);
    }
    Ok(document)
}

fn encode_document(document: &RegistryDocument) -> Result<Vec<u8>, BrowserProfileRegistryError> {
    let bytes = toml::to_string(document)?.into_bytes();
    if bytes.len() > MAX_BROWSER_PROFILE_REGISTRY_BYTES {
        return Err(BrowserProfileRegistryError::RegistryTooLarge);
    }
    Ok(bytes)
}

fn validate_document(document: &RegistryDocument) -> Result<(), BrowserProfileRegistryError> {
    if document.schema_version != BROWSER_PROFILE_REGISTRY_SCHEMA_VERSION {
        return Err(BrowserProfileRegistryError::UnsupportedSchema(
            document.schema_version,
        ));
    }
    if document.generation == 0 {
        return Err(BrowserProfileRegistryError::InvalidRegistryGeneration);
    }
    if document.profiles.len() > MAX_BROWSER_PROFILE_ENTRIES {
        return Err(BrowserProfileRegistryError::TooManyProfiles);
    }

    let mut scopes = BTreeSet::new();
    let mut profile_ids = BTreeSet::new();
    let mut previous_scope: Option<&PersistentProfileScope> = None;
    for profile in &document.profiles {
        validate_scope(&profile.scope)?;
        validate_v4_uuid(&profile.profile_id)
            .map_err(|_| BrowserProfileRegistryError::InvalidProfileIdentifier)?;
        if profile.data_generation == 0 {
            return Err(BrowserProfileRegistryError::InvalidDataGeneration);
        }
        if previous_scope.is_some_and(|previous| previous >= &profile.scope) {
            return Err(BrowserProfileRegistryError::UnsortedProfiles);
        }
        previous_scope = Some(&profile.scope);
        if !scopes.insert(profile.scope.clone()) {
            return Err(BrowserProfileRegistryError::DuplicateScope);
        }
        if !profile_ids.insert(profile.profile_id.clone()) {
            return Err(BrowserProfileRegistryError::DuplicateProfileIdentifier);
        }
        if let PersistentProfileLifecycle::ClearPending {
            operation_id,
            target_data_generation,
        } = &profile.lifecycle
        {
            validate_v4_uuid(operation_id)
                .map_err(|_| BrowserProfileRegistryError::InvalidOperationIdentifier)?;
            if *target_data_generation != next_data_generation(profile.data_generation)? {
                return Err(BrowserProfileRegistryError::InvalidDataGeneration);
            }
        }
    }
    Ok(())
}

fn validate_scope(scope: &PersistentProfileScope) -> Result<(), BrowserProfileRegistryError> {
    let identifiers: &[&str] = match scope {
        PersistentProfileScope::AppWide => &[],
        PersistentProfileScope::WorkspaceProject {
            workspace_id,
            project_id,
        } => &[workspace_id, project_id],
        PersistentProfileScope::Chat {
            workspace_id,
            chat_id,
        } => &[workspace_id, chat_id],
    };
    if identifiers.iter().any(|value| !valid_identifier(value)) {
        return Err(BrowserProfileRegistryError::InvalidScope(
            "scope identifiers must use bounded portable identifier syntax",
        ));
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BROWSER_PROFILE_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

fn validate_v4_uuid(value: &str) -> Result<(), ()> {
    let parsed = Uuid::parse_str(value).map_err(|_| ())?;
    if parsed.to_string() != value
        || parsed.get_variant() != Variant::RFC4122
        || parsed.get_version() != Some(Version::Random)
    {
        return Err(());
    }
    Ok(())
}

fn next_generation(generation: u64) -> Result<u64, BrowserProfileRegistryError> {
    generation
        .checked_add(1)
        .ok_or(BrowserProfileRegistryError::RegistryGenerationOverflow)
}

fn next_data_generation(generation: u64) -> Result<u64, BrowserProfileRegistryError> {
    generation
        .checked_add(1)
        .filter(|next| *next > 1)
        .ok_or(BrowserProfileRegistryError::InvalidDataGeneration)
}

fn ensure_parent(path: &Path) -> Result<(), BrowserProfileRegistryError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or(BrowserProfileRegistryError::InvalidParent)?;
    match fs::symlink_metadata(parent) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(BrowserProfileRegistryError::InvalidParent);
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(parent)
                .map_err(|source| io_error("create registry parent", source))?;
            let metadata = fs::symlink_metadata(parent)
                .map_err(|source| io_error("inspect registry parent", source))?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(BrowserProfileRegistryError::InvalidParent);
            }
        }
        Err(source) => return Err(io_error("inspect registry parent", source)),
    }
    Ok(())
}

fn read_registry_file(path: &Path) -> Result<Vec<u8>, BrowserProfileRegistryError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|source| io_error("inspect registry", source))?;
    if metadata.file_type().is_symlink() {
        return Err(BrowserProfileRegistryError::SymbolicLink);
    }
    if !metadata.is_file() {
        return Err(BrowserProfileRegistryError::NonRegularFile);
    }
    require_private_permissions(&metadata)?;
    if metadata.len() > MAX_BROWSER_PROFILE_REGISTRY_BYTES as u64 {
        return Err(BrowserProfileRegistryError::RegistryTooLarge);
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options
        .open(path)
        .map_err(|source| io_error("open registry", source))?;
    let opened = file
        .metadata()
        .map_err(|source| io_error("inspect open registry", source))?;
    if !opened.is_file() {
        return Err(BrowserProfileRegistryError::NonRegularFile);
    }
    require_private_permissions(&opened)?;
    if opened.len() > MAX_BROWSER_PROFILE_REGISTRY_BYTES as u64 {
        return Err(BrowserProfileRegistryError::RegistryTooLarge);
    }
    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.read_to_end(&mut bytes)
        .map_err(|source| io_error("read registry", source))?;
    if bytes.len() > MAX_BROWSER_PROFILE_REGISTRY_BYTES {
        return Err(BrowserProfileRegistryError::RegistryTooLarge);
    }
    Ok(bytes)
}

fn persist_document(
    path: &Path,
    bytes: &[u8],
    expected: ExpectedDiskState,
) -> Result<(), BrowserProfileRegistryError> {
    ensure_parent(path)?;
    verify_disk_state(path, expected)?;
    let parent = path
        .parent()
        .ok_or(BrowserProfileRegistryError::InvalidParent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(BrowserProfileRegistryError::InvalidParent)?;
    let temporary = parent.join(format!(".{file_name}.c4os-{}.tmp", Uuid::new_v4()));

    let write_result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|source| io_error("create temporary registry", source))?;
        file.write_all(bytes)
            .map_err(|source| io_error("write temporary registry", source))?;
        file.sync_all()
            .map_err(|source| io_error("sync temporary registry", source))?;
        drop(file);

        // A second check prevents an unnoticed replacement while the temporary
        // file was being written. The process-local registry service remains the
        // single intended writer.
        verify_disk_state(path, expected)?;
        fs::rename(&temporary, path).map_err(|source| io_error("replace registry", source))?;
        sync_parent(parent)?;
        Ok::<(), BrowserProfileRegistryError>(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn verify_disk_state(
    path: &Path,
    expected: ExpectedDiskState,
) -> Result<(), BrowserProfileRegistryError> {
    match expected {
        ExpectedDiskState::Missing => match fs::symlink_metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(BrowserProfileRegistryError::ExternalReplacement),
            Err(source) => Err(io_error("inspect registry base", source)),
        },
        ExpectedDiskState::Fingerprint(expected) => {
            let bytes = read_registry_file(path).map_err(|error| match error {
                BrowserProfileRegistryError::Io { source, .. }
                    if source.kind() == io::ErrorKind::NotFound =>
                {
                    BrowserProfileRegistryError::ExternalReplacement
                }
                other => other,
            })?;
            if fingerprint(&bytes) == expected {
                Ok(())
            } else {
                Err(BrowserProfileRegistryError::ExternalReplacement)
            }
        }
    }
}

#[cfg(unix)]
fn require_private_permissions(metadata: &fs::Metadata) -> Result<(), BrowserProfileRegistryError> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(BrowserProfileRegistryError::UnsafePermissions);
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_private_permissions(
    _metadata: &fs::Metadata,
) -> Result<(), BrowserProfileRegistryError> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent(parent: &Path) -> Result<(), BrowserProfileRegistryError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| io_error("sync registry parent", source))
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Path) -> Result<(), BrowserProfileRegistryError> {
    Ok(())
}

fn fingerprint(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn io_error(operation: &'static str, source: io::Error) -> BrowserProfileRegistryError {
    BrowserProfileRegistryError::Io { operation, source }
}
