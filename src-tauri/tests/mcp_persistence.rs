use c4os_lib::core::database::{
    DatabaseActor, DatabaseDescriptor, DatabaseError, McpEventRecord, McpStateDocumentRecord,
    SnapshotQuery,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn state(generation: u64, event_id: u64) -> McpStateDocumentRecord {
    McpStateDocumentRecord {
        generation,
        canonical_document: serde_json::json!({
            "schemaVersion": 1,
            "generation": generation,
            "servers": [{
                "serverId": "fixture",
                "bearer": { "kind": "vault", "credentialReference": "credential:opaque" }
            }],
            "lastEventId": event_id
        })
        .to_string(),
        updated_at_ms: 1_721_500_000_000 + generation,
    }
}

fn event(event_id: u64, generation: u64, kind: &str) -> McpEventRecord {
    let occurred_at_ms = 1_721_500_000_000 + generation;
    let canonical_document = serde_json::json!({
        "schemaVersion": 1,
        "eventId": event_id,
        "generation": generation,
        "lifecycleGeneration": generation,
        "operationId": format!("operation-{event_id}"),
        "serverId": "fixture",
        "eventKind": kind,
        "target": null,
        "result": "succeeded",
        "detail": null,
        "occurredAtMs": occurred_at_ms
    })
    .to_string();
    McpEventRecord {
        event_id,
        generation,
        lifecycle_generation: generation,
        operation_id: format!("operation-{event_id}"),
        server_id: "fixture".into(),
        event_kind: kind.into(),
        target: None,
        result: "succeeded".into(),
        canonical_document,
        occurred_at_ms,
    }
}

#[test]
fn mcp_transition_is_atomic_cas_and_restart_authoritative() {
    let temporary = TempDir::new().expect("temporary home");
    let descriptor = DatabaseDescriptor::app(temporary.path());
    {
        let (database, report) =
            DatabaseActor::start(descriptor.clone()).expect("open app database");
        assert_eq!(report.current_version, 9);
        assert_eq!(database.mcp_state_document().unwrap(), None);

        database
            .save_mcp_transition(state(1, 1), event(1, 1, "definition-saved"), None)
            .expect("bootstrap MCP transition");
        assert_eq!(database.mcp_state_document().unwrap(), Some(state(1, 1)));

        let conflict = database
            .save_mcp_transition(state(2, 2), event(2, 2, "server-enabled"), Some(99))
            .expect_err("stale MCP transition must fail");
        assert!(matches!(conflict, DatabaseError::Conflict(_)));
        assert_eq!(database.mcp_state_document().unwrap(), Some(state(1, 1)));

        database
            .save_mcp_transition(state(2, 2), event(2, 2, "server-enabled"), Some(1))
            .expect("second MCP transition");
        let page = database
            .mcp_event_page(None, SnapshotQuery::new(1).unwrap())
            .unwrap();
        assert_eq!(page.events, vec![event(2, 2, "server-enabled")]);
        assert_eq!(page.next_before_event_id, Some(2));
    }

    let (database, report) = DatabaseActor::start(descriptor).expect("restart app database");
    assert_eq!(report.previous_version, 9);
    assert_eq!(database.mcp_state_document().unwrap(), Some(state(2, 2)));
    let page = database
        .mcp_event_page(Some(2), SnapshotQuery::new(10).unwrap())
        .unwrap();
    assert_eq!(page.events, vec![event(1, 1, "definition-saved")]);
}

#[test]
fn v7_database_migrates_to_current_mcp_authority_without_touching_existing_state() {
    let temporary = TempDir::new().expect("temporary home");
    let descriptor = DatabaseDescriptor::app(temporary.path());
    {
        let (_database, report) =
            DatabaseActor::start(descriptor.clone()).expect("seed current database");
        assert_eq!(report.current_version, 9);
    }
    let connection = Connection::open(&descriptor.path).expect("open seed database");
    connection
        .execute_batch(
            "DROP TABLE mcp_events;
             DROP TABLE mcp_state;
             PRAGMA user_version = 7;",
        )
        .expect("restore exact pre-MCP schema");
    drop(connection);

    let (database, report) = DatabaseActor::start(descriptor).expect("migrate v7 to current");
    assert_eq!(report.previous_version, 7);
    assert_eq!(report.current_version, 9);
    assert!(report.backup_path.is_some_and(|path| path.exists()));
    assert_eq!(database.mcp_state_document().unwrap(), None);
}

#[test]
fn mcp_persistence_contains_references_not_secret_material_and_detects_tampering() {
    const SECRET: &str = "task12-raw-bearer-must-never-persist";
    let temporary = TempDir::new().expect("temporary home");
    let descriptor = DatabaseDescriptor::app(temporary.path());
    {
        let (database, _) = DatabaseActor::start(descriptor.clone()).expect("app database");
        database
            .save_mcp_transition(state(1, 1), event(1, 1, "definition-saved"), None)
            .expect("persist redacted MCP state");
    }
    let bytes = std::fs::read(&descriptor.path).expect("read database bytes");
    assert!(
        !bytes
            .windows(SECRET.len())
            .any(|window| window == SECRET.as_bytes())
    );
    assert!(
        bytes
            .windows("credential:opaque".len())
            .any(|window| { window == "credential:opaque".as_bytes() })
    );

    let connection = Connection::open(&descriptor.path).expect("open database for corruption");
    connection
        .execute(
            "UPDATE mcp_events SET canonical_document = '{\"corrupt\":true}' WHERE event_id = 1",
            [],
        )
        .expect("tamper MCP event without digest");
    drop(connection);
    let error = match DatabaseActor::start(descriptor) {
        Ok(_) => panic!("tampered MCP journal must fail closed"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("MCP event 1 digest mismatch"));
}
