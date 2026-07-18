use c4os_lib::core::database::{DatabaseActor, DatabaseDescriptor};
use c4os_lib::execution::*;
use c4os_lib::security::authorization::{
    CANONICAL_ACTION_SCHEMA_VERSION, CanonicalAction, CanonicalRisk, LiveAuthorityState,
};
use c4os_lib::security::gateway::{
    ActionGateway, GatewayProposal, NormalizedActionResult, NormalizedActionStatus,
};
use c4os_lib::security::policy::{
    ActionEffect, ActionFacts, ActionInitiator, ActionRequestOrigin, ActionReversibility,
    ActionScope, ActionSensitivity, ActionSurface, ApprovalPreset, ClassificationConfidence,
    PolicyConfiguration, RepositoryState,
};
use serde_json::{Value, json};
use std::collections::{BTreeSet, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

const HEAD: &str = "1111111111111111111111111111111111111111";
const TARGET: &str = "2222222222222222222222222222222222222222";
const CHANGE_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[derive(Default)]
struct FakeExecutionRunner {
    calls: Vec<WorkerInvocation>,
    substitute_environment: Option<ExecutionEnvironmentIdentity>,
    substitute_target: Option<String>,
}

impl ExecutionCommandRunner for FakeExecutionRunner {
    fn run(
        &mut self,
        invocation: &WorkerInvocation,
    ) -> Result<WorkerExecutionReport, ExecutionError> {
        self.calls.push(invocation.clone());
        Ok(WorkerExecutionReport {
            request_id: invocation.expected_request_id.clone(),
            request_sha256: invocation.expected_request_sha256.clone(),
            environment: self
                .substitute_environment
                .clone()
                .unwrap_or_else(|| invocation.expected_environment.clone()),
            runtime_target: self
                .substitute_target
                .clone()
                .unwrap_or_else(|| invocation.expected_runtime_target.clone()),
            exit_code: 0,
            bounded_stdout: Vec::new(),
            bounded_stderr: Vec::new(),
        })
    }
}

fn local_identity(id: &str) -> ExecutionEnvironmentIdentity {
    ExecutionEnvironmentIdentity::new(ExecutionEnvironmentKind::Local, id, 1).unwrap()
}

fn docker_identity() -> ExecutionEnvironmentIdentity {
    ExecutionEnvironmentIdentity::new(ExecutionEnvironmentKind::Docker, "docker-1", 1).unwrap()
}

fn ssh_identity() -> ExecutionEnvironmentIdentity {
    ExecutionEnvironmentIdentity::new(ExecutionEnvironmentKind::Ssh, "ssh-1", 7).unwrap()
}

fn execution_fixture(identity: ExecutionEnvironmentIdentity) -> (TempDir, ExactExecutionRequest) {
    let temporary = tempfile::tempdir().unwrap();
    fs::write(temporary.path().join("notes.txt"), b"before").unwrap();
    let root = TrustedProjectRoot::open(temporary.path()).unwrap();
    let target = root.resolve_existing(Path::new("notes.txt")).unwrap();
    let request = ExactExecutionRequest {
        request_id: "request-1".into(),
        workspace_id: "workspace-1".into(),
        session_id: "session-1".into(),
        runtime_id: "runtime-1".into(),
        process_generation: 3,
        environment: identity,
        target,
        effect: ExactExecutionEffect::ReadFile,
        credential_reference: None,
    };
    (temporary, request)
}

fn action(
    action_id: &str,
    tool: &str,
    authority: &str,
    arguments: Value,
    context: ActionContext<'_>,
) -> CanonicalAction {
    CanonicalAction {
        schema_version: CANONICAL_ACTION_SCHEMA_VERSION,
        action_id: action_id.into(),
        tool_call_id: format!("tool-call-{action_id}"),
        tool: tool.into(),
        arguments,
        risk: CanonicalRisk::Medium,
        requested_authority: BTreeSet::from([authority.into()]),
        canonical_target: "opaque:executor-target".into(),
        target_version: "sha256:fixture".into(),
        workspace_id: context.workspace_id.into(),
        session_id: context.session_id.into(),
        run_id: "run-1".into(),
        runtime_id: context.runtime_id.into(),
        environment_id: context.environment_id.into(),
        process_generation: context.process_generation,
        configuration_version: 11,
        policy_version: 12,
        revocation_epoch: 13,
    }
}

struct ActionContext<'a> {
    workspace_id: &'a str,
    session_id: &'a str,
    runtime_id: &'a str,
    environment_id: &'a str,
    process_generation: u64,
}

fn live(action: &CanonicalAction) -> LiveAuthorityState {
    LiveAuthorityState {
        process_generation: action.process_generation,
        configuration_version: action.configuration_version,
        policy_version: action.policy_version,
        revocation_epoch: action.revocation_epoch,
    }
}

fn gateway_facts(action: &CanonicalAction, surface: ActionSurface) -> ActionFacts {
    ActionFacts {
        action_kind: "executor.request".into(),
        native_tool: action.tool.clone(),
        surface,
        effects: BTreeSet::from([ActionEffect::Read]),
        scope: ActionScope::Workspace,
        initiator: ActionInitiator::Agent,
        sensitivity: ActionSensitivity::Ordinary,
        reversibility: ActionReversibility::Reversible,
        confidence: ClassificationConfidence::Known,
        request_origin: ActionRequestOrigin::DirectUserEdit,
        repository_state: RepositoryState::VersionControlled,
        inside_active_project: true,
        canonical_target: action.canonical_target.clone(),
        workspace_id: action.workspace_id.clone(),
        session_id: action.session_id.clone(),
        runtime_id: action.runtime_id.clone(),
        environment_id: action.environment_id.clone(),
        plugin_or_mcp_id: None,
        target_resolved: true,
        authenticated: false,
        trusted_root: true,
        explicit_scope_grant: false,
        sandbox_allows: true,
        declaration_exceeded: false,
    }
}

fn gateway() -> (TempDir, ActionGateway) {
    let temporary = tempfile::tempdir().unwrap();
    let descriptor = DatabaseDescriptor::app(temporary.path());
    let (database, _) = DatabaseActor::start(descriptor).unwrap();
    let policy = PolicyConfiguration {
        preset: ApprovalPreset::ApproveForMe,
        ..PolicyConfiguration::default()
    };
    (temporary, ActionGateway::new(policy, Arc::new(database)))
}

fn success() -> NormalizedActionResult {
    NormalizedActionResult {
        status: NormalizedActionStatus::Succeeded,
        result_code: "ok".into(),
        exit_code: Some(0),
        changed_targets: Vec::new(),
        output_sha256: Some(format!("sha256:{}", "7".repeat(64))),
        completed_at_ms: 103,
    }
}

fn execution_authorization_from_gateway_action(
    request: &ExactExecutionRequest,
    arguments: Value,
) -> Result<ExecutionAuthorization, ExecutionError> {
    let action = action(
        &request.request_id,
        "c4os.execution",
        "execution.execute",
        arguments,
        ActionContext {
            workspace_id: &request.workspace_id,
            session_id: &request.session_id,
            runtime_id: &request.runtime_id,
            environment_id: &request.environment.environment_id,
            process_generation: request.process_generation,
        },
    );
    let facts = gateway_facts(&action, ActionSurface::Process);
    let (_temporary, mut gateway) = gateway();
    let token = match gateway.propose(&facts, action.clone(), 100).unwrap() {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected gateway authorization, found {other:?}"),
    };
    let mut authorization = None;
    gateway
        .execute(&token, &action, live(&action), None, 101, |permit| {
            authorization = Some(ExecutionAuthorization::from_gateway(
                permit,
                request.clone(),
            ));
            success()
        })
        .unwrap();
    authorization.expect("gateway effect closure must mint an execution authorization")
}

fn execution_authorization_with_binding(
    request: &ExactExecutionRequest,
    execution_request_sha256: String,
) -> Result<ExecutionAuthorization, ExecutionError> {
    execution_authorization_from_gateway_action(
        request,
        json!({"executionRequestSha256": execution_request_sha256}),
    )
}

fn execution_authorization(request: &ExactExecutionRequest) -> ExecutionAuthorization {
    execution_authorization_with_binding(request, request.authorization_binding_sha256().unwrap())
        .unwrap()
}

fn docker_environment(identity: ExecutionEnvironmentIdentity) -> ExecutionEnvironment {
    ExecutionEnvironment {
        identity,
        runtime_workspace_root: "/workspace".into(),
        driver: ExecutionEnvironmentDriver::Docker(DockerExecutionProfile {
            image: format!("c4os/worker@sha256:{}", "a".repeat(64)),
            docker_program: PathBuf::from("/usr/local/bin/docker"),
        }),
    }
}

fn ssh_profile(validation: SshProfileValidation) -> NamedSshExecutionProfile {
    NamedSshExecutionProfile {
        alias: "production-a".into(),
        host: "example.test".into(),
        port: 22,
        user: "c4os".into(),
        known_hosts_file: PathBuf::from("/private/tmp/c4os-known-hosts"),
        ssh_program: PathBuf::from("/usr/bin/ssh"),
        remote_worker_program: "/usr/local/libexec/c4os-execution-worker".into(),
        validation,
    }
}

#[test]
fn local_docker_and_validated_ssh_preserve_the_exact_environment_qualified_contract() {
    let environments = [
        ExecutionEnvironment {
            identity: local_identity("desktop"),
            runtime_workspace_root: "/unused-for-local".into(),
            driver: ExecutionEnvironmentDriver::Local {
                worker_program: PathBuf::from("/usr/local/libexec/c4os-execution-worker"),
            },
        },
        docker_environment(docker_identity()),
        ExecutionEnvironment {
            identity: ssh_identity(),
            runtime_workspace_root: "/srv/c4os/workspace".into(),
            driver: ExecutionEnvironmentDriver::Ssh(Box::new(ssh_profile(
                SshProfileValidation::Validated {
                    environment_generation: 7,
                    host_key_fingerprint: "SHA256:host".into(),
                    filesystem_probe: "fs-ok".into(),
                    shell_probe: "shell-ok".into(),
                    cancellation_probe: "cancel-ok".into(),
                },
            ))),
        },
    ];

    for environment in environments {
        let (_temporary, request) = execution_fixture(environment.identity.clone());
        let mut runner = FakeExecutionRunner::default();
        let report = environment
            .dispatch(
                request.clone(),
                execution_authorization(&request),
                &mut runner,
            )
            .unwrap();
        assert_eq!(report.environment, environment.identity);
        assert_eq!(runner.calls.len(), 1);
        assert_eq!(
            runner.calls[0].expected_runtime_target,
            match environment.identity.kind {
                ExecutionEnvironmentKind::Local => request
                    .target
                    .resolved_path()
                    .to_string_lossy()
                    .into_owned(),
                ExecutionEnvironmentKind::Docker => "/workspace/notes.txt".into(),
                ExecutionEnvironmentKind::Ssh => "/srv/c4os/workspace/notes.txt".into(),
            }
        );
        let serialized = String::from_utf8(runner.calls[0].stdin.clone()).unwrap();
        if environment.identity.kind != ExecutionEnvironmentKind::Local {
            assert!(!serialized.contains(temporary_root_fragment(&request).as_str()));
        }
    }
}

fn temporary_root_fragment(request: &ExactExecutionRequest) -> String {
    request
        .target
        .trusted_root()
        .canonical_root()
        .to_string_lossy()
        .into_owned()
}

#[test]
fn denial_mutation_and_environment_substitution_never_reach_the_worker() {
    let identity = local_identity("desktop");
    let environment = ExecutionEnvironment {
        identity: identity.clone(),
        runtime_workspace_root: "/unused".into(),
        driver: ExecutionEnvironmentDriver::Local {
            worker_program: PathBuf::from("/usr/local/libexec/c4os-execution-worker"),
        },
    };
    let (_temporary, request) = execution_fixture(identity.clone());
    let mut runner = FakeExecutionRunner::default();
    let denied = environment.dispatch(
        request.clone(),
        ExecutionAuthorization::Denied {
            reason: "policy".into(),
        },
        &mut runner,
    );
    assert!(matches!(denied, Err(ExecutionError::Denied(_))));
    assert!(runner.calls.is_empty());

    let authorization = execution_authorization(&request);
    let mut mutated = request.clone();
    mutated.effect = ExactExecutionEffect::WriteFile {
        bytes: b"changed".to_vec(),
    };
    assert!(matches!(
        environment.dispatch(mutated, authorization, &mut runner),
        Err(ExecutionError::AuthorizationMismatch(_))
    ));
    assert!(runner.calls.is_empty());

    let substituted_environment = ExecutionEnvironment {
        identity: local_identity("other-desktop"),
        runtime_workspace_root: "/unused".into(),
        driver: ExecutionEnvironmentDriver::Local {
            worker_program: PathBuf::from("/usr/local/libexec/c4os-execution-worker"),
        },
    };
    let authorization = execution_authorization(&request);
    assert!(matches!(
        substituted_environment.dispatch(request, authorization, &mut runner),
        Err(ExecutionError::EnvironmentSubstitution)
    ));
    assert!(runner.calls.is_empty());
}

#[test]
fn worker_substitution_is_detected_and_named_ssh_stays_gated() {
    let identity = docker_identity();
    let environment = docker_environment(identity.clone());
    let (_temporary, request) = execution_fixture(identity);
    let authorization = execution_authorization(&request);
    let mut runner = FakeExecutionRunner {
        substitute_target: Some("/workspace/other.txt".into()),
        ..Default::default()
    };
    assert!(matches!(
        environment.dispatch(request, authorization, &mut runner),
        Err(ExecutionError::WorkerSubstitution)
    ));

    let ssh_environment = ExecutionEnvironment {
        identity: ssh_identity(),
        runtime_workspace_root: "/srv/c4os/workspace".into(),
        driver: ExecutionEnvironmentDriver::Ssh(Box::new(ssh_profile(
            SshProfileValidation::Pending,
        ))),
    };
    let (_temporary, request) = execution_fixture(ssh_environment.identity.clone());
    let authorization = execution_authorization(&request);
    let mut runner = FakeExecutionRunner::default();
    assert!(matches!(
        ssh_environment.dispatch(request, authorization, &mut runner),
        Err(ExecutionError::EnvironmentGated(_))
    ));
    assert!(runner.calls.is_empty());
}

#[test]
fn only_opaque_credential_references_cross_the_worker_contract() {
    assert!(CredentialReference::new("ghp_raw-secret").is_err());
    assert!(CredentialReference::new("secret://raw-secret").is_err());
    let identity = docker_identity();
    let environment = docker_environment(identity.clone());
    let (_temporary, mut request) = execution_fixture(identity);
    request.credential_reference =
        Some(CredentialReference::new("credential://ssh/profile-1").unwrap());
    let authorization = execution_authorization(&request);
    let mut runner = FakeExecutionRunner::default();
    environment
        .dispatch(request, authorization, &mut runner)
        .unwrap();
    let invocation = &runner.calls[0];
    assert!(String::from_utf8_lossy(&invocation.stdin).contains("credential://ssh/profile-1"));
    assert!(
        !invocation
            .arguments
            .iter()
            .any(|argument| argument.to_string_lossy().contains("credential://"))
    );
}

#[test]
fn symlink_escape_and_post_authorization_target_changes_fail_closed() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.txt"), b"secret").unwrap();
    symlink(outside.path(), project.path().join("escape")).unwrap();
    let root = TrustedProjectRoot::open(project.path()).unwrap();
    assert!(matches!(
        root.resolve_existing(Path::new("escape/secret.txt")),
        Err(ExecutionError::TargetEscapesRoot)
    ));

    let create = root.resolve_for_create(Path::new("new.txt")).unwrap();
    fs::write(project.path().join("new.txt"), b"raced").unwrap();
    assert!(matches!(
        create.revalidate(),
        Err(ExecutionError::StaleTarget(_))
    ));
}

#[derive(Default)]
struct FakeGitRunner {
    outputs: VecDeque<Result<GitCommandOutput, GitError>>,
    calls: Vec<GitCommandInvocation>,
}

impl FakeGitRunner {
    fn with(outputs: impl IntoIterator<Item = GitCommandOutput>) -> Self {
        Self {
            outputs: outputs.into_iter().map(Ok).collect(),
            calls: Vec::new(),
        }
    }
}

impl GitCommandRunner for FakeGitRunner {
    fn run(&mut self, invocation: &GitCommandInvocation) -> Result<GitCommandOutput, GitError> {
        self.calls.push(invocation.clone());
        self.outputs
            .pop_front()
            .expect("fake Git output for every invocation")
    }
}

fn visible_repository() -> (TempDir, ActiveProjectRepository) {
    let temporary = tempfile::tempdir().unwrap();
    let root = TrustedProjectRoot::open(temporary.path()).unwrap();
    let mut runner = FakeGitRunner::with([GitCommandOutput::success(format!(
        "{}\n",
        temporary.path().display()
    ))]);
    let BranchControlVisibility::Visible(repository) =
        inspect_branch_control(root, Path::new("/usr/bin/git"), &mut runner).unwrap()
    else {
        panic!("fixture should be visible")
    };
    (temporary, repository)
}

fn state(dirty: &[u8]) -> GitStateVersion {
    GitStateVersion {
        head_oid: HEAD.into(),
        worktree_porcelain_v1_z: dirty.to_vec(),
    }
}

fn request(
    repository: &ActiveProjectRepository,
    dirty: &[u8],
    operation: GitBranchOperation,
) -> GitBranchRequest {
    GitBranchRequest {
        request_id: "git-request-1".into(),
        repository_root: repository.repository_root().to_path_buf(),
        expected_state: state(dirty),
        operation,
    }
}

fn git_authorization_with_binding(
    repository: &ActiveProjectRepository,
    request: &GitBranchRequest,
    git_request_sha256: String,
) -> Result<GitOperationAuthorization, GitError> {
    let action = action(
        &request.request_id,
        "c4os.git",
        "git.branch",
        json!({"gitRequestSha256": git_request_sha256}),
        ActionContext {
            workspace_id: "workspace-1",
            session_id: "session-1",
            runtime_id: "runtime-1",
            environment_id: "desktop",
            process_generation: 3,
        },
    );
    let facts = gateway_facts(&action, ActionSurface::Git);
    let (_temporary, mut gateway) = gateway();
    let token = match gateway.propose(&facts, action.clone(), 100).unwrap() {
        GatewayProposal::Authorized { token, .. } => token,
        other => panic!("expected gateway authorization, found {other:?}"),
    };
    let mut authorization = None;
    gateway
        .execute(&token, &action, live(&action), None, 101, |permit| {
            authorization = Some(GitOperationAuthorization::from_gateway(
                permit,
                repository,
                request.clone(),
            ));
            success()
        })
        .unwrap();
    authorization.expect("gateway effect closure must mint a Git authorization")
}

fn git_authorization(
    repository: &ActiveProjectRepository,
    request: &GitBranchRequest,
) -> GitOperationAuthorization {
    git_authorization_with_binding(
        repository,
        request,
        request.authorization_binding_sha256(repository).unwrap(),
    )
    .unwrap()
}

#[test]
fn mismatched_or_dual_digest_gateway_actions_cannot_mint_executor_grants() {
    let (_temporary, execution_request) = execution_fixture(local_identity("desktop"));
    assert!(matches!(
        execution_authorization_with_binding(
            &execution_request,
            format!("sha256:{}", "0".repeat(64))
        ),
        Err(ExecutionError::AuthorizationMismatch(_))
    ));
    assert!(matches!(
        execution_authorization_from_gateway_action(
            &execution_request,
            json!({
                "executionRequestSha256": execution_request
                    .authorization_binding_sha256()
                    .unwrap(),
                "gitRequestSha256": format!("sha256:{}", "1".repeat(64)),
            })
        ),
        Err(ExecutionError::AuthorizationMismatch(_))
    ));

    let (_temporary, repository) = visible_repository();
    let git_request = request(
        &repository,
        b"",
        GitBranchOperation::Create {
            branch: "feature/new".into(),
        },
    );
    assert!(matches!(
        git_authorization_with_binding(
            &repository,
            &git_request,
            format!("sha256:{}", "0".repeat(64))
        ),
        Err(GitError::AuthorizationMismatch)
    ));
}

#[test]
fn branch_control_is_visible_only_for_a_repository_exactly_scoped_to_the_project() {
    let project = tempfile::tempdir().unwrap();
    let root = TrustedProjectRoot::open(project.path()).unwrap();
    let mut non_repository =
        FakeGitRunner::with([GitCommandOutput::failure(128, b"not a repository".to_vec())]);
    assert!(matches!(
        inspect_branch_control(root.clone(), Path::new("/usr/bin/git"), &mut non_repository)
            .unwrap(),
        BranchControlVisibility::HiddenNonRepository
    ));

    let ancestor = project.path().parent().unwrap();
    let mut crossing = FakeGitRunner::with([GitCommandOutput::success(format!(
        "{}\n",
        ancestor.display()
    ))]);
    assert!(matches!(
        inspect_branch_control(root, Path::new("/usr/bin/git"), &mut crossing).unwrap(),
        BranchControlVisibility::HiddenRepositoryCrossesProjectBoundary { .. }
    ));

    let (_temporary, repository) = visible_repository();
    assert_eq!(
        repository.repository_root(),
        repository.project_root().canonical_root()
    );
}

#[test]
fn chat_write_matrix_honors_repository_project_and_explicit_ask_boundaries() {
    let (temporary, repository) = visible_repository();
    fs::write(temporary.path().join("tracked.txt"), b"content").unwrap();
    let visible = BranchControlVisibility::Visible(repository.clone());
    let inside = classify_project_target(
        repository.project_root(),
        &visible,
        Path::new("tracked.txt"),
    )
    .unwrap();
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Allow, inside.clone(), CHANGE_SHA)
            .unwrap(),
        ChatWriteDisposition::AllowWithoutAdditionalChangePrompt
    ));
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Ask, inside, CHANGE_SHA).unwrap(),
        ChatWriteDisposition::RequirePathAndChangeApproval { .. }
    ));

    let hidden = BranchControlVisibility::HiddenNonRepository;
    let non_repository =
        classify_project_target(repository.project_root(), &hidden, Path::new("tracked.txt"))
            .unwrap();
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Allow, non_repository, CHANGE_SHA)
            .unwrap(),
        ChatWriteDisposition::RequirePathAndChangeApproval { .. }
    ));

    let outside = tempfile::NamedTempFile::new().unwrap();
    let outside =
        classify_project_target(repository.project_root(), &visible, outside.path()).unwrap();
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Allow, outside, CHANGE_SHA).unwrap(),
        ChatWriteDisposition::RequirePathAndChangeApproval { .. }
    ));

    let denied = classify_project_target(
        repository.project_root(),
        &visible,
        Path::new("tracked.txt"),
    )
    .unwrap();
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Deny, denied, CHANGE_SHA).unwrap(),
        ChatWriteDisposition::Deny { .. }
    ));
}

#[test]
fn symlink_escape_is_denied_even_when_policy_would_otherwise_allow() {
    let (temporary, repository) = visible_repository();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("target.txt"), b"content").unwrap();
    symlink(outside.path(), temporary.path().join("escape")).unwrap();
    let visible = BranchControlVisibility::Visible(repository.clone());
    let classification = classify_project_target(
        repository.project_root(),
        &visible,
        Path::new("escape/target.txt"),
    )
    .unwrap();
    assert!(matches!(
        classification,
        ProjectTargetClassification::SymlinkEscape { .. }
    ));
    assert!(matches!(
        resolve_chat_write_disposition(EffectiveWritePolicy::Allow, classification, CHANGE_SHA)
            .unwrap(),
        ChatWriteDisposition::Deny { .. }
    ));
}

fn branch_outputs(
    dirty: &[u8],
    switch: GitCommandOutput,
    dirty_after: &[u8],
) -> Vec<GitCommandOutput> {
    let succeeded = switch.exit_code == 0;
    let mut outputs = vec![
        GitCommandOutput::success(format!("{HEAD}\n")),
        GitCommandOutput::success(dirty.to_vec()),
        GitCommandOutput::failure(1, Vec::new()),
        GitCommandOutput::success(Vec::new()),
        GitCommandOutput::success(format!("{TARGET}\n")),
        switch,
        GitCommandOutput::success(format!("{}\n", if succeeded { TARGET } else { HEAD })),
        GitCommandOutput::success(dirty_after.to_vec()),
    ];
    if succeeded {
        outputs.push(GitCommandOutput::success(b"feature/safe\n".to_vec()));
    }
    outputs
}

#[test]
fn brokered_safe_dirty_switch_preserves_state_and_never_runs_destructive_helpers() {
    let (_temporary, repository) = visible_repository();
    let dirty = b" M tracked.txt\0?? new.txt\0";
    let request = request(
        &repository,
        dirty,
        GitBranchOperation::Switch {
            branch: "feature/safe".into(),
            expected_target_oid: TARGET.into(),
        },
    );
    let authorization = git_authorization(&repository, &request);
    let mut runner = FakeGitRunner::with(branch_outputs(
        dirty,
        GitCommandOutput::success(Vec::new()),
        dirty,
    ));
    let outcome =
        execute_branch_operation(&repository, request, authorization, &mut runner).unwrap();
    assert_eq!(
        outcome,
        GitBranchOutcome::Switched {
            branch: "feature/safe".into(),
            preserved_dirty_state: true,
        }
    );
    let arguments = runner
        .calls
        .iter()
        .flat_map(|call| call.arguments.iter())
        .map(|value| value.to_string_lossy())
        .collect::<Vec<_>>();
    assert!(
        arguments
            .iter()
            .any(|argument| argument == "core.hooksPath=/dev/null")
    );
    assert!(
        arguments
            .iter()
            .any(|argument| argument == "core.fsmonitor=false")
    );
    for forbidden in ["stash", "commit", "reset", "revert", "checkout", "--force"] {
        assert!(!arguments.iter().any(|argument| argument == forbidden));
    }
}

#[test]
fn conflicting_switch_reports_paths_and_proves_the_worktree_snapshot_is_unchanged() {
    let (_temporary, repository) = visible_repository();
    let dirty = b" M src/main.rs\0";
    let request = request(
        &repository,
        dirty,
        GitBranchOperation::Switch {
            branch: "feature/conflict".into(),
            expected_target_oid: TARGET.into(),
        },
    );
    let authorization = git_authorization(&repository, &request);
    let stderr = b"error: Your local changes to the following files would be overwritten by checkout:\n\tsrc/main.rs\n\tREADME.md\nPlease commit your changes or stash them before you switch branches.\nAborting\n";
    let mut runner = FakeGitRunner::with(branch_outputs(
        dirty,
        GitCommandOutput::failure(1, stderr.to_vec()),
        dirty,
    ));
    let outcome =
        execute_branch_operation(&repository, request, authorization, &mut runner).unwrap();
    assert_eq!(
        outcome,
        GitBranchOutcome::Blocked {
            conflicting_paths: vec![PathBuf::from("README.md"), PathBuf::from("src/main.rs")],
            message: String::from_utf8(stderr.to_vec()).unwrap().trim().into(),
        }
    );
}

#[test]
fn failed_switch_with_a_changed_worktree_is_an_invariant_violation() {
    let (_temporary, repository) = visible_repository();
    let dirty = b" M src/main.rs\0";
    let request = request(
        &repository,
        dirty,
        GitBranchOperation::Switch {
            branch: "feature/conflict".into(),
            expected_target_oid: TARGET.into(),
        },
    );
    let authorization = git_authorization(&repository, &request);
    let mut runner = FakeGitRunner::with(branch_outputs(
        dirty,
        GitCommandOutput::failure(1, b"Aborting".to_vec()),
        b" M other.txt\0",
    ));
    assert!(matches!(
        execute_branch_operation(&repository, request, authorization, &mut runner),
        Err(GitError::WorktreeInvariantViolation)
    ));
}

#[test]
fn denied_or_mutated_git_operations_never_reach_git() {
    let (_temporary, repository) = visible_repository();
    let request = request(
        &repository,
        b"",
        GitBranchOperation::Create {
            branch: "feature/new".into(),
        },
    );
    let mut runner = FakeGitRunner::default();
    assert!(matches!(
        execute_branch_operation(
            &repository,
            request.clone(),
            GitOperationAuthorization::Denied {
                reason: "policy".into()
            },
            &mut runner
        ),
        Err(GitError::Denied(_))
    ));
    assert!(runner.calls.is_empty());

    let authorization = git_authorization(&repository, &request);
    let mut mutated = request;
    mutated.operation = GitBranchOperation::Create {
        branch: "feature/substituted".into(),
    };
    assert!(matches!(
        execute_branch_operation(&repository, mutated, authorization, &mut runner),
        Err(GitError::AuthorizationMismatch)
    ));
    assert!(runner.calls.is_empty());
}

#[test]
fn executable_checkout_filters_block_before_the_branch_effect() {
    let (_temporary, repository) = visible_repository();
    let request = request(
        &repository,
        b"",
        GitBranchOperation::Switch {
            branch: "feature/filtered".into(),
            expected_target_oid: TARGET.into(),
        },
    );
    let authorization = git_authorization(&repository, &request);
    let mut runner = FakeGitRunner::with([
        GitCommandOutput::success(format!("{HEAD}\n")),
        GitCommandOutput::success(Vec::new()),
        GitCommandOutput::success(b"filter.lfs.process git-lfs filter-process\n".to_vec()),
    ]);
    assert!(matches!(
        execute_branch_operation(&repository, request, authorization, &mut runner),
        Err(GitError::UnsafeRepositoryConfiguration)
    ));
    assert!(!runner.calls.iter().any(|call| {
        call.arguments
            .iter()
            .any(|argument| argument.to_string_lossy() == "switch")
    }));
}

#[test]
fn branch_creation_uses_only_an_explicit_safe_create_and_preserves_dirty_state() {
    let (_temporary, repository) = visible_repository();
    let dirty = b"?? draft.txt\0";
    let request = request(
        &repository,
        dirty,
        GitBranchOperation::Create {
            branch: "feature/new".into(),
        },
    );
    let authorization = git_authorization(&repository, &request);
    let mut outputs = branch_outputs(dirty, GitCommandOutput::success(Vec::new()), dirty);
    outputs.remove(4); // Create does not look up an existing local branch.
    *outputs.last_mut().unwrap() = GitCommandOutput::success(b"feature/new\n".to_vec());
    let mut runner = FakeGitRunner::with(outputs);
    let outcome =
        execute_branch_operation(&repository, request, authorization, &mut runner).unwrap();
    assert_eq!(
        outcome,
        GitBranchOutcome::Created {
            branch: "feature/new".into(),
            preserved_dirty_state: true,
        }
    );
    assert!(runner.calls.iter().any(|call| {
        call.arguments.ends_with(&[
            OsString::from("switch"),
            OsString::from("--no-recurse-submodules"),
            OsString::from("--create"),
            OsString::from("feature/new"),
        ])
    }));
}
