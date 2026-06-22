use crate::menu::{evaluate_menu_state, menu_contract, FocusState, MenuContract, MenuState};
use crate::mock_data::{mock_workspace, BrowserState, TerminalState, WorkspacePayload};
use crate::openrouter::{run_chat_stream_with_model, RuntimeEvent};
use crate::provider_models::{
    delete_provider_profile as delete_provider_profile_record, provider_api_key_configured,
    provider_models, provider_profiles, save_provider_profile as save_provider_profile_record,
    select_session_model as select_provider_session_model,
    set_model_enabled as set_provider_model_enabled,
    set_provider_enabled as set_provider_profile_enabled, ModelEnablementRequest,
    ProviderDeleteRequest, ProviderDeleteResponse, ProviderEnablementRequest, ProviderModel,
    ProviderProfile, ProviderProfileSaveRequest, SessionModelSelection,
    SessionModelSelectionRequest, DEFAULT_MODEL,
};
use crate::workspace::{activate_workspace, WorkspaceActivation, WorkspaceDescriptor};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};

const FAKE_RUN_MESSAGE: &str = "Mock agent completed the requested transition.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResponse {
    pub prompt: String,
    pub run: String,
    pub agent: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub id: String,
    pub project: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenRequest {
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceOpenResponse {
    pub path: String,
    pub trusted: bool,
    pub workspace: crate::mock_data::WorkspaceSummary,
    pub descriptor: WorkspaceDescriptor,
    pub payload: WorkspacePayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRequest {
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileReadResponse {
    pub path: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSaveResponse {
    pub path: String,
    pub saved: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalCommandRequest {
    pub command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalCommandResponse {
    pub command: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionList {
    pub plugins: Vec<String>,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
}

#[tauri::command]
pub fn load_workspace() -> WorkspacePayload {
    let mut payload = mock_workspace();
    payload.providers = provider_profiles()
        .into_iter()
        .map(provider_record_from_profile)
        .collect();
    payload.models = provider_models()
        .into_iter()
        .map(model_record_from_provider_model)
        .collect();
    payload
}

#[tauri::command]
pub fn send_prompt<R: Runtime>(
    app: AppHandle<R>,
    prompt: String,
    model: Option<String>,
) -> AgentRunResponse {
    if !provider_api_key_configured() {
        return fake_successful_run(prompt);
    }

    let selected_model = model.unwrap_or_else(|| DEFAULT_MODEL.into());
    run_chat_stream_with_model(&prompt, &selected_model, |event| {
        let _ = app.emit("c4os://runtime-event", event);
    })
    .map(|result| AgentRunResponse {
        prompt: result.prompt,
        run: result.run,
        agent: result.agent,
        events: result.events,
    })
    .unwrap_or_else(|error| AgentRunResponse {
        prompt,
        run: error.clone(),
        agent: "The OpenRouter request did not complete.".into(),
        events: vec![RuntimeEvent {
            kind: "error".into(),
            text: error,
            sequence: 1,
        }],
    })
}

#[tauri::command]
pub fn open_workspace(
    request: Option<WorkspaceOpenRequest>,
) -> Result<WorkspaceOpenResponse, String> {
    let activation = activate_workspace(request.and_then(|input| input.path))?;
    Ok(open_workspace_response(activation))
}

#[tauri::command]
pub fn create_session(project: Option<String>, model: Option<String>) -> SessionResponse {
    SessionResponse {
        id: "mock-session-task-003".into(),
        project: project.unwrap_or_else(|| "Mock Workspace Alpha".into()),
        status: model
            .map(|selected| format!("created-with-model:{selected}"))
            .unwrap_or_else(|| "mock-created".into()),
    }
}

#[tauri::command]
pub fn read_file(request: Option<FileRequest>) -> FileReadResponse {
    let workspace = mock_workspace();
    FileReadResponse {
        path: request
            .and_then(|input| input.path)
            .unwrap_or_else(|| "frontend/mock-main.js".into()),
        lines: workspace.files.lines,
    }
}

#[tauri::command]
pub fn save_file(request: Option<FileRequest>) -> FileSaveResponse {
    FileSaveResponse {
        path: request
            .and_then(|input| input.path)
            .unwrap_or_else(|| "frontend/mock-main.js".into()),
        saved: true,
    }
}

#[tauri::command]
pub fn run_terminal_command(request: Option<TerminalCommandRequest>) -> TerminalCommandResponse {
    let terminal: TerminalState = mock_workspace().terminal;
    TerminalCommandResponse {
        command: request
            .and_then(|input| input.command)
            .unwrap_or_else(|| "npm run mock:task-003".into()),
        output: terminal.output,
    }
}

#[tauri::command]
pub fn open_browser_preview() -> BrowserState {
    mock_workspace().browser
}

#[tauri::command]
pub fn list_extensions() -> ExtensionList {
    let workspace = mock_workspace();
    ExtensionList {
        plugins: workspace.plugin_catalog,
        skills: workspace.skill_catalog,
        mcp_servers: workspace.mcp_servers,
    }
}

#[tauri::command]
pub fn list_provider_profiles() -> Vec<ProviderProfile> {
    provider_profiles()
}

#[tauri::command]
pub fn save_provider_profile(
    request: ProviderProfileSaveRequest,
) -> Result<ProviderProfile, String> {
    save_provider_profile_record(request)
}

#[tauri::command]
pub fn delete_provider_profile(
    request: ProviderDeleteRequest,
) -> Result<ProviderDeleteResponse, String> {
    delete_provider_profile_record(request)
}

#[tauri::command]
pub fn list_provider_models() -> Vec<ProviderModel> {
    provider_models()
}

#[tauri::command]
pub fn set_model_enabled(request: ModelEnablementRequest) -> Result<ProviderModel, String> {
    set_provider_model_enabled(request)
}

#[tauri::command]
pub fn set_provider_enabled(request: ProviderEnablementRequest) -> Result<ProviderProfile, String> {
    set_provider_profile_enabled(request)
}

#[tauri::command]
pub fn select_session_model(
    request: SessionModelSelectionRequest,
) -> Result<SessionModelSelection, String> {
    select_provider_session_model(request)
}

#[tauri::command]
pub fn native_menu_contract() -> MenuContract {
    menu_contract()
}

#[tauri::command]
pub fn native_menu_state<R: Runtime>(app: AppHandle<R>, focus_state: FocusState) -> MenuState {
    crate::menu::apply_native_menu_state(&app, &focus_state)
        .unwrap_or_else(|_| evaluate_menu_state(&focus_state))
}

pub fn fake_successful_run(prompt: String) -> AgentRunResponse {
    AgentRunResponse {
        prompt,
        run: FAKE_RUN_MESSAGE.into(),
        agent: FAKE_RUN_MESSAGE.into(),
        events: Vec::new(),
    }
}

pub fn fake_failed_run(prompt: String) -> String {
    format!("Mock agent failed before producing output. prompt={prompt}")
}

fn open_workspace_response(activation: WorkspaceActivation) -> WorkspaceOpenResponse {
    let mut payload = activation.payload;
    payload.providers = provider_profiles()
        .into_iter()
        .map(provider_record_from_profile)
        .collect();
    payload.models = provider_models()
        .into_iter()
        .map(model_record_from_provider_model)
        .collect();

    WorkspaceOpenResponse {
        path: activation.path,
        trusted: activation.trusted,
        workspace: payload.workspace.clone(),
        descriptor: activation.descriptor,
        payload,
    }
}

fn provider_record_from_profile(profile: ProviderProfile) -> crate::mock_data::ProviderRecord {
    crate::mock_data::ProviderRecord {
        id: profile.id,
        label: profile.label,
        kind: profile.kind,
        base_url: profile.base_url,
        endpoint: profile.endpoint,
        status: profile.status,
        key_status: crate::mock_data::ApiKeyStatus {
            state: profile.key_status.state,
            source: profile.key_status.source,
        },
        enabled: profile.enabled,
        supports_discovery: profile.supports_discovery,
    }
}

fn model_record_from_provider_model(model: ProviderModel) -> crate::mock_data::ModelRecord {
    crate::mock_data::ModelRecord {
        id: model.id,
        label: model.label,
        provider_id: model.provider_id,
        provider: model.provider,
        enabled: model.enabled,
        active: model.active,
        source: model.source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_inventory_returns_task_002_mock_payloads() {
        assert_eq!(load_workspace().workspace.project, "Mock Workspace Alpha");
        assert_eq!(open_browser_preview().title, "Mock rendered page");
        assert_eq!(
            list_extensions().mcp_servers,
            vec!["mock_node_repl", "mock_docs_mcp"]
        );
    }

    #[test]
    fn fake_run_preserves_task_002_success_and_failure_text() {
        let success = fake_successful_run("Summarize backend parity".into());

        assert_eq!(success.prompt, "Summarize backend parity");
        assert_eq!(
            success.run,
            "Mock agent completed the requested transition."
        );
        assert_eq!(
            fake_failed_run("fail this run".into()),
            "Mock agent failed before producing output. prompt=fail this run"
        );
    }

    #[test]
    fn mock_file_and_terminal_commands_do_not_claim_real_io() {
        assert_eq!(read_file(None).path, "frontend/mock-main.js");
        assert_eq!(save_file(None).saved, true);
        assert!(run_terminal_command(None)
            .output
            .contains("fake agent run channel connected"));
    }

    #[test]
    fn task_004_open_workspace_returns_real_descriptor_backed_payload() {
        let root = std::env::temp_dir().join("c4os-task-004-command");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create workspace root");

        let response = open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");

        assert_eq!(response.trusted, true);
        assert_eq!(response.workspace.project, "c4os-task-004-command");
        assert_eq!(response.payload.workspace.project, "c4os-task-004-command");
        assert_eq!(response.descriptor.name, "c4os-task-004-command");
        assert_ne!(response.workspace.project, "Mock Workspace Alpha");

        let _ = std::fs::remove_dir_all(root);
    }
}
