use crate::storage::{AppStore, NewMessage, NewToolCall};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RuntimeEvent {
    #[serde(rename = "session.status")]
    SessionStatus { session_id: String, status: String },
    #[serde(rename = "message.updated")]
    MessageUpdated {
        session_id: String,
        message_id: String,
        role: String,
        content: String,
        status: String,
    },
    #[serde(rename = "tool.updated")]
    ToolUpdated {
        session_id: String,
        tool_call_id: String,
        message_id: Option<String>,
        tool_name: String,
        arguments: serde_json::Value,
        status: String,
    },
}

#[derive(Debug)]
pub enum EventNormalizationError {
    InvalidEvent(serde_json::Error),
    Store(rusqlite::Error),
}

pub struct EventNormalizer<'a> {
    app_store: &'a AppStore,
}

impl<'a> EventNormalizer<'a> {
    pub fn new(app_store: &'a AppStore) -> Self {
        Self { app_store }
    }

    pub fn ingest_json(&self, payload: &str) -> Result<(), EventNormalizationError> {
        let event: RuntimeEvent =
            serde_json::from_str(payload).map_err(EventNormalizationError::InvalidEvent)?;

        self.ingest(event)
    }

    pub fn ingest(&self, event: RuntimeEvent) -> Result<(), EventNormalizationError> {
        match event {
            RuntimeEvent::SessionStatus { session_id, status } => self
                .app_store
                .update_session_status(&session_id, normalize_session_status(&status))
                .map_err(EventNormalizationError::Store),
            RuntimeEvent::MessageUpdated {
                session_id,
                message_id,
                role,
                content,
                status,
            } => self
                .app_store
                .append_message(NewMessage {
                    id: &format!("runtime-message-{message_id}"),
                    session_id: &session_id,
                    role: normalize_role(&role),
                    content: &content,
                    status: normalize_message_status(&status),
                    metadata_json: &format!(r#"{{"runtimeMessageRef":"{message_id}"}}"#),
                })
                .map_err(EventNormalizationError::Store),
            RuntimeEvent::ToolUpdated {
                session_id,
                tool_call_id,
                message_id,
                tool_name,
                arguments,
                status,
            } => self
                .app_store
                .record_tool_call(NewToolCall {
                    id: &format!("runtime-tool-{tool_call_id}"),
                    session_id: &session_id,
                    message_id: message_id.as_deref(),
                    tool_source: "opencode",
                    tool_name: &tool_name,
                    arguments_json: &arguments.to_string(),
                    status: normalize_tool_status(&status),
                    runtime_call_ref: Some(&tool_call_id),
                })
                .map_err(EventNormalizationError::Store),
        }
    }
}

fn normalize_session_status(status: &str) -> &str {
    match status {
        "idle" => "complete",
        "error" => "failed",
        "aborted" => "stopped",
        "running" => "running",
        _ => "running",
    }
}

fn normalize_message_status(status: &str) -> &str {
    match status {
        "done" => "complete",
        "error" => "failed",
        "aborted" => "stopped",
        "streaming" => "streaming",
        _ => "submitted",
    }
}

fn normalize_role(role: &str) -> &str {
    match role {
        "assistant" => "assistant",
        "system" => "system",
        "tool" => "tool",
        _ => "user",
    }
}

fn normalize_tool_status(status: &str) -> &str {
    match status {
        "completed" => "complete",
        "failed" => "failed",
        "denied" => "denied",
        _ => "running",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::storage::NewProject;

    #[test]
    fn maps_message_event_to_app_owned_message_with_runtime_metadata() {
        let store = prepared_running_store();
        let normalizer = EventNormalizer::new(&store);

        normalizer
            .ingest_json(
                r#"{
                    "type":"message.updated",
                    "session_id":"session-1",
                    "message_id":"runtime-1",
                    "role":"assistant",
                    "content":"hello",
                    "status":"done"
                }"#,
            )
            .expect("event ingested");
        let messages = store.list_messages("session-1").expect("messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "runtime-message-runtime-1");
        assert_eq!(messages[0].status, "complete");
        assert!(messages[0].metadata_json.contains("runtime-1"));
    }

    #[test]
    fn maps_tool_event_to_app_owned_tool_call() {
        let store = prepared_running_store();
        let normalizer = EventNormalizer::new(&store);

        normalizer
            .ingest_json(
                r#"{
                    "type":"tool.updated",
                    "session_id":"session-1",
                    "tool_call_id":"tool-1",
                    "message_id":null,
                    "tool_name":"file.read",
                    "arguments":{"path":"README.md"},
                    "status":"completed"
                }"#,
            )
            .expect("event ingested");

        assert_eq!(store.tool_call_count("session-1").expect("tool count"), 1);
    }

    #[test]
    fn maps_runtime_idle_status_to_app_owned_complete_status() {
        let store = prepared_running_store();
        let normalizer = EventNormalizer::new(&store);

        normalizer
            .ingest_json(
                r#"{
                    "type":"session.status",
                    "session_id":"session-1",
                    "status":"idle"
                }"#,
            )
            .expect("event ingested");
        let session = store
            .get_session("session-1")
            .expect("session query")
            .expect("session exists");

        assert_eq!(session.status, "complete");
    }

    fn prepared_running_store() -> AppStore {
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: "/tmp/example",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");
        store
            .start_openrouter_session("session-1", "project-1", "Run")
            .expect("session started");

        store
    }
}
