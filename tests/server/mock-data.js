export function createMockWorkspace() {
  return {
    workspace: {
      project: "Mock Workspace Alpha",
      session: "Mock integration run",
      branch: "mock/task-002",
      model: "mock-fast-coder"
    },
    projects: [
      { name: "Mock Workspace Alpha", sessions: ["Mock integration run", "Mock review session"] },
      { name: "Mock Agent Lab", sessions: ["Harness rehearsal"] },
      { name: "Mock Docs Workbench", sessions: [] }
    ],
    providers: [
      { label: "Mock OpenRouter", endpoint: "https://mock.openrouter.local/v1", status: "Mock API key saved" },
      { label: "Mock OpenAI", endpoint: "https://mock.openai.local/v1", status: "Mock API key saved" }
    ],
    models: [
      { label: "mock-fast-coder", provider: "Mock OpenRouter", active: true },
      { label: "mock-reviewer", provider: "Mock OpenRouter" },
      { label: "mock-local-small", provider: "Mock OpenAI" }
    ],
    pluginCatalog: [
      extensionRecord("plugin", "github", "GitHub", {
        provenance: "Installed from Built by C4OS",
        scopes: ["repo:read", "pull_request:read"],
        toolAccess: ["review_threads"],
        audit: ["Installed by workspace owner"]
      })
    ],
    pluginMarketplaces: [
      { label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }
    ],
    skillCatalog: [
      extensionRecord("skill", "chrisai-agents", "ChrisAI Agents", {
        provenance: "Installed from project .agents skills",
        scopes: ["project"],
        sharedData: ["current transcript", "selected files"],
        audit: ["Enabled state reviewed"]
      })
    ],
    mcpServers: [
      extensionRecord("mcp", "docs-mcp", "Docs MCP", {
        provenance: "Connected from workspace settings",
        scopes: ["project"],
        toolAccess: ["read_docs"],
        audit: ["Connection recorded"]
      })
    ],
    browser: {
      url: "http://127.0.0.1:43123/mock-preview",
      title: "Mock rendered page",
      summary: "Mock Browser state from tests/server."
    },
    artifacts: [],
    files: {
      roots: [
        ["backend", "folder", "file-explorer"],
        ["frontend", "folder", "file-explorer"],
        ["mock-main.js", "file", "file-editor"],
        ["mock-index.html", "file", "file-editor"],
        ["tests", "folder", "file-explorer"]
      ],
      breadcrumbs: ["Mock Workspace Alpha", "frontend", "mock-main.js"],
      lines: [
        "import { startMockWorkspace } from './mock-runtime';",
        "",
        "const trustedRoot = 'mocked';",
        "",
        "startMockWorkspace({ trustedRoot });"
      ]
    },
    terminal: {
      output: "$ npm run mock:task-002\nmock server ready\nfake agent run channel connected",
      title: "Mock command preview/results",
      summary: "Read-only mock terminal output. No command execution is implied."
    },
    thread: {
      user: "Confirm the mock server harness state.",
      agent: "The TASK-002 mock harness is connected behind the accepted r04 surface.",
      extra: "This state is returned by tests/server and remains fake for workspace trust, provider/model records, session creation, agent processing, files, Browser, Terminal, extensions, approvals, memory, actions, descriptors, and persistence.",
      tool: "Mock project instructions loaded",
      run: "Mock agent ready"
    }
  };
}

function extensionRecord(kind, id, label, overrides = {}) {
  return {
    id,
    kind,
    label,
    summary: `${label} app-owned extension record`,
    provenance: overrides.provenance,
    scopes: overrides.scopes || ["project"],
    workspaceScope: "Mock Workspace Alpha",
    projectScope: "Mock Workspace Alpha",
    sharedData: overrides.sharedData || ["workspace metadata"],
    runtimeAccess: "disabled",
    toolAccess: overrides.toolAccess || [],
    enabled: false,
    audit: overrides.audit || ["Record created"]
  };
}
