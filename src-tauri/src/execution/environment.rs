use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

use crate::security::gateway::ExecutionPermit;

const MAX_IDENTIFIER_BYTES: usize = 160;
const MAX_ARGUMENTS: usize = 256;
const MAX_ARGUMENT_BYTES: usize = 32 * 1024;
const MAX_WRITE_BYTES: usize = 16 * 1024 * 1024;
const MAX_WORKER_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionEnvironmentKind {
    Local,
    Docker,
    Ssh,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEnvironmentIdentity {
    pub kind: ExecutionEnvironmentKind,
    pub environment_id: String,
    pub generation: u64,
}

impl ExecutionEnvironmentIdentity {
    pub fn new(
        kind: ExecutionEnvironmentKind,
        environment_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, ExecutionError> {
        let environment_id = environment_id.into();
        validate_identifier("environment id", &environment_id)?;
        if generation == 0 {
            return Err(ExecutionError::InvalidRequest(
                "environment generation must be non-zero".into(),
            ));
        }
        Ok(Self {
            kind,
            environment_id,
            generation,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FilesystemObjectKind {
    Directory,
    File,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemObjectIdentity {
    pub device: u64,
    pub inode: u64,
    pub kind: FilesystemObjectKind,
}

impl FilesystemObjectIdentity {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        let kind = if metadata.is_dir() {
            FilesystemObjectKind::Directory
        } else if metadata.is_file() {
            FilesystemObjectKind::File
        } else {
            FilesystemObjectKind::Other
        };
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            kind,
        }
    }
}

/// One picker- or policy-trusted Project root, resolved to a physical identity.
///
/// The root and every resolved target are revalidated immediately before an
/// execution worker is invoked. This does not replace descriptor-relative I/O
/// in a facility implementation, but provides the identity needed for that
/// final TOCTOU check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedProjectRoot {
    canonical_root: PathBuf,
    identity: FilesystemObjectIdentity,
}

impl TrustedProjectRoot {
    pub fn open(path: &Path) -> Result<Self, ExecutionError> {
        let link_metadata = fs::symlink_metadata(path).map_err(|error| {
            ExecutionError::TrustedRootUnavailable(path.to_path_buf(), error.to_string())
        })?;
        if link_metadata.file_type().is_symlink() || !link_metadata.is_dir() {
            return Err(ExecutionError::UntrustedRoot(
                "trusted Project root must be a physical directory".into(),
            ));
        }
        let canonical_root = fs::canonicalize(path).map_err(|error| {
            ExecutionError::TrustedRootUnavailable(path.to_path_buf(), error.to_string())
        })?;
        let metadata = fs::metadata(&canonical_root).map_err(|error| {
            ExecutionError::TrustedRootUnavailable(canonical_root.clone(), error.to_string())
        })?;
        Ok(Self {
            canonical_root,
            identity: FilesystemObjectIdentity::from_metadata(&metadata),
        })
    }

    pub fn canonical_root(&self) -> &Path {
        &self.canonical_root
    }

    pub fn identity(&self) -> &FilesystemObjectIdentity {
        &self.identity
    }

    pub fn revalidate(&self) -> Result<(), ExecutionError> {
        let link_metadata = fs::symlink_metadata(&self.canonical_root).map_err(|_| {
            ExecutionError::StaleTarget("trusted Project root is unavailable".into())
        })?;
        if link_metadata.file_type().is_symlink() || !link_metadata.is_dir() {
            return Err(ExecutionError::StaleTarget(
                "trusted Project root changed type".into(),
            ));
        }
        let metadata = fs::metadata(&self.canonical_root)
            .map_err(|_| ExecutionError::StaleTarget("trusted Project root changed".into()))?;
        if FilesystemObjectIdentity::from_metadata(&metadata) != self.identity {
            return Err(ExecutionError::StaleTarget(
                "trusted Project root identity changed".into(),
            ));
        }
        Ok(())
    }

    pub fn resolve_existing(
        &self,
        relative_path: &Path,
    ) -> Result<ResolvedTargetIdentity, ExecutionError> {
        self.resolve(relative_path, TargetResolution::Existing)
    }

    pub fn resolve_for_create(
        &self,
        relative_path: &Path,
    ) -> Result<ResolvedTargetIdentity, ExecutionError> {
        self.resolve(relative_path, TargetResolution::Create)
    }

    fn resolve(
        &self,
        relative_path: &Path,
        resolution: TargetResolution,
    ) -> Result<ResolvedTargetIdentity, ExecutionError> {
        self.revalidate()?;
        validate_relative_path(relative_path)?;
        let requested_path = self.canonical_root.join(relative_path);
        let (resolved_path, anchor_path, target_identity) = match resolution {
            TargetResolution::Existing => {
                let resolved_path = fs::canonicalize(&requested_path).map_err(|error| {
                    ExecutionError::TargetUnavailable(requested_path.clone(), error.to_string())
                })?;
                ensure_under_root(&self.canonical_root, &resolved_path)?;
                let metadata = fs::metadata(&resolved_path).map_err(|error| {
                    ExecutionError::TargetUnavailable(resolved_path.clone(), error.to_string())
                })?;
                (
                    resolved_path.clone(),
                    resolved_path,
                    Some(FilesystemObjectIdentity::from_metadata(&metadata)),
                )
            }
            TargetResolution::Create => {
                if fs::symlink_metadata(&requested_path).is_ok() {
                    return Err(ExecutionError::InvalidRequest(
                        "create target already exists".into(),
                    ));
                }
                let (existing_ancestor, missing_suffix) =
                    nearest_existing_ancestor(&requested_path)?;
                let anchor_path = fs::canonicalize(&existing_ancestor).map_err(|error| {
                    ExecutionError::TargetUnavailable(existing_ancestor, error.to_string())
                })?;
                ensure_under_root(&self.canonical_root, &anchor_path)?;
                let resolved_path = missing_suffix
                    .iter()
                    .fold(anchor_path.clone(), |path, component| path.join(component));
                (resolved_path, anchor_path, None)
            }
        };
        let anchor_metadata = fs::metadata(&anchor_path).map_err(|error| {
            ExecutionError::TargetUnavailable(anchor_path.clone(), error.to_string())
        })?;
        let traversed_symlink = requested_path != resolved_path;

        Ok(ResolvedTargetIdentity {
            trusted_root: self.clone(),
            relative_path: relative_path.to_path_buf(),
            requested_path,
            resolved_path,
            anchor_path,
            anchor_identity: FilesystemObjectIdentity::from_metadata(&anchor_metadata),
            target_identity,
            resolution,
            traversed_symlink,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum TargetResolution {
    Existing,
    Create,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedTargetIdentity {
    trusted_root: TrustedProjectRoot,
    relative_path: PathBuf,
    requested_path: PathBuf,
    resolved_path: PathBuf,
    anchor_path: PathBuf,
    anchor_identity: FilesystemObjectIdentity,
    target_identity: Option<FilesystemObjectIdentity>,
    resolution: TargetResolution,
    traversed_symlink: bool,
}

impl ResolvedTargetIdentity {
    pub fn trusted_root(&self) -> &TrustedProjectRoot {
        &self.trusted_root
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn resolved_path(&self) -> &Path {
        &self.resolved_path
    }

    pub fn traversed_symlink(&self) -> bool {
        self.traversed_symlink
    }

    pub fn revalidate(&self) -> Result<(), ExecutionError> {
        self.trusted_root.revalidate()?;
        let anchor = fs::metadata(&self.anchor_path)
            .map_err(|_| ExecutionError::StaleTarget("target anchor is unavailable".into()))?;
        if FilesystemObjectIdentity::from_metadata(&anchor) != self.anchor_identity {
            return Err(ExecutionError::StaleTarget(
                "target anchor identity changed".into(),
            ));
        }

        match self.resolution {
            TargetResolution::Existing => {
                let current = fs::canonicalize(&self.requested_path)
                    .map_err(|_| ExecutionError::StaleTarget("target is unavailable".into()))?;
                ensure_under_root(self.trusted_root.canonical_root(), &current)?;
                if current != self.resolved_path {
                    return Err(ExecutionError::StaleTarget(
                        "target path now resolves to a different object".into(),
                    ));
                }
                let metadata = fs::metadata(&current)
                    .map_err(|_| ExecutionError::StaleTarget("target is unavailable".into()))?;
                if Some(FilesystemObjectIdentity::from_metadata(&metadata)) != self.target_identity
                {
                    return Err(ExecutionError::StaleTarget(
                        "target identity changed".into(),
                    ));
                }
            }
            TargetResolution::Create => {
                if fs::symlink_metadata(&self.requested_path).is_ok() {
                    return Err(ExecutionError::StaleTarget(
                        "create target appeared after authorization".into(),
                    ));
                }
                let (existing_ancestor, suffix) = nearest_existing_ancestor(&self.requested_path)?;
                let current_anchor = fs::canonicalize(existing_ancestor).map_err(|_| {
                    ExecutionError::StaleTarget("create target anchor is unavailable".into())
                })?;
                let current_path = suffix
                    .iter()
                    .fold(current_anchor.clone(), |path, component| {
                        path.join(component)
                    });
                ensure_under_root(self.trusted_root.canonical_root(), &current_path)?;
                if current_anchor != self.anchor_path || current_path != self.resolved_path {
                    return Err(ExecutionError::StaleTarget(
                        "create target path changed after authorization".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExactExecutionEffect {
    ReadFile,
    WriteFile {
        bytes: Vec<u8>,
    },
    Command {
        program: String,
        arguments: Vec<String>,
    },
}

impl ExactExecutionEffect {
    fn validate(&self) -> Result<(), ExecutionError> {
        match self {
            Self::ReadFile => Ok(()),
            Self::WriteFile { bytes } if bytes.len() <= MAX_WRITE_BYTES => Ok(()),
            Self::WriteFile { .. } => Err(ExecutionError::InvalidRequest(
                "write payload exceeds its byte limit".into(),
            )),
            Self::Command { program, arguments } => {
                if !program.starts_with('/')
                    || program.len() > MAX_ARGUMENT_BYTES
                    || program.split('/').any(|component| component == "..")
                    || program.chars().any(char::is_control)
                {
                    return Err(ExecutionError::InvalidRequest(
                        "command program must be an absolute normalized path".into(),
                    ));
                }
                if arguments.len() > MAX_ARGUMENTS
                    || arguments.iter().map(String::len).sum::<usize>() > MAX_ARGUMENT_BYTES
                    || arguments
                        .iter()
                        .any(|argument| argument.chars().any(|character| character == '\0'))
                {
                    return Err(ExecutionError::InvalidRequest(
                        "command arguments exceed their bounds".into(),
                    ));
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CredentialReference(String);

impl CredentialReference {
    pub fn new(reference: impl Into<String>) -> Result<Self, ExecutionError> {
        let reference = reference.into();
        let opaque_id = reference.strip_prefix("credential://");
        if reference.len() > 512
            || !opaque_id.is_some_and(|value| {
                !value.is_empty()
                    && value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':' | b'@')
                    })
            })
        {
            return Err(ExecutionError::InvalidRequest(
                "credential reference is invalid".into(),
            ));
        }
        Ok(Self(reference))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Bridges the credential vault's opaque product reference into the
    /// execution envelope without exposing or transforming secret material.
    pub fn from_product_reference(reference: &str) -> Result<Self, ExecutionError> {
        Self::new(format!("credential://{reference}"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactExecutionRequest {
    pub request_id: String,
    pub workspace_id: String,
    pub session_id: String,
    pub runtime_id: String,
    pub process_generation: u64,
    pub environment: ExecutionEnvironmentIdentity,
    pub target: ResolvedTargetIdentity,
    pub effect: ExactExecutionEffect,
    pub credential_reference: Option<CredentialReference>,
}

impl ExactExecutionRequest {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        validate_identifier("request id", &self.request_id)?;
        validate_identifier("workspace id", &self.workspace_id)?;
        validate_identifier("session id", &self.session_id)?;
        validate_identifier("runtime id", &self.runtime_id)?;
        if self.process_generation == 0 {
            return Err(ExecutionError::InvalidRequest(
                "process generation must be non-zero".into(),
            ));
        }
        self.effect.validate()?;
        match (
            &self.effect,
            self.target.resolution,
            &self.target.target_identity,
        ) {
            (
                ExactExecutionEffect::ReadFile,
                TargetResolution::Existing,
                Some(FilesystemObjectIdentity {
                    kind: FilesystemObjectKind::File,
                    ..
                }),
            )
            | (
                ExactExecutionEffect::WriteFile { .. },
                TargetResolution::Existing,
                Some(FilesystemObjectIdentity {
                    kind: FilesystemObjectKind::File,
                    ..
                }),
            )
            | (ExactExecutionEffect::WriteFile { .. }, TargetResolution::Create, None)
            | (
                ExactExecutionEffect::Command { .. },
                TargetResolution::Existing,
                Some(FilesystemObjectIdentity {
                    kind: FilesystemObjectKind::Directory,
                    ..
                }),
            ) => Ok(()),
            _ => Err(ExecutionError::InvalidRequest(
                "execution effect does not match the resolved target type".into(),
            )),
        }
    }

    /// Stable, opaque binding placed in the canonical Action Gateway request.
    /// It commits to the resolved physical target and every executor input
    /// without persisting any of those raw fields in the security journal.
    pub fn authorization_binding_sha256(&self) -> Result<String, ExecutionError> {
        self.validate()?;
        let binding = ExecutionRequestBinding {
            request_id: &self.request_id,
            workspace_id: &self.workspace_id,
            session_id: &self.session_id,
            runtime_id: &self.runtime_id,
            process_generation: self.process_generation,
            environment: &self.environment,
            trusted_root_identity: self.target.trusted_root().identity(),
            relative_target: portable_relative_path(self.target.relative_path())?,
            resolved_target: self.target.resolved_path().to_string_lossy(),
            resolution: self.target.resolution,
            anchor_identity: &self.target.anchor_identity,
            target_identity: self.target.target_identity.as_ref(),
            effect: &self.effect,
            credential_reference: self.credential_reference.as_ref(),
        };
        let bytes = serde_json::to_vec(&binding)
            .map_err(|error| ExecutionError::InvalidRequest(error.to_string()))?;
        Ok(format!("sha256:{}", sha256_hex(&bytes)))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionRequestBinding<'a> {
    request_id: &'a str,
    workspace_id: &'a str,
    session_id: &'a str,
    runtime_id: &'a str,
    process_generation: u64,
    environment: &'a ExecutionEnvironmentIdentity,
    trusted_root_identity: &'a FilesystemObjectIdentity,
    relative_target: String,
    resolved_target: std::borrow::Cow<'a, str>,
    resolution: TargetResolution,
    anchor_identity: &'a FilesystemObjectIdentity,
    target_identity: Option<&'a FilesystemObjectIdentity>,
    effect: &'a ExactExecutionEffect,
    credential_reference: Option<&'a CredentialReference>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExactExecutionGrant {
    authorization_id: String,
    request: ExactExecutionRequest,
}

impl ExactExecutionGrant {
    fn from_gateway(
        authorization_id: &str,
        request: ExactExecutionRequest,
    ) -> Result<Self, ExecutionError> {
        validate_identifier("authorization id", authorization_id)?;
        request.validate()?;
        Ok(Self {
            authorization_id: authorization_id.into(),
            request,
        })
    }

    pub fn authorization_id(&self) -> &str {
        &self.authorization_id
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecutionAuthorization {
    Allowed(Box<ExactExecutionGrant>),
    Denied { reason: String },
}

impl ExecutionAuthorization {
    /// The only production constructor for an allowed executor grant. A
    /// consumed permit cannot be built outside the authorization state machine.
    pub fn from_gateway(
        permit: ExecutionPermit,
        request: ExactExecutionRequest,
    ) -> Result<Self, ExecutionError> {
        let action = permit.action();
        let expected_binding = request.authorization_binding_sha256()?;
        let supplied_binding = action
            .arguments
            .get("executionRequestSha256")
            .and_then(serde_json::Value::as_str);
        let exact_arguments = action.arguments.as_object().is_some_and(|arguments| {
            arguments.len() == 1 && arguments.contains_key("executionRequestSha256")
        });
        if action.tool != "c4os.execution"
            || action.requested_authority.len() != 1
            || !action.requested_authority.contains("execution.execute")
            || !exact_arguments
            || action.action_id != request.request_id
            || action.workspace_id != request.workspace_id
            || action.session_id != request.session_id
            || action.runtime_id != request.runtime_id
            || action.environment_id != request.environment.environment_id
            || action.process_generation != request.process_generation
            || supplied_binding != Some(expected_binding.as_str())
        {
            return Err(ExecutionError::AuthorizationMismatch(
                "Action Gateway permit does not bind the exact execution request".into(),
            ));
        }
        Ok(Self::Allowed(Box::new(ExactExecutionGrant::from_gateway(
            permit.authorization_id(),
            request,
        )?)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockerExecutionProfile {
    pub image: String,
    pub docker_program: PathBuf,
}

impl DockerExecutionProfile {
    pub fn validate(&self) -> Result<(), ExecutionError> {
        let Some((name, digest)) = self.image.rsplit_once("@sha256:") else {
            return Err(ExecutionError::EnvironmentGated(
                "Docker image must be pinned by sha256 digest".into(),
            ));
        };
        if name.is_empty()
            || name.starts_with('-')
            || name.chars().any(char::is_whitespace)
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ExecutionError::EnvironmentGated(
                "Docker image digest is invalid".into(),
            ));
        }
        if !self.docker_program.is_absolute() {
            return Err(ExecutionError::EnvironmentGated(
                "Docker program must use an absolute trusted path".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SshProfileValidation {
    Pending,
    Validated {
        environment_generation: u64,
        host_key_fingerprint: String,
        filesystem_probe: String,
        shell_probe: String,
        cancellation_probe: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedSshExecutionProfile {
    pub alias: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub known_hosts_file: PathBuf,
    pub ssh_program: PathBuf,
    pub remote_worker_program: String,
    pub validation: SshProfileValidation,
}

impl NamedSshExecutionProfile {
    fn validate_for(&self, identity: &ExecutionEnvironmentIdentity) -> Result<(), ExecutionError> {
        validate_identifier("SSH alias", &self.alias)?;
        if !valid_ssh_component(&self.host)
            || self.user.is_empty()
            || self.user.starts_with('-')
            || !self
                .user
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
            || self.port == 0
            || !self.known_hosts_file.is_absolute()
            || !self.ssh_program.is_absolute()
            || !valid_remote_program(&self.remote_worker_program)
        {
            return Err(ExecutionError::EnvironmentGated(
                "named SSH profile is incomplete".into(),
            ));
        }
        let SshProfileValidation::Validated {
            environment_generation,
            host_key_fingerprint,
            filesystem_probe,
            shell_probe,
            cancellation_probe,
        } = &self.validation
        else {
            return Err(ExecutionError::EnvironmentGated(format!(
                "named SSH profile {} has not passed target-specific validation",
                self.alias
            )));
        };
        if *environment_generation != identity.generation
            || [
                host_key_fingerprint,
                filesystem_probe,
                shell_probe,
                cancellation_probe,
            ]
            .iter()
            .any(|value| value.is_empty())
        {
            return Err(ExecutionError::EnvironmentGated(format!(
                "named SSH profile {} validation is stale or incomplete",
                self.alias
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionEnvironmentDriver {
    Local { worker_program: PathBuf },
    Docker(DockerExecutionProfile),
    Ssh(Box<NamedSshExecutionProfile>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEnvironment {
    pub identity: ExecutionEnvironmentIdentity,
    pub runtime_workspace_root: String,
    pub driver: ExecutionEnvironmentDriver,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerExecutionEnvelope {
    request_id: String,
    workspace_id: String,
    session_id: String,
    runtime_id: String,
    process_generation: u64,
    environment: ExecutionEnvironmentIdentity,
    workspace_relative_target: String,
    runtime_target: String,
    effect: ExactExecutionEffect,
    credential_reference: Option<CredentialReference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerInvocation {
    pub program: OsString,
    pub arguments: Vec<OsString>,
    pub stdin: Vec<u8>,
    pub expected_request_id: String,
    pub expected_request_sha256: String,
    pub expected_environment: ExecutionEnvironmentIdentity,
    pub expected_runtime_target: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerExecutionReport {
    pub request_id: String,
    pub request_sha256: String,
    pub environment: ExecutionEnvironmentIdentity,
    pub runtime_target: String,
    pub exit_code: i32,
    pub bounded_stdout: Vec<u8>,
    pub bounded_stderr: Vec<u8>,
}

pub trait ExecutionCommandRunner {
    fn run(
        &mut self,
        invocation: &WorkerInvocation,
    ) -> Result<WorkerExecutionReport, ExecutionError>;
}

impl ExecutionEnvironment {
    pub fn dispatch<R: ExecutionCommandRunner>(
        &self,
        request: ExactExecutionRequest,
        authorization: ExecutionAuthorization,
        runner: &mut R,
    ) -> Result<WorkerExecutionReport, ExecutionError> {
        let grant = match authorization {
            ExecutionAuthorization::Denied { reason } => {
                return Err(ExecutionError::Denied(reason));
            }
            ExecutionAuthorization::Allowed(grant) => grant,
        };
        request.validate()?;
        if grant.request != request {
            return Err(ExecutionError::AuthorizationMismatch(
                "authorization does not bind the exact execution request".into(),
            ));
        }
        if request.environment != self.identity {
            return Err(ExecutionError::EnvironmentSubstitution);
        }
        request.target.revalidate()?;
        let invocation = self.build_invocation(&request)?;
        let report = runner.run(&invocation)?;
        if report.bounded_stdout.len() > MAX_WORKER_OUTPUT_BYTES
            || report.bounded_stderr.len() > MAX_WORKER_OUTPUT_BYTES
        {
            return Err(ExecutionError::Worker(
                "execution worker output exceeded its bound".into(),
            ));
        }
        if report.request_id != invocation.expected_request_id
            || report.request_sha256 != invocation.expected_request_sha256
            || report.environment != invocation.expected_environment
            || report.runtime_target != invocation.expected_runtime_target
        {
            return Err(ExecutionError::WorkerSubstitution);
        }
        Ok(report)
    }

    fn build_invocation(
        &self,
        request: &ExactExecutionRequest,
    ) -> Result<WorkerInvocation, ExecutionError> {
        if self.identity.kind
            != match &self.driver {
                ExecutionEnvironmentDriver::Local { .. } => ExecutionEnvironmentKind::Local,
                ExecutionEnvironmentDriver::Docker(_) => ExecutionEnvironmentKind::Docker,
                ExecutionEnvironmentDriver::Ssh(_) => ExecutionEnvironmentKind::Ssh,
            }
        {
            return Err(ExecutionError::EnvironmentSubstitution);
        }
        let relative_target = portable_relative_path(request.target.relative_path())?;
        let runtime_target = match &self.driver {
            ExecutionEnvironmentDriver::Local { .. } => request
                .target
                .resolved_path()
                .to_string_lossy()
                .into_owned(),
            ExecutionEnvironmentDriver::Docker(_) | ExecutionEnvironmentDriver::Ssh(_) => {
                join_runtime_path(&self.runtime_workspace_root, &relative_target)?
            }
        };
        let envelope = WorkerExecutionEnvelope {
            request_id: request.request_id.clone(),
            workspace_id: request.workspace_id.clone(),
            session_id: request.session_id.clone(),
            runtime_id: request.runtime_id.clone(),
            process_generation: request.process_generation,
            environment: request.environment.clone(),
            workspace_relative_target: relative_target,
            runtime_target: runtime_target.clone(),
            effect: request.effect.clone(),
            credential_reference: request.credential_reference.clone(),
        };
        let stdin = serde_json::to_vec(&envelope)
            .map_err(|error| ExecutionError::InvalidRequest(error.to_string()))?;
        let request_sha256 = sha256_hex(&stdin);

        let (program, arguments) = match &self.driver {
            ExecutionEnvironmentDriver::Local { worker_program } => {
                if !worker_program.is_absolute() {
                    return Err(ExecutionError::EnvironmentGated(
                        "Local worker must use an absolute trusted path".into(),
                    ));
                }
                (
                    worker_program.as_os_str().to_os_string(),
                    vec![OsString::from("--transport"), OsString::from("local")],
                )
            }
            ExecutionEnvironmentDriver::Docker(profile) => {
                profile.validate()?;
                let source = request.target.trusted_root().canonical_root();
                if source.as_os_str().to_string_lossy().contains(',')
                    || self.runtime_workspace_root.contains(',')
                {
                    return Err(ExecutionError::EnvironmentGated(
                        "Docker bind-mount paths cannot contain commas".into(),
                    ));
                }
                let mount = format!(
                    "type=bind,source={},target={}",
                    source.display(),
                    self.runtime_workspace_root
                );
                (
                    profile.docker_program.as_os_str().to_os_string(),
                    vec![
                        OsString::from("run"),
                        OsString::from("--rm"),
                        OsString::from("--network"),
                        OsString::from("none"),
                        OsString::from("--mount"),
                        OsString::from(mount),
                        OsString::from("--interactive"),
                        OsString::from(&profile.image),
                        OsString::from("c4os-execution-worker"),
                        OsString::from("--transport"),
                        OsString::from("docker"),
                    ],
                )
            }
            ExecutionEnvironmentDriver::Ssh(profile) => {
                profile.validate_for(&self.identity)?;
                (
                    profile.ssh_program.as_os_str().to_os_string(),
                    vec![
                        OsString::from("-F"),
                        OsString::from("/dev/null"),
                        OsString::from("-p"),
                        OsString::from(profile.port.to_string()),
                        OsString::from("-o"),
                        OsString::from("BatchMode=yes"),
                        OsString::from("-o"),
                        OsString::from("IdentitiesOnly=yes"),
                        OsString::from("-o"),
                        OsString::from("StrictHostKeyChecking=yes"),
                        OsString::from("-o"),
                        OsString::from(format!(
                            "UserKnownHostsFile={}",
                            profile.known_hosts_file.display()
                        )),
                        OsString::from(format!("{}@{}", profile.user, profile.host)),
                        OsString::from(&profile.remote_worker_program),
                        OsString::from("--transport"),
                        OsString::from("ssh"),
                    ],
                )
            }
        };

        Ok(WorkerInvocation {
            program,
            arguments,
            stdin,
            expected_request_id: request.request_id.clone(),
            expected_request_sha256: request_sha256,
            expected_environment: self.identity.clone(),
            expected_runtime_target: runtime_target,
        })
    }
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("invalid execution request: {0}")]
    InvalidRequest(String),
    #[error("trusted root is unavailable at {0}: {1}")]
    TrustedRootUnavailable(PathBuf, String),
    #[error("trusted root rejected: {0}")]
    UntrustedRoot(String),
    #[error("target is unavailable at {0}: {1}")]
    TargetUnavailable(PathBuf, String),
    #[error("target escapes the trusted Project root")]
    TargetEscapesRoot,
    #[error("target state changed after authorization: {0}")]
    StaleTarget(String),
    #[error("execution was denied before dispatch: {0}")]
    Denied(String),
    #[error("authorization mismatch: {0}")]
    AuthorizationMismatch(String),
    #[error("execution environment substitution was rejected")]
    EnvironmentSubstitution,
    #[error("execution worker substituted the target or environment")]
    WorkerSubstitution,
    #[error("execution environment is gated: {0}")]
    EnvironmentGated(String),
    #[error("execution worker failed: {0}")]
    Worker(String),
}

fn validate_identifier(kind: &str, value: &str) -> Result<(), ExecutionError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_IDENTIFIER_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        });
    if valid {
        Ok(())
    } else {
        Err(ExecutionError::InvalidRequest(format!("invalid {kind}")))
    }
}

fn validate_relative_path(path: &Path) -> Result<(), ExecutionError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ExecutionError::InvalidRequest(
            "target must be a normalized Project-relative path".into(),
        ));
    }
    Ok(())
}

fn ensure_under_root(root: &Path, target: &Path) -> Result<(), ExecutionError> {
    if target.starts_with(root) {
        Ok(())
    } else {
        Err(ExecutionError::TargetEscapesRoot)
    }
}

fn nearest_existing_ancestor(path: &Path) -> Result<(PathBuf, Vec<OsString>), ExecutionError> {
    let mut current = path.to_path_buf();
    let mut missing = Vec::new();
    loop {
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    let canonical = fs::canonicalize(&current).map_err(|error| {
                        ExecutionError::TargetUnavailable(current.clone(), error.to_string())
                    })?;
                    return Ok((canonical, missing));
                }
                if !metadata.is_dir() {
                    return Err(ExecutionError::InvalidRequest(
                        "create target ancestor is not a directory".into(),
                    ));
                }
                missing.reverse();
                return Ok((current, missing));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(name) = current.file_name() else {
                    return Err(ExecutionError::TargetUnavailable(
                        path.to_path_buf(),
                        "no existing target ancestor".into(),
                    ));
                };
                missing.push(name.to_os_string());
                let Some(parent) = current.parent() else {
                    return Err(ExecutionError::TargetUnavailable(
                        path.to_path_buf(),
                        "no existing target ancestor".into(),
                    ));
                };
                current = parent.to_path_buf();
            }
            Err(error) => {
                return Err(ExecutionError::TargetUnavailable(
                    current,
                    error.to_string(),
                ));
            }
        }
    }
}

fn portable_relative_path(path: &Path) -> Result<String, ExecutionError> {
    validate_relative_path(path)?;
    let components =
        path.components()
            .map(|component| match component {
                Component::Normal(value) => value.to_str().map(str::to_owned).ok_or_else(|| {
                    ExecutionError::InvalidRequest("target path is not UTF-8".into())
                }),
                _ => unreachable!("path was validated"),
            })
            .collect::<Result<Vec<_>, _>>()?;
    Ok(components.join("/"))
}

fn join_runtime_path(root: &str, relative: &str) -> Result<String, ExecutionError> {
    if !root.starts_with('/')
        || root.ends_with('/')
        || root.contains("/../")
        || root.chars().any(char::is_control)
    {
        return Err(ExecutionError::EnvironmentGated(
            "runtime Workspace root must be an absolute normalized path".into(),
        ));
    }
    Ok(format!("{root}/{relative}"))
}

fn valid_ssh_component(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'[' | b']')
        })
}

fn valid_remote_program(value: &str) -> bool {
    value.starts_with('/')
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.'))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}
