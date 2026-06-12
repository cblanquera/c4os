use crate::storage::AppStore;
use serde_json::{json, Value};
use std::path::Path;

pub struct ProjectExportService<'a> {
    store: &'a AppStore,
}

impl<'a> ProjectExportService<'a> {
    pub fn new(store: &'a AppStore) -> Self {
        Self { store }
    }

    pub fn export_project_json(&self, project_id: &str) -> rusqlite::Result<Value> {
        let project = self
            .store
            .get_project(project_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let sessions = self
            .store
            .list_sessions_for_project(project_id)?
            .into_iter()
            .map(|session| {
                let messages = self
                    .store
                    .list_messages(&session.id)?
                    .into_iter()
                    .map(|message| {
                        Ok(json!({
                            "id": message.id,
                            "role": message.role,
                            "content": redact_export_text(&message.content),
                            "status": message.status,
                            "metadata": redact_json_text(&message.metadata_json),
                        }))
                    })
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                let tool_calls = self
                    .store
                    .list_tool_timeline(&session.id)?
                    .into_iter()
                    .map(|tool| {
                        json!({
                            "id": tool.id,
                            "toolName": tool.tool_name,
                            "status": tool.status,
                            "outputSummary": tool.output_summary.map(|summary| redact_export_text(&summary)),
                            "outputTruncated": tool.output_truncated,
                            "redactionApplied": tool.redaction_applied,
                            "outputSummaryReasonLabels": tool.output_summary_reason_labels,
                            "denialCategory": tool.denial_category,
                            "denialMessage": tool.denial_message.map(|message| redact_export_text(&message)),
                        })
                    })
                    .collect::<Vec<_>>();

                Ok(json!({
                    "id": session.id,
                    "title": redact_export_text(&session.title),
                    "workflowPurpose": session.workflow_purpose,
                    "status": session.status,
                    "mode": session.mode,
                    "modelId": session.model_id,
                    "runtime": session.runtime,
                    "agentRef": Value::Null,
                    "runtimeSessionRef": Value::Null,
                    "messages": messages,
                    "toolCalls": tool_calls,
                }))
            })
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let artifacts = self
            .store
            .list_artifacts_for_project(project_id)?
            .into_iter()
            .map(|artifact| {
                json!({
                    "id": artifact.id,
                    "sessionId": artifact.session_id,
                    "toolCallId": artifact.tool_call_id,
                    "type": artifact.artifact_type,
                    "title": redact_export_text(&artifact.title),
                    "mimeType": artifact.mime_type,
                    "fileName": artifact.file_path.as_deref().and_then(file_name),
                    "previewFileName": artifact.preview_path.as_deref().and_then(file_name),
                    "filePath": Value::Null,
                    "previewPath": Value::Null,
                    "metadata": redact_json_text(&artifact.metadata_json),
                    "payloadIncluded": false,
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "format": "c4os.project_export.v1",
            "supportTier": "project_json_export_only",
            "importAvailable": false,
            "roundTripCompatibility": false,
            "project": {
                "id": project.id,
                "name": redact_export_text(&project.name),
                "workflowPurpose": project.workflow_purpose,
                "rootPath": Value::Null,
                "rootPathPolicy": "excluded_absolute_local_path",
                "defaultModel": project.default_model,
                "defaultAgentRef": Value::Null,
            },
            "sessions": sessions,
            "artifacts": artifacts,
            "security": {
                "rawSecretsIncluded": false,
                "rawShellOutputIncluded": false,
                "providerKeysIncluded": false,
                "absoluteLocalPathsIncluded": false,
                "artifactPayloadsIncluded": false,
            },
        }))
    }
}

fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

fn redact_json_text(value: &str) -> Value {
    match serde_json::from_str::<Value>(value) {
        Ok(json) => redact_json_value(json),
        Err(_) => Value::String(redact_export_text(value)),
    }
}

fn redact_json_value(value: Value) -> Value {
    match value {
        Value::String(value) => Value::String(redact_export_text(&value)),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_json_value).collect()),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| {
                    if contains_secret_material(&key) {
                        (key, Value::String("[REDACTED_SECRET_VALUE]".into()))
                    } else {
                        (key, redact_json_value(value))
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

fn redact_export_text(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if contains_secret_material(line) {
                "[REDACTED_SECRET_LINE]".into()
            } else {
                line.into()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn contains_secret_material(value: &str) -> bool {
    let lower = value.to_lowercase();

    lower.contains("sk-or-")
        || lower.contains("sk-")
        || lower.contains("token=")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("bearer ")
        || lower.contains("-----begin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewArtifact, NewMessage, NewProject, NewSession, NewToolCall};

    #[test]
    fn exports_selected_project_json_without_import_or_round_trip_claims() {
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ProjectExportService::new(&store);

        let export = service
            .export_project_json("project-1")
            .expect("project exported");

        assert_eq!(export["format"], "c4os.project_export.v1");
        assert_eq!(export["supportTier"], "project_json_export_only");
        assert_eq!(export["importAvailable"], false);
        assert_eq!(export["roundTripCompatibility"], false);
        assert_eq!(export["project"]["id"], "project-1");
        assert_eq!(export["project"]["name"], "Example");
        assert_eq!(export["project"]["workflowPurpose"], "research");
        assert_eq!(export["project"]["rootPath"], Value::Null);
        assert_eq!(export["sessions"][0]["id"], "session-1");
        assert_eq!(export["sessions"][0]["workflowPurpose"], "writing");
        assert_eq!(export["sessions"][0]["messages"][0]["content"], "hello");
        assert_eq!(export["sessions"][0]["toolCalls"][0]["outputSummary"], "ok");
        assert_eq!(export["artifacts"][0]["title"], "notes.txt");
        assert_eq!(export["artifacts"][0]["filePath"], Value::Null);
        assert_eq!(export["artifacts"][0]["previewPath"], Value::Null);
        assert_eq!(export["artifacts"][0]["payloadIncluded"], false);
    }

    #[test]
    fn export_redacts_secret_shaped_message_and_tool_summary_values() {
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        store
            .append_message(NewMessage {
                id: "message-secret",
                session_id: "session-1",
                role: "user",
                content: "OPENROUTER_API_KEY=sk-secret\nkeep",
                status: "submitted",
                metadata_json: "{}",
            })
            .expect("message inserted");
        store
            .update_tool_summary(
                "tool-1",
                "token=secret\nsummary",
                r#"["redacted_secret_pattern"]"#,
                false,
                true,
            )
            .expect("tool summary updated");
        let service = ProjectExportService::new(&store);

        let export = service
            .export_project_json("project-1")
            .expect("project exported");

        assert_eq!(
            export["sessions"][0]["messages"][1]["content"],
            "[REDACTED_SECRET_LINE]\nkeep"
        );
        assert_eq!(
            export["sessions"][0]["toolCalls"][0]["outputSummary"],
            "[REDACTED_SECRET_LINE]\nsummary"
        );
        assert_eq!(export["security"]["rawSecretsIncluded"], false);
        assert_eq!(export["security"]["rawShellOutputIncluded"], false);
        assert_eq!(export["security"]["providerKeysIncluded"], false);
    }

    fn prepared_records(store: &AppStore) {
        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: "/Users/example/project",
                default_model: Some("openrouter/model"),
                default_agent_ref: Some("agent-secret-ref"),
            })
            .expect("project inserted");
        store
            .set_project_workflow_purpose("project-1", Some("research"))
            .expect("project workflow purpose set");
        store
            .create_session(NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Run",
                status: "complete",
                mode: "agent",
                agent_ref: Some("agent-secret-ref"),
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: Some("runtime-secret-ref"),
            })
            .expect("session inserted");
        store
            .set_session_workflow_purpose("session-1", Some("writing"))
            .expect("session workflow purpose set");
        store
            .append_message(NewMessage {
                id: "message-1",
                session_id: "session-1",
                role: "user",
                content: "hello",
                status: "submitted",
                metadata_json: "{}",
            })
            .expect("message inserted");
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: Some("message-1"),
                tool_source: "opencode",
                tool_name: "shell",
                arguments_json: r#"{"command":"npm test"}"#,
                status: "complete",
                runtime_call_ref: Some("runtime-call-ref"),
            })
            .expect("tool inserted");
        store
            .update_tool_summary("tool-1", "ok", "[]", false, false)
            .expect("tool summary updated");
        store
            .record_artifact(NewArtifact {
                id: "artifact-1",
                project_id: "project-1",
                session_id: Some("session-1"),
                tool_call_id: Some("tool-1"),
                artifact_type: "text",
                title: "notes.txt",
                mime_type: Some("text/plain"),
                file_path: Some("/Users/example/project/.c4os/artifact/original"),
                preview_path: Some("/Users/example/project/.c4os/artifact/preview"),
                metadata_json: r#"{"textLike":true}"#,
            })
            .expect("artifact inserted");
    }
}
