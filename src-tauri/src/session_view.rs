use crate::storage::{AppStore, MessageRecord, SessionRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct SessionScreen {
    pub session_id: String,
    pub title: String,
    pub runtime_state: RuntimeState,
    pub runtime_label: String,
    pub failure_display: Option<String>,
    pub messages: Vec<TranscriptMessage>,
    pub start_guard: SessionStartGuard,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TranscriptMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub status: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionStartGuard {
    pub can_start: bool,
    pub blocking_session_id: Option<String>,
    pub blocking_state: Option<RuntimeState>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeState {
    Running,
    WaitingForApproval,
    Stopped,
    Failed,
    Complete,
}

pub struct SessionView;

impl SessionView {
    pub fn for_session(
        app_store: &AppStore,
        session_id: &str,
    ) -> rusqlite::Result<Option<SessionScreen>> {
        app_store
            .get_session(session_id)?
            .map(|session| project_session(app_store, session))
            .transpose()
    }

    pub fn latest_for_project(
        app_store: &AppStore,
        project_id: &str,
    ) -> rusqlite::Result<Option<SessionScreen>> {
        app_store
            .latest_session_for_project(project_id)?
            .map(|session| project_session(app_store, session))
            .transpose()
    }

    pub fn start_guard(app_store: &AppStore) -> rusqlite::Result<SessionStartGuard> {
        start_guard(app_store)
    }
}

fn project_session(
    app_store: &AppStore,
    session: SessionRecord,
) -> rusqlite::Result<SessionScreen> {
    let runtime_state = runtime_state_from_status(&session.status);
    let messages = app_store
        .list_messages(&session.id)?
        .into_iter()
        .map(project_message)
        .collect();

    Ok(SessionScreen {
        session_id: session.id,
        title: session.title,
        runtime_label: runtime_state.label().into(),
        failure_display: failure_display(&runtime_state),
        runtime_state,
        messages,
        start_guard: start_guard(app_store)?,
    })
}

fn project_message(message: MessageRecord) -> TranscriptMessage {
    TranscriptMessage {
        id: message.id,
        role: message.role,
        content: message.content,
        status: message.status,
    }
}

fn start_guard(app_store: &AppStore) -> rusqlite::Result<SessionStartGuard> {
    let active_session = app_store.active_session()?;

    Ok(match active_session {
        Some(session) => {
            let blocking_state = runtime_state_from_status(&session.status);
            SessionStartGuard {
                can_start: false,
                blocking_session_id: Some(session.id),
                blocking_state: Some(blocking_state.clone()),
                reason: Some(format!(
                    "Finish or stop the {} session before starting another run.",
                    blocking_state.label()
                )),
            }
        }
        None => SessionStartGuard {
            can_start: true,
            blocking_session_id: None,
            blocking_state: None,
            reason: None,
        },
    })
}

fn runtime_state_from_status(status: &str) -> RuntimeState {
    match status {
        "running" => RuntimeState::Running,
        "waiting_for_approval" => RuntimeState::WaitingForApproval,
        "failed" => RuntimeState::Failed,
        "complete" => RuntimeState::Complete,
        "stopped" | "interrupted" | "created" => RuntimeState::Stopped,
        _ => RuntimeState::Stopped,
    }
}

fn failure_display(runtime_state: &RuntimeState) -> Option<String> {
    if runtime_state == &RuntimeState::Failed {
        Some("Session failed. Review the transcript and tool timeline before retrying.".into())
    } else {
        None
    }
}

impl RuntimeState {
    fn label(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::WaitingForApproval => "Waiting for approval",
            Self::Stopped => "Stopped",
            Self::Failed => "Failed",
            Self::Complete => "Complete",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewMessage, NewProject, NewSession};

    #[test]
    fn transcript_projects_appended_messages_in_order() {
        let store = prepared_store("running");
        append_message(&store, "message-2", "assistant", "Second", "complete");
        append_message(&store, "message-1", "user", "First", "submitted");

        let screen = SessionView::for_session(&store, "session-1")
            .expect("screen query")
            .expect("session screen");

        assert_eq!(screen.runtime_state, RuntimeState::Running);
        assert_eq!(
            screen
                .messages
                .iter()
                .map(|message| message.content.as_str())
                .collect::<Vec<_>>(),
            vec!["Second", "First"]
        );
    }

    #[test]
    fn failure_state_includes_actionable_display() {
        let store = prepared_store("failed");
        append_message(
            &store,
            "message-1",
            "assistant",
            "failed response",
            "failed",
        );

        let screen = SessionView::for_session(&store, "session-1")
            .expect("screen query")
            .expect("session screen");

        assert_eq!(screen.runtime_state, RuntimeState::Failed);
        assert_eq!(screen.runtime_label, "Failed");
        assert!(screen
            .failure_display
            .expect("failure display")
            .contains("Session failed"));
    }

    #[test]
    fn stopped_and_complete_states_are_projected_clearly() {
        let stopped_store = prepared_store("stopped");
        let complete_store = prepared_store("complete");

        let stopped = SessionView::for_session(&stopped_store, "session-1")
            .expect("stopped query")
            .expect("stopped screen");
        let complete = SessionView::for_session(&complete_store, "session-1")
            .expect("complete query")
            .expect("complete screen");

        assert_eq!(stopped.runtime_state, RuntimeState::Stopped);
        assert_eq!(stopped.runtime_label, "Stopped");
        assert_eq!(complete.runtime_state, RuntimeState::Complete);
        assert_eq!(complete.runtime_label, "Complete");
    }

    #[test]
    fn waiting_for_approval_blocks_starting_another_session() {
        let store = prepared_store("waiting_for_approval");

        let guard = SessionView::start_guard(&store).expect("guard query");

        assert!(!guard.can_start);
        assert_eq!(guard.blocking_session_id, Some("session-1".into()));
        assert_eq!(guard.blocking_state, Some(RuntimeState::WaitingForApproval));
        assert!(guard
            .reason
            .expect("reason")
            .contains("Waiting for approval"));
    }

    #[test]
    fn no_active_session_allows_new_run() {
        let store = prepared_store("complete");

        let guard = SessionView::start_guard(&store).expect("guard query");

        assert!(guard.can_start);
        assert_eq!(guard.blocking_session_id, None);
    }

    #[test]
    fn latest_project_session_uses_updated_session_order() {
        let store = prepared_store("complete");
        store
            .create_session(NewSession {
                id: "session-2",
                project_id: "project-1",
                title: "Latest run",
                status: "running",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("second session inserted");

        let screen = SessionView::latest_for_project(&store, "project-1")
            .expect("screen query")
            .expect("session screen");

        assert_eq!(screen.session_id, "session-2");
        assert_eq!(screen.runtime_state, RuntimeState::Running);
    }

    fn prepared_store(status: &str) -> AppStore {
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
                status,
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");

        store
    }

    fn append_message(store: &AppStore, id: &str, role: &str, content: &str, status: &str) {
        store
            .append_message(NewMessage {
                id,
                session_id: "session-1",
                role,
                content,
                status,
                metadata_json: "{}",
            })
            .expect("message inserted");
    }
}
