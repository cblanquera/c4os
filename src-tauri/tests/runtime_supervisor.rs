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
    RuntimeInstallation, RuntimeKind, RuntimeLifecycle, RuntimeRecoveryAction, RuntimeSupervisor,
    RuntimeTraceKind, SupervisorError, pinned_compatibility, sha256_file,
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
fn exact_workspace_rebinding_preserves_crash_loop_recovery_state() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    let installation = installation(temporary.path(), OPENCODE_NATIVE_VERSION);
    supervisor
        .register_installation(installation.clone(), NOW)
        .unwrap();
    let mut next_start_ms = NOW + 1;
    for attempt in 1_u16..=5 {
        let generation = supervisor
            .reserve_managed_start("opencode-primary", next_start_ms)
            .unwrap();
        let failed_at_ms = next_start_ms + 1;
        supervisor
            .abort_managed_start("opencode-primary", generation, failed_at_ms)
            .unwrap();
        if attempt < 5 {
            next_start_ms = failed_at_ms + (1_u64 << (attempt - 1)) * 1_000;
        }
    }
    let before = supervisor.snapshot().records[0].clone();

    supervisor
        .replace_workspace_installations("workspace-1", vec![installation.clone()], NOW + 50_000)
        .unwrap();
    let rebound = &supervisor.snapshot().records[0];
    assert_eq!(rebound.process_generation, before.process_generation);
    assert_eq!(rebound.restart_attempts, before.restart_attempts);
    assert_eq!(rebound.next_restart_at_ms, before.next_restart_at_ms);
    assert_eq!(rebound.recovery_action, before.recovery_action);

    let mut changed = installation;
    changed.arguments.push("--replacement-binding".into());
    supervisor
        .replace_workspace_installations("workspace-1", vec![changed], NOW + 50_001)
        .unwrap();
    let replaced = &supervisor.snapshot().records[0];
    assert_eq!(replaced.process_generation, before.process_generation);
    assert_eq!(replaced.restart_attempts, 0);
    assert_eq!(replaced.next_restart_at_ms, None);
    assert_eq!(replaced.recovery_action, None);
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
fn crash_loop_backoff_is_bounded_persisted_and_requires_explicit_review() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let mut next_start_ms = NOW + 1;
    for attempt in 1_u16..=5 {
        let generation = supervisor
            .reserve_managed_start("opencode-primary", next_start_ms)
            .unwrap();
        let failed_at_ms = next_start_ms + 1;
        supervisor
            .abort_managed_start("opencode-primary", generation, failed_at_ms)
            .unwrap();
        let record = &supervisor.snapshot().records[0];
        assert_eq!(record.restart_attempts, attempt);
        if attempt < 5 {
            let retry_at_ms = failed_at_ms + (1_u64 << (attempt - 1)) * 1_000;
            assert_eq!(record.next_restart_at_ms, Some(retry_at_ms));
            assert_eq!(record.recovery_action, None);
            assert!(matches!(
                supervisor.reserve_managed_start("opencode-primary", retry_at_ms - 1),
                Err(SupervisorError::RestartBackoff)
            ));
            next_start_ms = retry_at_ms;
        } else {
            assert_eq!(record.next_restart_at_ms, None);
            assert_eq!(
                record.recovery_action,
                Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop)
            );
        }
    }
    assert!(matches!(
        supervisor.reserve_managed_start("opencode-primary", next_start_ms + 1),
        Err(SupervisorError::CrashLoopBudgetExceeded)
    ));

    let mut restored =
        RuntimeSupervisor::restore(pinned_compatibility(), supervisor.snapshot()).unwrap();
    let record = &restored.snapshot().records[0];
    let process_generation = record.process_generation;
    assert_eq!(record.restart_attempts, 5);
    assert_eq!(
        record.recovery_action,
        Some(RuntimeRecoveryAction::ReviewRuntimeCrashLoop)
    );
    assert!(matches!(
        restored.review_crash_loop("opencode-primary", process_generation + 1, NOW + 50_000,),
        Err(SupervisorError::RecoveryUnavailable)
    ));
    restored
        .review_crash_loop("opencode-primary", process_generation, NOW + 50_001)
        .expect("explicit crash-loop review");
    let reviewed = &restored.snapshot().records[0];
    assert_eq!(reviewed.restart_attempts, 0);
    assert_eq!(reviewed.recovery_action, None);
    assert!(
        restored
            .snapshot()
            .events
            .iter()
            .any(|event| event.kind == RuntimeTraceKind::RecoveryReviewed)
    );
    restored
        .reserve_managed_start("opencode-primary", NOW + 50_002)
        .expect("review reauthorizes a fresh bounded start");
}

#[test]
fn healthy_attachment_resets_crash_budget_and_restart_overflow_stays_blocked() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let first = supervisor
        .reserve_managed_start("opencode-primary", NOW + 1)
        .unwrap();
    supervisor
        .abort_managed_start("opencode-primary", first, NOW + 2)
        .unwrap();
    let retry_at = supervisor.snapshot().records[0].next_restart_at_ms.unwrap();
    let second = supervisor
        .reserve_managed_start("opencode-primary", retry_at)
        .unwrap();
    supervisor
        .attach_managed_process(
            "opencode-primary",
            second,
            42,
            HealthState::Healthy,
            retry_at + 1,
        )
        .unwrap();
    let healthy = &supervisor.snapshot().records[0];
    assert_eq!(healthy.restart_attempts, 0);
    assert_eq!(healthy.next_restart_at_ms, None);
    assert_eq!(healthy.recovery_action, None);

    supervisor
        .finish_managed_shutdown("opencode-primary", second, u64::MAX - 501)
        .unwrap();
    let third = supervisor
        .reserve_managed_start("opencode-primary", u64::MAX - 500)
        .unwrap();
    supervisor
        .abort_managed_start("opencode-primary", third, u64::MAX - 499)
        .unwrap();
    assert_eq!(
        supervisor.snapshot().records[0].next_restart_at_ms,
        Some(u64::MAX)
    );
    assert!(matches!(
        supervisor.reserve_managed_start("opencode-primary", u64::MAX - 1),
        Err(SupervisorError::RestartBackoff)
    ));
}

#[test]
fn restoring_an_active_runtime_drops_pid_authority_and_never_replays_a_lease() {
    let temporary = TempDir::new().unwrap();
    let mut supervisor = RuntimeSupervisor::pinned();
    supervisor
        .register_installation(installation(temporary.path(), OPENCODE_NATIVE_VERSION), NOW)
        .unwrap();
    let generation = supervisor
        .reserve_managed_start("opencode-primary", NOW + 1)
        .unwrap();
    supervisor
        .attach_managed_process(
            "opencode-primary",
            generation,
            42,
            HealthState::Degraded,
            NOW + 2,
        )
        .unwrap();

    let mut restored =
        RuntimeSupervisor::restore(pinned_compatibility(), supervisor.snapshot()).unwrap();
    let recovered = &restored.snapshot().records[0];
    assert_eq!(recovered.lifecycle, RuntimeLifecycle::Stopped);
    assert_eq!(recovered.process_id, None);
    assert_eq!(recovered.restart_attempts, 1);
    assert!(
        restored
            .snapshot()
            .events
            .iter()
            .any(|event| event.kind == RuntimeTraceKind::RecoveredInterrupted)
    );
    assert!(matches!(
        restored.reserve_managed_start("opencode-primary", NOW + 2),
        Err(SupervisorError::RestartBackoff)
    ));
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
