use crate::storage::{AppStore, ProjectRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectSelectorState {
    pub projects: Vec<ProjectSelectorItem>,
    pub active_project_id: Option<String>,
    pub active_project: Option<ProjectSelectorItem>,
    pub multiple_active_projects_allowed: bool,
    pub search_available: bool,
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
    pub is_active: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProjectSelectionError {
    MissingProject,
    StoreFailed,
}

pub struct ProjectSelector;

impl ProjectSelector {
    pub fn list(app_store: &AppStore) -> rusqlite::Result<ProjectSelectorState> {
        let selected_project_id = app_store.selected_project_id()?;
        let projects = app_store.list_projects()?;
        let items = projects
            .into_iter()
            .map(|project| project_item(project, selected_project_id.as_deref()))
            .collect::<Vec<_>>();
        let active_project = items.iter().find(|project| project.is_active).cloned();

        Ok(ProjectSelectorState {
            projects: items,
            active_project_id: active_project.as_ref().map(|project| project.id.clone()),
            active_project,
            multiple_active_projects_allowed: false,
            search_available: false,
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

fn project_item(project: ProjectRecord, selected_project_id: Option<&str>) -> ProjectSelectorItem {
    let is_active = selected_project_id == Some(project.id.as_str());

    ProjectSelectorItem {
        id: project.id,
        name: project.name,
        root_path: project.root_path,
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

        assert!(!selector.search_available);
        assert!(!selector.grouping_available);
        assert!(!selector.archive_available);
        assert!(!selector.delete_available);
        assert!(!selector.favorites_available);
        assert!(!selector.metadata_editing_available);
        assert!(!selector.cross_project_views_available);
        assert!(!selector.non_git_projects_allowed);
        assert!(!selector.worktree_management_available);
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
