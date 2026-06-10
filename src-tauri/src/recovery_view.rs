use crate::storage::{AppStore, NewMessage, SessionRecord};

#[derive(Debug, Eq, PartialEq)]
pub struct RecoveryControls {
    pub can_stop: bool,
    pub can_retry: bool,
    pub retry_appends_new_action: bool,
    pub auto_continue_after_restart: bool,
    pub reattach_after_restart: bool,
    pub resend_pending_prompts_after_restart: bool,
    pub banner: Option<RecoveryBanner>,
    pub minimized_window_message: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RecoveryBanner {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RetryError {
    MissingSession,
    SessionStillActive,
    StoreFailed,
}

pub struct RecoveryView;

impl RecoveryView {
    pub fn controls_for_session(
        app_store: &AppStore,
        session_id: &str,
    ) -> rusqlite::Result<Option<RecoveryControls>> {
        app_store
            .get_session(session_id)?
            .map(|session| controls_for_status(&session))
            .transpose()
    }

    pub fn append_retry_action(
        app_store: &AppStore,
        session_id: &str,
        message_id: &str,
        retry_prompt: &str,
    ) -> Result<(), RetryError> {
        let session = app_store
            .get_session(session_id)
            .map_err(|_| RetryError::StoreFailed)?
            .ok_or(RetryError::MissingSession)?;

        if is_active_status(&session.status) {
            return Err(RetryError::SessionStillActive);
        }

        app_store
            .append_message(NewMessage {
                id: message_id,
                session_id,
                role: "user",
                content: retry_prompt,
                status: "submitted",
                metadata_json: r#"{"retry":true}"#,
            })
            .map_err(|_| RetryError::StoreFailed)?;

        Ok(())
    }

    pub fn restart_recovery_policy() -> RecoveryControls {
        RecoveryControls {
            can_stop: false,
            can_retry: false,
            retry_appends_new_action: true,
            auto_continue_after_restart: false,
            reattach_after_restart: false,
            resend_pending_prompts_after_restart: false,
            banner: Some(RecoveryBanner {
                kind: "restart_recovery".into(),
                message: "Previous active sessions were interrupted. Review the transcript before retrying.".into(),
            }),
            minimized_window_message: None,
        }
    }
}

fn controls_for_status(session: &SessionRecord) -> rusqlite::Result<RecoveryControls> {
    let active = is_active_status(&session.status);
    let retryable = matches!(
        session.status.as_str(),
        "failed" | "stopped" | "interrupted"
    );

    Ok(RecoveryControls {
        can_stop: active,
        can_retry: retryable,
        retry_appends_new_action: true,
        auto_continue_after_restart: false,
        reattach_after_restart: false,
        resend_pending_prompts_after_restart: false,
        banner: recovery_banner(&session.status),
        minimized_window_message: if active {
            Some("Execution continues while the app window is minimized.".into())
        } else {
            None
        },
    })
}

fn is_active_status(status: &str) -> bool {
    matches!(status, "running" | "waiting_for_approval")
}

fn recovery_banner(status: &str) -> Option<RecoveryBanner> {
    match status {
        "failed" => Some(RecoveryBanner {
            kind: "provider_failure".into(),
            message: "The run failed. Retry will append a new action to the transcript.".into(),
        }),
        "interrupted" => Some(RecoveryBanner {
            kind: "crash_recovery".into(),
            message: "The previous run was interrupted after restart and will not auto-continue."
                .into(),
        }),
        "stopped" => Some(RecoveryBanner {
            kind: "stopped".into(),
            message: "The run was stopped. Retry will append a new action to the transcript."
                .into(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval::{ApprovalManager, PendingApproval};
    use crate::storage::{AppStore, NewProject, NewSession};

    #[test]
    fn running_session_exposes_stop_control_and_minimized_message() {
        let store = prepared_store("running");

        let controls = RecoveryView::controls_for_session(&store, "session-1")
            .expect("controls query")
            .expect("controls");

        assert!(controls.can_stop);
        assert!(!controls.can_retry);
        assert_eq!(
            controls.minimized_window_message,
            Some("Execution continues while the app window is minimized.".into())
        );
    }

    #[test]
    fn provider_failure_retry_appends_without_replacing_failed_message() {
        let store = prepared_store("failed");
        store
            .append_message(NewMessage {
                id: "message-1",
                session_id: "session-1",
                role: "assistant",
                content: "provider failed",
                status: "failed",
                metadata_json: "{}",
            })
            .expect("failed message inserted");

        RecoveryView::append_retry_action(&store, "session-1", "message-2", "try again")
            .expect("retry appended");

        let messages = store.list_messages("session-1").expect("messages");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "provider failed");
        assert_eq!(messages[0].status, "failed");
        assert_eq!(messages[1].content, "try again");
        assert_eq!(messages[1].status, "submitted");
    }

    #[test]
    fn retry_is_blocked_while_session_is_active() {
        let store = prepared_store("waiting_for_approval");

        let result =
            RecoveryView::append_retry_action(&store, "session-1", "message-1", "try again");

        assert_eq!(result, Err(RetryError::SessionStillActive));
    }

    #[test]
    fn crash_marker_disables_auto_continue_reattach_and_prompt_replay() {
        let store = prepared_store("interrupted");

        let controls = RecoveryView::controls_for_session(&store, "session-1")
            .expect("controls query")
            .expect("controls");

        assert_eq!(
            controls.banner,
            Some(RecoveryBanner {
                kind: "crash_recovery".into(),
                message:
                    "The previous run was interrupted after restart and will not auto-continue."
                        .into()
            })
        );
        assert!(!controls.auto_continue_after_restart);
        assert!(!controls.reattach_after_restart);
        assert!(!controls.resend_pending_prompts_after_restart);
    }

    #[test]
    fn restart_policy_discards_in_memory_pending_approval_prompts() {
        let mut manager = ApprovalManager::new();
        manager.create_pending(PendingApproval {
            id: "approval-1".into(),
            tool_call_id: "tool-1".into(),
            action_type: "shell".into(),
            target_summary: "npm test".into(),
            risk_level: "medium".into(),
        });

        let restarted_manager = ApprovalManager::new();
        let policy = RecoveryView::restart_recovery_policy();

        assert_eq!(manager.pending_count(), 1);
        assert_eq!(restarted_manager.pending_count(), 0);
        assert!(!policy.resend_pending_prompts_after_restart);
        assert_eq!(
            policy.banner.expect("banner").kind,
            "restart_recovery".to_string()
        );
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
}
