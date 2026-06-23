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
  model: "",
  sessionId: ""
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
export const providers = [];

// Fallback records keep the r04 routes renderable until a connector hydrates
// provider/model state. They do not carry raw API keys or secret material.
export const models = [];

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
  summary: "Single browser surface; tabs are out of scope. This is frontend-local fixture state.",
  previewMode: "public",
  profileId: "",
  localPath: "",
  html: ""
};

export const artifactState = [];

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
export const persistentSessions = [];

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

export const sessionState = {
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

export async function loadConnectorSession(sessionId) {
  if (!sessionId || !connectorState.connector.available || !connectorState.connector.loadSession) return null;
  const record = await connectorState.connector.loadSession(sessionId);
  applySessionRecord(record);
  captureActiveSessionState();
  return record;
}

export async function sendConnectorPrompt(prompt, options = {}) {
  if (!connectorState.connector.available || connectorState.runPending) return;

  connectorState.runPending = true;
  const activeTurn = createPendingTurn(prompt || threadState.user);

  try {
    if (options.createSession) {
      captureActiveSessionState();
      const label = sessionLabel(prompt);
      const created = connectorState.connector.createSession
        ? await connectorState.connector.createSession(workspace.project, options.model || workspace.model, label)
        : null;
      ensureSessionForPrompt(prompt, created);
      activateSessionState(workspace.project, workspace.session, {
        model: options.model || workspace.model,
        reset: true,
        sessionId: workspace.sessionId,
        skipCapture: true,
        turns: []
      });
    }
    threadTurns.push(activeTurn);
    syncThreadStateFromTurn(activeTurn);
    options.onTurnCreated?.(activeTurn);

    if (options.createSession && connectorState.connector.createSession) {
      // Session creation is intentionally handled before the pending turn so
      // the runtime append targets the backend-owned C4OS session id.
    }
    const payload = await connectorState.connector.sendPrompt(prompt, {
      project: workspace.project,
      sessionId: workspace.sessionId,
      model: options.model || workspace.model,
      onEvent: (event) => {
        applyRuntimeEvent(event, activeTurn);
        syncThreadStateFromTurn(activeTurn);
        options.onStateChange?.(activeTurn);
      }
    });
    if (payload?.session) {
      applySessionRecord(payload.session);
    }
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
    captureActiveSessionState();
  }
}

export async function setConnectorModelEnabled(modelId, enabled) {
  const record = models.find((model) => model.id === modelId);
  if (!record) return null;

  record.enabled = enabled;
  if (!connectorState.connector.available || !connectorState.connector.setModelEnabled) return record;

  const updated = await connectorState.connector.setModelEnabled(modelId, enabled);
  Object.assign(record, updated);
  return record;
}

export async function setConnectorProviderEnabled(providerId, enabled) {
  const record = providers.find((provider) => provider.id === providerId);
  if (!record) return null;

  record.enabled = enabled;
  if (!connectorState.connector.available || !connectorState.connector.setProviderEnabled) return record;

  const updated = await connectorState.connector.setProviderEnabled(providerId, enabled);
  Object.assign(record, updated);
  return record;
}

export async function selectConnectorSessionModel(session, model) {
  workspace.model = model;
  models.forEach((record) => {
    record.active = record.label === model;
  });
  captureActiveSessionState();

  if (!connectorState.connector.available || !connectorState.connector.selectSessionModel) {
    return { session, model };
  }

  return connectorState.connector.selectSessionModel(session, model);
}

export async function openConnectorFile(path) {
  const fallback = applyFileState({
    path,
    currentPath: path,
    content: filesState.content || filesState.lines.join("\n"),
    savedContent: filesState.savedContent || filesState.lines.join("\n"),
    lines: filesState.lines,
    roots: filesState.roots,
    breadcrumbs: [workspace.project, path],
    dirty: false,
    status: "Opened"
  });

  if (!connectorState.connector.available || !connectorState.connector.readFile) return fallback;

  try {
    const response = await connectorState.connector.readFile(path, { sessionId: workspace.sessionId });
    return applyFileState(response);
  } catch (error) {
    filesState.status = connectorErrorMessage(error);
    captureActiveSessionState();
    throw error;
  }
}

export async function openConnectorBrowser(target) {
  const currentTarget = String(target || "").trim();
  if (!currentTarget) {
    browserState.status = "Browser target is required";
    captureActiveSessionState();
    throw new Error(browserState.status);
  }

  if (!connectorState.connector.available || !connectorState.connector.openBrowser) {
    browserState.url = currentTarget;
    browserState.title = currentTarget;
    browserState.summary = "Browser target staged locally; connector is not available.";
    browserState.previewMode = currentTarget.startsWith("http://") || currentTarget.startsWith("https://") ? "public" : "local-file";
    browserState.localPath = browserState.previewMode === "local-file" ? currentTarget : "";
    browserState.status = "Opened";
    captureActiveSessionState();
    return { browser: browserState };
  }

  try {
    const response = await connectorState.connector.openBrowser(currentTarget, {
      sessionId: workspace.sessionId,
      actor: "user",
      clearRequest: false
    });
    assignObject(browserState, response.browser || response);
    browserState.status = "Opened";
    replaceArray(artifactState, []);
    captureActiveSessionState();
    return response;
  } catch (error) {
    browserState.status = connectorErrorMessage(error);
    captureActiveSessionState();
    throw error;
  }
}

export async function saveConnectorFile(path, content) {
  if (!connectorState.connector.available || !connectorState.connector.saveFile) {
    return applyFileState({
      path,
      currentPath: path,
      content,
      savedContent: content,
      lines: content.split("\n"),
      roots: filesState.roots,
      breadcrumbs: filesState.breadcrumbs,
      dirty: false,
      status: "Saved"
    });
  }

  try {
    const response = await connectorState.connector.saveFile(path, content, { sessionId: workspace.sessionId });
    return applyFileState(response);
  } catch (error) {
    filesState.status = connectorErrorMessage(error);
    captureActiveSessionState();
    throw error;
  }
}

export async function updateConnectorNativeMenuState(focusState) {
  if (!connectorState.connector.available || !connectorState.connector.updateNativeMenuState) {
    return null;
  }
  return connectorState.connector.updateNativeMenuState(focusState);
}

export function editConnectorFile(content) {
  filesState.content = content;
  filesState.lines = content.split("\n");
  filesState.dirty = content !== (filesState.savedContent || "");
  filesState.status = filesState.dirty ? "Unsaved changes" : "Saved";
  captureActiveSessionState();
  return filesState;
}

export async function restoreConnectorExplorerScope(path) {
  filesState.expandedFolders = {};
  const root = connectorState.connector.available && connectorState.connector.readFile
    ? await connectorState.connector.readFile("", { sessionId: workspace.sessionId })
    : { roots: filesState.roots || [], breadcrumbs: [workspace.project], status: "Ready" };
  applyFileState({
    ...root,
    currentPath: filesState.currentPath,
    content: filesState.content,
    savedContent: filesState.savedContent,
    lines: filesState.lines,
    dirty: filesState.dirty,
    status: root.status || "Ready"
  });

  const segments = path.split("/").filter(Boolean);
  let current = "";
  for (const segment of segments) {
    current = current ? `${current}/${segment}` : segment;
    await expandConnectorFolder(current);
  }
  captureActiveSessionState();
  return filesState;
}

export async function toggleConnectorFolder(path) {
  filesState.expandedFolders ||= {};
  const currentRows = filesState.roots || [];
  const parentIndex = currentRows.findIndex((row) => fileRowPath(row) === path);
  if (parentIndex < 0) return filesState;

  if (filesState.expandedFolders[path]) {
    delete filesState.expandedFolders[path];
    filesState.roots = currentRows.filter((row) => {
      const rowPath = fileRowPath(row);
      return rowPath === path || !rowPath.startsWith(`${path}/`);
    });
    captureActiveSessionState();
    return filesState;
  }

  const parentDepth = Number(currentRows[parentIndex][4] || 0);
  await expandConnectorFolder(path, parentIndex, parentDepth);
  return filesState;
}

async function expandConnectorFolder(path, knownParentIndex = -1, knownParentDepth = 0) {
  filesState.expandedFolders ||= {};
  const currentRows = filesState.roots || [];
  const parentIndex = knownParentIndex >= 0 ? knownParentIndex : currentRows.findIndex((row) => fileRowPath(row) === path);
  if (parentIndex < 0 || filesState.expandedFolders[path]) return filesState;
  const parentDepth = knownParentIndex >= 0 ? knownParentDepth : Number(currentRows[parentIndex][4] || 0);
  const response = connectorState.connector.available && connectorState.connector.readFile
    ? await connectorState.connector.readFile(path, { sessionId: workspace.sessionId })
    : { roots: [] };
  const children = (response.roots || []).map((row) => [
    row[0],
    row[1],
    row[2],
    row[3]?.startsWith(`${path}/`) ? row[3] : `${path}/${row[3] || row[0]}`,
    row[4] || String(parentDepth + 1),
    row[5] || "",
    row[6] || ""
  ]);
  filesState.roots = [
    ...currentRows.slice(0, parentIndex + 1),
    ...children,
    ...currentRows.slice(parentIndex + 1)
  ];
  filesState.expandedFolders[path] = true;
  filesState.status = response.status || filesState.status || "Ready";
  captureActiveSessionState();
  return filesState;
}

export async function saveConnectorProviderProfile(request) {
  const fallback = {
    id: request.providerId || providerIdFromLabel(request.label),
    label: request.label,
    kind: request.kind || "OpenAI-compatible",
    baseUrl: request.baseUrl || defaultProviderBaseUrl(request.kind),
    endpoint: request.baseUrl || defaultProviderBaseUrl(request.kind),
    status: request.apiKey ? "Key configured" : "Key not configured",
    keyStatus: { state: request.apiKey ? "present" : "missing", source: "session" },
    enabled: true,
    supportsDiscovery: true
  };

  const saved = connectorState.connector.available && connectorState.connector.saveProviderProfile
    ? await connectorState.connector.saveProviderProfile(request)
    : fallback;

  removeProviderPlaceholders(saved);
  const index = providers.findIndex((provider) => provider.id === saved.id || provider.label === saved.label);
  if (index >= 0) providers.splice(index, 1, saved);
  else providers.push(saved);
  await refreshConnectorProviderModels();
  return saved;
}

export async function deleteConnectorProviderProfile(providerId) {
  const response = connectorState.connector.available && connectorState.connector.deleteProviderProfile
    ? await connectorState.connector.deleteProviderProfile(providerId)
    : { providerId, deleted: true };

  replaceArray(providers, providers.filter((provider) => provider.id !== providerId));
  replaceArray(models, models.filter((model) => model.providerId !== providerId));
  return response;
}

export async function refreshConnectorProviderModels() {
  if (!connectorState.connector.available || !connectorState.connector.listProviderModels) return models;
  const next = await connectorState.connector.listProviderModels();
  replaceArray(models, next);
  return models;
}

function removeProviderPlaceholders(saved) {
  const savedBase = (saved.baseUrl || saved.endpoint || "").trim();
  for (let index = providers.length - 1; index >= 0; index -= 1) {
    const provider = providers[index];
    if (provider.id === saved.id || provider.label === saved.label) continue;
    const providerBase = (provider.baseUrl || provider.endpoint || "").trim();
    const placeholder = provider.keyStatus?.state === "unknown" || provider.status === "Key status managed by backend";
    if (placeholder && providerBase === savedBase) providers.splice(index, 1);
  }
}

function providerIdFromLabel(label) {
  return String(label || "provider")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "") || "provider";
}

function defaultProviderBaseUrl(kind) {
  if (kind === "OpenAI") return "https://api.openai.com/v1";
  return "https://openrouter.ai/api/v1";
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

function ensureSessionForPrompt(prompt, created = null) {
  const label = created?.title || sessionLabel(prompt);
  const id = created?.title && created?.id ? created.id : `local-${providerIdFromLabel(label)}`;
  workspace.session = label;
  workspace.sessionId = id;
  const project = projects.find((record) => record.name === workspace.project);
  const sessionRecord = { id, label };
  if (!project) {
    projects.unshift({ name: workspace.project, sessions: [sessionRecord] });
    return;
  }
  project.sessions = [
    sessionRecord,
    ...project.sessions.filter((session) => sessionLabelOf(session) !== label && sessionIdOf(session) !== id)
  ];
}

function sessionLabel(prompt) {
  const text = (prompt || "New chat session").trim().replace(/\s+/g, " ");
  return text.length > 48 ? `${text.slice(0, 45)}...` : text;
}

function applyConnectorWorkspace(payload) {
  assignObject(workspace, payload.workspace);
  replaceArray(persistentSessions, payload.sessions || []);
  replaceArray(projects, payload.projects);
  replaceArray(providers, payload.providers);
  replaceArray(models, payload.models);
  replaceArray(pluginCatalog, payload.pluginCatalog);
  replaceArray(pluginMarketplaces, payload.pluginMarketplaces);
  replaceArray(skillCatalog, payload.skillCatalog);
  replaceArray(mcpServers, payload.mcpServers);
  assignObject(browserState, payload.browser);
  replaceArray(artifactState, payload.artifacts || []);
  assignObject(filesState, payload.files);
  assignObject(terminalState, payload.terminal);
  assignObject(threadState, payload.thread);
  const activeRecord = persistentSessions.find((session) => session.id === workspace.sessionId);
  if (activeRecord) applySessionRecord(activeRecord, { skipWorkspace: true });
  else replaceArray(threadTurns, workspace.session ? [turnFromThreadState(threadState, "connector-turn")] : []);
  captureActiveSessionState();
}

export function activateSessionState(project, session, options = {}) {
  if (!options.skipCapture) captureActiveSessionState();

  workspace.project = project;
  workspace.session = sessionLabelOf(session);
  const nextSessionId = options.sessionId !== undefined ? options.sessionId : sessionIdOf(session);
  workspace.sessionId = nextSessionId || "";
  const key = sessionKey(project, workspace.session, workspace.sessionId);
  if (options.reset || !sessionState.bySurface[key]) {
    sessionState.bySurface[key] = sessionSnapshot({
      model: options.model ?? workspace.model,
      sessionId: workspace.sessionId,
      turns: options.turns || []
    });
  }

  applySessionSnapshot(sessionState.bySurface[key]);
  return key;
}

function captureActiveSessionState() {
  if (!workspace.project || !workspace.session) return;
  sessionState.bySurface[sessionKey(workspace.project, workspace.session, workspace.sessionId)] = sessionSnapshot({
    browser: browserState,
    artifacts: artifactState,
    files: filesState,
    model: workspace.model,
    sessionId: workspace.sessionId,
    terminal: terminalState,
    thread: threadState,
    turns: threadTurns
  });
}

function applySessionSnapshot(snapshot) {
  workspace.model = snapshot.model || "";
  workspace.sessionId = snapshot.sessionId || workspace.sessionId || "";
  assignObject(browserState, cloneValue(snapshot.browser));
  replaceArray(artifactState, cloneValue(snapshot.artifacts || []));
  assignObject(filesState, cloneValue(snapshot.files));
  assignObject(terminalState, cloneValue(snapshot.terminal));
  assignObject(threadState, cloneValue(snapshot.thread));
  replaceArray(threadTurns, cloneValue(snapshot.turns));
}

function applySessionRecord(record, options = {}) {
  if (!record) return;
  const turns = Array.isArray(record.turns) && record.turns.length > 0
    ? record.turns.map((turn, index) => ({
        id: turn.id || `${record.id}-turn-${index + 1}`,
        user: turn.user,
        agent: turn.agent,
        extra: turn.extra,
        tool: turn.tool,
        run: turn.run,
        logs: [turn.tool, turn.run].filter(Boolean),
        startedAt: Date.now(),
        completedAt: Date.now(),
        workExpanded: false,
        permission: null,
        failed: false,
        pending: false
      }))
    : (record.thread?.user || record.thread?.agent ? [turnFromThreadState(record.thread, `${record.id}-turn`)] : []);

  if (!options.skipWorkspace) {
    workspace.project = record.project || workspace.project;
    workspace.session = record.title || workspace.session;
    workspace.sessionId = record.id || workspace.sessionId;
  }
  const key = sessionKey(record.project || workspace.project, record.title || workspace.session, record.id || workspace.sessionId);
  sessionState.bySurface[key] = sessionSnapshot({
    browser: record.browser,
    artifacts: record.artifacts || [],
    files: record.files,
    model: record.selectedModel || record.selected_model || "",
    sessionId: record.id || workspace.sessionId,
    terminal: record.terminal,
    thread: record.thread,
    turns
  });
  if (record.files?.currentPath) {
    toolState.fileViewBySurface ||= {};
    toolState.fileViewBySurface[key] = "editor";
  }
  applySessionSnapshot(sessionState.bySurface[key]);
}

function applyFileState(response = {}) {
  const currentPath = response.currentPath || response.path || filesState.currentPath || "";
  const content = response.content ?? response.lines?.join("\n") ?? filesState.content ?? "";
  const savedContent = response.savedContent ?? response.saved_content ?? content;
  const existingRoots = filesState.roots || [];
  const responseRoots = response.roots || existingRoots;
  const preserveExpandedRoots = Boolean(
    currentPath &&
    response.roots &&
    Object.keys(filesState.expandedFolders || {}).length > 0 &&
    existingRoots.length > responseRoots.length
  );
  assignObject(filesState, {
    roots: preserveExpandedRoots ? existingRoots : responseRoots,
    breadcrumbs: response.breadcrumbs || (currentPath ? [workspace.project, currentPath] : [workspace.project]),
    lines: response.lines || content.split("\n"),
    currentPath,
    content,
    savedContent,
    dirty: Boolean(response.dirty),
    status: response.status || (response.saved ? "Saved" : "Opened"),
    expandedFolders: filesState.expandedFolders || {}
  });
  captureActiveSessionState();
  return filesState;
}

function fileRowPath(row) {
  return row?.[3] || row?.[0] || "";
}

function sessionSnapshot(overrides = {}) {
  const turns = cloneValue(overrides.turns || []);
  return {
    browser: cloneValue(overrides.browser || browserState),
    artifacts: cloneValue(overrides.artifacts || artifactState),
    files: cloneValue(overrides.files || filesState),
    model: overrides.model || "",
    sessionId: overrides.sessionId || workspace.sessionId || "",
    terminal: cloneValue(overrides.terminal || terminalState),
    thread: cloneValue(overrides.thread || threadStateFromTurns(turns)),
    turns
  };
}

function sessionLabelOf(session) {
  if (typeof session === "string") return session;
  return session?.label || session?.title || "";
}

function sessionIdOf(session) {
  if (typeof session === "string") return "";
  return session?.id || session?.sessionId || "";
}

function threadStateFromTurns(turns) {
  const latest = turns[turns.length - 1];
  return latest
    ? {
        user: latest.user,
        agent: latest.agent,
        extra: latest.extra,
        tool: latest.tool,
        run: latest.run
      }
    : { user: "", agent: "", extra: "", tool: "", run: "" };
}

function sessionKey(project, session, sessionId = "") {
  return `chat:${project}:${sessionId || session || "untitled"}`;
}

function cloneValue(value) {
  return JSON.parse(JSON.stringify(value));
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
