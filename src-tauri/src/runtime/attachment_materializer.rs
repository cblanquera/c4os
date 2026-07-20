//! Rust-owned materialization of immutable Workspace attachment blobs.
//!
//! A durable attachment carries metadata only. This boundary is the only path
//! that turns its content-addressed reference into bounded bytes for a runtime
//! peer. Candidate paths are derived solely from the lowercase SHA-256 digest,
//! and every read rejects links, missing content, metadata drift, and digest
//! mismatch before dispatch may mutate a Chat.

use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use thiserror::Error;

use crate::runtime::session::AttachmentSnapshot;

const MAX_ATTACHMENTS: usize = 32;
const MAX_ATTACHMENT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MATERIALIZED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_WORKSPACE_ID_BYTES: usize = 192;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAttachmentContent {
    snapshot: AttachmentSnapshot,
    content: Vec<u8>,
}

impl VerifiedAttachmentContent {
    pub fn snapshot(&self) -> &AttachmentSnapshot {
        &self.snapshot
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentContentPlan {
    workspace_id: String,
    attachments: Vec<VerifiedAttachmentContent>,
}

impl AttachmentContentPlan {
    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    pub fn attachments(&self) -> &[VerifiedAttachmentContent] {
        &self.attachments
    }
}

#[derive(Clone, Debug)]
pub struct WorkspaceAttachmentMaterializer {
    workspace_id: String,
    workspace_directory: Arc<File>,
}

impl WorkspaceAttachmentMaterializer {
    pub fn bind(
        workspace_id: impl Into<String>,
        workspace_root: impl AsRef<Path>,
    ) -> Result<Self, AttachmentMaterializationError> {
        let workspace_id = workspace_id.into();
        if !valid_workspace_id(&workspace_id) {
            return Err(AttachmentMaterializationError::InvalidBinding);
        }
        let unresolved = workspace_root.as_ref();
        let metadata = fs::symlink_metadata(unresolved)
            .map_err(|_| AttachmentMaterializationError::InvalidBinding)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AttachmentMaterializationError::InvalidBinding);
        }
        let workspace_root = unresolved
            .canonicalize()
            .map_err(|_| AttachmentMaterializationError::InvalidBinding)?;
        let workspace_directory = Arc::new(
            open_directory(&workspace_root)
                .map_err(|_| AttachmentMaterializationError::InvalidBinding)?,
        );
        Ok(Self {
            workspace_id,
            workspace_directory,
        })
    }

    pub fn materialize(
        &self,
        attachments: &[AttachmentSnapshot],
    ) -> Result<AttachmentContentPlan, AttachmentMaterializationError> {
        if attachments.is_empty() || attachments.len() > MAX_ATTACHMENTS {
            return Err(AttachmentMaterializationError::InvalidAttachment);
        }
        let mut attachment_ids = BTreeSet::new();
        let mut aggregate_bytes = 0_u64;
        let mut digest_hexes = Vec::with_capacity(attachments.len());
        for attachment in attachments {
            if !attachment_ids.insert(attachment.attachment_id.as_str()) {
                return Err(AttachmentMaterializationError::InvalidAttachment);
            }
            digest_hexes.push(validate_snapshot_reference(attachment)?);
            aggregate_bytes = aggregate_bytes
                .checked_add(attachment.byte_length)
                .filter(|total| *total <= MAX_MATERIALIZED_BYTES)
                .ok_or(AttachmentMaterializationError::AggregateLimitExceeded)?;
        }

        let blobs_directory = openat_directory(&self.workspace_directory, "blobs")?;
        let digest_directory = openat_directory(&blobs_directory, "sha256")?;
        let mut verified = Vec::with_capacity(attachments.len());
        for (attachment, digest_hex) in attachments.iter().zip(digest_hexes) {
            let content = read_verified_blob(&digest_directory, digest_hex, attachment)?;
            verified.push(VerifiedAttachmentContent {
                snapshot: attachment.clone(),
                content,
            });
        }
        Ok(AttachmentContentPlan {
            workspace_id: self.workspace_id.clone(),
            attachments: verified,
        })
    }
}

#[cfg(unix)]
fn open_directory(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    options.open(path)
}

#[cfg(unix)]
fn openat_directory(parent: &File, name: &str) -> Result<File, AttachmentMaterializationError> {
    openat(parent, name, libc::O_RDONLY | libc::O_DIRECTORY)
}

#[cfg(unix)]
fn openat(
    parent: &File,
    name: &str,
    flags: libc::c_int,
) -> Result<File, AttachmentMaterializationError> {
    let name = CString::new(name).map_err(|_| AttachmentMaterializationError::BlobMissing)?;
    // SAFETY: `parent` is a live owned directory descriptor, `name` is a
    // NUL-terminated single validated component, and the returned descriptor
    // is immediately transferred into one owning `File`.
    let descriptor = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        return Err(map_blob_open_error(io::Error::last_os_error()));
    }
    // SAFETY: `openat` returned a new descriptor and ownership is transferred
    // exactly once to this `File`.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

fn validate_snapshot_reference(
    attachment: &AttachmentSnapshot,
) -> Result<&str, AttachmentMaterializationError> {
    let digest_hex = attachment
        .content_sha256
        .strip_prefix("sha256:")
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        })
        .ok_or(AttachmentMaterializationError::InvalidAttachment)?;
    if attachment.attachment_id.is_empty()
        || attachment.media_type.trim().is_empty()
        || attachment.display_name.trim().is_empty()
        || attachment.byte_length == 0
        || attachment.byte_length > MAX_ATTACHMENT_BYTES
        || attachment.snapshot_version == 0
        || attachment.stable_reference
            != format!(
                "workspace-blob:{}:v{}",
                attachment.content_sha256, attachment.snapshot_version
            )
    {
        return Err(AttachmentMaterializationError::ReferenceMismatch);
    }
    Ok(digest_hex)
}

fn read_verified_blob(
    digest_directory: &File,
    digest_hex: &str,
    attachment: &AttachmentSnapshot,
) -> Result<Vec<u8>, AttachmentMaterializationError> {
    let file = openat(digest_directory, digest_hex, libc::O_RDONLY)?;
    read_and_verify(file, attachment)
}

fn read_and_verify(
    mut file: File,
    attachment: &AttachmentSnapshot,
) -> Result<Vec<u8>, AttachmentMaterializationError> {
    let before = file
        .metadata()
        .map_err(|_| AttachmentMaterializationError::BlobMissing)?;
    if !before.is_file() || before.len() != attachment.byte_length {
        return Err(AttachmentMaterializationError::LengthMismatch);
    }
    let capacity = usize::try_from(attachment.byte_length)
        .map_err(|_| AttachmentMaterializationError::LengthMismatch)?;
    let mut content = Vec::with_capacity(capacity);
    file.by_ref()
        .take(MAX_ATTACHMENT_BYTES + 1)
        .read_to_end(&mut content)
        .map_err(|_| AttachmentMaterializationError::BlobMissing)?;
    let after = file
        .metadata()
        .map_err(|_| AttachmentMaterializationError::BlobMissing)?;
    if content.len() as u64 != attachment.byte_length || before.len() != after.len() {
        return Err(AttachmentMaterializationError::LengthMismatch);
    }
    if prefixed_sha256(&content) != attachment.content_sha256 {
        return Err(AttachmentMaterializationError::DigestMismatch);
    }
    Ok(content)
}

fn map_blob_open_error(error: io::Error) -> AttachmentMaterializationError {
    #[cfg(unix)]
    if let Some(code) = error.raw_os_error()
        && code == libc::ELOOP
    {
        return AttachmentMaterializationError::SymlinkRejected;
    }
    AttachmentMaterializationError::BlobMissing
}

fn prefixed_sha256(content: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in Sha256::digest(content) {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn valid_workspace_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_WORKSPACE_ID_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AttachmentMaterializationError {
    #[error("Workspace attachment materializer binding is invalid")]
    InvalidBinding,
    #[error("durable attachment metadata is invalid")]
    InvalidAttachment,
    #[error("durable attachment content exceeds the aggregate materialization bound")]
    AggregateLimitExceeded,
    #[error("durable attachment reference does not match its digest and version")]
    ReferenceMismatch,
    #[error("Workspace attachment blob is missing")]
    BlobMissing,
    #[error("Workspace attachment path contains a symbolic link")]
    SymlinkRejected,
    #[error("Workspace attachment length changed or mismatches its snapshot")]
    LengthMismatch,
    #[error("Workspace attachment digest does not match its snapshot")]
    DigestMismatch,
}
