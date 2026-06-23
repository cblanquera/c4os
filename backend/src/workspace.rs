use crate::mock_data::{
    mock_workspace, ProjectRecord, ThreadState, WorkspacePayload, WorkspaceSummary,
};
use crate::provider_models::DEFAULT_MODEL;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDescriptor {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub trusted: bool,
    pub created_at: u64,
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

static ACTIVE_WORKSPACE: OnceLock<Mutex<Option<WorkspaceDescriptor>>> = OnceLock::new();

pub fn activate_workspace(path: Option<String>) -> Result<WorkspaceActivation, String> {
    let root = match path {
        Some(input) if !input.trim().is_empty() => PathBuf::from(input),
        _ => choose_workspace_folder()?,
    };

    activate_workspace_at(root)
}

pub fn activate_workspace_at(root: impl AsRef<Path>) -> Result<WorkspaceActivation, String> {
    let root = root.as_ref();
    let canonical =
        fs::canonicalize(root).map_err(|error| format!("Cannot open workspace folder: {error}"))?;

    if !canonical.is_dir() {
        return Err("Workspace path must be a folder".into());
    }

    let descriptor = load_or_create_descriptor(&canonical)?;
    let payload = payload_for_descriptor(&descriptor);
    set_active_workspace(descriptor.clone());

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
        let output = Command::new("osascript")
            .args([
                "-e",
                r#"POSIX path of (choose folder with prompt "Open C4OS Workspace Folder")"#,
            ])
            .output()
            .map_err(|error| format!("Cannot open workspace folder chooser: {error}"))?;

        if !output.status.success() {
            return Err("No workspace folder selected".into());
        }

        let path = String::from_utf8(output.stdout)
            .map_err(|error| format!("Workspace folder chooser returned invalid path: {error}"))?;
        let path = path.trim();
        if path.is_empty() {
            return Err("No workspace folder selected".into());
        }
        return Ok(PathBuf::from(path));
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Workspace folder chooser is not available on this platform yet".into())
    }
}

fn write_descriptor(path: &Path, descriptor: &WorkspaceDescriptor) -> Result<(), String> {
    let json = serde_json::to_string_pretty(descriptor)
        .map_err(|error| format!("Cannot serialize workspace descriptor: {error}"))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("Cannot write workspace descriptor: {error}"))
}

fn descriptor_path(root: &Path) -> PathBuf {
    root.join(".c4os").join("workspace.json")
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
    payload.projects = vec![ProjectRecord {
        name: descriptor.name.clone(),
        sessions: vec![],
    }];
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

fn active_workspace() -> &'static Mutex<Option<WorkspaceDescriptor>> {
    ACTIVE_WORKSPACE.get_or_init(|| Mutex::new(None))
}

fn workspace_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Local Workspace")
        .to_string()
}

fn workspace_id(root: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    root.to_string_lossy().hash(&mut hasher);
    format!("c4os-ws-{:016x}", hasher.finish())
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

    fn test_workspace_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("c4os-task-004-{name}"));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
