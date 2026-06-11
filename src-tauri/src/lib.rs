use crate::credentials::{CredentialStore, MacOsKeychainCredentialStore};
use crate::project::{ProjectError, ProjectService};
use crate::project_selector::{ProjectSelectionError, ProjectSelector};
use crate::provider::{
    MetadataStatus, OpenRouterValidator, ProviderSetupError, ProviderSetupService,
    ProviderValidation,
};
use crate::runtime::{
    CodingRuntimeRunner, OpenCodeJsonRunner, RuntimePromptError, RuntimePromptRequest,
};
use crate::session_view::{RuntimeState, SessionView};
use crate::storage::{AppStore, NewMessage};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

pub mod action_classifier;
pub mod approval;
pub mod approval_prompt;
pub mod artifact;
pub mod credentials;
pub mod event_normalizer;
pub mod file_browser;
pub mod file_tools;
pub mod instruction_preflight;
pub mod path_resolver;
pub mod project;
pub mod project_selector;
pub mod provider;
pub mod recovery_view;
pub mod runtime;
pub mod runtime_control;
pub mod session_allow;
pub mod session_catalog;
pub mod session_view;
pub mod shell_executor;
pub mod shell_summary;
pub mod storage;
pub mod tool_timeline;

struct C4OsState {
    db_path: PathBuf,
    store: Mutex<AppStore>,
}

struct BasicOpenRouterValidator;

#[derive(Debug)]
enum SubmitPromptError {
    Message(String),
    Runtime(RuntimePromptError),
    Store(rusqlite::Error),
}

impl OpenRouterValidator for BasicOpenRouterValidator {
    fn validate_key(&self, key: &str) -> ProviderValidation {
        let key = key.trim();

        if key.is_empty() {
            return ProviderValidation::Invalid("OpenRouter key is required.".into());
        }

        if !key.starts_with("sk-or-") {
            return ProviderValidation::Invalid(
                "OpenRouter key should start with sk-or-.".into(),
            );
        }

        ProviderValidation::Valid
    }
}

#[tauri::command]
fn get_app_status(state: tauri::State<'_, C4OsState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;

    app_status(&store).map_err(|error| error.to_string())
}

#[tauri::command]
fn configure_openrouter(
    state: tauri::State<'_, C4OsState>,
    api_key: String,
    selected_model: Option<String>,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let validator = BasicOpenRouterValidator;
    let service = ProviderSetupService::new(&store, &credential_store, &validator);
    let selected_model = selected_model.as_deref();
    let provider = service
        .configure_openrouter(&api_key, selected_model)
        .map_err(provider_setup_error)?;

    Ok(json!({
        "id": provider.provider,
        "ready": provider.ready,
        "selectedModel": provider.selected_model,
        "metadataStatus": metadata_status(&provider.metadata_status),
        "disclosure": provider.disclosure,
    }))
}

#[tauri::command]
fn register_project(
    state: tauri::State<'_, C4OsState>,
    root_path: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let service = ProjectService::new(&store);
    let project = service
        .register_git_project(root_path)
        .map_err(project_error)?;
    let selector = ProjectSelector::select(&store, &project.id).map_err(selection_error)?;

    Ok(json!({
        "active": true,
        "id": project.id,
        "name": project.name,
        "rootPath": project.root_path,
        "branch": project.git_status.branch,
        "dirty": project.git_status.dirty,
        "changedFileCount": project.git_status.changed_files.len(),
        "hasRootAgents": project.has_root_agents,
        "registeredProjectCount": selector.projects.len(),
    }))
}

#[tauri::command]
fn submit_prompt(
    state: tauri::State<'_, C4OsState>,
    prompt: String,
) -> Result<Value, String> {
    let store = state.store.lock().map_err(|_| "store lock failed")?;
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let run = prepare_prompt_run(&store, &credential_store, &prompt)
        .map_err(|error| error.to_string())?;
    let session = session_state(&store).map_err(|error| error.to_string())?;
    let db_path = state.db_path.clone();

    std::thread::spawn(move || {
        let Ok(store) = AppStore::open(db_path) else {
            return;
        };
        let runner = OpenCodeJsonRunner::default_from_workspace();

        finalize_prompt_run(&store, &runner, run);
    });

    Ok(session)
}

#[cfg(test)]
fn submit_prompt_with_runner(
    store: &AppStore,
    credential_store: &impl CredentialStore,
    runner: &impl CodingRuntimeRunner,
    prompt: &str,
) -> Result<Value, SubmitPromptError> {
    let run = prepare_prompt_run(store, credential_store, prompt)?;

    finalize_prompt_run(store, runner, run);

    Ok(session_state(store)?)
}

fn prepare_prompt_run(
    store: &AppStore,
    credential_store: &impl CredentialStore,
    prompt: &str,
) -> Result<PromptRun, SubmitPromptError> {
    let provider = provider_state(store)?;
    let selector = ProjectSelector::list(store)?;
    let prompt = prompt.trim();

    if !provider.get("ready").and_then(Value::as_bool).unwrap_or(false) {
        return Err("Configure OpenRouter before starting a session.".into());
    }

    let Some(active_project) = selector.active_project else {
        return Err("Register and select a Git project before starting a session.".into());
    };

    if prompt.is_empty() {
        return Err("Enter a prompt before starting a session.".into());
    }

    let start_guard = SessionView::start_guard(store)?;

    if !start_guard.can_start {
        return Err(SubmitPromptError::Message(start_guard
            .reason
            .unwrap_or_else(|| "A session is already running.".into())));
    }

    let credential_reference = store
        .latest_openrouter_credential_reference()?
        .ok_or_else(|| SubmitPromptError::Message("Missing OpenRouter credential reference.".into()))?;
    let openrouter_api_key = credential_store
        .read_openrouter_key(&credential_reference)
        .map_err(|error| SubmitPromptError::Message(format!("{error:?}")))?;
    let selected_model = provider
        .get("selectedModel")
        .and_then(Value::as_str)
        .ok_or_else(|| SubmitPromptError::Message("Select a model before starting a session.".into()))?
        .to_string();
    let session_id = format!("session-{}", timestamp());
    let message_id = format!("message-{}", timestamp());
    let title = first_line(prompt);

    store.start_openrouter_session(&session_id, &active_project.id, &title)?;
    store
        .append_message(NewMessage {
            id: &message_id,
            session_id: &session_id,
            role: "user",
            content: prompt,
            status: "submitted",
            metadata_json: "{}",
        })?;

    Ok(PromptRun {
        session_id,
        message_id,
        project_root: active_project.root_path.into(),
        prompt: prompt.into(),
        model_id: selected_model,
        openrouter_api_key,
    })
}

fn finalize_prompt_run(
    store: &AppStore,
    runner: &impl CodingRuntimeRunner,
    run: PromptRun,
) {
    match runner.run_prompt(
        store,
        RuntimePromptRequest {
            session_id: run.session_id.clone(),
            project_root: run.project_root,
            prompt: run.prompt,
            model_id: run.model_id,
            openrouter_api_key: run.openrouter_api_key,
        },
    ) {
        Ok(result) => {
            let _ = store.append_message(NewMessage {
                id: &format!("{}-assistant", run.message_id),
                session_id: &run.session_id,
                role: "assistant",
                content: &result.assistant_text,
                status: "complete",
                metadata_json: "{}",
            });
            let _ = store.update_session_status(&run.session_id, "complete");
        }
        Err(error) => {
            let _ = store.append_message(NewMessage {
                id: &format!("{}-assistant", run.message_id),
                session_id: &run.session_id,
                role: "assistant",
                content: &runtime_prompt_error_message(&error),
                status: "failed",
                metadata_json: "{}",
            });
            let _ = store.update_session_status(&run.session_id, "failed");
        }
    }
}

struct PromptRun {
    session_id: String,
    message_id: String,
    project_root: PathBuf,
    prompt: String,
    model_id: String,
    openrouter_api_key: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;

            fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("c4os.sqlite");
            let store = AppStore::open(&db_path)?;

            store.mark_active_sessions_interrupted()?;
            app.manage(C4OsState {
                db_path,
                store: Mutex::new(store),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_openrouter,
            register_project,
            submit_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while running C4OS");
}

fn app_status(store: &AppStore) -> rusqlite::Result<Value> {
    let selector = ProjectSelector::list(store)?;
    let active_project = selector.active_project.clone();
    let session = session_state(store)?;

    Ok(json!({
        "appName": "C4OS",
        "backendReady": true,
        "telemetryEnabled": false,
        "diagnostics": {
            "storageMode": "local-only",
            "productTelemetry": "disabled",
            "automaticCrashUpload": false,
            "supportBundleUpload": false,
        },
        "provider": provider_state(store)?,
        "project": {
            "active": active_project.is_some(),
            "rootPath": active_project.as_ref().map(|project| project.root_path.clone()),
            "branch": null,
            "dirty": false,
            "changedFileCount": 0,
            "browserMode": "read-only",
            "rootAgentsDisplay": "passive",
            "instructionResolution": {
                "nestedAgentsSupported": true,
                "supportTier": "display_guidance_order_only",
                "permissionEffect": "none",
                "modelContextEffect": "none_without_explicit_runtime_read",
                "editReloadBehavior": "next_preflight_only",
                "conflictDiagnostics": "ordered_sources_no_semantic_merge",
            },
            "selector": {
                "available": true,
                "listRegisteredProjects": true,
                "selectExactlyOneActive": true,
                "registeredProjectCount": selector.projects.len(),
                "multipleActiveProjectsAllowed": false,
                "searchAvailable": false,
                "groupingAvailable": false,
                "archiveAvailable": false,
                "deleteAvailable": false,
                "favoritesAvailable": false,
                "metadataEditingAvailable": false,
                "crossProjectViewsAvailable": false,
                "nonGitProjectsAllowed": false,
                "worktreeManagementAvailable": false,
            },
        },
        "timeline": {
            "toolActivityVisible": true,
            "rawOutputExportAvailable": false,
            "liveDrawerLabel": "live/ephemeral",
        },
        "session": session,
        "approvals": {
            "pendingCount": 0,
            "choices": ["allow_once", "allow_session", "deny"],
            "highRiskSessionAllow": false,
            "policyBlocksExecute": false,
        },
        "recovery": {
            "canStop": session.get("active").and_then(Value::as_bool).unwrap_or(false),
            "canRetry": false,
            "retryAppendsNewAction": true,
            "autoContinueAfterRestart": false,
            "reattachAfterRestart": false,
            "resendPendingPromptsAfterRestart": false,
        },
        "changes": {
            "changedFileCount": 0,
            "diffViewerAvailable": true,
            "gitInspectionLogged": true,
            "approvalRequiredForInspection": false,
            "refreshAfterToolExecution": true,
        },
        "artifacts": {
            "visibleRecords": true,
            "searchAvailable": false,
            "activeHtmlPreview": false,
            "richPreview": false,
            "exportAvailable": false,
            "includeInModelContextByDefault": false,
        },
    }))
}

fn provider_state(store: &AppStore) -> rusqlite::Result<Value> {
    let credential_store = MacOsKeychainCredentialStore::new("default");
    let validator = BasicOpenRouterValidator;
    let service = ProviderSetupService::new(store, &credential_store, &validator);
    let provider = service.current_state()?;

    Ok(json!({
        "id": provider.provider,
        "ready": provider.ready,
        "selectedModel": provider.selected_model,
        "metadataStatus": metadata_status(&provider.metadata_status),
        "disclosure": provider.disclosure,
    }))
}

fn session_state(store: &AppStore) -> rusqlite::Result<Value> {
    let selector = ProjectSelector::list(store)?;
    let screen = match selector.active_project {
        Some(project) => SessionView::latest_for_project(store, &project.id)?,
        None => None,
    };
    let start_guard = SessionView::start_guard(store)?;

    Ok(match screen {
        Some(screen) => json!({
            "active": matches!(
                screen.runtime_state,
                RuntimeState::Running | RuntimeState::WaitingForApproval
            ),
            "id": screen.session_id,
            "title": screen.title,
            "runtimeState": runtime_state(&screen.runtime_state),
            "runtimeStates": [
                "running",
                "waiting_for_approval",
                "stopped",
                "failed",
                "complete",
            ],
            "transcriptAppendOnly": true,
            "canStartNewRun": start_guard.can_start,
            "failureDisplay": screen.failure_display,
            "messages": screen.messages.into_iter().map(|message| {
                json!({
                    "id": message.id,
                    "role": message.role,
                    "content": message.content,
                    "status": message.status,
                })
            }).collect::<Vec<_>>(),
        }),
        None => json!({
            "active": false,
            "id": null,
            "title": null,
            "runtimeState": "stopped",
            "runtimeStates": [
                "running",
                "waiting_for_approval",
                "stopped",
                "failed",
                "complete",
            ],
            "transcriptAppendOnly": true,
            "canStartNewRun": start_guard.can_start,
            "failureDisplay": null,
            "messages": [],
        }),
    })
}

fn metadata_status(status: &MetadataStatus) -> &'static str {
    match status {
        MetadataStatus::Current => "current",
        MetadataStatus::Unknown => "unknown",
        MetadataStatus::Stale => "stale",
    }
}

fn runtime_state(status: &RuntimeState) -> &'static str {
    match status {
        RuntimeState::Running => "running",
        RuntimeState::WaitingForApproval => "waiting_for_approval",
        RuntimeState::Stopped => "stopped",
        RuntimeState::Failed => "failed",
        RuntimeState::Complete => "complete",
    }
}

fn provider_setup_error(error: ProviderSetupError) -> String {
    match error {
        ProviderSetupError::ActiveSession(message) => message,
        ProviderSetupError::ValidationFailed(message) => message,
        ProviderSetupError::CredentialStorage(error) => format!("{error:?}"),
    }
}

fn runtime_prompt_error_message(error: &RuntimePromptError) -> String {
    match error {
        RuntimePromptError::InvalidProjectRoot => {
            "OpenCode runtime failed because the selected project root is invalid.".into()
        }
        RuntimePromptError::MissingAssistantOutput => {
            "OpenCode completed without assistant output.".into()
        }
        RuntimePromptError::SpawnFailed(message) => {
            format!("OpenCode runtime could not start: {message}")
        }
        RuntimePromptError::ExecutionFailed(message) => {
            format!("OpenCode runtime failed: {message}")
        }
        RuntimePromptError::StoreFailed(message) => {
            format!("OpenCode runtime event storage failed: {message}")
        }
    }
}

fn project_error(error: ProjectError) -> String {
    match error {
        ProjectError::UnreadablePath(message) => message,
        ProjectError::NotGitRepository(message) => message,
        ProjectError::GitFailed(message) => message,
        ProjectError::StoreFailed(error) => error.to_string(),
    }
}

fn selection_error(error: ProjectSelectionError) -> String {
    match error {
        ProjectSelectionError::MissingProject => "Registered project was not found.".into(),
        ProjectSelectionError::StoreFailed => "Project selection failed.".into(),
    }
}

fn first_line(value: &str) -> String {
    value.lines().next().unwrap_or("Session").chars().take(80).collect()
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

impl From<&str> for SubmitPromptError {
    fn from(value: &str) -> Self {
        Self::Message(value.into())
    }
}

impl From<String> for SubmitPromptError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<rusqlite::Error> for SubmitPromptError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Store(value)
    }
}

impl From<RuntimePromptError> for SubmitPromptError {
    fn from(value: RuntimePromptError) -> Self {
        Self::Runtime(value)
    }
}

impl std::fmt::Display for SubmitPromptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => write!(formatter, "{message}"),
            Self::Runtime(error) => write!(formatter, "{}", runtime_prompt_error_message(error)),
            Self::Store(error) => write!(formatter, "{error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use crate::runtime::{RuntimePromptResult, RuntimePromptRequest};
    use crate::storage::NewProject;
    use tempfile::tempdir;

    struct FakeRunner {
        assistant_text: String,
    }

    impl CodingRuntimeRunner for FakeRunner {
        fn run_prompt(
            &self,
            _app_store: &AppStore,
            request: RuntimePromptRequest,
        ) -> Result<RuntimePromptResult, RuntimePromptError> {
            assert_eq!(request.prompt, "Say hello");
            assert_eq!(request.model_id, "openai/gpt-4.1");
            assert_eq!(request.openrouter_api_key, "sk-or-test-secret");

            Ok(RuntimePromptResult {
                assistant_text: self.assistant_text.clone(),
            })
        }
    }

    #[test]
    fn submit_prompt_persists_runtime_assistant_output() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();
        let runner = FakeRunner {
            assistant_text: "Hello from OpenCode.".into(),
        };

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
            .set_selected_project("project-1")
            .expect("project selected");
        store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");

        let session = submit_prompt_with_runner(
            &store,
            &credential_store,
            &runner,
            "Say hello",
        )
        .expect("prompt submitted");
        let messages = session
            .get("messages")
            .and_then(Value::as_array)
            .expect("messages");

        assert_eq!(session.get("runtimeState").and_then(Value::as_str), Some("complete"));
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[1].get("content").and_then(Value::as_str),
            Some("Hello from OpenCode.")
        );
    }
}
