use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMemoryRecord {
    pub id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub scope: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRecord {
    pub id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub surface: String,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub status: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub category: String,
    pub summary: String,
    pub resource: String,
    pub status: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLocalMemoryRequest {
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub scope: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RecordStore {
    local_memory: Vec<LocalMemoryRecord>,
    actions: Vec<ActionRecord>,
    audit: Vec<AuditRecord>,
}

static RECORD_STORE: OnceLock<Mutex<RecordStore>> = OnceLock::new();

pub fn list_local_memory_records() -> Vec<LocalMemoryRecord> {
    load_store().local_memory
}

pub fn save_local_memory_record(
    request: SaveLocalMemoryRequest,
) -> Result<LocalMemoryRecord, String> {
    let content = bounded_record_text(&request.content, "Local memory content")?;
    let mut store = load_store();
    let timestamp = now_unix_seconds();
    let workspace_id = normalized_optional(request.workspace_id)
        .or_else(active_workspace_id)
        .unwrap_or_else(|| "workspace:untrusted".into());
    let record = LocalMemoryRecord {
        id: format!("local-memory-{}", store.local_memory.len() + 1),
        workspace_id: workspace_id.clone(),
        session_id: normalized_optional(request.session_id),
        scope: normalized_optional(request.scope).unwrap_or_else(|| "workspace".into()),
        content,
        created_at: timestamp,
        updated_at: timestamp,
    };
    store.local_memory.push(record.clone());
    store.audit.push(AuditRecord {
        id: format!("audit-{}", store.audit.len() + 1),
        workspace_id,
        session_id: record.session_id.clone(),
        category: "local-memory".into(),
        summary: "Local memory record saved".into(),
        resource: record.id.clone(),
        status: "recorded".into(),
        created_at: timestamp,
    });
    save_store(&store);
    Ok(record)
}

pub fn list_action_records() -> Vec<ActionRecord> {
    load_store().actions
}

pub fn list_audit_records() -> Vec<AuditRecord> {
    load_store().audit
}

pub fn record_browser_action(
    workspace_id: &str,
    session_id: &str,
    actor: &str,
    action: &str,
    target: &str,
    status: &str,
) -> ActionRecord {
    record_action_and_audit(
        workspace_id,
        Some(session_id),
        "browser",
        actor,
        action,
        target,
        status,
        "Browser action recorded",
    )
}

pub fn record_terminal_action(
    workspace_id: &str,
    session_id: &str,
    actor: &str,
    action: &str,
    target: &str,
    status: &str,
) -> ActionRecord {
    record_action_and_audit(
        workspace_id,
        Some(session_id),
        "terminal",
        actor,
        action,
        target,
        status,
        "Terminal action recorded",
    )
}

pub fn record_file_action(
    workspace_id: &str,
    session_id: Option<&str>,
    action: &str,
    target: &str,
    status: &str,
) -> ActionRecord {
    record_action_and_audit(
        workspace_id,
        session_id,
        "files",
        "user",
        action,
        target,
        status,
        "File action recorded",
    )
}

pub fn record_security_audit(
    workspace_id: &str,
    session_id: Option<&str>,
    category: &str,
    summary: &str,
    resource: &str,
    status: &str,
) -> AuditRecord {
    let mut store = load_store();
    let timestamp = now_unix_seconds();
    let record = AuditRecord {
        id: format!("audit-{}", store.audit.len() + 1),
        workspace_id: normalized_id(workspace_id),
        session_id: session_id.and_then(|value| normalized_optional(Some(value.into()))),
        category: normalized_label(category, "security"),
        summary: bounded_target(summary),
        resource: bounded_target(resource),
        status: normalized_label(status, "recorded"),
        created_at: timestamp,
    };
    store.audit.push(record.clone());
    save_store(&store);
    record
}

fn record_action_and_audit(
    workspace_id: &str,
    session_id: Option<&str>,
    surface: &str,
    actor: &str,
    action: &str,
    target: &str,
    status: &str,
    summary: &str,
) -> ActionRecord {
    let mut store = load_store();
    let timestamp = now_unix_seconds();
    let record = ActionRecord {
        id: format!("action-{}", store.actions.len() + 1),
        workspace_id: normalized_id(workspace_id),
        session_id: session_id.and_then(|value| normalized_optional(Some(value.into()))),
        surface: normalized_label(surface, "unknown"),
        actor: normalized_label(actor, "system"),
        action: normalized_label(action, "record"),
        target: bounded_target(target),
        status: normalized_label(status, "recorded"),
        created_at: timestamp,
    };
    store.actions.push(record.clone());
    store.audit.push(AuditRecord {
        id: format!("audit-{}", store.audit.len() + 1),
        workspace_id: record.workspace_id.clone(),
        session_id: record.session_id.clone(),
        category: record.surface.clone(),
        summary: summary.into(),
        resource: record.target.clone(),
        status: record.status.clone(),
        created_at: timestamp,
    });
    save_store(&store);
    record
}

fn load_store() -> RecordStore {
    let mut store = record_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Ok(raw) = fs::read_to_string(record_store_path()) {
        if let Ok(saved) = serde_json::from_str::<RecordStore>(&raw) {
            *store = saved;
        }
    }
    store.clone()
}

fn save_store(next: &RecordStore) {
    {
        let mut store = record_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *store = next.clone();
    }
    if let Ok(serialized) = serde_json::to_string_pretty(next) {
        let path = record_store_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, format!("{serialized}\n"));
    }
}

fn record_store() -> &'static Mutex<RecordStore> {
    RECORD_STORE.get_or_init(|| Mutex::new(RecordStore::default()))
}

fn record_store_path() -> PathBuf {
    std::env::var("C4OS_RECORD_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("c4os-records.json"))
}

fn active_workspace_id() -> Option<String> {
    crate::workspace::active_workspace_descriptor().map(|descriptor| descriptor.id)
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn normalized_id(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "workspace:unknown".into()
    } else {
        value.into()
    }
}

fn normalized_label(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.into()
    } else {
        value.into()
    }
}

fn bounded_record_text(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} is required"));
    }
    Ok(value.chars().take(2048).collect())
}

fn bounded_target(value: &str) -> String {
    value.trim().chars().take(512).collect()
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn reset_records_store_for_test(name: &str) {
    let path = std::env::temp_dir().join(format!("c4os-{name}-records.json"));
    std::env::set_var("C4OS_RECORD_STORE", path);
    *record_store()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = RecordStore::default();
    let _ = fs::remove_file(record_store_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_014_local_memory_persists_in_app_owned_record_store() {
        reset_records_store_for_test("task-014-memory");

        let saved = save_local_memory_record(SaveLocalMemoryRequest {
            workspace_id: Some("workspace-alpha".into()),
            session_id: Some("session-one".into()),
            scope: Some("project".into()),
            content: "Use the checked-in QA workspace fixture.".into(),
        })
        .expect("save memory");

        *record_store()
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = RecordStore::default();
        let restored = list_local_memory_records();
        let audit = list_audit_records();

        assert_eq!(restored, vec![saved]);
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].category, "local-memory");
        assert_eq!(audit[0].resource, "local-memory-1");
    }

    #[test]
    fn task_014_action_records_are_separate_from_session_provider_and_workspace_stores() {
        reset_records_store_for_test("task-014-actions");
        crate::runtime_sessions::reset_session_store_for_test("task-014-sessions");
        let workspace_dir = std::env::temp_dir().join("c4os-task-014-workspace");
        let _ = fs::remove_dir_all(&workspace_dir);
        fs::create_dir_all(&workspace_dir).expect("create workspace");
        let activation =
            crate::workspace::activate_workspace_at(&workspace_dir).expect("activate workspace");

        let session = crate::runtime_sessions::create_session(
            crate::runtime_sessions::CreateSessionRequest {
                project: Some("Record Workspace".into()),
                label: Some("Record Session".into()),
                model: Some("model/records".into()),
            },
        );
        record_browser_action(
            &session.workspace_id,
            &session.id,
            "agent",
            "open",
            "https://example.com",
            "recorded",
        );
        record_terminal_action(
            &session.workspace_id,
            &session.id,
            "agent",
            "run",
            "pwd",
            "completed",
        );

        let descriptor_path = workspace_dir.join(".c4os").join("workspace.json");
        let descriptor = fs::read_to_string(descriptor_path).expect("descriptor");
        let session_store = fs::read_to_string(std::env::var("C4OS_SESSION_STORE").unwrap())
            .expect("session store");
        let record_store =
            fs::read_to_string(std::env::var("C4OS_RECORD_STORE").unwrap()).expect("record store");

        assert_eq!(activation.descriptor.id, session.workspace_id);
        assert!(!descriptor.contains("localMemory"));
        assert!(!descriptor.contains("audit"));
        assert!(!descriptor.contains("https://example.com"));
        assert!(!session_store.contains("https://example.com"));
        assert!(record_store.contains("https://example.com"));
        assert!(record_store.contains("\"surface\": \"browser\""));
        assert!(record_store.contains("\"surface\": \"terminal\""));
    }
}
