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
    pub files: FilesState,
    pub terminal: TerminalState,
    pub thread: ThreadState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub project: String,
    pub session: String,
    pub branch: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub name: String,
    pub sessions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub label: String,
    pub endpoint: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRecord {
    pub label: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesState {
    pub roots: Vec<[String; 3]>,
    pub breadcrumbs: Vec<String>,
    pub lines: Vec<String>,
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
            model: "mock-fast-coder".into(),
        },
        projects: vec![
            ProjectRecord {
                name: "Mock Workspace Alpha".into(),
                sessions: vec!["Mock integration run".into(), "Mock review session".into()],
            },
            ProjectRecord {
                name: "Mock Agent Lab".into(),
                sessions: vec!["Harness rehearsal".into()],
            },
            ProjectRecord {
                name: "Mock Docs Workbench".into(),
                sessions: vec![],
            },
        ],
        providers: vec![
            ProviderRecord {
                label: "Mock OpenRouter".into(),
                endpoint: "https://mock.openrouter.local/v1".into(),
                status: "Mock API key saved".into(),
            },
            ProviderRecord {
                label: "Mock OpenAI".into(),
                endpoint: "https://mock.openai.local/v1".into(),
                status: "Mock API key saved".into(),
            },
        ],
        models: vec![
            ModelRecord {
                label: "mock-fast-coder".into(),
                provider: "Mock OpenRouter".into(),
                active: Some(true),
            },
            ModelRecord {
                label: "mock-reviewer".into(),
                provider: "Mock OpenRouter".into(),
                active: None,
            },
            ModelRecord {
                label: "mock-local-small".into(),
                provider: "Mock OpenAI".into(),
                active: None,
            },
        ],
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
        },
        files: FilesState {
            roots: vec![
                ["backend".into(), "folder".into(), "file-explorer".into()],
                ["frontend".into(), "folder".into(), "file-explorer".into()],
                ["mock-main.js".into(), "file".into(), "file-editor".into()],
                ["mock-index.html".into(), "file".into(), "file-editor".into()],
                ["tests".into(), "folder".into(), "file-explorer".into()],
            ],
            breadcrumbs: vec!["Mock Workspace Alpha".into(), "frontend".into(), "mock-main.js".into()],
            lines: vec![
                "import { startMockWorkspace } from './mock-runtime';".into(),
                "".into(),
                "const trustedRoot = 'mocked';".into(),
                "".into(),
                "startMockWorkspace({ trustedRoot });".into(),
            ],
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
        assert_eq!(payload["pluginCatalog"], json!(["Mock GitHub", "Mock Slack", "Mock Linear"]));
        assert_eq!(payload["browser"]["summary"], "Mock Browser state from tests/server.");
        assert_eq!(payload["thread"]["run"], "Mock agent ready");
    }
}
