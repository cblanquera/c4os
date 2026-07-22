use c4os_lib::core::database::{
    DatabaseActor, DatabaseDescriptor, SecurityJournalRecord, SnapshotQuery,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn start_app(temp: &TempDir) -> (DatabaseDescriptor, DatabaseActor) {
    let descriptor = DatabaseDescriptor::app(temp.path());
    let (actor, report) = DatabaseActor::start(descriptor.clone()).expect("app database");
    assert_eq!(report.current_version, 8);
    (descriptor, actor)
}

fn record(state: &str, at: u64) -> SecurityJournalRecord {
    let canonical_document = serde_json::json!({
        "schemaVersion": 1,
        "recordKind": "action-intent",
        "recordId": "intent-1",
        "runId": "run-1",
        "actionId": "action-1",
        "state": state,
        "recordedAtMs": at,
        "payload": {
            "payloadSchema": "action-intent-v1",
            "state": state,
            "opaqueReference": "credential:opaque"
        }
    })
    .to_string();
    SecurityJournalRecord {
        record_kind: "action-intent".into(),
        record_id: "intent-1".into(),
        run_id: "run-1".into(),
        action_id: "action-1".into(),
        state: state.into(),
        canonical_document,
        recorded_at_ms: at,
    }
}

fn generic_record(
    kind: &str,
    id: &str,
    run_id: &str,
    action_id: &str,
    state: &str,
    at: u64,
) -> SecurityJournalRecord {
    SecurityJournalRecord {
        record_kind: kind.into(),
        record_id: id.into(),
        run_id: run_id.into(),
        action_id: action_id.into(),
        state: state.into(),
        canonical_document: serde_json::json!({
            "schemaVersion": 1,
            "recordKind": kind,
            "recordId": id,
            "runId": run_id,
            "actionId": action_id,
            "state": state,
            "recordedAtMs": at,
            "payload": {
                "payloadSchema": format!("{kind}-v1"),
                "state": state,
                "safeDigest": format!("sha256:{:064x}", at)
            }
        })
        .to_string(),
        recorded_at_ms: at,
    }
}

#[test]
fn current_security_state_and_every_transition_survive_restart() {
    let temp = TempDir::new().expect("temporary directory");
    let (descriptor, actor) = start_app(&temp);
    actor
        .save_security_record(record("proposed", 10))
        .expect("persist pre-effect intent");
    actor
        .save_security_record(record("effect-started", 11))
        .expect("persist started state");

    let current = actor
        .security_records(SnapshotQuery::new(10).expect("query"))
        .expect("current security records");
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].state, "effect-started");
    let events = actor
        .security_events(SnapshotQuery::new(10).expect("query"))
        .expect("security event journal");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].state, "effect-started");
    assert_eq!(events[1].state, "proposed");
    drop(actor);

    let (actor, report) = DatabaseActor::start(descriptor).expect("reopen app database");
    assert_eq!(report.previous_version, 8);
    assert_eq!(
        actor
            .security_records(SnapshotQuery::new(10).expect("query"))
            .expect("restart current state")[0]
            .state,
        "effect-started"
    );
    assert_eq!(
        actor
            .security_events(SnapshotQuery::new(10).expect("query"))
            .expect("restart events")
            .len(),
        2
    );
}

#[test]
fn raw_credentials_are_rejected_before_the_journal_write() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, actor) = start_app(&temp);
    let mut unsafe_record = record("proposed", 10);
    unsafe_record.canonical_document = serde_json::json!({
        "schemaVersion": 1,
        "recordKind": "action-intent",
        "recordId": "intent-1",
        "runId": "run-1",
        "actionId": "action-1",
        "state": "proposed",
        "recordedAtMs": 10,
        "payload": {
            "payloadSchema": "action-intent-v1",
            "password": "do-not-store",
            "state": "proposed"
        }
    })
    .to_string();
    let error = actor
        .save_security_record(unsafe_record)
        .expect_err("raw password must fail closed");
    assert!(error.to_string().contains("credential-like"));
    assert!(
        actor
            .security_events(SnapshotQuery::new(10).expect("query"))
            .expect("empty journal")
            .is_empty()
    );
}

#[test]
fn tampered_security_documents_fail_digest_validation() {
    let temp = TempDir::new().expect("temporary directory");
    let (descriptor, actor) = start_app(&temp);
    actor
        .save_security_record(record("proposed", 10))
        .expect("persist record");
    drop(actor);

    let connection = Connection::open(&descriptor.path).expect("open for corruption");
    connection
        .execute(
            "UPDATE security_records
             SET canonical_document = replace(canonical_document, 'credential:opaque', 'credential:mutated')",
            [],
        )
        .expect("tamper without digest update");
    drop(connection);

    let error = match DatabaseActor::start(descriptor) {
        Ok(_) => panic!("digest mismatch must fail closed on restart"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("digest mismatch"));
}

#[test]
fn terminal_state_cannot_regress_and_record_identity_cannot_rebind() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, actor) = start_app(&temp);
    actor
        .save_security_record(generic_record(
            "authorization",
            "authorization-1",
            "run-1",
            "action-1",
            "issued",
            10,
        ))
        .expect("issued authorization");
    actor
        .save_security_record(generic_record(
            "authorization",
            "authorization-1",
            "run-1",
            "action-1",
            "consumed",
            11,
        ))
        .expect("consume authorization");

    let regression = actor
        .save_security_record(generic_record(
            "authorization",
            "authorization-1",
            "run-1",
            "action-1",
            "issued",
            12,
        ))
        .expect_err("terminal authorization cannot regress");
    assert!(regression.to_string().contains("invalid authorization"));
    let rebind = actor
        .save_security_record(generic_record(
            "authorization",
            "authorization-1",
            "run-2",
            "action-2",
            "expired",
            12,
        ))
        .expect_err("record identity cannot be rebound");
    assert!(rebind.to_string().contains("cannot be rebound"));
}

#[test]
fn invalid_member_rolls_back_an_entire_multi_prompt_transition() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, actor) = start_app(&temp);
    actor
        .save_security_records(vec![
            generic_record(
                "approval-prompt",
                "prompt-1",
                "run-1",
                "action-1",
                "pending",
                10,
            ),
            generic_record(
                "approval-prompt",
                "prompt-2",
                "run-1",
                "action-2",
                "queued",
                10,
            ),
        ])
        .expect("initial queue");
    let error = actor
        .save_security_records(vec![
            generic_record(
                "approval-prompt",
                "prompt-1",
                "run-1",
                "action-1",
                "approved",
                11,
            ),
            // A no-op queued -> queued save is invalid and must roll back prompt-1 too.
            generic_record(
                "approval-prompt",
                "prompt-2",
                "run-1",
                "action-2",
                "queued",
                11,
            ),
        ])
        .expect_err("invalid member must roll back batch");
    assert!(error.to_string().contains("invalid approval-prompt"));
    let states = actor
        .security_records(SnapshotQuery::new(10).expect("query"))
        .expect("current states");
    assert!(
        states
            .iter()
            .any(|record| record.record_id == "prompt-1" && record.state == "pending")
    );
    assert!(
        states
            .iter()
            .any(|record| record.record_id == "prompt-2" && record.state == "queued")
    );
    assert_eq!(
        actor
            .security_events(SnapshotQuery::new(10).expect("query"))
            .expect("only initial events")
            .len(),
        2
    );
}

#[test]
fn append_only_audit_pages_have_stable_exclusive_cursors() {
    let temp = TempDir::new().expect("temporary directory");
    let (_, actor) = start_app(&temp);
    for index in 0..7_u64 {
        actor
            .save_security_record(generic_record(
                "action-decision",
                &format!("decision-{index}"),
                &format!("run-{index}"),
                &format!("action-{index}"),
                "allow",
                100 + index,
            ))
            .expect("decision event");
    }
    let query = SnapshotQuery::new(3).expect("page query");
    let first = actor.security_event_page(None, query).expect("first page");
    assert_eq!(first.events.len(), 3);
    let second = actor
        .security_event_page(first.next_before_event_id, query)
        .expect("second page");
    assert_eq!(second.events.len(), 3);
    let third = actor
        .security_event_page(second.next_before_event_id, query)
        .expect("third page");
    assert_eq!(third.events.len(), 1);
    assert!(third.next_before_event_id.is_none());
    let ids = first
        .events
        .iter()
        .chain(&second.events)
        .chain(&third.events)
        .map(|event| event.event_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 7);
}

#[test]
fn app_schema_three_migrates_to_app_owned_security_journal() {
    let temp = TempDir::new().expect("temporary directory");
    let descriptor = DatabaseDescriptor::app(temp.path());
    {
        let (_actor, report) = DatabaseActor::start(descriptor.clone()).expect("current app db");
        assert_eq!(report.current_version, 8);
    }
    let connection = Connection::open(&descriptor.path).expect("open seed database");
    connection
        .execute_batch(
            "DROP TABLE security_events;
             DROP TABLE security_records;
             DROP TABLE runtime_state_documents;
             DROP TABLE extension_events;
             DROP TABLE extension_state;
             DROP TABLE mcp_events;
             DROP TABLE mcp_state;
             PRAGMA user_version = 3;",
        )
        .expect("restore exact pre-security schema");
    drop(connection);

    let (actor, report) = DatabaseActor::start(descriptor).expect("migrate v3 to current");
    assert_eq!(report.previous_version, 3);
    assert_eq!(report.current_version, 8);
    assert!(report.backup_path.is_some_and(|path| path.exists()));
    assert!(
        actor
            .security_records(SnapshotQuery::new(1).expect("query"))
            .expect("new security table")
            .is_empty()
    );
}

#[test]
fn event_metadata_tampering_is_detected_when_that_page_is_read() {
    let temp = TempDir::new().expect("temporary directory");
    let (descriptor, actor) = start_app(&temp);
    actor
        .save_security_record(generic_record(
            "action-decision",
            "decision-1",
            "run-1",
            "action-1",
            "allow",
            10,
        ))
        .expect("decision");
    drop(actor);
    let connection = Connection::open(&descriptor.path).expect("open for event tamper");
    connection
        .execute("UPDATE security_events SET recorded_at_ms = 11", [])
        .expect("tamper event metadata");
    drop(connection);

    let (actor, _) = DatabaseActor::start(descriptor).expect("bounded startup skips history scan");
    let error = actor
        .security_event_page(None, SnapshotQuery::new(10).expect("query"))
        .expect_err("event page must validate its envelope and digest");
    assert!(
        error.to_string().contains("indexed columns")
            || error.to_string().contains("digest mismatch")
    );
}
