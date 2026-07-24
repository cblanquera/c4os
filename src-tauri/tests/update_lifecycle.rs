#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, sync::Arc};

use c4os_lib::{
    core::database::{
        DatabaseActor, DatabaseDescriptor, RuntimeStateDocumentRecord, SnapshotQuery,
    },
    update::{
        DiagnosticCategory, DiagnosticSeverity, LocalUpdateStageInput, MAX_DIAGNOSTIC_RECORDS,
        OperationalDiagnosticObservation, OperationalDiagnosticSignal, UpdateActionInput,
        UpdateChannel, UpdateCoordinator, UpdateError, UpdateFaultPoint, UpdateLifecycleState,
        UpdateRecoveryAction, UpdateRecoveryInput, UpdateRevocationInput,
    },
};
use tempfile::TempDir;

const NOW: u64 = 1_721_800_000_000;
const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn coordinator(temporary: &TempDir) -> UpdateCoordinator {
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    UpdateCoordinator::restore(Arc::new(database), NOW).expect("update coordinator")
}

#[test]
fn interrupted_activation_never_manufactures_current_version_and_recovers_staged_intent() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    coordinator
        .register_authoritative_compatibility_binding(
            UpdateChannel::Application,
            "c4os",
            b"c4os-local-development:1.0.0",
        )
        .expect("compiled compatibility");
    let current = coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.0",
            "test:startup",
            NOW + 1,
        )
        .expect("register authoritative version");
    let artifact = temporary.path().join("candidate.bin");
    fs::write(&artifact, b"immutable candidate bytes").expect("candidate bytes");
    let discovered = coordinator
        .discover_file_candidate(
            UpdateChannel::Application,
            "c4os",
            "1.1.0",
            &artifact,
            b"c4os-local-development:1.0.0",
            "test:discovery",
            NOW + 2,
        )
        .expect("discover candidate");
    assert!(discovered.generation > current.generation);
    let candidate_id = discovered.candidates[0].candidate_id.clone();
    let staged = coordinator
        .stage(
            &LocalUpdateStageInput {
                expected_generation: discovered.generation,
                channel: UpdateChannel::Application,
                component_id: "c4os".into(),
                candidate_id,
            },
            "test:stage",
            NOW + 3,
        )
        .expect("stage candidate");
    let activating = coordinator
        .begin_activation(
            &UpdateActionInput {
                expected_generation: staged.generation,
                channel: UpdateChannel::Application,
                component_id: "c4os".into(),
            },
            "test:activate",
            NOW + 4,
        )
        .expect("journal activation");
    assert_eq!(activating.pending_operations.len(), 1);
    drop(coordinator);
    drop(database);

    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("restart database");
    let mut restored =
        UpdateCoordinator::restore(Arc::new(database), NOW + 5).expect("restart coordinator");
    let snapshot = restored.snapshot();
    let component = &snapshot.channels[0];
    assert_eq!(component.current_version, "1.0.0");
    assert_eq!(component.candidate_version.as_deref(), Some("1.1.0"));
    assert_eq!(component.state, UpdateLifecycleState::Failed);
    assert_eq!(
        component.recovery_action,
        Some(UpdateRecoveryAction::RecoverInterrupted)
    );
    assert!(snapshot.pending_operations.is_empty());
    assert_eq!(snapshot.recovery_notices.len(), 1);

    let recovered = restored
        .recover(
            &UpdateRecoveryInput {
                expected_generation: snapshot.generation,
                channel: UpdateChannel::Application,
                component_id: "c4os".into(),
                recovery_id: snapshot.recovery_notices[0].recovery_id.clone(),
            },
            "test:recover",
            NOW + 6,
        )
        .expect("review recovery");
    assert_eq!(recovered.channels[0].state, UpdateLifecycleState::Staged);
    assert_eq!(recovered.channels[0].current_version, "1.0.0");
}

#[test]
fn update_fault_matrix_preserves_plugin_selector_truth_and_cleanup_is_ancillary() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    let plugin = coordinator
        .reconcile_plugin_component(
            "plugin.fixture",
            "1.0.0",
            None,
            None,
            None,
            false,
            None,
            UpdateLifecycleState::Current,
            "test:plugin",
            NOW + 1,
        )
        .expect("register plugin");
    let available = coordinator
        .register_extension_candidate(
            "plugin.fixture",
            "1.1.0",
            DIGEST,
            "c4os-plugin-api:1",
            "test:plugin",
            NOW + 2,
        )
        .expect("register ExtensionService candidate");
    let stage_input = LocalUpdateStageInput {
        expected_generation: available.generation,
        channel: UpdateChannel::Plugin,
        component_id: "plugin.fixture".into(),
        candidate_id: available.candidates[0].candidate_id.clone(),
    };
    coordinator.inject_fault_for_test(UpdateFaultPoint::JournalPrepare);
    assert!(matches!(
        coordinator.stage(&stage_input, "test:stage", NOW + 3),
        Err(c4os_lib::update::UpdateError::InjectedFault(
            UpdateFaultPoint::JournalPrepare
        ))
    ));
    assert_eq!(coordinator.snapshot().generation, available.generation);
    let staged = coordinator
        .stage(&stage_input, "test:stage", NOW + 4)
        .expect("stage plugin");
    let mut action = UpdateActionInput {
        expected_generation: staged.generation,
        channel: UpdateChannel::Plugin,
        component_id: "plugin.fixture".into(),
    };
    coordinator.inject_fault_for_test(UpdateFaultPoint::MigrationSnapshot);
    assert!(matches!(
        coordinator.begin_activation(&action, "test:activate", NOW + 5),
        Err(c4os_lib::update::UpdateError::InjectedFault(
            UpdateFaultPoint::MigrationSnapshot
        ))
    ));
    let activating = coordinator
        .begin_activation(&action, "test:activate", NOW + 6)
        .expect("journal plugin activation");
    action.expected_generation = activating.generation;
    for point in [
        UpdateFaultPoint::SelectorPublication,
        UpdateFaultPoint::Health,
        UpdateFaultPoint::DurableCompletion,
    ] {
        coordinator.inject_fault_for_test(point);
        assert!(matches!(
            coordinator.complete_activation(
                &action,
                "test:activate",
                true,
                None,
                NOW + 7,
            ),
            Err(c4os_lib::update::UpdateError::InjectedFault(actual)) if actual == point
        ));
        assert_eq!(coordinator.snapshot().pending_operations.len(), 1);
    }
    let activated = coordinator
        .complete_activation(&action, "test:activate", true, None, NOW + 8)
        .expect("complete after selector and health confirmation");
    assert_eq!(activated.channels[0].current_version, "1.1.0");
    assert_eq!(
        activated.channels[0].last_known_good_version.as_deref(),
        Some("1.0.0")
    );

    let rollback = UpdateActionInput {
        expected_generation: activated.generation,
        channel: UpdateChannel::Plugin,
        component_id: "plugin.fixture".into(),
    };
    coordinator.inject_fault_for_test(UpdateFaultPoint::RollbackPublication);
    assert!(matches!(
        coordinator.request_rollback(&rollback, "test:rollback", NOW + 9),
        Err(c4os_lib::update::UpdateError::InjectedFault(
            UpdateFaultPoint::RollbackPublication
        ))
    ));
    assert_eq!(coordinator.snapshot().channels[0].current_version, "1.1.0");
    let requested = coordinator
        .request_rollback(&rollback, "test:rollback", NOW + 10)
        .expect("record truthful deferred rollback");
    assert_eq!(requested.channels[0].state, UpdateLifecycleState::Failed);
    assert_eq!(requested.channels[0].current_version, "1.1.0");
    assert_eq!(
        requested.channels[0].candidate_version.as_deref(),
        Some("1.0.0")
    );

    coordinator.inject_fault_for_test(UpdateFaultPoint::Cleanup);
    let revoked = coordinator
        .revoke(
            &UpdateRevocationInput {
                expected_generation: requested.generation,
                channel: UpdateChannel::Plugin,
                component_id: "plugin.fixture".into(),
                reason_code: "user_requested".into(),
            },
            "test:revoke",
            NOW + 11,
        )
        .expect("cleanup residue must not reject committed revocation");
    assert_eq!(revoked.channels[0].state, UpdateLifecycleState::Revoked);
    assert!(
        coordinator
            .diagnostics_snapshot()
            .records
            .iter()
            .any(|record| record.component_boundary == "update-coordinator"
                && record.recovery_action == Some(UpdateRecoveryAction::RetryStage))
    );
    assert!(plugin.generation < revoked.generation);
}

#[test]
fn candidate_copy_verification_compatibility_and_diagnostic_redaction_fail_closed() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Runtime,
            "opencode",
            "1.18.3",
            "test:runtime",
            NOW + 1,
        )
        .expect("register runtime");
    let artifact = temporary.path().join("runtime.bin");
    fs::write(&artifact, b"runtime candidate").expect("candidate bytes");
    for point in [
        UpdateFaultPoint::CandidateCopy,
        UpdateFaultPoint::Verification,
        UpdateFaultPoint::Compatibility,
    ] {
        coordinator.inject_fault_for_test(point);
        assert!(matches!(
            coordinator.discover_file_candidate(
                UpdateChannel::Runtime,
                "opencode",
                "1.19.0",
                &artifact,
                b"runtime-adapter:opencode:1.18.3",
                "test:discovery",
                NOW + 2,
            ),
            Err(c4os_lib::update::UpdateError::InjectedFault(actual)) if actual == point
        ));
    }
    assert!(coordinator.snapshot().candidates.is_empty());

    coordinator
        .apply_diagnostic_preferences(Some(30), &["sensitive-host".into()], NOW + 2)
        .expect("redaction preferences");
    coordinator
        .ingest_diagnostic(
            DiagnosticCategory::Security,
            DiagnosticSeverity::Error,
            "credential",
            "Connection to sensitive-host failed under /Users/example/private",
            Some(UpdateRecoveryAction::ReviewRuntimeCrashLoop),
            "test:diagnostics",
            NOW + 3,
        )
        .expect("ingest sanitized diagnostic");
    let diagnostics = coordinator.diagnostics_snapshot();
    assert_eq!(diagnostics.schema_version, 1);
    assert!(
        diagnostics.records[0]
            .message
            .contains("[redacted diagnostic]")
    );
    let exported = coordinator
        .diagnostics_export(diagnostics.generation, "test:export", NOW + 4)
        .expect("export diagnostics");
    let json = serde_json::to_string(&exported).expect("serialize export");
    assert!(!json.contains("sensitive-host"));
    assert!(!json.contains("/Users/"));
    assert!(exported.sha256.starts_with("sha256:"));
}

#[test]
fn changed_diagnostic_preferences_replace_and_remove_persisted_rows_atomically() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    coordinator
        .ingest_diagnostic(
            DiagnosticCategory::Security,
            DiagnosticSeverity::Warning,
            "connectivity",
            "Connection to rotate-host failed",
            None,
            "test:preferences",
            NOW + 1,
        )
        .expect("ingest diagnostic before preference change");
    let diagnostic_id = coordinator.diagnostics_snapshot().records[0]
        .diagnostic_id
        .clone();
    let persisted_before = database
        .app_diagnostics(SnapshotQuery::new(10).unwrap())
        .expect("read diagnostics before preference change");
    assert!(
        persisted_before
            .iter()
            .any(|record| record.message.contains("rotate-host"))
    );

    coordinator
        .apply_diagnostic_preferences(Some(30), &["rotate-host".into()], NOW + 2)
        .expect("apply new redaction preference");
    let current = coordinator.diagnostics_snapshot();
    assert_eq!(current.records.len(), 1);
    assert!(current.records[0].message.contains("[redacted diagnostic]"));
    let persisted_after = database
        .app_diagnostics(SnapshotQuery::new(10).unwrap())
        .expect("read replaced diagnostics");
    assert_eq!(persisted_after.len(), 1);
    assert_eq!(persisted_after[0].diagnostic_id, diagnostic_id);
    assert!(persisted_after[0].message.contains("[redacted diagnostic]"));
    assert!(!persisted_after[0].message.contains("rotate-host"));

    coordinator
        .apply_diagnostic_preferences(Some(1), &["rotate-host".into()], NOW + 2 + (2 * 86_400_000))
        .expect("apply shorter retention preference");
    assert!(coordinator.diagnostics_snapshot().records.is_empty());
    assert!(
        database
            .app_diagnostics(SnapshotQuery::new(10).unwrap())
            .expect("read diagnostics after retention")
            .iter()
            .all(|record| record.diagnostic_id != diagnostic_id)
    );
}

#[test]
fn startup_diagnostics_roll_forward_after_the_bounded_journal_is_saturated() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    let boundaries = [
        "database",
        "configuration",
        "runtime-provider",
        "extension",
        "mcp",
        "browser-terminal",
        "credential",
        "workspace",
        "startup",
    ];
    for index in 0..(MAX_DIAGNOSTIC_RECORDS + boundaries.len()) {
        let boundary = boundaries[index % boundaries.len()];
        coordinator
            .ingest_diagnostic(
                DiagnosticCategory::Recovery,
                DiagnosticSeverity::Info,
                boundary,
                "Startup authority restored successfully",
                None,
                "startup:diagnostics",
                NOW + 1,
            )
            .unwrap_or_else(|error| {
                panic!("diagnostic {index} at {boundary} failed after saturation: {error:?}")
            });
    }

    let saturated = coordinator.diagnostics_snapshot();
    assert_eq!(saturated.records.len(), MAX_DIAGNOSTIC_RECORDS);
    assert!(saturated.truncated);
    assert_eq!(
        saturated.records.last().unwrap().component_boundary,
        boundaries[(MAX_DIAGNOSTIC_RECORDS + boundaries.len() - 1) % boundaries.len()]
    );
    drop(coordinator);
    drop(database);

    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("restart database");
    let restored =
        UpdateCoordinator::restore(Arc::new(database), NOW + 2).expect("restart coordinator");
    assert_eq!(
        restored.diagnostics_snapshot().records.len(),
        MAX_DIAGNOSTIC_RECORDS
    );
}

#[test]
fn operational_diagnostics_surface_bounded_current_authority_without_read_noise() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    let signals = [
        OperationalDiagnosticSignal::ConfigurationRecovery,
        OperationalDiagnosticSignal::RuntimeCrashLoop,
        OperationalDiagnosticSignal::ProviderFailure,
        OperationalDiagnosticSignal::ExtensionFailure,
        OperationalDiagnosticSignal::ExtensionRevoked,
        OperationalDiagnosticSignal::McpFailure,
        OperationalDiagnosticSignal::McpRevoked,
        OperationalDiagnosticSignal::WorkspaceRecovery,
        OperationalDiagnosticSignal::BrowserClearPending,
        OperationalDiagnosticSignal::TerminalDormantRecovery,
        OperationalDiagnosticSignal::CredentialFallback,
    ];
    let observations = signals
        .into_iter()
        .enumerate()
        .map(|(index, signal)| {
            OperationalDiagnosticObservation::from_signal(signal, &format!("authority-{index}"))
                .expect("safe operational observation")
        })
        .collect::<Vec<_>>();
    let synchronized = coordinator
        .synchronize_operational_diagnostics(&observations, "test:operational", NOW + 1)
        .expect("synchronize operational diagnostics");
    assert_eq!(synchronized.records.len(), observations.len());
    assert!(synchronized.records.iter().any(|record| {
        record.recovery_action == Some(UpdateRecoveryAction::ReviewRuntimeCrashLoop)
            && record.component_boundary == "runtime"
    }));
    assert!(synchronized.records.iter().any(|record| {
        record.component_boundary == "credential"
            && record.message.contains("explicit fallback decision")
    }));
    let generation = synchronized.generation;
    let repeated = coordinator
        .synchronize_operational_diagnostics(&observations, "test:different-correlation", NOW + 2)
        .expect("deduplicated operational diagnostics");
    assert_eq!(repeated.generation, generation);
    assert!(
        repeated
            .records
            .iter()
            .all(|record| record.correlation_id.starts_with("correlation-")
                && record.correlation_id != "test:operational")
    );
    let cleared = coordinator
        .synchronize_operational_diagnostics(&[], "test:clear", NOW + 3)
        .expect("clear resolved operational state");
    assert!(cleared.records.is_empty());
}

#[test]
fn renderer_correlation_ids_are_core_minted_before_persistence_and_export() {
    let temporary = TempDir::new().expect("temporary home");
    let home = temporary
        .path()
        .canonicalize()
        .expect("canonical temporary home");
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    let renderer_correlation = "sk-proj-secret-shaped-correlation";
    coordinator
        .ingest_diagnostic(
            DiagnosticCategory::Security,
            DiagnosticSeverity::Warning,
            "renderer-boundary",
            "A renderer-originated diagnostic was recorded",
            None,
            renderer_correlation,
            NOW + 1,
        )
        .expect("ingest renderer diagnostic");

    let snapshot = coordinator.diagnostics_snapshot();
    assert_eq!(snapshot.records.len(), 1);
    assert!(
        snapshot.records[0]
            .correlation_id
            .starts_with("correlation-")
    );
    assert_ne!(snapshot.records[0].correlation_id, renderer_correlation);
    let exported = coordinator
        .diagnostics_export(snapshot.generation, "test:export", NOW + 2)
        .expect("export diagnostics");
    assert!(
        !serde_json::to_string(&exported)
            .expect("serialize export")
            .contains(renderer_correlation)
    );
    assert!(
        !database
            .runtime_state_document("update-coordinator", "local-development")
            .expect("read update journal")
            .expect("update journal")
            .canonical_document
            .contains(renderer_correlation)
    );
}

#[test]
fn uppercase_digest_is_rejected_at_the_extension_boundary() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    coordinator
        .reconcile_plugin_component(
            "plugin.fixture",
            "1.0.0",
            None,
            None,
            None,
            false,
            None,
            UpdateLifecycleState::Current,
            "test:plugin",
            NOW + 1,
        )
        .expect("register plugin");
    assert!(matches!(
        coordinator.register_extension_candidate(
            "plugin.fixture",
            "1.1.0",
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "c4os-plugin-api:1",
            "test:plugin",
            NOW + 2,
        ),
        Err(UpdateError::InvalidInput)
    ));
}

#[test]
fn mutated_content_addressed_bytes_fail_stage_without_changing_current_truth() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    coordinator
        .register_authoritative_compatibility_binding(
            UpdateChannel::Application,
            "c4os",
            b"c4os-local-development:1.0.0",
        )
        .expect("compiled compatibility");
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.0",
            "test:startup",
            NOW + 1,
        )
        .expect("register current application");
    let artifact = temporary.path().join("candidate.bin");
    fs::write(&artifact, b"verified application candidate").expect("candidate bytes");
    let discovered = coordinator
        .discover_file_candidate(
            UpdateChannel::Application,
            "c4os",
            "1.1.0",
            &artifact,
            b"c4os-local-development:1.0.0",
            "test:discovery",
            NOW + 2,
        )
        .expect("discover candidate");
    let candidate = discovered.candidates[0].clone();
    let stored = temporary
        .path()
        .join("updates/store/sha256")
        .join(candidate.artifact_sha256.trim_start_matches("sha256:"));
    let mut permissions = fs::metadata(&stored)
        .expect("stored metadata")
        .permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&stored, permissions).expect("make test candidate writable");
    fs::write(&stored, b"mutated after discovery").expect("mutate store bytes");
    assert!(matches!(
        coordinator.stage(
            &LocalUpdateStageInput {
                expected_generation: discovered.generation,
                channel: UpdateChannel::Application,
                component_id: "c4os".into(),
                candidate_id: candidate.candidate_id,
            },
            "test:stage",
            NOW + 3,
        ),
        Err(UpdateError::InvalidState)
    ));
    let current = coordinator.snapshot();
    assert_eq!(current.channels[0].state, UpdateLifecycleState::Current);
    assert_eq!(current.channels[0].current_version, "1.0.0");
    assert!(current.channels[0].candidate_version.is_none());
}

#[cfg(unix)]
#[test]
fn content_store_symlink_rebinding_fails_closed_outside_c4os_home() {
    use std::os::unix::fs::symlink;

    let temporary = TempDir::new().expect("temporary home");
    let external = TempDir::new().expect("external directory");
    let mut coordinator = coordinator(&temporary);
    let store_root = temporary.path().join("updates/store/sha256");
    fs::remove_dir(&store_root).expect("remove empty prepared store");
    symlink(external.path(), &store_root).expect("replace store with external symlink");
    let artifact = temporary.path().join("candidate.bin");
    fs::write(&artifact, b"candidate must remain inside c4os home").expect("candidate bytes");

    assert!(matches!(
        coordinator.discover_file_candidate(
            UpdateChannel::Application,
            "c4os",
            "1.1.0",
            &artifact,
            b"c4os-local-development:1.0.0",
            "test:symlink-rebinding",
            NOW + 1,
        ),
        Err(UpdateError::InvalidState)
    ));
    assert_eq!(
        fs::read_dir(external.path())
            .expect("external directory remains readable")
            .count(),
        0
    );
    assert!(coordinator.snapshot().candidates.is_empty());
}

#[test]
fn bounded_store_cleanup_removes_abandoned_candidate_copy_residue() {
    let temporary = TempDir::new().expect("temporary home");
    let home = temporary
        .path()
        .canonicalize()
        .expect("canonical temporary home");
    let (database, _) = DatabaseActor::start(DatabaseDescriptor::app(&home)).expect("app database");
    let mut coordinator =
        UpdateCoordinator::restore(Arc::new(database), NOW).expect("update coordinator");
    let residue = home.join("updates/store/sha256/.candidate-abandoned-copy");
    fs::write(&residue, b"incomplete candidate").expect("candidate residue");
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.0",
            "test:cleanup",
            NOW + 1,
        )
        .expect("trigger bounded cleanup");
    assert!(!residue.exists());
}

#[test]
fn restart_requires_current_compiled_compatibility_before_stage() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    coordinator
        .register_authoritative_compatibility_binding(
            UpdateChannel::Runtime,
            "opencode",
            b"runtime-adapter:opencode:1.18.3",
        )
        .expect("old compiled binding");
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Runtime,
            "opencode",
            "1.18.3",
            "test:startup",
            NOW + 1,
        )
        .expect("register runtime");
    let artifact = temporary.path().join("runtime.bin");
    fs::write(&artifact, b"runtime update candidate").expect("candidate bytes");
    let discovered = coordinator
        .discover_file_candidate(
            UpdateChannel::Runtime,
            "opencode",
            "1.19.0",
            &artifact,
            b"runtime-adapter:opencode:1.18.3",
            "test:discovery",
            NOW + 2,
        )
        .expect("discover candidate");
    let candidate_id = discovered.candidates[0].candidate_id.clone();
    drop(coordinator);
    drop(database);

    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("restart database");
    let mut restored =
        UpdateCoordinator::restore(Arc::new(database), NOW + 3).expect("restart coordinator");
    restored
        .register_authoritative_compatibility_binding(
            UpdateChannel::Runtime,
            "opencode",
            b"runtime-adapter:opencode:1.18.4",
        )
        .expect("new compiled binding");
    assert!(matches!(
        restored.stage(
            &LocalUpdateStageInput {
                expected_generation: restored.snapshot().generation,
                channel: UpdateChannel::Runtime,
                component_id: "opencode".into(),
                candidate_id,
            },
            "test:stage",
            NOW + 4,
        ),
        Err(UpdateError::InvalidState)
    ));
    assert_eq!(
        restored.snapshot().channels[0].state,
        UpdateLifecycleState::Current
    );
}

#[test]
fn authoritative_plugin_rollback_is_not_relabelled_as_activation() {
    let temporary = TempDir::new().expect("temporary home");
    let mut coordinator = coordinator(&temporary);
    coordinator
        .reconcile_plugin_component(
            "plugin.fixture",
            "1.1.0",
            None,
            None,
            Some("1.0.0"),
            false,
            None,
            UpdateLifecycleState::Current,
            "test:plugin",
            NOW + 1,
        )
        .expect("register active plugin");
    let rolled_back = coordinator
        .reconcile_plugin_component(
            "plugin.fixture",
            "1.0.0",
            None,
            None,
            Some("1.0.0"),
            false,
            Some("activation_failed"),
            UpdateLifecycleState::RolledBack,
            "test:rollback",
            NOW + 2,
        )
        .expect("synchronize authoritative rollback");
    assert_eq!(
        rolled_back.channels[0].state,
        UpdateLifecycleState::RolledBack
    );
}

#[cfg(unix)]
#[test]
fn real_store_cleanup_failure_is_durable_and_does_not_rollback_authority() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.0",
            "test:startup",
            NOW + 1,
        )
        .expect("register current application");
    let store = temporary.path().join("updates/store/sha256");
    fs::create_dir_all(&store).expect("create store");
    fs::write(store.join("f".repeat(64)), b"unreferenced residue").expect("residue bytes");
    fs::set_permissions(&store, fs::Permissions::from_mode(0o500))
        .expect("make cleanup removal fail");
    let result = coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.1",
            "test:restart",
            NOW + 2,
        )
        .expect("authority transition remains committed");
    assert_eq!(result.channels[0].current_version, "1.0.1");
    assert!(
        coordinator
            .diagnostics_snapshot()
            .records
            .iter()
            .any(|record| {
                record.diagnostic_id == "diagnostic-update-cleanup-residue"
                    && record.component_boundary == "update-coordinator"
            })
    );
    fs::set_permissions(&store, fs::Permissions::from_mode(0o700))
        .expect("restore cleanup permissions");
    drop(coordinator);
    drop(database);

    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("restart database");
    let restored =
        UpdateCoordinator::restore(Arc::new(database), NOW + 3).expect("restart coordinator");
    assert_eq!(restored.snapshot().channels[0].current_version, "1.0.1");
    assert!(
        restored
            .diagnostics_snapshot()
            .records
            .iter()
            .any(|record| { record.diagnostic_id == "diagnostic-update-cleanup-residue" })
    );
}

#[test]
fn corrupt_persisted_activation_graph_fails_closed_on_restart() {
    let temporary = TempDir::new().expect("temporary home");
    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("app database");
    let database = Arc::new(database);
    let mut coordinator =
        UpdateCoordinator::restore(Arc::clone(&database), NOW).expect("coordinator");
    coordinator
        .reconcile_authoritative_component(
            UpdateChannel::Application,
            "c4os",
            "1.0.0",
            "test:startup",
            NOW + 1,
        )
        .expect("register current application");
    let record = database
        .runtime_state_document("update-coordinator", "local-development")
        .expect("read update document")
        .expect("update document");
    let corrupt_generation = record.generation + 1;
    let canonical_document = record
        .canonical_document
        .replacen(
            &format!("\"generation\":{}", record.generation),
            &format!("\"generation\":{corrupt_generation}"),
            1,
        )
        .replacen("\"state\":\"current\"", "\"state\":\"activating\"", 1);
    database
        .save_runtime_state_document(
            RuntimeStateDocumentRecord {
                document_kind: "update-coordinator".into(),
                document_id: "local-development".into(),
                generation: corrupt_generation,
                canonical_document,
                updated_at_ms: NOW + 2,
            },
            Some(record.generation),
        )
        .expect("persist corrupt fixture");
    drop(coordinator);
    drop(database);

    let (database, _) =
        DatabaseActor::start(DatabaseDescriptor::app(temporary.path())).expect("restart database");
    assert!(matches!(
        UpdateCoordinator::restore(Arc::new(database), NOW + 3),
        Err(UpdateError::InvalidState)
    ));
}
