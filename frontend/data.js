// Route ids are the public hash contract covered by the r04 browser tests.
export const routes = new Set([
  "app-start",
  "new-session",
  "chat-session",
  "providers-popover",
  "models-popover",
  "file-explorer",
  "file-editor",
  "terminal",
  "settings-providers",
  "settings-add-provider",
  "settings-models",
  "settings-runtimes",
  "settings-configuration",
  "settings-plugins",
  "settings-skills",
  "settings-mcp"
]);

// Frontend-local workspace state keeps route rendering realistic until TASK-002
// replaces fixture records with mock-server data.
export const workspace = {
  project: "Project Alpha",
  session: "Integration planning",
  branch: "main",
  model: "fast-coder"
};

// Sidebar project/session fixtures preserve the accepted r04 information shape.
export const projects = [
  { name: "Project Alpha", sessions: ["Integration planning", "UI handoff review"] },
  { name: "Runtime Lab", sessions: [] },
  { name: "Docs Workbench", sessions: [] },
  { name: "Tech Ops", sessions: [] },
  { name: "Agent Studio", sessions: [] }
];

// Provider rows are display fixtures only; credentials are not persisted here.
export const providers = [
  { label: "OpenRouter - Personal", endpoint: "https://openrouter.ai/api/v1", status: "API key saved" },
  { label: "OpenAI - Work", endpoint: "https://api.openai.com/v1", status: "API key saved" },
  { label: "LiteLLM Local", endpoint: "http://localhost:4000/v1", status: "API key saved" }
];

// Model rows mirror the provider/model picker routes without making live calls.
export const models = [
  { label: "Gemini 2.5 Flash", provider: "OpenRouter - Personal", active: true },
  { label: "ChatGPT o4", provider: "OpenRouter - Personal" },
  { label: "Grok 2.0", provider: "OpenRouter - Personal" },
  { label: "gpt-4.1", provider: "OpenAI - Work" },
  { label: "local/coder-small", provider: "LiteLLM Local" }
];

// Settings navigation order is an r04 parity surface, not generated metadata.
export const settingsItems = [
  ["Providers", "settings-providers", "key"],
  ["Models", "settings-models", "bot"],
  ["Runtimes", "settings-runtimes", "terminal"],
  ["Configuration", "settings-configuration", "settings"],
  ["Plugins", "settings-plugins", "plug", true],
  ["Skills", "settings-skills", "file"],
  ["MCP Servers", "settings-mcp", "server"]
];

// Plugin directory rows are local catalog fixtures for visual review only.
export const pluginCatalog = [
  "GitHub",
  "Slack",
  "Notion",
  "Linear",
  "Gmail",
  "Google Calendar",
  "Google Drive",
  "Figma",
  "Vercel"
];

// Marketplaces are selectable plugin-directory sources. Built by C4OS is the
// default source until TASK-002 wires configured marketplace records.
export const pluginMarketplaces = [
  { label: "Built by C4OS", summary: "Default marketplace", active: true }
];

// Skill rows are fixture records for the r04 Skills settings route.
export const skillCatalog = [
  "Ab Testing",
  "Ad Creative",
  "Ads",
  "Ai Seo",
  "Analytics",
  "Aso",
  "ChrisAI Agents"
];

// MCP rows are fixture records for the r04 MCP Servers settings route.
export const mcpServers = [
  "node_repl",
  "openaiDeveloperDocs",
  "stackpress_blog_mcp"
];
