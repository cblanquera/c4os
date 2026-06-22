use crate::menu::{evaluate_menu_state, menu_contract, FocusState, MenuContract, MenuState};
use crate::mock_data::{mock_workspace, BrowserState, TerminalState, WorkspacePayload};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

const FAKE_RUN_MESSAGE: &str = "Mock agent completed the requested transition.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResponse {
    pub prompt: String,
    pub run: String,
    pub agent: String,
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
    mock_workspace()
}

#[tauri::command]
pub fn send_prompt(prompt: String) -> AgentRunResponse {
    fake_successful_run(prompt)
}

#[tauri::command]
pub fn open_workspace(request: Option<WorkspaceOpenRequest>) -> WorkspaceOpenResponse {
    let workspace = mock_workspace();
    WorkspaceOpenResponse {
        path: request
            .and_then(|input| input.path)
            .unwrap_or_else(|| "/mock/workspace".into()),
        trusted: true,
        workspace: workspace.workspace,
    }
}

#[tauri::command]
pub fn create_session(project: Option<String>) -> SessionResponse {
    SessionResponse {
        id: "mock-session-task-003".into(),
        project: project.unwrap_or_else(|| "Mock Workspace Alpha".into()),
        status: "mock-created".into(),
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
    }
}

pub fn fake_failed_run(prompt: String) -> String {
    format!("Mock agent failed before producing output. prompt={prompt}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_inventory_returns_task_002_mock_payloads() {
        assert_eq!(load_workspace().workspace.project, "Mock Workspace Alpha");
        assert_eq!(open_browser_preview().title, "Mock rendered page");
        assert_eq!(list_extensions().mcp_servers, vec!["mock_node_repl", "mock_docs_mcp"]);
    }

    #[test]
    fn fake_run_preserves_task_002_success_and_failure_text() {
        let success = send_prompt("Summarize backend parity".into());

        assert_eq!(success.prompt, "Summarize backend parity");
        assert_eq!(success.run, "Mock agent completed the requested transition.");
        assert_eq!(
            fake_failed_run("fail this run".into()),
            "Mock agent failed before producing output. prompt=fail this run"
        );
    }

    #[test]
    fn mock_file_and_terminal_commands_do_not_claim_real_io() {
        assert_eq!(read_file(None).path, "frontend/mock-main.js");
        assert_eq!(save_file(None).saved, true);
        assert!(run_terminal_command(None).output.contains("fake agent run channel connected"));
    }
}
