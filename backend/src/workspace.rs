use crate::mock_data::{
    mock_workspace, ProjectRecord, ThreadState, WorkspacePayload, WorkspaceSummary,
};
use crate::provider_models::DEFAULT_MODEL;
#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel, NSSavePanel};
#[cfg(target_os = "macos")]
use objc2_foundation::NSString;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDescriptor {
    pub schema_version: u32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub root_path: String,
    #[serde(default)]
    pub trusted: bool,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceActivation {
    pub path: String,
    pub trusted: bool,
    pub descriptor: WorkspaceDescriptor,
    pub payload: WorkspacePayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceFile {
    pub schema_version: u32,
    #[serde(default)]
    pub name: String,
    pub active: WorkspaceDescriptor,
    #[serde(default)]
    pub projects: Vec<WorkspaceDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRecentsStore {
    pub projects: Vec<WorkspaceDescriptor>,
    #[serde(default)]
    pub workspaces: Vec<RecentWorkspaceRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentWorkspaceRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub active: WorkspaceDescriptor,
    #[serde(default)]
    pub projects: Vec<WorkspaceDescriptor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceBootstrapRequest {
    Path(String),
    File(String),
}

static ACTIVE_WORKSPACE: OnceLock<Mutex<Option<WorkspaceDescriptor>>> = OnceLock::new();
static WORKSPACE_PROJECTS: OnceLock<Mutex<Vec<WorkspaceDescriptor>>> = OnceLock::new();
static WORKSPACE_PROJECTS_LOADED: OnceLock<Mutex<bool>> = OnceLock::new();
static WORKSPACE_BOOTSTRAP_REQUEST: OnceLock<Mutex<Option<WorkspaceBootstrapRequest>>> =
    OnceLock::new();

pub fn activate_workspace(path: Option<String>) -> Result<WorkspaceActivation, String> {
    let root = match path {
        Some(input) if !input.trim().is_empty() => PathBuf::from(input),
        _ => choose_workspace_folder()?,
    };

    activate_workspace_at(root)
}

pub fn set_workspace_bootstrap_request(request: Option<WorkspaceBootstrapRequest>) {
    *workspace_bootstrap_request()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = request;
}

pub fn bootstrap_workspace_from_launch_flag() -> Result<Option<WorkspaceActivation>, String> {
    if active_workspace_descriptor().is_some() {
        return Ok(None);
    }

    match workspace_bootstrap_request()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
    {
        Some(WorkspaceBootstrapRequest::Path(path)) => activate_workspace_at(path).map(Some),
        Some(WorkspaceBootstrapRequest::File(path)) => open_workspace_file(path).map(Some),
        None => Ok(None),
    }
}

pub fn activate_workspace_at(root: impl AsRef<Path>) -> Result<WorkspaceActivation, String> {
    let root = root.as_ref();
    let canonical =
        fs::canonicalize(root).map_err(|error| format!("Cannot open workspace folder: {error}"))?;

    if !canonical.is_dir() {
        return Err("Workspace path must be a folder".into());
    }

    let descriptor = load_or_create_descriptor(&canonical)?;
    set_active_workspace(descriptor.clone());
    remember_workspace_descriptor(descriptor.clone());
    let payload = payload_for_descriptor(&descriptor);

    Ok(WorkspaceActivation {
        path: descriptor.root_path.clone(),
        trusted: descriptor.trusted,
        descriptor,
        payload,
    })
}

pub fn active_workspace_descriptor() -> Option<WorkspaceDescriptor> {
    active_workspace()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

pub fn active_workspace_root() -> Option<PathBuf> {
    active_workspace_descriptor().map(|descriptor| PathBuf::from(descriptor.root_path))
}

pub fn save_workspace_file(path: impl AsRef<Path>) -> Result<WorkspaceDescriptor, String> {
    let descriptor = active_workspace_descriptor()
        .ok_or_else(|| "No trusted workspace is active".to_string())?;
    write_workspace_file(path.as_ref(), &descriptor)?;
    Ok(descriptor)
}

pub fn open_workspace_file(path: impl AsRef<Path>) -> Result<WorkspaceActivation, String> {
    let path = path.as_ref();
    let raw =
        fs::read_to_string(path).map_err(|error| format!("Cannot read workspace file: {error}"))?;
    let workspace_file = parse_workspace_file(path, &raw)?;
    if workspace_file.active.root_path.trim().is_empty() {
        return Err("Workspace file is missing root path".into());
    }
    replace_workspace_project_descriptors(workspace_file.projects.clone());
    remember_workspace_file(path, &workspace_file);
    activate_workspace_at(workspace_file.active.root_path)
}

fn load_or_create_descriptor(root: &Path) -> Result<WorkspaceDescriptor, String> {
    let path = descriptor_path(root);

    if path.exists() {
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("Cannot read workspace descriptor: {error}"))?;
        let mut descriptor: WorkspaceDescriptor = serde_json::from_str(&raw)
            .map_err(|error| format!("Cannot parse workspace descriptor: {error}"))?;
        descriptor.root_path = root.to_string_lossy().into_owned();
        descriptor.trusted = true;
        descriptor.updated_at = now_unix_seconds();
        write_descriptor(&path, &descriptor)?;
        return Ok(descriptor);
    }

    fs::create_dir_all(path.parent().expect("descriptor has parent"))
        .map_err(|error| format!("Cannot create .c4os descriptor folder: {error}"))?;

    let timestamp = now_unix_seconds();
    let descriptor = WorkspaceDescriptor {
        schema_version: 1,
        id: workspace_id(root),
        name: workspace_name(root),
        root_path: root.to_string_lossy().into_owned(),
        trusted: true,
        created_at: timestamp,
        updated_at: timestamp,
    };

    write_descriptor(&path, &descriptor)?;
    Ok(descriptor)
}

fn choose_workspace_folder() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        choose_workspace_folder_on_main().ok_or_else(|| "No workspace folder selected".to_string())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Workspace folder chooser is not available on this platform yet".into())
    }
}

#[cfg(target_os = "macos")]
pub fn choose_workspace_folder_on_main() -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setTitle(Some(&NSString::from_str("Open C4OS Workspace Folder")));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    panel.setCanChooseFiles(false);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(false);

    if panel.runModal() == NSModalResponseOK {
        return panel.URL().and_then(|url| url.to_file_path());
    }
    None
}

#[cfg(target_os = "macos")]
pub fn choose_workspace_file_on_main() -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setTitle(Some(&NSString::from_str("Open C4OS Workspace File")));
    panel.setPrompt(Some(&NSString::from_str("Open")));
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(false);

    if panel.runModal() == NSModalResponseOK {
        return panel.URL().and_then(|url| url.to_file_path());
    }
    None
}

#[cfg(target_os = "macos")]
pub fn choose_save_workspace_file_on_main() -> Option<PathBuf> {
    let mtm = MainThreadMarker::new()?;
    let panel = NSSavePanel::savePanel(mtm);
    panel.setTitle(Some(&NSString::from_str("Save C4OS Workspace File")));
    panel.setPrompt(Some(&NSString::from_str("Save")));
    panel.setNameFieldStringValue(&NSString::from_str("workspace.c4os.json"));

    if panel.runModal() == NSModalResponseOK {
        return panel.URL().and_then(|url| url.to_file_path());
    }
    None
}

fn write_descriptor(path: &Path, descriptor: &WorkspaceDescriptor) -> Result<(), String> {
    let json = serde_json::to_string_pretty(descriptor)
        .map_err(|error| format!("Cannot serialize workspace descriptor: {error}"))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("Cannot write workspace descriptor: {error}"))
}

fn write_workspace_file(path: &Path, active: &WorkspaceDescriptor) -> Result<(), String> {
    let mut projects = workspace_descriptors_for_payload(active);
    if projects.is_empty() {
        projects.push(active.clone());
    }
    let workspace_file = WorkspaceFile {
        schema_version: 1,
        name: workspace_file_name(path),
        active: active.clone(),
        projects,
    };
    let json = serde_json::to_string_pretty(&workspace_file)
        .map_err(|error| format!("Cannot serialize workspace file: {error}"))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("Cannot write workspace descriptor: {error}"))
}

fn parse_workspace_file(path: &Path, raw: &str) -> Result<WorkspaceFile, String> {
    if let Ok(workspace_file) = serde_json::from_str::<WorkspaceFile>(raw) {
        let WorkspaceFile {
            schema_version,
            name,
            active,
            projects,
        } = workspace_file;
        let mut projects = projects;
        if projects.is_empty() {
            projects.push(active.clone());
        }
        let mut normalized = WorkspaceFile {
            schema_version,
            name: workspace_file_name_from_optional(path, &name),
            active: normalize_workspace_descriptor(path, active)?,
            projects: projects
                .into_iter()
                .map(|descriptor| normalize_workspace_descriptor(path, descriptor))
                .collect::<Result<Vec<_>, _>>()?,
        };
        if !normalized
            .projects
            .iter()
            .any(|project| project.root_path == normalized.active.root_path)
        {
            normalized.projects.insert(0, normalized.active.clone());
        }
        normalized.projects = dedupe_workspace_descriptors(normalized.projects);
        return Ok(WorkspaceFile {
            projects: normalized.projects,
            ..normalized
        });
    }

    let descriptor: WorkspaceDescriptor = serde_json::from_str(raw)
        .map_err(|error| format!("Cannot parse workspace file: {error}"))?;
    let descriptor = normalize_workspace_descriptor(path, descriptor)?;
    Ok(WorkspaceFile {
        schema_version: 1,
        name: workspace_file_name(path),
        active: descriptor.clone(),
        projects: vec![descriptor],
    })
}

fn normalize_workspace_descriptor(
    workspace_file_path: &Path,
    mut descriptor: WorkspaceDescriptor,
) -> Result<WorkspaceDescriptor, String> {
    let root = resolve_workspace_root(workspace_file_path, &descriptor.root_path);
    let canonical = fs::canonicalize(&root)
        .map_err(|error| format!("Cannot open workspace folder: {error}"))?;
    if !canonical.is_dir() {
        return Err("Workspace path must be a folder".into());
    }
    descriptor.root_path = canonical.to_string_lossy().into_owned();
    descriptor.id = workspace_id(&canonical);
    if descriptor.name.trim().is_empty() {
        descriptor.name = workspace_name(&canonical);
    }
    descriptor.trusted = true;
    Ok(descriptor)
}

fn resolve_workspace_root(workspace_file_path: &Path, root_path: &str) -> PathBuf {
    let root = PathBuf::from(root_path);
    if root.is_absolute() {
        return root;
    }
    workspace_file_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(root)
}

fn descriptor_path(root: &Path) -> PathBuf {
    root.join(".c4os").join("workspace.json")
}

pub fn project_records_for_descriptor(descriptor: &WorkspaceDescriptor) -> Vec<ProjectRecord> {
    workspace_descriptors_for_payload(descriptor)
        .into_iter()
        .map(|descriptor| ProjectRecord {
            id: descriptor.id,
            name: descriptor.name,
            root_path: descriptor.root_path,
            workspace_file_path: String::new(),
            sessions: vec![],
        })
        .collect()
}

pub fn recent_project_records() -> Vec<ProjectRecord> {
    load_workspace_projects_from_disk();
    let workspace_recents = workspace_recents()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .workspaces
        .clone();
    if !workspace_recents.is_empty() {
        return workspace_recents
            .into_iter()
            .map(|workspace| ProjectRecord {
                id: workspace.id,
                name: workspace.name,
                root_path: workspace.active.root_path,
                workspace_file_path: workspace.path,
                sessions: vec![],
            })
            .collect();
    }

    workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
        .into_iter()
        .map(|descriptor| ProjectRecord {
            id: descriptor.id,
            name: descriptor.name,
            root_path: descriptor.root_path,
            workspace_file_path: String::new(),
            sessions: vec![],
        })
        .collect()
}

fn payload_for_descriptor(descriptor: &WorkspaceDescriptor) -> WorkspacePayload {
    let mut payload = mock_workspace();

    payload.workspace = WorkspaceSummary {
        project: descriptor.name.clone(),
        session: "".into(),
        branch: "main".into(),
        model: DEFAULT_MODEL.into(),
        session_id: "".into(),
    };
    payload.projects = project_records_for_descriptor(descriptor);
    payload.files = crate::files::list_files_state(Path::new(&descriptor.root_path), None)
        .unwrap_or_else(|_| {
            payload.files.breadcrumbs = vec![descriptor.name.clone()];
            payload.files.clone()
        });
    payload.thread = ThreadState {
        user: "Open trusted local workspace.".into(),
        agent: format!(
            "Workspace '{}' is active from a real local descriptor.",
            descriptor.name
        ),
        extra: "Provider/model records, agent processing, Browser, Files content, Terminal output, extensions, approvals, memory, artifacts, actions, and deeper persistence remain mocked.".into(),
        tool: "Workspace descriptor loaded".into(),
        run: "Ready for first session".into(),
    };

    payload
}

fn set_active_workspace(descriptor: WorkspaceDescriptor) {
    *active_workspace()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(descriptor);
}

fn remember_workspace_descriptor(descriptor: WorkspaceDescriptor) {
    load_workspace_projects_from_disk();
    let mut descriptors = workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    descriptors
        .retain(|record| record.root_path != descriptor.root_path && record.id != descriptor.id);
    descriptors.insert(0, descriptor);
    drop(descriptors);
    save_workspace_projects_to_disk();
}

fn remember_workspace_file(path: &Path, workspace_file: &WorkspaceFile) {
    load_workspace_projects_from_disk();
    let path = path.to_string_lossy().into_owned();
    let record = RecentWorkspaceRecord {
        id: workspace_file_id(&path),
        name: workspace_file.name.clone(),
        path: path.clone(),
        active: workspace_file.active.clone(),
        projects: workspace_file.projects.clone(),
    };
    let mut recents = workspace_recents()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    recents
        .workspaces
        .retain(|workspace| workspace.path != path && workspace.id != record.id);
    recents.workspaces.insert(0, record);
    drop(recents);
    save_workspace_projects_to_disk();
}

fn replace_workspace_project_descriptors(descriptors: Vec<WorkspaceDescriptor>) {
    *workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = dedupe_workspace_descriptors(descriptors);
    *workspace_projects_loaded()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = true;
    save_workspace_projects_to_disk();
}

fn workspace_descriptors_for_payload(active: &WorkspaceDescriptor) -> Vec<WorkspaceDescriptor> {
    load_workspace_projects_from_disk();
    let descriptors = workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    let mut ordered = Vec::with_capacity(descriptors.len() + 1);
    ordered.push(active.clone());
    ordered.extend(
        descriptors
            .into_iter()
            .filter(|record| record.root_path != active.root_path && record.id != active.id),
    );
    dedupe_workspace_descriptors(ordered)
        .into_iter()
        .filter(|descriptor| Path::new(&descriptor.root_path).is_dir())
        .collect()
}

fn dedupe_workspace_descriptors(descriptors: Vec<WorkspaceDescriptor>) -> Vec<WorkspaceDescriptor> {
    let mut deduped = Vec::new();
    for descriptor in descriptors {
        if descriptor.root_path.trim().is_empty() {
            continue;
        }
        if deduped.iter().any(|record: &WorkspaceDescriptor| {
            record.root_path == descriptor.root_path || record.id == descriptor.id
        }) {
            continue;
        }
        deduped.push(descriptor);
    }
    deduped
}

fn load_workspace_projects_from_disk() {
    {
        let loaded = workspace_projects_loaded()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *loaded {
            return;
        }
    }
    *workspace_projects_loaded()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = true;
    let Some(path) = workspace_recents_path() else {
        return;
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let Ok(store) = serde_json::from_str::<WorkspaceRecentsStore>(&raw) else {
        return;
    };
    *workspace_recents()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = store.clone();
    *workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = dedupe_workspace_descriptors(store.projects);
}

fn save_workspace_projects_to_disk() {
    let Some(path) = workspace_recents_path() else {
        return;
    };
    let mut store = workspace_recents()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    store.projects = workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    let Ok(serialized) = serde_json::to_string_pretty(&store) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, format!("{serialized}\n"));
}

fn workspace_recents_path() -> Option<PathBuf> {
    match std::env::var("C4OS_WORKSPACE_RECENTS_STORE") {
        Ok(path) if !path.trim().is_empty() => Some(PathBuf::from(path)),
        #[cfg(not(test))]
        _ => Some(std::env::temp_dir().join("c4os-workspace-recents.json")),
        #[cfg(test)]
        _ => None,
    }
}

fn active_workspace() -> &'static Mutex<Option<WorkspaceDescriptor>> {
    ACTIVE_WORKSPACE.get_or_init(|| Mutex::new(None))
}

fn workspace_projects() -> &'static Mutex<Vec<WorkspaceDescriptor>> {
    WORKSPACE_PROJECTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn workspace_recents() -> &'static Mutex<WorkspaceRecentsStore> {
    static WORKSPACE_RECENTS: OnceLock<Mutex<WorkspaceRecentsStore>> = OnceLock::new();
    WORKSPACE_RECENTS.get_or_init(|| Mutex::new(WorkspaceRecentsStore::default()))
}

fn workspace_projects_loaded() -> &'static Mutex<bool> {
    WORKSPACE_PROJECTS_LOADED.get_or_init(|| Mutex::new(false))
}

fn workspace_bootstrap_request() -> &'static Mutex<Option<WorkspaceBootstrapRequest>> {
    WORKSPACE_BOOTSTRAP_REQUEST.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn reset_active_workspace_for_test() {
    *active_workspace()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = None;
    workspace_projects()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
    *workspace_recents()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = WorkspaceRecentsStore::default();
    *workspace_projects_loaded()
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = false;
    set_workspace_bootstrap_request(None);
}

fn workspace_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Local Workspace")
        .to_string()
}

fn workspace_file_name(path: &Path) -> String {
    workspace_file_name_from_optional(path, "")
}

fn workspace_file_name_from_optional(path: &Path, configured: &str) -> String {
    let configured = configured.trim();
    if !configured.is_empty() {
        return configured.to_string();
    }
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    filename
        .strip_suffix(".c4os.json")
        .or_else(|| filename.strip_suffix(".json"))
        .filter(|name| !name.is_empty())
        .unwrap_or("workspace")
        .to_string()
}

fn workspace_id(root: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    root.to_string_lossy().hash(&mut hasher);
    format!("c4os-ws-{:016x}", hasher.finish())
}

fn workspace_file_id(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("c4os-file-{:016x}", hasher.finish())
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_004_creates_descriptor_for_real_local_workspace() {
        let root = test_workspace_root("creates");
        fs::create_dir_all(&root).expect("create temp workspace");

        let activation = activate_workspace_at(&root).expect("activate workspace");
        let descriptor_path = root.join(".c4os").join("workspace.json");
        let descriptor = fs::read_to_string(descriptor_path).expect("read descriptor");

        assert_eq!(activation.trusted, true);
        assert_eq!(activation.descriptor.name, "c4os-task-004-creates");
        assert_eq!(
            activation.payload.workspace.project,
            "c4os-task-004-creates"
        );
        assert_eq!(activation.payload.workspace.session, "");
        assert_eq!(
            activation.payload.projects[0].sessions,
            Vec::<crate::runtime_sessions::ProjectSessionRecord>::new()
        );
        assert!(descriptor.contains("\"trusted\": true"));
        assert!(descriptor.contains("\"schemaVersion\": 1"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_004_loads_existing_descriptor_instead_of_mock_workspace_identity() {
        let root = test_workspace_root("loads");
        fs::create_dir_all(root.join(".c4os")).expect("create descriptor dir");
        let descriptor = WorkspaceDescriptor {
            schema_version: 1,
            id: "c4os-ws-existing".into(),
            name: "Existing Product Repo".into(),
            root_path: root.to_string_lossy().into_owned(),
            trusted: true,
            created_at: 10,
            updated_at: 10,
        };
        write_descriptor(&root.join(".c4os").join("workspace.json"), &descriptor)
            .expect("write descriptor");

        let activation = activate_workspace_at(&root).expect("activate existing workspace");

        assert_eq!(activation.descriptor.id, "c4os-ws-existing");
        assert_eq!(
            activation.payload.workspace.project,
            "Existing Product Repo"
        );
        assert_ne!(activation.payload.workspace.project, "Mock Workspace Alpha");
        assert_eq!(
            activation.payload.thread.tool,
            "Workspace descriptor loaded"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_013a_saves_and_opens_portable_workspace_file_without_secrets() {
        let root = test_workspace_root("portable-file");
        fs::create_dir_all(&root).expect("create temp workspace");
        let activation = activate_workspace_at(&root).expect("activate workspace");
        let workspace_file = root.join("review-workspace.c4os.json");

        let saved = save_workspace_file(&workspace_file).expect("save workspace file");
        let raw = fs::read_to_string(&workspace_file).expect("read saved workspace file");
        let reopened = open_workspace_file(&workspace_file).expect("open saved workspace file");

        assert_eq!(saved.root_path, activation.descriptor.root_path);
        assert_eq!(reopened.descriptor.id, activation.descriptor.id);
        assert_eq!(
            reopened.payload.workspace.project,
            activation.descriptor.name
        );
        assert!(!raw.contains("review-only-secret"));
        assert!(!raw.contains("apiKey"));
        assert!(!raw.contains("messages"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn task_013a_workspace_file_restores_all_open_workspace_projects() {
        let first_root = test_workspace_root("portable-file-first-project");
        let second_root = test_workspace_root("portable-file-second-project");
        fs::create_dir_all(&first_root).expect("create first temp workspace");
        fs::create_dir_all(&second_root).expect("create second temp workspace");
        reset_active_workspace_for_test();

        let first = activate_workspace_at(&first_root).expect("activate first workspace");
        let second = activate_workspace_at(&second_root).expect("activate second workspace");
        let workspace_file = second_root.join("multi-project-workspace.c4os.json");

        save_workspace_file(&workspace_file).expect("save multi-project workspace file");
        let raw = fs::read_to_string(&workspace_file).expect("read saved workspace file");
        reset_active_workspace_for_test();
        let reopened =
            open_workspace_file(&workspace_file).expect("open multi-project workspace file");

        assert_eq!(reopened.descriptor.id, second.descriptor.id);
        assert_eq!(reopened.payload.workspace.project, second.descriptor.name);
        assert_eq!(reopened.payload.projects.len(), 2);
        assert!(raw.contains(&first.descriptor.root_path));
        assert!(raw.contains(&second.descriptor.root_path));
        assert!(reopened
            .payload
            .projects
            .iter()
            .any(|project| project.root_path == first.descriptor.root_path));
        assert!(reopened
            .payload
            .projects
            .iter()
            .any(|project| project.root_path == second.descriptor.root_path));

        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }

    #[test]
    fn task_013a_workspace_file_resolves_relative_roots_and_generates_ids() {
        reset_active_workspace_for_test();
        let workspace_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .join("tests/projects/workspace.c4os.json");
        let project_a = workspace_file
            .parent()
            .expect("fixture parent")
            .join("project-a")
            .canonicalize()
            .expect("canonical project-a");
        let project_b = workspace_file
            .parent()
            .expect("fixture parent")
            .join("project-b")
            .canonicalize()
            .expect("canonical project-b");

        let activation = open_workspace_file(&workspace_file).expect("open relative fixture");
        let recents = recent_project_records();

        assert_eq!(activation.descriptor.root_path, project_a.to_string_lossy());
        assert_eq!(activation.descriptor.id, workspace_id(&project_a));
        assert_eq!(activation.payload.projects.len(), 2);
        assert!(activation
            .payload
            .projects
            .iter()
            .any(|project| project.root_path == project_b.to_string_lossy()));
        assert_eq!(recents[0].name, "workspace");
        assert_eq!(
            recents[0].workspace_file_path,
            workspace_file.to_string_lossy()
        );
    }

    #[test]
    fn task_013a_recent_workspace_projects_persist_without_active_workspace() {
        let store_path = std::env::temp_dir().join("c4os-task-013a-recents-store.json");
        let first_root = test_workspace_root("recent-first-project");
        let second_root = test_workspace_root("recent-second-project");
        let _ = fs::remove_file(&store_path);
        fs::create_dir_all(&first_root).expect("create first temp workspace");
        fs::create_dir_all(&second_root).expect("create second temp workspace");
        std::env::set_var("C4OS_WORKSPACE_RECENTS_STORE", &store_path);
        reset_active_workspace_for_test();

        let first = activate_workspace_at(&first_root).expect("activate first workspace");
        let second = activate_workspace_at(&second_root).expect("activate second workspace");
        reset_active_workspace_for_test();
        let recents = recent_project_records();

        assert_eq!(recents.len(), 2);
        assert!(active_workspace_descriptor().is_none());
        assert_eq!(recents[0].root_path, second.descriptor.root_path);
        assert!(recents
            .iter()
            .any(|project| project.root_path == first.descriptor.root_path));

        std::env::remove_var("C4OS_WORKSPACE_RECENTS_STORE");
        let _ = fs::remove_file(store_path);
        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }

    #[test]
    fn task_013a_ignores_ambient_workspace_environment_without_launch_flag() {
        let root = test_workspace_root("env-ignored");
        fs::create_dir_all(&root).expect("create temp workspace");
        reset_active_workspace_for_test();
        std::env::set_var("C4OS_WORKSPACE_PATH", root.to_string_lossy().to_string());
        std::env::remove_var("C4OS_WORKSPACE_FILE");

        let activation = bootstrap_workspace_from_launch_flag().expect("bootstrap result");

        assert!(activation.is_none());
        assert!(active_workspace_descriptor().is_none());

        std::env::remove_var("C4OS_WORKSPACE_PATH");
        let _ = fs::remove_dir_all(root);
    }

    fn test_workspace_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("c4os-task-004-{name}"));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
