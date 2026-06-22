// Route ids are the public hash contract covered by the r04 browser tests.
import { createConnector } from "./connectors.js";

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

// Frontend fallback state keeps routes renderable before a connector hydrates
// the shell from server or native app state.
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

export const browserState = {
  url: "http://127.0.0.1:13000",
  title: "Rendered page mock",
  summary: "Single browser surface; tabs are out of scope. This is frontend-local fixture state."
};

export const filesState = {
  roots: [
    ["backend", "folder", "file-explorer"],
    ["frontend", "folder", "file-explorer"],
    ["main.js", "file", "file-editor"],
    ["index.html", "file", "file-editor"],
    ["tests", "folder", "file-explorer"]
  ],
  breadcrumbs: ["Project Alpha", "frontend", "main.js"],
  lines: [
    "import { startWorkspace } from './runtime';",
    "",
    "const trustedRoot = true;",
    "",
    "startWorkspace({ trustedRoot });"
  ]
};

export const terminalState = {
  output: "$ npm run dev\nready in 614ms\nlocal preview available at 127.0.0.1:3000",
  title: "AI command preview/results",
  summary: "Read-only frontend fixture panel. No command execution is implied."
};

export const threadState = {
  user: "Locate the desktop integration path and confirm the trusted project state.",
  agent: "The shell boundary is visible. The next step is to connect state without changing the r04 surface.",
  extra: "This fixture is frontend-local and does not imply backend, filesystem, terminal, browser, approval, or persistence behavior.",
  tool: "Read project instructions",
  run: "Sensitive action waiting"
};

export const threadTurns = [turnFromThreadState(threadState, "fixture-turn")];

export const connectorState = {
  connector: createConnector(),
  delay: "0",
  error: null,
  loading: false,
  runPending: false,
  scenario: "success"
};

export const toolState = {
  bySurface: {}
};

function replaceArray(target, next) {
  target.splice(0, target.length, ...next);
}

function assignObject(target, next) {
  Object.keys(target).forEach((key) => {
    if (!(key in next)) delete target[key];
  });
  Object.assign(target, next);
}

export function beginConnectorStateLoad() {
  connectorState.error = null;

  if (!connectorState.connector.available) return null;

  connectorState.loading = true;
  return connectorState.connector.loadWorkspace()
    .then((payload) => {
      applyConnectorWorkspace(payload);
    })
    .catch((error) => {
      connectorState.error = error.message;
    })
    .finally(() => {
      connectorState.loading = false;
    });
}

export async function openConnectorWorkspace(path) {
  if (!connectorState.connector.available || !connectorState.connector.openWorkspace) return;

  connectorState.error = null;
  connectorState.loading = true;

  try {
    const response = await connectorState.connector.openWorkspace(path);
    const payload = response.payload || response;
    applyConnectorWorkspace(payload);
    return payload;
  } catch (error) {
    connectorState.error = error.message;
    throw error;
  } finally {
    connectorState.loading = false;
  }
}

export async function sendConnectorPrompt(prompt, options = {}) {
  if (!connectorState.connector.available || connectorState.runPending) return;

  connectorState.runPending = true;
  const activeTurn = createPendingTurn(prompt || threadState.user);
  threadTurns.push(activeTurn);
  syncThreadStateFromTurn(activeTurn);
  options.onTurnCreated?.(activeTurn);
  if (options.createSession) ensureSessionForPrompt(prompt);

  try {
    if (options.createSession && connectorState.connector.createSession) {
      await connectorState.connector.createSession(workspace.project);
    }
    const payload = await connectorState.connector.sendPrompt(prompt, {
      onEvent: (event) => {
        applyRuntimeEvent(event, activeTurn);
        syncThreadStateFromTurn(activeTurn);
        options.onStateChange?.(activeTurn);
      }
    });
    applyRuntimeEventsFromPayload(payload, activeTurn, options);
    if (payload.agent) activeTurn.agent = payload.agent;
    activeTurn.extra = "Connector run events completed successfully. Files, Browser, Terminal, approval, memory, artifacts, audit, and non-chat feature behavior can still be mocked.";
    if (activeTurn.tool === "Thinking") activeTurn.tool = "Thinking complete";
    activeTurn.run = runLabelFromPayload(payload, activeTurn);
  } catch (error) {
    const message = connectorErrorMessage(error);
    activeTurn.agent = "The connector run did not complete.";
    activeTurn.extra = "Connector failure details are shown in the work log.";
    activeTurn.tool = "Thinking stopped";
    activeTurn.run = message;
    appendWorkLog(activeTurn, message);
    activeTurn.failed = true;
    options.onStateChange?.(activeTurn);
  } finally {
    await options.beforeComplete?.(activeTurn);
    activeTurn.pending = false;
    activeTurn.completedAt = Date.now();
    activeTurn.workExpanded = Boolean(activeTurn.failed);
    syncThreadStateFromTurn(activeTurn);
    connectorState.runPending = false;
    options.onStateChange?.(activeTurn);
  }
}

function createPendingTurn(prompt) {
  return {
    id: `turn-${Date.now()}-${threadTurns.length}`,
    user: prompt,
    agent: "Connector is processing the request.",
    extra: "Waiting for a normalized thread/run event from the active connector.",
    tool: "Thinking",
    run: "Waiting on connector",
    logs: ["Waiting on connector"],
    startedAt: Date.now(),
    completedAt: null,
    workExpanded: true,
    permission: null,
    failed: false,
    pending: true
  };
}

function connectorErrorMessage(error) {
  if (typeof error === "string" && error.trim()) return error.trim();
  if (error?.message && String(error.message).trim()) return String(error.message).trim();
  if (error?.error && String(error.error).trim()) return String(error.error).trim();
  return "The connector failed before returning error details. Check the desktop app terminal for the backend error.";
}

function applyRuntimeEvent(event = {}, turn = threadState) {
  const text = String(event.text || "").trim();
  if (!text) return;

  if (event.kind === "reasoning") {
    turn.tool = `Thinking: ${text}`;
    appendWorkLog(turn, text);
    return;
  }

  if (event.kind === "activity") {
    turn.run = text;
    appendWorkLog(turn, text);
    return;
  }

  if (event.kind === "assistant") {
    turn.agent = turn.agent === "Connector is processing the request." || !turn.agent ? text : `${turn.agent}${text}`;
    return;
  }

  if (event.kind === "complete") {
    turn.run = text;
    appendWorkLog(turn, text);
    return;
  }

  if (event.kind === "error") {
    turn.run = text;
    appendWorkLog(turn, text);
    turn.agent = "The connector run did not complete.";
  }
}

function appendWorkLog(turn, text) {
  turn.logs ||= [];
  if (turn.logs[turn.logs.length - 1] !== text) turn.logs.push(text);
}

function applyRuntimeEventsFromPayload(payload = {}, turn = threadState, options = {}) {
  if (!Array.isArray(payload.events)) return;
  for (const event of payload.events) {
    applyRuntimeEvent(event, turn);
    syncThreadStateFromTurn(turn);
    options.onStateChange?.(turn);
  }
}

function runLabelFromPayload(payload = {}, turn = {}) {
  const run = String(payload.run || "").trim();
  if (!run) return "Connector run completed";
  const current = String(turn.run || "").trim();
  if (current && current !== "Waiting on connector" && current !== run) {
    appendWorkLog(turn, run);
    return `${current} ${run}`;
  }
  return run === String(payload.agent || "").trim() ? "Connector run completed" : run;
}

function ensureSessionForPrompt(prompt) {
  const label = sessionLabel(prompt);
  workspace.session = label;
  const project = projects.find((record) => record.name === workspace.project);
  if (!project) {
    projects.unshift({ name: workspace.project, sessions: [label] });
    return;
  }
  project.sessions = [label, ...project.sessions.filter((session) => session !== label)];
}

function sessionLabel(prompt) {
  const text = (prompt || "New chat session").trim().replace(/\s+/g, " ");
  return text.length > 48 ? `${text.slice(0, 45)}...` : text;
}

function applyConnectorWorkspace(payload) {
  assignObject(workspace, payload.workspace);
  replaceArray(projects, payload.projects);
  replaceArray(providers, payload.providers);
  replaceArray(models, payload.models);
  replaceArray(pluginCatalog, payload.pluginCatalog);
  replaceArray(pluginMarketplaces, payload.pluginMarketplaces);
  replaceArray(skillCatalog, payload.skillCatalog);
  replaceArray(mcpServers, payload.mcpServers);
  assignObject(browserState, payload.browser);
  assignObject(filesState, payload.files);
  assignObject(terminalState, payload.terminal);
  assignObject(threadState, payload.thread);
  replaceArray(threadTurns, workspace.session ? [turnFromThreadState(threadState, "connector-turn")] : []);
}

function turnFromThreadState(state, id) {
  return {
    id,
    user: state.user,
    agent: state.agent,
    extra: state.extra,
    tool: state.tool,
    run: state.run,
    logs: [state.tool, state.run].filter(Boolean),
    startedAt: Date.now(),
    completedAt: Date.now(),
    workExpanded: false,
    permission: null,
    failed: false,
    pending: false
  };
}

function syncThreadStateFromTurn(turn) {
  assignObject(threadState, {
    user: turn.user,
    agent: turn.agent,
    extra: turn.extra,
    tool: turn.tool,
    run: turn.run
  });
}
