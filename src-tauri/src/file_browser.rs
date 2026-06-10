use crate::path_resolver::{
    FileAccessActor, FileAccessOperation, PathResolutionError, ProjectPathResolver,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct BrowserEntry {
    pub relative_path: PathBuf,
    pub is_directory: bool,
    pub is_secret_denied: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReadOnlyFile {
    pub relative_path: PathBuf,
    pub content: Option<String>,
    pub is_secret_denied: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FileBrowserError {
    Path(PathResolutionError),
    NotDirectory,
    NotFile,
    ReadFailed,
}

pub struct ReadOnlyProjectBrowser {
    resolver: ProjectPathResolver,
}

impl ReadOnlyProjectBrowser {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, FileBrowserError> {
        Ok(Self {
            resolver: ProjectPathResolver::new(project_root).map_err(FileBrowserError::Path)?,
        })
    }

    pub fn list_directory(
        &self,
        relative_path: impl AsRef<Path>,
    ) -> Result<Vec<BrowserEntry>, FileBrowserError> {
        let resolved = self
            .resolver
            .resolve_existing(
                relative_path,
                FileAccessActor::Browser,
                FileAccessOperation::Read,
            )
            .map_err(FileBrowserError::Path)?;

        if !resolved.resolved_path.is_dir() {
            return Err(FileBrowserError::NotDirectory);
        }

        let mut entries = Vec::new();

        for entry in
            fs::read_dir(resolved.resolved_path).map_err(|_| FileBrowserError::ReadFailed)?
        {
            let entry = entry.map_err(|_| FileBrowserError::ReadFailed)?;
            let child_relative = entry
                .path()
                .strip_prefix(&resolved.project_root)
                .map_err(|_| FileBrowserError::Path(PathResolutionError::OutsideProjectRoot))?
                .to_path_buf();
            let child_resolved = self.resolver.resolve_existing(
                &child_relative,
                FileAccessActor::Browser,
                FileAccessOperation::Read,
            );

            if let Ok(child) = child_resolved {
                entries.push(BrowserEntry {
                    relative_path: child.relative_path,
                    is_directory: child.resolved_path.is_dir(),
                    is_secret_denied: child.is_secret_denied,
                });
            }
        }

        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

        Ok(entries)
    }

    pub fn open_read_only(
        &self,
        relative_path: impl AsRef<Path>,
    ) -> Result<ReadOnlyFile, FileBrowserError> {
        let resolved = self
            .resolver
            .resolve_existing(
                relative_path,
                FileAccessActor::Browser,
                FileAccessOperation::Read,
            )
            .map_err(FileBrowserError::Path)?;

        if !resolved.resolved_path.is_file() {
            return Err(FileBrowserError::NotFile);
        }

        if resolved.is_secret_denied {
            return Ok(ReadOnlyFile {
                relative_path: resolved.relative_path,
                content: None,
                is_secret_denied: true,
            });
        }

        Ok(ReadOnlyFile {
            relative_path: resolved.relative_path,
            content: Some(
                fs::read_to_string(resolved.resolved_path)
                    .map_err(|_| FileBrowserError::ReadFailed)?,
            ),
            is_secret_denied: false,
        })
    }

    pub fn root_agents_guidance(&self) -> Result<Option<ReadOnlyFile>, FileBrowserError> {
        match self.open_read_only("AGENTS.md") {
            Ok(file) => Ok(Some(file)),
            Err(FileBrowserError::Path(PathResolutionError::TargetUnavailable)) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn opens_allowed_project_file_read_only() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let browser = ReadOnlyProjectBrowser::new(directory.path()).expect("browser");

        let file = browser.open_read_only("README.md").expect("file opened");

        assert_eq!(file.content, Some("hello".into()));
        assert!(!file.is_secret_denied);
    }

    #[test]
    fn blocks_outside_root_open() {
        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let outside_file = outside.path().join("outside.txt");
        fs::write(&outside_file, "outside").expect("outside written");
        let browser = ReadOnlyProjectBrowser::new(directory.path()).expect("browser");

        let result = browser.open_read_only(outside_file);

        assert!(matches!(
            result,
            Err(FileBrowserError::Path(
                PathResolutionError::OutsideProjectRoot
            ))
        ));
    }

    #[test]
    fn blocks_secret_preview_contents() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join(".env"), "OPENROUTER_API_KEY=secret").expect("env written");
        let browser = ReadOnlyProjectBrowser::new(directory.path()).expect("browser");

        let file = browser.open_read_only(".env").expect("secret detected");

        assert!(file.is_secret_denied);
        assert_eq!(file.content, None);
    }

    #[test]
    fn displays_root_agents_as_passive_file() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("AGENTS.md"), "guidance").expect("agents written");
        let browser = ReadOnlyProjectBrowser::new(directory.path()).expect("browser");

        let guidance = browser
            .root_agents_guidance()
            .expect("guidance lookup")
            .expect("guidance exists");

        assert_eq!(guidance.relative_path, PathBuf::from("AGENTS.md"));
        assert_eq!(guidance.content, Some("guidance".into()));
    }

    #[test]
    fn directory_listing_marks_secret_files_without_previewing() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("readme written");
        fs::write(directory.path().join(".env"), "secret").expect("env written");
        let browser = ReadOnlyProjectBrowser::new(directory.path()).expect("browser");

        let entries = browser.list_directory(".").expect("directory listed");

        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .any(|entry| entry.relative_path == PathBuf::from(".env") && entry.is_secret_denied));
    }
}
