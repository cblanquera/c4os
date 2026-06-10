use crate::storage::{AppStore, NewApproval};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingApproval {
    pub id: String,
    pub tool_call_id: String,
    pub action_type: String,
    pub target_summary: String,
    pub risk_level: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalDecision {
    AllowOnce,
    AllowSession,
    Deny,
}

#[derive(Debug)]
pub enum ApprovalError {
    MissingPendingApproval,
    StoreFailed(rusqlite::Error),
}

pub struct ApprovalManager {
    pending: HashMap<String, PendingApproval>,
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    pub fn create_pending(&mut self, approval: PendingApproval) {
        self.pending.insert(approval.id.clone(), approval);
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn answer(
        &mut self,
        app_store: &AppStore,
        approval_id: &str,
        decision: ApprovalDecision,
        decided_by: &str,
    ) -> Result<(), ApprovalError> {
        let pending = self
            .pending
            .remove(approval_id)
            .ok_or(ApprovalError::MissingPendingApproval)?;
        let decision_value = match decision {
            ApprovalDecision::AllowOnce => "allow_once",
            ApprovalDecision::AllowSession => "allow_session",
            ApprovalDecision::Deny => "deny",
        };

        app_store
            .record_approval(NewApproval {
                id: approval_id,
                tool_call_id: &pending.tool_call_id,
                approval_source: &pending.action_type,
                decision: decision_value,
                scope: &pending.target_summary,
                decided_by,
            })
            .map_err(ApprovalError::StoreFailed)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject, NewSession, NewToolCall};

    #[test]
    fn answered_approval_is_durable_and_pending_is_removed() {
        let store = prepared_store_with_tool_call();
        let mut manager = ApprovalManager::new();

        manager.create_pending(PendingApproval {
            id: "approval-1".into(),
            tool_call_id: "tool-1".into(),
            action_type: "file_write".into(),
            target_summary: "README.md".into(),
            risk_level: "medium".into(),
        });
        manager
            .answer(
                &store,
                "approval-1",
                ApprovalDecision::AllowOnce,
                "local-user",
            )
            .expect("approval answered");

        assert_eq!(manager.pending_count(), 0);
        assert_eq!(store.approval_count().expect("approval count"), 1);
    }

    #[test]
    fn pending_approvals_are_not_recreated_after_restart() {
        let mut manager = ApprovalManager::new();

        manager.create_pending(PendingApproval {
            id: "approval-1".into(),
            tool_call_id: "tool-1".into(),
            action_type: "shell".into(),
            target_summary: "npm test".into(),
            risk_level: "medium".into(),
        });
        let restarted_manager = ApprovalManager::new();

        assert_eq!(manager.pending_count(), 1);
        assert_eq!(restarted_manager.pending_count(), 0);
    }

    #[test]
    fn answering_missing_pending_approval_fails_closed() {
        let store = prepared_store_with_tool_call();
        let mut manager = ApprovalManager::new();

        let result = manager.answer(&store, "missing", ApprovalDecision::Deny, "local-user");

        assert!(matches!(result, Err(ApprovalError::MissingPendingApproval)));
        assert_eq!(store.approval_count().expect("approval count"), 0);
    }

    fn prepared_store_with_tool_call() -> AppStore {
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
                status: "running",
                mode: "agent",
                agent_ref: None,
                model_id: "openai/gpt-4.1",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: None,
                tool_source: "opencode",
                tool_name: "file.write",
                arguments_json: "{}",
                status: "pending_approval",
                runtime_call_ref: Some("runtime-tool-1"),
            })
            .expect("tool inserted");

        store
    }
}
