use crate::session_view::{RuntimeState, SessionScreen, SessionStartGuard, SessionView};
use crate::storage::{AppStore, SessionRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectSessionCatalog {
    pub project_id: String,
    pub sessions: Vec<SessionCatalogItem>,
    pub workflow_purpose_groups: Vec<SessionWorkflowPurposeGroup>,
    pub latest_session_id: Option<String>,
    pub search_query: Option<String>,
    pub workflow_purpose_filter: Option<String>,
    pub start_guard: SessionStartGuard,
    pub search_available: bool,
    pub workflow_purpose_filter_available: bool,
    pub workflow_purpose_grouping_available: bool,
    pub archive_available: bool,
    pub delete_available: bool,
    pub concurrent_active_sessions: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionCatalogItem {
    pub id: String,
    pub title: String,
    pub workflow_purpose: Option<String>,
    pub runtime_state: RuntimeState,
    pub model_id: String,
    pub is_latest: bool,
    pub is_active: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SessionWorkflowPurposeGroup {
    pub workflow_purpose: Option<String>,
    pub session_count: usize,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct SessionCatalogQuery<'a> {
    pub search: Option<&'a str>,
    pub workflow_purpose: Option<&'a str>,
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
        Self::for_project_filtered(app_store, project_id, SessionCatalogQuery::default())
    }

    pub fn for_project_filtered(
        app_store: &AppStore,
        project_id: &str,
        query: SessionCatalogQuery<'_>,
    ) -> rusqlite::Result<ProjectSessionCatalog> {
        validate_workflow_purpose_filter(query.workflow_purpose)?;
        let search_query = normalized_search(query.search);
        let sessions = app_store.list_sessions_for_project(project_id)?;
        let visible_sessions = sessions
            .into_iter()
            .filter(|session| matches_search(session, search_query.as_deref()))
            .filter(|session| matches_workflow_purpose(session, query.workflow_purpose))
            .collect::<Vec<_>>();
        let latest_session_id = visible_sessions.first().map(|session| session.id.clone());
        let workflow_purpose_groups = workflow_groups(&visible_sessions);
        let items = visible_sessions
            .into_iter()
            .map(|session| project_item(session, latest_session_id.as_deref()))
            .collect();

        Ok(ProjectSessionCatalog {
            project_id: project_id.into(),
            sessions: items,
            workflow_purpose_groups,
            latest_session_id,
            search_query,
            workflow_purpose_filter: query.workflow_purpose.map(str::to_string),
            start_guard: SessionView::start_guard(app_store)?,
            search_available: true,
            workflow_purpose_filter_available: true,
            workflow_purpose_grouping_available: true,
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

fn normalized_search(search: Option<&str>) -> Option<String> {
    search
        .map(str::trim)
        .filter(|search| !search.is_empty())
        .map(|search| search.to_lowercase())
}

fn matches_search(session: &SessionRecord, search_query: Option<&str>) -> bool {
    let Some(search_query) = search_query else {
        return true;
    };
    let title = session.title.to_lowercase();
    let model_id = session.model_id.to_lowercase();
    let status = session.status.to_lowercase();

    title.contains(search_query) || model_id.contains(search_query) || status.contains(search_query)
}

fn matches_workflow_purpose(session: &SessionRecord, workflow_purpose: Option<&str>) -> bool {
    workflow_purpose.is_none() || session.workflow_purpose.as_deref() == workflow_purpose
}

fn validate_workflow_purpose_filter(workflow_purpose: Option<&str>) -> rusqlite::Result<()> {
    match workflow_purpose {
        None | Some("research") | Some("writing") | Some("documentation") | Some("analysis") => {
            Ok(())
        }
        Some(value) => Err(rusqlite::Error::InvalidParameterName(format!(
            "unsupported workflow purpose filter: {value}"
        ))),
    }
}

fn workflow_groups(sessions: &[SessionRecord]) -> Vec<SessionWorkflowPurposeGroup> {
    let mut groups = Vec::<SessionWorkflowPurposeGroup>::new();

    for session in sessions {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.workflow_purpose == session.workflow_purpose)
        {
            group.session_count += 1;
        } else {
            groups.push(SessionWorkflowPurposeGroup {
                workflow_purpose: session.workflow_purpose.clone(),
                session_count: 1,
            });
        }
    }

    groups
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
        workflow_purpose: session.workflow_purpose,
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
        assert_eq!(catalog.search_query, None);
        assert_eq!(catalog.workflow_purpose_filter, None);
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

        assert!(catalog.search_available);
        assert!(catalog.workflow_purpose_filter_available);
        assert!(catalog.workflow_purpose_grouping_available);
        assert!(!catalog.archive_available);
        assert!(!catalog.delete_available);
    }

    #[test]
    fn catalog_filters_sessions_by_search_within_selected_project_only() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Research notes", "complete");
        insert_session(&store, "session-2", "Writing draft", "complete");
        insert_session(
            &store,
            "session-other",
            "Research other project",
            "complete",
        );

        let catalog = SessionCatalog::for_project_filtered(
            &store,
            "project-1",
            SessionCatalogQuery {
                search: Some("research"),
                workflow_purpose: None,
            },
        )
        .expect("filtered catalog");

        assert_eq!(catalog.search_query, Some("research".into()));
        assert_eq!(
            catalog
                .sessions
                .iter()
                .map(|session| session.id.as_str())
                .collect::<Vec<_>>(),
            vec!["session-1"]
        );
        assert!(!catalog
            .sessions
            .iter()
            .any(|session| session.id == "session-other"));
    }

    #[test]
    fn catalog_filters_and_groups_sessions_by_bounded_workflow_purpose() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Research notes", "complete");
        insert_session(&store, "session-2", "Writing draft", "complete");
        insert_session(&store, "session-3", "More research", "failed");
        store
            .set_session_workflow_purpose("session-1", Some("research"))
            .expect("session one workflow purpose set");
        store
            .set_session_workflow_purpose("session-2", Some("writing"))
            .expect("session two workflow purpose set");
        store
            .set_session_workflow_purpose("session-3", Some("research"))
            .expect("session three workflow purpose set");

        let catalog = SessionCatalog::for_project_filtered(
            &store,
            "project-1",
            SessionCatalogQuery {
                search: None,
                workflow_purpose: Some("research"),
            },
        )
        .expect("filtered catalog");

        assert_eq!(catalog.workflow_purpose_filter, Some("research".into()));
        assert_eq!(
            catalog
                .sessions
                .iter()
                .map(|session| session.id.as_str())
                .collect::<Vec<_>>(),
            vec!["session-3", "session-1"]
        );
        assert_eq!(
            catalog.workflow_purpose_groups,
            vec![SessionWorkflowPurposeGroup {
                workflow_purpose: Some("research".into()),
                session_count: 2,
            }]
        );
        assert!(catalog
            .sessions
            .iter()
            .all(|session| session.workflow_purpose == Some("research".into())));

        let invalid = SessionCatalog::for_project_filtered(
            &store,
            "project-1",
            SessionCatalogQuery {
                search: None,
                workflow_purpose: Some("coding"),
            },
        );

        assert!(invalid.is_err());
    }

    #[test]
    fn catalog_filtering_does_not_enable_concurrent_active_sessions() {
        let store = prepared_store();
        insert_session(&store, "session-1", "Done", "complete");
        insert_session(&store, "session-2", "Running", "running");
        store
            .set_session_workflow_purpose("session-1", Some("research"))
            .expect("session one workflow purpose set");
        store
            .set_session_workflow_purpose("session-2", Some("writing"))
            .expect("session two workflow purpose set");

        let catalog = SessionCatalog::for_project_filtered(
            &store,
            "project-1",
            SessionCatalogQuery {
                search: None,
                workflow_purpose: Some("research"),
            },
        )
        .expect("filtered catalog");

        assert_eq!(
            catalog
                .sessions
                .iter()
                .map(|session| session.id.as_str())
                .collect::<Vec<_>>(),
            vec!["session-1"]
        );
        assert!(!catalog.concurrent_active_sessions);
        assert!(!catalog.start_guard.can_start);
        assert_eq!(
            catalog.start_guard.blocking_session_id,
            Some("session-2".into())
        );
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
