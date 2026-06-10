use crate::action_classifier::is_high_risk_shell;
use crate::action_classifier::DenialCategory;
use crate::path_resolver::ProjectPathResolver;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellSessionAllow {
    pub command: String,
    pub approved_working_directory: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SessionAllowDecision {
    Allowed,
    RequiresApproval,
    Blocked(DenialResult),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DenialResult {
    pub category: String,
    pub message: String,
}

pub struct SessionAllowMatcher {
    project_root: PathBuf,
}

impl SessionAllowMatcher {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, String> {
        let resolver =
            ProjectPathResolver::new(project_root).map_err(|error| format!("{error:?}"))?;

        Ok(Self {
            project_root: resolver
                .resolve_existing(
                    ".",
                    crate::path_resolver::FileAccessActor::Browser,
                    crate::path_resolver::FileAccessOperation::Read,
                )
                .map_err(|error| format!("{error:?}"))?
                .project_root,
        })
    }

    pub fn evaluate_shell(
        &self,
        rule: Option<&ShellSessionAllow>,
        command: &str,
        working_directory: impl AsRef<Path>,
    ) -> SessionAllowDecision {
        let working_directory = match std::fs::canonicalize(working_directory) {
            Ok(path) => path,
            Err(_) => {
                return SessionAllowDecision::Blocked(denial_result(
                    DenialCategory::OutsideProjectRoot,
                ));
            }
        };

        if !working_directory.starts_with(&self.project_root) {
            return SessionAllowDecision::Blocked(denial_result(
                DenialCategory::OutsideProjectRoot,
            ));
        }

        if is_high_risk_shell(command) {
            return SessionAllowDecision::RequiresApproval;
        }

        let Some(rule) = rule else {
            return SessionAllowDecision::RequiresApproval;
        };

        if rule.command == command && working_directory == rule.approved_working_directory {
            SessionAllowDecision::Allowed
        } else {
            SessionAllowDecision::RequiresApproval
        }
    }
}

pub fn denial_result(category: DenialCategory) -> DenialResult {
    match category {
        DenialCategory::OutsideProjectRoot => DenialResult {
            category: "outside_project_root".into(),
            message: "The action targets a path outside the selected project root.".into(),
        },
        DenialCategory::SecretDenied => DenialResult {
            category: "secret_denied".into(),
            message: "The action targets a secret-deny file and is blocked by MVP policy.".into(),
        },
        DenialCategory::MvpScopeBlocked => DenialResult {
            category: "mvp_scope_blocked".into(),
            message: "The action is outside the MVP execution scope.".into(),
        },
    }
}

pub fn user_denied_result() -> DenialResult {
    DenialResult {
        category: "user_denied".into(),
        message: "The user denied this action.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn exact_non_destructive_shell_command_can_use_session_allow() {
        let directory = tempdir().expect("tempdir");
        let matcher = SessionAllowMatcher::new(directory.path()).expect("matcher");
        let rule = ShellSessionAllow {
            command: "npm test".into(),
            approved_working_directory: directory.path().canonicalize().expect("canonical"),
        };

        let decision = matcher.evaluate_shell(Some(&rule), "npm test", directory.path());

        assert_eq!(decision, SessionAllowDecision::Allowed);
    }

    #[test]
    fn different_shell_command_requires_fresh_approval() {
        let directory = tempdir().expect("tempdir");
        let matcher = SessionAllowMatcher::new(directory.path()).expect("matcher");
        let rule = ShellSessionAllow {
            command: "npm test".into(),
            approved_working_directory: directory.path().canonicalize().expect("canonical"),
        };

        let decision = matcher.evaluate_shell(Some(&rule), "npm run build", directory.path());

        assert_eq!(decision, SessionAllowDecision::RequiresApproval);
    }

    #[test]
    fn destructive_shell_command_never_uses_session_allow() {
        let directory = tempdir().expect("tempdir");
        let matcher = SessionAllowMatcher::new(directory.path()).expect("matcher");
        let rule = ShellSessionAllow {
            command: "rm -rf dist".into(),
            approved_working_directory: directory.path().canonicalize().expect("canonical"),
        };

        let decision = matcher.evaluate_shell(Some(&rule), "rm -rf dist", directory.path());

        assert_eq!(decision, SessionAllowDecision::RequiresApproval);
    }

    #[test]
    fn outside_root_working_directory_is_structured_denial() {
        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let matcher = SessionAllowMatcher::new(directory.path()).expect("matcher");

        let decision = matcher.evaluate_shell(None, "npm test", outside.path());

        assert_eq!(
            decision,
            SessionAllowDecision::Blocked(DenialResult {
                category: "outside_project_root".into(),
                message: "The action targets a path outside the selected project root.".into(),
            })
        );
    }

    #[test]
    fn user_denial_has_structured_result() {
        assert_eq!(
            user_denied_result(),
            DenialResult {
                category: "user_denied".into(),
                message: "The user denied this action.".into(),
            }
        );
    }
}
