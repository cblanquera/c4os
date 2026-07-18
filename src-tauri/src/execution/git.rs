use super::environment::{FilesystemObjectIdentity, TrustedProjectRoot};
use crate::security::gateway::ExecutionPermit;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

const MAX_GIT_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_BRANCH_BYTES: usize = 255;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommandInvocation {
    pub program: PathBuf,
    pub current_dir: PathBuf,
    pub arguments: Vec<OsString>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitCommandOutput {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl GitCommandOutput {
    pub fn success(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: Vec::new(),
        }
    }

    pub fn failure(exit_code: i32, stderr: impl Into<Vec<u8>>) -> Self {
        Self {
            exit_code,
            stdout: Vec::new(),
            stderr: stderr.into(),
        }
    }

    fn validate_bounds(&self) -> Result<(), GitError> {
        if self.stdout.len() > MAX_GIT_OUTPUT_BYTES || self.stderr.len() > MAX_GIT_OUTPUT_BYTES {
            return Err(GitError::OutputTooLarge);
        }
        Ok(())
    }
}

pub trait GitCommandRunner {
    fn run(&mut self, invocation: &GitCommandInvocation) -> Result<GitCommandOutput, GitError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveProjectRepository {
    project_root: TrustedProjectRoot,
    repository_root: PathBuf,
    repository_identity: FilesystemObjectIdentity,
    git_program: PathBuf,
}

impl ActiveProjectRepository {
    pub fn project_root(&self) -> &TrustedProjectRoot {
        &self.project_root
    }

    pub fn repository_root(&self) -> &Path {
        &self.repository_root
    }

    pub fn repository_identity(&self) -> &FilesystemObjectIdentity {
        &self.repository_identity
    }

    fn revalidate(&self) -> Result<(), GitError> {
        self.project_root
            .revalidate()
            .map_err(|error| GitError::StaleRepository(error.to_string()))?;
        let metadata = fs::symlink_metadata(&self.repository_root)
            .map_err(|_| GitError::StaleRepository("repository root is unavailable".into()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(GitError::StaleRepository(
                "repository root changed type".into(),
            ));
        }
        let current = FilesystemObjectIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            kind: super::environment::FilesystemObjectKind::Directory,
        };
        if current != self.repository_identity {
            return Err(GitError::StaleRepository(
                "repository root identity changed".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BranchControlVisibility {
    Visible(ActiveProjectRepository),
    HiddenNonRepository,
    /// Git owns an ancestor or descendant beyond the active Project boundary.
    /// A branch switch would affect paths outside the Project and is therefore
    /// not exposed as the Project-scoped Branch control.
    HiddenRepositoryCrossesProjectBoundary {
        repository_root: PathBuf,
    },
}

pub fn inspect_branch_control<R: GitCommandRunner>(
    project_root: TrustedProjectRoot,
    git_program: &Path,
    runner: &mut R,
) -> Result<BranchControlVisibility, GitError> {
    if !git_program.is_absolute() {
        return Err(GitError::InvalidRequest(
            "Git program must use an absolute trusted path".into(),
        ));
    }
    let output = run_git(
        runner,
        git_program,
        project_root.canonical_root(),
        ["rev-parse", "--show-toplevel"],
    )?;
    if output.exit_code != 0 {
        return Ok(BranchControlVisibility::HiddenNonRepository);
    }
    let repository_text = single_line(&output.stdout, "repository root")?;
    let repository_root = fs::canonicalize(repository_text).map_err(|error| {
        GitError::RepositoryUnavailable(PathBuf::from(repository_text), error.to_string())
    })?;
    if repository_root != project_root.canonical_root() {
        return Ok(
            BranchControlVisibility::HiddenRepositoryCrossesProjectBoundary { repository_root },
        );
    }
    let metadata = fs::symlink_metadata(&repository_root).map_err(|error| {
        GitError::RepositoryUnavailable(repository_root.clone(), error.to_string())
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(GitError::RepositoryUnavailable(
            repository_root,
            "repository root is not a physical directory".into(),
        ));
    }
    Ok(BranchControlVisibility::Visible(ActiveProjectRepository {
        project_root,
        repository_root,
        repository_identity: FilesystemObjectIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            kind: super::environment::FilesystemObjectKind::Directory,
        },
        git_program: git_program.to_path_buf(),
    }))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveWritePolicy {
    Allow,
    Ask,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectTargetClassification {
    VersionControlledInProject { canonical_target: PathBuf },
    NonVersionControlledInProject { canonical_target: PathBuf },
    OutsideActiveProject { canonical_target: PathBuf },
    SymlinkEscape { canonical_target: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChatWriteDisposition {
    AllowWithoutAdditionalChangePrompt,
    RequirePathAndChangeApproval {
        canonical_target: PathBuf,
        change_sha256: String,
    },
    Deny {
        reason: String,
    },
}

pub fn classify_project_target(
    project_root: &TrustedProjectRoot,
    repository: &BranchControlVisibility,
    requested_target: &Path,
) -> Result<ProjectTargetClassification, GitError> {
    project_root
        .revalidate()
        .map_err(|error| GitError::StaleRepository(error.to_string()))?;
    let lexical_target = if requested_target.is_absolute() {
        normalize_absolute(requested_target)?
    } else {
        normalize_absolute(&project_root.canonical_root().join(requested_target))?
    };
    let canonical_target = canonicalize_allow_missing(&lexical_target)?;
    let lexically_inside = lexical_target.starts_with(project_root.canonical_root());
    let physically_inside = canonical_target.starts_with(project_root.canonical_root());
    if lexically_inside && !physically_inside {
        return Ok(ProjectTargetClassification::SymlinkEscape { canonical_target });
    }
    if !physically_inside {
        return Ok(ProjectTargetClassification::OutsideActiveProject { canonical_target });
    }
    match repository {
        BranchControlVisibility::Visible(repository)
            if canonical_target.starts_with(repository.repository_root()) =>
        {
            Ok(ProjectTargetClassification::VersionControlledInProject { canonical_target })
        }
        _ => Ok(ProjectTargetClassification::NonVersionControlledInProject { canonical_target }),
    }
}

pub fn resolve_chat_write_disposition(
    policy: EffectiveWritePolicy,
    classification: ProjectTargetClassification,
    change_sha256: impl Into<String>,
) -> Result<ChatWriteDisposition, GitError> {
    let change_sha256 = change_sha256.into();
    validate_sha256(&change_sha256)?;
    if let ProjectTargetClassification::SymlinkEscape { canonical_target } = classification {
        return Ok(ChatWriteDisposition::Deny {
            reason: format!(
                "target resolves outside the trusted Project root: {}",
                canonical_target.display()
            ),
        });
    }
    if policy == EffectiveWritePolicy::Deny {
        return Ok(ChatWriteDisposition::Deny {
            reason: "effective policy denies the file change".into(),
        });
    }
    match (policy, classification) {
        (
            EffectiveWritePolicy::Allow,
            ProjectTargetClassification::VersionControlledInProject { .. },
        ) => Ok(ChatWriteDisposition::AllowWithoutAdditionalChangePrompt),
        (
            EffectiveWritePolicy::Allow | EffectiveWritePolicy::Ask,
            ProjectTargetClassification::VersionControlledInProject { canonical_target }
            | ProjectTargetClassification::NonVersionControlledInProject { canonical_target }
            | ProjectTargetClassification::OutsideActiveProject { canonical_target },
        ) => Ok(ChatWriteDisposition::RequirePathAndChangeApproval {
            canonical_target,
            change_sha256,
        }),
        (EffectiveWritePolicy::Deny, _)
        | (_, ProjectTargetClassification::SymlinkEscape { .. }) => {
            unreachable!("denial and symlink escape returned above")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStateVersion {
    pub head_oid: String,
    pub worktree_porcelain_v1_z: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum GitBranchOperation {
    Switch {
        branch: String,
        expected_target_oid: String,
    },
    Create {
        branch: String,
    },
}

impl GitBranchOperation {
    fn branch(&self) -> &str {
        match self {
            Self::Switch { branch, .. } | Self::Create { branch } => branch,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchRequest {
    pub request_id: String,
    pub repository_root: PathBuf,
    pub expected_state: GitStateVersion,
    pub operation: GitBranchOperation,
}

impl GitBranchRequest {
    fn validate(&self, repository: &ActiveProjectRepository) -> Result<(), GitError> {
        validate_identifier("request id", &self.request_id)?;
        if self.repository_root != repository.repository_root {
            return Err(GitError::RepositorySubstitution);
        }
        validate_branch_name(self.operation.branch())?;
        if let GitBranchOperation::Switch {
            expected_target_oid,
            ..
        } = &self.operation
            && !valid_object_id(expected_target_oid)
        {
            return Err(GitError::InvalidRequest(
                "expected target branch object id is invalid".into(),
            ));
        }
        if !valid_object_id(&self.expected_state.head_oid)
            || self.expected_state.worktree_porcelain_v1_z.len() > MAX_GIT_OUTPUT_BYTES
        {
            return Err(GitError::InvalidRequest(
                "expected Git state is invalid".into(),
            ));
        }
        Ok(())
    }

    pub fn authorization_binding_sha256(
        &self,
        repository: &ActiveProjectRepository,
    ) -> Result<String, GitError> {
        self.validate(repository)?;
        let bytes = serde_json::to_vec(&GitAuthorizationBinding {
            request: self,
            repository_identity: &repository.repository_identity,
        })
        .map_err(|error| GitError::InvalidRequest(error.to_string()))?;
        Ok(format!("sha256:{}", sha256_hex(&bytes)))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitAuthorizationBinding<'a> {
    request: &'a GitBranchRequest,
    repository_identity: &'a FilesystemObjectIdentity,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExactGitOperationGrant {
    authorization_id: String,
    repository_identity: FilesystemObjectIdentity,
    request: GitBranchRequest,
}

impl ExactGitOperationGrant {
    fn from_gateway(
        authorization_id: &str,
        repository: &ActiveProjectRepository,
        request: GitBranchRequest,
    ) -> Result<Self, GitError> {
        validate_identifier("authorization id", authorization_id)?;
        request.validate(repository)?;
        Ok(Self {
            authorization_id: authorization_id.into(),
            repository_identity: repository.repository_identity.clone(),
            request,
        })
    }

    pub fn authorization_id(&self) -> &str {
        &self.authorization_id
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum GitOperationAuthorization {
    Allowed(ExactGitOperationGrant),
    Denied { reason: String },
}

impl GitOperationAuthorization {
    /// Produces an exact Git capability only from a consumed Action Gateway
    /// permit. The operation digest includes repository identity and Git state.
    pub fn from_gateway(
        permit: ExecutionPermit,
        repository: &ActiveProjectRepository,
        request: GitBranchRequest,
    ) -> Result<Self, GitError> {
        let action = permit.action();
        let expected_binding = request.authorization_binding_sha256(repository)?;
        let supplied_binding = action
            .arguments
            .get("gitRequestSha256")
            .and_then(serde_json::Value::as_str);
        let exact_arguments = action.arguments.as_object().is_some_and(|arguments| {
            arguments.len() == 1 && arguments.contains_key("gitRequestSha256")
        });
        if action.tool != "c4os.git"
            || action.requested_authority.len() != 1
            || !action.requested_authority.contains("git.branch")
            || !exact_arguments
            || action.action_id != request.request_id
            || supplied_binding != Some(expected_binding.as_str())
        {
            return Err(GitError::AuthorizationMismatch);
        }
        Ok(Self::Allowed(ExactGitOperationGrant::from_gateway(
            permit.authorization_id(),
            repository,
            request,
        )?))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitBranchOutcome {
    Switched {
        branch: String,
        preserved_dirty_state: bool,
    },
    Created {
        branch: String,
        preserved_dirty_state: bool,
    },
    Blocked {
        conflicting_paths: Vec<PathBuf>,
        message: String,
    },
}

pub fn capture_git_state<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    runner: &mut R,
) -> Result<GitStateVersion, GitError> {
    repository.revalidate()?;
    let head = run_repository_git(repository, runner, ["rev-parse", "--verify", "HEAD"])?;
    if head.exit_code != 0 {
        return Err(GitError::CommandFailed(safe_message(&head.stderr)));
    }
    let status = run_repository_git(
        repository,
        runner,
        ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    if status.exit_code != 0 {
        return Err(GitError::CommandFailed(safe_message(&status.stderr)));
    }
    Ok(GitStateVersion {
        head_oid: single_line(&head.stdout, "HEAD object id")?.to_owned(),
        worktree_porcelain_v1_z: status.stdout,
    })
}

pub fn execute_branch_operation<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    request: GitBranchRequest,
    authorization: GitOperationAuthorization,
    runner: &mut R,
) -> Result<GitBranchOutcome, GitError> {
    let grant = match authorization {
        GitOperationAuthorization::Denied { reason } => return Err(GitError::Denied(reason)),
        GitOperationAuthorization::Allowed(grant) => grant,
    };
    request.validate(repository)?;
    if grant.request != request || grant.repository_identity != repository.repository_identity {
        return Err(GitError::AuthorizationMismatch);
    }
    repository.revalidate()?;

    let observed_before = capture_git_state(repository, runner)?;
    if observed_before != request.expected_state {
        return Err(GitError::StaleGitState);
    }
    ensure_no_external_checkout_filters(repository, runner)?;
    validate_branch_with_git(repository, request.operation.branch(), runner)?;
    if let GitBranchOperation::Switch {
        branch,
        expected_target_oid,
    } = &request.operation
    {
        validate_existing_local_branch(repository, branch, expected_target_oid, runner)?;
    }

    let branch = request.operation.branch().to_owned();
    let arguments: Vec<OsString> = match &request.operation {
        GitBranchOperation::Switch { branch, .. } => vec![
            OsString::from("switch"),
            OsString::from("--no-recurse-submodules"),
            OsString::from("--"),
            OsString::from(branch),
        ],
        GitBranchOperation::Create { branch } => vec![
            OsString::from("switch"),
            OsString::from("--no-recurse-submodules"),
            OsString::from("--create"),
            OsString::from(branch),
        ],
    };
    let switch = run_repository_git_os(repository, runner, arguments)?;
    let observed_after = capture_git_state(repository, runner)?;
    if observed_after.worktree_porcelain_v1_z != observed_before.worktree_porcelain_v1_z {
        return Err(GitError::WorktreeInvariantViolation);
    }

    if switch.exit_code == 0 {
        let current_branch = run_repository_git(
            repository,
            runner,
            ["symbolic-ref", "--quiet", "--short", "HEAD"],
        )?;
        if current_branch.exit_code != 0
            || single_line(&current_branch.stdout, "current branch")? != branch
        {
            return Err(GitError::BranchSubstitution);
        }
        let preserved_dirty_state = !observed_before.worktree_porcelain_v1_z.is_empty();
        return Ok(match request.operation {
            GitBranchOperation::Switch { .. } => GitBranchOutcome::Switched {
                branch,
                preserved_dirty_state,
            },
            GitBranchOperation::Create { .. } => GitBranchOutcome::Created {
                branch,
                preserved_dirty_state,
            },
        });
    }

    if observed_after != observed_before {
        return Err(GitError::WorktreeInvariantViolation);
    }
    let conflicting_paths = parse_conflict_paths(&switch.stderr);
    if conflicting_paths.is_empty() {
        return Err(GitError::CommandFailed(safe_message(&switch.stderr)));
    }
    Ok(GitBranchOutcome::Blocked {
        conflicting_paths,
        message: safe_message(&switch.stderr),
    })
}

#[derive(Debug, Error)]
pub enum GitError {
    #[error("invalid Git request: {0}")]
    InvalidRequest(String),
    #[error("Git repository is unavailable at {0}: {1}")]
    RepositoryUnavailable(PathBuf, String),
    #[error("Git repository state changed: {0}")]
    StaleRepository(String),
    #[error("Git repository substitution was rejected")]
    RepositorySubstitution,
    #[error("Git authorization did not bind the exact operation")]
    AuthorizationMismatch,
    #[error("Git operation was denied before effect: {0}")]
    Denied(String),
    #[error("Git state changed after authorization")]
    StaleGitState,
    #[error("Git output exceeded its byte limit")]
    OutputTooLarge,
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    #[error("Git branch operation changed the dirty worktree unexpectedly")]
    WorktreeInvariantViolation,
    #[error("Git completed a different branch operation than the authorized request")]
    BranchSubstitution,
    #[error("Git repository has executable checkout filters that require separate authorization")]
    UnsafeRepositoryConfiguration,
}

fn run_repository_git<R: GitCommandRunner, const N: usize>(
    repository: &ActiveProjectRepository,
    runner: &mut R,
    arguments: [&str; N],
) -> Result<GitCommandOutput, GitError> {
    run_git(
        runner,
        &repository.git_program,
        &repository.repository_root,
        arguments,
    )
}

fn run_repository_git_os<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    runner: &mut R,
    arguments: Vec<OsString>,
) -> Result<GitCommandOutput, GitError> {
    let output = runner.run(&GitCommandInvocation {
        program: repository.git_program.clone(),
        current_dir: repository.repository_root.clone(),
        arguments: hardened_git_arguments(arguments),
    })?;
    output.validate_bounds()?;
    Ok(output)
}

fn run_git<R: GitCommandRunner, const N: usize>(
    runner: &mut R,
    git_program: &Path,
    current_dir: &Path,
    arguments: [&str; N],
) -> Result<GitCommandOutput, GitError> {
    let output = runner.run(&GitCommandInvocation {
        program: git_program.to_path_buf(),
        current_dir: current_dir.to_path_buf(),
        arguments: hardened_git_arguments(arguments.into_iter().map(OsString::from).collect()),
    })?;
    output.validate_bounds()?;
    Ok(output)
}

fn hardened_git_arguments(arguments: Vec<OsString>) -> Vec<OsString> {
    [
        OsString::from("-c"),
        OsString::from("core.hooksPath=/dev/null"),
        OsString::from("-c"),
        OsString::from("core.fsmonitor=false"),
        OsString::from("-c"),
        OsString::from("core.pager=cat"),
    ]
    .into_iter()
    .chain(arguments)
    .collect()
}

fn validate_branch_with_git<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    branch: &str,
    runner: &mut R,
) -> Result<(), GitError> {
    let output = run_repository_git_os(
        repository,
        runner,
        vec![
            OsString::from("check-ref-format"),
            OsString::from("--branch"),
            OsString::from(branch),
        ],
    )?;
    if output.exit_code == 0 {
        Ok(())
    } else {
        Err(GitError::InvalidRequest("invalid Git branch name".into()))
    }
}

fn ensure_no_external_checkout_filters<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    runner: &mut R,
) -> Result<(), GitError> {
    let output = run_repository_git(
        repository,
        runner,
        [
            "config",
            "--local",
            "--get-regexp",
            r"^filter\..*\.(clean|smudge|process|required)$",
        ],
    )?;
    match output.exit_code {
        1 if output.stdout.is_empty() => Ok(()),
        0 if output.stdout.is_empty() => Ok(()),
        0 => Err(GitError::UnsafeRepositoryConfiguration),
        _ => Err(GitError::CommandFailed(safe_message(&output.stderr))),
    }
}

fn validate_existing_local_branch<R: GitCommandRunner>(
    repository: &ActiveProjectRepository,
    branch: &str,
    expected_target_oid: &str,
    runner: &mut R,
) -> Result<(), GitError> {
    let reference = format!("refs/heads/{branch}");
    let output = run_repository_git_os(
        repository,
        runner,
        vec![
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from(reference),
        ],
    )?;
    if output.exit_code != 0 {
        Err(GitError::InvalidRequest(
            "Git branch does not exist locally".into(),
        ))
    } else if single_line(&output.stdout, "target branch object id")? != expected_target_oid {
        Err(GitError::StaleGitState)
    } else {
        Ok(())
    }
}

fn validate_identifier(kind: &str, value: &str) -> Result<(), GitError> {
    let valid = !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        });
    if valid {
        Ok(())
    } else {
        Err(GitError::InvalidRequest(format!("invalid {kind}")))
    }
}

fn validate_branch_name(branch: &str) -> Result<(), GitError> {
    if branch.is_empty()
        || branch.len() > MAX_BRANCH_BYTES
        || branch.starts_with('-')
        || branch.chars().any(char::is_control)
    {
        Err(GitError::InvalidRequest(
            "Git branch name is invalid".into(),
        ))
    } else {
        Ok(())
    }
}

fn validate_sha256(value: &str) -> Result<(), GitError> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(GitError::InvalidRequest(
            "change digest must be a sha256 value".into(),
        ))
    }
}

fn valid_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn single_line<'a>(bytes: &'a [u8], label: &str) -> Result<&'a str, GitError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| GitError::CommandFailed(format!("{label} is not UTF-8")))?
        .trim();
    if value.is_empty() || value.contains('\n') || value.contains('\r') {
        return Err(GitError::CommandFailed(format!("invalid {label}")));
    }
    Ok(value)
}

fn normalize_absolute(path: &Path) -> Result<PathBuf, GitError> {
    if !path.is_absolute() {
        return Err(GitError::InvalidRequest(
            "target classification requires an absolute path".into(),
        ));
    }
    let mut prefix = PathBuf::new();
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => prefix.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(value) => names.push(value.to_os_string()),
            Component::ParentDir => {
                if names.pop().is_none() {
                    return Err(GitError::InvalidRequest("target path is invalid".into()));
                }
            }
        }
    }
    let normalized = names.into_iter().fold(prefix, |base, name| base.join(name));
    Ok(normalized)
}

fn canonicalize_allow_missing(path: &Path) -> Result<PathBuf, GitError> {
    let mut current = path.to_path_buf();
    let mut missing = Vec::new();
    loop {
        match fs::canonicalize(&current) {
            Ok(canonical) => {
                missing.reverse();
                return Ok(missing
                    .into_iter()
                    .fold(canonical, |base: PathBuf, name: OsString| base.join(name)));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = current.file_name() else {
                    return Err(GitError::RepositoryUnavailable(
                        path.to_path_buf(),
                        "target has no existing ancestor".into(),
                    ));
                };
                missing.push(name.to_os_string());
                let Some(parent) = current.parent() else {
                    return Err(GitError::RepositoryUnavailable(
                        path.to_path_buf(),
                        "target has no existing ancestor".into(),
                    ));
                };
                current = parent.to_path_buf();
            }
            Err(error) => {
                return Err(GitError::RepositoryUnavailable(current, error.to_string()));
            }
        }
    }
}

fn parse_conflict_paths(stderr: &[u8]) -> Vec<PathBuf> {
    let text = String::from_utf8_lossy(stderr);
    let mut in_paths = false;
    let mut paths = BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("would be overwritten by checkout")
            || trimmed.contains("would be overwritten by switch")
        {
            in_paths = true;
            continue;
        }
        if in_paths
            && (trimmed.is_empty()
                || trimmed.starts_with("Please ")
                || trimmed.starts_with("Aborting"))
        {
            in_paths = false;
            continue;
        }
        if in_paths && !trimmed.starts_with("error:") {
            paths.insert(PathBuf::from(trimmed));
        }
    }
    paths.into_iter().collect()
}

fn safe_message(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr);
    let trimmed = message.trim();
    if trimmed.is_empty() {
        "Git operation could not be completed".into()
    } else {
        trimmed.chars().take(8_192).collect()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    encoded
}
