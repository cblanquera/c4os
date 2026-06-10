use crate::storage::{AppStore, ToolTimelineRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct ToolTimelineItem {
    pub id: String,
    pub label: String,
    pub status: String,
    pub safe_summary: Option<String>,
    pub safe_labels: Vec<String>,
    pub denial: Option<String>,
    pub live_drawer_label: Option<String>,
    pub has_raw_output_export: bool,
}

pub struct ToolTimeline;

impl ToolTimeline {
    pub fn for_session(
        app_store: &AppStore,
        session_id: &str,
    ) -> rusqlite::Result<Vec<ToolTimelineItem>> {
        app_store
            .list_tool_timeline(session_id)?
            .into_iter()
            .map(project_item)
            .collect::<Vec<_>>()
            .pipe(Ok)
    }
}

fn project_item(record: ToolTimelineRecord) -> ToolTimelineItem {
    ToolTimelineItem {
        id: record.id,
        label: record.tool_name,
        status: record.status.clone(),
        safe_summary: record.output_summary,
        safe_labels: parse_labels(&record.output_summary_reason_labels),
        denial: record.denial_category.or(record.denial_message),
        live_drawer_label: if record.status == "running" {
            Some("live/ephemeral".into())
        } else {
            None
        },
        has_raw_output_export: false,
    }
}

fn parse_labels(labels_json: &str) -> Vec<String> {
    serde_json::from_str(labels_json).unwrap_or_default()
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject, NewSession, NewToolCall};

    #[test]
    fn timeline_projects_completed_tool_with_safe_summary() {
        let store = prepared_store();
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: None,
                tool_source: "opencode",
                tool_name: "bash",
                arguments_json: "{}",
                status: "complete",
                runtime_call_ref: Some("runtime-tool-1"),
            })
            .expect("tool inserted");
        store
            .update_tool_summary(
                "tool-1",
                "stdout:\nok",
                r#"["truncated_by_size"]"#,
                true,
                false,
            )
            .expect("summary updated");

        let timeline = ToolTimeline::for_session(&store, "session-1").expect("timeline");

        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].safe_summary, Some("stdout:\nok".into()));
        assert_eq!(timeline[0].safe_labels, vec!["truncated_by_size"]);
        assert!(!timeline[0].has_raw_output_export);
    }

    #[test]
    fn running_tool_drawer_is_labeled_live_ephemeral() {
        let store = prepared_store();
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: None,
                tool_source: "opencode",
                tool_name: "bash",
                arguments_json: "{}",
                status: "running",
                runtime_call_ref: Some("runtime-tool-1"),
            })
            .expect("tool inserted");

        let timeline = ToolTimeline::for_session(&store, "session-1").expect("timeline");

        assert_eq!(timeline[0].live_drawer_label, Some("live/ephemeral".into()));
        assert!(!timeline[0].has_raw_output_export);
    }

    fn prepared_store() -> AppStore {
        let store = AppStore::open_in_memory().expect("store opens");

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
            .create_session(NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Run",
                status: "running",
                mode: "agent",
                agent_ref: None,
                model_id: "openai/gpt-4.1",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");

        store
    }
}
