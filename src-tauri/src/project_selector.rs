use crate::storage::{AppStore, ProjectRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectSelectorState {
    pub projects: Vec<ProjectSelectorItem>,
    pub active_project_id: Option<String>,
    pub active_project: Option<ProjectSelectorItem>,
    pub search_query: Option<String>,
    pub workflow_purpose_filter: Option<String>,
    pub multiple_active_projects_allowed: bool,
    pub search_available: bool,
    pub workflow_purpose_filter_available: bool,
    pub grouping_available: bool,
    pub archive_available: bool,
    pub delete_available: bool,
    pub favorites_available: bool,
    pub metadata_editing_available: bool,
    pub cross_project_views_available: bool,
    pub non_git_projects_allowed: bool,
    pub worktree_management_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectSelectorItem {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub workflow_purpose: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct ProjectSelectorQuery<'a> {
    pub search: Option<&'a str>,
    pub workflow_purpose: Option<&'a str>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProjectSelectionError {
    MissingProject,
    StoreFailed,
}

pub struct ProjectSelector;

impl ProjectSelector {
    pub fn list(app_store: &AppStore) -> rusqlite::Result<ProjectSelectorState> {
        Self::list_filtered(app_store, ProjectSelectorQuery::default())
    }

    pub fn list_filtered(
        app_store: &AppStore,
        query: ProjectSelectorQuery<'_>,
    ) -> rusqlite::Result<ProjectSelectorState> {
        validate_workflow_purpose_filter(query.workflow_purpose)?;
        let selected_project_id = app_store.selected_project_id()?;
        let search_query = normalized_search(query.search);
        let projects = app_store.list_projects()?;
        let items = projects
            .into_iter()
            .filter(|project| matches_search(project, search_query.as_deref()))
            .filter(|project| matches_workflow_purpose(project, query.workflow_purpose))
            .map(|project| project_item(project, selected_project_id.as_deref()))
            .collect::<Vec<_>>();
        let active_project = items.iter().find(|project| project.is_active).cloned();

        Ok(ProjectSelectorState {
            projects: items,
            active_project_id: active_project.as_ref().map(|project| project.id.clone()),
            active_project,
            search_query,
            workflow_purpose_filter: query.workflow_purpose.map(str::to_string),
            multiple_active_projects_allowed: false,
            search_available: true,
            workflow_purpose_filter_available: true,
            grouping_available: false,
            archive_available: false,
            delete_available: false,
            favorites_available: false,
            metadata_editing_available: false,
            cross_project_views_available: false,
            non_git_projects_allowed: false,
            worktree_management_available: false,
        })
    }

    pub fn select(
        app_store: &AppStore,
        project_id: &str,
    ) -> Result<ProjectSelectorState, ProjectSelectionError> {
        if app_store
            .get_project(project_id)
            .map_err(|_| ProjectSelectionError::StoreFailed)?
            .is_none()
        {
            return Err(ProjectSelectionError::MissingProject);
        }

        app_store
            .set_selected_project(project_id)
            .map_err(|_| ProjectSelectionError::StoreFailed)?;
        Self::list(app_store).map_err(|_| ProjectSelectionError::StoreFailed)
    }
}

fn normalized_search(search: Option<&str>) -> Option<String> {
    search
        .map(str::trim)
        .filter(|search| !search.is_empty())
        .map(|search| search.to_lowercase())
}

fn matches_search(project: &ProjectRecord, search_query: Option<&str>) -> bool {
    let Some(search_query) = search_query else {
        return true;
    };
    let name = project.name.to_lowercase();
    let root_path = project.root_path.to_lowercase();

    name.contains(search_query) || root_path.contains(search_query)
}

fn matches_workflow_purpose(project: &ProjectRecord, workflow_purpose: Option<&str>) -> bool {
    workflow_purpose.is_none() || project.workflow_purpose.as_deref() == workflow_purpose
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

fn project_item(project: ProjectRecord, selected_project_id: Option<&str>) -> ProjectSelectorItem {
    let is_active = selected_project_id == Some(project.id.as_str());

    ProjectSelectorItem {
        id: project.id,
        name: project.name,
        root_path: project.root_path,
        workflow_purpose: project.workflow_purpose,
        is_active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject};

    #[test]
    fn selector_lists_registered_projects_without_active_project_by_default() {
        let store = prepared_store();

        let selector = ProjectSelector::list(&store).expect("selector");

        assert_eq!(selector.projects.len(), 2);
        assert_eq!(selector.active_project_id, None);
        assert_eq!(selector.active_project, None);
        assert_eq!(selector.search_query, None);
        assert_eq!(selector.workflow_purpose_filter, None);
        assert!(!selector.projects.iter().any(|project| project.is_active));
    }

    #[test]
    fn selecting_project_marks_exactly_one_active_project() {
        let store = prepared_store();

        let selector = ProjectSelector::select(&store, "project-2").expect("project selected");

        assert_eq!(selector.active_project_id, Some("project-2".into()));
        assert_eq!(
            selector
                .projects
                .iter()
                .filter(|project| project.is_active)
                .count(),
            1
        );
        assert_eq!(
            selector.active_project.map(|project| project.root_path),
            Some("/tmp/two".into())
        );
        assert!(!selector.multiple_active_projects_allowed);
    }

    #[test]
    fn selected_project_survives_relisted_selector_state() {
        let store = prepared_store();
        ProjectSelector::select(&store, "project-1").expect("project selected");

        let selector = ProjectSelector::list(&store).expect("selector");

        assert_eq!(selector.active_project_id, Some("project-1".into()));
        assert!(
            selector
                .projects
                .iter()
                .find(|project| project.id == "project-1")
                .expect("project exists")
                .is_active
        );
    }

    #[test]
    fn missing_project_selection_fails_closed() {
        let store = prepared_store();

        let result = ProjectSelector::select(&store, "missing");

        assert_eq!(result, Err(ProjectSelectionError::MissingProject));
        assert_eq!(
            ProjectSelector::list(&store)
                .expect("selector")
                .active_project_id,
            None
        );
    }

    #[test]
    fn selector_does_not_expose_postponed_project_management_controls() {
        let store = prepared_store();

        let selector = ProjectSelector::list(&store).expect("selector");

        assert!(selector.search_available);
        assert!(selector.workflow_purpose_filter_available);
        assert!(!selector.grouping_available);
        assert!(!selector.archive_available);
        assert!(!selector.delete_available);
        assert!(!selector.favorites_available);
        assert!(!selector.metadata_editing_available);
        assert!(!selector.cross_project_views_available);
        assert!(!selector.non_git_projects_allowed);
        assert!(!selector.worktree_management_available);
    }

    #[test]
    fn selector_filters_projects_by_search_without_changing_active_project() {
        let store = prepared_store();
        ProjectSelector::select(&store, "project-1").expect("project selected");

        let selector = ProjectSelector::list_filtered(
            &store,
            ProjectSelectorQuery {
                search: Some("two"),
                workflow_purpose: None,
            },
        )
        .expect("filtered selector");

        assert_eq!(selector.search_query, Some("two".into()));
        assert_eq!(
            selector
                .projects
                .iter()
                .map(|project| project.id.as_str())
                .collect::<Vec<_>>(),
            vec!["project-2"]
        );
        assert_eq!(selector.active_project_id, None);
        assert_eq!(
            ProjectSelector::list(&store)
                .expect("selector")
                .active_project_id,
            Some("project-1".into())
        );
    }

    #[test]
    fn selector_filters_projects_by_bounded_workflow_purpose() {
        let store = prepared_store();
        store
            .set_project_workflow_purpose("project-1", Some("research"))
            .expect("project one workflow purpose set");
        store
            .set_project_workflow_purpose("project-2", Some("writing"))
            .expect("project two workflow purpose set");

        let selector = ProjectSelector::list_filtered(
            &store,
            ProjectSelectorQuery {
                search: None,
                workflow_purpose: Some("research"),
            },
        )
        .expect("filtered selector");

        assert_eq!(selector.workflow_purpose_filter, Some("research".into()));
        assert_eq!(selector.projects.len(), 1);
        assert_eq!(selector.projects[0].id, "project-1");
        assert_eq!(
            selector.projects[0].workflow_purpose,
            Some("research".into())
        );

        let invalid = ProjectSelector::list_filtered(
            &store,
            ProjectSelectorQuery {
                search: None,
                workflow_purpose: Some("coding"),
            },
        );

        assert!(invalid.is_err());
    }

    fn prepared_store() -> AppStore {
        let store = AppStore::open_in_memory().expect("store opens");

        store
            .create_project(NewProject {
                id: "project-1",
                name: "One",
                root_path: "/tmp/one",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project one inserted");
        store
            .create_project(NewProject {
                id: "project-2",
                name: "Two",
                root_path: "/tmp/two",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project two inserted");

        store
    }
}
