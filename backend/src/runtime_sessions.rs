use crate::mock_data::{
    mock_workspace, ArtifactRecord, BrowserState, FilesState, SafeArtifactPreview, TerminalState,
    ThreadState,
};
use crate::openrouter::RuntimeEvent;
use crate::runtime_adapter::{C4osRuntimeAdapter, C4osRuntimeResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osWorkspaceRecord {
    pub id: String,
    pub project: String,
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osSessionRecord {
    pub id: String,
    pub workspace_id: String,
    pub project: String,
    pub title: String,
    pub selected_model: String,
    pub browser: BrowserState,
    pub artifacts: Vec<ArtifactRecord>,
    pub files: FilesState,
    pub terminal: TerminalState,
    pub thread: ThreadState,
    #[serde(default)]
    pub browser_actions: Vec<BrowserActionRecord>,
    pub turns: Vec<C4osTurnRecord>,
    pub messages: Vec<C4osMessageRecord>,
    pub runs: Vec<C4osRunRecord>,
    pub runtime_reference: C4osRuntimeReference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osTurnRecord {
    pub id: String,
    pub user: String,
    pub agent: String,
    pub extra: String,
    pub tool: String,
    pub run: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osMessageRecord {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osRunRecord {
    pub id: String,
    pub session_id: String,
    pub prompt_message_id: String,
    pub response_message_id: Option<String>,
    pub selected_model: String,
    pub status: String,
    pub runtime_reference_id: String,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActionRecord {
    pub id: String,
    pub session_id: String,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct C4osRuntimeReference {
    pub id: String,
    pub label: String,
    pub adapter: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub project: Option<String>,
    pub label: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPromptRequest {
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub prompt: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPromptResult {
    pub session: C4osSessionRecord,
    pub prompt: String,
    pub run: String,
    pub agent: String,
    pub model: String,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSessionRecord {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SessionStore {
    workspace: C4osWorkspaceRecord,
    sessions: Vec<C4osSessionRecord>,
}

static STORE: OnceLock<Mutex<SessionStore>> = OnceLock::new();

pub fn create_session(request: CreateSessionRequest) -> C4osSessionRecord {
    let mut store = load_store();
    let project = request
        .project
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| store.workspace.project.clone());
    let title = request
        .label
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "New chat session".into());
    let selected_model = request.model.unwrap_or_default();
    let id = unique_session_id(&store, &title);
    let mut session = empty_session(&store.workspace, &id, &project, &title, &selected_model);
    session.thread = ThreadState {
        user: title.clone(),
        agent: "Session created. Send a prompt to start the runtime.".into(),
        extra: "C4OS session record created.".into(),
        tool: "Session initialized".into(),
        run: "Waiting for first prompt".into(),
    };
    store.sessions.push(session.clone());
    save_store(&store);
    session
}

pub fn append_prompt<F>(request: SendPromptRequest, emit: F) -> Result<SendPromptResult, String>
where
    F: FnMut(RuntimeEvent),
{
    let mut store = load_store();
    let session_id = match request.session_id.filter(|value| !value.trim().is_empty()) {
        Some(id) => id,
        None => {
            let created = create_session(CreateSessionRequest {
                project: request.project.clone(),
                label: Some(session_title(&request.prompt)),
                model: request.model.clone(),
            });
            created.id
        }
    };
    let index = store
        .sessions
        .iter()
        .position(|session| session.id == session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{session_id}'"))?;
    let selected_model = request
        .model
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| store.sessions[index].selected_model.clone());
    let runtime = C4osRuntimeAdapter::run_chat(&request.prompt, &selected_model, emit)?;

    apply_runtime_result(&mut store.sessions[index], &request.prompt, runtime);
    let session = store.sessions[index].clone();
    save_store(&store);

    Ok(SendPromptResult {
        prompt: request.prompt,
        run: session
            .runs
            .last()
            .map(|run| run.status.clone())
            .unwrap_or_else(|| "completed".into()),
        agent: session.thread.agent.clone(),
        model: session.selected_model.clone(),
        events: session
            .runs
            .last()
            .map(|run| run.events.clone())
            .unwrap_or_default(),
        session,
    })
}

pub fn load_session(session_id: &str) -> Option<C4osSessionRecord> {
    load_store()
        .sessions
        .into_iter()
        .find(|session| session.id == session_id)
}

pub fn sessions() -> Vec<C4osSessionRecord> {
    load_store().sessions
}

pub fn project_sessions(project: &str) -> Vec<ProjectSessionRecord> {
    load_store()
        .sessions
        .into_iter()
        .filter(|session| session.project == project)
        .map(|session| ProjectSessionRecord {
            id: session.id,
            label: session.title,
        })
        .collect()
}

pub fn active_session() -> Option<C4osSessionRecord> {
    load_store().sessions.into_iter().next()
}

pub fn set_session_model(session_id: &str, model: &str) -> Result<(), String> {
    let mut store = load_store();
    let session = store
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{session_id}'"))?;
    session.selected_model = model.into();
    save_store(&store);
    Ok(())
}

pub fn set_session_files(session_id: &str, files: FilesState) -> Result<(), String> {
    let mut store = load_store();
    let session = store
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{session_id}'"))?;
    session.files = files;
    save_store(&store);
    Ok(())
}

pub fn create_session_artifact_preview(
    session_id: &str,
    title: &str,
    html: &str,
) -> Result<(ArtifactRecord, BrowserState), String> {
    let mut store = load_store();
    let session = store
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{session_id}'"))?;
    let id = format!("artifact-{}", session.artifacts.len() + 1);
    let title = if title.trim().is_empty() {
        "Generated HTML artifact"
    } else {
        title.trim()
    };
    let artifact = ArtifactRecord {
        id: id.clone(),
        title: title.into(),
        kind: "html".into(),
        origin: "generated".into(),
        safe_preview: SafeArtifactPreview {
            url: format!("artifact://{id}"),
            title: title.into(),
            summary: "Generated HTML artifact preview".into(),
            html: html.into(),
        },
    };
    session.browser = BrowserState {
        url: artifact.safe_preview.url.clone(),
        title: artifact.safe_preview.title.clone(),
        summary: artifact.safe_preview.summary.clone(),
        artifact_id: artifact.id.clone(),
        preview_mode: "artifact".into(),
        profile_id: browser_profile_id(session),
        local_path: String::new(),
        html: String::new(),
    };
    session.artifacts.push(artifact.clone());
    let browser = session.browser.clone();
    save_store(&store);
    Ok((artifact, browser))
}

pub fn set_session_browser(
    session_id: &str,
    browser: BrowserState,
    actor: &str,
    action: &str,
    target: &str,
) -> Result<BrowserActionRecord, String> {
    let mut store = load_store();
    let session = store
        .sessions
        .iter_mut()
        .find(|session| session.id == session_id)
        .ok_or_else(|| format!("Unknown C4OS session '{session_id}'"))?;
    session.browser = browser;
    let record = BrowserActionRecord {
        id: format!("browser-action-{}", session.browser_actions.len() + 1),
        session_id: session.id.clone(),
        actor: actor.into(),
        action: action.into(),
        target: target.into(),
        status: "recorded".into(),
    };
    session.browser_actions.push(record.clone());
    save_store(&store);
    Ok(record)
}

fn apply_runtime_result(session: &mut C4osSessionRecord, prompt: &str, runtime: C4osRuntimeResult) {
    session.selected_model = runtime.model.clone();
    let run_id = format!("c4os-run-{}", session.runs.len() + 1);
    let prompt_message_id = format!("c4os-message-{}-user", session.messages.len() + 1);
    let response_message_id = format!("c4os-message-{}-assistant", session.messages.len() + 2);

    session.messages.push(C4osMessageRecord {
        id: prompt_message_id.clone(),
        session_id: session.id.clone(),
        role: "user".into(),
        content: prompt.into(),
        run_id: Some(run_id.clone()),
    });
    session.messages.push(C4osMessageRecord {
        id: response_message_id.clone(),
        session_id: session.id.clone(),
        role: "assistant".into(),
        content: runtime.agent.clone(),
        run_id: Some(run_id.clone()),
    });
    session.runs.push(C4osRunRecord {
        id: run_id.clone(),
        session_id: session.id.clone(),
        prompt_message_id,
        response_message_id: Some(response_message_id),
        selected_model: runtime.model.clone(),
        status: runtime.run.clone(),
        runtime_reference_id: session.runtime_reference.id.clone(),
        events: runtime.events,
    });
    session.thread = ThreadState {
        user: prompt.into(),
        agent: runtime.agent,
        extra: "C4OS-owned session, run, message, and runtime-reference records persisted for this prompt.".into(),
        tool: session.runtime_reference.label.clone(),
        run: runtime.run,
    };
    session.turns.push(C4osTurnRecord {
        id: format!("{}-turn-{}", session.id, session.turns.len() + 1),
        user: session.thread.user.clone(),
        agent: session.thread.agent.clone(),
        extra: session.thread.extra.clone(),
        tool: session.thread.tool.clone(),
        run: session.thread.run.clone(),
    });
}

fn empty_session(
    workspace: &C4osWorkspaceRecord,
    id: &str,
    project: &str,
    title: &str,
    selected_model: &str,
) -> C4osSessionRecord {
    let fallback = mock_workspace();
    let files = crate::workspace::active_workspace_root()
        .and_then(|root| crate::files::list_files_state(&root, None).ok())
        .unwrap_or_else(|| fallback.files.clone());
    let mut browser = fallback.browser;
    browser.profile_id = format!("browser-profile-{}", stable_id(project));
    C4osSessionRecord {
        id: id.into(),
        workspace_id: workspace.id.clone(),
        project: project.into(),
        title: title.into(),
        selected_model: selected_model.into(),
        browser,
        artifacts: fallback.artifacts,
        files,
        terminal: fallback.terminal,
        thread: ThreadState {
            user: String::new(),
            agent: String::new(),
            extra: String::new(),
            tool: String::new(),
            run: String::new(),
        },
        browser_actions: Vec::new(),
        turns: Vec::new(),
        messages: Vec::new(),
        runs: Vec::new(),
        runtime_reference: C4osRuntimeReference {
            id: format!("{id}-runtime"),
            label: "C4OS runtime adapter".into(),
            adapter: "c4os-runtime-adapter".into(),
        },
    }
}

pub fn browser_profile_id(session: &C4osSessionRecord) -> String {
    format!("browser-profile-{}", stable_id(&session.project))
}

fn load_store() -> SessionStore {
    let mut store = store().lock().unwrap_or_else(|error| error.into_inner());
    if let Ok(raw) = fs::read_to_string(store_path()) {
        if let Ok(saved) = serde_json::from_str::<SessionStore>(&raw) {
            *store = saved;
        }
    }
    store.clone()
}

fn save_store(next: &SessionStore) {
    {
        let mut store = store().lock().unwrap_or_else(|error| error.into_inner());
        *store = next.clone();
    }
    if let Ok(serialized) = serde_json::to_string_pretty(next) {
        let path = store_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, serialized);
    }
}

fn store() -> &'static Mutex<SessionStore> {
    STORE.get_or_init(|| Mutex::new(default_store()))
}

fn default_store() -> SessionStore {
    let workspace = mock_workspace().workspace;
    SessionStore {
        workspace: C4osWorkspaceRecord {
            id: stable_id(&workspace.project),
            project: workspace.project,
            branch: workspace.branch,
        },
        sessions: Vec::new(),
    }
}

fn store_path() -> PathBuf {
    std::env::var("C4OS_SESSION_STORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("c4os-task-007-sessions.json"))
}

fn unique_session_id(store: &SessionStore, title: &str) -> String {
    let base = format!("c4os-session-{}", stable_id(title));
    if !store.sessions.iter().any(|session| session.id == base) {
        return base;
    }
    let mut counter = 2;
    loop {
        let candidate = format!("{base}-{counter}");
        if !store.sessions.iter().any(|session| session.id == candidate) {
            return candidate;
        }
        counter += 1;
    }
}

fn session_title(prompt: &str) -> String {
    let text = prompt.trim();
    if text.is_empty() {
        return "New chat session".into();
    }
    if text.len() > 48 {
        format!("{}...", &text[..45])
    } else {
        text.into()
    }
}

fn stable_id(label: &str) -> String {
    let normalized = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if normalized.is_empty() {
        "untitled".into()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_007_sessions_isolate_thread_run_message_and_surface_records() {
        reset_task_007_state("isolate");
        let first = create_session(CreateSessionRequest {
            project: Some("Runtime Session Repo".into()),
            label: Some("First persistent chat".into()),
            model: Some("model/alpha".into()),
        });
        let second = create_session(CreateSessionRequest {
            project: Some("Runtime Session Repo".into()),
            label: Some("Second persistent chat".into()),
            model: Some("model/beta".into()),
        });

        let first_result = append_prompt(
            SendPromptRequest {
                session_id: Some(first.id.clone()),
                project: None,
                prompt: "First prompt".into(),
                model: Some("model/alpha".into()),
            },
            |_| {},
        )
        .expect("append first");
        let second_result = append_prompt(
            SendPromptRequest {
                session_id: Some(second.id.clone()),
                project: None,
                prompt: "Second prompt".into(),
                model: Some("model/beta".into()),
            },
            |_| {},
        )
        .expect("append second");

        assert_eq!(first_result.session.messages.len(), 2);
        assert_eq!(second_result.session.messages.len(), 2);
        assert_eq!(first_result.session.selected_model, "model/alpha");
        assert_eq!(second_result.session.selected_model, "model/beta");
        assert_eq!(first_result.session.messages[0].session_id, first.id);
        assert_eq!(second_result.session.messages[0].session_id, second.id);
        assert_ne!(
            first_result.session.runtime_reference.id,
            second_result.session.runtime_reference.id
        );
    }

    #[test]
    fn task_007_prompt_append_targets_existing_session_only() {
        reset_task_007_state("append");
        let first = create_session(CreateSessionRequest {
            project: Some("Runtime Session Repo".into()),
            label: Some("First chat".into()),
            model: Some("model/alpha".into()),
        });
        let second = create_session(CreateSessionRequest {
            project: Some("Runtime Session Repo".into()),
            label: Some("Second chat".into()),
            model: Some("model/beta".into()),
        });

        append_prompt(
            SendPromptRequest {
                session_id: Some(second.id.clone()),
                project: None,
                prompt: "Append to second".into(),
                model: None,
            },
            |_| {},
        )
        .expect("append second");

        assert_eq!(load_session(&first.id).expect("first").messages.len(), 0);
        assert_eq!(load_session(&second.id).expect("second").messages.len(), 2);
    }

    #[test]
    fn task_007_restart_resume_restores_persisted_c4os_session_records() {
        reset_task_007_state("resume");
        let created = create_session(CreateSessionRequest {
            project: Some("Runtime Session Repo".into()),
            label: Some("Resume chat".into()),
            model: Some("model/resume".into()),
        });
        append_prompt(
            SendPromptRequest {
                session_id: Some(created.id.clone()),
                project: None,
                prompt: "Persist across restart".into(),
                model: None,
            },
            |_| {},
        )
        .expect("append");

        STORE.get().expect("store").lock().unwrap().sessions.clear();
        let restored = load_session(&created.id).expect("restored session");

        assert_eq!(restored.title, "Resume chat");
        assert_eq!(restored.selected_model, "model/resume");
        assert_eq!(restored.messages.len(), 2);
        assert!(restored.thread.extra.contains("C4OS-owned session"));
        assert!(!serde_json::to_string(&restored)
            .expect("serialize")
            .to_lowercase()
            .contains("opencode"));
    }

    fn reset_task_007_state(name: &str) {
        let path = std::env::temp_dir().join(format!("c4os-task-007-{name}.json"));
        std::env::set_var("C4OS_SESSION_STORE", path);
        let mut store = STORE
            .get_or_init(|| Mutex::new(default_store()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *store = default_store();
        let _ = fs::remove_file(store_path());
    }
}
