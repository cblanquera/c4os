use c4os_lib::core::workspace::{
    ArchiveLimits, ArchiveViolation, C4osHomeLayout, ManifestFile, OpenWorkspaceOutcome,
    ProjectPathStatus, ProjectReference, RecoveryDisposition, WorkspaceError, WorkspaceLayout,
    WorkspaceLockOwner, WorkspaceManifest, WorkspaceResult, WorkspaceSemanticValidationTarget,
    WorkspaceWriterLock, WriterAccess, acquire_workspace_writer_lock,
    create_untitled_working_copy as create_untitled_working_copy_guarded, detect_recovery,
    open_workspace_archive, persist_committed_generation as persist_committed_generation_guarded,
    preflight_workspace_archive_source, prepare_open_workspace_archive,
    prepare_workspace_archive_save, save_workspace_archive as save_workspace_archive_guarded,
    save_workspace_archive_by_copy as save_workspace_archive_by_copy_guarded, validate_archive,
};
use rusqlite::Connection;
use rusqlite::backup::Backup;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const APP_VERSION: &str = "0.1.0";

#[derive(Clone)]
struct FixtureEntry {
    name: String,
    data: Vec<u8>,
    compression: CompressionMethod,
    unix_permissions: u32,
}

type HostileCase = (&'static str, FixtureEntry, fn(&ArchiveViolation) -> bool);

impl FixtureEntry {
    fn file(name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
            compression: CompressionMethod::Deflated,
            unix_permissions: 0o600,
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("write digest");
    }
    output
}

fn manifest_with(entries: &[FixtureEntry]) -> WorkspaceManifest {
    let mut files: Vec<_> = entries
        .iter()
        .filter(|entry| entry.name != "manifest.toml")
        .map(|entry| ManifestFile {
            path: entry.name.clone(),
            sha256: digest(&entry.data),
            expanded_size: entry.data.len() as u64,
        })
        .collect();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    WorkspaceManifest {
        schema_version: 1,
        workspace_id: Uuid::new_v4(),
        generation: 1,
        minimum_app_version: APP_VERSION.into(),
        projects: Vec::new(),
        files,
    }
}

fn write_archive(
    path: &Path,
    manifest: &WorkspaceManifest,
    entries: &[FixtureEntry],
    duplicate_manifest: bool,
) {
    write_archive_with_manifest_text(
        path,
        &toml::to_string(manifest).expect("serialize manifest"),
        entries,
        duplicate_manifest,
    );
}

fn write_archive_with_manifest_text(
    path: &Path,
    manifest_text: &str,
    entries: &[FixtureEntry],
    duplicate_manifest: bool,
) {
    let output = File::create(path).expect("create fixture zip");
    let mut writer = ZipWriter::new(output);
    let manifest_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    writer
        .start_file("manifest.toml", manifest_options)
        .expect("manifest entry");
    writer
        .write_all(manifest_text.as_bytes())
        .expect("manifest bytes");
    if duplicate_manifest {
        writer
            .start_file("manifesx.toml", manifest_options)
            .expect("placeholder duplicate entry");
        writer.write_all(b"duplicate").expect("duplicate bytes");
    }
    for entry in entries {
        let options = SimpleFileOptions::default()
            .compression_method(entry.compression)
            .unix_permissions(entry.unix_permissions);
        if entry.unix_permissions & 0o170000 == 0o120000 {
            writer
                .add_symlink(&entry.name, String::from_utf8_lossy(&entry.data), options)
                .expect("fixture symlink");
            continue;
        }
        writer
            .start_file(&entry.name, options)
            .expect("fixture entry");
        writer.write_all(&entry.data).expect("fixture data");
    }
    writer.finish().expect("finish fixture zip");
    if duplicate_manifest {
        let mut bytes = fs::read(path).expect("read duplicate fixture");
        let placeholder = b"manifesx.toml";
        let duplicate = b"manifest.toml";
        let mut replacements = 0;
        for offset in 0..=bytes.len().saturating_sub(placeholder.len()) {
            if &bytes[offset..offset + placeholder.len()] == placeholder {
                bytes[offset..offset + duplicate.len()].copy_from_slice(duplicate);
                replacements += 1;
            }
        }
        assert_eq!(replacements, 2, "local and central names were patched");
        fs::write(path, bytes).expect("write duplicate fixture");
    }
}

fn assert_archive_violation(error: WorkspaceError, expected: fn(&ArchiveViolation) -> bool) {
    match error {
        WorkspaceError::Archive(violation) if expected(&violation) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

fn lock_owner(label: &str) -> WorkspaceLockOwner {
    WorkspaceLockOwner {
        process_id: std::process::id(),
        app_instance_id: Uuid::new_v4(),
        acquired_unix_ms: 1_721_312_000_000,
        label: label.into(),
    }
}

fn test_writer_lock(anchor: &Path) -> WorkspaceWriterLock {
    let parent = anchor.parent().unwrap_or(anchor);
    let lock_path = parent
        .join(".workspace-test-locks")
        .join(format!("{}.lock", Uuid::new_v4()));
    match acquire_workspace_writer_lock(&lock_path, lock_owner("test-writer"))
        .expect("acquire test writer lock")
    {
        WriterAccess::Writable(lock) => lock,
        WriterAccess::ReadOnly { .. } => panic!("unique test lock should be writable"),
    }
}

fn create_untitled_working_copy(
    working_root: &Path,
    project_folder: &Path,
    display_name: impl Into<String>,
    minimum_app_version: &str,
) -> WorkspaceResult<WorkspaceManifest> {
    let writer_lock = test_writer_lock(working_root);
    create_untitled_working_copy_guarded(
        &writer_lock,
        working_root,
        project_folder,
        display_name,
        minimum_app_version,
    )
}

fn persist_committed_generation(
    working_root: &Path,
    manifest: &mut WorkspaceManifest,
) -> WorkspaceResult<u64> {
    let writer_lock = test_writer_lock(working_root);
    persist_committed_generation_guarded(&writer_lock, working_root, manifest)
}

fn save_workspace_archive_by_copy(
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
) -> WorkspaceResult<c4os_lib::core::workspace::SavedWorkspace> {
    let writer_lock = test_writer_lock(working_root);
    save_workspace_archive_by_copy_guarded(
        &writer_lock,
        working_root,
        archive_path,
        manifest,
        current_app_version,
        limits,
    )
}

fn save_workspace_archive<F>(
    working_root: &Path,
    archive_path: &Path,
    manifest: &WorkspaceManifest,
    current_app_version: &str,
    limits: ArchiveLimits,
    backup_workspace_database: F,
) -> WorkspaceResult<c4os_lib::core::workspace::SavedWorkspace>
where
    F: FnOnce(&Path, &Path) -> WorkspaceResult<()>,
{
    let writer_lock = test_writer_lock(working_root);
    save_workspace_archive_guarded(
        &writer_lock,
        working_root,
        archive_path,
        manifest,
        current_app_version,
        limits,
        backup_workspace_database,
    )
}

fn create_working_copy(parent: &Path, project: &Path, name: &str) -> (PathBuf, WorkspaceManifest) {
    let root = parent.join(format!("working-{}", Uuid::new_v4()));
    let manifest = create_untitled_working_copy(&root, project, name, APP_VERSION)
        .expect("create working copy");
    (root, manifest)
}

#[test]
fn hostile_archive_matrix_rejects_paths_duplicates_types_and_declarations() {
    let temp = TempDir::new().expect("tempdir");
    let cases: Vec<HostileCase> = vec![
        (
            "traversal",
            FixtureEntry::file("../state/workspace.sqlite3", b"db"),
            |violation| matches!(violation, ArchiveViolation::InvalidEntryName(_)),
        ),
        (
            "absolute",
            FixtureEntry::file("/state/workspace.sqlite3", b"db"),
            |violation| matches!(violation, ArchiveViolation::InvalidEntryName(_)),
        ),
        (
            "backslash",
            FixtureEntry::file("state\\workspace.sqlite3", b"db"),
            |violation| matches!(violation, ArchiveViolation::InvalidEntryName(_)),
        ),
        (
            "credential-root",
            FixtureEntry::file("vault/credentials.vault", b"secret"),
            |violation| matches!(violation, ArchiveViolation::DisallowedRoot(_)),
        ),
        (
            "browser-root",
            FixtureEntry::file("browser/raw-cookies", b"cookie"),
            |violation| matches!(violation, ArchiveViolation::DisallowedRoot(_)),
        ),
        (
            "project-folder",
            FixtureEntry::file("projects/source-code.rs", b"source"),
            |violation| matches!(violation, ArchiveViolation::DisallowedRoot(_)),
        ),
        (
            "symlink",
            FixtureEntry {
                name: "state/workspace.sqlite3".into(),
                data: b"../../outside".to_vec(),
                compression: CompressionMethod::Deflated,
                unix_permissions: 0o120777,
            },
            |violation| matches!(violation, ArchiveViolation::UnsupportedEntryType(_)),
        ),
        (
            "unsupported-compression",
            FixtureEntry {
                name: "state/workspace.sqlite3".into(),
                data: b"db".to_vec(),
                compression: CompressionMethod::Stored,
                unix_permissions: 0o600,
            },
            |violation| matches!(violation, ArchiveViolation::UnsupportedCompression(_)),
        ),
    ];

    for (label, entry, expected) in cases {
        let path = temp.path().join(format!("{label}.zip"));
        let manifest = manifest_with(std::slice::from_ref(&entry));
        write_archive(&path, &manifest, &[entry], false);
        let error = validate_archive(&path, APP_VERSION, ArchiveLimits::default())
            .expect_err("hostile archive must fail");
        assert_archive_violation(error, expected);
    }

    let duplicate = temp.path().join("duplicate.zip");
    write_archive(&duplicate, &manifest_with(&[]), &[], true);
    let error = validate_archive(&duplicate, APP_VERSION, ArchiveLimits::default())
        .expect_err("duplicate entries must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::DuplicateEntry(_))
    });

    let undeclared = temp.path().join("undeclared.zip");
    let entry = FixtureEntry::file("config/workspace.toml", b"schema_version = 1");
    write_archive(&undeclared, &manifest_with(&[]), &[entry], false);
    let error = validate_archive(&undeclared, APP_VERSION, ArchiveLimits::default())
        .expect_err("undeclared entry must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::UndeclaredEntry(_))
    });
}

#[test]
fn hostile_archive_matrix_rejects_digest_size_count_ratio_encryption_and_versions() {
    let temp = TempDir::new().expect("tempdir");
    let entry = FixtureEntry::file("config/workspace.toml", pseudo_random_bytes(4_096));

    let digest_path = temp.path().join("digest.zip");
    let mut wrong_digest = manifest_with(std::slice::from_ref(&entry));
    wrong_digest.files[0].sha256 = "0".repeat(64);
    write_archive(
        &digest_path,
        &wrong_digest,
        std::slice::from_ref(&entry),
        false,
    );
    let error = validate_archive(&digest_path, APP_VERSION, ArchiveLimits::default())
        .expect_err("digest mismatch must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::DigestMismatch(_))
    });

    let size_path = temp.path().join("declared-size.zip");
    let mut wrong_size = manifest_with(std::slice::from_ref(&entry));
    wrong_size.files[0].expanded_size += 1;
    write_archive(&size_path, &wrong_size, std::slice::from_ref(&entry), false);
    let error = validate_archive(&size_path, APP_VERSION, ArchiveLimits::default())
        .expect_err("declared size mismatch must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::DeclaredSizeMismatch(_))
    });

    let missing_path = temp.path().join("missing.zip");
    let missing_manifest = manifest_with(std::slice::from_ref(&entry));
    write_archive(&missing_path, &missing_manifest, &[], false);
    let error = validate_archive(&missing_path, APP_VERSION, ArchiveLimits::default())
        .expect_err("missing declared entry must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::MissingDeclaredEntry(_))
    });

    let ratio_path = temp.path().join("ratio.zip");
    let ratio_entry = FixtureEntry::file("config/workspace.toml", vec![b'a'; 65_537]);
    let ratio_manifest = manifest_with(std::slice::from_ref(&ratio_entry));
    write_archive(
        &ratio_path,
        &ratio_manifest,
        std::slice::from_ref(&ratio_entry),
        false,
    );
    let ratio_limits = ArchiveLimits {
        max_compression_ratio: 2,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&ratio_path, APP_VERSION, ratio_limits)
        .expect_err("excessive compression ratio must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::CompressionRatioExceeded(_))
    });

    let mut ratio_zip = ZipArchive::new(File::open(&ratio_path).expect("ratio archive"))
        .expect("read ratio archive");
    let ratio_file = ratio_zip
        .by_name("config/workspace.toml")
        .expect("ratio entry");
    let truncated_ratio = ratio_file.size() / ratio_file.compressed_size();
    assert_ne!(
        ratio_file.size() % ratio_file.compressed_size(),
        0,
        "fixture must exercise the fractional ratio boundary"
    );
    drop(ratio_file);
    let exact_ratio_limits = ArchiveLimits {
        max_compression_ratio: truncated_ratio,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&ratio_path, APP_VERSION, exact_ratio_limits)
        .expect_err("fractional ratio above the exact limit must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::CompressionRatioExceeded(_))
    });

    let count_limits = ArchiveLimits {
        max_entries: 1,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&ratio_path, APP_VERSION, count_limits)
        .expect_err("entry count must be bounded");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::EntryCountExceeded)
    });

    let zero_count_limits = ArchiveLimits {
        max_entries: 0,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&ratio_path, APP_VERSION, zero_count_limits)
        .expect_err("entry-count exhaustion must use its dedicated violation");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::EntryCountExceeded)
    });

    let entry_limits = ArchiveLimits {
        max_entry_bytes: 1_024,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&digest_path, APP_VERSION, entry_limits)
        .expect_err("individual entry size must be bounded");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::EntrySizeExceeded(_))
    });

    let expanded_limits = ArchiveLimits {
        max_expanded_bytes: 100,
        ..ArchiveLimits::default()
    };
    let error = validate_archive(&ratio_path, APP_VERSION, expanded_limits)
        .expect_err("expanded size must be bounded");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::ExpandedSizeExceeded)
    });

    let version_path = temp.path().join("version.zip");
    let mut future_manifest = manifest_with(&[]);
    future_manifest.minimum_app_version = "999.0.0".into();
    write_archive(&version_path, &future_manifest, &[], false);
    let error = validate_archive(&version_path, APP_VERSION, ArchiveLimits::default())
        .expect_err("future minimum version must fail");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::MinimumVersionNotMet(_))
    });

    let encrypted_path = temp.path().join("encrypted.zip");
    write_archive(&encrypted_path, &manifest_with(&[]), &[], false);
    mark_first_entry_encrypted(&encrypted_path);
    let error = validate_archive(&encrypted_path, APP_VERSION, ArchiveLimits::default())
        .expect_err("encrypted entry must fail before reading");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::EncryptedEntry(_))
    });
}

#[test]
fn project_trust_is_runtime_only_and_archive_claims_fail_closed() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let mut manifest = WorkspaceManifest::new(APP_VERSION);
    let reference = ProjectReference::from_folder(&project, "Project").expect("reference");
    let project_id = reference.project_id;
    manifest.add_project(reference).expect("add Project");
    manifest
        .resolve_project_trust(project_id, true)
        .expect("resolve local picker grant");
    assert_eq!(
        manifest.project_status(project_id).expect("local status"),
        ProjectPathStatus::Trusted
    );

    let portable = toml::to_string(&manifest).expect("portable manifest");
    assert!(!portable.contains("trusted_root"));
    let malicious = format!("{portable}trusted_root = true\n");
    let archive = temp.path().join("claimed-trust.zip");
    write_archive_with_manifest_text(&archive, &malicious, &[], false);
    let error = validate_archive(&archive, APP_VERSION, ArchiveLimits::default())
        .expect_err("portable trusted-root claims must be rejected as unknown fields");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::InvalidManifest(_))
    });
}

#[test]
fn minimum_application_version_uses_full_semver_prerelease_precedence() {
    let temp = TempDir::new().expect("tempdir");
    let cases = [
        ("1.0.0-rc.1", "1.0.0", true),
        ("1.0.0-rc.1", "1.0.0-rc.2", true),
        ("1.0.0-rc.2", "1.0.0-rc.1", false),
        ("1.0.0-beta.2", "1.0.0-beta.11", true),
        ("1.0.0-beta.11", "1.0.0-beta.2", false),
        ("1.0.0", "1.0.0-alpha", false),
        ("1.0.0+archive", "1.0.0+local", true),
        ("1.0.0-alpha.1", "1.0.0-alpha.beta", true),
    ];
    for (minimum, current, should_open) in cases {
        let path = temp.path().join(format!("{}.zip", Uuid::new_v4()));
        let mut manifest = manifest_with(&[]);
        manifest.minimum_app_version = minimum.into();
        write_archive(&path, &manifest, &[], false);
        let result = validate_archive(&path, current, ArchiveLimits::default());
        if should_open {
            result.expect("current SemVer satisfies minimum");
        } else {
            assert_archive_violation(
                result.expect_err("minimum must reject current"),
                |violation| matches!(violation, ArchiveViolation::MinimumVersionNotMet(_)),
            );
        }
    }

    let invalid = temp.path().join("invalid-semver.zip");
    let mut manifest = manifest_with(&[]);
    manifest.minimum_app_version = "01.0.0".into();
    write_archive(&invalid, &manifest, &[], false);
    assert_archive_violation(
        validate_archive(&invalid, "1.0.0", ArchiveLimits::default())
            .expect_err("leading-zero SemVer core must fail"),
        |violation| matches!(violation, ArchiveViolation::InvalidManifest(_)),
    );
}

#[test]
fn content_addressed_blob_path_matches_manifest_and_actual_sha256() {
    let temp = TempDir::new().expect("tempdir");
    let content = b"content-addressed Workspace blob".to_vec();
    let actual_digest = digest(&content);
    let valid_entry = FixtureEntry::file(format!("blobs/sha256/{actual_digest}"), content.clone());
    let valid_archive = temp.path().join("valid-blob.zip");
    write_archive(
        &valid_archive,
        &manifest_with(std::slice::from_ref(&valid_entry)),
        std::slice::from_ref(&valid_entry),
        false,
    );
    validate_archive(&valid_archive, APP_VERSION, ArchiveLimits::default())
        .expect("self-addressed blob validates");

    let false_digest = "0".repeat(64);
    assert_ne!(false_digest, actual_digest);
    let mismatched_entry =
        FixtureEntry::file(format!("blobs/sha256/{false_digest}"), content.clone());
    let mismatched_archive = temp.path().join("mismatched-blob.zip");
    write_archive(
        &mismatched_archive,
        &manifest_with(std::slice::from_ref(&mismatched_entry)),
        std::slice::from_ref(&mismatched_entry),
        false,
    );
    assert_archive_violation(
        validate_archive(&mismatched_archive, APP_VERSION, ArchiveLimits::default())
            .expect_err("manifest digest cannot disagree with blob path"),
        |violation| matches!(violation, ArchiveViolation::BlobDigestMismatch(_)),
    );

    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let blob_path = WorkspaceLayout::new(&working).blob(&false_digest);
    fs::write(&blob_path, content).expect("mislabeled working blob");
    let output = temp.path().join("must-not-save.zip");
    assert_archive_violation(
        save_workspace_archive_by_copy(
            &working,
            &output,
            &manifest,
            APP_VERSION,
            ArchiveLimits::default(),
        )
        .expect_err("save must reject mislabeled blob"),
        |violation| matches!(violation, ArchiveViolation::BlobDigestMismatch(_)),
    );
    assert!(!output.exists());
}

fn mark_first_entry_encrypted(path: &Path) {
    let mut bytes = fs::read(path).expect("read zip");
    let mut local_done = false;
    let mut central_done = false;
    for index in 0..bytes.len().saturating_sub(10) {
        if !local_done && bytes[index..].starts_with(b"PK\x03\x04") {
            bytes[index + 6] |= 1;
            local_done = true;
        }
        if !central_done && bytes[index..].starts_with(b"PK\x01\x02") {
            bytes[index + 8] |= 1;
            central_done = true;
        }
        if local_done && central_done {
            break;
        }
    }
    assert!(local_done && central_done);
    fs::write(path, bytes).expect("write modified zip");
}

fn pseudo_random_bytes(length: usize) -> Vec<u8> {
    let mut state = 0x9e37_79b9_u32;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}

#[test]
fn advisory_lock_reports_owner_and_never_silently_grants_a_second_writer() {
    let temp = TempDir::new().expect("tempdir");
    let lock_path = temp.path().join("active.lock");
    let first_owner = lock_owner("first");
    let first = acquire_workspace_writer_lock(&lock_path, first_owner.clone())
        .expect("first lock succeeds");
    let WriterAccess::Writable(first_guard) = first else {
        panic!("first lock should be writable");
    };

    let second = acquire_workspace_writer_lock(&lock_path, lock_owner("second"))
        .expect("contention is a read-only outcome");
    match second {
        WriterAccess::ReadOnly { owner } => assert_eq!(owner, Some(first_owner)),
        WriterAccess::Writable(_) => panic!("second writer must never be granted"),
    }

    drop(first_guard);
    assert!(matches!(
        acquire_workspace_writer_lock(&lock_path, lock_owner("third"))
            .expect("lock released on drop"),
        WriterAccess::Writable(_)
    ));
}

#[test]
fn read_only_open_rejects_semantically_hostile_archive_without_mutating_active_state() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    fs::write(
        WorkspaceLayout::new(&working).workspace_configuration(),
        b"structurally-valid-but-semantically-invalid = true\n",
    )
    .expect("hostile config");
    let archive = temp.path().join("hostile-read-only.zip");
    save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("structurally valid archive");

    let home = C4osHomeLayout::new(temp.path().join("home"));
    home.ensure_roots().expect("home roots");
    fs::create_dir_all(home.active_workspace().join("state")).expect("active state");
    fs::write(
        home.active_workspace().join("state/preserve-me"),
        b"writer-owned active state",
    )
    .expect("active marker");
    let active_before = directory_snapshot(&home.active_workspace());
    let archive_before = fs::read(&archive).expect("archive bytes");

    let blocking_owner = lock_owner("blocking-writer");
    let blocking_access = acquire_workspace_writer_lock(&home.workspace_lock(), blocking_owner)
        .expect("blocking lock");
    let WriterAccess::Writable(_blocking_lock) = blocking_access else {
        panic!("blocking writer should own the lock");
    };

    let semantic_checks = Cell::new(0);
    let extracted_root = RefCell::new(None);
    let error = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("read-only-opener"),
        |root, candidate, target| {
            semantic_checks.set(semantic_checks.get() + 1);
            extracted_root.replace(Some(root.to_path_buf()));
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            assert_ne!(root, home.active_workspace());
            assert_eq!(candidate.workspace_id, manifest.workspace_id);
            assert_eq!(
                fs::read(root.join("config/workspace.toml")).expect("extracted config"),
                b"structurally-valid-but-semantically-invalid = true\n"
            );
            Err(WorkspaceError::Conflict(
                "read-only semantic Workspace validation failed".into(),
            ))
        },
    )
    .expect_err("lock contention must not bypass semantic validation");

    assert!(matches!(error, WorkspaceError::Conflict(_)));
    assert_eq!(semantic_checks.get(), 1);
    let extracted_root = extracted_root
        .into_inner()
        .expect("semantic callback received extraction root");
    assert!(
        !extracted_root.exists(),
        "read-only extraction must be discarded"
    );
    assert_eq!(directory_snapshot(&home.active_workspace()), active_before);
    assert!(
        fs::read_dir(home.workspace_recovery_root())
            .expect("recovery root")
            .next()
            .is_none()
    );
    assert_eq!(
        fs::read(&archive).expect("preserved archive"),
        archive_before
    );
}

#[test]
fn workspace_roundtrip_uses_authoritative_copy_and_atomic_archive() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("external-project");
    fs::create_dir(&project).expect("project folder");
    fs::write(project.join("README.md"), b"external source").expect("project file");
    let project_before = directory_snapshot(&project);

    let (working, manifest) = create_working_copy(temp.path(), &project, "External Project");
    let layout = WorkspaceLayout::new(&working);
    fs::write(layout.database(), b"consistent sqlite backup").expect("db fixture");
    fs::write(layout.workspace_configuration(), b"schema_version = 1\n").expect("workspace config");
    let project_id = manifest.projects[0].project_id;
    fs::create_dir_all(
        layout
            .project_configuration(project_id)
            .parent()
            .expect("parent"),
    )
    .expect("project config parent");
    fs::write(
        layout.project_configuration(project_id),
        b"schema_version = 1\n",
    )
    .expect("project config");

    let archive = temp.path().join("portable-workspace.zip");
    let saved = save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("atomic save");
    let validated = validate_archive(&archive, APP_VERSION, ArchiveLimits::default())
        .expect("saved archive validates");
    assert_eq!(validated.manifest.workspace_id, saved.manifest.workspace_id);
    assert_eq!(validated.manifest.generation, saved.manifest.generation);
    assert_eq!(validated.manifest.files, saved.manifest.files);
    assert_eq!(validated.manifest.projects.len(), 1);
    assert_eq!(
        saved
            .manifest
            .project_status(project_id)
            .expect("local picker grant"),
        ProjectPathStatus::Trusted
    );
    assert_eq!(
        validated
            .manifest
            .project_status(project_id)
            .expect("portable archive trust"),
        ProjectPathStatus::NeedsTrust
    );
    assert!(
        validated
            .manifest
            .files
            .iter()
            .any(|file| file.path == "state/workspace.sqlite3")
    );

    let home = C4osHomeLayout::new(temp.path().join("home"));
    let opened = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("roundtrip"),
        |_root, manifest, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            Ok(manifest.clone())
        },
    )
    .expect("validate, extract, and atomically promote");
    let OpenWorkspaceOutcome::Writable(opened) = opened else {
        panic!("roundtrip should hold the writer lock");
    };
    let recent = opened.recent_handoff();
    assert_eq!(opened.manifest.workspace_id, manifest.workspace_id);
    assert_eq!(
        opened
            .manifest
            .project_status(project_id)
            .expect("opened archive trust"),
        ProjectPathStatus::NeedsTrust
    );
    assert_eq!(recent.workspace_id, manifest.workspace_id);
    assert_eq!(recent.archive_path, Some(archive.clone()));
    assert_eq!(recent.project_count, 1);
    assert_eq!(
        fs::read(opened.working_root.join("state/workspace.sqlite3")).expect("opened db"),
        b"consistent sqlite backup"
    );
    assert_eq!(directory_snapshot(&project), project_before);
    assert!(!project.join(".c4os").exists());
}

#[test]
fn live_sqlite_sidecars_are_ignored_and_online_backup_is_the_only_portable_database() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let layout = WorkspaceLayout::new(&working);
    let source = Connection::open(layout.database()).expect("open live Workspace database");
    source
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA wal_autocheckpoint = 0;
             CREATE TABLE durable_state (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO durable_state (key, value) VALUES ('generation', '7');",
        )
        .expect("create uncheckpointed WAL state");
    assert!(layout.database().with_extension("sqlite3-wal").is_file());
    assert!(layout.database().with_extension("sqlite3-shm").is_file());

    let archive = temp.path().join("live-database.zip");
    let saved = save_workspace_archive(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |_source_path, destination| {
            let mut backup_connection = Connection::open(destination)
                .map_err(|error| WorkspaceError::Conflict(error.to_string()))?;
            let backup = Backup::new(&source, &mut backup_connection)
                .map_err(|error| WorkspaceError::Conflict(error.to_string()))?;
            backup
                .run_to_completion(64, Duration::from_millis(1), None)
                .map_err(|error| WorkspaceError::Conflict(error.to_string()))?;
            Ok(())
        },
    )
    .expect("save through online backup while WAL sidecars remain open");

    assert!(
        saved
            .manifest
            .files
            .iter()
            .any(|file| file.path == "state/workspace.sqlite3")
    );
    assert!(saved.manifest.files.iter().all(|file| {
        !file.path.ends_with("-wal")
            && !file.path.ends_with("-shm")
            && !file.path.ends_with("-journal")
    }));
    let mut zip = ZipArchive::new(File::open(&archive).expect("archive")).expect("zip");
    let names: Vec<_> = (0..zip.len())
        .map(|index| zip.by_index(index).expect("entry").name().to_string())
        .collect();
    assert!(names.iter().any(|name| name == "state/workspace.sqlite3"));
    assert!(names.iter().all(|name| {
        !name.ends_with("-wal") && !name.ends_with("-shm") && !name.ends_with("-journal")
    }));
}

#[test]
fn save_staging_enforces_local_limits_before_database_backup_or_archive_replacement() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let layout = WorkspaceLayout::new(&working);
    fs::write(layout.database(), b"db").expect("database fixture");
    fs::write(layout.workspace_configuration(), pseudo_random_bytes(256))
        .expect("configuration fixture");
    let archive = temp.path().join("bounded-staging.zip");
    save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("baseline archive");
    let archive_before = fs::read(&archive).expect("baseline bytes");

    let hostile_cache = working.join(format!("chats/{}/cache/ratio.bin", Uuid::new_v4()));
    fs::create_dir_all(hostile_cache.parent().expect("cache parent")).expect("cache root");
    fs::write(&hostile_cache, vec![0_u8; 64 * 1024]).expect("compressible hostile file");

    let cases = [
        (
            ArchiveLimits {
                max_entries: 2,
                ..ArchiveLimits::default()
            },
            "entry count",
            0_u8,
        ),
        (
            ArchiveLimits {
                max_entry_bytes: 1_024,
                ..ArchiveLimits::default()
            },
            "entry size",
            1_u8,
        ),
        (
            ArchiveLimits {
                max_expanded_bytes: 1_024,
                ..ArchiveLimits::default()
            },
            "expanded size",
            2_u8,
        ),
        (
            ArchiveLimits {
                max_compression_ratio: 10,
                ..ArchiveLimits::default()
            },
            "compression ratio",
            3_u8,
        ),
    ];

    for (limits, label, expected) in cases {
        let source_before = directory_snapshot(&working);
        let preflight_error =
            preflight_workspace_archive_source(&working, &manifest, APP_VERSION, limits)
                .expect_err(label);
        assert!(
            matches!(preflight_error, WorkspaceError::Archive(_)),
            "{label} is an archive-source violation"
        );
        assert_eq!(
            directory_snapshot(&working),
            source_before,
            "{label} preflight must be side-effect-free"
        );

        let database_backups = Cell::new(0);
        let error = save_workspace_archive(
            &working,
            &archive,
            &manifest,
            APP_VERSION,
            limits,
            |source, destination| {
                database_backups.set(database_backups.get() + 1);
                fs::copy(source, destination)?;
                Ok(())
            },
        )
        .expect_err(label);
        assert_eq!(
            database_backups.get(),
            0,
            "{label} must fail before database backup or staging"
        );
        let WorkspaceError::Archive(violation) = error else {
            panic!("unexpected {label} error");
        };
        assert!(match expected {
            0 => matches!(violation, ArchiveViolation::EntryCountExceeded),
            1 => matches!(violation, ArchiveViolation::EntrySizeExceeded(_)),
            2 => matches!(violation, ArchiveViolation::ExpandedSizeExceeded),
            3 => matches!(violation, ArchiveViolation::CompressionRatioExceeded(_)),
            _ => false,
        });
        assert_eq!(
            fs::read(&archive).expect("preserved archive"),
            archive_before,
            "{label} cannot replace the last validated archive"
        );
    }

    assert!(
        fs::read_dir(temp.path())
            .expect("temporary root")
            .filter_map(Result::ok)
            .all(|entry| {
                !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".c4os-snapshot-")
            }),
        "bounded staging must clean every temporary tree"
    );
}

#[cfg(unix)]
#[test]
fn source_preflight_rejects_forbidden_roots_and_types_before_database_backup() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let layout = WorkspaceLayout::new(&working);
    fs::write(layout.database(), b"database").expect("database fixture");
    fs::write(layout.workspace_configuration(), b"schema_version = 1\n").expect("config");
    let archive_parent = temp.path().join("must-not-exist-before-preflight");
    let archive = archive_parent.join("source-preflight.zip");

    fs::create_dir_all(working.join("vault")).expect("forbidden root");
    fs::write(working.join("vault/credentials.vault"), b"secret").expect("forbidden file");
    let source_before = directory_snapshot(&working);
    let backups = Cell::new(0);
    let error = save_workspace_archive(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |source, destination| {
            backups.set(backups.get() + 1);
            fs::copy(source, destination)?;
            Ok(())
        },
    )
    .expect_err("forbidden root must fail preflight");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::DisallowedRoot(_))
    });
    assert_eq!(backups.get(), 0);
    assert_eq!(directory_snapshot(&working), source_before);
    assert!(!archive.exists());
    assert!(
        !archive_parent.exists(),
        "failed source preflight cannot create archive staging state"
    );

    fs::remove_dir_all(working.join("vault")).expect("remove forbidden fixture");
    let link = working.join(format!("chats/{}/cache/link", Uuid::new_v4()));
    fs::create_dir_all(link.parent().expect("link parent")).expect("link root");
    symlink(layout.workspace_configuration(), &link).expect("hostile symlink");
    let source_before = directory_snapshot(&working);
    let backups = Cell::new(0);
    let error = save_workspace_archive(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |source, destination| {
            backups.set(backups.get() + 1);
            fs::copy(source, destination)?;
            Ok(())
        },
    )
    .expect_err("unsupported type must fail preflight");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::UnsupportedEntryType(_))
    });
    assert_eq!(backups.get(), 0);
    assert_eq!(directory_snapshot(&working), source_before);
    assert!(!archive.exists());
    assert!(!archive_parent.exists());

    fs::remove_file(link).expect("remove symlink fixture");
    fs::write(
        format!("{}-wal", layout.database().display()),
        b"excluded WAL",
    )
    .expect("WAL fixture");
    fs::write(working.join(".manifest.interrupted.tmp"), b"excluded temp")
        .expect("manifest temp fixture");
    let source_before = directory_snapshot(&working);
    let preflight = preflight_workspace_archive_source(
        &working,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("sanctioned transient files are excluded");
    assert_eq!(preflight.entry_count, 3, "manifest, config, and database");
    assert_eq!(directory_snapshot(&working), source_before);

    let backups = Cell::new(0);
    save_workspace_archive(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |source, destination| {
            backups.set(backups.get() + 1);
            fs::copy(source, destination)?;
            Ok(())
        },
    )
    .expect("valid preflight proceeds to one database backup");
    assert_eq!(backups.get(), 1);
}

#[test]
fn pending_save_restores_prior_archive_on_abort_and_releases_it_on_commit() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let layout = WorkspaceLayout::new(&working);
    fs::write(layout.database(), b"database").expect("database fixture");
    fs::write(layout.workspace_configuration(), b"schema_version = 1\n").expect("first config");
    let archive = temp.path().join("transactional-save.zip");
    save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("initial archive");
    let prior_archive = fs::read(&archive).expect("prior archive");

    fs::write(
        layout.workspace_configuration(),
        b"schema_version = 1\nrevision = 2\n",
    )
    .expect("second config");
    let writer_lock = test_writer_lock(&working);
    let pending = prepare_workspace_archive_save(
        &writer_lock,
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |source, destination| {
            fs::copy(source, destination)?;
            File::open(destination)?.sync_all()?;
            Ok(())
        },
    )
    .expect("prepared replacement");
    assert_ne!(fs::read(&archive).expect("prepared archive"), prior_archive);
    drop(pending);
    assert_eq!(
        fs::read(&archive).expect("restored prior archive"),
        prior_archive
    );

    let pending = prepare_workspace_archive_save(
        &writer_lock,
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
        |source, destination| {
            fs::copy(source, destination)?;
            File::open(destination)?.sync_all()?;
            Ok(())
        },
    )
    .expect("prepared committed replacement");
    let expected = pending.saved().manifest.clone();
    let committed = pending.commit();
    assert_eq!(committed.manifest, expected);
    assert_ne!(
        fs::read(&archive).expect("committed archive"),
        prior_archive
    );
    let validated = validate_archive(&archive, APP_VERSION, ArchiveLimits::default())
        .expect("committed archive validates")
        .manifest;
    assert_eq!(validated.workspace_id, committed.manifest.workspace_id);
    assert_eq!(validated.generation, committed.manifest.generation);
    assert_eq!(validated.files, committed.manifest.files);
    assert!(
        fs::read_dir(temp.path())
            .expect("temporary root")
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().ends_with(".previous")),
        "commit removes the retained prior archive link"
    );
}

#[test]
fn pending_open_restores_prior_active_copy_until_explicit_commit() {
    let temp = TempDir::new().expect("tempdir");
    let first_project = temp.path().join("first-project");
    let second_project = temp.path().join("second-project");
    fs::create_dir(&first_project).expect("first project");
    fs::create_dir(&second_project).expect("second project");
    let (first_root, first_manifest) =
        create_working_copy(temp.path(), &first_project, "First Project");
    let (second_root, second_manifest) =
        create_working_copy(temp.path(), &second_project, "Second Project");
    fs::write(
        WorkspaceLayout::new(&first_root).database(),
        b"first database",
    )
    .expect("first database");
    fs::write(
        WorkspaceLayout::new(&second_root).database(),
        b"second database",
    )
    .expect("second database");
    let first_archive = temp.path().join("first-pending-open.zip");
    let second_archive = temp.path().join("second-pending-open.zip");
    save_workspace_archive_by_copy(
        &first_root,
        &first_archive,
        &first_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("first archive");
    save_workspace_archive_by_copy(
        &second_root,
        &second_archive,
        &second_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("second archive");

    let home = C4osHomeLayout::new(temp.path().join("home"));
    let first = open_workspace_archive(
        &home,
        &first_archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("first-open"),
        |_root, candidate, _target| Ok(candidate.clone()),
    )
    .expect("first open");
    let OpenWorkspaceOutcome::Writable(first) = first else {
        panic!("first open is writable");
    };
    drop(first);
    let prior_active = directory_snapshot(&home.active_workspace());

    let prepared = prepare_open_workspace_archive(
        &home,
        &second_archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("aborted-open"),
        |_root, candidate, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            Ok(candidate.clone())
        },
    )
    .expect("prepared second open");
    let c4os_lib::core::workspace::PreparedOpenWorkspaceOutcome::Writable(pending) = prepared
    else {
        panic!("prepared open is writable");
    };
    assert_eq!(
        pending.workspace().manifest.workspace_id,
        second_manifest.workspace_id
    );
    assert_ne!(directory_snapshot(&home.active_workspace()), prior_active);
    drop(pending);
    assert_eq!(directory_snapshot(&home.active_workspace()), prior_active);
    assert!(
        fs::read_dir(home.workspace_recovery_root())
            .expect("recovery root")
            .next()
            .is_none(),
        "aborted open restores, rather than retaining, the prior active copy"
    );

    let prepared = prepare_open_workspace_archive(
        &home,
        &second_archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("committed-open"),
        |_root, candidate, _target| Ok(candidate.clone()),
    )
    .expect("prepared committed open");
    let c4os_lib::core::workspace::PreparedOpenWorkspaceOutcome::Writable(pending) = prepared
    else {
        panic!("prepared open is writable");
    };
    let committed = pending.commit();
    assert_eq!(
        committed.manifest.workspace_id,
        second_manifest.workspace_id
    );
    assert_ne!(directory_snapshot(&home.active_workspace()), prior_active);
    assert_eq!(
        fs::read_dir(home.workspace_recovery_root())
            .expect("committed recovery root")
            .count(),
        1,
        "commit retains the prior active copy as recovery evidence"
    );
}

#[test]
fn semantic_validation_failure_preserves_prior_active_copy_and_both_archives() {
    let temp = TempDir::new().expect("tempdir");
    let first_project = temp.path().join("first-project");
    let second_project = temp.path().join("second-project");
    fs::create_dir(&first_project).expect("first project");
    fs::create_dir(&second_project).expect("second project");

    let (first_root, first_manifest) =
        create_working_copy(temp.path(), &first_project, "First Project");
    fs::write(
        WorkspaceLayout::new(&first_root).workspace_configuration(),
        b"schema_version = 1\n",
    )
    .expect("first config");
    let first_archive = temp.path().join("first.zip");
    save_workspace_archive_by_copy(
        &first_root,
        &first_archive,
        &first_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("first save");

    let home = C4osHomeLayout::new(temp.path().join("home"));
    let first_open = open_workspace_archive(
        &home,
        &first_archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("first-semantic-open"),
        |root, manifest, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            assert!(root.join("manifest.toml").is_file());
            assert_eq!(manifest.workspace_id, first_manifest.workspace_id);
            Ok(manifest.clone())
        },
    )
    .expect("first semantic validation");
    let OpenWorkspaceOutcome::Writable(first_open) = first_open else {
        panic!("first open should be writable");
    };
    drop(first_open);

    let (second_root, second_manifest) =
        create_working_copy(temp.path(), &second_project, "Second Project");
    fs::write(
        WorkspaceLayout::new(&second_root).workspace_configuration(),
        b"structurally-valid-but-semantically-invalid = true\n",
    )
    .expect("second config");
    let second_archive = temp.path().join("second.zip");
    save_workspace_archive_by_copy(
        &second_root,
        &second_archive,
        &second_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("second structural save");

    let active_before = directory_snapshot(&home.active_workspace());
    let first_archive_before = fs::read(&first_archive).expect("first archive bytes");
    let second_archive_before = fs::read(&second_archive).expect("second archive bytes");
    let semantic_checks = Cell::new(0);
    let error = open_workspace_archive(
        &home,
        &second_archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("rejected-semantic-open"),
        |root, manifest, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            semantic_checks.set(semantic_checks.get() + 1);
            assert_ne!(root, home.active_workspace());
            assert_eq!(manifest.workspace_id, second_manifest.workspace_id);
            assert!(root.join("config/workspace.toml").is_file());
            Err(WorkspaceError::Conflict(
                "semantic Workspace validation failed".into(),
            ))
        },
    )
    .expect_err("semantic failure must stop before promotion");
    assert!(matches!(error, WorkspaceError::Conflict(_)));
    assert_eq!(semantic_checks.get(), 1);
    assert_eq!(directory_snapshot(&home.active_workspace()), active_before);
    assert_eq!(
        fs::read(&first_archive).expect("first preserved"),
        first_archive_before
    );
    assert_eq!(
        fs::read(&second_archive).expect("second preserved"),
        second_archive_before
    );
    assert!(
        fs::read_dir(home.workspace_recovery_root())
            .expect("recovery root")
            .next()
            .is_none(),
        "active state must not move to recovery before semantic validation"
    );
}

#[test]
fn corrupt_active_copy_is_quarantined_only_after_candidate_semantics_pass() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    fs::write(
        WorkspaceLayout::new(&working).workspace_configuration(),
        b"schema_version = 1\n",
    )
    .expect("candidate config");
    let archive = temp.path().join("candidate.zip");
    save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("candidate archive");

    let home = C4osHomeLayout::new(temp.path().join("home"));
    home.ensure_roots().expect("home roots");
    fs::create_dir_all(home.active_workspace().join("state")).expect("corrupt active root");
    fs::write(
        home.active_workspace().join("manifest.toml"),
        b"this is not a Workspace manifest",
    )
    .expect("corrupt manifest");
    fs::write(
        home.active_workspace().join("state/preserve-me"),
        b"corrupt active evidence",
    )
    .expect("corrupt evidence");
    let corrupt_before = directory_snapshot(&home.active_workspace());
    let archive_before = fs::read(&archive).expect("candidate bytes");

    let rejected = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("reject-corrupt-replacement"),
        |_root, _candidate, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            Err(WorkspaceError::Conflict(
                "candidate semantic validation failed".into(),
            ))
        },
    )
    .expect_err("failed candidate cannot move corrupt active state");
    assert!(matches!(rejected, WorkspaceError::Conflict(_)));
    assert_eq!(directory_snapshot(&home.active_workspace()), corrupt_before);
    assert!(
        fs::read_dir(home.workspace_recovery_root())
            .expect("recovery root")
            .next()
            .is_none()
    );
    assert_eq!(
        fs::read(&archive).expect("preserved archive"),
        archive_before
    );

    let opened = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("accept-corrupt-replacement"),
        |_root, candidate, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            Ok(candidate.clone())
        },
    )
    .expect("valid candidate replaces corrupt active state");
    let OpenWorkspaceOutcome::Writable(opened) = opened else {
        panic!("candidate should be writable");
    };
    assert_eq!(opened.manifest.workspace_id, manifest.workspace_id);
    let recovery_entries: Vec<_> = fs::read_dir(home.workspace_recovery_root())
        .expect("recovery entries")
        .map(|entry| entry.expect("recovery entry").path())
        .collect();
    assert_eq!(recovery_entries.len(), 1);
    assert!(
        recovery_entries[0]
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("corrupt-active-"))
    );
    assert_eq!(
        directory_snapshot(&recovery_entries[0]),
        corrupt_before,
        "corrupt state is preserved without being trusted"
    );
    assert_eq!(
        fs::read(&archive).expect("archive unchanged"),
        archive_before
    );
}

#[test]
fn newer_durable_working_generation_is_recovered_before_archive_save() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    let layout = WorkspaceLayout::new(&working);
    fs::write(layout.workspace_configuration(), b"schema_version = 1\n").expect("config");
    let archive = temp.path().join("workspace.zip");
    let saved = save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("save");

    let home = C4osHomeLayout::new(temp.path().join("home"));
    let first_open = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("first-open"),
        |_root, manifest, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ArchiveCandidate);
            Ok(manifest.clone())
        },
    )
    .expect("first open");
    let OpenWorkspaceOutcome::Writable(first_open) = first_open else {
        panic!("first open should be writable");
    };
    let mut working_manifest = first_open.manifest.clone();
    fs::write(
        first_open.working_root.join("config/workspace.toml"),
        b"schema_version = 1\nrestore_last_workspace = true\n",
    )
    .expect("committed state change");
    persist_committed_generation(&first_open.working_root, &mut working_manifest)
        .expect("durable generation update");
    drop(first_open);

    // Simulate interruption after the database-derived generation advanced
    // but before the portable manifest generation was refreshed.
    let active_manifest_path = home.active_workspace().join("manifest.toml");
    let active_text = fs::read_to_string(&active_manifest_path).expect("active manifest");
    let stale_text = active_text.replace(
        &format!("generation = {}", working_manifest.generation),
        &format!("generation = {}", saved.manifest.generation),
    );
    assert_ne!(stale_text, active_text);
    fs::write(&active_manifest_path, stale_text).expect("stale manifest generation fixture");

    assert_eq!(
        detect_recovery(&working_manifest, &saved.manifest).expect("recovery compare"),
        RecoveryDisposition::RecoverWorkingCopy {
            working_generation: working_manifest.generation,
            archive_generation: saved.manifest.generation,
        }
    );
    let active_before = directory_snapshot(&home.active_workspace());
    let archive_before = fs::read(&archive).expect("archive before recovery validation");
    let rejected_checks = Cell::new(0);
    let error = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("rejected-recovery"),
        |root, recovered_manifest, target| {
            rejected_checks.set(rejected_checks.get() + 1);
            match target {
                WorkspaceSemanticValidationTarget::ActiveRecovery => {
                    assert_eq!(root, home.active_workspace());
                    assert_eq!(recovered_manifest.generation, saved.manifest.generation);
                }
                WorkspaceSemanticValidationTarget::ArchiveCandidate => {
                    assert_ne!(root, home.active_workspace());
                    assert_eq!(recovered_manifest.generation, saved.manifest.generation);
                }
            }
            Err(WorkspaceError::Conflict(
                "recovered Workspace failed semantic validation".into(),
            ))
        },
    )
    .expect_err("recovery semantic failure must fail closed");
    assert!(matches!(error, WorkspaceError::Conflict(_)));
    assert_eq!(rejected_checks.get(), 2);
    assert_eq!(directory_snapshot(&home.active_workspace()), active_before);
    assert_eq!(
        fs::read(&archive).expect("archive after rejected recovery"),
        archive_before
    );

    let accepted_checks = Cell::new(0);
    let reopened = open_workspace_archive(
        &home,
        &archive,
        APP_VERSION,
        ArchiveLimits::default(),
        lock_owner("reopen"),
        |root, recovered_manifest, target| {
            assert_eq!(target, WorkspaceSemanticValidationTarget::ActiveRecovery);
            accepted_checks.set(accepted_checks.get() + 1);
            assert_eq!(root, home.active_workspace());
            assert_eq!(recovered_manifest.generation, saved.manifest.generation);
            Ok(working_manifest.clone())
        },
    )
    .expect("automatic recovery open");
    let OpenWorkspaceOutcome::Writable(reopened) = reopened else {
        panic!("recovery should reacquire writer lock");
    };
    let notice = reopened.recovery_notice.expect("recovery notice");
    assert_eq!(accepted_checks.get(), 1);
    assert_eq!(notice.working_generation, working_manifest.generation);
    assert_eq!(notice.archive_generation, saved.manifest.generation);
    assert!(notice.must_notify_before_next_save);
    assert_eq!(
        c4os_lib::core::workspace::validate_working_copy(
            &home.active_workspace(),
            APP_VERSION,
            ArchiveLimits::default(),
        )
        .expect("canonical manifest persisted without generation advance")
        .generation,
        working_manifest.generation
    );
}

#[test]
fn two_workspaces_keep_distinct_metadata_for_the_same_project_folder() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("shared-project");
    fs::create_dir(&project).expect("project");
    fs::write(project.join("source.rs"), b"fn shared() {}").expect("source");
    let before = directory_snapshot(&project);

    let (first_root, first_manifest) = create_working_copy(temp.path(), &project, "First Name");
    let (second_root, second_manifest) = create_working_copy(temp.path(), &project, "Second Name");
    fs::write(
        WorkspaceLayout::new(&first_root).workspace_configuration(),
        b"schema_version = 1\napproval_preset = \"balanced\"\n",
    )
    .expect("first config");
    fs::write(
        WorkspaceLayout::new(&second_root).workspace_configuration(),
        b"schema_version = 1\napproval_preset = \"strict\"\n",
    )
    .expect("second config");

    let first_archive = temp.path().join("first.zip");
    let second_archive = temp.path().join("second.zip");
    let first = save_workspace_archive_by_copy(
        &first_root,
        &first_archive,
        &first_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("first save");
    let second = save_workspace_archive_by_copy(
        &second_root,
        &second_archive,
        &second_manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("second save");

    assert_ne!(first.manifest.workspace_id, second.manifest.workspace_id);
    assert_eq!(
        first.manifest.projects[0].last_known_path,
        second.manifest.projects[0].last_known_path
    );
    assert_ne!(
        first.manifest.projects[0].display_name,
        second.manifest.projects[0].display_name
    );
    assert_eq!(directory_snapshot(&project), before);
    assert!(!project.join(".c4os").exists());
}

#[test]
fn missing_relocate_reorder_and_trust_states_are_explicit() {
    let temp = TempDir::new().expect("tempdir");
    let first_folder = temp.path().join("first");
    let second_folder = temp.path().join("second");
    let relocated = temp.path().join("relocated");
    fs::create_dir(&first_folder).expect("first");
    fs::create_dir(&second_folder).expect("second");
    fs::create_dir(&relocated).expect("relocated");

    let mut manifest = WorkspaceManifest::new(APP_VERSION);
    let first = ProjectReference::from_folder(&first_folder, "First").expect("first ref");
    let second = ProjectReference::from_folder(&second_folder, "Second").expect("second ref");
    let first_id = first.project_id;
    let second_id = second.project_id;
    manifest.add_project(first).expect("add first");
    manifest.add_project(second).expect("add second");
    manifest
        .resolve_project_trust(first_id, true)
        .expect("resolve local picker grant");
    assert_eq!(
        manifest.project_status(first_id).expect("first status"),
        ProjectPathStatus::Trusted
    );
    assert_eq!(
        manifest.project_status(second_id).expect("second status"),
        ProjectPathStatus::NeedsTrust
    );

    fs::remove_dir_all(&first_folder).expect("remove external fixture");
    assert_eq!(
        manifest.project_status(first_id).expect("missing status"),
        ProjectPathStatus::Missing
    );
    manifest
        .relocate_project(first_id, &relocated)
        .expect("relocate and clear prior trust");
    assert_eq!(
        manifest
            .project_status(first_id)
            .expect("relocated unresolved status"),
        ProjectPathStatus::NeedsTrust
    );
    manifest
        .resolve_project_trust(first_id, true)
        .expect("resolve relocated local grant separately");
    manifest
        .reorder_projects(&[second_id, first_id])
        .expect("reorder");
    assert_eq!(manifest.projects[0].project_id, second_id);
    assert_eq!(manifest.projects[1].project_id, first_id);
    assert_eq!(
        manifest.project_status(first_id).expect("relocated status"),
        ProjectPathStatus::Trusted
    );
}

#[test]
fn failed_save_preserves_previous_archive_and_excluded_roots_never_enter_it() {
    let temp = TempDir::new().expect("tempdir");
    let project = temp.path().join("project");
    fs::create_dir(&project).expect("project");
    let (working, manifest) = create_working_copy(temp.path(), &project, "Project");
    fs::write(
        WorkspaceLayout::new(&working).workspace_configuration(),
        b"schema_version = 1\n",
    )
    .expect("config");
    let archive = temp.path().join("preserved.zip");
    save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect("initial save");
    let original = fs::read(&archive).expect("original archive bytes");

    fs::create_dir_all(working.join("vault")).expect("hostile excluded root");
    fs::write(working.join("vault/credentials.vault"), b"must-not-archive")
        .expect("excluded fixture");
    let error = save_workspace_archive_by_copy(
        &working,
        &archive,
        &manifest,
        APP_VERSION,
        ArchiveLimits::default(),
    )
    .expect_err("excluded authority must fail closed");
    assert_archive_violation(error, |violation| {
        matches!(violation, ArchiveViolation::DisallowedRoot(_))
    });
    assert_eq!(fs::read(&archive).expect("preserved archive"), original);

    let mut zip = ZipArchive::new(File::open(&archive).expect("archive")).expect("zip");
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).expect("entry");
        assert!(!entry.name().starts_with("vault/"));
        assert!(!entry.name().starts_with("browser/"));
        let mut sink = Vec::new();
        entry.read_to_end(&mut sink).expect("entry readable");
        assert!(
            !sink
                .windows(b"must-not-archive".len())
                .any(|window| window == b"must-not-archive")
        );
    }
}

fn directory_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(root: &Path, directory: &Path, output: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(directory).expect("read directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.push((
                    path.strip_prefix(root).expect("relative").to_path_buf(),
                    fs::read(&path).expect("read file"),
                ));
            }
        }
    }
    let mut output = Vec::new();
    visit(root, root, &mut output);
    output.sort_by(|left, right| left.0.cmp(&right.0));
    output
}
