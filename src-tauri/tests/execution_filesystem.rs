use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::Path;

use c4os_lib::execution::{
    ExpectedFileState, FolderEntryKind, ProjectFilesystem, ProjectFilesystemError,
    ProjectFilesystemLimits, TrustedProjectRoot,
};
use tempfile::TempDir;

fn bind(root: &Path) -> ProjectFilesystem {
    ProjectFilesystem::bind(TrustedProjectRoot::open(root).expect("trusted root"))
        .expect("filesystem binding")
}

fn temporary_names(root: &Path) -> Vec<String> {
    fs::read_dir(root)
        .expect("read root")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(".c4os-write-"))
        .collect()
}

#[test]
fn descriptor_rooted_read_rejects_final_and_ancestor_symlinks() {
    let project = TempDir::new().expect("project");
    let outside = TempDir::new().expect("outside");
    fs::write(outside.path().join("secret.txt"), "outside").expect("outside File");
    symlink(
        outside.path().join("secret.txt"),
        project.path().join("final-link"),
    )
    .expect("final symlink");
    symlink(outside.path(), project.path().join("ancestor-link")).expect("ancestor symlink");
    let filesystem = bind(project.path());

    assert!(matches!(
        filesystem.read_utf8(Path::new("final-link")),
        Err(ProjectFilesystemError::SymlinkRejected)
    ));
    assert!(matches!(
        filesystem.read_utf8(Path::new("ancestor-link/secret.txt")),
        Err(ProjectFilesystemError::SymlinkRejected)
    ));
}

#[test]
fn bound_root_rejects_path_substitution_instead_of_reading_replacement() {
    let parent = TempDir::new().expect("parent");
    let root = parent.path().join("project");
    let displaced = parent.path().join("displaced");
    fs::create_dir(&root).expect("root");
    fs::write(root.join("identity.txt"), "trusted").expect("trusted File");
    let filesystem = bind(&root);

    fs::rename(&root, &displaced).expect("displace root");
    fs::create_dir(&root).expect("replacement root");
    fs::write(root.join("identity.txt"), "replacement").expect("replacement File");

    assert!(matches!(
        filesystem.read_utf8(Path::new("identity.txt")),
        Err(ProjectFilesystemError::RootChanged)
    ));
    assert_eq!(
        fs::read_to_string(displaced.join("identity.txt")).expect("displaced content"),
        "trusted"
    );
}

#[test]
fn reads_are_bounded_and_require_utf8_regular_files() {
    let project = TempDir::new().expect("project");
    fs::write(project.path().join("large.txt"), "12345").expect("large File");
    fs::write(project.path().join("binary.txt"), [0xff, 0xfe]).expect("binary File");
    fs::create_dir(project.path().join("folder")).expect("folder");
    let limits = ProjectFilesystemLimits::new(4, 8, 128).expect("limits");
    let filesystem = ProjectFilesystem::bind_with_limits(
        TrustedProjectRoot::open(project.path()).expect("trusted root"),
        limits,
    )
    .expect("filesystem binding");

    assert!(matches!(
        filesystem.read_utf8(Path::new("large.txt")),
        Err(ProjectFilesystemError::FileLimitExceeded { max_bytes: 4 })
    ));
    assert!(matches!(
        filesystem.read_utf8(Path::new("binary.txt")),
        Err(ProjectFilesystemError::InvalidUtf8)
    ));
    assert!(matches!(
        filesystem.read_utf8(Path::new("folder")),
        Err(ProjectFilesystemError::NotRegularFile)
    ));
}

#[test]
fn folder_listing_is_sorted_bounded_non_recursive_and_does_not_follow_links() {
    let project = TempDir::new().expect("project");
    let outside = TempDir::new().expect("outside");
    fs::write(outside.path().join("secret.txt"), "secret").expect("secret");
    fs::create_dir(project.path().join("nested")).expect("nested");
    fs::write(project.path().join("nested/child.txt"), "child").expect("child");
    fs::write(project.path().join("z.txt"), "z").expect("z");
    fs::write(project.path().join("a.txt"), "a").expect("a");
    symlink(outside.path(), project.path().join("outside-link")).expect("link");
    let filesystem = bind(project.path());

    let first = filesystem.list_folder(Path::new("")).expect("listing");
    let second = filesystem
        .list_folder(Path::new(""))
        .expect("repeat listing");
    let names = first
        .entries()
        .iter()
        .map(|entry| entry.name())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["a.txt", "nested", "outside-link", "z.txt"]);
    assert_eq!(first.target_version(), second.target_version());
    assert_eq!(
        first.entries()[2].kind(),
        FolderEntryKind::Symlink,
        "the link itself is reported without inspecting its target"
    );
    assert!(matches!(
        filesystem.list_folder(Path::new("outside-link")),
        Err(ProjectFilesystemError::SymlinkRejected)
    ));
    assert!(!names.contains(&"child.txt"));
    assert!(!names.contains(&"secret.txt"));

    let limits = ProjectFilesystemLimits::new(32, 3, 128).expect("limits");
    let bounded = ProjectFilesystem::bind_with_limits(
        TrustedProjectRoot::open(project.path()).expect("trusted root"),
        limits,
    )
    .expect("bounded binding");
    assert!(matches!(
        bounded.list_folder(Path::new("")),
        Err(ProjectFilesystemError::FolderEntryLimitExceeded { max_entries: 3 })
    ));
}

#[test]
fn folder_listing_enforces_aggregate_name_bound() {
    let project = TempDir::new().expect("project");
    fs::write(project.path().join("alpha"), "a").expect("alpha");
    fs::write(project.path().join("beta"), "b").expect("beta");
    let limits = ProjectFilesystemLimits::new(32, 8, 8).expect("limits");
    let filesystem = ProjectFilesystem::bind_with_limits(
        TrustedProjectRoot::open(project.path()).expect("trusted root"),
        limits,
    )
    .expect("filesystem binding");

    assert!(matches!(
        filesystem.list_folder(Path::new("")),
        Err(ProjectFilesystemError::FolderNameLimitExceeded { max_bytes: 8 })
    ));
}

#[test]
fn atomic_replace_uses_expected_version_and_cleans_temporary_file() {
    let project = TempDir::new().expect("project");
    let target = project.path().join("document.txt");
    fs::write(&target, "before").expect("initial File");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o751))
        .expect("executable source mode");
    let filesystem = bind(project.path());
    let before = filesystem
        .read_utf8(Path::new("document.txt"))
        .expect("initial read");

    let outcome = filesystem
        .write_utf8(
            Path::new("document.txt"),
            "after",
            &ExpectedFileState::Existing(before.version().clone()),
        )
        .expect("atomic replace");

    assert!(!outcome.created());
    assert_eq!(fs::read_to_string(&target).expect("new content"), "after");
    assert_eq!(
        fs::metadata(&target)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777,
        0o751
    );
    assert_ne!(
        before.version().target_version(),
        outcome.version().target_version()
    );
    assert!(temporary_names(project.path()).is_empty());
}

#[test]
fn atomic_create_is_no_replace_and_cleans_temporary_file() {
    let project = TempDir::new().expect("project");
    let filesystem = bind(project.path());

    let outcome = filesystem
        .write_utf8(
            Path::new("created.txt"),
            "created",
            &ExpectedFileState::Absent,
        )
        .expect("atomic create");
    assert!(outcome.created());
    assert_eq!(
        fs::read_to_string(project.path().join("created.txt")).expect("created content"),
        "created"
    );
    assert_eq!(
        fs::metadata(project.path().join("created.txt"))
            .expect("created metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(matches!(
        filesystem.write_utf8(
            Path::new("created.txt"),
            "replacement",
            &ExpectedFileState::Absent,
        ),
        Err(ProjectFilesystemError::Conflict)
    ));
    assert_eq!(
        fs::read_to_string(project.path().join("created.txt")).expect("preserved content"),
        "created"
    );
    assert!(temporary_names(project.path()).is_empty());
}

#[test]
fn destination_identity_swap_conflicts_even_when_content_hash_matches() {
    let project = TempDir::new().expect("project");
    let target = project.path().join("document.txt");
    fs::write(&target, "same bytes").expect("initial File");
    let filesystem = bind(project.path());
    let expected = filesystem
        .read_utf8(Path::new("document.txt"))
        .expect("initial read")
        .version()
        .clone();

    fs::rename(&target, project.path().join("displaced.txt")).expect("destination swap");
    fs::write(&target, "same bytes").expect("replacement with same hash");
    let replacement_identity = fs::metadata(&target).expect("replacement metadata").ino();

    assert!(matches!(
        filesystem.write_utf8(
            Path::new("document.txt"),
            "intended write",
            &ExpectedFileState::Existing(expected),
        ),
        Err(ProjectFilesystemError::Conflict)
    ));
    assert_eq!(
        fs::read_to_string(&target).expect("replacement remains"),
        "same bytes"
    );
    assert_eq!(
        fs::metadata(&target).expect("identity remains").ino(),
        replacement_identity
    );
    assert!(temporary_names(project.path()).is_empty());
}

#[test]
fn write_rejects_symlink_destination_and_ancestor_without_touching_outside() {
    let project = TempDir::new().expect("project");
    let outside = TempDir::new().expect("outside");
    let outside_file = outside.path().join("outside.txt");
    fs::write(&outside_file, "outside").expect("outside File");
    symlink(&outside_file, project.path().join("destination-link")).expect("destination link");
    symlink(outside.path(), project.path().join("ancestor-link")).expect("ancestor link");
    let filesystem = bind(project.path());

    assert!(matches!(
        filesystem.write_utf8(
            Path::new("destination-link"),
            "changed",
            &ExpectedFileState::Absent,
        ),
        Err(ProjectFilesystemError::Conflict)
    ));
    assert!(matches!(
        filesystem.write_utf8(
            Path::new("ancestor-link/created.txt"),
            "changed",
            &ExpectedFileState::Absent,
        ),
        Err(ProjectFilesystemError::SymlinkRejected)
    ));
    assert_eq!(
        fs::read_to_string(&outside_file).expect("outside preserved"),
        "outside"
    );
    assert!(!outside.path().join("created.txt").exists());
    assert!(temporary_names(project.path()).is_empty());
}

#[test]
fn paths_must_be_normalized_and_project_relative() {
    let project = TempDir::new().expect("project");
    fs::write(project.path().join("file.txt"), "content").expect("File");
    let filesystem = bind(project.path());

    for path in [
        "../file.txt",
        "./file.txt",
        "nested/../file.txt",
        "/file.txt",
    ] {
        assert!(matches!(
            filesystem.read_utf8(Path::new(path)),
            Err(ProjectFilesystemError::InvalidPath)
        ));
    }
}
