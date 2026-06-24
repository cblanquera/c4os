use crate::files::{list_files_state, read_file_state, save_file_state};
use crate::menu::{evaluate_menu_state, menu_contract, FocusState, MenuContract, MenuState};
use crate::mock_data::{
    mock_workspace, ArtifactRecord, BrowserState, TerminalState, WorkspacePayload,
};
use crate::openrouter::RuntimeEvent;
use crate::provider_models::{
    delete_provider_profile as delete_provider_profile_record, provider_models, provider_profiles,
    save_provider_profile as save_provider_profile_record,
    select_session_model as select_provider_session_model,
    set_model_enabled as set_provider_model_enabled,
    set_provider_enabled as set_provider_profile_enabled, ModelEnablementRequest,
    ProviderDeleteRequest, ProviderDeleteResponse, ProviderEnablementRequest, ProviderModel,
    ProviderProfile, ProviderProfileSaveRequest, SessionModelSelection,
    SessionModelSelectionRequest, DEFAULT_MODEL,
};
use crate::runtime_sessions::{
    active_session, append_prompt, create_session as create_c4os_session,
    create_session_artifact_preview, load_session as load_c4os_session, project_sessions,
    sessions as c4os_sessions, set_session_browser as set_c4os_session_browser,
    set_session_files as set_c4os_session_files, set_session_model as set_c4os_session_model,
    BrowserActionRecord, C4osSessionRecord, CreateSessionRequest, SendPromptRequest,
};
use crate::workspace::{
    activate_workspace, active_workspace_root, WorkspaceActivation, WorkspaceDescriptor,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Runtime};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResponse {
    pub prompt: String,
    pub run: String,
    pub agent: String,
    pub model: String,
    pub session: C4osSessionRecord,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLoadRequest {
    pub session_id: String,
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
#[serde(rename_all = "camelCase")]
pub struct FileRequest {
    pub path: Option<String>,
    pub content: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReadResponse {
    pub path: String,
    pub content: String,
    pub saved_content: String,
    pub lines: Vec<String>,
    pub roots: Vec<Vec<String>>,
    pub breadcrumbs: Vec<String>,
    pub dirty: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSaveResponse {
    pub path: String,
    pub saved: bool,
    pub content: String,
    pub saved_content: String,
    pub lines: Vec<String>,
    pub roots: Vec<Vec<String>>,
    pub breadcrumbs: Vec<String>,
    pub dirty: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactPreviewRequest {
    pub session_id: Option<String>,
    pub title: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactPreviewResponse {
    pub artifact: ArtifactRecord,
    pub browser: BrowserState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserOpenRequest {
    pub session_id: Option<String>,
    pub target: String,
    pub actor: Option<String>,
    pub clear_request: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserOpenResponse {
    pub browser: BrowserState,
    pub action: BrowserActionRecord,
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
    apply_runtime_session_payload(&mut payload);
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
    session_id: Option<String>,
    project: Option<String>,
    model: Option<String>,
) -> AgentRunResponse {
    let request = SendPromptRequest {
        session_id,
        project,
        prompt: prompt.clone(),
        model,
    };
    append_prompt(request, |event| {
        let _ = app.emit("c4os://runtime-event", event);
    })
    .map(|result| AgentRunResponse {
        prompt: result.prompt,
        run: result.run,
        agent: result.agent,
        model: result.model,
        session: result.session,
        events: result.events,
    })
    .unwrap_or_else(|error| AgentRunResponse {
        prompt,
        run: error.clone(),
        agent: "The OpenRouter request did not complete.".into(),
        model: DEFAULT_MODEL.into(),
        session: active_session().unwrap_or_else(|| {
            create_c4os_session(CreateSessionRequest {
                project: None,
                label: Some("Failed runtime request".into()),
                model: Some(DEFAULT_MODEL.into()),
            })
        }),
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
pub fn create_session(
    project: Option<String>,
    label: Option<String>,
    model: Option<String>,
) -> C4osSessionRecord {
    create_c4os_session(CreateSessionRequest {
        project,
        label,
        model,
    })
}

#[tauri::command]
pub fn load_session(request: SessionLoadRequest) -> Result<C4osSessionRecord, String> {
    load_c4os_session(&request.session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{}'", request.session_id))
}

#[tauri::command]
pub fn read_file(request: Option<FileRequest>) -> Result<FileReadResponse, String> {
    let request = request.unwrap_or(FileRequest {
        path: None,
        content: None,
        session_id: None,
    });
    let root =
        active_workspace_root().ok_or_else(|| "No trusted workspace root is active".to_string())?;
    let path = request.path.unwrap_or_default();
    let state = if path.trim().is_empty() {
        list_files_state(&root, None)?
    } else if crate::files::trusted_path(&root, &path)?.is_dir() {
        list_files_state(&root, Some(&path))?
    } else {
        read_file_state(&root, &path)?
    };
    if let Some(session_id) = active_file_session_id(request.session_id) {
        let _ = set_c4os_session_files(&session_id, state.clone());
    }
    Ok(FileReadResponse {
        path: state.current_path,
        content: state.content,
        saved_content: state.saved_content,
        lines: state.lines,
        roots: state.roots,
        breadcrumbs: state.breadcrumbs,
        dirty: state.dirty,
        status: state.status,
    })
}

#[tauri::command]
pub fn save_file(request: Option<FileRequest>) -> Result<FileSaveResponse, String> {
    let request = request.ok_or_else(|| "File save request is required".to_string())?;
    let root =
        active_workspace_root().ok_or_else(|| "No trusted workspace root is active".to_string())?;
    let path = request
        .path
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "File path is required".to_string())?;
    let content = request.content.unwrap_or_default();
    let state = save_file_state(&root, &path, &content)?;
    if let Some(session_id) = active_file_session_id(request.session_id) {
        let _ = set_c4os_session_files(&session_id, state.clone());
    }
    Ok(FileSaveResponse {
        path: state.current_path,
        saved: true,
        content: state.content,
        saved_content: state.saved_content,
        lines: state.lines,
        roots: state.roots,
        breadcrumbs: state.breadcrumbs,
        dirty: state.dirty,
        status: state.status,
    })
}

#[tauri::command]
pub fn create_artifact_preview(
    request: ArtifactPreviewRequest,
) -> Result<ArtifactPreviewResponse, String> {
    let session_id = request
        .session_id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| active_session().map(|session| session.id))
        .ok_or_else(|| "A C4OS session is required before previewing an artifact".to_string())?;
    let (artifact, browser) =
        create_session_artifact_preview(&session_id, &request.title, &request.html)?;
    Ok(ArtifactPreviewResponse { artifact, browser })
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
pub fn open_browser(request: BrowserOpenRequest) -> Result<BrowserOpenResponse, String> {
    let session = request
        .session_id
        .as_deref()
        .and_then(load_c4os_session)
        .or_else(active_session)
        .ok_or_else(|| "A C4OS session is required before opening Browser state".to_string())?;
    let actor = normalized_browser_actor(request.actor.as_deref());
    if actor == "agent" && !request.clear_request {
        return Err("Agent Browser access must be clearly requested for this request".into());
    }
    let target = request.target.trim();
    if target.is_empty() {
        return Err("Browser target is required".into());
    }
    let (browser, action) = browser_state_for_target(&session, target, &actor)?;
    let action_record =
        set_c4os_session_browser(&session.id, browser.clone(), &actor, action, target)?;
    Ok(BrowserOpenResponse {
        browser,
        action: action_record,
    })
}

#[tauri::command]
pub fn open_browser_preview(session_id: Option<String>) -> BrowserState {
    session_id
        .as_deref()
        .and_then(load_c4os_session)
        .map(|session| session.browser)
        .unwrap_or_else(|| mock_workspace().browser)
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
    let selection = select_provider_session_model(request)?;
    let _ = set_c4os_session_model(&selection.session, &selection.model);
    Ok(selection)
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

pub fn fake_failed_run(prompt: String) -> String {
    format!("Mock agent failed before producing output. prompt={prompt}")
}

fn open_workspace_response(activation: WorkspaceActivation) -> WorkspaceOpenResponse {
    let mut payload = activation.payload;
    apply_runtime_session_payload(&mut payload);
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

fn apply_runtime_session_payload(payload: &mut WorkspacePayload) {
    let sessions = c4os_sessions();
    payload.sessions = sessions.clone();
    for project in &mut payload.projects {
        project.sessions = project_sessions(&project.name);
    }
    if let Some(active) = sessions
        .iter()
        .find(|session| session.project == payload.workspace.project)
    {
        payload.workspace.project = active.project.clone();
        payload.workspace.session = active.title.clone();
        payload.workspace.session_id = active.id.clone();
        payload.workspace.model = active.selected_model.clone();
        payload.browser = active.browser.clone();
        payload.files = active.files.clone();
        payload.terminal = active.terminal.clone();
        payload.thread = active.thread.clone();
    } else if let Some(root) = active_workspace_root() {
        if let Ok(files) = list_files_state(&root, None) {
            payload.files = files;
        }
    }
}

fn active_file_session_id(request_session_id: Option<String>) -> Option<String> {
    request_session_id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| active_session().map(|session| session.id))
}

fn normalized_browser_actor(actor: Option<&str>) -> String {
    match actor.unwrap_or("user").trim().to_ascii_lowercase().as_str() {
        "agent" => "agent".into(),
        _ => "user".into(),
    }
}

fn browser_state_for_target(
    session: &C4osSessionRecord,
    target: &str,
    actor: &str,
) -> Result<(BrowserState, &'static str), String> {
    if let Some(public_url) = normalize_public_browser_url(target) {
        return Ok((
            BrowserState {
                url: public_url.clone(),
                title: browser_title_from_url(&public_url),
                summary: "Public page opened in the project-scoped Browser surface.".into(),
                artifact_id: String::new(),
                preview_mode: "public".into(),
                profile_id: crate::runtime_sessions::browser_profile_id(session),
                local_path: String::new(),
                html: String::new(),
            },
            "open-public-url",
        ));
    }

    let file = resolve_browser_file_target(target, actor)?;
    if !file.is_file() {
        return Err("Browser local target must be a trusted project file".into());
    }
    let html = local_browser_dom_preview_content(&file)?;
    let local_path = file.to_string_lossy().into_owned();
    Ok((
        BrowserState {
            url: format!("file://{local_path}"),
            title: target
                .rsplit('/')
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or("Local file")
                .into(),
            summary: "Trusted project file opened through Files authority.".into(),
            artifact_id: String::new(),
            preview_mode: "local-file".into(),
            profile_id: crate::runtime_sessions::browser_profile_id(session),
            local_path,
            html,
        },
        "open-local-file",
    ))
}

fn local_browser_dom_preview_content(file: &Path) -> Result<String, String> {
    if local_browser_file_is_markdown(file) {
        return std::fs::read_to_string(file).map_err(|error| format!("Cannot read file: {error}"));
    }
    Ok(String::new())
}

fn local_browser_file_is_markdown(file: &Path) -> bool {
    matches!(
        local_browser_extension(file).as_deref(),
        Some("md" | "markdown")
    )
}

fn local_browser_extension(file: &Path) -> Option<String> {
    file.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

fn normalize_public_browser_url(target: &str) -> Option<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.into());
    }

    if target.contains('/') || target.starts_with('.') || target.starts_with('~') {
        return None;
    }

    let root = active_workspace_root()?;
    if root.join(target).exists() {
        return None;
    }

    if looks_like_public_host(target) {
        return Some(format!("https://{target}/"));
    }

    None
}

fn looks_like_public_host(target: &str) -> bool {
    let value = target.trim();
    value.contains('.')
        && !value.contains(char::is_whitespace)
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}

fn resolve_browser_file_target(target: &str, actor: &str) -> Result<PathBuf, String> {
    let root =
        active_workspace_root().ok_or_else(|| "No trusted workspace root is active".to_string())?;
    if actor == "agent" {
        return crate::files::trusted_path(&root, target);
    }

    let path = file_path_from_browser_target(target);
    let candidate = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    std::fs::canonicalize(&candidate).map_err(|error| format!("Cannot resolve file path: {error}"))
}

fn file_path_from_browser_target(target: &str) -> PathBuf {
    if let Some(path) = target.strip_prefix("file://") {
        return PathBuf::from(path);
    }
    Path::new(target).to_path_buf()
}

fn browser_title_from_url(target: &str) -> String {
    target
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("Public page")
        .into()
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
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn task_008_trusted_root_browsing_omits_root_row_and_reads_file() {
        let root = task_008_root("browse");
        fs::create_dir_all(root.join("frontend")).expect("create frontend");
        fs::create_dir_all(root.join("node_modules")).expect("create ignored folder");
        fs::create_dir_all(root.join("mcp.md").join("node_modules"))
            .expect("create nested ignored folder");
        fs::write(root.join(".gitignore"), "node_modules/\n").expect("write gitignore");
        fs::write(root.join("README.md"), "Task 008 readme").expect("write readme");
        fs::write(
            root.join("frontend").join("app.js"),
            "console.log('task008');",
        )
        .expect("write app");
        initialize_git_repo(&root);

        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");
        let payload = load_workspace();

        assert_eq!(payload.files.breadcrumbs, vec!["c4os-task-008-browse"]);
        assert!(payload.files.roots.iter().any(|row| row[0] == "README.md"));
        assert!(payload.files.roots.iter().all(|row| row.len() >= 5));
        assert!(payload
            .files
            .roots
            .iter()
            .any(|row| row[0] == "node_modules"
                && row.get(6).is_some_and(|state| state == "ignored")));
        assert!(payload
            .files
            .roots
            .iter()
            .any(|row| row[0] == "mcp.md" && !row.get(6).is_some_and(|state| state == "ignored")));
        assert!(!payload
            .files
            .roots
            .first()
            .map(|row| row[0].as_str())
            .is_some_and(|name| name == "c4os-task-008-browse"));

        let opened = read_file(Some(FileRequest {
            path: Some("README.md".into()),
            content: None,
            session_id: None,
        }))
        .expect("read trusted file");
        assert_eq!(opened.path, "README.md");
        assert_eq!(opened.content, "Task 008 readme");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_008_edits_save_revert_and_update_session_owned_file_state() {
        let root = task_008_root("save");
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("notes.md"), "persisted").expect("write notes");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");
        let session = create_session(
            Some("c4os-task-008-save".into()),
            Some("Files save session".into()),
            Some("model/files".into()),
        );

        let opened = read_file(Some(FileRequest {
            path: Some("notes.md".into()),
            content: None,
            session_id: Some(session.id.clone()),
        }))
        .expect("read notes");
        assert_eq!(opened.saved_content, "persisted");

        let saved = save_file(Some(FileRequest {
            path: Some("notes.md".into()),
            content: Some("changed".into()),
            session_id: Some(session.id.clone()),
        }))
        .expect("save notes");

        assert!(saved.saved);
        assert_eq!(
            fs::read_to_string(root.join("notes.md")).expect("read saved"),
            "changed"
        );
        let restored = load_c4os_session(&session.id).expect("session restored");
        assert_eq!(restored.files.current_path, "notes.md");
        assert_eq!(restored.files.content, "changed");
        assert_eq!(restored.files.dirty, false);

        let reverted = read_file(Some(FileRequest {
            path: Some("notes.md".into()),
            content: None,
            session_id: Some(session.id.clone()),
        }))
        .expect("revert by reread");
        assert_eq!(reverted.content, "changed");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_008_new_sessions_inherit_real_workspace_file_roots() {
        let root = task_008_root("session-files");
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("README.md"), "real workspace").expect("write readme");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");

        let session = create_session(
            Some("c4os-task-008-session-files".into()),
            Some("Real files session".into()),
            Some("model/files".into()),
        );
        let restored = load_c4os_session(&session.id).expect("session restored");
        let rows = restored
            .files
            .roots
            .iter()
            .map(|row| row[0].as_str())
            .collect::<Vec<_>>();

        assert!(rows.contains(&"README.md"));
        assert!(rows.contains(&"src"));
        assert!(!rows.contains(&"mock-main.js"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_008_rejects_traversal_and_casual_git_access_before_read_or_write() {
        let root = task_008_root("guards");
        fs::create_dir_all(root.join(".git")).expect("create git");
        fs::write(root.join("safe.txt"), "safe").expect("write safe");
        fs::write(root.join(".git").join("config"), "secret").expect("write git");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");

        let outside = read_file(Some(FileRequest {
            path: Some("../outside.txt".into()),
            content: None,
            session_id: None,
        }))
        .expect_err("reject traversal read");
        assert!(outside.contains("outside trusted root"));

        let git = save_file(Some(FileRequest {
            path: Some(".git/config".into()),
            content: Some("mutated".into()),
            session_id: None,
        }))
        .expect_err("reject git write");
        assert!(git.contains(".git"));
        assert_eq!(
            fs::read_to_string(root.join(".git").join("config")).expect("read git"),
            "secret"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_009_artifact_preview_records_are_product_owned_and_secret_free() {
        let session = create_session(
            Some("c4os-task-009-artifacts".into()),
            Some("Artifact preview session".into()),
            Some("model/artifacts".into()),
        );
        let preview = create_artifact_preview(ArtifactPreviewRequest {
            session_id: Some(session.id.clone()),
            title: "Generated dashboard".into(),
            html: "<!doctype html><html><body><h1>Dashboard</h1></body></html>".into(),
        })
        .expect("create artifact preview");

        assert_eq!(preview.artifact.id, "artifact-1");
        assert_eq!(preview.browser.preview_mode, "artifact");
        assert_eq!(preview.browser.artifact_id, "artifact-1");
        assert_eq!(preview.artifact.safe_preview.title, "Generated dashboard");

        let restored = load_c4os_session(&session.id).expect("session restored");
        assert_eq!(restored.artifacts.len(), 1);
        assert_eq!(restored.browser.artifact_id, "artifact-1");
        let serialized = serde_json::to_string(&restored).expect("serialize session");
        assert!(serialized.contains("safePreview"));
        assert!(!serialized.contains("review-only-secret"));
        assert!(!serialized.contains("apiKey"));
        assert!(!serialized.contains("terminal-secret"));
    }

    #[test]
    fn task_010_browser_state_is_session_owned_and_supports_public_browsing() {
        let session = create_session(
            Some("c4os-task-010-browser".into()),
            Some("Browser session".into()),
            Some("model/browser".into()),
        );

        let opened = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "https://example.com/docs".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open public browser target");

        assert_eq!(opened.browser.url, "https://example.com/docs");
        assert_eq!(opened.browser.preview_mode, "public");
        assert!(opened.browser.profile_id.contains("c4os-task-010-browser"));
        assert_eq!(opened.action.actor, "user");
        assert_eq!(opened.action.action, "open-public-url");

        let restored = load_c4os_session(&session.id).expect("session restored");
        assert_eq!(restored.browser.url, "https://example.com/docs");
        assert_eq!(restored.browser.preview_mode, "public");
        assert_eq!(restored.browser_actions.len(), 1);
        assert_eq!(
            open_browser_preview(Some(session.id.clone())).url,
            "https://example.com/docs"
        );
    }

    #[test]
    fn task_010_browser_local_file_opening_uses_trusted_root_authority() {
        let root = std::env::temp_dir().join("c4os-task-010-browser-files");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        fs::write(
            root.join("preview.html"),
            r#"<link rel="stylesheet" href="form.css"><h1>Trusted local preview</h1><script src="form.js"></script>"#,
        )
        .expect("write preview");
        fs::write(root.join("form.css"), "body { background: #f0fdfa; }").expect("write css");
        fs::write(root.join("form.js"), "window.formLoaded = true;").expect("write js");
        let canonical_preview =
            fs::canonicalize(root.join("preview.html")).expect("canonical preview");
        fs::create_dir_all(root.join(".git")).expect("create git");
        fs::write(root.join(".git").join("config"), "secret").expect("write git");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");
        let session = create_session(
            Some("c4os-task-010-browser-files".into()),
            Some("Browser file session".into()),
            Some("model/browser".into()),
        );

        let opened = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "preview.html".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open trusted browser file");

        assert_eq!(opened.browser.preview_mode, "local-file");
        assert_eq!(
            opened.browser.local_path,
            canonical_preview.to_string_lossy()
        );
        assert_eq!(
            opened.browser.url,
            format!("file://{}", canonical_preview.to_string_lossy())
        );
        assert_eq!(opened.browser.html, "");

        let image = root.join("icon.png");
        fs::write(&image, [137, 80, 78, 71, 13, 10, 26, 10]).expect("write png");
        let opened_image = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "icon.png".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open trusted browser image");
        assert_eq!(opened_image.browser.preview_mode, "local-file");
        assert_eq!(opened_image.browser.html, "");
        assert!(opened_image.browser.url.ends_with("/icon.png"));

        fs::write(root.join("README.md"), "# Trusted markdown").expect("write markdown");
        let opened_markdown = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "README.md".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open trusted browser markdown");
        assert_eq!(opened_markdown.browser.preview_mode, "local-file");
        assert_eq!(opened_markdown.browser.html, "# Trusted markdown");

        let outside = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "../outside.html".into(),
            actor: Some("agent".into()),
            clear_request: true,
        })
        .expect_err("reject traversal");
        assert!(outside.contains("outside trusted root"));

        let git = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: ".git/config".into(),
            actor: Some("agent".into()),
            clear_request: true,
        })
        .expect_err("reject git");
        assert!(git.contains(".git"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_010a_browser_address_resolution_normalizes_public_and_user_local_targets() {
        let root = std::env::temp_dir().join("c4os-task-010a-address-resolution");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        let local = root.join("bin.js");
        fs::write(
            &local,
            "#!/usr/bin/env node\nrequire('./cjs/scripts/bin.js');",
        )
        .expect("write local browser file");
        let canonical_local = fs::canonicalize(&local).expect("canonical local path");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");
        let session = create_session(
            Some("c4os-task-010a-browser-resolution".into()),
            Some("Browser resolution session".into()),
            Some("model/browser".into()),
        );

        let public = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "iamawesome.com".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open normalized public target");
        assert_eq!(public.browser.url, "https://iamawesome.com/");
        assert_eq!(public.browser.preview_mode, "public");

        let relative = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "bin.js".into(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("open user local relative target");
        assert_eq!(
            relative.browser.url,
            format!("file://{}", canonical_local.to_string_lossy())
        );
        assert_eq!(
            relative.browser.local_path,
            canonical_local.to_string_lossy()
        );

        let outside_root = std::env::temp_dir().join("c4os-task-010a-outside-local.js");
        fs::write(&outside_root, "console.log('outside trusted root');")
            .expect("write outside file");
        let canonical_outside = fs::canonicalize(&outside_root).expect("canonical outside file");
        let absolute = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: outside_root.to_string_lossy().into_owned(),
            actor: Some("user".into()),
            clear_request: false,
        })
        .expect("user may open absolute local file");
        assert_eq!(
            absolute.browser.url,
            format!("file://{}", canonical_outside.to_string_lossy())
        );

        let agent_outside = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: outside_root.to_string_lossy().into_owned(),
            actor: Some("agent".into()),
            clear_request: true,
        })
        .expect_err("agent local file remains trusted-root scoped");
        assert!(agent_outside.contains("outside trusted root"));

        let _ = fs::remove_file(outside_root);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_010_agent_browser_actions_require_clear_request_and_are_recorded() {
        let session = create_session(
            Some("c4os-task-010-agent-browser".into()),
            Some("Agent Browser session".into()),
            Some("model/browser".into()),
        );

        let rejected = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "https://example.com/private".into(),
            actor: Some("agent".into()),
            clear_request: false,
        })
        .expect_err("agent browsing requires clear request");
        assert!(rejected.contains("clearly requested"));

        let opened = open_browser(BrowserOpenRequest {
            session_id: Some(session.id.clone()),
            target: "https://example.com/research".into(),
            actor: Some("agent".into()),
            clear_request: true,
        })
        .expect("open agent requested browser target");

        assert_eq!(opened.action.actor, "agent");
        assert_eq!(opened.action.action, "open-public-url");
        assert_eq!(opened.action.session_id, session.id);
        let restored = load_c4os_session(&session.id).expect("session restored");
        assert_eq!(restored.browser_actions.len(), 1);
        assert_eq!(
            restored.browser_actions[0].target,
            "https://example.com/research"
        );
    }

    #[test]
    fn command_inventory_returns_task_002_mock_payloads() {
        assert_eq!(load_workspace().workspace.project, "Mock Workspace Alpha");
        assert_eq!(open_browser_preview(None).title, "Mock rendered page");
        assert_eq!(
            list_extensions().mcp_servers,
            vec!["mock_node_repl", "mock_docs_mcp"]
        );
    }

    #[test]
    fn fake_run_preserves_task_002_failure_text() {
        assert_eq!(
            fake_failed_run("fail this run".into()),
            "Mock agent failed before producing output. prompt=fail this run"
        );
    }

    #[test]
    fn mock_file_and_terminal_commands_do_not_claim_real_io() {
        assert!(read_file(None).is_err());
        assert!(save_file(None).is_err());
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

    fn task_008_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("c4os-task-008-{name}"));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn initialize_git_repo(root: &std::path::Path) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("init")
            .arg("--quiet")
            .status()
            .expect("run git init");
        assert!(status.success());
    }
}
