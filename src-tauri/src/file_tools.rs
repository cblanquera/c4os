use crate::path_resolver::{
    FileAccessActor, FileAccessOperation, PathResolutionError, ProjectPathResolver,
};
use std::fs;
use std::path::Path;

const MAX_BATCH_WRITES: usize = 10;
const MAX_PREVIEW_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileWriteProposal {
    pub relative_path: String,
    pub content: String,
    pub preview: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedFileWrite {
    proposal: FileWriteProposal,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FileToolError {
    Path(PathResolutionError),
    SecretDenied,
    BatchTooLarge,
    PreviewTooLarge,
    ReadFailed,
    WriteFailed,
}

pub struct FileToolExecutor {
    resolver: ProjectPathResolver,
}

impl FileToolExecutor {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, FileToolError> {
        Ok(Self {
            resolver: ProjectPathResolver::new(project_root).map_err(FileToolError::Path)?,
        })
    }

    pub fn read_file(&self, path: &str) -> Result<String, FileToolError> {
        let resolved = self
            .resolver
            .resolve_existing(path, FileAccessActor::Agent, FileAccessOperation::Read)
            .map_err(map_resolution_error)?;

        fs::read_to_string(resolved.resolved_path).map_err(|_| FileToolError::ReadFailed)
    }

    pub fn propose_write(
        &self,
        path: &str,
        content: &str,
    ) -> Result<FileWriteProposal, FileToolError> {
        let resolved = self
            .resolver
            .resolve_for_write(path, FileAccessActor::Agent)
            .map_err(map_resolution_error)?;
        let preview = build_preview(content)?;

        Ok(FileWriteProposal {
            relative_path: resolved.relative_path.to_string_lossy().to_string(),
            content: content.into(),
            preview,
        })
    }

    pub fn approve_write(proposal: FileWriteProposal) -> ApprovedFileWrite {
        ApprovedFileWrite { proposal }
    }

    pub fn execute_write(&self, approved: ApprovedFileWrite) -> Result<(), FileToolError> {
        let resolved = self
            .resolver
            .resolve_for_write(&approved.proposal.relative_path, FileAccessActor::Agent)
            .map_err(map_resolution_error)?;

        fs::write(resolved.resolved_path, approved.proposal.content)
            .map_err(|_| FileToolError::WriteFailed)
    }

    pub fn propose_batch(
        &self,
        writes: &[(&str, &str)],
    ) -> Result<Vec<FileWriteProposal>, FileToolError> {
        if writes.len() > MAX_BATCH_WRITES {
            return Err(FileToolError::BatchTooLarge);
        }

        let mut proposals = Vec::new();
        let mut preview_bytes = 0;

        for (path, content) in writes {
            let proposal = self.propose_write(path, content)?;
            preview_bytes += proposal.preview.len();

            if preview_bytes > MAX_PREVIEW_BYTES {
                return Err(FileToolError::PreviewTooLarge);
            }

            proposals.push(proposal);
        }

        Ok(proposals)
    }
}

fn build_preview(content: &str) -> Result<String, FileToolError> {
    if content.len() > MAX_PREVIEW_BYTES {
        return Err(FileToolError::PreviewTooLarge);
    }

    Ok(content.lines().take(80).collect::<Vec<_>>().join("\n"))
}

fn map_resolution_error(error: PathResolutionError) -> FileToolError {
    match error {
        PathResolutionError::SecretDenied => FileToolError::SecretDenied,
        other => FileToolError::Path(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reads_allowed_project_file() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let executor = FileToolExecutor::new(directory.path()).expect("executor");

        let content = executor.read_file("README.md").expect("file read");

        assert_eq!(content, "hello");
    }

    #[test]
    fn blocks_secret_file_reads() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join(".env"), "secret").expect("file written");
        let executor = FileToolExecutor::new(directory.path()).expect("executor");

        let result = executor.read_file(".env");

        assert_eq!(result, Err(FileToolError::SecretDenied));
    }

    #[test]
    fn denied_write_proposal_does_not_modify_file() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "before").expect("file written");
        let executor = FileToolExecutor::new(directory.path()).expect("executor");

        let _proposal = executor
            .propose_write("README.md", "after")
            .expect("write proposed");

        assert_eq!(
            fs::read_to_string(directory.path().join("README.md")).expect("file read"),
            "before"
        );
    }

    #[test]
    fn approved_write_modifies_file() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "before").expect("file written");
        let executor = FileToolExecutor::new(directory.path()).expect("executor");
        let proposal = executor
            .propose_write("README.md", "after")
            .expect("write proposed");
        let approved = FileToolExecutor::approve_write(proposal);

        executor.execute_write(approved).expect("write executed");

        assert_eq!(
            fs::read_to_string(directory.path().join("README.md")).expect("file read"),
            "after"
        );
    }

    #[test]
    fn blocks_batch_over_ten_files() {
        let directory = tempdir().expect("tempdir");
        let executor = FileToolExecutor::new(directory.path()).expect("executor");
        let writes = vec![
            ("a0.txt", "x"),
            ("a1.txt", "x"),
            ("a2.txt", "x"),
            ("a3.txt", "x"),
            ("a4.txt", "x"),
            ("a5.txt", "x"),
            ("a6.txt", "x"),
            ("a7.txt", "x"),
            ("a8.txt", "x"),
            ("a9.txt", "x"),
            ("a10.txt", "x"),
        ];

        let result = executor.propose_batch(&writes);

        assert_eq!(result, Err(FileToolError::BatchTooLarge));
    }
}
