use crate::storage::{AppStore, NewProject};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Eq, PartialEq)]
pub struct GitStatus {
    pub root_path: String,
    pub branch: String,
    pub dirty: bool,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ChangedFileEntry {
    pub path: String,
    pub status: String,
    pub diff_available: bool,
    pub tool_context: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct FileDiffView {
    pub path: String,
    pub status: String,
    pub diff: String,
    pub git_inspection_logged: bool,
    pub approval_required: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RegisteredProject {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub git_status: GitStatus,
    pub has_root_agents: bool,
}

#[derive(Debug)]
pub enum ProjectError {
    UnreadablePath(String),
    NotGitRepository(String),
    GitFailed(String),
    StoreFailed(rusqlite::Error),
}

pub struct ProjectService<'a> {
    app_store: &'a AppStore,
}

impl<'a> ProjectService<'a> {
    pub fn new(app_store: &'a AppStore) -> Self {
        Self { app_store }
    }

    pub fn register_git_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<RegisteredProject, ProjectError> {
        let root_path = canonicalize_path(path.as_ref())?;
        let git_root = resolve_git_root(&root_path)?;
        let git_status = inspect_git_status(&git_root)?;
        let name = git_root
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| git_root.to_string_lossy().to_string());
        let root_path_string = git_root.to_string_lossy().to_string();
        let id = stable_project_id(&root_path_string);

        self.app_store
            .create_project(NewProject {
                id: &id,
                name: &name,
                root_path: &root_path_string,
                default_model: None,
                default_agent_ref: None,
            })
            .map_err(ProjectError::StoreFailed)?;

        Ok(RegisteredProject {
            id,
            name,
            root_path: root_path_string,
            git_status,
            has_root_agents: git_root.join("AGENTS.md").is_file(),
        })
    }
}

pub fn inspect_git_status(root_path: impl AsRef<Path>) -> Result<GitStatus, ProjectError> {
    let root_path = canonicalize_path(root_path.as_ref())?;
    let branch = run_git(&root_path, &["branch", "--show-current"])?;
    let porcelain = run_git(&root_path, &["status", "--porcelain=v1"])?;
    let changed_files = parse_changed_files(&porcelain);

    Ok(GitStatus {
        root_path: root_path.to_string_lossy().to_string(),
        branch: if branch.trim().is_empty() {
            "HEAD".into()
        } else {
            branch.trim().into()
        },
        dirty: !changed_files.is_empty(),
        changed_files,
    })
}

pub fn inspect_file_diff(
    root_path: impl AsRef<Path>,
    file_path: &str,
) -> Result<String, ProjectError> {
    let root_path = canonicalize_path(root_path.as_ref())?;

    run_git(&root_path, &["diff", "--", file_path])
}

pub fn inspect_changed_files(
    root_path: impl AsRef<Path>,
) -> Result<Vec<ChangedFileEntry>, ProjectError> {
    let root_path = canonicalize_path(root_path.as_ref())?;
    let porcelain = run_git(&root_path, &["status", "--porcelain=v1"])?;

    Ok(parse_changed_file_entries(&porcelain))
}

pub fn inspect_changed_file_diff(
    root_path: impl AsRef<Path>,
    file_path: &str,
) -> Result<FileDiffView, ProjectError> {
    let root_path = canonicalize_path(root_path.as_ref())?;
    let status = inspect_changed_files(&root_path)?
        .into_iter()
        .find(|entry| entry.path == file_path)
        .map(|entry| entry.status)
        .unwrap_or_else(|| "clean".into());
    let diff = match status.as_str() {
        "untracked" => untracked_file_preview(&root_path, file_path)?,
        "added" => first_non_empty_git_diff(&root_path, file_path, true)?,
        _ => first_non_empty_git_diff(&root_path, file_path, false)?,
    };

    Ok(FileDiffView {
        path: file_path.into(),
        status,
        diff,
        git_inspection_logged: true,
        approval_required: false,
    })
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, ProjectError> {
    fs::canonicalize(path).map_err(|error| {
        ProjectError::UnreadablePath(format!("unable to read path {}: {error}", path.display()))
    })
}

fn resolve_git_root(path: &Path) -> Result<PathBuf, ProjectError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| ProjectError::GitFailed(format!("git unavailable: {error}")))?;

    if !output.status.success() {
        return Err(ProjectError::NotGitRepository(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    ))
}

fn run_git(root_path: &Path, args: &[&str]) -> Result<String, ProjectError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root_path)
        .args(args)
        .output()
        .map_err(|error| ProjectError::GitFailed(format!("git unavailable: {error}")))?;

    if !output.status.success() {
        return Err(ProjectError::GitFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_changed_files(porcelain: &str) -> Vec<String> {
    parse_changed_file_entries(porcelain)
        .into_iter()
        .map(|entry| entry.path)
        .collect()
}

fn parse_changed_file_entries(porcelain: &str) -> Vec<ChangedFileEntry> {
    porcelain
        .lines()
        .filter_map(parse_changed_file_entry)
        .collect()
}

fn parse_changed_file_entry(line: &str) -> Option<ChangedFileEntry> {
    let status_code = line.get(0..2)?;
    let path = line.get(3..)?;
    let path = path.split(" -> ").last().unwrap_or(path).trim().to_string();

    if path.is_empty() {
        return None;
    }

    Some(ChangedFileEntry {
        path,
        status: status_from_porcelain(status_code),
        diff_available: true,
        tool_context: None,
    })
}

fn status_from_porcelain(status_code: &str) -> String {
    if status_code == "??" {
        return "untracked".into();
    }

    if status_code.contains('D') {
        return "deleted".into();
    }

    if status_code.contains('A') {
        return "added".into();
    }

    if status_code.contains('M') {
        return "modified".into();
    }

    if status_code.contains('R') {
        return "renamed".into();
    }

    "changed".into()
}

fn first_non_empty_git_diff(
    root_path: &Path,
    file_path: &str,
    prefer_cached: bool,
) -> Result<String, ProjectError> {
    let first_args = if prefer_cached {
        vec!["diff", "--cached", "--", file_path]
    } else {
        vec!["diff", "--", file_path]
    };
    let second_args = if prefer_cached {
        vec!["diff", "--", file_path]
    } else {
        vec!["diff", "--cached", "--", file_path]
    };
    let first = run_git(root_path, &first_args)?;

    if !first.trim().is_empty() {
        return Ok(first);
    }

    run_git(root_path, &second_args)
}

fn untracked_file_preview(root_path: &Path, file_path: &str) -> Result<String, ProjectError> {
    let path = root_path.join(file_path);
    let content = fs::read_to_string(&path).map_err(|error| {
        ProjectError::UnreadablePath(format!("unable to read path {}: {error}", path.display()))
    })?;

    Ok(format!("Untracked file: {file_path}\n\n{content}"))
}

fn _legacy_parse_changed_files(porcelain: &str) -> Vec<String> {
    porcelain
        .lines()
        .filter_map(|line| line.get(3..))
        .map(|path| path.split(" -> ").last().unwrap_or(path).trim().to_string())
        .filter(|path| !path.is_empty())
        .collect()
}

fn stable_project_id(root_path: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in root_path.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("project-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn registers_git_project_and_persists_root_status() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        fs::write(directory.path().join("AGENTS.md"), "project guidance").expect("agents written");

        let store = AppStore::open_in_memory().expect("store opens");
        let service = ProjectService::new(&store);
        let project = service
            .register_git_project(directory.path())
            .expect("project registered");
        let persisted = store
            .get_project(&project.id)
            .expect("project query")
            .expect("project exists");

        assert_eq!(persisted.root_path, project.root_path);
        assert!(project.has_root_agents);
        assert!(project.git_status.dirty);
        assert_eq!(project.git_status.changed_files, vec!["AGENTS.md"]);
    }

    #[test]
    fn rejects_non_git_directory() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let service = ProjectService::new(&store);

        let result = service.register_git_project(directory.path());

        assert!(matches!(result, Err(ProjectError::NotGitRepository(_))));
    }

    #[test]
    fn reports_dirty_state_and_changed_files() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        fs::write(directory.path().join("changed.txt"), "hello").expect("file written");

        let status = inspect_git_status(directory.path()).expect("git status");

        assert!(status.dirty);
        assert_eq!(status.changed_files, vec!["changed.txt"]);
    }

    #[test]
    fn changed_file_list_classifies_modified_added_deleted_and_untracked_files() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        write_and_commit_file(directory.path(), "modified.txt", "before\n");
        write_and_commit_file(directory.path(), "deleted.txt", "gone\n");
        fs::write(directory.path().join("modified.txt"), "after\n").expect("modified written");
        fs::remove_file(directory.path().join("deleted.txt")).expect("deleted");
        fs::write(directory.path().join("added.txt"), "new\n").expect("added written");
        run_git_command(directory.path(), &["add", "added.txt"]);
        fs::write(directory.path().join("untracked.txt"), "loose\n").expect("untracked written");

        let changed = inspect_changed_files(directory.path()).expect("changed files");

        assert!(changed.contains(&ChangedFileEntry {
            path: "modified.txt".into(),
            status: "modified".into(),
            diff_available: true,
            tool_context: None,
        }));
        assert!(changed.contains(&ChangedFileEntry {
            path: "added.txt".into(),
            status: "added".into(),
            diff_available: true,
            tool_context: None,
        }));
        assert!(changed.contains(&ChangedFileEntry {
            path: "deleted.txt".into(),
            status: "deleted".into(),
            diff_available: true,
            tool_context: None,
        }));
        assert!(changed.contains(&ChangedFileEntry {
            path: "untracked.txt".into(),
            status: "untracked".into(),
            diff_available: true,
            tool_context: None,
        }));
    }

    #[test]
    fn diff_viewer_returns_logged_non_gated_modified_diff() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        write_and_commit_file(directory.path(), "modified.txt", "before\n");
        fs::write(directory.path().join("modified.txt"), "after\n").expect("modified written");

        let diff = inspect_changed_file_diff(directory.path(), "modified.txt").expect("diff");

        assert_eq!(diff.status, "modified");
        assert!(diff.diff.contains("-before"));
        assert!(diff.diff.contains("+after"));
        assert!(diff.git_inspection_logged);
        assert!(!diff.approval_required);
    }

    #[test]
    fn diff_viewer_returns_cached_added_diff() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        fs::write(directory.path().join("added.txt"), "new\n").expect("added written");
        run_git_command(directory.path(), &["add", "added.txt"]);

        let diff = inspect_changed_file_diff(directory.path(), "added.txt").expect("diff");

        assert_eq!(diff.status, "added");
        assert!(diff.diff.contains("new file mode"));
        assert!(diff.diff.contains("+new"));
    }

    #[test]
    fn diff_viewer_returns_deleted_diff() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        write_and_commit_file(directory.path(), "deleted.txt", "gone\n");
        fs::remove_file(directory.path().join("deleted.txt")).expect("deleted");

        let diff = inspect_changed_file_diff(directory.path(), "deleted.txt").expect("diff");

        assert_eq!(diff.status, "deleted");
        assert!(diff.diff.contains("deleted file mode"));
        assert!(diff.diff.contains("-gone"));
    }

    #[test]
    fn diff_viewer_returns_untracked_preview_where_supported() {
        let directory = tempdir().expect("tempdir");
        init_git_repo(directory.path());
        fs::write(directory.path().join("untracked.txt"), "loose\n").expect("untracked written");

        let diff = inspect_changed_file_diff(directory.path(), "untracked.txt").expect("diff");

        assert_eq!(diff.status, "untracked");
        assert!(diff.diff.contains("Untracked file: untracked.txt"));
        assert!(diff.diff.contains("loose"));
    }

    fn init_git_repo(path: &Path) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("init")
            .output()
            .expect("git init runs");

        assert!(output.status.success());
    }

    fn write_and_commit_file(path: &Path, file_path: &str, content: &str) {
        fs::write(path.join(file_path), content).expect("file written");
        run_git_command(path, &["add", file_path]);
        run_git_command(
            path,
            &[
                "-c",
                "user.name=C4OS Test",
                "-c",
                "user.email=c4os@example.test",
                "commit",
                "-m",
                "baseline",
            ],
        );
    }

    fn run_git_command(path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("git command runs");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
