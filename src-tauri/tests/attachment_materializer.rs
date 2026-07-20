//! Exact Workspace content-addressed attachment materialization coverage.

use std::fs;
use std::os::unix::fs::symlink;

use c4os_lib::runtime::attachment_materializer::{
    AttachmentMaterializationError, WorkspaceAttachmentMaterializer,
};
use c4os_lib::runtime::session::AttachmentSnapshot;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(71);
    encoded.push_str("sha256:");
    for byte in Sha256::digest(bytes) {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn fixture(
    media_type: &str,
    display_name: &str,
    bytes: &[u8],
) -> (
    TempDir,
    std::path::PathBuf,
    WorkspaceAttachmentMaterializer,
    AttachmentSnapshot,
) {
    let temporary = TempDir::new().unwrap();
    let workspace = temporary.path().join("workspace-active");
    let blobs = workspace.join("blobs/sha256");
    fs::create_dir_all(&blobs).unwrap();
    let content_sha256 = digest(bytes);
    let digest_hex = content_sha256.strip_prefix("sha256:").unwrap();
    fs::write(blobs.join(digest_hex), bytes).unwrap();
    let materializer = WorkspaceAttachmentMaterializer::bind("workspace-1", &workspace).unwrap();
    let attachment = AttachmentSnapshot {
        attachment_id: "attachment-1".into(),
        stable_reference: format!("workspace-blob:{content_sha256}:v3"),
        display_name: display_name.into(),
        media_type: media_type.into(),
        byte_length: bytes.len() as u64,
        content_sha256,
        snapshot_version: 3,
    };
    (temporary, workspace, materializer, attachment)
}

#[test]
fn image_and_pdf_are_read_only_from_the_bound_content_addressed_store() {
    for (media_type, name, bytes) in [
        ("image/png", "concept.png", b"known-png-content".as_slice()),
        (
            "application/pdf",
            "brief.pdf",
            b"%PDF-known-content".as_slice(),
        ),
    ] {
        let (_temporary, _workspace, materializer, attachment) = fixture(media_type, name, bytes);
        let plan = materializer
            .materialize(std::slice::from_ref(&attachment))
            .unwrap();
        assert_eq!(plan.workspace_id(), "workspace-1");
        assert_eq!(plan.attachments().len(), 1);
        assert_eq!(plan.attachments()[0].snapshot(), &attachment);
        assert_eq!(plan.attachments()[0].content(), bytes);
    }
}

#[test]
fn aggregate_limit_rejects_multiple_valid_snapshots_before_any_blob_read() {
    let (_temporary, _workspace, materializer, mut first) =
        fixture("image/png", "concept.png", b"known");
    first.byte_length = 33 * 1024 * 1024;
    let mut second = first.clone();
    second.attachment_id = "attachment-2".into();
    second.display_name = "concept-2.png".into();
    second.content_sha256 = format!("sha256:{}", "b".repeat(64));
    second.stable_reference = format!(
        "workspace-blob:{}:v{}",
        second.content_sha256, second.snapshot_version
    );

    assert_eq!(
        materializer.materialize(&[first, second]).unwrap_err(),
        AttachmentMaterializationError::AggregateLimitExceeded
    );
}

#[test]
fn missing_tampered_length_and_version_mismatches_fail_closed() {
    let (_temporary, workspace, materializer, attachment) =
        fixture("image/png", "concept.png", b"known");
    let digest_hex = attachment.content_sha256.strip_prefix("sha256:").unwrap();
    let blob = workspace.join("blobs/sha256").join(digest_hex);

    fs::remove_file(&blob).unwrap();
    assert_eq!(
        materializer
            .materialize(std::slice::from_ref(&attachment))
            .unwrap_err(),
        AttachmentMaterializationError::BlobMissing
    );
    fs::write(&blob, b"tampered").unwrap();
    assert_eq!(
        materializer
            .materialize(std::slice::from_ref(&attachment))
            .unwrap_err(),
        AttachmentMaterializationError::LengthMismatch
    );
    fs::write(&blob, b"bad!!").unwrap();
    assert_eq!(
        materializer
            .materialize(std::slice::from_ref(&attachment))
            .unwrap_err(),
        AttachmentMaterializationError::DigestMismatch
    );
    fs::write(&blob, b"known").unwrap();
    let mut wrong_length = attachment.clone();
    wrong_length.byte_length += 1;
    assert_eq!(
        materializer.materialize(&[wrong_length]).unwrap_err(),
        AttachmentMaterializationError::LengthMismatch
    );
    let mut wrong_version = attachment;
    wrong_version.snapshot_version = 4;
    assert_eq!(
        materializer.materialize(&[wrong_version]).unwrap_err(),
        AttachmentMaterializationError::ReferenceMismatch
    );
}

#[test]
fn symlink_blob_is_rejected_without_following_ambient_filesystem_content() {
    let (_temporary, workspace, materializer, attachment) =
        fixture("image/png", "concept.png", b"known");
    let digest_hex = attachment.content_sha256.strip_prefix("sha256:").unwrap();
    let blob = workspace.join("blobs/sha256").join(digest_hex);
    let outside = workspace.parent().unwrap().join("outside");
    fs::write(&outside, b"known").unwrap();
    fs::remove_file(&blob).unwrap();
    symlink(&outside, &blob).unwrap();

    assert_eq!(
        materializer.materialize(&[attachment]).unwrap_err(),
        AttachmentMaterializationError::SymlinkRejected
    );
}

#[test]
fn bound_directory_descriptor_prevents_workspace_root_substitution() {
    let (temporary, workspace, materializer, attachment) =
        fixture("image/png", "concept.png", b"known");
    let moved_workspace = temporary.path().join("workspace-original");
    fs::rename(&workspace, &moved_workspace).unwrap();
    let replacement_blobs = workspace.join("blobs/sha256");
    fs::create_dir_all(&replacement_blobs).unwrap();
    let digest_hex = attachment.content_sha256.strip_prefix("sha256:").unwrap();
    fs::write(replacement_blobs.join(digest_hex), b"evil!").unwrap();

    let plan = materializer.materialize(&[attachment]).unwrap();
    assert_eq!(plan.attachments()[0].content(), b"known");
}
