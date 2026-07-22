use c4os_lib::runtime::capability::{
    CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor, CapabilityEvidence, CapabilityKey,
    CapabilityLayer, CapabilityState, ModelLifecycle, RouteIdentity,
};
use c4os_lib::runtime::session::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

#[derive(Default)]
struct RepositoryState {
    records: Mutex<BTreeMap<String, SessionRecord>>,
    provisional_load_barrier: Option<Arc<Barrier>>,
    synchronized_loads: AtomicUsize,
}

#[derive(Clone, Default)]
struct MemoryRepository(Arc<RepositoryState>);

impl MemoryRepository {
    fn with_first_submit_barrier() -> Self {
        Self(Arc::new(RepositoryState {
            records: Mutex::new(BTreeMap::new()),
            provisional_load_barrier: Some(Arc::new(Barrier::new(2))),
            synchronized_loads: AtomicUsize::new(0),
        }))
    }
}

impl SessionRepository for MemoryRepository {
    fn load(&self, session_id: &str) -> Result<Option<SessionRecord>, SessionRepositoryError> {
        let record = self
            .0
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .get(session_id)
            .cloned();
        if matches!(
            record.as_ref().map(|record| &record.lifecycle),
            Some(SessionLifecycle::Provisional)
        ) && self
            .0
            .provisional_load_barrier
            .as_ref()
            .is_some_and(|_| self.0.synchronized_loads.fetch_add(1, Ordering::SeqCst) < 2)
        {
            self.0
                .provisional_load_barrier
                .as_ref()
                .expect("barrier checked")
                .wait();
        }
        Ok(record)
    }

    fn list(&self) -> Result<Vec<SessionRecord>, SessionRepositoryError> {
        Ok(self
            .0
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?
            .values()
            .cloned()
            .collect())
    }

    fn create(&self, record: &SessionRecord) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .0
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        if records.contains_key(&record.session_id) {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(record.session_id.clone(), record.clone());
        Ok(())
    }

    fn compare_and_swap(
        &self,
        session_id: &str,
        expected_revision: u64,
        replacement: &SessionRecord,
    ) -> Result<(), SessionRepositoryError> {
        let mut records = self
            .0
            .records
            .lock()
            .map_err(|_| SessionRepositoryError::Unavailable)?;
        let current = records
            .get(session_id)
            .ok_or(SessionRepositoryError::Conflict)?;
        if current.revision != expected_revision {
            return Err(SessionRepositoryError::Conflict);
        }
        records.insert(session_id.into(), replacement.clone());
        Ok(())
    }
}

fn digest(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn configuration(version: u64) -> ConfigurationSnapshot {
    ConfigurationSnapshot {
        snapshot_id: format!("configuration-{version}"),
        version,
        sha256: digest('a'),
    }
}

fn resources(version: u64) -> ResourceSnapshot {
    ResourceSnapshot {
        snapshot_id: format!("resources-{version}"),
        version,
        sha256: digest('b'),
        resource_ids: vec!["skill:project/review".into(), "mcp:workspace/files".into()],
    }
}

fn capabilities(version: u64) -> CapabilitySnapshot {
    exact_capabilities_for("claude-sonnet-4", RuntimeKind::OpenCode, "1.18.3", version)
}

fn exact_capabilities(version: u64) -> CapabilitySnapshot {
    capabilities(version)
}

fn exact_capabilities_for(
    model_id: &str,
    runtime_kind: RuntimeKind,
    native_version: &str,
    version: u64,
) -> CapabilitySnapshot {
    let runtime_kind = match runtime_kind {
        RuntimeKind::OpenCode => "opencode",
        RuntimeKind::Pi => "pi",
    };
    let descriptor = CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer: CapabilityLayer::Effective,
        route: RouteIdentity {
            provider_id: "provider-anthropic".into(),
            endpoint_id: "endpoint-default".into(),
            provider_model_id: model_id.into(),
            model_revision: "2026-07-01".into(),
            adapter_kind: runtime_kind.into(),
            adapter_version: "1.0.0".into(),
            runtime_kind: runtime_kind.into(),
            native_runtime_version: native_version.into(),
            session_configuration_sha256: digest('a'),
        },
        lifecycle: ModelLifecycle::Active,
        features: BTreeMap::from([(
            CapabilityKey::Reasoning,
            CapabilityEvidence {
                state: CapabilityState::Supported,
                layer: CapabilityLayer::Effective,
                source: "c4os.effective-intersection".into(),
                checked_at_ms: 100,
                expires_at_ms: Some(1_000),
                constraints: Vec::new(),
                allowed_values: Vec::new(),
                reason: None,
            },
        )]),
        numeric_limits: BTreeMap::new(),
        raw_evidence_sha256: digest('e'),
    };
    capability_snapshot_from_effective_descriptor(descriptor, version)
        .expect("exact effective capability snapshot")
}

fn model_route(model: &str) -> ModelRouteSnapshot {
    ModelRouteSnapshot {
        route_id: format!("route:{model}"),
        provider_id: "provider-anthropic".into(),
        endpoint_id: "endpoint-default".into(),
        model_id: model.into(),
        model_revision: "2026-07-01".into(),
    }
}

fn binding() -> SessionBinding {
    SessionBinding {
        workspace_id: "workspace-1".into(),
        project_id: Some("project-1".into()),
        runtime_id: "runtime:opencode".into(),
        runtime_kind: RuntimeKind::OpenCode,
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
        initial_model_route: model_route("claude-sonnet-4"),
        initial_configuration: configuration(1),
        initial_resources: resources(1),
        initial_capabilities: capabilities(1),
        bound_at_ms: 20,
    }
}

fn attachment(id: &str) -> AttachmentSnapshot {
    AttachmentSnapshot {
        attachment_id: id.into(),
        stable_reference: format!("picker:{id}"),
        display_name: "requirements.md".into(),
        media_type: "text/markdown".into(),
        byte_length: 123,
        content_sha256: digest('d'),
        snapshot_version: 1,
        original_reference: 1,
    }
}

fn first_submission(session_id: &str, suffix: &str) -> FirstSubmission {
    FirstSubmission {
        session_id: session_id.into(),
        turn_id: format!("turn-{suffix}"),
        attempt_id: format!("attempt-{suffix}"),
        authorization_scope_id: format!("authority-{suffix}"),
        correlation_id: format!("correlation-{suffix}"),
        process_generation: 7,
        prompt: Some(format!("Implement feature {suffix}")),
        attachments: vec![attachment(&format!("attachment-{suffix}"))],
        skill_context: vec![],
        mcp_turn: None,
        binding: binding(),
        submitted_at_ms: 20,
    }
}

fn identity(suffix: &str, generation: u64) -> AttemptIdentity {
    AttemptIdentity {
        attempt_id: format!("attempt-{suffix}"),
        correlation_id: format!("correlation-{suffix}"),
        process_generation: generation,
    }
}

fn promoted() -> (SessionService<MemoryRepository>, SessionRecord) {
    let service = SessionService::new(MemoryRepository::default());
    service.create_provisional("session-1", 1).unwrap();
    let record = service
        .submit_first(first_submission("session-1", "1"))
        .unwrap();
    (service, record)
}

fn finish_failed(service: &SessionService<MemoryRepository>, error_code: &str) -> SessionRecord {
    service
        .finish_attempt(
            "session-1",
            &identity("1", 7),
            TerminalAttemptOutcome::Failed {
                error_code: error_code.into(),
            },
            30,
        )
        .unwrap()
}

#[test]
fn invalid_empty_submission_leaves_provisional_session_unchanged() {
    let repository = MemoryRepository::default();
    let service = SessionService::new(repository.clone());
    let initial = service.create_provisional("session-1", 1).unwrap();
    assert!(
        repository.load("session-1").unwrap().is_none(),
        "a blank Chat must not have a durable repository record"
    );
    let mut submission = first_submission("session-1", "1");
    submission.prompt = Some(" \n\t ".into());
    submission.attachments.clear();

    assert_eq!(
        service.submit_first(submission),
        Err(SessionError::EmptySubmission)
    );
    assert_eq!(service.session("session-1").unwrap(), initial);
    service.discard_provisional("session-1").unwrap();
    assert_eq!(service.session("session-1"), Err(SessionError::NotFound));
}

#[test]
fn attachment_only_submission_promotes_and_captures_one_immutable_binding() {
    let service = SessionService::new(MemoryRepository::default());
    service.create_provisional("session-1", 1).unwrap();
    let mut submission = first_submission("session-1", "1");
    submission.prompt = None;
    let expected_binding = submission.binding.clone();
    let expected_attachment = submission.attachments[0].clone();

    let promoted = service.submit_first(submission).unwrap();

    assert_eq!(promoted.binding(), Some(&expected_binding));
    assert_eq!(promoted.turns[0].attachments, vec![expected_attachment]);
    assert_eq!(promoted.title.as_deref(), Some("requirements.md"));
    assert_eq!(
        promoted.attempts[0].context,
        AttemptContextSnapshot::from_binding(&expected_binding)
    );
}

#[test]
fn duplicate_child_identifiers_are_rejected_without_replacing_prior_records() {
    let (service, promoted) = promoted();
    finish_failed(&service, "provider-unavailable");
    let duplicate = RetryRequest {
        session_id: "session-1".into(),
        parent_attempt_id: "attempt-1".into(),
        attempt_id: "attempt-1".into(),
        authorization_scope_id: "authority-new".into(),
        correlation_id: "correlation-new".into(),
        process_generation: 8,
        context: AttemptContextSnapshot::from_binding(promoted.binding().unwrap()),
        automatic: false,
        reviewed_unknown_effect: false,
        created_at_ms: 40,
    };

    assert_eq!(
        service.retry(duplicate),
        Err(SessionError::DuplicateIdentifier("attempt id"))
    );
    let after = service.session("session-1").unwrap();
    assert_eq!(after.attempts.len(), 1);
    assert_eq!(after.turns, promoted.turns);
}

#[test]
fn duplicate_attachment_ids_within_one_immutable_turn_are_rejected() {
    let service = SessionService::new(MemoryRepository::default());
    service.create_provisional("session-1", 1).unwrap();
    let mut submission = first_submission("session-1", "1");
    submission
        .attachments
        .push(submission.attachments[0].clone());

    assert_eq!(
        service.submit_first(submission),
        Err(SessionError::DuplicateIdentifier("attachment id"))
    );
}

#[test]
fn concurrent_first_submit_race_has_exactly_one_durable_winner() {
    let repository = MemoryRepository::with_first_submit_barrier();
    let service = Arc::new(SessionService::new(repository));
    service.create_provisional("session-race", 1).unwrap();
    let first = Arc::clone(&service);
    let second = Arc::clone(&service);

    let first =
        std::thread::spawn(move || first.submit_first(first_submission("session-race", "a")));
    let second =
        std::thread::spawn(move || second.submit_first(first_submission("session-race", "b")));
    let outcomes = [first.join().unwrap(), second.join().unwrap()];

    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert!(
        outcomes
            .iter()
            .any(|outcome| matches!(outcome, Err(SessionError::AlreadyBound)))
    );
    let stored = service.session("session-race").unwrap();
    assert_eq!(stored.turns.len(), 1);
    assert_eq!(stored.attempts.len(), 1);
    assert_eq!(stored.attempts[0].turn_id, stored.turns[0].turn_id);
}

#[test]
fn later_defaults_cannot_mutate_session_binding_or_cross_runtime() {
    let (service, promoted) = promoted();
    finish_failed(&service, "provider-unavailable");
    let original_binding = promoted.binding().unwrap().clone();
    let mut changed_context = AttemptContextSnapshot::from_binding(&original_binding);
    changed_context.runtime_id = "runtime:pi".into();
    changed_context.runtime_kind = RuntimeKind::Pi;
    changed_context.adapter.adapter_id = "adapter:pi".into();
    changed_context.adapter.native_version = "0.80.10".into();
    changed_context.capabilities =
        exact_capabilities_for("claude-sonnet-4", RuntimeKind::Pi, "0.80.10", 1);

    assert_eq!(
        service.retry(RetryRequest {
            session_id: "session-1".into(),
            parent_attempt_id: "attempt-1".into(),
            attempt_id: "attempt-2".into(),
            authorization_scope_id: "authority-2".into(),
            correlation_id: "correlation-2".into(),
            process_generation: 9,
            context: changed_context,
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: 40,
        }),
        Err(SessionError::CrossRuntimeMigration)
    );
    assert_eq!(
        service.session("session-1").unwrap().binding(),
        Some(&original_binding)
    );
}

#[test]
fn fresh_turn_may_capture_current_snapshots_but_not_runtime_identity() {
    let (service, promoted) = promoted();
    service
        .finish_attempt(
            "session-1",
            &identity("1", 7),
            TerminalAttemptOutcome::Completed,
            30,
        )
        .unwrap();
    let mut context = AttemptContextSnapshot::from_binding(promoted.binding().unwrap());
    context.model_route = model_route("claude-opus-4");
    context.configuration = configuration(2);
    context.resources = resources(2);
    context.capabilities =
        exact_capabilities_for("claude-opus-4", RuntimeKind::OpenCode, "1.18.3", 2);

    let updated = service
        .submit_turn(TurnSubmission {
            session_id: "session-1".into(),
            turn_id: "turn-2".into(),
            attempt_id: "attempt-2".into(),
            authorization_scope_id: "authority-2".into(),
            correlation_id: "correlation-2".into(),
            process_generation: 8,
            prompt: Some("Use the current approved route".into()),
            attachments: Vec::new(),
            skill_context: Vec::new(),
            reply_context: None,
            mcp_turn: None,
            context: context.clone(),
            submitted_at_ms: 40,
        })
        .unwrap();

    assert_eq!(updated.attempts[1].context, context);
    assert_eq!(updated.binding(), promoted.binding());
    assert_eq!(updated.turns[0], promoted.turns[0]);
}

#[test]
fn stale_correlation_generation_and_sequence_events_fail_closed() {
    let (service, _) = promoted();
    let event = |sequence, correlation: &str, generation| RunEventRecord {
        sequence,
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        runtime_id: "runtime:opencode".into(),
        environment_id: "environment:local".into(),
        correlation_id: correlation.into(),
        process_generation: generation,
        kind: RunEventKind::TextDelta,
        payload: "hello".into(),
        recorded_at_ms: 21,
    };

    assert_eq!(
        service.append_event(
            "session-1",
            &identity("1", 7),
            event(1, "correlation-stale", 7),
        ),
        Err(SessionError::StaleCorrelation)
    );
    assert_eq!(
        service.append_event("session-1", &identity("1", 7), event(1, "correlation-1", 6),),
        Err(SessionError::StaleGeneration)
    );
    assert_eq!(
        service.append_event("session-1", &identity("1", 7), event(2, "correlation-1", 7),),
        Err(SessionError::StaleEventSequence)
    );
    let mut wrong_runtime = event(1, "correlation-1", 7);
    wrong_runtime.runtime_id = "runtime:pi".into();
    assert_eq!(
        service.append_event("session-1", &identity("1", 7), wrong_runtime),
        Err(SessionError::StaleRunIdentity)
    );
    assert!(
        service.session("session-1").unwrap().attempts[0]
            .events
            .is_empty()
    );
}

#[test]
fn accepted_events_are_append_only_and_bounded() {
    let (service, _) = promoted();
    let accepted = service
        .append_event(
            "session-1",
            &identity("1", 7),
            RunEventRecord {
                sequence: 1,
                session_id: "session-1".into(),
                turn_id: "turn-1".into(),
                attempt_id: "attempt-1".into(),
                runtime_id: "runtime:opencode".into(),
                environment_id: "environment:local".into(),
                correlation_id: "correlation-1".into(),
                process_generation: 7,
                kind: RunEventKind::WorkActivity,
                payload: "reading".into(),
                recorded_at_ms: 21,
            },
        )
        .unwrap();
    assert!(matches!(
        accepted.attempts[0].status,
        RunAttemptStatus::Streaming { .. }
    ));

    let oversized = RunEventRecord {
        sequence: 2,
        session_id: "session-1".into(),
        turn_id: "turn-1".into(),
        attempt_id: "attempt-1".into(),
        runtime_id: "runtime:opencode".into(),
        environment_id: "environment:local".into(),
        correlation_id: "correlation-1".into(),
        process_generation: 7,
        kind: RunEventKind::TextDelta,
        payload: "x".repeat(MAX_RUN_EVENT_BYTES + 1),
        recorded_at_ms: 22,
    };
    assert_eq!(
        service.append_event("session-1", &identity("1", 7), oversized),
        Err(SessionError::BoundExceeded("run event payload"))
    );
    assert_eq!(
        service.session("session-1").unwrap().attempts[0].events,
        accepted.attempts[0].events
    );
}

#[test]
fn record_validation_rejects_event_count_above_the_fixed_bound() {
    let (_, mut record) = promoted();
    record.attempts[0].events = (1..=(MAX_RUN_EVENTS_PER_ATTEMPT as u64 + 1))
        .map(|sequence| RunEventRecord {
            sequence,
            session_id: "session-1".into(),
            turn_id: "turn-1".into(),
            attempt_id: "attempt-1".into(),
            runtime_id: "runtime:opencode".into(),
            environment_id: "environment:local".into(),
            correlation_id: "correlation-1".into(),
            process_generation: 7,
            kind: RunEventKind::Usage,
            payload: String::new(),
            recorded_at_ms: 21 + sequence,
        })
        .collect();

    assert_eq!(
        record.validate(),
        Err(SessionError::BoundExceeded("run events"))
    );
}

#[test]
fn cancellation_targets_only_the_active_attempt_and_becomes_terminal() {
    let (service, _) = promoted();
    assert_eq!(
        service.request_cancellation("session-1", &identity("other", 7), 22),
        Err(SessionError::NotActiveAttempt)
    );
    let requested = service
        .request_cancellation("session-1", &identity("1", 7), 23)
        .unwrap();
    assert!(matches!(
        requested.attempts[0].status,
        RunAttemptStatus::CancellationRequested {
            requested_at_ms: 23
        }
    ));
    let cancelled = service
        .finish_attempt(
            "session-1",
            &identity("1", 7),
            TerminalAttemptOutcome::Cancelled,
            24,
        )
        .unwrap();
    assert!(matches!(
        cancelled.attempts[0].status,
        RunAttemptStatus::Cancelled {
            cancelled_at_ms: 24
        }
    ));
    assert_eq!(cancelled.active_attempt_id, None);
}

#[test]
fn retry_preserves_parent_and_turn_and_uses_fresh_authority_scope() {
    let (service, promoted) = promoted();
    let failed = finish_failed(&service, "provider-unavailable");
    let parent_before_retry = failed.attempts[0].clone();
    let turn_before_retry = promoted.turns[0].clone();
    let mut current_context = AttemptContextSnapshot::from_binding(promoted.binding().unwrap());
    current_context.configuration = configuration(2);

    let retried = service
        .retry(RetryRequest {
            session_id: "session-1".into(),
            parent_attempt_id: "attempt-1".into(),
            attempt_id: "attempt-2".into(),
            authorization_scope_id: "authority-2".into(),
            correlation_id: "correlation-2".into(),
            process_generation: 8,
            context: current_context.clone(),
            automatic: true,
            reviewed_unknown_effect: false,
            created_at_ms: 40,
        })
        .unwrap();

    assert_eq!(retried.turns, vec![turn_before_retry]);
    assert_eq!(retried.attempts[0], parent_before_retry);
    assert_eq!(
        retried.attempts[1].parent_attempt_id.as_deref(),
        Some("attempt-1")
    );
    assert_eq!(retried.attempts[1].authorization_scope_id, "authority-2");
    assert!(retried.attempts[1].side_effects.is_empty());
    assert_eq!(retried.attempts[1].context, current_context);
}

#[test]
fn unknown_effect_blocks_retry_until_explicit_review_and_never_allows_auto_retry() {
    let (service, promoted) = promoted();
    service
        .set_side_effect_state(
            "session-1",
            &identity("1", 7),
            SideEffectState::Started {
                action_id: "action-1".into(),
                idempotent: false,
            },
            22,
        )
        .unwrap();
    service
        .finish_attempt(
            "session-1",
            &identity("1", 7),
            TerminalAttemptOutcome::Interrupted {
                reason_code: "worker-crash".into(),
            },
            30,
        )
        .unwrap();
    let retry = |reviewed, automatic| RetryRequest {
        session_id: "session-1".into(),
        parent_attempt_id: "attempt-1".into(),
        attempt_id: "attempt-2".into(),
        authorization_scope_id: "authority-2".into(),
        correlation_id: "correlation-2".into(),
        process_generation: 8,
        context: AttemptContextSnapshot::from_binding(promoted.binding().unwrap()),
        automatic,
        reviewed_unknown_effect: reviewed,
        created_at_ms: 40,
    };

    assert_eq!(
        service.retry(retry(false, false)),
        Err(SessionError::UnknownEffectReviewRequired)
    );
    assert_eq!(
        service.retry(retry(true, true)),
        Err(SessionError::UnsafeAutomaticRetry)
    );
    assert!(service.retry(retry(true, false)).is_ok());
}

#[test]
fn automatic_retry_requires_no_effect_or_proven_idempotent_noncompletion() {
    let (service, promoted) = promoted();
    service
        .set_side_effect_state(
            "session-1",
            &identity("1", 7),
            SideEffectState::Started {
                action_id: "action-1".into(),
                idempotent: true,
            },
            22,
        )
        .unwrap();
    service
        .set_side_effect_state(
            "session-1",
            &identity("1", 7),
            SideEffectState::ProvenNotCompleted {
                action_id: "action-1".into(),
                idempotent: true,
            },
            23,
        )
        .unwrap();
    finish_failed(&service, "network-loss");

    let retried = service.retry(RetryRequest {
        session_id: "session-1".into(),
        parent_attempt_id: "attempt-1".into(),
        attempt_id: "attempt-2".into(),
        authorization_scope_id: "authority-2".into(),
        correlation_id: "correlation-2".into(),
        process_generation: 8,
        context: AttemptContextSnapshot::from_binding(promoted.binding().unwrap()),
        automatic: true,
        reviewed_unknown_effect: false,
        created_at_ms: 40,
    });
    assert!(retried.is_ok());
}

#[test]
fn restart_recovery_interrupts_active_attempt_and_review_gates_started_effect() {
    let (service, promoted) = promoted();
    service
        .set_side_effect_state(
            "session-1",
            &identity("1", 7),
            SideEffectState::Started {
                action_id: "action-1".into(),
                idempotent: false,
            },
            22,
        )
        .unwrap();

    let recovered = service.recover_interrupted("session-1", 50).unwrap();
    assert_eq!(recovered.active_attempt_id, None);
    assert!(matches!(
        recovered.attempts[0].status,
        RunAttemptStatus::Interrupted { .. }
    ));
    assert_eq!(
        recovered.attempts[0].side_effects,
        vec![SideEffectState::Unknown {
            action_id: "action-1".into()
        }]
    );
    assert_eq!(
        service.retry(RetryRequest {
            session_id: "session-1".into(),
            parent_attempt_id: "attempt-1".into(),
            attempt_id: "attempt-2".into(),
            authorization_scope_id: "authority-2".into(),
            correlation_id: "correlation-2".into(),
            process_generation: 8,
            context: AttemptContextSnapshot::from_binding(promoted.binding().unwrap()),
            automatic: false,
            reviewed_unknown_effect: false,
            created_at_ms: 60,
        }),
        Err(SessionError::UnknownEffectReviewRequired)
    );
}

#[test]
fn startup_recovery_enumerates_every_active_durable_session() {
    let service = SessionService::new(MemoryRepository::default());
    for (session_id, suffix) in [("session-1", "1"), ("session-2", "2")] {
        service.create_provisional(session_id, 1).unwrap();
        service
            .submit_first(first_submission(session_id, suffix))
            .unwrap();
    }

    let recovered = service.recover_all_interrupted(50).unwrap();
    assert_eq!(recovered.len(), 2);
    assert!(
        recovered
            .iter()
            .all(|record| record.active_attempt_id.is_none())
    );
    assert!(recovered.iter().all(|record| matches!(
        record.attempts[0].status,
        RunAttemptStatus::Interrupted { .. }
    )));
    assert!(service.recover_all_interrupted(60).unwrap().is_empty());
}

#[test]
fn every_ambiguous_effect_survives_failed_and_cancelled_terminal_paths() {
    for outcome in [
        TerminalAttemptOutcome::Failed {
            error_code: "worker-failed".into(),
        },
        TerminalAttemptOutcome::Cancelled,
    ] {
        let (service, promoted) = promoted();
        for action_id in ["action-1", "action-2"] {
            service
                .set_side_effect_state(
                    "session-1",
                    &identity("1", 7),
                    SideEffectState::Started {
                        action_id: action_id.into(),
                        idempotent: false,
                    },
                    22,
                )
                .unwrap();
        }
        let terminal = service
            .finish_attempt("session-1", &identity("1", 7), outcome.clone(), 30)
            .unwrap();
        assert_eq!(
            terminal.attempts[0].side_effects,
            vec![
                SideEffectState::Unknown {
                    action_id: "action-1".into()
                },
                SideEffectState::Unknown {
                    action_id: "action-2".into()
                },
            ]
        );
        assert_eq!(
            service.retry(RetryRequest {
                session_id: "session-1".into(),
                parent_attempt_id: "attempt-1".into(),
                attempt_id: "attempt-2".into(),
                authorization_scope_id: "authority-2".into(),
                correlation_id: "correlation-2".into(),
                process_generation: 8,
                context: AttemptContextSnapshot::from_binding(promoted.binding().unwrap()),
                automatic: false,
                reviewed_unknown_effect: false,
                created_at_ms: 40,
            }),
            Err(SessionError::UnknownEffectReviewRequired)
        );
    }
}

#[test]
fn effective_capability_descriptor_round_trips_and_tampering_fails_closed() {
    let service = SessionService::new(MemoryRepository::default());
    service.create_provisional("session-exact", 1).unwrap();
    let mut submission = first_submission("session-exact", "exact");
    submission.binding.initial_capabilities = exact_capabilities(17);
    let expected_descriptor = submission
        .binding
        .initial_capabilities
        .effective_descriptor
        .clone();
    let record = service.submit_first(submission).unwrap();

    let json = serde_json::to_string(&record).unwrap();
    let restored: SessionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, record);
    assert_eq!(
        restored
            .binding()
            .unwrap()
            .initial_capabilities
            .effective_descriptor,
        expected_descriptor
    );
    assert_eq!(
        restored.attempts[0]
            .context
            .capabilities
            .effective_descriptor,
        expected_descriptor
    );
    restored.validate().unwrap();

    let mut missing_binding_descriptor = serde_json::to_value(&record).unwrap();
    missing_binding_descriptor
        .pointer_mut("/lifecycle/binding/initialCapabilities")
        .and_then(serde_json::Value::as_object_mut)
        .expect("serialized binding capabilities")
        .remove("effectiveDescriptor");
    let missing_binding_descriptor: SessionRecord =
        serde_json::from_value(missing_binding_descriptor).unwrap();
    assert_eq!(
        missing_binding_descriptor.validate(),
        Err(SessionError::InvalidRecord("capability descriptor"))
    );

    let mut missing_attempt_descriptor = serde_json::to_value(&record).unwrap();
    missing_attempt_descriptor
        .pointer_mut("/attempts/0/context/capabilities")
        .and_then(serde_json::Value::as_object_mut)
        .expect("serialized attempt capabilities")
        .remove("effectiveDescriptor");
    let missing_attempt_descriptor: SessionRecord =
        serde_json::from_value(missing_attempt_descriptor).unwrap();
    assert_eq!(
        missing_attempt_descriptor.validate(),
        Err(SessionError::InvalidRecord("capability descriptor"))
    );

    let mut downgraded_schema = record.clone();
    downgraded_schema.schema_version = 2;
    assert_eq!(
        downgraded_schema.validate(),
        Err(SessionError::UnsupportedSchema)
    );

    let mut descriptor_tampered = restored.clone();
    descriptor_tampered.attempts[0]
        .context
        .capabilities
        .effective_descriptor
        .as_mut()
        .unwrap()
        .lifecycle = ModelLifecycle::Preview;
    assert_eq!(
        descriptor_tampered.validate(),
        Err(SessionError::InvalidRecord("capability snapshot"))
    );

    let mut hash_tampered = restored.clone();
    hash_tampered.attempts[0].context.capabilities.sha256 = digest('f');
    assert_eq!(
        hash_tampered.validate(),
        Err(SessionError::InvalidRecord("capability snapshot"))
    );

    let mut flattened_tampered = restored;
    flattened_tampered.attempts[0]
        .context
        .capabilities
        .capabilities[0]
        .support = CapabilitySupport::Unsupported;
    assert_eq!(
        flattened_tampered.validate(),
        Err(SessionError::InvalidRecord("capability snapshot"))
    );
}

#[test]
fn persisted_record_round_trip_is_strict_and_recovery_ready() {
    let (_, record) = promoted();
    let json = serde_json::to_string(&record).unwrap();
    let restored: SessionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, record);
    restored.validate().unwrap();

    let mut value = serde_json::to_value(&record).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<SessionRecord>(value).is_err());
}
