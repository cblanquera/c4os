use crate::approval::PendingApproval;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalPromptInput {
    pub pending: PendingApproval,
    pub preview: ApprovalPreview,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalPreview {
    FileWrite {
        path: String,
        diff_summary: Option<String>,
    },
    BatchWrite {
        file_count: usize,
        paths: Vec<String>,
    },
    ShellCommand {
        command: String,
        working_directory: String,
        destructive: bool,
    },
    GitStateChange {
        args: Vec<String>,
        destructive: bool,
    },
    PolicyBlock {
        category: String,
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalPrompt {
    pub approval_id: String,
    pub tool_call_id: String,
    pub action_type: String,
    pub target: String,
    pub risk_category: String,
    pub title: String,
    pub preview_state: String,
    pub summary: String,
    pub choices: Vec<ApprovalChoice>,
    pub blocked_by_policy: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalChoice {
    AllowOnce,
    AllowSession,
    Deny,
}

pub struct ApprovalPromptView;

impl ApprovalPromptView {
    pub fn project(input: ApprovalPromptInput) -> ApprovalPrompt {
        let blocked_by_policy = matches!(input.preview, ApprovalPreview::PolicyBlock { .. });
        let high_risk =
            input.pending.risk_level == "high" || preview_is_destructive(&input.preview);
        let choices = approval_choices(high_risk, blocked_by_policy);
        let (title, preview_state, summary) = prompt_copy(&input.preview, high_risk);

        ApprovalPrompt {
            approval_id: input.pending.id,
            tool_call_id: input.pending.tool_call_id,
            action_type: input.pending.action_type,
            target: input.pending.target_summary,
            risk_category: if high_risk {
                "high".into()
            } else {
                input.pending.risk_level
            },
            title,
            preview_state,
            summary,
            choices,
            blocked_by_policy,
        }
    }
}

fn approval_choices(high_risk: bool, blocked_by_policy: bool) -> Vec<ApprovalChoice> {
    if blocked_by_policy {
        return vec![ApprovalChoice::Deny];
    }

    if high_risk {
        return vec![ApprovalChoice::AllowOnce, ApprovalChoice::Deny];
    }

    vec![
        ApprovalChoice::AllowOnce,
        ApprovalChoice::AllowSession,
        ApprovalChoice::Deny,
    ]
}

fn preview_is_destructive(preview: &ApprovalPreview) -> bool {
    match preview {
        ApprovalPreview::ShellCommand { destructive, .. }
        | ApprovalPreview::GitStateChange { destructive, .. } => *destructive,
        _ => false,
    }
}

fn prompt_copy(preview: &ApprovalPreview, high_risk: bool) -> (String, String, String) {
    match preview {
        ApprovalPreview::FileWrite { path, diff_summary } => (
            "Approve file write".into(),
            "file_write_preview".into(),
            diff_summary
                .clone()
                .unwrap_or_else(|| format!("Write changes to {path}.")),
        ),
        ApprovalPreview::BatchWrite { file_count, paths } => (
            "Approve batch file write".into(),
            "batch_write_preview".into(),
            format!("Write changes to {file_count} files: {}.", paths.join(", ")),
        ),
        ApprovalPreview::ShellCommand {
            command,
            working_directory,
            ..
        } => (
            if high_risk {
                "Approve high-risk shell command".into()
            } else {
                "Approve shell command".into()
            },
            "shell_command_summary".into(),
            format!("Run `{command}` in {working_directory}."),
        ),
        ApprovalPreview::GitStateChange { args, .. } => (
            if high_risk {
                "Approve high-risk Git change".into()
            } else {
                "Approve Git change".into()
            },
            "git_state_change_summary".into(),
            format!("Run `git {}`.", args.join(" ")),
        ),
        ApprovalPreview::PolicyBlock { category, message } => (
            "Blocked by policy".into(),
            "blocked_by_policy".into(),
            format!("{category}: {message}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_write_prompt_includes_preview_and_all_choices() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "file_write", "README.md", "medium"),
            preview: ApprovalPreview::FileWrite {
                path: "README.md".into(),
                diff_summary: Some("+ added setup notes".into()),
            },
        });

        assert_eq!(prompt.title, "Approve file write");
        assert_eq!(prompt.preview_state, "file_write_preview");
        assert_eq!(prompt.summary, "+ added setup notes");
        assert_eq!(
            prompt.choices,
            vec![
                ApprovalChoice::AllowOnce,
                ApprovalChoice::AllowSession,
                ApprovalChoice::Deny
            ]
        );
    }

    #[test]
    fn batch_write_prompt_summarizes_file_count_and_paths() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "file_write_batch", "3 files", "medium"),
            preview: ApprovalPreview::BatchWrite {
                file_count: 3,
                paths: vec!["a.md".into(), "b.md".into(), "c.md".into()],
            },
        });

        assert_eq!(prompt.preview_state, "batch_write_preview");
        assert!(prompt.summary.contains("3 files"));
        assert!(prompt.summary.contains("a.md, b.md, c.md"));
    }

    #[test]
    fn shell_prompt_allows_session_for_non_destructive_command() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "shell", "npm test", "medium"),
            preview: ApprovalPreview::ShellCommand {
                command: "npm test".into(),
                working_directory: "/project".into(),
                destructive: false,
            },
        });

        assert_eq!(prompt.title, "Approve shell command");
        assert_eq!(prompt.preview_state, "shell_command_summary");
        assert!(prompt.choices.contains(&ApprovalChoice::AllowSession));
    }

    #[test]
    fn destructive_shell_prompt_never_allows_session_scope() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "shell", "rm -rf dist", "high"),
            preview: ApprovalPreview::ShellCommand {
                command: "rm -rf dist".into(),
                working_directory: "/project".into(),
                destructive: true,
            },
        });

        assert_eq!(prompt.title, "Approve high-risk shell command");
        assert_eq!(prompt.risk_category, "high");
        assert_eq!(
            prompt.choices,
            vec![ApprovalChoice::AllowOnce, ApprovalChoice::Deny]
        );
    }

    #[test]
    fn high_risk_git_change_never_allows_session_scope() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "git", "git reset --hard", "high"),
            preview: ApprovalPreview::GitStateChange {
                args: vec!["reset".into(), "--hard".into()],
                destructive: true,
            },
        });

        assert_eq!(prompt.title, "Approve high-risk Git change");
        assert_eq!(
            prompt.choices,
            vec![ApprovalChoice::AllowOnce, ApprovalChoice::Deny]
        );
    }

    #[test]
    fn deny_choice_is_always_present_for_reviewable_prompts() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "shell", "npm run build", "medium"),
            preview: ApprovalPreview::ShellCommand {
                command: "npm run build".into(),
                working_directory: "/project".into(),
                destructive: false,
            },
        });

        assert!(prompt.choices.contains(&ApprovalChoice::Deny));
    }

    #[test]
    fn policy_block_prompt_has_no_execution_approval_choices() {
        let prompt = ApprovalPromptView::project(ApprovalPromptInput {
            pending: pending("approval-1", "file_read", ".env", "blocked"),
            preview: ApprovalPreview::PolicyBlock {
                category: "secret_denied".into(),
                message: "Secret-deny files are blocked.".into(),
            },
        });

        assert!(prompt.blocked_by_policy);
        assert_eq!(prompt.preview_state, "blocked_by_policy");
        assert_eq!(prompt.choices, vec![ApprovalChoice::Deny]);
    }

    fn pending(
        id: &str,
        action_type: &str,
        target_summary: &str,
        risk_level: &str,
    ) -> PendingApproval {
        PendingApproval {
            id: id.into(),
            tool_call_id: "tool-1".into(),
            action_type: action_type.into(),
            target_summary: target_summary.into(),
            risk_level: risk_level.into(),
        }
    }
}
