use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;

use c4os_lib::core::database::{
    ChatRecord, DatabaseActor, DatabaseDescriptor, InactiveEntity, LifecycleState,
    ProjectPathState, ProjectRecord, WorkspaceRecord,
};
use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityLayer, ModelLifecycle, RouteIdentity,
};
use c4os_lib::runtime::capability_evidence::CapabilityEvidenceRegistry;
use c4os_lib::runtime::persistence::{
    ProviderStateStore, RuntimeControlPlaneStore, SqliteSessionRepository, SupervisorStateStore,
};
use c4os_lib::runtime::provider::{
    PROVIDER_SCHEMA_VERSION, ProviderEndpoint, ProviderKind, ProviderProfile, ProviderService,
};
use c4os_lib::runtime::session::{
    AdapterBinding, CapabilitySnapshot, ConfigurationSnapshot, ExecutionEnvironmentBinding,
    FirstSubmission, ModelRouteSnapshot, ResourceSnapshot, RuntimeKind as SessionRuntimeKind,
    SessionBinding, SessionError, SessionLifecycle, SessionRepository, SessionRepositoryError,
    SessionService, capability_snapshot_from_effective_descriptor,
};
use c4os_lib::runtime::supervisor::{
    HealthState, OPENCODE_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION, RuntimeInstallation,
    RuntimeKind, RuntimeLifecycle, RuntimeSupervisor, RuntimeTraceKind, pinned_compatibility,
    sha256_file,
};
use c4os_lib::security::credentials::CredentialVault;
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;

fn app_profile() -> ProviderProfile {
    let vault = CredentialVault::session_only().unwrap();
    ProviderProfile {
        schema_version: PROVIDER_SCHEMA_VERSION,
        provider_id: "provider-openrouter".into(),
        kind: ProviderKind::OpenRouter,
        display_name: "OpenRouter".into(),
        endpoint: ProviderEndpoint {
            endpoint_id: "openrouter-chat".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            api_kind: "openai-compatible".into(),
        },
        credential_reference: vault
            .store("openrouter-key", b"never-in-product-state")
            .unwrap(),
        enabled: true,
    }
}

fn runtime_installation(root: &std::path::Path) -> RuntimeInstallation {
    let executable = root.join("bin/fixture-worker");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"#!/bin/sh\n/bin/sleep 30\n").unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&executable, permissions).unwrap();
    RuntimeInstallation {
        runtime_id: "opencode-primary".into(),
        workspace_id: "workspace-1".into(),
        runtime_kind: RuntimeKind::OpenCode,
        native_version: OPENCODE_NATIVE_VERSION.into(),
        adapter_version: "1.0.0".into(),
        protocol_version: RUNTIME_PROTOCOL_VERSION,
        install_root: root.into(),
        asset_tree_sha256: sha256_file(&executable).unwrap(),
        executable_sha256: sha256_file(&executable).unwrap(),
        executable,
        state_namespace: root.join("state/isolated"),
        arguments: vec![],
        sanitized_environment: BTreeMap::new(),
    }
}

#[test]
fn provider_and_supervisor_state_round_trip_with_cas_and_interrupted_recovery() {
    let temporary = TempDir::new().unwrap();
    let descriptor = DatabaseDescriptor::app(temporary.path().join("home"));
    {
        let (database, report) = DatabaseActor::start(descriptor.clone()).unwrap();
        assert_eq!(report.current_version, 6);
        let database = Arc::new(database);

        let mut providers = ProviderService::new();
        providers.save_profile(app_profile(), 0).unwrap();
        let provider_store = ProviderStateStore::new(Arc::clone(&database)).unwrap();
        provider_store.save(&providers, None, NOW).unwrap();
        let stored = database
            .runtime_state_document("provider-snapshot", "providers")
            .unwrap()
            .unwrap();
        assert!(stored.canonical_document.contains("credential:"));
        assert!(!stored.canonical_document.contains("never-in-product-state"));

        let mut supervisor = RuntimeSupervisor::pinned();
        supervisor
            .register_installation(runtime_installation(temporary.path()), NOW)
            .unwrap();
        supervisor.start("opencode-primary", NOW + 1).unwrap();
        let supervisor_store = SupervisorStateStore::new(Arc::clone(&database)).unwrap();
        supervisor_store.save(&supervisor, None, NOW + 2).unwrap();
        supervisor
            .shutdown("opencode-primary", NOW + 3, Duration::from_millis(300))
            .unwrap();
    }

    let (database, report) = DatabaseActor::start(descriptor).unwrap();
    assert_eq!(report.previous_version, 6);
    let database = Arc::new(database);
    let provider_store = ProviderStateStore::new(Arc::clone(&database)).unwrap();
    let providers = provider_store.load().unwrap().unwrap();
    assert_eq!(providers.snapshot().providers.len(), 1);
    assert_eq!(providers.snapshot().generation, 1);

    let supervisor_store = SupervisorStateStore::new(Arc::clone(&database)).unwrap();
    let supervisor = supervisor_store
        .load(pinned_compatibility())
        .unwrap()
        .unwrap();
    let snapshot = supervisor.snapshot();
    assert_eq!(snapshot.records[0].lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(snapshot.records[0].process_id, None);
    assert_eq!(snapshot.records[0].process_generation, 1);
    assert!(snapshot.events.iter().any(|event| {
        event.kind == RuntimeTraceKind::RecoveredInterrupted && event.process_generation == 1
    }));
}

#[test]
fn control_plane_restore_atomically_clears_recovered_process_authority() {
    let temporary = TempDir::new().unwrap();
    let descriptor = DatabaseDescriptor::app(temporary.path().join("control-plane-home"));
    let (database, _) = DatabaseActor::start(descriptor.clone()).unwrap();
    let database = Arc::new(database);
    let store = RuntimeControlPlaneStore::new(Arc::clone(&database)).unwrap();

    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(runtime_installation(temporary.path()), NOW)
        .unwrap();
    let process_generation = supervisor
        .reserve_managed_start("opencode-primary", NOW + 1)
        .unwrap();
    supervisor
        .attach_managed_process(
            "opencode-primary",
            process_generation,
            42_424,
            HealthState::Healthy,
            NOW + 2,
        )
        .unwrap();
    let mut capabilities = CapabilityEvidenceRegistry::new();
    assert_eq!(
        capabilities
            .replace_process_routes(0, "opencode-primary", process_generation, Vec::new(),)
            .unwrap(),
        1
    );
    assert_eq!(
        store.save(&supervisor, &capabilities, 0, NOW + 3).unwrap(),
        1
    );

    drop(store);
    drop(database);
    let (database, _) = DatabaseActor::start(descriptor).unwrap();
    let database = Arc::new(database);
    let store = RuntimeControlPlaneStore::new(Arc::clone(&database)).unwrap();
    let restored = store.load(pinned_compatibility()).unwrap().unwrap();
    assert_eq!(restored.revision, 2);
    let supervisor_snapshot = restored.supervisor.snapshot();
    assert_eq!(
        supervisor_snapshot.records[0].lifecycle,
        RuntimeLifecycle::Stopped
    );
    assert_eq!(supervisor_snapshot.records[0].process_id, None);
    assert!(supervisor_snapshot.events.iter().any(|event| {
        event.kind == RuntimeTraceKind::RecoveredInterrupted
            && event.process_generation == process_generation
    }));
    let capability_snapshot = restored.capabilities.snapshot();
    assert_eq!(capability_snapshot.generation, 2);
    assert!(capability_snapshot.active_processes.is_empty());
    assert!(capability_snapshot.active_routes.is_empty());
    assert_eq!(
        database
            .runtime_state_document("runtime-control-plane", "runtime")
            .unwrap()
            .unwrap()
            .generation,
        2
    );

    let stable = store.load(pinned_compatibility()).unwrap().unwrap();
    assert_eq!(stable.revision, 2);
    assert_eq!(stable.supervisor.snapshot(), supervisor_snapshot);
    assert_eq!(stable.capabilities.snapshot(), capability_snapshot);
}

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

fn session_capabilities() -> CapabilitySnapshot {
    capability_snapshot_from_effective_descriptor(
        CapabilityDescriptor {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            layer: CapabilityLayer::Effective,
            route: RouteIdentity {
                provider_id: "provider-anthropic".into(),
                endpoint_id: "endpoint-default".into(),
                provider_model_id: "claude-sonnet".into(),
                model_revision: "2026-07-01".into(),
                adapter_kind: "opencode".into(),
                adapter_version: "1.0.0".into(),
                runtime_kind: "opencode".into(),
                native_runtime_version: "1.18.3".into(),
                session_configuration_sha256: digest('a'),
            },
            lifecycle: ModelLifecycle::Active,
            features: BTreeMap::new(),
            numeric_limits: BTreeMap::new(),
            raw_evidence_sha256: digest('c'),
        },
        1,
    )
    .expect("exact session capabilities")
}

fn session_binding() -> SessionBinding {
    SessionBinding {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "runtime:opencode".into(),
        runtime_kind: SessionRuntimeKind::OpenCode,
        adapter: AdapterBinding {
            adapter_id: "adapter:opencode".into(),
            adapter_version: "1.0.0".into(),
            native_version: "1.18.3".into(),
        },
        environment: ExecutionEnvironmentBinding {
            environment_id: "environment:local".into(),
            environment_kind: "local".into(),
            host_alias: None,
        },
        initial_model_route: ModelRouteSnapshot {
            route_id: "route:claude".into(),
            provider_id: "provider-anthropic".into(),
            endpoint_id: "endpoint-default".into(),
            model_id: "claude-sonnet".into(),
            model_revision: "2026-07-01".into(),
        },
        initial_configuration: ConfigurationSnapshot {
            snapshot_id: "configuration-1".into(),
            version: 1,
            sha256: digest('a'),
        },
        initial_resources: ResourceSnapshot {
            snapshot_id: "resources-1".into(),
            version: 1,
            sha256: digest('b'),
            resource_ids: vec!["skill:project/review".into()],
        },
        initial_capabilities: session_capabilities(),
        bound_at_ms: NOW + 1,
    }
}

fn seed_workspace(database: &DatabaseActor) {
    database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-1".into(),
            display_name: "C4OS".into(),
            created_at: 1,
            updated_at: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_project(ProjectRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            display_name: "Project One".into(),
            current_path: "/tmp/project-1".into(),
            last_known_path: "/tmp/project-1".into(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_chat(ChatRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            chat_id: "session-1".into(),
            title: "New Chat".into(),
            created_at: 1,
            updated_at: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
}

fn add_second_project(database: &DatabaseActor) {
    database
        .add_project(ProjectRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-2".into(),
            display_name: "Project Two".into(),
            current_path: "/tmp/project-2".into(),
            last_known_path: "/tmp/project-2".into(),
            path_state: ProjectPathState::Found,
            position: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
}

fn first_submission(binding: SessionBinding) -> FirstSubmission {
    FirstSubmission {
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        authorization_scope_id: "authority-1".into(),
        correlation_id: "correlation-1".into(),
        process_generation: 7,
        prompt: Some("Implement durable sessions".into()),
        attachments: vec![],
        binding,
        submitted_at_ms: NOW + 1,
    }
}

#[test]
fn sqlite_session_repository_atomically_persists_first_binding_and_restart_recovery() {
    let temporary = TempDir::new().unwrap();
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temporary.path().join("active"),
        "workspace-1",
        temporary.path().join("recovery"),
    );
    {
        let (database, report) = DatabaseActor::start(descriptor.clone()).unwrap();
        assert_eq!(report.current_version, 4);
        seed_workspace(&database);
        let repository = SqliteSessionRepository::new(Arc::new(database)).unwrap();
        let service = SessionService::new(repository.clone());
        service.create_provisional("session-1", NOW).unwrap();
        let bound = service
            .submit_first(first_submission(session_binding()))
            .unwrap();
        assert_eq!(bound.revision, 1);
        assert!(matches!(bound.lifecycle, SessionLifecycle::Bound { .. }));
        let mut stale = bound.clone();
        stale.revision = 2;
        stale.updated_at_ms = NOW + 2;
        repository
            .compare_and_swap("session-1", 0, &stale)
            .expect_err("stale compare-and-swap must lose");
    }

    let (database, report) = DatabaseActor::start(descriptor).unwrap();
    assert_eq!(report.previous_version, 4);
    let repository = SqliteSessionRepository::new(Arc::new(database)).unwrap();
    let service = SessionService::new(repository);
    let restored = service.session("session-1").unwrap();
    assert_eq!(restored.turns.len(), 1);
    assert_eq!(restored.attempts.len(), 1);
    assert_eq!(restored.active_attempt_id.as_deref(), Some("attempt-1"));
    let recovered = service.recover_interrupted("session-1", NOW + 10).unwrap();
    assert_eq!(recovered.revision, 2);
    assert_eq!(recovered.active_attempt_id, None);
    assert!(recovered.attempts[0].status.is_terminal());
}

#[test]
fn first_submission_atomically_creates_chat_and_session_without_a_saved_blank() {
    let temporary = TempDir::new().unwrap();
    let descriptor = DatabaseDescriptor::workspace_with_recovery_dir(
        temporary.path().join("active-atomic-promotion"),
        "workspace-1",
        temporary.path().join("recovery-atomic-promotion"),
    );
    let (database, _) = DatabaseActor::start(descriptor).unwrap();
    database
        .create_workspace(WorkspaceRecord {
            workspace_id: "workspace-1".into(),
            display_name: "C4OS".into(),
            created_at: 1,
            updated_at: 1,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    database
        .add_project(ProjectRecord {
            workspace_id: "workspace-1".into(),
            project_id: "project-1".into(),
            display_name: "Project One".into(),
            current_path: "/tmp/project-1".into(),
            last_known_path: "/tmp/project-1".into(),
            path_state: ProjectPathState::Found,
            position: 0,
            lifecycle_state: LifecycleState::Active,
            inactivated_at: None,
        })
        .unwrap();
    let database = Arc::new(database);
    assert!(database.session_documents().unwrap().is_empty());
    let repository = SqliteSessionRepository::new(Arc::clone(&database)).unwrap();
    let service = SessionService::new(repository);
    service.create_provisional("session-1", NOW).unwrap();
    let promoted = service
        .submit_first(first_submission(session_binding()))
        .unwrap();
    assert_eq!(
        promoted.title.as_deref(),
        Some("Implement durable sessions")
    );
    let snapshot = match database
        .snapshot(c4os_lib::core::database::SnapshotQuery::new(16).unwrap())
        .unwrap()
    {
        c4os_lib::core::database::DatabaseSnapshot::Workspace(snapshot) => snapshot,
        _ => panic!("expected Workspace snapshot"),
    };
    assert_eq!(snapshot.chats.len(), 1);
    assert_eq!(snapshot.chats[0].chat_id, "session-1");
    assert_eq!(snapshot.chats[0].title, "Implement durable sessions");
    assert_eq!(database.session_documents().unwrap().len(), 1);
}

#[test]
fn sqlite_session_repository_rejects_stale_or_substituted_chat_project_authority() {
    enum InvalidBinding {
        MissingProject,
        InactiveWorkspace,
        InactiveProject,
        InactiveChat,
        OtherActiveProject,
        ReassignedChat,
    }

    for case in [
        InvalidBinding::MissingProject,
        InvalidBinding::InactiveWorkspace,
        InvalidBinding::InactiveProject,
        InvalidBinding::InactiveChat,
        InvalidBinding::OtherActiveProject,
        InvalidBinding::ReassignedChat,
    ] {
        let temporary = TempDir::new().unwrap();
        let descriptor =
            DatabaseDescriptor::workspace(temporary.path().join("active"), "workspace-1");
        let (database, _) = DatabaseActor::start(descriptor).unwrap();
        seed_workspace(&database);
        assert!(database.active_project_exists("project-1").unwrap());

        let mut binding = session_binding();
        match case {
            InvalidBinding::MissingProject => {
                binding.project_id = Some("project-missing".into());
            }
            InvalidBinding::InactiveWorkspace => {
                database.inactivate(InactiveEntity::Workspace, 2).unwrap();
            }
            InvalidBinding::InactiveProject => {
                database
                    .inactivate(
                        InactiveEntity::Project {
                            project_id: "project-1".into(),
                        },
                        2,
                    )
                    .unwrap();
            }
            InvalidBinding::InactiveChat => {
                database
                    .inactivate(
                        InactiveEntity::Chat {
                            chat_id: "session-1".into(),
                        },
                        2,
                    )
                    .unwrap();
            }
            InvalidBinding::OtherActiveProject => {
                add_second_project(&database);
                binding.project_id = Some("project-2".into());
            }
            InvalidBinding::ReassignedChat => {
                add_second_project(&database);
                database
                    .add_chat(ChatRecord {
                        workspace_id: "workspace-1".into(),
                        project_id: "project-2".into(),
                        chat_id: "session-1".into(),
                        title: "Moved Chat".into(),
                        created_at: 1,
                        updated_at: 2,
                        lifecycle_state: LifecycleState::Active,
                        inactivated_at: None,
                    })
                    .unwrap();
            }
        }

        let repository = SqliteSessionRepository::new(Arc::new(database)).unwrap();
        let service = SessionService::new(repository.clone());
        service.create_provisional("session-1", NOW).unwrap();
        assert_eq!(
            service.submit_first(first_submission(binding)),
            Err(SessionError::Repository(SessionRepositoryError::Conflict))
        );
        assert_eq!(repository.load("session-1").unwrap(), None);
    }
}

#[test]
fn sqlite_session_cas_rechecks_active_chat_project_authority() {
    let temporary = TempDir::new().unwrap();
    let descriptor = DatabaseDescriptor::workspace(temporary.path().join("active"), "workspace-1");
    let (database, _) = DatabaseActor::start(descriptor).unwrap();
    seed_workspace(&database);
    let database = Arc::new(database);
    let repository = SqliteSessionRepository::new(Arc::clone(&database)).unwrap();
    let service = SessionService::new(repository.clone());
    service.create_provisional("session-1", NOW).unwrap();
    let bound = service
        .submit_first(first_submission(session_binding()))
        .unwrap();

    database
        .inactivate(
            InactiveEntity::Chat {
                chat_id: "session-1".into(),
            },
            2,
        )
        .unwrap();
    let mut replacement = bound;
    replacement.revision += 1;
    replacement.updated_at_ms += 1;
    assert_eq!(
        repository.compare_and_swap("session-1", 1, &replacement),
        Err(SessionRepositoryError::Conflict)
    );
    assert_eq!(repository.load("session-1").unwrap().unwrap().revision, 1);
}

#[test]
fn app_state_store_rejects_stale_generations() {
    let temporary = TempDir::new().unwrap();
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).unwrap();
    let database = Arc::new(database);
    let store = ProviderStateStore::new(database).unwrap();
    let mut service = ProviderService::new();
    let mut profile = app_profile();
    service.save_profile(profile.clone(), 0).unwrap();
    store.save(&service, None, NOW).unwrap();
    profile.display_name = "OpenRouter updated".into();
    service.save_profile(profile, 1).unwrap();
    store.save(&service, Some(1), NOW + 1).unwrap();
    assert!(store.save(&service, Some(1), NOW + 2).is_err());
}
