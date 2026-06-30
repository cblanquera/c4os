use crate::files::{list_files_state, read_file_state, save_file_state};
use crate::mock_data::{ArtifactRecord, BrowserState, FilesState, TerminalState};
use crate::runtime_sessions::{
    active_session, browser_profile_id, create_session as create_c4os_session,
    create_session_artifact_preview, load_session as load_c4os_session,
    run_session_terminal_command, session_workspace_root as c4os_session_workspace_root,
    set_session_browser as set_c4os_session_browser, set_session_files as set_c4os_session_files,
    BrowserActionRecord, C4osSessionRecord, CreateSessionRequest, TerminalActionRecord,
};
use crate::workspace::{active_workspace_descriptor, active_workspace_root};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const TOOL_TERMINAL_RUN: &str = "terminal.run";
pub const TOOL_FILES_LIST: &str = "files.list";
pub const TOOL_FILES_READ: &str = "files.read";
pub const TOOL_FILES_WRITE: &str = "files.write";
pub const TOOL_BROWSER_OPEN: &str = "browser.open";
pub const TOOL_ARTIFACT_PREVIEW: &str = "artifact.preview";

pub const EVENT_TOOL_CALL_REQUESTED: &str = "tool_call_requested";
pub const EVENT_TOOL_CALL_STARTED: &str = "tool_call_started";
pub const EVENT_TOOL_OUTPUT_DELTA: &str = "tool_output_delta";
pub const EVENT_TOOL_CALL_COMPLETED: &str = "tool_call_completed";
pub const EVENT_TOOL_CALL_REJECTED: &str = "tool_call_rejected";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolApproval {
    Deny,
    AskEveryTime,
    AutoReadonly,
    AutoWorkspace,
    AutoAlways,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAccess {
    WorkspaceRead,
    WorkspaceWrite,
    WorkspaceShell,
    PublicOrTrustedLocal,
    ArtifactPreview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPermission {
    pub enabled: bool,
    pub access: ToolAccess,
    pub approval: ToolApproval,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolConfig {
    pub tools: BTreeMap<String, ToolPermission>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub tool: String,
    pub access: ToolAccess,
    pub default_approval: ToolApproval,
    pub max_approval: ToolApproval,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRequest {
    pub tool: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub args: Value,
    #[serde(default)]
    pub session_config: Option<SessionToolConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallLifecycleEvent {
    pub kind: String,
    pub tool: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCallPayload {
    Files {
        state: FilesState,
        saved: bool,
    },
    Browser {
        browser: BrowserState,
        action: BrowserActionRecord,
    },
    ArtifactPreview {
        artifact: ArtifactRecord,
        browser: BrowserState,
    },
    Terminal {
        terminal: TerminalState,
        action: TerminalActionRecord,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallResponse {
    pub tool: String,
    pub status: String,
    pub events: Vec<ToolCallLifecycleEvent>,
    pub payload: ToolCallPayload,
}

pub fn dispatch_tool_call(request: ToolCallRequest) -> Result<ToolCallResponse, String> {
    let definition = tool_definition(&request.tool)?;
    evaluate_tool_config(&definition, request.session_config.as_ref())?;
    let tool = request.tool.clone();
    let mut events = vec![lifecycle_event(
        EVENT_TOOL_CALL_REQUESTED,
        &tool,
        "Runtime requested C4OS tool execution.",
    )];
    events.push(lifecycle_event(
        EVENT_TOOL_CALL_STARTED,
        &tool,
        "C4OS started product-owned tool execution.",
    ));
    let payload = match request.tool.as_str() {
        TOOL_FILES_LIST => {
            dispatch_files_read(request.session_id, string_arg(&request.args, "path"), true)?
        }
        TOOL_FILES_READ => {
            dispatch_files_read(request.session_id, string_arg(&request.args, "path"), false)?
        }
        TOOL_FILES_WRITE => dispatch_files_write(
            request.session_id,
            required_string_arg(&request.args, "path")?,
            string_arg(&request.args, "content").unwrap_or_default(),
        )?,
        TOOL_BROWSER_OPEN => dispatch_browser_open(request)?,
        TOOL_ARTIFACT_PREVIEW => dispatch_artifact_preview(request)?,
        TOOL_TERMINAL_RUN => dispatch_terminal_run(request)?,
        _ => return Err(format!("Unsupported C4OS tool '{}'", request.tool)),
    };
    events.push(lifecycle_event(
        EVENT_TOOL_CALL_COMPLETED,
        &tool,
        "C4OS completed product-owned tool execution.",
    ));
    Ok(ToolCallResponse {
        tool,
        status: "completed".into(),
        events,
        payload,
    })
}

pub fn tool_definition(tool: &str) -> Result<ToolDefinition, String> {
    let definition = match tool {
        TOOL_TERMINAL_RUN => ToolDefinition {
            tool: tool.into(),
            access: ToolAccess::WorkspaceShell,
            default_approval: ToolApproval::AskEveryTime,
            max_approval: ToolApproval::AskEveryTime,
        },
        TOOL_FILES_LIST | TOOL_FILES_READ => ToolDefinition {
            tool: tool.into(),
            access: ToolAccess::WorkspaceRead,
            default_approval: ToolApproval::AutoReadonly,
            max_approval: ToolApproval::AutoReadonly,
        },
        TOOL_FILES_WRITE => ToolDefinition {
            tool: tool.into(),
            access: ToolAccess::WorkspaceWrite,
            default_approval: ToolApproval::AskEveryTime,
            max_approval: ToolApproval::AskEveryTime,
        },
        TOOL_BROWSER_OPEN => ToolDefinition {
            tool: tool.into(),
            access: ToolAccess::PublicOrTrustedLocal,
            default_approval: ToolApproval::AutoReadonly,
            max_approval: ToolApproval::AutoReadonly,
        },
        TOOL_ARTIFACT_PREVIEW => ToolDefinition {
            tool: tool.into(),
            access: ToolAccess::ArtifactPreview,
            default_approval: ToolApproval::AutoReadonly,
            max_approval: ToolApproval::AutoReadonly,
        },
        _ => return Err(format!("Unknown C4OS tool '{tool}'")),
    };
    Ok(definition)
}

fn evaluate_tool_config(
    definition: &ToolDefinition,
    config: Option<&SessionToolConfig>,
) -> Result<(), String> {
    let Some(permission) = config.and_then(|config| config.tools.get(&definition.tool)) else {
        return Ok(());
    };
    if !permission.enabled {
        return Err(format!("C4OS tool '{}' is disabled", definition.tool));
    }
    if permission.approval == ToolApproval::Deny {
        return Err(format!("C4OS tool '{}' is denied", definition.tool));
    }
    if permission.access != definition.access {
        return Err(format!(
            "C4OS tool '{}' requested incompatible access",
            definition.tool
        ));
    }
    if approval_rank(&permission.approval) > approval_rank(&definition.max_approval) {
        return Err(format!(
            "C4OS tool '{}' requested wider approval than allowed",
            definition.tool
        ));
    }
    Ok(())
}

fn dispatch_files_read(
    request_session_id: Option<String>,
    path: Option<String>,
    force_list: bool,
) -> Result<ToolCallPayload, String> {
    let session_id = active_file_session_id(request_session_id);
    let root = file_command_root(session_id.as_deref())?;
    let path = path.unwrap_or_default();
    let state = if force_list || path.trim().is_empty() {
        list_files_state(&root, path_option(&path))?
    } else if crate::files::trusted_path(&root, &path)?.is_dir() {
        list_files_state(&root, Some(&path))?
    } else {
        read_file_state(&root, &path)?
    };
    if let Some(session_id) = session_id {
        let _ = set_c4os_session_files(&session_id, state.clone());
    }
    Ok(ToolCallPayload::Files {
        state,
        saved: false,
    })
}

fn dispatch_files_write(
    request_session_id: Option<String>,
    path: String,
    content: String,
) -> Result<ToolCallPayload, String> {
    let session_id = active_file_session_id(request_session_id);
    let root = file_command_root(session_id.as_deref())?;
    let state = save_file_state(&root, &path, &content)?;
    if let Some(session_id) = session_id {
        let _ = set_c4os_session_files(&session_id, state.clone());
        if let Some(workspace) = active_workspace_descriptor() {
            crate::records::record_file_action(
                &workspace.id,
                Some(&session_id),
                "save",
                &state.current_path,
                "saved",
            );
        }
    }
    Ok(ToolCallPayload::Files { state, saved: true })
}

fn dispatch_terminal_run(request: ToolCallRequest) -> Result<ToolCallPayload, String> {
    let session_id = request
        .session_id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| active_session().map(|session| session.id))
        .ok_or_else(|| "A C4OS session is required before running Terminal commands".to_string())?;
    let command = required_string_arg(&request.args, "command")?;
    let terminal_kind = string_arg(&request.args, "terminalKind")
        .or_else(|| string_arg(&request.args, "terminal_kind"))
        .unwrap_or_else(|| "user".into());
    let (terminal, action) = run_session_terminal_command(&session_id, &command, &terminal_kind)?;
    Ok(ToolCallPayload::Terminal { terminal, action })
}

fn dispatch_artifact_preview(request: ToolCallRequest) -> Result<ToolCallPayload, String> {
    let session_id = request
        .session_id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| active_session().map(|session| session.id))
        .ok_or_else(|| "A C4OS session is required before previewing an artifact".to_string())?;
    let title = required_string_arg(&request.args, "title")?;
    let html = required_string_arg(&request.args, "html")?;
    let (artifact, browser) = create_session_artifact_preview(&session_id, &title, &html)?;
    Ok(ToolCallPayload::ArtifactPreview { artifact, browser })
}

fn dispatch_browser_open(request: ToolCallRequest) -> Result<ToolCallPayload, String> {
    let session = browser_session_from_request(request.session_id)?;
    let actor = normalized_browser_actor(request.actor.as_deref());
    let clear_request = bool_arg(&request.args, "clearRequest");
    if actor == "agent" && !clear_request {
        return Err("Agent Browser access must be clearly requested for this request".into());
    }
    let target = required_string_arg(&request.args, "target")?;
    let root = c4os_session_workspace_root(&session.id)
        .or_else(active_workspace_root)
        .or_else(|| public_browser_target_without_root(&target).then(PathBuf::new))
        .ok_or_else(|| "No trusted workspace root is active".to_string())?;
    let (browser, action) = browser_state_for_target(&session, &root, &target, &actor)?;
    let action_record =
        set_c4os_session_browser(&session.id, browser.clone(), &actor, action, &target)?;
    Ok(ToolCallPayload::Browser {
        browser,
        action: action_record,
    })
}

fn browser_session_from_request(
    request_session_id: Option<String>,
) -> Result<C4osSessionRecord, String> {
    if let Some(session) = request_session_id
        .filter(|value| !value.trim().is_empty())
        .as_deref()
        .and_then(load_c4os_session)
    {
        return Ok(session);
    }
    if let Some(session) = active_session() {
        return Ok(session);
    }
    let descriptor = active_workspace_descriptor().ok_or_else(|| {
        "A trusted workspace is required before opening Browser state".to_string()
    })?;
    Ok(create_c4os_session(CreateSessionRequest {
        project: Some(descriptor.name),
        label: Some("Browser session".into()),
        model: None,
    }))
}

fn browser_state_for_target(
    session: &C4osSessionRecord,
    root: &Path,
    target: &str,
    actor: &str,
) -> Result<(BrowserState, &'static str), String> {
    if let Some(public_url) = normalize_public_browser_url(root, target) {
        return Ok((
            BrowserState {
                url: public_url.clone(),
                title: browser_title_from_url(&public_url),
                summary: "Public page opened in the project-scoped Browser surface.".into(),
                artifact_id: String::new(),
                preview_mode: "public".into(),
                profile_id: browser_profile_id(session),
                local_path: String::new(),
                html: String::new(),
            },
            "open-public-url",
        ));
    }

    let file = resolve_browser_file_target(root, target, actor)?;
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
            profile_id: browser_profile_id(session),
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

fn normalize_public_browser_url(root: &Path, target: &str) -> Option<String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.into());
    }

    if target.contains('/') || target.starts_with('.') || target.starts_with('~') {
        return None;
    }

    if root.join(target).exists() {
        return None;
    }

    if looks_like_public_host(target) {
        return Some(format!("https://{target}/"));
    }

    None
}

fn public_browser_target_without_root(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || looks_like_public_host(target)
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

fn resolve_browser_file_target(root: &Path, target: &str, actor: &str) -> Result<PathBuf, String> {
    if actor == "agent" {
        return crate::files::trusted_path(root, target);
    }

    let path = file_path_from_browser_target(target);
    let candidate = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("Cannot resolve Browser target: {error}"))?;
    if canonical
        .components()
        .any(|component| component.as_os_str() == ".git")
    {
        return Err("Browser target cannot open .git internals".into());
    }
    Ok(canonical)
}

fn file_path_from_browser_target(target: &str) -> PathBuf {
    let value = target.trim();
    if let Some(path) = value.strip_prefix("file://") {
        return PathBuf::from(path);
    }
    PathBuf::from(value)
}

fn browser_title_from_url(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .into()
}

fn active_file_session_id(request_session_id: Option<String>) -> Option<String> {
    request_session_id
        .filter(|value| !value.trim().is_empty())
        .or_else(|| active_session().map(|session| session.id))
}

fn file_command_root(session_id: Option<&str>) -> Result<PathBuf, String> {
    if let Some(root) = session_id.and_then(c4os_session_workspace_root) {
        return Ok(root);
    }
    active_workspace_root().ok_or_else(|| "No trusted workspace root is active".to_string())
}

fn normalized_browser_actor(actor: Option<&str>) -> String {
    match actor.unwrap_or("user").trim().to_ascii_lowercase().as_str() {
        "agent" => "agent".into(),
        _ => "user".into(),
    }
}

fn path_option(path: &str) -> Option<&str> {
    if path.trim().is_empty() {
        None
    } else {
        Some(path)
    }
}

fn string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn required_string_arg(args: &Value, key: &str) -> Result<String, String> {
    string_arg(args, key).ok_or_else(|| format!("Tool argument '{key}' is required"))
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn lifecycle_event(kind: &str, tool: &str, text: &str) -> ToolCallLifecycleEvent {
    ToolCallLifecycleEvent {
        kind: kind.into(),
        tool: tool.into(),
        text: text.into(),
    }
}

fn approval_rank(approval: &ToolApproval) -> u8 {
    match approval {
        ToolApproval::Deny => 0,
        ToolApproval::AskEveryTime => 1,
        ToolApproval::AutoReadonly => 2,
        ToolApproval::AutoWorkspace => 3,
        ToolApproval::AutoAlways => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wo_006_rejects_session_config_that_widens_tool_approval() {
        let mut tools = BTreeMap::new();
        tools.insert(
            TOOL_TERMINAL_RUN.into(),
            ToolPermission {
                enabled: true,
                access: ToolAccess::WorkspaceShell,
                approval: ToolApproval::AutoAlways,
            },
        );
        let request = ToolCallRequest {
            tool: TOOL_TERMINAL_RUN.into(),
            session_id: Some("missing-session".into()),
            actor: Some("agent".into()),
            args: serde_json::json!({ "command": "git status", "terminalKind": "agent" }),
            session_config: Some(SessionToolConfig { tools }),
        };

        let error = dispatch_tool_call(request).expect_err("widened approval rejected");
        assert!(error.contains("wider approval"));
    }

    #[test]
    fn wo_006_rejects_session_config_that_denies_tool() {
        let mut tools = BTreeMap::new();
        tools.insert(
            TOOL_FILES_READ.into(),
            ToolPermission {
                enabled: true,
                access: ToolAccess::WorkspaceRead,
                approval: ToolApproval::Deny,
            },
        );
        let request = ToolCallRequest {
            tool: TOOL_FILES_READ.into(),
            session_id: None,
            actor: Some("agent".into()),
            args: serde_json::json!({ "path": "" }),
            session_config: Some(SessionToolConfig { tools }),
        };

        let error = dispatch_tool_call(request).expect_err("denied tool rejected");
        assert!(error.contains("is denied"));
    }

    #[test]
    fn wo_006_allows_narrowed_readonly_file_config_before_execution_boundary() {
        let mut tools = BTreeMap::new();
        tools.insert(
            TOOL_FILES_READ.into(),
            ToolPermission {
                enabled: true,
                access: ToolAccess::WorkspaceRead,
                approval: ToolApproval::AskEveryTime,
            },
        );
        let definition = tool_definition(TOOL_FILES_READ).expect("files.read definition");
        let config = SessionToolConfig { tools };

        evaluate_tool_config(&definition, Some(&config)).expect("narrowed approval is allowed");
    }
}
