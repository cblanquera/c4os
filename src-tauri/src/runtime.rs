use crate::path_resolver::ProjectPathResolver;
use crate::storage::AppStore;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct RuntimeLaunchRequest {
    pub session_id: String,
    pub project_id: String,
    pub project_root: PathBuf,
    pub title: String,
}

#[derive(Debug)]
pub struct SupervisedRuntimeProcess {
    pub session_id: String,
    pub pid: u32,
    child: Child,
}

impl SupervisedRuntimeProcess {
    pub fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
pub enum RuntimeLaunchError {
    InvalidProjectRoot,
    MissingProviderState(String),
    SpawnFailed(String),
    StoreFailed(rusqlite::Error),
}

pub struct OpenCodeRuntimeLauncher {
    command_path: PathBuf,
    command_args: Vec<String>,
}

impl OpenCodeRuntimeLauncher {
    pub fn new(command_path: impl Into<PathBuf>, command_args: Vec<String>) -> Self {
        Self {
            command_path: command_path.into(),
            command_args,
        }
    }

    pub fn launch(
        &self,
        app_store: &AppStore,
        request: RuntimeLaunchRequest,
    ) -> Result<SupervisedRuntimeProcess, RuntimeLaunchError> {
        ProjectPathResolver::new(&request.project_root)
            .map_err(|_| RuntimeLaunchError::InvalidProjectRoot)?;

        app_store
            .start_openrouter_session(&request.session_id, &request.project_id, &request.title)
            .map_err(|error| RuntimeLaunchError::MissingProviderState(error.to_string()))?;

        let session = app_store
            .get_session(&request.session_id)
            .map_err(RuntimeLaunchError::StoreFailed)?
            .ok_or_else(|| RuntimeLaunchError::MissingProviderState("missing session".into()))?;
        let credential_reference = app_store
            .session_credential_reference(&request.session_id)
            .map_err(RuntimeLaunchError::StoreFailed)?
            .ok_or_else(|| {
                RuntimeLaunchError::MissingProviderState(
                    "missing session credential reference".into(),
                )
            })?;

        let mut command = Command::new(&self.command_path);
        command
            .args(&self.command_args)
            .current_dir(&request.project_root)
            .env("C4OS_SESSION_ID", &request.session_id)
            .env("C4OS_PROJECT_ROOT", &request.project_root)
            .env("C4OS_MODEL_ID", &session.model_id)
            .env("C4OS_CREDENTIAL_REF", credential_reference)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = command
            .spawn()
            .map_err(|error| RuntimeLaunchError::SpawnFailed(error.to_string()))?;
        let pid = child.id();

        app_store
            .record_adapter_ref(
                &format!("session-{}-runtime-process", request.session_id),
                Some(&request.session_id),
                "opencode",
                "process_id",
                &pid.to_string(),
                r#"{"supervisedByApp":true}"#,
            )
            .map_err(RuntimeLaunchError::StoreFailed)?;

        Ok(SupervisedRuntimeProcess {
            session_id: request.session_id,
            pid,
            child,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::storage::NewProject;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn launches_supervised_runtime_for_registered_project_and_provider() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
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

        assert!(process.pid > 0);
        assert_eq!(
            store
                .get_session("session-1")
                .expect("session query")
                .expect("session exists")
                .model_id,
            "openai/gpt-4.1"
        );

        process.terminate();
    }

    #[test]
    fn launch_fails_before_spawn_when_project_root_is_invalid() {
        let store = AppStore::open_in_memory().expect("store opens");
        let launcher = OpenCodeRuntimeLauncher::new("/bin/sh", vec![]);

        let result = launcher.launch(
            &store,
            RuntimeLaunchRequest {
                session_id: "session-1".into(),
                project_id: "project-1".into(),
                project_root: PathBuf::from("/definitely/not/a/c4os/project"),
                title: "Run".into(),
            },
        );

        assert!(matches!(
            result,
            Err(RuntimeLaunchError::InvalidProjectRoot)
        ));
    }

    #[test]
    fn launch_failure_is_reported_when_command_is_missing() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("README.md"), "hello").expect("file written");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
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

        let launcher = OpenCodeRuntimeLauncher::new("/missing/c4os-opencode", vec![]);
        let result = launcher.launch(
            &store,
            RuntimeLaunchRequest {
                session_id: "session-1".into(),
                project_id: "project-1".into(),
                project_root: directory.path().to_path_buf(),
                title: "Run".into(),
            },
        );

        assert!(matches!(result, Err(RuntimeLaunchError::SpawnFailed(_))));
    }
}
