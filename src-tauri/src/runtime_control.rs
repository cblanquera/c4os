use crate::runtime::SupervisedRuntimeProcess;
use crate::storage::AppStore;

#[derive(Debug)]
pub enum RuntimeControlError {
    StoreFailed(rusqlite::Error),
}

pub struct RuntimeController;

impl RuntimeController {
    pub fn stop(
        app_store: &AppStore,
        process: &mut SupervisedRuntimeProcess,
    ) -> Result<(), RuntimeControlError> {
        process.terminate();
        app_store
            .update_session_status(&process.session_id, "stopped")
            .map_err(RuntimeControlError::StoreFailed)?;

        Ok(())
    }

    pub fn mark_interrupted_after_restart(
        app_store: &AppStore,
    ) -> Result<usize, RuntimeControlError> {
        app_store
            .mark_active_sessions_interrupted()
            .map_err(RuntimeControlError::StoreFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::runtime::{OpenCodeRuntimeLauncher, RuntimeLaunchRequest};
    use crate::storage::{AppStore, NewMessage, NewProject, NewSession, NewToolCall};
    use tempfile::tempdir;

    #[test]
    fn stop_terminates_supervised_process_and_preserves_records() {
        let directory = tempdir().expect("tempdir");
        let store = prepared_provider_store(directory.path().to_string_lossy().as_ref());
        let launcher = OpenCodeRuntimeLauncher::new("/bin/sh", vec!["-c".into(), "sleep 2".into()]);
        let mut process = launcher
            .launch(
                &store,
                RuntimeLaunchRequest {
                    session_id: "session-1".into(),
                    project_id: "project-1".into(),
                    project_root: directory.path().to_path_buf(),
                    title: "Run".into(),
                },
            )
            .expect("runtime launched");

        store
            .append_message(NewMessage {
                id: "message-1",
                session_id: "session-1",
                role: "assistant",
                content: "partial",
                status: "stopped",
                metadata_json: "{}",
            })
            .expect("message inserted");
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: None,
                tool_source: "opencode",
                tool_name: "bash",
                arguments_json: "{}",
                status: "running",
                runtime_call_ref: Some("runtime-tool-1"),
            })
            .expect("tool inserted");

        RuntimeController::stop(&store, &mut process).expect("stopped");

        let session = store
            .get_session("session-1")
            .expect("session query")
            .expect("session exists");
        let messages = store.list_messages("session-1").expect("messages");

        assert_eq!(session.status, "stopped");
        assert_eq!(messages.len(), 1);
        assert_eq!(store.tool_call_count("session-1").expect("tool count"), 1);
    }

    #[test]
    fn restart_recovery_marks_active_sessions_interrupted() {
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

        let changed =
            RuntimeController::mark_interrupted_after_restart(&store).expect("interrupted");
        let session = store
            .get_session("session-1")
            .expect("session query")
            .expect("session exists");

        assert_eq!(changed, 1);
        assert_eq!(session.status, "interrupted");
    }

    fn prepared_provider_store(root_path: &str) -> AppStore {
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path,
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");

        store
    }
}
