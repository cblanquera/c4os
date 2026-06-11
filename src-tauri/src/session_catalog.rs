use crate::session_view::{RuntimeState, SessionScreen, SessionStartGuard, SessionView};
use crate::storage::{AppStore, SessionRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectSessionCatalog {
    pub project_id: String,
    pub sessions: Vec<SessionCatalogItem>,
    pub latest_session_id: Option<String>,
    pub start_guard: SessionStartGuard,
    pub search_available: bool,
    pub archive_available: bool,
    pub delete_available: bool,
    pub concurrent_active_sessions: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionCatalogItem {
    pub id: String,
    pub title: String,
    pub runtime_state: RuntimeState,
    pub model_id: String,
    pub is_latest: bool,
    pub is_active: bool,
}

pub struct SessionCatalog;

#[derive(Debug, Eq, PartialEq)]
pub enum SessionSelectionError {
    MissingSession,
    SessionOutsideProject,
    StoreFailed,
}

impl SessionCatalog {
    pub fn for_project(
        app_store: &AppStore,
        project_id: &str,
    ) -> rusqlite::Result<ProjectSessionCatalog> {
        let sessions = app_store.list_sessions_for_project(project_id)?;
        let latest_session_id = sessions.first().map(|session| session.id.clone());
        let items = sessions
            .into_iter()
            .map(|session| project_item(session, latest_session_id.as_deref()))
            .collect();

        Ok(ProjectSessionCatalog {
            project_id: project_id.into(),
            sessions: items,
            latest_session_id,
            start_guard: SessionView::start_guard(app_store)?,
            search_available: false,
            archive_available: false,
            delete_available: false,
            concurrent_active_sessions: false,
        })
    }

    pub fn open_for_project(
        app_store: &AppStore,
        project_id: &str,
        selected_session_id: Option<&str>,
    ) -> Result<Option<SessionScreen>, SessionSelectionError> {
        let Some(session_id) = selected_session_id else {
            return SessionView::latest_for_project(app_store, project_id)
                .map_err(|_| SessionSelectionError::StoreFailed);
        };
        let session = app_store
            .get_session(session_id)
            .map_err(|_| SessionSelectionError::StoreFailed)?
            .ok_or(SessionSelectionError::MissingSession)?;

        if session.project_id != project_id {
            return Err(SessionSelectionError::SessionOutsideProject);
        }

        SessionView::for_session(app_store, session_id)
            .map_err(|_| SessionSelectionError::StoreFailed)?
            .ok_or(SessionSelectionError::MissingSession)
            .map(Some)
    }
}

fn project_item(session: SessionRecord, latest_session_id: Option<&str>) -> SessionCatalogItem {
    let runtime_state = runtime_state_from_status(&session.status);
    let is_active = matches!(
        runtime_state,
        RuntimeState::Running | RuntimeState::WaitingForApproval
    );
    let is_latest = latest_session_id == Some(session.id.as_str());

    SessionCatalogItem {
        id: session.id,
        title: session.title,
        runtime_state,
        model_id: session.model_id,
        is_latest,
        is_active,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject, NewSession};

    #[test]
    fn catalog_lists_multiple_project_sessions_newest_first() {
        let store = prepared_store();
        insert_session(&store, "session-1", "First", "complete");
        insert_session(&store, "session-2", "Second", "failed");
        insert_session(&store, "session-other", "Other", "complete");

        let catalog = SessionCatalog::for_project(&store, "project-1").expect("catalog");

        assert_eq!(
            catalog
                .sessions
                .iter()
                .map(|session| session.id.as_str())
                .collect::<Vec<_>>(),
            vec!["session-2", "session-1"]
        );
        assert_eq!(catalog.latest_session_id, Some("session-2".into()));
        assert_eq!(
            catalog
                .sessions
                .iter()
                .filter(|session| session.is_latest)
                .count(),
            1
        );
        assert_eq!(catalog.sessions[0].runtime_state, RuntimeState::Failed);
    }

    #[test]
    fn empty_project_catalog_has_no_latest_session() {
        let store = prepared_store();

        let catalog = SessionCatalog::for_project(&store, "project-1").expect("catalog");

        assert!(catalog.sessions.is_empty());
        assert_eq!(catalog.latest_session_id, None);
        assert!(catalog.start_guard.can_start);
    }

    #[test]
    fn active_session_blocks_new_run_even_with_multiple_sessions() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Done", "complete");
        insert_session(&store, "session-2", "Running", "running");

        let catalog = SessionCatalog::for_project(&store, "project-1").expect("catalog");

        assert!(!catalog.start_guard.can_start);
        assert_eq!(
            catalog.start_guard.blocking_session_id,
            Some("session-2".into())
        );
        assert!(catalog.sessions[0].is_active);
        assert!(!catalog.concurrent_active_sessions);
    }

    #[test]
    fn catalog_does_not_expose_postponed_session_management_controls() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Done", "complete");

        let catalog = SessionCatalog::for_project(&store, "project-1").expect("catalog");

        assert!(!catalog.search_available);
        assert!(!catalog.archive_available);
        assert!(!catalog.delete_available);
    }

    #[test]
    fn selected_session_opens_only_when_it_belongs_to_project() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Done", "complete");

        let screen = SessionCatalog::open_for_project(&store, "project-1", Some("session-1"))
            .expect("selection")
            .expect("screen");

        assert_eq!(screen.session_id, "session-1");
        assert_eq!(screen.title, "Done");
    }

    #[test]
    fn omitted_selection_opens_latest_project_session() {
        let store = prepared_store();
        insert_session(&store, "session-1", "First", "complete");
        insert_session(&store, "session-2", "Second", "complete");

        let screen = SessionCatalog::open_for_project(&store, "project-1", None)
            .expect("selection")
            .expect("screen");

        assert_eq!(screen.session_id, "session-2");
    }

    #[test]
    fn missing_selected_session_fails_closed() {
        let store = prepared_store();

        let result = SessionCatalog::open_for_project(&store, "project-1", Some("missing"));

        assert_eq!(result, Err(SessionSelectionError::MissingSession));
    }

    #[test]
    fn cross_project_session_selection_is_rejected() {
        let store = prepared_store();
        insert_session(&store, "session-other", "Other", "complete");

        let result = SessionCatalog::open_for_project(&store, "project-1", Some("session-other"));

        assert_eq!(result, Err(SessionSelectionError::SessionOutsideProject));
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
            .create_project(NewProject {
                id: "project-2",
                name: "Other",
                root_path: "/tmp/other",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("other project inserted");

        store
    }

    fn insert_session(store: &AppStore, id: &str, title: &str, status: &str) {
        let project_id = if id == "session-other" {
            "project-2"
        } else {
            "project-1"
        };

        store
            .create_session(NewSession {
                id,
                project_id,
                title,
                status,
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");
    }
}
