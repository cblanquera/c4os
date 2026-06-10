use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileAccessActor {
    Agent,
    Browser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileAccessOperation {
    Read,
    Write,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ResolvedPath {
    pub project_root: PathBuf,
    pub requested_path: PathBuf,
    pub resolved_path: PathBuf,
    pub relative_path: PathBuf,
    pub is_secret_denied: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PathResolutionError {
    ProjectRootUnavailable,
    TargetUnavailable,
    OutsideProjectRoot,
    SecretDenied,
    TraversalEscapedRoot,
}

pub struct ProjectPathResolver {
    project_root: PathBuf,
}

impl ProjectPathResolver {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, PathResolutionError> {
        let project_root = fs::canonicalize(project_root)
            .map_err(|_| PathResolutionError::ProjectRootUnavailable)?;

        Ok(Self { project_root })
    }

    pub fn resolve_existing(
        &self,
        requested_path: impl AsRef<Path>,
        actor: FileAccessActor,
        operation: FileAccessOperation,
    ) -> Result<ResolvedPath, PathResolutionError> {
        let requested_path = self.normalize_request(requested_path.as_ref())?;
        let resolved_path = fs::canonicalize(&requested_path)
            .map_err(|_| PathResolutionError::TargetUnavailable)?;

        self.evaluate(requested_path, resolved_path, actor, operation)
    }

    pub fn resolve_for_write(
        &self,
        requested_path: impl AsRef<Path>,
        actor: FileAccessActor,
    ) -> Result<ResolvedPath, PathResolutionError> {
        let requested_path = self.normalize_request(requested_path.as_ref())?;
        let parent = requested_path
            .parent()
            .ok_or(PathResolutionError::OutsideProjectRoot)?;
        let resolved_parent =
            fs::canonicalize(parent).map_err(|_| PathResolutionError::TargetUnavailable)?;

        if !resolved_parent.starts_with(&self.project_root) {
            return Err(PathResolutionError::OutsideProjectRoot);
        }

        let resolved_path = resolved_parent.join(
            requested_path
                .file_name()
                .ok_or(PathResolutionError::OutsideProjectRoot)?,
        );

        self.evaluate(
            requested_path,
            resolved_path,
            actor,
            FileAccessOperation::Write,
        )
    }

    fn normalize_request(&self, requested_path: &Path) -> Result<PathBuf, PathResolutionError> {
        let combined = if requested_path.is_absolute() {
            requested_path.to_path_buf()
        } else {
            self.project_root.join(requested_path)
        };

        let mut normalized = PathBuf::new();

        for component in combined.components() {
            match component {
                Component::ParentDir => {
                    if !normalized.pop() {
                        return Err(PathResolutionError::TraversalEscapedRoot);
                    }
                }
                Component::CurDir => {}
                other => normalized.push(other.as_os_str()),
            }
        }

        Ok(normalized)
    }

    fn evaluate(
        &self,
        requested_path: PathBuf,
        resolved_path: PathBuf,
        actor: FileAccessActor,
        operation: FileAccessOperation,
    ) -> Result<ResolvedPath, PathResolutionError> {
        if !resolved_path.starts_with(&self.project_root) {
            return Err(PathResolutionError::OutsideProjectRoot);
        }

        let relative_path = resolved_path
            .strip_prefix(&self.project_root)
            .map_err(|_| PathResolutionError::OutsideProjectRoot)?
            .to_path_buf();
        let is_secret_denied = is_secret_denied_path(&relative_path);

        if actor == FileAccessActor::Agent && is_secret_denied {
            return Err(PathResolutionError::SecretDenied);
        }

        if actor == FileAccessActor::Browser
            && operation == FileAccessOperation::Read
            && is_secret_denied
        {
            return Ok(ResolvedPath {
                project_root: self.project_root.clone(),
                requested_path,
                resolved_path,
                relative_path,
                is_secret_denied,
            });
        }

        Ok(ResolvedPath {
            project_root: self.project_root.clone(),
            requested_path,
            resolved_path,
            relative_path,
            is_secret_denied,
        })
    }
}

pub fn is_secret_denied_path(relative_path: &Path) -> bool {
    let path = relative_path.to_string_lossy().replace('\\', "/");
    let file_name = relative_path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let lower_path = path.to_lowercase();

    file_name == ".env"
        || file_name.starts_with(".env.")
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.contains("credential")
        || file_name.contains("token")
        || lower_path.contains(".aws/credentials")
        || lower_path.contains(".config/gh/hosts.yml")
        || lower_path.contains(".npmrc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn allows_inside_root_file() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let resolved = resolver
            .resolve_existing(
                "README.md",
                FileAccessActor::Agent,
                FileAccessOperation::Read,
            )
            .expect("path resolved");

        assert_eq!(resolved.relative_path, PathBuf::from("README.md"));
        assert!(!resolved.is_secret_denied);
    }

    #[test]
    fn blocks_traversal_outside_root() {
        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let outside_file = outside.path().join("outside.txt");
        fs::write(&outside_file, "outside").expect("outside file written");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let result = resolver.resolve_existing(
            outside_file,
            FileAccessActor::Agent,
            FileAccessOperation::Read,
        );

        assert_eq!(result, Err(PathResolutionError::OutsideProjectRoot));
    }

    #[test]
    #[cfg(unix)]
    fn allows_symlink_when_target_remains_inside_root() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("tempdir");
        fs::create_dir(directory.path().join("real")).expect("dir written");
        fs::write(directory.path().join("real/file.txt"), "inside").expect("file written");
        symlink(
            directory.path().join("real/file.txt"),
            directory.path().join("link.txt"),
        )
        .expect("symlink");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let resolved = resolver
            .resolve_existing(
                "link.txt",
                FileAccessActor::Agent,
                FileAccessOperation::Read,
            )
            .expect("path resolved");

        assert_eq!(resolved.relative_path, PathBuf::from("real/file.txt"));
    }

    #[test]
    #[cfg(unix)]
    fn blocks_symlink_escape_outside_root() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        fs::write(outside.path().join("secret.txt"), "outside").expect("outside file written");
        symlink(
            outside.path().join("secret.txt"),
            directory.path().join("escape.txt"),
        )
        .expect("symlink");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let result = resolver.resolve_existing(
            "escape.txt",
            FileAccessActor::Agent,
            FileAccessOperation::Read,
        );

        assert_eq!(result, Err(PathResolutionError::OutsideProjectRoot));
    }

    #[test]
    fn blocks_secret_denied_files_for_agent_access() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join(".env"), "OPENROUTER_API_KEY=secret").expect("env written");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let result =
            resolver.resolve_existing(".env", FileAccessActor::Agent, FileAccessOperation::Read);

        assert_eq!(result, Err(PathResolutionError::SecretDenied));
    }

    #[test]
    fn browser_can_detect_secret_file_without_previewing_contents() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join(".env"), "OPENROUTER_API_KEY=secret").expect("env written");
        let resolver = ProjectPathResolver::new(directory.path()).expect("resolver");

        let resolved = resolver
            .resolve_existing(".env", FileAccessActor::Browser, FileAccessOperation::Read)
            .expect("secret metadata visible");

        assert!(resolved.is_secret_denied);
    }
}
