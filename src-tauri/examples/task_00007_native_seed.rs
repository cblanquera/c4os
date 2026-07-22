//! Seeds one isolated Task 00007 acceptance home through production services.
//!
//! This example refuses non-empty targets. The resulting home is consumed only
//! by the debug-only `--c4os-acceptance-home` startup switch.

use c4os_lib::core::configuration::{ManagedCeilings, SecurityConstraints};
use c4os_lib::core::database::{
    DatabaseActor, DatabaseDescriptor, WorkspaceConversationStateRecord,
};
use c4os_lib::core::services::{
    create_workspace_from_project, restore_app_configuration, save_app_configuration,
};
use c4os_lib::core::workspace::{C4osHomeLayout, WorkspaceLockOwner};
use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, ModelLifecycle, RouteIdentity,
};
use c4os_lib::runtime::persistence::SqliteSessionRepository;
use c4os_lib::runtime::session::{
    AdapterBinding, AttemptIdentity, ConfigurationSnapshot, ExecutionEnvironmentBinding,
    FirstSubmission, ModelRouteSnapshot, ResourceSnapshot, RunEventKind, RunEventRecord,
    RuntimeKind, SessionBinding, SessionService, TerminalAttemptOutcome,
    capability_snapshot_from_effective_descriptor,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn Error>> {
    let home_path = acceptance_home_argument()?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?;
    let now = i64::try_from(now_ms)?;
    let home = C4osHomeLayout::new(&home_path);

    let (app_database, _) = DatabaseActor::start(DatabaseDescriptor::app(home.root()))?;
    let mut app_configuration = restore_app_configuration(
        &app_database,
        &home,
        ManagedCeilings::default(),
        SecurityConstraints::default(),
    )?;
    save_app_configuration(
        &app_database,
        &home,
        &mut app_configuration,
        "schema_version = 1\nrestore_last_workspace = true\n",
        0,
        now,
    )?;
    drop(app_configuration);
    drop(app_database);

    let project_root = home_path.join("task-00007-project");
    fs::create_dir(&project_root)?;
    fs::write(
        project_root.join("README.md"),
        b"# Task 00007 native acceptance project\n",
    )?;
    let mut active = create_workspace_from_project(
        &home,
        &project_root,
        "Task 00007 Project",
        "Task 00007 Service-backed Workspace",
        env!("CARGO_PKG_VERSION"),
        WorkspaceLockOwner {
            process_id: std::process::id(),
            app_instance_id: Uuid::new_v4(),
            acquired_unix_ms: now_ms,
            label: "task-00007-native-seed".into(),
        },
        now,
    )?;
    let workspace_id = active.manifest().workspace_id;
    let project_id = active.manifest().projects[0].project_id;
    let session_id = Uuid::new_v4();
    active.create_chat(
        project_id,
        session_id,
        "Verify service-backed Chat",
        now + 1,
    )?;
    drop(active);

    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        home.active_workspace(),
        workspace_id.to_string(),
        home.workspace_recovery_root()
            .join(workspace_id.to_string()),
    );
    let (workspace_database, _) = DatabaseActor::start(descriptor)?;
    let session = session_id.to_string();
    let turn = "turn:acceptance";
    let mut drafts = serde_json::Map::new();
    drafts.insert(
        session.clone(),
        serde_json::json!({
            "prompt": "Continue this durable Reply after restart",
            "attachments": [],
            "nextAttachmentReference": 1,
            "providerId": "provider-acceptance",
            "modelId": "model-acceptance",
            "reasoningMode": "medium",
            "mode": "chat",
            "replyTargetId": turn,
        }),
    );
    workspace_database.save_conversation_state(
        WorkspaceConversationStateRecord {
            workspace_id: workspace_id.to_string(),
            generation: 1,
            canonical_document: serde_json::json!({
                "schemaVersion": 1,
                "activeProjectId": project_id.to_string(),
                "activeSessionId": session.clone(),
                "drafts": drafts,
            })
            .to_string(),
            updated_at_ms: now_ms + 1,
        },
        None,
    )?;
    let repository = SqliteSessionRepository::new(Arc::new(workspace_database))?;
    let sessions = SessionService::new(repository);
    let attempt = "attempt:acceptance";
    let correlation = "correlation:acceptance";
    sessions.create_provisional(&session, now_ms + 2)?;
    sessions.submit_first(FirstSubmission {
        session_id: session.clone(),
        turn_id: turn.into(),
        attempt_id: attempt.into(),
        authorization_scope_id: "authority:acceptance".into(),
        correlation_id: correlation.into(),
        process_generation: 1,
        prompt: Some("Verify the production-composed native Chat golden path.".into()),
        attachments: Vec::new(),
        skill_context: Vec::new(),
        binding: acceptance_binding(&workspace_id.to_string(), &project_id.to_string(), now_ms)?,
        submitted_at_ms: now_ms + 2,
    })?;
    let identity = AttemptIdentity {
        attempt_id: attempt.into(),
        correlation_id: correlation.into(),
        process_generation: 1,
    };
    for (index, (kind, payload)) in [
        (
            RunEventKind::TextDelta,
            "The **service-backed native Chat** is restored from the Rust-owned Workspace and SessionRecord.",
        ),
        (
            RunEventKind::ReasoningSummary,
            "Checked the active Workspace, durable Chat binding, and bounded renderer projection.",
        ),
        (
            RunEventKind::WorkActivity,
            "private runtime work payload is intentionally normalized",
        ),
        (
            RunEventKind::Usage,
            r#"{"inputTokens":1200,"outputTokens":480}"#,
        ),
        (RunEventKind::Completion, "private completion payload"),
    ]
    .into_iter()
    .enumerate()
    {
        sessions.append_event(
            &session,
            &identity,
            RunEventRecord {
                sequence: u64::try_from(index)? + 1,
                session_id: session.clone(),
                turn_id: turn.into(),
                attempt_id: attempt.into(),
                runtime_id: "runtime:acceptance".into(),
                environment_id: "environment:local".into(),
                correlation_id: correlation.into(),
                process_generation: 1,
                kind,
                payload: payload.into(),
                recorded_at_ms: now_ms + 10 + u64::try_from(index)?,
            },
        )?;
    }
    sessions.finish_attempt(
        &session,
        &identity,
        TerminalAttemptOutcome::Completed,
        now_ms + 800,
    )?;

    println!("{}", home_path.display());
    Ok(())
}

fn acceptance_home_argument() -> Result<PathBuf, Box<dyn Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: task_00007_native_seed <empty-absolute-mode-0700-directory>")?;
    if !path.is_absolute() {
        return Err("acceptance home must be absolute".into());
    }
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err("acceptance home must be an owned mode-0700 directory".into());
    }
    let canonical = path.canonicalize()?;
    if canonical != path || fs::read_dir(&canonical)?.next().is_some() {
        return Err("acceptance home must be canonical and empty".into());
    }
    let permitted = [std::env::temp_dir(), PathBuf::from("/private/tmp")]
        .into_iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| canonical.starts_with(root));
    if !permitted {
        return Err("acceptance home must remain under the system temporary root".into());
    }
    Ok(canonical)
}

fn acceptance_binding(
    workspace_id: &str,
    project_id: &str,
    now_ms: u64,
) -> Result<SessionBinding, Box<dyn Error>> {
    let configuration_sha256 = digest('a');
    let evidence = CapabilityEvidence {
        state: CapabilityState::Supported,
        layer: CapabilityLayer::Effective,
        source: "c4os.acceptance".into(),
        checked_at_ms: now_ms,
        expires_at_ms: Some(now_ms + 60_000),
        constraints: Vec::new(),
        allowed_values: Vec::new(),
        reason: None,
    };
    let descriptor = CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Effective,
        route: RouteIdentity {
            provider_id: "provider-acceptance".into(),
            endpoint_id: "endpoint-acceptance".into(),
            provider_model_id: "model-acceptance".into(),
            model_revision: "revision-acceptance".into(),
            adapter_kind: "opencode".into(),
            adapter_version: "1.0.0".into(),
            runtime_kind: "opencode".into(),
            native_runtime_version: "1.18.3".into(),
            session_configuration_sha256: configuration_sha256.clone(),
        },
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([
            (CapabilityKey::InputText, evidence.clone()),
            (CapabilityKey::OutputText, evidence.clone()),
            (CapabilityKey::ReasoningSummary, evidence),
        ]),
        numeric_limits: BTreeMap::new(),
        raw_evidence_sha256: digest('c'),
    };
    Ok(SessionBinding {
        workspace_id: workspace_id.into(),
        project_id: Some(project_id.into()),
        runtime_id: "runtime:acceptance".into(),
        runtime_kind: RuntimeKind::OpenCode,
        adapter: AdapterBinding {
            adapter_id: "adapter:acceptance".into(),
            adapter_version: "1.0.0".into(),
            native_version: "1.18.3".into(),
        },
        environment: ExecutionEnvironmentBinding {
            environment_id: "environment:local".into(),
            environment_kind: "local".into(),
            host_alias: None,
        },
        initial_model_route: ModelRouteSnapshot {
            route_id: "route:acceptance".into(),
            provider_id: "provider-acceptance".into(),
            endpoint_id: "endpoint-acceptance".into(),
            model_id: "model-acceptance".into(),
            model_revision: "revision-acceptance".into(),
        },
        initial_configuration: ConfigurationSnapshot {
            snapshot_id: "configuration:acceptance".into(),
            version: 1,
            sha256: configuration_sha256,
        },
        initial_resources: ResourceSnapshot {
            snapshot_id: "resources:acceptance".into(),
            version: 1,
            sha256: digest('b'),
            resource_ids: Vec::new(),
        },
        initial_capabilities: capability_snapshot_from_effective_descriptor(descriptor, 1)?,
        bound_at_ms: now_ms,
    })
}

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}
