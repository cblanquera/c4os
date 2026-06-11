use crate::action_classifier::{ActionClassifier, ActionDecision, RiskLevel, RuntimeAction};
use crate::storage::AppStore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct McpCapabilityStatus {
    pub support_tier: String,
    pub local_stdio_available: bool,
    pub remote_available: bool,
    pub explicit_registration_required: bool,
    pub auto_start_from_project_files: bool,
    pub approval_required_for_launch: bool,
    pub roots_limited_to_selected_project: bool,
    pub tools_require_approval: bool,
    pub resources_auto_context: bool,
    pub prompts_auto_context: bool,
    pub sampling_auto_approved: bool,
    pub elicitation_auto_disclosure: bool,
    pub full_compatibility_claim: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalStdioMcpRegistration {
    pub name: String,
    pub transport: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub remote_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalStdioMcpServer {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub root_scope: PathBuf,
    pub auto_start: bool,
    pub remote_url: Option<String>,
    pub resources_auto_context: bool,
    pub prompts_auto_context: bool,
    pub sampling_auto_approved: bool,
    pub elicitation_auto_disclosure: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct McpLaunchProposal {
    pub server_id: String,
    pub command: String,
    pub working_directory: PathBuf,
    pub root_scope: PathBuf,
    pub requires_approval: bool,
    pub risk_level: String,
    pub approval_action_type: String,
    pub filtered_environment: bool,
    pub starts_process: bool,
    pub unsupported_capabilities: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum McpRegistryError {
    MissingProject,
    RemoteUnavailable,
    InvalidTransport,
    InvalidCommand,
    SecretDenied,
    OutsideProjectRoot,
    StoreFailed,
}

pub struct LocalStdioMcpRegistry<'a> {
    store: &'a AppStore,
    project_id: String,
    project_root: PathBuf,
}

impl<'a> LocalStdioMcpRegistry<'a> {
    pub fn new(store: &'a AppStore, project_id: &str) -> Result<Self, McpRegistryError> {
        let project = store
            .get_project(project_id)
            .map_err(|_| McpRegistryError::StoreFailed)?
            .ok_or(McpRegistryError::MissingProject)?;
        let project_root = std::fs::canonicalize(project.root_path)
            .map_err(|_| McpRegistryError::OutsideProjectRoot)?;

        Ok(Self {
            store,
            project_id: project_id.into(),
            project_root,
        })
    }

    pub fn list(&self) -> Result<Vec<LocalStdioMcpServer>, McpRegistryError> {
        let Some(value) = self
            .store
            .read_setting(&self.setting_key())
            .map_err(|_| McpRegistryError::StoreFailed)?
        else {
            return Ok(Vec::new());
        };

        serde_json::from_str(&value).map_err(|_| McpRegistryError::StoreFailed)
    }

    pub fn register(
        &self,
        registration: LocalStdioMcpRegistration,
    ) -> Result<LocalStdioMcpServer, McpRegistryError> {
        if registration.remote_url.is_some() {
            return Err(McpRegistryError::RemoteUnavailable);
        }

        if registration.transport != "stdio" {
            return Err(McpRegistryError::InvalidTransport);
        }

        if registration.command.trim().is_empty() {
            return Err(McpRegistryError::InvalidCommand);
        }

        if contains_secret_material(&registration.command)
            || registration
                .args
                .iter()
                .any(|arg| contains_secret_material(arg))
        {
            return Err(McpRegistryError::SecretDenied);
        }

        let working_directory =
            resolve_working_directory(&self.project_root, &registration.working_directory)?;
        let server = LocalStdioMcpServer {
            id: format!("mcp-{}", stable_id(&registration.name)),
            name: registration.name,
            transport: "stdio".into(),
            command: registration.command,
            args: registration.args,
            working_directory,
            root_scope: self.project_root.clone(),
            auto_start: false,
            remote_url: None,
            resources_auto_context: false,
            prompts_auto_context: false,
            sampling_auto_approved: false,
            elicitation_auto_disclosure: false,
        };
        let mut servers = self.list()?;

        servers.retain(|existing| existing.id != server.id);
        servers.push(server.clone());
        self.store
            .set_setting(
                &self.setting_key(),
                &serde_json::to_string(&servers).map_err(|_| McpRegistryError::StoreFailed)?,
            )
            .map_err(|_| McpRegistryError::StoreFailed)?;

        Ok(server)
    }

    pub fn launch_proposal(&self, server_id: &str) -> Result<McpLaunchProposal, McpRegistryError> {
        let server = self
            .list()?
            .into_iter()
            .find(|server| server.id == server_id)
            .ok_or(McpRegistryError::MissingProject)?;
        let command = command_line(&server.command, &server.args);
        let classifier = ActionClassifier::new(&self.project_root)
            .map_err(|_| McpRegistryError::OutsideProjectRoot)?;
        let decision = classifier.classify(RuntimeAction::ShellCommand {
            command: &command,
            working_directory: &server.working_directory,
        });
        let risk_level = match decision {
            ActionDecision::RequiresApproval { risk } => risk_label(&risk).into(),
            ActionDecision::AllowLogged => "medium".into(),
            ActionDecision::Blocked { .. } => return Err(McpRegistryError::OutsideProjectRoot),
        };

        Ok(McpLaunchProposal {
            server_id: server.id,
            command,
            working_directory: server.working_directory,
            root_scope: server.root_scope,
            requires_approval: true,
            risk_level,
            approval_action_type: "mcp_stdio_launch".into(),
            filtered_environment: true,
            starts_process: false,
            unsupported_capabilities: unsupported_capabilities(),
        })
    }

    fn setting_key(&self) -> String {
        format!("mcp.local_stdio.servers.{}", self.project_id)
    }
}

pub fn mcp_capability_status() -> McpCapabilityStatus {
    McpCapabilityStatus {
        support_tier: "local_stdio_explicit_approval_only".into(),
        local_stdio_available: true,
        remote_available: false,
        explicit_registration_required: true,
        auto_start_from_project_files: false,
        approval_required_for_launch: true,
        roots_limited_to_selected_project: true,
        tools_require_approval: true,
        resources_auto_context: false,
        prompts_auto_context: false,
        sampling_auto_approved: false,
        elicitation_auto_disclosure: false,
        full_compatibility_claim: false,
    }
}

pub fn unsupported_capabilities() -> Vec<String> {
    vec![
        "remote_mcp_unavailable".into(),
        "authorization_flows_unavailable".into(),
        "automatic_project_file_startup_unavailable".into(),
        "automatic_resource_context_unavailable".into(),
        "automatic_prompt_use_unavailable".into(),
        "unapproved_tool_invocation_unavailable".into(),
        "automatic_sampling_unavailable".into(),
        "automatic_elicitation_disclosure_unavailable".into(),
        "full_mcp_compatibility_unavailable".into(),
    ]
}

fn resolve_working_directory(
    project_root: &Path,
    working_directory: &str,
) -> Result<PathBuf, McpRegistryError> {
    let requested = Path::new(working_directory);
    let joined = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        project_root.join(requested)
    };
    let resolved =
        std::fs::canonicalize(joined).map_err(|_| McpRegistryError::OutsideProjectRoot)?;

    if !resolved.starts_with(project_root) {
        return Err(McpRegistryError::OutsideProjectRoot);
    }

    Ok(resolved)
}

fn command_line(command: &str, args: &[String]) -> String {
    std::iter::once(command.to_string())
        .chain(args.iter().map(|arg| shell_display(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_display(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:@".contains(character))
    {
        value.into()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn stable_id(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

fn risk_label(risk: &RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    }
}

fn contains_secret_material(value: &str) -> bool {
    let lower = value.to_lowercase();

    lower.contains("sk-or-")
        || lower.contains("sk-")
        || lower.contains("token=")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("bearer ")
        || lower.contains("-----begin")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn registers_explicit_local_stdio_server_for_selected_project_root() {
        let directory = tempdir().expect("tempdir");
        let store = prepared_store(directory.path());
        let registry = LocalStdioMcpRegistry::new(&store, "project-1").expect("registry");

        let server = registry
            .register(LocalStdioMcpRegistration {
                name: "filesystem".into(),
                transport: "stdio".into(),
                command: "node".into(),
                args: vec!["server.js".into()],
                working_directory: ".".into(),
                remote_url: None,
            })
            .expect("server registered");
        let servers = registry.list().expect("servers listed");

        assert_eq!(server.transport, "stdio");
        assert_eq!(
            server.root_scope,
            directory.path().canonicalize().expect("canonical")
        );
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "filesystem");
        assert!(!servers[0].auto_start);
    }

    #[test]
    fn launch_request_returns_approval_proposal_without_starting_process() {
        let directory = tempdir().expect("tempdir");
        let store = prepared_store(directory.path());
        let registry = LocalStdioMcpRegistry::new(&store, "project-1").expect("registry");
        registry
            .register(LocalStdioMcpRegistration {
                name: "filesystem".into(),
                transport: "stdio".into(),
                command: "node".into(),
                args: vec!["server.js".into()],
                working_directory: ".".into(),
                remote_url: None,
            })
            .expect("server registered");

        let proposal = registry
            .launch_proposal("mcp-filesystem")
            .expect("launch proposal");

        assert_eq!(proposal.server_id, "mcp-filesystem");
        assert_eq!(proposal.command, "node server.js");
        assert!(proposal.requires_approval);
        assert!(!proposal.starts_process);
        assert_eq!(proposal.approval_action_type, "mcp_stdio_launch");
        assert_eq!(
            proposal.root_scope,
            directory.path().canonicalize().expect("canonical")
        );
        assert!(proposal.filtered_environment);
    }

    #[test]
    fn project_mcp_files_are_not_auto_imported_or_auto_started() {
        let directory = tempdir().expect("tempdir");
        fs::write(
            directory.path().join(".mcp.json"),
            r#"{"servers":{"auto":{"command":"node","args":["server.js"]}}}"#,
        )
        .expect("mcp file written");
        let store = prepared_store(directory.path());
        let registry = LocalStdioMcpRegistry::new(&store, "project-1").expect("registry");

        let servers = registry.list().expect("servers listed");

        assert!(servers.is_empty());
    }

    #[test]
    fn remote_mcp_registration_is_rejected() {
        let directory = tempdir().expect("tempdir");
        let store = prepared_store(directory.path());
        let registry = LocalStdioMcpRegistry::new(&store, "project-1").expect("registry");

        let result = registry.register(LocalStdioMcpRegistration {
            name: "remote".into(),
            transport: "http".into(),
            command: "remote".into(),
            args: vec![],
            working_directory: ".".into(),
            remote_url: Some("https://example.com/mcp".into()),
        });

        assert_eq!(result, Err(McpRegistryError::RemoteUnavailable));
    }

    #[test]
    fn capabilities_keep_context_sampling_and_elicitation_automatic_paths_disabled() {
        let capabilities = mcp_capability_status();

        assert_eq!(
            capabilities.support_tier,
            "local_stdio_explicit_approval_only"
        );
        assert!(capabilities.local_stdio_available);
        assert!(!capabilities.remote_available);
        assert!(!capabilities.auto_start_from_project_files);
        assert!(!capabilities.resources_auto_context);
        assert!(!capabilities.prompts_auto_context);
        assert!(!capabilities.sampling_auto_approved);
        assert!(!capabilities.elicitation_auto_disclosure);
    }

    fn prepared_store(project_root: &std::path::Path) -> AppStore {
        let store = AppStore::open_in_memory().expect("store opens");

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: project_root.to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");

        store
    }
}
