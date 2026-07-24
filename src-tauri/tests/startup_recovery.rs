use c4os_lib::startup_recovery::{
    MAX_STARTUP_RECOVERY_HISTORY, StartupRecoveryAction, StartupRecoveryActionInput,
    StartupRecoveryActionOutcome, StartupRecoveryBoundary, StartupRecoveryController,
    StartupRecoveryError, StartupRecoveryFailureInput, StartupRecoveryLifecycle,
};

fn failure(boundary: StartupRecoveryBoundary, failed_at_ms: u64) -> StartupRecoveryFailureInput {
    StartupRecoveryFailureInput {
        boundary,
        correlation_id: format!("correlation:{failed_at_ms}"),
        diagnostic_code: "startup-boundary-unavailable".into(),
        message: "The startup boundary is unavailable.".into(),
        failed_at_ms,
        validated_backup_available: false,
        recovery_location_available: true,
    }
}

fn action(
    expected_generation: u64,
    action: StartupRecoveryAction,
    requested_at_ms: u64,
) -> StartupRecoveryActionInput {
    StartupRecoveryActionInput {
        expected_generation,
        action,
        requested_at_ms,
    }
}

#[test]
fn boundary_and_action_wire_names_are_exact_and_complete() {
    let boundaries = [
        StartupRecoveryBoundary::Database,
        StartupRecoveryBoundary::Configuration,
        StartupRecoveryBoundary::BrowserRegistry,
        StartupRecoveryBoundary::Extension,
        StartupRecoveryBoundary::Workspace,
        StartupRecoveryBoundary::Runtime,
        StartupRecoveryBoundary::Mcp,
        StartupRecoveryBoundary::Update,
    ];
    assert_eq!(
        boundaries
            .into_iter()
            .map(|boundary| serde_json::to_string(&boundary).expect("serialize boundary"))
            .collect::<Vec<_>>(),
        vec![
            "\"database\"",
            "\"configuration\"",
            "\"browserRegistry\"",
            "\"extension\"",
            "\"workspace\"",
            "\"runtime\"",
            "\"mcp\"",
            "\"update\"",
        ]
    );
    assert_eq!(
        [
            StartupRecoveryAction::Retry,
            StartupRecoveryAction::RestoreValidatedBackup,
            StartupRecoveryAction::OpenRecoveryLocation,
        ]
        .into_iter()
        .map(|action| serde_json::to_string(&action).expect("serialize action"))
        .collect::<Vec<_>>(),
        vec![
            "\"retry\"",
            "\"restoreValidatedBackup\"",
            "\"openRecoveryLocation\"",
        ]
    );
}

#[test]
fn unsafe_diagnostics_fail_before_state_publication() {
    let mut controller = StartupRecoveryController::new();
    for (correlation_id, message) in [
        (
            "correlation:unsafe",
            "The provider returned Authorization: Bearer exposed.",
        ),
        (
            "correlation:unsafe",
            "Recovery state was written under /Users/example/private.",
        ),
        ("correlation:unsafe", "Unsafe\u{0000}diagnostic."),
        ("not a safe correlation", "The startup boundary failed."),
    ] {
        let mut input = failure(StartupRecoveryBoundary::Database, 1);
        input.correlation_id = correlation_id.into();
        input.message = message.into();
        assert_eq!(
            controller.report_failure(input),
            Err(StartupRecoveryError::InvalidInput)
        );
    }
    let snapshot = controller.snapshot();
    assert_eq!(snapshot.generation, 0);
    assert_eq!(snapshot.lifecycle, StartupRecoveryLifecycle::Healthy);
    assert!(snapshot.normal_work_authorized);
    assert!(snapshot.history.is_empty());
}

#[test]
fn stale_and_unavailable_actions_leave_degraded_state_unchanged() {
    let mut controller = StartupRecoveryController::new();
    let degraded = controller
        .report_failure(failure(StartupRecoveryBoundary::Configuration, 1))
        .expect("publish failure");
    assert!(controller.is_degraded());

    assert_eq!(
        controller.prepare_action(action(0, StartupRecoveryAction::Retry, 2)),
        Err(StartupRecoveryError::StaleGeneration {
            expected_generation: 0,
            actual_generation: degraded.generation,
        })
    );
    assert_eq!(
        controller.prepare_action(action(
            degraded.generation,
            StartupRecoveryAction::RestoreValidatedBackup,
            2,
        )),
        Err(StartupRecoveryError::UnavailableAction)
    );
    assert_eq!(controller.snapshot(), degraded);
}

#[test]
fn retry_failure_remains_blocked_and_retry_success_recovers_authority() {
    let mut controller = StartupRecoveryController::new();
    let degraded = controller
        .report_failure(failure(StartupRecoveryBoundary::Runtime, 1))
        .expect("publish runtime failure");
    assert!(!degraded.normal_work_authorized);

    let first = controller
        .prepare_action(action(degraded.generation, StartupRecoveryAction::Retry, 2))
        .expect("prepare retry");
    assert_eq!(
        controller.snapshot().lifecycle,
        StartupRecoveryLifecycle::Retrying
    );
    assert!(controller.is_degraded());
    let failed = controller
        .complete_action(
            &first,
            StartupRecoveryActionOutcome::Failed {
                diagnostic_code: "runtime-retry-failed".into(),
                message: "The runtime retry did not pass its health check.".into(),
                completed_at_ms: 3,
            },
        )
        .expect("record failed retry");
    assert_eq!(failed.lifecycle, StartupRecoveryLifecycle::Degraded);
    assert!(!failed.normal_work_authorized);
    assert_eq!(
        failed
            .failure
            .as_ref()
            .map(|value| value.diagnostic_code.as_str()),
        Some("runtime-retry-failed")
    );

    let second = controller
        .prepare_action(action(failed.generation, StartupRecoveryAction::Retry, 4))
        .expect("prepare second retry");
    let recovered = controller
        .complete_action(
            &second,
            StartupRecoveryActionOutcome::Succeeded { completed_at_ms: 5 },
        )
        .expect("complete retry");
    assert_eq!(recovered.lifecycle, StartupRecoveryLifecycle::Recovered);
    assert!(recovered.normal_work_authorized);
    assert!(recovered.failure.is_none());
    assert!(!controller.is_degraded());
}

#[test]
fn validated_restore_and_native_location_actions_are_exact_and_path_free() {
    let mut controller = StartupRecoveryController::new();
    let mut input = failure(StartupRecoveryBoundary::Database, 1);
    input.validated_backup_available = true;
    let degraded = controller.report_failure(input).expect("publish failure");
    assert_eq!(
        degraded.available_actions,
        vec![
            StartupRecoveryAction::Retry,
            StartupRecoveryAction::RestoreValidatedBackup,
            StartupRecoveryAction::OpenRecoveryLocation,
        ]
    );

    let open = controller
        .prepare_action(action(
            degraded.generation,
            StartupRecoveryAction::OpenRecoveryLocation,
            2,
        ))
        .expect("prepare native open");
    assert_eq!(open.boundary(), StartupRecoveryBoundary::Database);
    assert_eq!(open.action(), StartupRecoveryAction::OpenRecoveryLocation);
    let open_snapshot = controller
        .complete_action(
            &open,
            StartupRecoveryActionOutcome::Succeeded { completed_at_ms: 3 },
        )
        .expect("complete native open");
    assert_eq!(open_snapshot.lifecycle, StartupRecoveryLifecycle::Degraded);
    assert!(!open_snapshot.normal_work_authorized);
    let serialized = serde_json::to_string(&open_snapshot).expect("serialize safe snapshot");
    assert!(!serialized.contains("/Users/"));
    assert!(!serialized.contains("/private/"));
    assert!(!serialized.contains("file://"));

    let restore = controller
        .prepare_action(action(
            open_snapshot.generation,
            StartupRecoveryAction::RestoreValidatedBackup,
            4,
        ))
        .expect("prepare validated restore");
    let recovered = controller
        .complete_action(
            &restore,
            StartupRecoveryActionOutcome::Succeeded { completed_at_ms: 5 },
        )
        .expect("complete validated restore");
    assert_eq!(recovered.lifecycle, StartupRecoveryLifecycle::Recovered);
    assert!(recovered.normal_work_authorized);
}

#[test]
fn recovery_history_is_bounded_and_records_truncation() {
    let mut controller = StartupRecoveryController::new();
    let reports = MAX_STARTUP_RECOVERY_HISTORY + 17;
    for index in 0..reports {
        let timestamp = u64::try_from(index + 1).expect("bounded test timestamp");
        controller
            .report_failure(failure(StartupRecoveryBoundary::Update, timestamp))
            .expect("publish bounded failure");
    }
    let snapshot = controller.snapshot();
    assert_eq!(snapshot.history.len(), MAX_STARTUP_RECOVERY_HISTORY);
    assert_eq!(snapshot.history_truncated, 17);
    assert_eq!(
        snapshot.generation,
        u64::try_from(reports).expect("bounded generation")
    );
    assert_eq!(
        snapshot.history.first().map(|record| record.occurred_at_ms),
        Some(18)
    );
    assert!(controller.is_degraded());
}
