use crate::path_resolver::{
    FileAccessActor, FileAccessOperation, PathResolutionError, ProjectPathResolver,
};
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub enum RuntimeAction<'a> {
    FileRead {
        path: &'a str,
    },
    FileWrite {
        path: &'a str,
    },
    ShellCommand {
        command: &'a str,
        working_directory: &'a Path,
    },
    GitCommand {
        args: Vec<&'a str>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ActionDecision {
    AllowLogged,
    RequiresApproval { risk: RiskLevel },
    Blocked { category: DenialCategory },
}

#[derive(Debug, Eq, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DenialCategory {
    OutsideProjectRoot,
    SecretDenied,
    MvpScopeBlocked,
}

pub struct ActionClassifier {
    resolver: ProjectPathResolver,
}

impl ActionClassifier {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, PathResolutionError> {
        Ok(Self {
            resolver: ProjectPathResolver::new(project_root)?,
        })
    }

    pub fn classify(&self, action: RuntimeAction<'_>) -> ActionDecision {
        match action {
            RuntimeAction::FileRead { path } => self.classify_file_read(path),
            RuntimeAction::FileWrite { path } => self.classify_file_write(path),
            RuntimeAction::ShellCommand {
                command,
                working_directory,
            } => self.classify_shell(command, working_directory),
            RuntimeAction::GitCommand { args } => classify_git(args),
        }
    }

    fn classify_file_read(&self, path: &str) -> ActionDecision {
        match self.resolver.resolve_existing(
            path,
            FileAccessActor::Agent,
            FileAccessOperation::Read,
        ) {
            Ok(_) => ActionDecision::AllowLogged,
            Err(PathResolutionError::SecretDenied) => ActionDecision::Blocked {
                category: DenialCategory::SecretDenied,
            },
            Err(PathResolutionError::OutsideProjectRoot) => ActionDecision::Blocked {
                category: DenialCategory::OutsideProjectRoot,
            },
            Err(_) => ActionDecision::Blocked {
                category: DenialCategory::MvpScopeBlocked,
            },
        }
    }

    fn classify_file_write(&self, path: &str) -> ActionDecision {
        match self.resolver.resolve_existing(
            path,
            FileAccessActor::Agent,
            FileAccessOperation::Write,
        ) {
            Ok(_) => ActionDecision::RequiresApproval {
                risk: RiskLevel::Medium,
            },
            Err(PathResolutionError::SecretDenied) => ActionDecision::Blocked {
                category: DenialCategory::SecretDenied,
            },
            Err(PathResolutionError::OutsideProjectRoot) => ActionDecision::Blocked {
                category: DenialCategory::OutsideProjectRoot,
            },
            Err(_) => ActionDecision::RequiresApproval {
                risk: RiskLevel::Medium,
            },
        }
    }

    fn classify_shell(&self, command: &str, working_directory: &Path) -> ActionDecision {
        if ProjectPathResolver::new(working_directory).is_err() {
            return ActionDecision::Blocked {
                category: DenialCategory::OutsideProjectRoot,
            };
        }

        if is_high_risk_shell(command) {
            return ActionDecision::RequiresApproval {
                risk: RiskLevel::High,
            };
        }

        ActionDecision::RequiresApproval {
            risk: RiskLevel::Medium,
        }
    }
}

fn classify_git(args: Vec<&str>) -> ActionDecision {
    let read_only = matches!(
        args.as_slice(),
        ["status"] | ["diff"] | ["diff", ..] | ["branch", "--show-current"] | ["rev-parse", ..]
    );

    if read_only {
        return ActionDecision::AllowLogged;
    }

    ActionDecision::RequiresApproval {
        risk: if args.iter().any(|arg| {
            matches!(
                *arg,
                "reset" | "clean" | "checkout" | "restore" | "push" | "rebase"
            ) || *arg == "--hard"
        }) {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        },
    }
}

pub(crate) fn is_high_risk_shell(command: &str) -> bool {
    let lower = command.to_lowercase();

    lower.contains("rm -rf")
        || lower.contains("find . -delete")
        || lower.contains("git clean")
        || lower.contains("git reset --hard")
        || lower.contains("chmod -r")
        || lower.contains("chown -r")
        || lower.contains("chgrp -r")
        || lower.starts_with("dd ")
        || lower.contains("mkfs")
        || lower.contains("diskutil erase")
        || lower.contains("npm uninstall")
        || lower.contains("npm prune")
        || lower.contains("pnpm remove")
        || lower.contains("yarn remove")
        || lower.contains("pip uninstall")
        || lower.contains("cargo uninstall")
        || lower.contains("kill -9")
        || lower.contains("pkill")
        || lower.contains("killall")
        || lower.contains("curl ") && lower.contains("| sh")
        || lower.contains("wget ") && lower.contains("| sh")
        || lower.contains("eval ")
        || lower.contains("python -c")
        || lower.contains("base64") && lower.contains("| sh")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn allows_logged_project_root_file_reads() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::FileRead { path: "README.md" });

        assert_eq!(decision, ActionDecision::AllowLogged);
    }

    #[test]
    fn blocks_secret_denied_file_reads() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join(".env"), "secret").expect("file written");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::FileRead { path: ".env" });

        assert_eq!(
            decision,
            ActionDecision::Blocked {
                category: DenialCategory::SecretDenied
            }
        );
    }

    #[test]
    fn file_writes_require_approval() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::FileWrite { path: "README.md" });

        assert_eq!(
            decision,
            ActionDecision::RequiresApproval {
                risk: RiskLevel::Medium
            }
        );
    }

    #[test]
    fn read_only_git_inspection_is_allowed_logged() {
        let directory = tempdir().expect("tempdir");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::GitCommand {
            args: vec!["status"],
        });

        assert_eq!(decision, ActionDecision::AllowLogged);
    }

    #[test]
    fn git_state_changes_require_high_risk_approval() {
        let directory = tempdir().expect("tempdir");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::GitCommand {
            args: vec!["reset", "--hard"],
        });

        assert_eq!(
            decision,
            ActionDecision::RequiresApproval {
                risk: RiskLevel::High
            }
        );
    }

    #[test]
    fn high_risk_shell_commands_require_high_risk_approval() {
        let directory = tempdir().expect("tempdir");
        let classifier = ActionClassifier::new(directory.path()).expect("classifier");

        let decision = classifier.classify(RuntimeAction::ShellCommand {
            command: "rm -rf dist",
            working_directory: directory.path(),
        });

        assert_eq!(
            decision,
            ActionDecision::RequiresApproval {
                risk: RiskLevel::High
            }
        );
    }
}
