use crate::runtime_sessions::{C4osSessionRecord, ProjectSessionRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePayload {
    pub workspace: WorkspaceSummary,
    pub projects: Vec<ProjectRecord>,
    pub providers: Vec<ProviderRecord>,
    pub models: Vec<ModelRecord>,
    pub plugin_catalog: Vec<String>,
    pub plugin_marketplaces: Vec<PluginMarketplace>,
    pub skill_catalog: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub browser: BrowserState,
    pub artifacts: Vec<ArtifactRecord>,
    pub files: FilesState,
    pub terminal: TerminalState,
    pub thread: ThreadState,
    pub sessions: Vec<C4osSessionRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub project: String,
    pub session: String,
    pub branch: String,
    pub model: String,
    #[serde(rename = "sessionId", default)]
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub name: String,
    pub sessions: Vec<ProjectSessionRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub id: String,
    pub label: String,
    pub kind: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub endpoint: String,
    pub status: String,
    #[serde(rename = "keyStatus")]
    pub key_status: ApiKeyStatus,
    pub enabled: bool,
    #[serde(rename = "supportsDiscovery")]
    pub supports_discovery: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRecord {
    pub id: String,
    pub label: String,
    #[serde(rename = "providerId")]
    pub provider_id: String,
    pub provider: String,
    pub enabled: bool,
    pub active: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyStatus {
    pub state: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginMarketplace {
    pub label: String,
    pub summary: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowserState {
    pub url: String,
    pub title: String,
    pub summary: String,
    #[serde(rename = "artifactId", default)]
    pub artifact_id: String,
    #[serde(rename = "previewMode", default)]
    pub preview_mode: String,
    #[serde(rename = "profileId", default)]
    pub profile_id: String,
    #[serde(rename = "localPath", default)]
    pub local_path: String,
    #[serde(default)]
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRecord {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub origin: String,
    pub safe_preview: SafeArtifactPreview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafeArtifactPreview {
    pub url: String,
    pub title: String,
    pub summary: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesState {
    pub roots: Vec<Vec<String>>,
    pub breadcrumbs: Vec<String>,
    pub lines: Vec<String>,
    #[serde(default)]
    pub current_path: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub saved_content: String,
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalState {
    pub output: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadState {
    pub user: String,
    pub agent: String,
    pub extra: String,
    pub tool: String,
    pub run: String,
}

pub fn mock_workspace() -> WorkspacePayload {
    WorkspacePayload {
        workspace: WorkspaceSummary {
            project: "Mock Workspace Alpha".into(),
            session: "Mock integration run".into(),
            branch: "mock/task-002".into(),
            model: "".into(),
            session_id: "".into(),
        },
        projects: vec![
            ProjectRecord {
                name: "Mock Workspace Alpha".into(),
                sessions: vec![
                    ProjectSessionRecord {
                        id: "mock-integration-run".into(),
                        label: "Mock integration run".into(),
                    },
                    ProjectSessionRecord {
                        id: "mock-review-session".into(),
                        label: "Mock review session".into(),
                    },
                ],
            },
            ProjectRecord {
                name: "Mock Agent Lab".into(),
                sessions: vec![ProjectSessionRecord {
                    id: "harness-rehearsal".into(),
                    label: "Harness rehearsal".into(),
                }],
            },
            ProjectRecord {
                name: "Mock Docs Workbench".into(),
                sessions: vec![],
            },
        ],
        providers: vec![],
        models: vec![],
        plugin_catalog: vec!["Mock GitHub".into(), "Mock Slack".into(), "Mock Linear".into()],
        plugin_marketplaces: vec![PluginMarketplace {
            label: "Mock C4OS Marketplace".into(),
            summary: "Mock configured marketplace".into(),
            active: true,
        }],
        skill_catalog: vec![
            "Mock Skill Routing".into(),
            "Mock Code Review".into(),
            "Mock Browser QA".into(),
        ],
        mcp_servers: vec!["mock_node_repl".into(), "mock_docs_mcp".into()],
        browser: BrowserState {
            url: "http://127.0.0.1:43123/mock-preview".into(),
            title: "Mock rendered page".into(),
            summary: "Mock Browser state from tests/server.".into(),
            artifact_id: String::new(),
            preview_mode: String::new(),
            profile_id: String::new(),
            local_path: String::new(),
            html: String::new(),
        },
        artifacts: vec![],
        files: FilesState {
            roots: vec![
                vec!["backend".into(), "folder".into(), "file-explorer".into()],
                vec!["frontend".into(), "folder".into(), "file-explorer".into()],
                vec!["mock-main.js".into(), "file".into(), "file-editor".into()],
                vec!["mock-index.html".into(), "file".into(), "file-editor".into()],
                vec!["tests".into(), "folder".into(), "file-explorer".into()],
            ],
            breadcrumbs: vec!["Mock Workspace Alpha".into(), "frontend".into(), "mock-main.js".into()],
            lines: vec![
                "import { startMockWorkspace } from './mock-runtime';".into(),
                "".into(),
                "const trustedRoot = 'mocked';".into(),
                "".into(),
                "startMockWorkspace({ trustedRoot });".into(),
            ],
            current_path: "frontend/mock-main.js".into(),
            content: "import { startMockWorkspace } from './mock-runtime';\n\nconst trustedRoot = 'mocked';\n\nstartMockWorkspace({ trustedRoot });".into(),
            saved_content: "import { startMockWorkspace } from './mock-runtime';\n\nconst trustedRoot = 'mocked';\n\nstartMockWorkspace({ trustedRoot });".into(),
            dirty: false,
            status: "Mock file content".into(),
        },
        terminal: TerminalState {
            output: "$ npm run mock:task-002\nmock server ready\nfake agent run channel connected".into(),
            title: "Mock command preview/results".into(),
            summary: "Read-only mock terminal output. No command execution is implied.".into(),
        },
        thread: ThreadState {
            user: "Confirm the mock server harness state.".into(),
            agent: "The TASK-002 mock harness is connected behind the accepted r04 surface.".into(),
            extra: "This state is returned by tests/server and remains fake for workspace trust, provider/model records, session creation, agent processing, files, Browser, Terminal, extensions, approvals, memory, actions, descriptors, and persistence.".into(),
            tool: "Mock project instructions loaded".into(),
            run: "Mock agent ready".into(),
        },
        sessions: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_task_002_workspace_payload_shape() {
        let payload = serde_json::to_value(mock_workspace()).expect("serialize mock workspace");

        assert_eq!(payload["workspace"]["project"], "Mock Workspace Alpha");
        assert_eq!(payload["workspace"]["branch"], "mock/task-002");
        assert_eq!(
            payload["pluginCatalog"],
            json!(["Mock GitHub", "Mock Slack", "Mock Linear"])
        );
        assert_eq!(
            payload["browser"]["summary"],
            "Mock Browser state from tests/server."
        );
        assert_eq!(payload["thread"]["run"], "Mock agent ready");
    }
}
