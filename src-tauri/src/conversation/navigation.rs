use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// Minimal Project projection accepted from an authoritative Workspace snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationProject {
    pub project_id: String,
    pub display_name: String,
    pub position: i64,
    pub is_active: bool,
}

/// Minimal Chat projection accepted from authoritative Workspace/session data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationSession {
    pub session_id: String,
    pub project_id: String,
    pub title: String,
    pub position: i64,
    pub is_active: bool,
}

/// Flat deterministic result used while non-empty search replaces the tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSearchResult {
    pub session_id: String,
    pub session_title: String,
    pub project_id: String,
    pub project_name: String,
}

/// Active navigation may point at an active Project and one of its active Chats.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActiveNavigation {
    pub active_project_id: Option<String>,
    pub active_session_id: Option<String>,
    pub pending_project_id: Option<String>,
}

/// Only the requested product record is inactivated by a returned plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InactivationTarget {
    Project(String),
    Session(String),
}

/// Pure selection result; persistence remains owned by Workspace services.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InactivationPlan {
    pub target: InactivationTarget,
    pub next_navigation: ActiveNavigation,
    pub cancel_pending_chat: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InactivationError {
    UnknownTarget,
    AlreadyInactive,
    DuplicateIdentifier,
    InvalidActiveSelection,
}

/// Trims, collapses whitespace, and lowercases a search value consistently.
pub fn normalize_search_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Returns active Chat matches in deterministic Project/sidebar order.
pub fn search_sessions(
    projects: &[NavigationProject],
    sessions: &[NavigationSession],
    query: &str,
) -> Vec<SessionSearchResult> {
    let query = normalize_search_text(query);
    if query.is_empty() {
        return Vec::new();
    }

    let project_by_id = projects
        .iter()
        .filter(|project| project.is_active)
        .map(|project| (project.project_id.as_str(), project))
        .collect::<BTreeMap<_, _>>();
    let mut matches = sessions
        .iter()
        .filter(|session| session.is_active)
        .filter_map(|session| {
            let project = project_by_id.get(session.project_id.as_str())?;
            normalize_search_text(&session.title)
                .contains(&query)
                .then_some((project, session))
        })
        .collect::<Vec<_>>();
    matches.sort_by(
        |(left_project, left_session), (right_project, right_session)| {
            compare_project(left_project, right_project).then_with(|| {
                left_session
                    .position
                    .cmp(&right_session.position)
                    .then_with(|| {
                        normalize_search_text(&left_session.title)
                            .cmp(&normalize_search_text(&right_session.title))
                    })
                    .then_with(|| left_session.session_id.cmp(&right_session.session_id))
            })
        },
    );
    matches
        .into_iter()
        .map(|(project, session)| SessionSearchResult {
            session_id: session.session_id.clone(),
            session_title: session.title.clone(),
            project_id: project.project_id.clone(),
            project_name: project.display_name.clone(),
        })
        .collect()
}

/// Plans one inactivation and a valid fallback without mutating caller data.
pub fn plan_inactivation(
    projects: &[NavigationProject],
    sessions: &[NavigationSession],
    current: &ActiveNavigation,
    target: InactivationTarget,
) -> Result<InactivationPlan, InactivationError> {
    validate_inputs(projects, sessions, current)?;
    match &target {
        InactivationTarget::Project(project_id) => {
            let project = projects
                .iter()
                .find(|project| &project.project_id == project_id)
                .ok_or(InactivationError::UnknownTarget)?;
            if !project.is_active {
                return Err(InactivationError::AlreadyInactive);
            }
            let is_active_project = current.active_project_id.as_ref() == Some(project_id);
            let cancel_pending_chat = current.pending_project_id.as_ref() == Some(project_id);
            let next_navigation = if is_active_project {
                fallback_project_navigation(projects, sessions, project_id)
            } else {
                current.clone()
            };
            Ok(InactivationPlan {
                target,
                next_navigation,
                cancel_pending_chat,
            })
        }
        InactivationTarget::Session(session_id) => {
            let session = sessions
                .iter()
                .find(|session| &session.session_id == session_id)
                .ok_or(InactivationError::UnknownTarget)?;
            if !session.is_active {
                return Err(InactivationError::AlreadyInactive);
            }
            let mut next_navigation = current.clone();
            if current.active_session_id.as_ref() == Some(session_id) {
                next_navigation.active_session_id =
                    fallback_session_id(sessions, &session.project_id, session_id);
            }
            Ok(InactivationPlan {
                target,
                next_navigation,
                cancel_pending_chat: false,
            })
        }
    }
}

fn validate_inputs(
    projects: &[NavigationProject],
    sessions: &[NavigationSession],
    current: &ActiveNavigation,
) -> Result<(), InactivationError> {
    if has_duplicates(projects.iter().map(|project| project.project_id.as_str()))
        || has_duplicates(sessions.iter().map(|session| session.session_id.as_str()))
    {
        return Err(InactivationError::DuplicateIdentifier);
    }
    let active_project = current.active_project_id.as_deref().map(|project_id| {
        projects
            .iter()
            .find(|project| project.project_id == project_id && project.is_active)
    });
    if active_project.is_some_and(|project| project.is_none()) {
        return Err(InactivationError::InvalidActiveSelection);
    }
    if let Some(session_id) = current.active_session_id.as_deref() {
        let Some(project_id) = current.active_project_id.as_deref() else {
            return Err(InactivationError::InvalidActiveSelection);
        };
        if !sessions.iter().any(|session| {
            session.session_id == session_id
                && session.project_id == project_id
                && session.is_active
        }) {
            return Err(InactivationError::InvalidActiveSelection);
        }
    }
    if let Some(pending_project_id) = current.pending_project_id.as_deref()
        && (current.active_project_id.as_deref() != Some(pending_project_id)
            || !projects
                .iter()
                .any(|project| project.project_id == pending_project_id && project.is_active))
    {
        return Err(InactivationError::InvalidActiveSelection);
    }
    Ok(())
}

fn fallback_project_navigation(
    projects: &[NavigationProject],
    sessions: &[NavigationSession],
    removed_project_id: &str,
) -> ActiveNavigation {
    let ordered = ordered_active_projects(projects);
    let next_project_id = adjacent_fallback(&ordered, removed_project_id, |project| {
        project.project_id.as_str()
    })
    .map(|project| project.project_id.clone());
    let active_session_id = next_project_id
        .as_deref()
        .and_then(|project_id| first_session_id(sessions, project_id));
    ActiveNavigation {
        active_project_id: next_project_id,
        active_session_id,
        pending_project_id: None,
    }
}

fn fallback_session_id(
    sessions: &[NavigationSession],
    project_id: &str,
    removed_session_id: &str,
) -> Option<String> {
    let ordered = ordered_active_sessions(sessions, project_id);
    adjacent_fallback(&ordered, removed_session_id, |session| {
        session.session_id.as_str()
    })
    .map(|session| session.session_id.clone())
}

fn first_session_id(sessions: &[NavigationSession], project_id: &str) -> Option<String> {
    ordered_active_sessions(sessions, project_id)
        .first()
        .map(|session| session.session_id.clone())
}

fn ordered_active_projects(projects: &[NavigationProject]) -> Vec<&NavigationProject> {
    let mut ordered = projects
        .iter()
        .filter(|project| project.is_active)
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| compare_project(left, right));
    ordered
}

fn ordered_active_sessions<'a>(
    sessions: &'a [NavigationSession],
    project_id: &str,
) -> Vec<&'a NavigationSession> {
    let mut ordered = sessions
        .iter()
        .filter(|session| session.project_id == project_id && session.is_active)
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    ordered
}

fn compare_project(left: &NavigationProject, right: &NavigationProject) -> Ordering {
    left.position
        .cmp(&right.position)
        .then_with(|| left.project_id.cmp(&right.project_id))
}

fn adjacent_fallback<'a, T>(
    ordered: &'a [&T],
    removed_id: &str,
    id: impl Fn(&T) -> &str,
) -> Option<&'a T> {
    let removed_index = ordered
        .iter()
        .position(|candidate| id(candidate) == removed_id)?;
    ordered
        .get(removed_index + 1)
        .or_else(|| {
            removed_index
                .checked_sub(1)
                .and_then(|index| ordered.get(index))
        })
        .copied()
}

fn has_duplicates<'a>(values: impl IntoIterator<Item = &'a str>) -> bool {
    let mut unique = BTreeSet::new();
    values.into_iter().any(|value| !unique.insert(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, name: &str, position: i64) -> NavigationProject {
        NavigationProject {
            project_id: id.into(),
            display_name: name.into(),
            position,
            is_active: true,
        }
    }

    fn session(id: &str, project_id: &str, title: &str, position: i64) -> NavigationSession {
        NavigationSession {
            session_id: id.into(),
            project_id: project_id.into(),
            title: title.into(),
            position,
            is_active: true,
        }
    }

    #[test]
    fn search_is_normalized_flat_and_deterministic() {
        let projects = vec![
            project("project-b", "Beta", 2),
            project("project-a", "Alpha", 1),
        ];
        let sessions = vec![
            session("session-b", "project-b", "Release   Plan", 1),
            session("session-a2", "project-a", "release notes", 2),
            session("session-a1", "project-a", "Release plan", 1),
        ];
        let results = search_sessions(&projects, &sessions, "  RELEASE plan ");
        assert_eq!(
            results
                .iter()
                .map(|result| result.session_id.as_str())
                .collect::<Vec<_>>(),
            ["session-a1", "session-b"]
        );
        assert_eq!(results[0].project_name, "Alpha");
        assert!(search_sessions(&projects, &sessions, "  ").is_empty());
    }

    #[test]
    fn inactive_projects_and_sessions_never_leak_into_search() {
        let mut hidden_project = project("hidden", "Hidden", 1);
        hidden_project.is_active = false;
        let mut hidden_session = session("inactive", "visible", "Find me", 1);
        hidden_session.is_active = false;
        assert!(
            search_sessions(
                &[hidden_project, project("visible", "Visible", 2)],
                &[
                    session("under-hidden", "hidden", "Find me", 1),
                    hidden_session,
                ],
                "find me",
            )
            .is_empty()
        );
    }

    #[test]
    fn inactivating_active_session_selects_an_adjacent_active_fallback() {
        let projects = vec![project("project", "Project", 1)];
        let sessions = vec![
            session("one", "project", "One", 1),
            session("two", "project", "Two", 2),
            session("three", "project", "Three", 3),
        ];
        let plan = plan_inactivation(
            &projects,
            &sessions,
            &ActiveNavigation {
                active_project_id: Some("project".into()),
                active_session_id: Some("two".into()),
                pending_project_id: None,
            },
            InactivationTarget::Session("two".into()),
        )
        .unwrap();
        assert_eq!(
            plan.next_navigation.active_session_id.as_deref(),
            Some("three")
        );
        assert!(!plan.cancel_pending_chat);
    }

    #[test]
    fn inactivating_pending_project_cancels_pending_and_selects_valid_project() {
        let projects = vec![
            project("one", "One", 1),
            project("two", "Two", 2),
            project("three", "Three", 3),
        ];
        let sessions = vec![session("three-chat", "three", "Chat", 1)];
        let plan = plan_inactivation(
            &projects,
            &sessions,
            &ActiveNavigation {
                active_project_id: Some("two".into()),
                active_session_id: None,
                pending_project_id: Some("two".into()),
            },
            InactivationTarget::Project("two".into()),
        )
        .unwrap();
        assert!(plan.cancel_pending_chat);
        assert_eq!(
            plan.next_navigation.active_project_id.as_deref(),
            Some("three")
        );
        assert_eq!(
            plan.next_navigation.active_session_id.as_deref(),
            Some("three-chat")
        );
        assert_eq!(plan.next_navigation.pending_project_id, None);
    }

    #[test]
    fn invalid_detached_selection_fails_closed() {
        let result = plan_inactivation(
            &[project("project", "Project", 1)],
            &[],
            &ActiveNavigation {
                active_project_id: Some("missing".into()),
                active_session_id: None,
                pending_project_id: None,
            },
            InactivationTarget::Project("project".into()),
        );
        assert_eq!(result, Err(InactivationError::InvalidActiveSelection));
    }
}
