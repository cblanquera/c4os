use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use c4os_lib::runtime::pi::PI_NATIVE_VERSION;
use c4os_lib::runtime::supervisor::{
    CompatibilityState, HealthState, OPENCODE_NATIVE_VERSION, RUNTIME_PROTOCOL_VERSION,
    RuntimeInstallation, RuntimeKind, RuntimeLifecycle, RuntimeSupervisor, RuntimeTraceKind,
    SupervisorError, sha256_file,
};
use tempfile::TempDir;

const NOW: u64 = 1_721_300_000_000;

fn fixture_script(root: &Path) -> std::path::PathBuf {
    let executable = root.join("bin/fixture-worker");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(
        &executable,
        b"#!/bin/sh\n/bin/sleep 30 &\nchild=$!\nprintf '%s' \"$child\" > descendant.pid\nwait \"$child\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&executable, permissions).unwrap();
    executable
}

fn installation(root: &Path, native_version: &str) -> RuntimeInstallation {
    let executable = fixture_script(root);
    RuntimeInstallation {
        runtime_id: "opencode-primary".into(),
        workspace_id: "workspace-1".into(),
        runtime_kind: RuntimeKind::OpenCode,
        native_version: native_version.into(),
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

fn process_alive(process_id: u32) -> bool {
    Command::new("/bin/kill")
        .args(["-0", &process_id.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn wait_for_descendant(path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(value) = fs::read_to_string(path)
            && let Ok(process_id) = value.parse()
        {
            return process_id;
        }
        assert!(
            Instant::now() < deadline,
            "worker did not publish descendant identity"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn exact_compatibility_is_recorded_and_incompatible_versions_do_not_spawn() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    let compatibility = supervisor
        .register_installation(installation(temporary.path(), "1.18.4"), NOW)
        .unwrap();
    assert_eq!(compatibility, CompatibilityState::Incompatible);
    assert!(matches!(
        supervisor.start("opencode-primary", NOW + 1),
        Err(SupervisorError::Incompatible)
    ));
    let record = &supervisor.snapshot().records[0];
    assert_eq!(record.lifecycle, RuntimeLifecycle::Incompatible);
    assert_eq!(record.process_id, None);
}

#[test]
fn workspace_installation_batch_is_all_or_nothing_in_one_generation() {
    let temporary = TempDir::new().unwrap();
    let opencode = installation(&temporary.path().join("opencode"), OPENCODE_NATIVE_VERSION);
    let mut pi = installation(&temporary.path().join("pi"), PI_NATIVE_VERSION);
    pi.runtime_id = "pi-primary".into();
    pi.runtime_kind = RuntimeKind::Pi;
    let mut supervisor = RuntimeSupervisor::pinned();
    let before = supervisor.snapshot();

    let mut mixed_workspace = pi.clone();
    mixed_workspace.workspace_id = "workspace-2".into();
    assert!(matches!(
        supervisor.replace_workspace_installations(
            "workspace-1",
            vec![opencode.clone(), mixed_workspace],
            NOW,
        ),
        Err(SupervisorError::InvalidInstallation)
    ));
    assert_eq!(supervisor.snapshot(), before);

    let mut duplicate = pi.clone();
    duplicate.runtime_id = opencode.runtime_id.clone();
    assert!(matches!(
        supervisor.replace_workspace_installations(
            "workspace-1",
            vec![opencode.clone(), duplicate],
            NOW,
        ),
        Err(SupervisorError::InvalidInstallation)
    ));
    assert_eq!(supervisor.snapshot(), before);

    let mut invalid = pi.clone();
    invalid
        .sanitized_environment
        .insert("PROVIDER_API_KEY".into(), "must-not-publish".into());
    assert!(matches!(
        supervisor.replace_workspace_installations(
            "workspace-1",
            vec![opencode.clone(), invalid],
            NOW,
        ),
        Err(SupervisorError::InvalidInstallation)
    ));
    assert_eq!(supervisor.snapshot(), before);

    let compatibility = supervisor
        .replace_workspace_installations("workspace-1", vec![opencode, pi], NOW)
        .unwrap();
    assert_eq!(compatibility.len(), 2);
    assert!(
        compatibility
            .values()
            .all(|state| *state == CompatibilityState::Compatible)
    );
    let published = supervisor.snapshot();
    assert_eq!(published.state_generation, 1);
    assert_eq!(published.records.len(), 2);
    assert_eq!(published.events.len(), 2);
    assert!(published.records.iter().all(|record| {
        record.installation.workspace_id == "workspace-1"
            && record.lifecycle == RuntimeLifecycle::Stopped
    }));
}

#[test]
fn health_is_bound_to_the_exact_process_generation_and_restart_advances_it() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let first = supervisor.start("opencode-primary", NOW + 1).unwrap();
    supervisor
        .record_health("opencode-primary", first, HealthState::Healthy, NOW + 2)
        .unwrap();
    assert_eq!(
        supervisor.snapshot().records[0].lifecycle,
        RuntimeLifecycle::Ready
    );

    let second = supervisor
        .restart("opencode-primary", NOW + 3, Duration::from_millis(300))
        .unwrap();
    assert_eq!(second, first + 1);
    assert!(matches!(
        supervisor.record_health("opencode-primary", first, HealthState::Healthy, NOW + 5),
        Err(SupervisorError::StaleGeneration)
    ));
    assert_eq!(
        supervisor.snapshot().records[0].lifecycle,
        RuntimeLifecycle::Starting,
        "a stale ready event must not upgrade the new process"
    );
    supervisor
        .shutdown("opencode-primary", NOW + 6, Duration::from_millis(300))
        .unwrap();
    assert!(
        supervisor
            .snapshot()
            .events
            .iter()
            .any(|event| event.kind == RuntimeTraceKind::StaleEventRejected)
    );
}

#[test]
fn production_managed_process_is_not_ready_until_exact_attachment_and_requires_owned_shutdown() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();

    let generation = supervisor
        .reserve_managed_start("opencode-primary", NOW + 1)
        .unwrap();
    let reserved = &supervisor.snapshot().records[0];
    assert_eq!(reserved.lifecycle, RuntimeLifecycle::Starting);
    assert_eq!(reserved.process_id, None);
    assert!(matches!(
        supervisor.attach_managed_process(
            "opencode-primary",
            generation + 1,
            42,
            HealthState::Healthy,
            NOW + 2,
        ),
        Err(SupervisorError::StaleGeneration)
    ));

    supervisor
        .attach_managed_process(
            "opencode-primary",
            generation,
            42,
            HealthState::Healthy,
            NOW + 3,
        )
        .unwrap();
    let ready = &supervisor.snapshot().records[0];
    assert_eq!(ready.lifecycle, RuntimeLifecycle::Ready);
    assert_eq!(ready.process_id, Some(42));
    assert!(matches!(
        supervisor.shutdown("opencode-primary", NOW + 4, Duration::from_millis(100)),
        Err(SupervisorError::ExternallyManaged)
    ));

    supervisor
        .finish_managed_shutdown("opencode-primary", generation, NOW + 5)
        .unwrap();
    let stopped = &supervisor.snapshot().records[0];
    assert_eq!(stopped.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(stopped.process_id, None);
}

#[test]
fn failed_production_prepare_aborts_reserved_generation_without_publishing_a_pid() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let generation = supervisor
        .reserve_managed_start("opencode-primary", NOW + 1)
        .unwrap();

    supervisor
        .abort_managed_start("opencode-primary", generation, NOW + 2)
        .unwrap();
    let stopped = &supervisor.snapshot().records[0];
    assert_eq!(stopped.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(stopped.process_generation, generation);
    assert_eq!(stopped.process_id, None);
}

#[test]
fn shutdown_terminates_the_worker_process_group_including_descendants() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    supervisor.start("opencode-primary", NOW + 1).unwrap();
    let descendant = wait_for_descendant(
        &temporary
            .path()
            .join("state/isolated/opencode/1.18.3/workspace-1/generation-1/descendant.pid"),
    );
    assert!(process_alive(descendant));

    supervisor
        .shutdown("opencode-primary", NOW + 2, Duration::from_millis(300))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while process_alive(descendant) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !process_alive(descendant),
        "descendant survived process-group shutdown"
    );
    let record = &supervisor.snapshot().records[0];
    assert_eq!(record.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(record.process_id, None);
}

#[test]
fn executable_mutation_is_detected_before_launch() {
    let temporary = TempDir::new().unwrap();
    let installation = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    let executable = installation.executable.clone();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor.register_installation(installation, NOW).unwrap();
    fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    assert!(matches!(
        supervisor.start("opencode-primary", NOW + 1),
        Err(SupervisorError::DigestMismatch)
    ));
}

#[test]
fn secret_shaped_environment_fields_are_rejected() {
    let temporary = TempDir::new().unwrap();
    let mut candidate = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    candidate
        .sanitized_environment
        .insert("PROVIDER_API_KEY".into(), "must-not-launch".into());
    assert!(matches!(
        candidate.validate(),
        Err(SupervisorError::InvalidInstallation)
    ));
}

#[test]
fn secret_shaped_values_and_symlinked_state_roots_are_rejected() {
    let temporary = TempDir::new().unwrap();
    let mut argument = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    argument.arguments = vec!["--header=Authorization: Bearer hidden".into()];
    assert!(matches!(
        argument.validate(),
        Err(SupervisorError::InvalidInstallation)
    ));

    let mut value = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    value
        .sanitized_environment
        .insert("SERVICE_VALUE".into(), "sk-this-is-secret-material".into());
    assert!(matches!(
        value.validate(),
        Err(SupervisorError::InvalidInstallation)
    ));

    let outside = TempDir::new().unwrap();
    let escaped = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    fs::create_dir_all(temporary.path().join("state")).unwrap();
    std::os::unix::fs::symlink(outside.path(), &escaped.state_namespace).unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor.register_installation(escaped, NOW).unwrap();
    assert!(matches!(
        supervisor.start("opencode-primary", NOW + 1),
        Err(SupervisorError::InvalidInstallation)
    ));
}

#[test]
fn pinned_native_versions_are_explicit_in_the_snapshot() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let record = &supervisor.snapshot().records[0];
    assert_eq!(record.installation.native_version, "1.18.3");
    assert_eq!(record.installation.adapter_version, "1.0.0");
    assert_eq!(record.installation.protocol_version, 1);
}
