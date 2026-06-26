use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRecord {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub summary: String,
    pub provenance: String,
    pub scopes: Vec<String>,
    pub workspace_scope: String,
    pub project_scope: String,
    pub shared_data: Vec<String>,
    pub runtime_access: String,
    pub tool_access: Vec<String>,
    pub enabled: bool,
    pub audit: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionList {
    pub plugins: Vec<ExtensionRecord>,
    pub skills: Vec<ExtensionRecord>,
    pub mcp_servers: Vec<ExtensionRecord>,
}

pub fn extension_records() -> ExtensionList {
    ExtensionList {
        plugins: vec![ExtensionRecord {
            id: "github".into(),
            kind: "plugin".into(),
            label: "GitHub".into(),
            summary: "Review pull request context from the connected workspace.".into(),
            provenance: "Installed from Built by C4OS".into(),
            scopes: vec!["repo:read".into(), "pull_request:read".into()],
            workspace_scope: "workspace".into(),
            project_scope: "project".into(),
            shared_data: vec!["workspace metadata".into(), "selected transcript".into()],
            runtime_access: "disabled".into(),
            tool_access: vec!["review_threads".into()],
            enabled: false,
            audit: vec!["Installed by workspace owner".into()],
        }],
        skills: vec![ExtensionRecord {
            id: "chrisai-agents".into(),
            kind: "skill".into(),
            label: "ChrisAI Agents".into(),
            summary: "Project-local code review workflow guidance.".into(),
            provenance: "Installed from project .agents skills".into(),
            scopes: vec!["project".into()],
            workspace_scope: "workspace".into(),
            project_scope: "project".into(),
            shared_data: vec!["current transcript".into(), "selected files".into()],
            runtime_access: "disabled".into(),
            tool_access: vec![],
            enabled: false,
            audit: vec!["Enabled state reviewed".into()],
        }],
        mcp_servers: vec![ExtensionRecord {
            id: "docs-mcp".into(),
            kind: "mcp".into(),
            label: "Docs MCP".into(),
            summary: "Workspace documentation lookup over an explicit MCP connection.".into(),
            provenance: "Connected from workspace settings".into(),
            scopes: vec!["project".into()],
            workspace_scope: "workspace".into(),
            project_scope: "project".into(),
            shared_data: vec!["workspace metadata".into()],
            runtime_access: "disabled".into(),
            tool_access: vec!["read_docs".into()],
            enabled: false,
            audit: vec!["Connection recorded".into()],
        }],
    }
}

pub fn plugin_records() -> Vec<ExtensionRecord> {
    extension_records().plugins
}

pub fn skill_records() -> Vec<ExtensionRecord> {
    extension_records().skills
}

pub fn mcp_server_records() -> Vec<ExtensionRecord> {
    extension_records().mcp_servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_012_extension_records_are_inert_until_explicit_enablement() {
        let records = extension_records();
        let all = records
            .plugins
            .iter()
            .chain(records.skills.iter())
            .chain(records.mcp_servers.iter());

        for record in all {
            assert_eq!(record.enabled, false);
            assert_eq!(record.runtime_access, "disabled");
            assert!(!record.provenance.is_empty());
            assert!(!record.scopes.is_empty());
            assert!(!record.workspace_scope.is_empty());
            assert!(!record.project_scope.is_empty());
            assert!(!record.audit.is_empty());
        }
    }
}
