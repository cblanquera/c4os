import {
  beginConnectorStateLoad,
  artifactState,
  browserState,
  connectorState,
  filesState,
  loadConnectorSession,
  mcpServers,
  models,
  editConnectorFile,
  openConnectorBrowser,
  openConnectorFile,
  openConnectorWorkspace,
  openConnectorWorkspaceFile,
  pluginCatalog,
  pluginMarketplaces,
  projects,
  providers,
  routes,
  deleteConnectorProviderProfile,
  saveConnectorProviderProfile,
  saveConnectorFile,
  saveConnectorWorkspaceFile,
  restoreConnectorExplorerScope,
  runConnectorTerminalCommand,
  selectConnectorSessionModel,
  sendConnectorPrompt,
  setConnectorModelEnabled,
  setConnectorProviderEnabled,
  settingsItems,
  syncConnectorNativeBrowser,
  terminalState,
  threadTurns,
  toggleConnectorFolder,
  updateConnectorNativeMenuState,
  skillCatalog,
  workspace
} from "./data.js";
import { h, icon } from "./dom.js";
import { bindInteractions, bindMessage } from "./interactions.js";
import { renderMarkdown } from "./markdown.js";
import { appStore, sessionSurfaceKey } from "./state.js";

const app = document.querySelector("#app");
let providerFormDraft = null;
let providerSetupGate = false;
let lastAppRouteBeforeSettings = "app-start";
const modelSettingsFilters = {
  provider: "all",
  search: ""
};
let nativeFileMenuBound = false;
let nativeFileEditorOpen = false;
let nativeFileShortcutsBound = false;
let nativeFileFocusBound = false;
let nativeBrowserSurfaceActive = false;
let nativeBrowserResizeObserver = null;
let nativeBrowserObservedFrame = null;
let nativeBrowserWindowResizeBound = false;
let terminalInstance = null;
let terminalKey = "";
let terminalOutputUnlisten = null;
let terminalOutputVersion = 0;
let terminalOutputPollTimer = null;
let workspaceOpenBound = false;

//--------------------------------------------------------------------//
// Shared UI primitives
//--------------------------------------------------------------------//

/**
 * Build an internal hash link that routes without leaving the SPA.
 */
function link(route, className, children, attrs = {}) {
  return h("a", { class: className, href: `#${route}`, "data-link": route, ...attrs }, children);
}

/**
 * Build a text button, optionally with a leading icon.
 */
function button(label, className = "button secondary", iconName, attrs = {}) {
  return h("button", { class: className, type: "button", "aria-label": iconName ? label : null, ...attrs }, [
    iconName ? icon(iconName) : null,
    h("span", { text: label })
  ]);
}

/**
 * Build a square icon-only button with an accessible label.
 */
function iconButton(label, iconName, className = "icon-button", attrs = {}) {
  return h("button", { class: className, type: "button", "aria-label": label, ...attrs }, [icon(iconName)]);
}

//--------------------------------------------------------------------//
// Route orchestration
//--------------------------------------------------------------------//

/**
 * Resolve the current hash into a known r04 route id.
 */
function routeFromHash() {
  const next = window.location.hash.replace(/^#\/?/, "") || "app-start";
  return routes.has(next) ? next : "app-start";
}

/**
 * Render the active route and rebind local interactions after DOM replacement.
 */
function render() {
  const route = routeFromHash();
  const previousRoute = appStore.route;
  if (route.startsWith("settings") && !previousRoute.startsWith("settings")) {
    lastAppRouteBeforeSettings = previousRoute;
  }
  if (route === "new-session" && previousRoute === "chat-session") {
    const surface = `new:${activeProjectIdentity()}`;
    appStore.composer.bySurface[surface] = {
      ...appStore.composerFor(surface),
      model: null,
      openPicker: null,
      prompt: ""
    };
  }
  appStore.setRoute(route);
  document.body.dataset.route = route;
  if (route === "app-start") app.replaceChildren(renderStart());
  else if (route.startsWith("settings")) app.replaceChildren(renderSettings(route));
  else app.replaceChildren(renderShell(route));
  bindInteractions(render, pluginInitials);
  bindDelegatedConnectorRun();
  bindConnectorRun();
  bindWorkspaceOpen();
  bindNativeFileMenu();
  bindWorkLogDisclosure();
  bindToolTabs();
  bindTerminalEmulator();
  bindPanelLinks();
  bindFileEditor();
  bindBrowserAddress();
  bindSettingsNavigation();
  bindDelegatedComposerPickers();
  bindComposerPickers();
  bindModelEnablement();
  bindProviderRows();
  bindModelFilters();
  bindModelBulkToggle();
  bindProviderFormSave();
  bindSessionRows();
  applyShellState();
}

function bindDelegatedComposerPickers() {
  if (document.body.dataset.boundDelegatedComposerPickers) return;
  document.body.dataset.boundDelegatedComposerPickers = "true";
  document.body.addEventListener("click", (event) => {
    const trigger = event.target.closest("[data-local-picker-trigger]");
    if (trigger) {
      const composerNode = trigger.closest(".composer");
      if (!composerNode) return;
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "openPicker", trigger.dataset.localPickerTrigger);
      updateComposerPicker(composerNode, surface);
      return;
    }

    const provider = event.target.closest("[data-local-provider]");
    if (provider) {
      const composerNode = provider.closest(".composer");
      if (!composerNode) return;
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "openPicker", "models");
      updateComposerPicker(composerNode, surface);
      return;
    }

    const model = event.target.closest("[data-local-model]");
    if (model) {
      const composerNode = model.closest(".composer");
      if (!composerNode) return;
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "model", model.dataset.localModel);
      if (routeFromHash() === "chat-session") {
        workspace.model = model.dataset.localModel;
        selectConnectorSessionModel(workspace.sessionId || workspace.session || surface, model.dataset.localModel).catch((error) => {
          connectorState.error = error.message;
        });
      }
      appStore.setComposerValue(surface, "openPicker", null);
      composerNode.querySelector("[data-local-picker-trigger='models'] span").textContent = selectedModelLabel(surface);
      composerNode.querySelector("[data-local-picker-trigger='models']").setAttribute("aria-label", `Model: ${selectedModelLabel(surface)}`);
      updateComposerPicker(composerNode, surface);
    }
  });
}

function bindConnectorRun() {
  if (!connectorState.connector.available) return;
  document.querySelectorAll(".send-button").forEach((control) => {
    if (control.dataset.boundConnectorRun) return;
    control.dataset.boundConnectorRun = "true";
    control.addEventListener("click", async () => {
      await sendFromComposerControl(control);
    });
  });
}

function bindDelegatedConnectorRun() {
  if (document.body.dataset.boundDelegatedConnectorRun) return;
  document.body.dataset.boundDelegatedConnectorRun = "true";
  document.body.addEventListener("click", async (event) => {
    const control = event.target.closest(".send-button");
    if (!control || control.dataset.boundConnectorRun === "true") return;
    await sendFromComposerControl(control);
  });
}

async function sendFromComposerControl(control) {
  if (connectorState.runPending && !threadTurns.some((turn) => turn.pending)) {
    connectorState.runPending = false;
  }
  if (!connectorState.connector.available || connectorState.runPending) return;
  const composerNode = control.closest(".composer");
  const prompt = composerNode?.querySelector(".prompt-box")?.textContent?.trim() || "";
  const shouldCreateSession = routeFromHash() !== "chat-session";
  const surface = composerNode?.dataset.composerSurface || composerSurfaceKey();
  const selectedModel = selectedModelForSurface(surface);
  appStore.composer.bySurface[surface] = { prompt: "" };
  const promptBox = composerNode?.querySelector(".prompt-box");
  if (promptBox) promptBox.textContent = "";
  const pending = sendConnectorPrompt(prompt, {
    createSession: shouldCreateSession,
    model: selectedModel,
    onTurnCreated: (turn) => {
      if (!shouldCreateSession) appendTurnToThread(turn);
    },
    onStateChange: (turn) => updateTurnDom(turn),
    beforeComplete: minimumPendingFrame,
    onTerminalStateChange: updateAgentTerminalDom
  });
  window.location.hash = "chat-session";
  if (shouldCreateSession) render();
  await minimumPendingFrame();
  const pendingSessionSurface = shouldCreateSession ? appStore.activeSessionKey() : "";
  await pending;
  if (shouldCreateSession) appStore.setComposerValue(surface, "model", null);
  if (shouldCreateSession) {
    const reconciledSessionSurface = appStore.activeSessionKey();
    if (pendingSessionSurface && pendingSessionSurface !== reconciledSessionSurface) {
      const selectedTool = appStore.tools.bySurface[pendingSessionSurface];
      if (selectedTool && !appStore.tools.bySurface[reconciledSessionSurface]) {
        appStore.tools.bySurface[reconciledSessionSurface] = selectedTool;
      }
    }
    render();
  }
  updateAgentTerminalDom();
  updateTurnDom(threadTurns[threadTurns.length - 1]);
}

function minimumPendingFrame() {
  return new Promise((resolve) => window.setTimeout(resolve, 400));
}

function bindWorkLogDisclosure() {
  document.querySelectorAll("[data-work-toggle]").forEach((control) => {
    if (control.dataset.boundWorkToggle) return;
    control.dataset.boundWorkToggle = "true";
    control.addEventListener("click", () => {
      const turn = threadTurns.find((record) => record.id === control.dataset.workToggle);
      if (!turn || turn.pending) return;
      turn.workExpanded = !turn.workExpanded;
      updateWorkLogDom(turn);
    });
  });
}

function bindToolTabs() {
  document.querySelectorAll("[data-tool-tab]").forEach((control) => {
    if (control.dataset.boundToolTab) return;
    control.dataset.boundToolTab = "true";
    control.addEventListener("click", () => {
      const route = routeFromHash();
      setActiveToolForRoute(route, control.dataset.toolTab);
      const panel = document.querySelector(".tool-panel");
      panel?.replaceWith(renderToolPanel(activeToolForRoute(route), route));
      bindToolTabs();
      bindTerminalEmulator();
      bindPanelLinks();
      bindFileEditor();
      syncNativeBrowserSurface();
    });
  });
}

async function bindTerminalEmulator() {
  const host = document.querySelector("[data-terminal-emulator]");
  if (!host) return;
  bindTerminalAgentResize();
  updateAgentTerminalDom();
  const sessionId = workspace.sessionId || "";
  const key = `${sessionId}:user`;
  if (terminalInstance && terminalKey === key && host.querySelector(".xterm")) return;
  terminalInstance?.dispose?.();
  terminalInstance = null;
  stopTerminalOutputPolling();
  terminalKey = key;

  const TerminalRenderer = typeof window.Terminal === "function" ? window.Terminal : BasicTerminalRenderer;
  const term = new TerminalRenderer({
    cols: 102,
    cursorBlink: true,
    cursorStyle: "block",
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: 14,
    fontWeight: 700,
    lineHeight: 1.2,
    rows: 31,
    theme: {
      background: "#1e1e1f",
      cursor: "#c4c4bf",
      foreground: "#f4f4f0",
      selectionBackground: "#565a61"
    }
  });
  terminalInstance = term;
  term.open(host);
  host.addEventListener("pointerdown", () => window.setTimeout(() => term.focus(), 0));
  host.addEventListener("click", () => term.focus());
  term.focus();

  const hasLiveTerminalSession = Boolean(connectorState.connector.startTerminalSession);
  const existing = terminalState.userTerminal?.output || terminalState.output || "";
  if (!hasLiveTerminalSession && existing.trim()) {
    term.write(existing.endsWith("\n") ? existing : `${existing}\n`);
  }

  let terminalEventsAttached = false;
  term.onData((input) => {
    const writeInput = connectorState.connector.writeTerminalInput;
    if (!writeInput) {
      echoLocalTerminalInput(term, input);
      return;
    }
    const outputVersionBeforeWrite = terminalOutputVersion;
    Promise.resolve(writeInput(input, {
      sessionId: workspace.sessionId,
      terminalKind: "user"
    })).then(() => {
      drainTerminalOutput();
      window.setTimeout(() => {
        if ((terminalEventsAttached || !connectorState.connector.readTerminalOutput) && terminalOutputVersion === outputVersionBeforeWrite) {
          echoLocalTerminalInput(term, input);
        }
      }, 120);
    }).catch(() => echoLocalTerminalInput(term, input));
  });

  if (!terminalOutputUnlisten && connectorState.connector.listenTerminalOutput) {
    try {
      terminalOutputUnlisten = await connectorState.connector.listenTerminalOutput((event) => {
        writeTerminalOutputEvent(event);
      });
      terminalEventsAttached = Boolean(terminalOutputUnlisten);
    } catch {
      terminalOutputUnlisten = null;
    }
  }
  if (!terminalEventsAttached) startTerminalOutputPolling();

  try {
    await connectorState.connector.startTerminalSession?.({
      sessionId: workspace.sessionId,
      terminalKind: "user"
    });
    await drainTerminalOutput();
    await connectorState.connector.resizeTerminalSession?.({
      rows: term.rows || 31,
      cols: term.cols || 102
    }, {
      sessionId: workspace.sessionId,
      terminalKind: "user"
    });
  } catch (error) {
    term.write(`Terminal unavailable: ${terminalErrorMessage(error)}\r\n`);
  }
}

class BasicTerminalRenderer {
  constructor(options = {}) {
    this.cols = options.cols || 102;
    this.rows = options.rows || 31;
    this.text = "";
    this.onDataCallback = null;
  }

  open(host) {
    this.host = host;
    host.replaceChildren();
    host.classList.add("xterm", "basic-terminal");
    this.screen = document.createElement("div");
    this.screen.className = "xterm-screen basic-terminal-screen";
    this.screen.tabIndex = 0;
    this.screen.setAttribute("role", "textbox");
    this.screen.setAttribute("aria-label", "User terminal input");
    this.screen.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        event.preventDefault();
        this.onDataCallback?.("\r");
      } else if (event.key === "Backspace") {
        event.preventDefault();
        this.onDataCallback?.("\u007f");
      } else if (event.key.length === 1 && !event.metaKey && !event.ctrlKey) {
        event.preventDefault();
        this.onDataCallback?.(event.key);
      }
    });
    host.append(this.screen);
  }

  focus() {
    this.screen?.focus();
  }

  onData(callback) {
    this.onDataCallback = callback;
  }

  write(value) {
    this.text += value;
    this.screen.textContent = this.text;
  }

  dispose() {
    this.host?.replaceChildren();
  }
}

async function drainTerminalOutput() {
  const readOutput = connectorState.connector.readTerminalOutput;
  if (!terminalInstance || !readOutput) return;
  try {
    const event = await readOutput({
      sessionId: workspace.sessionId,
      terminalKind: "user"
    });
    writeTerminalOutputEvent(event);
  } catch {
    stopTerminalOutputPolling();
  }
}

function startTerminalOutputPolling() {
  if (terminalOutputPollTimer || !connectorState.connector.readTerminalOutput) return;
  terminalOutputPollTimer = window.setInterval(() => {
    drainTerminalOutput();
  }, 120);
}

function stopTerminalOutputPolling() {
  if (!terminalOutputPollTimer) return;
  window.clearInterval(terminalOutputPollTimer);
  terminalOutputPollTimer = null;
}

function writeTerminalOutputEvent(event) {
  if (!event) return;
  if (event.sessionId && workspace.sessionId && event.sessionId !== workspace.sessionId) return;
  if ((event.terminalKind || "user") !== "user") return;
  const text = event.text || "";
  if (!text) return;
  terminalOutputVersion += 1;
  terminalInstance?.write(text);
  if (terminalState.userTerminal) {
    terminalState.userTerminal.output = `${terminalState.userTerminal.output || ""}${text}`;
    terminalState.output = terminalState.userTerminal.output;
  }
}

function updateAgentTerminalDom() {
  const pane = document.querySelector("[data-agent-terminal]");
  if (!pane) return;
  const agentTerminal = agentTerminalViewForRoute(routeFromHash());
  const title = pane.querySelector(".terminal-agent-title");
  const transcript = pane.querySelector(".terminal-agent-transcript");
  if (title) title.textContent = agentTerminal.title || "Agent command terminal";
  if (transcript) transcript.textContent = agentTerminal.output || agentTerminal.summary || "";
  pane.scrollTop = pane.scrollHeight;
}

function agentTerminalViewForRoute(route) {
  if (route !== "chat-session") {
    return {
      title: "Agent command terminal",
      output: "Agent command output is not running.",
      summary: "Read-only agent command output."
    };
  }
  return terminalState.agentTerminal || {
    title: "Agent command terminal",
    output: "",
    summary: "Read-only agent command output."
  };
}

function bindTerminalAgentResize() {
  const handle = document.querySelector("[data-resize-stack='terminal-agent']");
  if (!handle || handle.dataset.boundTerminalAgentResize) return;
  handle.dataset.boundTerminalAgentResize = "true";
  handle.addEventListener("pointerdown", (event) => {
    const panel = handle.closest(".terminal-tool");
    const agentPane = panel?.querySelector("[data-agent-terminal]");
    if (!panel || !agentPane) return;
    const start = event.clientY;
    const startHeight = agentPane.getBoundingClientRect().height;
    handle.setPointerCapture(event.pointerId);
    const move = (moveEvent) => {
      const height = Math.max(96, Math.min(420, startHeight + start - moveEvent.clientY));
      panel.style.setProperty("--terminal-agent", `${Math.round(height)}px`);
      appStore.setShellValue("terminalBottom", Math.round(height));
    };
    const up = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", up);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", up);
  });
}

function echoLocalTerminalInput(term, input) {
  if (input === "\r") {
    term.write("\r\n");
    return;
  }
  if (input === "\u007f") {
    term.write("\b \b");
    return;
  }
  term.write(input);
}

function terminalErrorMessage(error) {
  if (!error) return "Terminal session failed to start";
  if (typeof error === "string") return error;
  return error.message || "Terminal session failed to start";
}

function bindSessionRows() {
  document.querySelectorAll("[data-session-target]").forEach((control) => {
    if (control.dataset.boundSessionTarget) return;
    control.dataset.boundSessionTarget = "true";
    control.addEventListener("click", (event) => {
      event.preventDefault();
      appStore.setActiveSession(
        control.dataset.projectTarget,
        {
          id: control.dataset.sessionId || "",
          label: control.dataset.sessionTarget
        },
        {
          projectId: control.dataset.projectId || "",
          rootPath: control.dataset.projectRootPath || ""
        }
      );
      const finalize = () => {
        window.location.hash = "chat-session";
        updateShellSessionDom();
      };
      if (control.dataset.sessionId && !control.dataset.sessionId.startsWith("local-")) {
        loadConnectorSession(control.dataset.sessionId).finally(finalize);
      } else {
        finalize();
      }
    });
  });
  document.querySelectorAll("[data-project-target]:not([data-session-target])").forEach((control) => {
    if (control.dataset.boundProjectTarget) return;
    control.dataset.boundProjectTarget = "true";
    control.addEventListener("click", async (event) => {
      event.preventDefault();
      const rootPath = control.dataset.projectRootPath || "";
      if (rootPath && connectorState.connector.available) {
        await openConnectorWorkspace(rootPath, { mergeProjects: true });
      } else {
        workspace.project = control.dataset.projectTarget;
        workspace.projectId = control.dataset.projectId || "";
        workspace.rootPath = rootPath;
      }
      window.location.hash = "new-session";
      render();
    });
  });
}

function bindComposerPickers() {
  document.querySelectorAll("[data-local-picker-trigger]").forEach((control) => {
    if (control.dataset.boundLocalPicker) return;
    control.dataset.boundLocalPicker = "true";
    control.addEventListener("click", () => {
      const composerNode = control.closest(".composer");
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "openPicker", control.dataset.localPickerTrigger);
      updateComposerPicker(composerNode, surface);
    });
  });
  document.querySelectorAll("[data-local-provider]").forEach((control) => {
    if (control.dataset.boundLocalProvider) return;
    control.dataset.boundLocalProvider = "true";
    control.addEventListener("click", () => {
      const composerNode = control.closest(".composer");
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "openPicker", "models");
      updateComposerPicker(composerNode, surface);
    });
  });
  document.querySelectorAll("[data-local-model]").forEach((control) => {
    if (control.dataset.boundLocalModel) return;
    control.dataset.boundLocalModel = "true";
    control.addEventListener("click", () => {
      const composerNode = control.closest(".composer");
      const surface = composerNode.dataset.composerSurface || composerSurfaceKey();
      appStore.setComposerValue(surface, "model", control.dataset.localModel);
      if (routeFromHash() === "chat-session") {
        workspace.model = control.dataset.localModel;
        selectConnectorSessionModel(workspace.sessionId || workspace.session || surface, control.dataset.localModel).catch((error) => {
          connectorState.error = error.message;
        });
      }
      appStore.setComposerValue(surface, "openPicker", null);
      composerNode.querySelector("[data-local-picker-trigger='models'] span").textContent = selectedModelLabel(surface);
      composerNode.querySelector("[data-local-picker-trigger='models']").setAttribute("aria-label", `Model: ${selectedModelLabel(surface)}`);
      updateComposerPicker(composerNode, surface);
    });
  });
}

function bindModelEnablement() {
  document.querySelectorAll("[data-model-enable]").forEach((control) => {
    if (control.dataset.boundModelEnable) return;
    control.dataset.boundModelEnable = "true";
    control.addEventListener("click", async () => {
      const next = control.dataset.modelEnabled !== "true";
      const model = await setConnectorModelEnabled(control.dataset.modelEnable, next);
      if (!model) return;
      setModelEnablementDom(control, model);
      document.querySelector("[data-model-bulk-toggle]")?.replaceWith(modelBulkToggle());
      bindModelBulkToggle();
    });
  });
}

function setModelEnablementDom(control, model) {
  control.dataset.modelEnabled = String(model.enabled);
  control.setAttribute("aria-label", `${model.label} ${model.enabled ? "enabled" : "disabled"}`);
  control.setAttribute("aria-pressed", String(model.enabled));
  const row = control.closest(".data-row");
  const status = row?.querySelector(".status-pill");
  if (status) status.textContent = model.enabled ? "Enabled" : "Disabled";
}

async function setFilteredModelsEnabled(enabled) {
  for (const model of filteredModels()) {
    if (model.enabled === enabled) continue;
    await setConnectorModelEnabled(model.id, enabled);
  }
  document.querySelector(".data-list")?.replaceWith(modelSettingsList());
  document.querySelector("[data-model-bulk-toggle]")?.replaceWith(modelBulkToggle());
  bindModelEnablement();
  bindModelBulkToggle();
}

function bindModelBulkToggle() {
  const control = document.querySelector("[data-model-bulk-toggle]");
  if (!control || control.dataset.boundModelBulkToggle) return;
  control.dataset.boundModelBulkToggle = "true";
  control.addEventListener("click", async () => {
    await setFilteredModelsEnabled(control.dataset.modelBulkEnabled !== "false");
  });
}

function bindProviderRows() {
  document.querySelectorAll("[data-provider-edit]").forEach((control) => {
    if (control.dataset.boundProviderEdit) return;
    control.dataset.boundProviderEdit = "true";
    control.addEventListener("click", () => {
      providerFormDraft = providers.find((provider) => provider.id === control.dataset.providerEdit) || null;
      window.location.hash = "settings-add-provider";
      render();
    });
  });
  document.querySelectorAll("[data-provider-new]").forEach((control) => {
    if (control.dataset.boundProviderNew) return;
    control.dataset.boundProviderNew = "true";
    control.addEventListener("click", () => {
      providerFormDraft = null;
    });
  });
  document.querySelectorAll("[data-provider-enable]").forEach((control) => {
    if (control.dataset.boundProviderEnable) return;
    control.dataset.boundProviderEnable = "true";
    control.addEventListener("click", async () => {
      const next = control.dataset.providerEnabled !== "true";
      const provider = await setConnectorProviderEnabled(control.dataset.providerEnable, next);
      if (!provider) return;
      control.dataset.providerEnabled = String(provider.enabled);
      control.setAttribute("aria-label", `${provider.label} ${provider.enabled ? "enabled" : "disabled"}`);
      control.setAttribute("aria-pressed", String(provider.enabled));
    });
  });
}

function bindModelFilters() {
  const search = document.querySelector("[data-model-search]");
  if (search && !search.dataset.boundModelSearch) {
    search.dataset.boundModelSearch = "true";
    search.addEventListener("input", () => {
      modelSettingsFilters.search = search.value;
      document.querySelector(".data-list")?.replaceWith(modelSettingsList());
      document.querySelector("[data-model-bulk-toggle]")?.replaceWith(modelBulkToggle());
      bindModelEnablement();
      bindModelBulkToggle();
    });
  }

  const provider = document.querySelector("[data-model-provider-filter]");
  if (provider && !provider.dataset.boundModelProvider) {
    provider.dataset.boundModelProvider = "true";
    provider.addEventListener("change", () => {
      modelSettingsFilters.provider = provider.value;
      document.querySelector(".data-list")?.replaceWith(modelSettingsList());
      document.querySelector("[data-model-bulk-toggle]")?.replaceWith(modelBulkToggle());
      bindModelEnablement();
      bindModelBulkToggle();
    });
  }
}

function bindProviderFormSave() {
  const form = document.querySelector(".provider-form");
  if (!form || form.dataset.boundProviderSave) return;
  form.dataset.boundProviderSave = "true";
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const request = providerFormRequest(form);
    const keyInput = form.querySelector("#api-key");
    try {
      await saveConnectorProviderProfile(request);
      if (keyInput) keyInput.value = "";
      providerFormDraft = null;
      window.location.hash = providerSetupGate ? "new-session" : "settings-providers";
      providerSetupGate = false;
      render();
    } catch (error) {
      connectorState.error = error.message;
    }
  });

  form.querySelector("[data-provider-delete]")?.addEventListener("click", async () => {
    if (!providerFormDraft?.id) return;
    try {
      await deleteConnectorProviderProfile(providerFormDraft.id);
      providerFormDraft = null;
      window.location.hash = "settings-providers";
      render();
    } catch (error) {
      connectorState.error = error.message;
    }
  });
}

function bindSettingsNavigation() {
  document.querySelectorAll("[data-settings-entry]").forEach((control) => {
    if (control.dataset.boundSettingsEntry) return;
    control.dataset.boundSettingsEntry = "true";
    control.addEventListener("click", () => {
      lastAppRouteBeforeSettings = routeFromHash();
    }, true);
  });
  document.querySelectorAll("[data-settings-back-action]").forEach((control) => {
    if (control.dataset.boundSettingsBack) return;
    control.dataset.boundSettingsBack = "true";
    control.addEventListener("click", (event) => {
      event.preventDefault();
      if (lastAppRouteBeforeSettings === "app-start" || projects.length === 0) {
        providerSetupGate = false;
        window.location.hash = "app-start";
      } else if (lastAppRouteBeforeSettings === "new-session" && providers.length === 0) {
        providerSetupGate = true;
        providerFormDraft = null;
        window.location.hash = "settings-add-provider";
      } else {
        window.location.hash = "new-session";
      }
      render();
    });
  });
}

function providerFormRequest(form) {
  const value = (selector) => form.querySelector(selector)?.value?.trim() || "";
  return {
    kind: value("#provider-type"),
    label: value("#provider-label"),
    baseUrl: value("#api-base-url"),
    apiKey: value("#api-key"),
    providerId: providerFormDraft?.id || undefined
  };
}

function bindPanelLinks() {
  document.querySelectorAll(".tool-panel [data-file-target]").forEach((anchor) => {
    if (anchor.dataset.boundFileTarget) return;
    anchor.dataset.boundFileTarget = "true";
    anchor.addEventListener("click", async (event) => {
      event.preventDefault();
      const route = routeFromHash();
      const surface = toolSurfaceKey(route);
      if (anchor.hasAttribute("data-breadcrumb-path")) {
        try {
          await restoreConnectorExplorerScope(anchor.dataset.breadcrumbPath || "");
        } catch {
          // The state status carries the backend rejection for rendering.
        }
        appStore.setFileView(surface, "explorer");
      } else if (anchor.dataset.fileKind === "folder") {
        try {
          await toggleConnectorFolder(anchor.dataset.filePath);
        } catch {
          // The state status carries the backend rejection for rendering.
        }
        appStore.setFileView(surface, "explorer");
      } else if (anchor.dataset.filePath) {
        try {
          await openConnectorFile(anchor.dataset.filePath);
        } catch {
          // The status text is rendered from filesState below.
        }
        appStore.setFileView(surface, anchor.dataset.fileTarget);
      } else {
        appStore.setFileView(surface, anchor.dataset.fileTarget);
      }
      const panel = document.querySelector(".tool-panel");
      panel?.replaceWith(renderToolPanel("files", route));
      bindToolTabs();
      bindTerminalEmulator();
      bindPanelLinks();
      bindFileEditor();
      bindBrowserAddress();
      syncNativeBrowserSurface();
    });
  });
}

function bindBrowserAddress() {
  if (document.body.dataset.boundBrowserAddress) return;
  document.body.dataset.boundBrowserAddress = "true";
  document.body.addEventListener("submit", async (event) => {
    const form = event.target.closest?.("[data-browser-address-form]");
    if (!form) return;
    await submitBrowserAddress(form, event);
  });
  document.body.addEventListener("keydown", async (event) => {
    if (event.key !== "Enter") return;
    const input = event.target.closest?.("[data-browser-address-input]");
    if (!input) return;
    const form = input.closest("[data-browser-address-form]");
    if (!form) return;
    await submitBrowserAddress(form, event);
  });
}

async function submitBrowserAddress(form, event) {
  event.preventDefault();
  const input = form.querySelector("[data-browser-address-input]");
  const target = input?.value?.trim() || "";
  if (input) input.disabled = true;
  try {
    await openConnectorBrowser(target);
  } catch {
    // Failure text is stored on browserState and rendered in-panel below.
  } finally {
    if (input) input.disabled = false;
    const panel = document.querySelector(".tool-panel");
    panel?.replaceWith(renderToolPanel("browser", routeFromHash()));
    bindToolTabs();
    bindTerminalEmulator();
    bindPanelLinks();
    bindFileEditor();
    syncNativeBrowserSurface();
  }
}

function bindFileEditor() {
  bindNativeFileFocusTracking();
  const editor = document.querySelector("[data-file-editor]");
  if (editor && !editor.dataset.boundFileEditor) {
    editor.dataset.boundFileEditor = "true";
    editor.addEventListener("input", () => {
      editConnectorFile(editor.value);
      updateFileEditorChrome(editor.closest(".editor-tool"));
    });
    editor.addEventListener("focus", () => updateNativeFileMenuState(true, { force: true }));
    editor.addEventListener("scroll", () => syncEditorScroll(editor));
  }
  if (editor) syncEditorScroll(editor);
  updateNativeFileMenuState(isFileEditorFocused(), { force: Boolean(editor) || nativeFileEditorOpen });
}

function updateFileEditorChrome(tool) {
  if (!tool) return;
  const dirtyMark = tool.querySelector("[data-dirty-marker]");
  if (dirtyMark) dirtyMark.textContent = filesState.dirty ? " *" : "";
  const gutter = tool.querySelector(".line-number-gutter");
  if (gutter) {
    const lineCount = Math.max(1, filesState.lines?.length || 1);
    gutter.replaceChildren(...Array.from({ length: lineCount }, (_, index) =>
      h("span", { class: "line-number", text: String(index + 1) })
    ));
  }
  const highlight = tool.querySelector(".syntax-highlight");
  if (highlight) {
    highlight.replaceChildren(...syntaxHighlightNodes(filesState.content || "", filesState.currentPath || ""));
  }
}

function updateNativeFileMenuState(fileEditorOpen, options = {}) {
  const force = Boolean(options.force);
  const editable = options.editable ?? fileEditorOpen;
  if (!fileEditorOpen && !nativeFileEditorOpen && !editable && !force) return;
  if (!force && fileEditorOpen === nativeFileEditorOpen) return;
  nativeFileEditorOpen = fileEditorOpen;
  updateConnectorNativeMenuState({
    editable,
    fileEditorOpen,
    fileCanSave: fileEditorOpen
  }).catch(() => {});
}

function syncEditorScroll(editor) {
  const surface = editor.closest(".editor-surface");
  const highlight = surface?.querySelector(".syntax-highlight");
  if (!highlight) return;
  highlight.scrollTop = editor.scrollTop;
  highlight.scrollLeft = editor.scrollLeft;
}

function bindNativeFileMenu() {
  bindNativeFileShortcuts();
  globalThis.__c4osNativeMenuCommand = handleNativeFileMenuCommand;
  if (nativeFileMenuBound) return;
  const listen = globalThis.__TAURI__?.event?.listen;
  if (!listen) return;
  nativeFileMenuBound = true;
  listen("c4os://native-menu", async (event) => {
    await handleNativeFileMenuCommand(event?.payload || event);
  }).catch(() => {
    nativeFileMenuBound = false;
  });
}

async function handleNativeFileMenuCommand(id) {
  if (id === "file.openWorkspace") await openWorkspaceFileFromMenu();
  if (id === "file.saveWorkspace") await saveWorkspaceFileFromMenu();
  if (id === "file.saveFile") await saveActiveFile();
  if (id === "file.revertFile") await revertActiveFile();
}

async function openWorkspaceFileFromMenu() {
  try {
    await openConnectorWorkspaceFile();
    window.location.hash = "new-session";
    render();
  } catch {
    render();
  }
}

async function saveWorkspaceFileFromMenu() {
  try {
    await saveConnectorWorkspaceFile();
  } catch {
    render();
  }
}

function bindNativeFileFocusTracking() {
  if (nativeFileFocusBound) return;
  nativeFileFocusBound = true;
  document.addEventListener("focusin", (event) => {
    const fileEditorFocused = Boolean(event.target?.closest?.("[data-file-editor]"));
    updateNativeFileMenuState(fileEditorFocused, {
      editable: isNativeEditableTarget(event.target),
      force: true
    });
  });
}

function isFileEditorFocused() {
  return Boolean(document.activeElement?.closest?.("[data-file-editor]"));
}

function isNativeEditableTarget(target) {
  if (!target?.closest) return false;
  return Boolean(target.closest("input, textarea, [contenteditable='true'], [data-terminal-emulator], [data-browser-address-input]"));
}

function bindNativeFileShortcuts() {
  if (nativeFileShortcutsBound) return;
  nativeFileShortcutsBound = true;
  window.addEventListener("keydown", async (event) => {
    if (!(event.metaKey || event.ctrlKey)) return;
    if (event.altKey || event.shiftKey) return;
    if (!document.querySelector("[data-file-editor]")) return;
    const key = event.key.toLowerCase();
    if (key === "s") {
      event.preventDefault();
      await saveActiveFile();
    } else if (key === "r") {
      event.preventDefault();
      await revertActiveFile();
    }
  });
}

async function saveActiveFile() {
  if (!filesState.currentPath) return;
  const editor = document.querySelector("[data-file-editor]");
  const current = editor?.value ?? filesState.content ?? "";
  const selection = editor
    ? { start: editor.selectionStart, end: editor.selectionEnd, direction: editor.selectionDirection }
    : null;
  try {
    await saveConnectorFile(filesState.currentPath, current);
  } catch {
    // The state status carries the backend rejection for rendering.
  }
  if (editor && document.contains(editor)) {
    updateFileEditorChrome(editor.closest(".editor-tool"));
    if (selection) {
      editor.setSelectionRange(selection.start, selection.end, selection.direction);
    }
    updateNativeFileMenuState(true, { force: true });
  } else {
    refreshFileToolPanel();
  }
}

async function revertActiveFile() {
  if (!filesState.currentPath) return;
  try {
    await openConnectorFile(filesState.currentPath);
  } catch {
    // The state status carries the backend rejection for rendering.
  }
  refreshFileToolPanel();
}

function refreshFileToolPanel() {
  document.querySelector(".tool-panel")?.replaceWith(renderToolPanel("files", routeFromHash()));
  bindToolTabs();
  bindPanelLinks();
  bindFileEditor();
}

function updateShellSessionDom() {
  const shell = document.querySelector(".app-shell");
  if (!shell) return render();
  shell.dataset.screen = "chat-session";
  document.body.dataset.route = "chat-session";
  shell.querySelector(".topbar strong").textContent = workspace.session;
  shell.querySelectorAll(".project-row").forEach((row) => {
    row.classList.toggle("is-active", projectDatasetIdentity(row.dataset) === activeProjectIdentity());
  });
  shell.querySelectorAll(".session-row").forEach((row) => {
    row.classList.toggle("is-active", (row.dataset.sessionId || row.dataset.sessionTarget) === (workspace.sessionId || workspace.session));
  });
  const workbench = shell.querySelector(".workbench");
  const currentMain = workbench.querySelector(":scope > main");
  currentMain?.replaceWith(renderThread());
  shell.querySelector(".tool-panel")?.replaceWith(renderToolPanel(activeToolForRoute("chat-session"), "chat-session"));
  bindWorkLogDisclosure();
  bindMessage();
  bindConnectorRun();
  bindToolTabs();
  bindTerminalEmulator();
  bindPanelLinks();
  bindFileEditor();
  syncNativeBrowserSurface();
  applyShellState();
}

function applyShellState() {
  const shell = document.querySelector(".app-shell");
  if (!shell) return;
  const route = routeFromHash();
  const routeOwnsToolPanel = route === "file-explorer" || route === "file-editor" || route === "terminal";
  shell.classList.toggle("is-left-collapsed", appStore.shell.leftCollapsed);
  shell.classList.toggle("is-right-collapsed", !routeOwnsToolPanel && appStore.shell.rightCollapsed);
  shell.querySelector("[data-panel-toggle='left']")?.setAttribute("aria-pressed", String(appStore.shell.leftCollapsed));
  shell.querySelector("[data-panel-toggle='right']")?.setAttribute("aria-pressed", String(!routeOwnsToolPanel && appStore.shell.rightCollapsed));
  if (appStore.shell.leftWidth) shell.style.setProperty("--left-panel", `${appStore.shell.leftWidth}px`);
  if (appStore.shell.rightWidth) shell.style.setProperty("--right-panel", `${appStore.shell.rightWidth}px`);
  if (appStore.shell.terminalBottom) {
    shell.querySelector(".terminal-tool")?.style.setProperty("--terminal-agent", `${appStore.shell.terminalBottom}px`);
  }
}

function bindWorkspaceOpen() {
  if (!connectorState.connector.available) return;
  if (workspaceOpenBound) return;
  workspaceOpenBound = true;
  document.body.addEventListener("click", async (event) => {
    const fileControl = event.target.closest?.("[data-open-workspace-file]");
    if (fileControl) {
      event.preventDefault();
      await openWorkspaceFileFromMenu();
      return;
    }

    const control = event.target.closest?.("[data-open-workspace]");
    if (!control) return;
    event.preventDefault();
    try {
    if (control.dataset.workspaceFilePath) {
      await openConnectorWorkspaceFile(control.dataset.workspaceFilePath);
    } else {
      await openConnectorWorkspace(control.dataset.workspacePath || undefined, {
        mergeProjects: control.dataset.openWorkspace === "append"
      });
    }
      window.location.hash = "new-session";
      render();
    } catch {
      render();
    }
  });
}

//--------------------------------------------------------------------//
// Start and workspace shell routes
//--------------------------------------------------------------------//

/**
 * Render the trusted-folder entry screen.
 */
function renderStart() {
  const recentProjects = connectorState.connector.available
    ? projects.slice(0, 3)
    : [];
  const hasRecentProjects = recentProjects.length > 0;
  const title = connectorState.loading
    ? "Loading workspace state"
    : connectorState.error
      ? "Workspace state unavailable"
      : "Open a folder to start working";
  const lead = connectorState.error
    ? connectorState.error
    : "Local files, instructions, skills, runtime policy, approvals, and workspace state stay unavailable until a trusted project folder scopes the session.";
  return h("main", { id: "main", class: `start-view${hasRecentProjects ? "" : " is-empty"}`, "data-screen": "app-start", tabindex: "-1" }, [
    h("section", { class: "start-copy" }, [
      h("p", { class: "kicker", text: "No trusted project folder" }),
      h("h1", { text: title }),
      h("p", { class: "lead", text: lead }),
      h("div", { class: "action-row" }, [
        button("Open Folder", "button primary", null, { "data-open-workspace": "" }),
        button("Clone Repository"),
        button("Open Workspace File", "button secondary", null, { "data-open-workspace-file": "" }),
        link("settings-providers", "button secondary", [icon("settings"), h("span", { text: "Settings" })], { "data-settings-entry": "" })
      ])
    ]),
    hasRecentProjects ? h("section", { class: "recent-panel", "aria-labelledby": "recent-title" }, [
      h("h2", { id: "recent-title", text: "Recent Workspaces" }),
      ...recentProjects.map((project, index) =>
        link("new-session", "workspace-row", [
          h("strong", { text: project.name }),
          h("span", { text: ["Trusted", "2 roots", "Saved"][index] })
        ], {
          "data-open-workspace": "",
          "data-workspace-path": project.rootPath || project.root_path || "",
          "data-workspace-file-path": project.workspaceFilePath || project.workspace_file_path || ""
        })
      )
    ]) : null
  ]);
}

function routeBootstrappedWorkspace() {
  const route = routeFromHash();
  if (route !== "app-start") return false;
  if (!connectorState.connector.available || connectorState.loading || connectorState.error) return false;
  if (!workspace.rootPath) return false;
  window.location.hash = workspace.session ? "chat-session" : "new-session";
  return true;
}

/**
 * Render the three-panel app shell used by non-settings routes.
 */
function renderShell(route) {
  const chat = route === "chat-session";
  const tool = activeToolForRoute(route);
  return h("div", { class: "app-shell", "data-screen": route }, [
    renderSidebar(chat),
    resizeHandle("left"),
    h("section", { id: "main", class: "workbench", tabindex: "-1" }, [
      h("header", { class: "topbar" }, [
        iconButton("Collapse left panel", "panelLeft", "icon-button", { "data-panel-toggle": "left", "aria-pressed": "false" }),
        h("strong", { text: chat ? workspace.session : workspace.project }),
        iconButton("Collapse right panel", "panelRight", "icon-button", { "data-panel-toggle": "right", "aria-pressed": "false" })
      ]),
      chat ? renderThread() : renderNewSession(route)
    ]),
    resizeHandle("right"),
    renderToolPanel(tool, route)
  ]);
}

function activeToolForRoute(route) {
  if (route === "file-explorer" || route === "file-editor" || route === "terminal") {
    return defaultToolForRoute(route);
  }
  const key = toolSurfaceKey(route);
  return appStore.toolFor(key, defaultToolForRoute(route));
}

function setActiveToolForRoute(route, tool) {
  appStore.setTool(toolSurfaceKey(route), tool);
}

function toolSurfaceKey(route) {
  if (route === "chat-session") return sessionSurfaceKey(activeProjectIdentity(), workspace.sessionId || workspace.session || "untitled");
  if (route === "new-session" || route === "providers-popover" || route === "models-popover") return `new:${activeProjectIdentity()}`;
  return `route:${route}`;
}

function defaultToolForRoute(route) {
  if (route === "file-explorer" || route === "file-editor") return "files";
  if (route === "terminal") return "terminal";
  return "browser";
}

function composerSurfaceKey() {
  return routeFromHash() === "chat-session" ? appStore.activeSessionKey() : `new:${activeProjectIdentity()}`;
}

/**
 * Render project/session navigation for the left shell panel.
 */
function renderSidebar(chat) {
  const activeProject = activeProjectIdentity();
  return h("aside", { class: "sidebar", "aria-label": "Projects" }, [
    h("label", { class: "search-field" }, [icon("search"), h("span", { class: "sr-only", text: "Search projects" }), h("input", { type: "search", placeholder: "Search projects" })]),
    h("div", { class: "nav-section" }, [
      h("div", { class: "section-head" }, [h("span", { text: "Projects" }), iconButton("Add Project", "add", "icon-button", { "data-open-workspace": "append" })]),
      ...projects.flatMap((project) => [
        h("a", {
          class: `project-row${projectIdentity(project) === activeProject && !chat ? " is-active" : ""}`,
          href: "#new-session",
          "data-project-target": project.name,
          "data-project-id": project.id || "",
          "data-project-root-path": project.rootPath || project.root_path || ""
        }, [icon("folder"), h("span", { text: project.name }), h("span", { class: "row-tools" }, [icon("pencil"), icon("trash")])]),
        ...project.sessions.map((session) => {
          const label = sessionLabel(session);
          const id = sessionId(session);
          return h("a", {
          class: `session-row${chat && (id || label) === (workspace.sessionId || workspace.session) ? " is-active" : ""}`,
          href: "#chat-session",
          "data-project-target": project.name,
          "data-project-id": project.id || "",
          "data-project-root-path": project.rootPath || project.root_path || "",
          "data-session-target": label,
          "data-session-id": id
        }, [h("span", { text: label })]);
        })
      ])
    ]),
    link("settings-providers", "settings-entry", [icon("settings"), h("span", { text: "Settings" })], { "data-settings-entry": "" })
  ]);
}

function sessionLabel(session) {
  return typeof session === "string" ? session : session?.label || session?.title || "";
}

function sessionId(session) {
  return typeof session === "string" ? "" : session?.id || session?.sessionId || "";
}

function projectIdentity(project) {
  return project?.id || project?.rootPath || project?.root_path || project?.name || "";
}

function activeProjectIdentity() {
  return workspace.projectId || workspace.rootPath || workspace.project;
}

function projectDatasetIdentity(dataset) {
  return dataset.projectId || dataset.projectRootPath || dataset.projectTarget || "";
}

/**
 * Render a draggable separator for a shell side panel.
 */
function resizeHandle(side) {
  return h("div", {
    class: `resize-handle ${side}-resizer`,
    role: "separator",
    tabindex: "0",
    "aria-label": side === "left" ? "Resize left panel" : "Resize right panel",
    "aria-orientation": "vertical",
    "data-resize-panel": side
  });
}

/**
 * Render the empty session route and any route-driven composer popover.
 */
function renderNewSession(route) {
  const popover = route === "providers-popover" ? providerPopover() : route === "models-popover" ? modelPopover() : null;
  return h("main", { class: "empty-workspace" }, [
    h("h1", { text: `What should we build in ${workspace.project}?` }),
    composer("Do anything", { popover })
  ]);
}

/**
 * Render the shared prompt composer surface.
 */
function composer(placeholder, options = {}) {
  const readonly = options.readonlyContext;
  const surface = options.surface || composerSurfaceKey();
  const state = appStore.composerFor(surface);
  return h("section", { class: "composer", "aria-label": "Prompt composer", "data-composer-surface": surface }, [
    h("input", { class: "attachment-input", type: "file", multiple: true, tabindex: "-1", "aria-hidden": "true" }),
    h("div", { class: "attachment-preview", "data-attachments": "", hidden: true, "aria-label": "Attached files" }),
    h("div", { class: "prompt-box", role: "textbox", "aria-label": "Prompt", "aria-multiline": "true", contenteditable: "true", "data-placeholder": placeholder, text: state.prompt }),
    h("div", { class: "composer-controls" }, [
      iconButton("Attach File", "paperclip", "icon-button", { "data-attach-button": "" }),
      h("button", { class: "chip", type: "button", "aria-label": `Approval policy: ${state.approval}`, "data-popover-trigger": "approval" }, [icon("shield"), h("span", { text: state.approval })]),
      h("span", { class: "spacer" }),
      iconButton("Use Microphone", "mic"),
      iconButton("Send Prompt", "send", "icon-button send-button")
    ]),
    h("div", { class: "context-strip" }, [
      readonly ? h("span", { class: "chip readonly-chip", "aria-label": "Branch locked for this chat" }, [icon("gitBranch"), h("span", { text: state.branch })]) : h("button", { class: "chip", type: "button", "aria-label": `Branch: ${state.branch}`, "data-popover-trigger": "branch" }, [icon("gitBranch"), h("span", { text: state.branch })]),
      h("span", { class: "spacer" }),
      readonly ? h("span", { class: "chip readonly-chip", "aria-label": "Model locked for this chat" }, [icon("bot"), h("span", { text: selectedModelLabel(surface) })]) : h("button", { class: "chip", type: "button", "aria-label": `Model: ${selectedModelLabel(surface)}`, "data-local-picker-trigger": "models" }, [icon("bot"), h("span", { text: selectedModelLabel(surface) })])
    ]),
    composerPopover("approval", ["Ask for approval", "Approve for me"]),
    readonly ? null : composerPopover("branch", ["main", "feature/trust-shell", "+ Create branch"]),
    state.openPicker ? localComposerPicker(state.openPicker) : null,
    options.popover
  ]);
}

/**
 * Render a small composer option menu.
 */
function composerPopover(kind, rows) {
  return h("div", { class: `composer-popover ${kind}-popover`, hidden: true, "data-popover": kind }, rows.map((row) =>
    h("button", { class: "popover-row", type: "button", "data-popover-option": row }, [h("span", { text: row })])
  ));
}

function localComposerPicker(kind) {
  if (kind === "providers") {
    return h("aside", { class: "popover", "aria-label": "Provider selector", "data-local-picker": "providers" }, [
      h("header", { class: "popover-title" }, [h("strong", { text: "Providers" })]),
      h("div", { class: "popover-scroll" }, providers.map((provider) =>
        h("button", { class: "popover-row", type: "button", "data-local-provider": provider.id || provider.label }, [h("span", { text: provider.label }), icon("chevronRight")])
      ))
    ]);
  }
  const selectableModels = enabledModels();
  return h("aside", { class: "popover model-picker", "aria-label": "Model selector", "data-local-picker": "models" }, [
    h("header", { class: "popover-backbar" }, [
      h("button", { class: "popover-back", type: "button", "aria-label": "Back to providers", "data-local-picker-trigger": "providers" }, [icon("chevronLeft"), h("strong", { text: activeProviderLabel() })])
    ]),
    h("div", { class: "popover-scroll" }, selectableModels.map((model) => h("button", { class: `popover-row${model.label === selectedModelForSurface(composerSurfaceKey()) ? " is-selected" : ""}`, type: "button", "data-local-model": model.label }, [
      h("span", { text: model.label }),
      model.label === selectedModelForSurface(composerSurfaceKey()) ? h("span", { "aria-label": "Current model" }, [icon("check")]) : h("span")
    ])))
  ]);
}

function enabledModels() {
  return models.filter((model) => model.enabled !== false);
}

function activeProviderLabel() {
  const activeModel = models.find((model) => model.label === workspace.model) || models.find((model) => model.enabled !== false);
  return activeModel?.provider || providers[0]?.label || "Provider";
}

function selectedModelForSurface(surface) {
  return appStore.composerFor(surface).model || (routeFromHash() === "chat-session" ? workspace.model : "") || enabledModels()[0]?.label || "";
}

function selectedModelLabel(surface) {
  return selectedModelForSurface(surface) || "Select model";
}

function updateComposerPicker(composerNode, surface) {
  const current = composerNode.querySelector("[data-local-picker]");
  current?.remove();
  const picker = appStore.composerFor(surface).openPicker;
  if (picker) composerNode.append(localComposerPicker(picker));
  bindComposerPickers();
}

/**
 * Render the provider selector route popover.
 */
function providerPopover() {
  return h("aside", { class: "popover", "aria-label": "Provider selector" }, [
    h("header", { class: "popover-title" }, [h("strong", { text: "Providers" })]),
    h("div", { class: "popover-scroll" }, providers.map((provider) => link("models-popover", "popover-row", [h("span", { text: provider.label }), icon("chevronRight")])))
  ]);
}

/**
 * Render the model selector route popover.
 */
function modelPopover() {
  const selectableModels = enabledModels();
  return h("aside", { class: "popover model-picker", "aria-label": "Model selector" }, [
    h("header", { class: "popover-backbar" }, [link("providers-popover", "popover-back", [icon("chevronLeft"), h("strong", { text: activeProviderLabel() })])]),
    h("div", { class: "popover-scroll" }, selectableModels.map((model) => link("new-session", `popover-row${model.label === selectedModelForSurface(composerSurfaceKey()) ? " is-selected" : ""}`, [
      h("span", { text: model.label }),
      model.label === selectedModelForSurface(composerSurfaceKey()) ? h("span", { "aria-label": "Current model" }, [icon("check")]) : h("span")
    ])))
  ]);
}

/**
 * Render the chat transcript fixture with disclosure and activity cards.
 */
function renderThread() {
  const latestTurn = threadTurns[threadTurns.length - 1];
  return h("main", { class: "thread-view" }, [
    h("div", { class: "thread-list", "aria-label": "Session messages" }, threadTurns.flatMap((turn, index) => renderTurn(turn, index === threadTurns.length - 1))),
    h("div", { class: "composer-dock" }, [
      permissionPrompt(latestTurn),
      composer("Ask for follow-up changes", { readonlyContext: true })
    ])
  ]);
}

function renderTurn(turn, interactive) {
  return [
    h("article", { class: "message user thread-item", "data-turn-id": turn.id, "data-turn-part": "user" }, [h("p", { text: turn.user })]),
    workLog(turn),
    h("article", { class: `message agent thread-item${turn.pending && !turn.agent ? " is-pending" : ""}`, "data-turn-id": turn.id, "data-turn-part": "agent" }, [
      h("div", { class: "markdown-body" }, renderMarkdown(turn.agent || "Waiting on connector")),
      h("div", { class: "message-extra markdown-body" }, renderMarkdown(turn.extra)),
      interactive ? h("button", { class: "text-button", type: "button", "data-toggle": "message", "aria-expanded": "false" }, ["Show More"]) : null
    ])
  ];
}

function workLog(turn) {
  const expanded = turn.pending || turn.workExpanded;
  const label = [
    h("span", { text: workLabel(turn) }),
    icon("chevronRight")
  ];
  return h("section", { class: `work-log thread-item${turn.pending ? " is-pending" : ""}${expanded ? " is-expanded" : ""}`, "data-turn-id": turn.id, "data-turn-part": "work" }, [
    h("button", {
      class: "work-log-toggle",
      type: "button",
      "data-work-toggle": turn.id,
      "aria-expanded": expanded ? "true" : "false"
    }, label),
    expanded ? h("div", { class: "work-log-body markdown-body" }, renderMarkdown(workLogText(turn))) : null
  ]);
}

function appendTurnToThread(turn) {
  const list = document.querySelector(".thread-list");
  if (!list || list.querySelector(`[data-turn-id='${cssEscape(turn.id)}']`)) return;
  const fragment = document.createDocumentFragment();
  for (const node of renderTurn(turn, true)) fragment.append(node);
  list.append(fragment);
  refreshLatestTurnControls();
  bindWorkLogDisclosure();
  bindMessage();
}

function updateTurnDom(turn) {
  if (!turn) return;
  updateWorkLogDom(turn);
  updateAgentTurnDom(turn);
}

function updateWorkLogDom(turn) {
  const node = document.querySelector(`[data-turn-id='${cssEscape(turn.id)}'][data-turn-part='work']`);
  if (!node) return;
  const replacement = workLog(turn);
  node.className = replacement.className;
  node.replaceChildren(...Array.from(replacement.childNodes));
  bindWorkLogDisclosure();
}

function updateAgentTurnDom(turn) {
  const node = document.querySelector(`[data-turn-id='${cssEscape(turn.id)}'][data-turn-part='agent']`);
  if (!node) return;
  node.classList.toggle("is-pending", Boolean(turn.pending && !turn.agent));
  node.querySelector(".markdown-body")?.replaceChildren(...renderMarkdown(turn.agent || "Waiting on connector"));
  node.querySelector(".message-extra")?.replaceChildren(...renderMarkdown(turn.extra));
}

function refreshLatestTurnControls() {
  const agentMessages = Array.from(document.querySelectorAll("[data-turn-part='agent']"));
  agentMessages.forEach((message, index) => {
    const latest = index === agentMessages.length - 1;
    const existing = message.querySelector("[data-toggle='message']");
    if (latest && !existing) {
      message.append(h("button", { class: "text-button", type: "button", "data-toggle": "message", "aria-expanded": "false" }, ["Show More"]));
    } else if (!latest && existing) {
      existing.remove();
    }
  });
}

function cssEscape(value) {
  return window.CSS?.escape ? window.CSS.escape(value) : String(value).replace(/'/g, "\\'");
}

function workLabel(turn) {
  const prefix = turn.pending ? "Working" : "Worked";
  return `${prefix} for ${formatDuration((turn.completedAt || Date.now()) - turn.startedAt)}`;
}

function workLogText(turn) {
  const logs = (turn.logs || []).filter(Boolean);
  return logs.length > 0 ? logs.map((log) => `- ${log}`).join("\n") : "- Waiting on connector";
}

function formatDuration(ms) {
  const seconds = Math.max(1, Math.round(ms / 1000));
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  return minutes > 0 ? `${minutes}m ${rest}s` : `${seconds}s`;
}

function permissionPrompt(turn) {
  if (!turn?.permission) return null;
  return h("section", { class: "permission-prompt", role: "dialog", "aria-label": "Permission request" }, [
    h("p", { text: turn.permission.question }),
    h("code", {}, [turn.permission.command]),
    h("div", { class: "permission-options" }, [
      button("Yes"),
      button("Yes, don't ask again"),
      button("No")
    ])
  ]);
}

/**
 * Render the right-side Browser, Files, or Terminal tool panel.
 */
function renderToolPanel(active, route) {
  const surface = toolSurfaceKey(route);
  const fileView = appStore.fileViewFor(surface, route === "file-editor" ? "editor" : "explorer");
  const tabs = [
    ["Browser", "browser", "globe"],
    ["Files", "files", "file"],
    ["Terminal", "terminal", "terminal"]
  ];
  return h("aside", { class: "tool-panel", "aria-label": "Workspace tools", "data-scoped-region": "tool-panel" }, [
    h("nav", { class: "tool-tabs", "aria-label": "Workspace tool tabs" }, tabs.map(([label, key, iconName]) =>
      h("button", {
        class: `tool-tab${active === key ? " is-active" : ""}`,
        type: "button",
        "aria-label": label,
        "aria-pressed": active === key ? "true" : "false",
        "data-tool-tab": key
      }, [icon(iconName), h("span", { text: label })])
    )),
    active === "files" ? filesTool(fileView === "editor") : active === "terminal" ? terminalTool(route) : browserTool()
  ]);
}

function browserTool() {
  const artifact = activeBrowserArtifact();
  if (artifact?.safePreview) {
    const preview = artifact.safePreview;
    return h("section", { class: "tool-body browser-tool", "data-artifact-preview": artifact.id }, [
      h("div", { class: "address-bar" }, [icon("globe"), h("span", { text: preview.url || browserState.url })]),
      h("div", { class: "preview-surface artifact-preview" }, [
        h("header", { class: "artifact-preview-header" }, [
          h("h2", { text: preview.title || artifact.title }),
          h("p", { text: preview.summary || "Generated artifact preview" })
        ]),
        artifactPreviewBody(artifact)
      ])
    ]);
  }
  const previewMode = browserState.previewMode || browserState.preview_mode || "public";
  const frameAttrs = {
    class: "browser-frame",
    title: browserState.title || "Browser",
    "data-browser-frame": ""
  };
  if (previewMode !== "public") frameAttrs.sandbox = "allow-forms allow-scripts";
  if (browserState.html) frameAttrs.src = browserDocumentUrl(browserState.html);
  else frameAttrs.src = browserState.url || "about:blank";
  const usesNativeBrowser =
    (previewMode === "public" && isPublicBrowserUrl(browserState.url)) ||
    (previewMode === "local-file" && !localBrowserFileIsMarkdown());
  const localFilePreview = localFileBrowserPreviewBody(previewMode);

  return h("section", {
    class: "tool-body browser-tool",
    "data-browser-preview": previewMode,
    "data-local-browser-file": browserState.localPath || browserState.local_path || null
  }, [
    h("form", { class: "address-bar", "data-browser-address-form": "" }, [
      icon("globe"),
      h("input", {
        type: "text",
        value: browserState.url || "",
        "aria-label": "Browser address",
        "data-browser-address-input": "",
        autocomplete: "off",
        spellcheck: "false"
      })
    ]),
    h("div", { class: "browser-preview" }, [
      browserStatusIsFailure() ? h("p", { class: "browser-status", role: "status", text: browserState.status }) : null,
      usesNativeBrowser ? h("div", {
        class: "native-browser-frame",
        "data-native-browser-frame": "",
        "data-native-browser-url": browserState.url,
        role: "presentation"
      }) : localFilePreview || h("iframe", frameAttrs)
    ])
  ]);
}

function isPublicBrowserUrl(url) {
  return /^https?:\/\//i.test(String(url || "").trim());
}

function localFileBrowserPreviewBody(previewMode) {
  if (previewMode !== "local-file") return null;
  if (localBrowserFileIsMarkdown()) {
    return h("article", {
      class: "browser-local-render artifact-render artifact-markdown markdown-body",
      "data-local-file-renderer": "markdown"
    }, renderMarkdown(browserState.html || ""));
  }
  return null;
}

function localBrowserFileIsMarkdown() {
  const target = String(browserState.localPath || browserState.local_path || browserState.url || "").toLowerCase();
  return target.endsWith(".md") || target.endsWith(".markdown");
}

function syncNativeBrowserSurface() {
  bindNativeBrowserResizeSync();
  window.requestAnimationFrame(async () => {
    const frame = document.querySelector("[data-native-browser-frame]");
    if (!frame) {
      unobserveNativeBrowserFrame();
      if (!nativeBrowserSurfaceActive) return;
      try {
        await syncConnectorNativeBrowser({ visible: false });
        nativeBrowserSurfaceActive = false;
      } catch {
        // Failure state is rendered only when the Browser panel is active.
      }
      return;
    }
    observeNativeBrowserFrame(frame);
    const rect = frame.getBoundingClientRect();
    const topInset = nativeBrowserTopInset(frame);
    const syncRequest = {
      url: frame.dataset.nativeBrowserUrl || browserState.url,
      x: rect.x,
      y: rect.y + topInset,
      width: rect.width,
      height: Math.max(1, rect.height),
      visible: rect.width > 0 && rect.height > 0
    };
    try {
      await syncConnectorNativeBrowser(syncRequest);
      nativeBrowserSurfaceActive = rect.width > 0 && rect.height > 0;
    } catch {
      const panel = document.querySelector(".tool-panel");
      panel?.replaceWith(renderToolPanel("browser", routeFromHash()));
      bindToolTabs();
      bindPanelLinks();
      bindFileEditor();
    }
  });
}

function nativeBrowserTopInset(frame) {
  const addressInput = frame.closest(".browser-tool")?.querySelector("[data-browser-address-input]");
  if (!addressInput) return 0;
  return Math.max(0, Math.round(addressInput.getBoundingClientRect().height - 2));
}

function bindNativeBrowserResizeSync() {
  if (nativeBrowserWindowResizeBound) return;
  nativeBrowserWindowResizeBound = true;
  window.addEventListener("resize", () => {
    if (nativeBrowserSurfaceActive || document.querySelector("[data-native-browser-frame]")) {
      syncNativeBrowserSurface();
    }
  });
}

function observeNativeBrowserFrame(frame) {
  if (nativeBrowserObservedFrame === frame) return;
  unobserveNativeBrowserFrame();
  if (!("ResizeObserver" in window)) return;
  nativeBrowserResizeObserver = new ResizeObserver(() => syncNativeBrowserSurface());
  nativeBrowserResizeObserver.observe(frame);
  nativeBrowserObservedFrame = frame;
}

function unobserveNativeBrowserFrame() {
  nativeBrowserResizeObserver?.disconnect();
  nativeBrowserResizeObserver = null;
  nativeBrowserObservedFrame = null;
}

function browserStatusIsFailure() {
  const status = String(browserState.status || "").trim();
  return Boolean(status && status !== "Ready" && status !== "Opened");
}

function activeBrowserArtifact() {
  if ((browserState.previewMode || browserState.preview_mode) !== "artifact") return null;
  const artifactId = browserState.artifactId || browserState.artifact_id;
  return artifactState.find((artifact) => artifact.id === artifactId) || artifactState[0] || null;
}

function artifactPreviewBody(artifact) {
  const preview = artifact.safePreview || artifact.safe_preview || {};
  const type = artifactPreviewType(artifact);
  const content = artifactPreviewContent(preview);
  if (type === "html") {
    return h("iframe", {
      class: "artifact-frame",
      title: preview.title || artifact.title,
      sandbox: "allow-scripts",
      src: artifactPreviewDocumentUrl(preview.html || content),
      "data-artifact-frame": "",
      "data-artifact-id": artifact.id
    });
  }
  if (type === "markdown") {
    return h("article", { class: "artifact-render artifact-markdown markdown-body", "data-artifact-renderer": "markdown" }, renderMarkdown(content));
  }
  if (type === "image") {
    const source = artifactPreviewSource(preview);
    return artifactMediaPreview("figure", "artifact-image", "image", source, "Image preview is unavailable.", () => h("img", {
      src: source,
      alt: preview.title || artifact.title || "Artifact image"
    }));
  }
  if (type === "pdf") {
    const source = artifactPreviewSource(preview);
    return artifactMediaPreview("div", "artifact-pdf", "pdf", source, "PDF preview is unavailable.", () => h("iframe", {
      src: source,
      title: preview.title || artifact.title || "PDF preview"
    }));
  }
  if (type === "json") {
    return artifactCodePreview("json", formatJsonPreview(content));
  }
  if (type === "code") {
    return artifactCodePreview("code", content);
  }
  return h("div", { class: "artifact-render artifact-text", "data-artifact-renderer": "text" }, [
    h("pre", { text: content || "Plain text preview is unavailable." })
  ]);
}

function artifactMediaPreview(tag, className, renderer, source, emptyMessage, renderSource) {
  return h(tag, { class: `artifact-render ${className}`, "data-artifact-renderer": renderer }, [
    source ? renderSource() : artifactEmptyState(emptyMessage)
  ]);
}

function artifactCodePreview(renderer, content) {
  return h("div", { class: "artifact-render artifact-code", "data-artifact-renderer": renderer }, [
    h("pre", {}, [h("code", { text: content })])
  ]);
}

function artifactPreviewType(artifact) {
  const preview = artifact.safePreview || artifact.safe_preview || {};
  const mime = String(artifact.mimeType || artifact.mime_type || preview.mimeType || preview.mime_type || "").toLowerCase();
  const filename = String(artifact.filename || artifact.fileName || artifact.file_name || preview.filename || preview.fileName || preview.file_name || "");
  const extension = filename.toLowerCase().split(".").pop() || "";
  if (preview.html || mime.includes("html") || extension === "html" || extension === "htm" || artifact.kind === "html") return "html";
  if (mime.includes("markdown") || ["md", "markdown"].includes(extension) || artifact.kind === "markdown") return "markdown";
  if (mime.startsWith("image/") || ["png", "jpg", "jpeg", "gif", "webp", "svg"].includes(extension) || artifact.kind === "image") return "image";
  if (mime === "application/pdf" || extension === "pdf" || artifact.kind === "pdf") return "pdf";
  if (mime.includes("json") || extension === "json" || artifact.kind === "json") return "json";
  if (isCodePreview(mime, extension, artifact.kind)) return "code";
  return "text";
}

function isCodePreview(mime, extension, kind) {
  return kind === "code" ||
    mime.includes("javascript") ||
    mime.includes("typescript") ||
    mime.includes("x-python") ||
    mime.includes("x-rust") ||
    mime.includes("x-shellscript") ||
    ["js", "jsx", "ts", "tsx", "py", "rs", "go", "java", "rb", "php", "sh", "css", "sql", "toml", "yaml", "yml"].includes(extension);
}

function artifactPreviewContent(preview) {
  return String(preview.content ?? preview.text ?? preview.markdown ?? preview.code ?? preview.html ?? "");
}

function artifactPreviewSource(preview) {
  return preview.dataUrl || preview.data_url || preview.src || preview.url || "";
}

function artifactEmptyState(message) {
  return h("p", { class: "artifact-empty", text: message });
}

function formatJsonPreview(content) {
  try {
    return JSON.stringify(JSON.parse(content), null, 2);
  } catch {
    return content || "{}";
  }
}

function artifactPreviewDocumentUrl(html) {
  return `data:text/html;charset=utf-8,${encodeURIComponent(html)}`;
}

function browserDocumentUrl(html) {
  return `data:text/html;charset=utf-8,${encodeURIComponent(html)}`;
}

/**
 * Render either the file list or lightweight editor fixture.
 */
function filesTool(editor) {
  if (editor) {
    const path = filesState.currentPath || filesState.breadcrumbs?.[filesState.breadcrumbs.length - 1] || "main.js";
    const content = filesState.content ?? filesState.lines.join("\n");
    return h("section", { class: "tool-body editor-tool no-status" }, [
      h("nav", { class: "breadcrumbs", "aria-label": "File breadcrumbs" }, [
        h("button", { type: "button", "data-file-target": "explorer", "data-breadcrumb-path": "" }, [filesState.breadcrumbs[0] || workspace.project]),
        ...filesState.breadcrumbs.slice(1, -1).flatMap((crumb, index) => [
          h("span", { text: ">" }),
          h("button", { type: "button", "data-file-target": "explorer", "data-breadcrumb-path": breadcrumbPathAt(index + 1) }, [crumb])
        ]),
        h("span", { text: ">" }),
        h("span", {}, [
          filesState.breadcrumbs[filesState.breadcrumbs.length - 1] || path,
          h("span", { "data-dirty-marker": "", text: filesState.dirty ? " *" : "" })
        ])
      ]),
      h("div", { class: "code-pane", tabindex: "0" }, [
        h("div", { class: "line-number-gutter", "aria-hidden": "true" }, (content.split("\n").length ? content.split("\n") : [""]).map((_, index) =>
          h("span", { class: "line-number", text: String(index + 1) })
        )),
        h("div", { class: "editor-compat-line", "aria-hidden": "true" }, [
          h("code", { class: "line-code", contenteditable: "true", spellcheck: "false" })
        ]),
        h("div", { class: "editor-surface" }, [
          h("pre", { class: `syntax-highlight ${languageClass(path)}`, "aria-hidden": "true" }, syntaxHighlightNodes(content, path)),
          h("textarea", { class: "code-editor", "aria-label": `Code editor for ${path}`, spellcheck: "false", "data-file-editor": "", text: content })
        ])
      ])
    ]);
  }
  return h("section", { class: "tool-body file-tool" }, [
    h("h2", { text: workspace.project }),
    h("div", { class: "file-list", "aria-label": "Project files" }, filesState.roots.map(([name, iconName, target, rowPath, depth = "0", gitState = "", ignoredState = ""]) => {
      const path = rowPath || name;
      const openFile = iconName === "file" && filesState.currentPath === path;
      return h("button", {
        class: fileRowClass({ name, kind: iconName, path, openFile, gitState, ignoredState }),
        type: "button",
        "data-file-target": target === "file-editor" ? "editor" : "explorer",
        "data-file-path": path,
        "data-file-kind": iconName,
        "data-git-state": gitState || null,
        "data-ignored": ignoredState || null,
        "aria-current": openFile ? "true" : null,
        "aria-expanded": iconName === "folder" ? String(Boolean(filesState.expandedFolders?.[path])) : null,
        style: `--file-depth: ${Number(depth) || 0}`
      }, [icon(iconName), h("span", { text: name })]);
    }))
  ]);
}

function breadcrumbPathAt(crumbIndex) {
  const parts = (filesState.currentPath || "").split("/").filter(Boolean);
  return parts.slice(0, crumbIndex).join("/");
}

function fileRowClass({ name, kind, path, openFile, gitState, ignoredState }) {
  const classes = ["file-row"];
  if (kind === "file") classes.push("is-file", fileTypeClass(path || name));
  else classes.push("is-folder");
  if (openFile) classes.push("is-active");
  if (gitState) classes.push(`git-${gitState}`);
  if (ignoredState === "ignored") classes.push("is-ignored");
  return classes.filter(Boolean).join(" ");
}

function fileTypeClass(path) {
  const lower = String(path).toLowerCase();
  if (lower.endsWith(".js") || lower.endsWith(".mjs") || lower.endsWith(".cjs")) return "file-type-js";
  if (lower.endsWith(".ts") || lower.endsWith(".tsx")) return "file-type-ts";
  if (lower.endsWith(".json")) return "file-type-json";
  if (lower.endsWith(".css")) return "file-type-css";
  if (lower.endsWith(".md") || lower.endsWith(".mdx")) return "file-type-md";
  if (lower.endsWith(".sh") || lower.endsWith(".bash") || lower.endsWith(".zsh")) return "file-type-shell";
  if (lower.endsWith(".py")) return "file-type-python";
  if (lower.endsWith(".html")) return "file-type-html";
  return "file-type-default";
}

function languageClass(path) {
  return fileTypeClass(path).replace("file-type-", "language-");
}

function syntaxHighlightNodes(content, path) {
  const language = languageClass(path);
  if (language === "language-js" || language === "language-ts") return tokenizeCode(content);
  if (language === "language-json") return tokenizeJson(content);
  if (language === "language-css") return tokenizeCss(content);
  if (language === "language-md") return tokenizeMarkdown(content);
  if (language === "language-shell") return tokenizeShell(content);
  return [content || ""];
}

function tokenizeCode(content) {
  return tokenize(content, [
    [/\/\/.*/y, "comment"],
    [/\/\*[\s\S]*?\*\//y, "comment"],
    [/'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"|`(?:\\.|[^`\\])*`/y, "string"],
    [/\b(?:import|from|export|const|let|var|function|return|if|else|for|while|class|new|async|await|try|catch|throw|type|interface)\b/y, "keyword"],
    [/\b(?:true|false|null|undefined)\b/y, "constant"],
    [/\b\d+(?:\.\d+)?\b/y, "number"]
  ]);
}

function tokenizeJson(content) {
  return tokenize(content, [
    [/"(?:\\.|[^"\\])*"(?=\s*:)/y, "property"],
    [/"(?:\\.|[^"\\])*"/y, "string"],
    [/\b(?:true|false|null)\b/y, "constant"],
    [/-?\b\d+(?:\.\d+)?\b/y, "number"]
  ]);
}

function tokenizeCss(content) {
  return tokenize(content, [
    [/\/\*[\s\S]*?\*\//y, "comment"],
    [/[.#]?[a-zA-Z_-][\w-]*(?=\s*\{)/y, "selector"],
    [/[a-zA-Z-]+(?=\s*:)/y, "property"],
    [/'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"/y, "string"],
    [/#(?:[0-9a-fA-F]{3,8})\b/y, "number"]
  ]);
}

function tokenizeMarkdown(content) {
  return tokenize(content, [
    [/^#{1,6}.*/ym, "keyword"],
    [/`[^`]*`/y, "string"],
    [/\*\*[^*]+\*\*/y, "strong"],
    [/\[[^\]]+\]\([^)]+\)/y, "string"]
  ]);
}

function tokenizeShell(content) {
  return tokenize(content, [
    [/#.*/y, "comment"],
    [/'(?:\\.|[^'\\])*'|"(?:\\.|[^"\\])*"/y, "string"],
    [/\b(?:if|then|else|fi|for|do|done|case|esac|function|export|set)\b/y, "keyword"],
    [/\$[A-Za-z_][\w]*/y, "constant"]
  ]);
}

function tokenize(content, rules) {
  const nodes = [];
  let index = 0;
  while (index < content.length) {
    let matched = false;
    for (const [pattern, className] of rules) {
      pattern.lastIndex = index;
      const match = pattern.exec(content);
      if (match && match.index === index) {
        nodes.push(h("span", { class: `syntax-token ${className}`, text: match[0] }));
        index += match[0].length;
        matched = true;
        break;
      }
    }
    if (!matched) {
      nodes.push(content[index]);
      index += 1;
    }
  }
  return nodes.length ? nodes : [""];
}

function terminalTool(route) {
  const agentTerminal = agentTerminalViewForRoute(route);
  return h("section", { class: "tool-body terminal-tool terminal-emulator-tool" }, [
    h("div", {
      class: "terminal-emulator",
      "data-terminal-emulator": "user",
      tabindex: "0"
    }),
    h("div", {
      class: "vertical-resize-handle terminal-agent-resize",
      role: "separator",
      tabindex: "0",
      "aria-label": "Resize agent command terminal",
      "aria-orientation": "horizontal",
      "data-resize-stack": "terminal-agent"
    }),
    h("div", { class: "terminal-agent-pane", "data-agent-terminal": "true" }, [
      h("div", { class: "terminal-agent-title", text: agentTerminal.title || "Agent command terminal" }),
      h("pre", {
        class: "terminal-agent-transcript",
        text: agentTerminal.output || agentTerminal.summary || ""
      })
    ])
  ]);
}

//--------------------------------------------------------------------//
// Settings routes
//--------------------------------------------------------------------//

/**
 * Render the settings layout and active settings body.
 */
function renderSettings(route) {
  const active = route.replace("settings-", "");
  if (providerSetupGate && active === "add-provider") {
    return h("div", { class: "settings-shell setup-only", "data-screen": route }, [
      h("main", { id: "main", class: "settings-main", tabindex: "-1" }, [settingsBody(active)])
    ]);
  }
  return h("div", { class: "settings-shell", "data-screen": route }, [
    h("aside", { class: "settings-nav", "aria-label": "Settings navigation" }, [
      h("a", { class: "settings-back", href: "#new-session", "data-settings-back-action": "" }, [icon("arrowLeft"), h("span", { text: "Back to app" })]),
      h("p", { class: "kicker", text: "Settings" }),
      ...settingsItems.flatMap(([label, target, iconName, divider]) => [
        divider ? h("div", { class: "settings-divider", role: "separator" }) : null,
        link(target, `settings-link${route === target ? " is-active" : ""}`, [icon(iconName), h("span", { text: label })], { "aria-current": route === target ? "page" : null })
      ])
    ]),
    h("main", { id: "main", class: "settings-main", tabindex: "-1" }, [settingsBody(active)])
  ]);
}

/**
 * Render a settings page header with optional trailing action.
 */
function settingsTitle(title, summary, action) {
  return h("header", { class: "settings-title" }, [h("div", {}, [h("h1", { text: title }), h("p", { text: summary })]), action]);
}

/**
 * Route the settings body to its concrete r04 settings page.
 */
function settingsBody(active) {
  if (active === "add-provider") return providerForm();
  if (active === "models") return modelSettings();
  if (active === "runtimes") return runtimes();
  if (active === "configuration") return configuration();
  if (active === "plugins") return plugins();
  if (active === "skills") return skills();
  if (active === "mcp") return mcp();
  return h("section", {}, [
    settingsTitle("Providers", "Manage OpenAI-compatible provider connections. Labels must be unique.", link("settings-add-provider", "button primary", [icon("add"), h("span", { text: "Add Provider" })], { "data-provider-new": "" })),
    h("div", { class: "data-list" }, providers.map((provider) => dataRow(
      provider.label,
      `${provider.endpoint || provider.baseUrl} - ${providerStatus(provider)}`,
      button("Edit", "button secondary", null, {
        "aria-label": `Edit ${provider.label}`,
        "data-provider-edit": provider.id
      }),
      {
        kind: "provider",
        label: `${provider.label} ${provider.enabled === false ? "disabled" : "enabled"}`,
        providerId: provider.id,
        enabled: provider.enabled !== false
      }
    )))
  ]);
}

function providerStatus(provider) {
  if (provider.keyStatus?.state === "present") return "Key configured";
  if (provider.keyStatus?.state === "missing") return "Key not configured";
  return provider.status || "Key status managed by backend";
}

function modelSettings() {
  const providerOptions = uniqueModelProviders();
  return h("section", {}, [
    settingsTitle("Models", "Models are fetched from enabled provider connections when available; manual model entries can be enabled here."),
    h("div", { class: "models-toolbar" }, [
      h("label", { class: "model-search" }, [
        icon("search"),
        h("span", { class: "sr-only", text: "Search models" }),
        h("input", {
          type: "search",
          "aria-label": "Search models",
          placeholder: "Search models",
          value: modelSettingsFilters.search,
          "data-model-search": ""
        })
      ]),
      h("select", { class: "model-provider-filter", "aria-label": "Filter models by provider", "data-model-provider-filter": "" }, [
        h("option", { value: "all", selected: modelSettingsFilters.provider === "all", text: "All providers" }),
        ...providerOptions.map((provider) => h("option", {
          value: provider,
          selected: modelSettingsFilters.provider === provider,
          text: provider
        }))
      ]),
      modelBulkToggle()
    ]),
    modelSettingsList()
  ]);
}

function modelBulkToggle() {
  const filtered = filteredModels();
  const hasModels = filtered.length > 0;
  const allEnabled = hasModels && filtered.every((model) => model.enabled !== false);
  const enable = !allEnabled;
  return h("button", {
    class: "button secondary model-bulk-toggle",
    type: "button",
    "aria-label": enable ? "Enable results" : "Disable results",
    "data-model-bulk-toggle": "",
    "data-model-bulk-enabled": String(enable),
    disabled: hasModels ? null : "true"
  }, [h("span", { text: enable ? "Enable results" : "Disable results" })]);
}

function modelSettingsList() {
  return h("div", { class: "data-list" }, filteredModels().map((model) => dataRow(
      model.label,
      `${model.provider} - ${model.source || "manual"}`,
      h("span", { class: "status-pill", text: model.enabled ? "Enabled" : "Disabled" }),
      {
        kind: "model",
        label: `${model.label} ${model.enabled ? "enabled" : "disabled"}`,
        modelId: model.id,
        enabled: model.enabled
      }
    )));
}

function filteredModels() {
  const search = modelSettingsFilters.search.trim().toLowerCase();
  return models.filter((model) => {
    const matchesProvider = modelSettingsFilters.provider === "all" || model.provider === modelSettingsFilters.provider;
    const haystack = `${model.label} ${model.provider} ${model.source || ""}`.toLowerCase();
    return matchesProvider && (!search || haystack.includes(search));
  });
}

function uniqueModelProviders() {
  return Array.from(new Set(models.map((model) => model.provider).filter(Boolean))).sort((left, right) => left.localeCompare(right));
}

/**
 * Render a standard settings data list.
 */
function dataList(title, summary, rows) {
  return h("section", {}, [
    settingsTitle(title, summary),
    h("div", { class: "data-list" }, rows.map(([name, meta, status]) => dataRow(name, meta, h("span", { class: "status-pill", text: status }), true)))
  ]);
}

/**
 * Render one row in a settings list.
 */
function dataRow(name, meta, middle, toggle) {
  const toggleKind = typeof toggle === "object" ? toggle.kind : null;
  return h("article", { class: "data-row" }, [
    h("div", {}, [h("strong", { text: name }), h("span", { text: meta })]),
    middle,
    toggle ? h("button", {
      class: "switch",
      type: "button",
      "aria-label": typeof toggle === "object" ? toggle.label : `${name} enabled`,
      "aria-pressed": typeof toggle === "object" ? String(toggle.enabled) : "true",
      "data-model-enable": toggleKind === "model" ? toggle.modelId : null,
      "data-model-enabled": toggleKind === "model" ? String(toggle.enabled) : null,
      "data-provider-enable": toggleKind === "provider" ? toggle.providerId : null,
      "data-provider-enabled": toggleKind === "provider" ? String(toggle.enabled) : null
    }, [h("span")]) : null
  ]);
}

/**
 * Render the Add Provider form with the documented provider type list.
 */
function providerForm() {
  const draft = providerFormDraft;
  const fields = [
    { id: "provider-type", label: "Provider Type", options: ["OpenAI", "OpenRouter", "Hugging Face router", "LiteLLM proxy", "Custom OpenAI-compatible endpoint"], type: "select", value: draft?.kind || "Custom OpenAI-compatible endpoint" },
    { id: "provider-label", label: "Label", type: "text", value: draft?.label || "OpenRouter - Personal" },
    { id: "api-base-url", label: "API Base URL", type: "url", value: draft?.baseUrl || draft?.endpoint || "https://openrouter.ai/api/v1" },
    { id: "api-key", label: "API Key", type: "password", value: "", placeholder: "Stored by backend credential status" },
    { id: "auth-type", label: "Auth", options: ["Bearer token", "Custom header"], type: "select", value: "Bearer token" },
    { id: "provider-headers", label: "Headers", type: "textarea", value: "{}" }
  ];
  return h("section", {}, [
    settingsTitle(draft ? "Edit Provider" : "Add Provider", "Save an OpenAI-compatible connection profile."),
    h("form", { class: "provider-form" }, [
      ...fields.map(settingsField),
      h("div", { class: "form-actions" }, [
        draft ? button("Remove Provider", "button danger", null, { "data-provider-delete": "" }) : null,
        button("Save Provider", "button primary", null, { type: "submit" }),
        link("settings-providers", "button secondary", ["Cancel"])
      ])
    ])
  ]);
}

/**
 * Render a settings form control from a simple field descriptor.
 */
function settingsField(field) {
  const control = field.type === "select"
    ? h("select", { id: field.id, name: field.id }, field.options.map((option) => h("option", { selected: option === field.value, text: option })))
    : field.type === "textarea"
      ? h("textarea", { id: field.id, name: field.id, rows: "5", spellcheck: "false" }, [field.value])
      : h("input", { id: field.id, name: field.id, type: field.type, value: field.value, placeholder: field.placeholder || null, spellcheck: field.type === "password" ? "false" : null });
  return h("label", { class: "field", "data-provider-field": field.id, for: field.id }, [h("span", { text: field.label }), control]);
}

/**
 * Render runtime selection fixtures.
 */
function runtimes() {
  return h("section", {}, [
    settingsTitle("Runtimes", "Choose the runtime used to execute agent work in this workspace."),
    h("div", { class: "data-list" }, ["OpenCode", "Pi"].map((name, index) => dataRow(name, index === 0 ? "https://opencode.ai/" : "https://pi.dev/", h("span", { class: "status-pill", text: index === 0 ? "Selected" : "Choose runtime" }), true)))
  ]);
}

/**
 * Render configuration fixtures.
 */
function configuration() {
  return h("section", {}, [
    settingsTitle("Configuration", "Configure approval policy and sandbox settings.", button("Open config.toml externally", "button secondary", "file")),
    h("div", { class: "config-grid" }, [
      dataRow("Approval Policy", "Choose when the app asks before high-impact actions.", button("On request")),
      dataRow("Sandbox Settings", "Choose how much agents can do when running commands.", button("Workspace write"))
    ])
  ]);
}

/**
 * Render the plugin directory, marketplace picker, and plugin dialogs.
 */
function plugins() {
  return h("section", {}, [
    settingsTitle("Plugins", "Manage installed plugins and extension surfaces."),
    h("div", { class: "plugin-store", "aria-label": "Plugin catalog" }, [
      h("div", { class: "plugin-toolbar" }, [
        h("label", { class: "plugin-search" }, [icon("search"), h("input", { type: "search", placeholder: "Search plugins" })]),
        h("div", { class: "plugin-filter-wrap" }, [
          h("button", { class: "plugin-filter", type: "button", "aria-expanded": "false", "aria-haspopup": "menu", "data-marketplace-menu-trigger": true }, [h("span", { text: "Built by C4OS" }), icon("chevronDown")]),
          h("div", { class: "plugin-filter-menu", role: "menu", "aria-label": "Plugin marketplaces", hidden: true, "data-marketplace-menu": true }, [
            ...pluginMarketplaces.map((marketplace) => h("button", {
              class: `marketplace-option${marketplace.active ? " is-selected" : ""}`,
              type: "button",
              role: "menuitemradio",
              "aria-checked": marketplace.active ? "true" : "false"
            }, [
              h("span", { class: "marketplace-option-copy" }, [
                h("strong", { text: marketplace.label }),
                h("small", { text: marketplace.summary })
              ]),
              marketplace.active ? h("span", { class: "marketplace-check", "aria-label": "Selected marketplace" }, [icon("check")]) : null
            ])),
            h("div", { class: "marketplace-menu-divider", role: "separator" }),
            h("button", { class: "marketplace-add-option", type: "button", role: "menuitem", "data-marketplace-open": true }, [
              icon("add"),
              h("span", { text: "Add Marketplace" })
            ])
          ])
        ])
      ]),
      pluginCatalog.length === 0 ? extensionEmptyState(
        "No plugins installed",
        "Add a marketplace when plugin installation is available for this workspace."
      ) : h("div", { class: "plugin-grid" }, pluginCatalog.map((record) => {
        const label = extensionLabel(record);
        return h("article", { class: "plugin-card", "data-extension-record": extensionId(record) }, [
          h("div", { class: "plugin-logo" }, [h("span", { text: pluginInitials(label) })]),
          h("div", { class: "plugin-copy" }, [
            h("strong", { text: label }),
            extensionMeta(record)
          ]),
          iconButton(`Add ${label}`, "add", "icon-button plugin-add-button", { "data-plugin-connect": label })
        ]);
      }))
    ]),
    pluginDialog(),
    marketplaceDialog()
  ]);
}

/**
 * Render the skill directory and skill detail dialog.
 */
function skills() {
  return h("section", {}, [
    settingsTitle("Skills", "Manage installed skills and per-project availability."),
    h("div", { class: "skills-panel" }, [
      h("label", { class: "skills-search" }, [icon("search"), h("input", { type: "search", placeholder: "Search skills" })]),
      skillCatalog.length === 0 ? extensionEmptyState(
        "No skills installed",
        "Skills will appear here after a project or workspace skill folder is connected."
      ) : h("div", { class: "skills-list" }, skillCatalog.map((record) => {
        const label = extensionLabel(record);
        return h("article", { class: "skill-row", "data-extension-record": extensionId(record) }, [
          h("button", { class: "skill-open", type: "button", "data-skill-open": label }, [h("span", { class: "skill-mark" }, [icon("file")]), h("span", { class: "skill-copy" }, [h("strong", { text: label }), extensionMeta(record)])]),
          h("span", { class: "skill-scope", text: `${extensionScope(record)} scope` }),
          h("button", { class: "skill-switch", type: "button", "aria-label": extensionEnabled(record) ? "Skill enabled" : "Skill disabled", "aria-pressed": String(extensionEnabled(record)) }, [h("span")])
        ]);
      }))
    ]),
    skillDialog()
  ]);
}

/**
 * Render the MCP Servers settings page and add-server dialog.
 */
function mcp() {
  return h("section", {}, [
    h("header", { class: "mcp-head" }, [
      h("div", {}, [h("h1", { text: "MCP Servers" }), h("p", { text: "Connect external tools and data sources." })]),
      h("div", { class: "header-actions" }, [
        button("Learn more", "button secondary", null, { "aria-label": "Learn more about MCP Servers" }),
        button("Add server", "button primary", "add", { "data-mcp-add": true })
      ])
    ]),
    h("section", { class: "mcp-section" }, [
      h("h2", { text: "Servers" }),
      mcpServers.length === 0 ? extensionEmptyState(
        "No MCP servers connected",
        "Connect a server when MCP connection persistence is available."
      ) : h("div", { class: "mcp-list" }, mcpServers.map((record) => dataRow(
      extensionLabel(record),
      extensionSummary(record),
      iconButton(`${extensionLabel(record)} settings`, "settings"),
      {
        kind: "extension",
        label: `${extensionLabel(record)} ${extensionEnabled(record) ? "enabled" : "disabled"}`,
        enabled: extensionEnabled(record)
      }
    )))
    ]),
    mcpDialog()
  ]);
}

function extensionEmptyState(title, copy) {
  return h("div", { class: "settings-empty", role: "status" }, [
    h("strong", { text: title }),
    h("span", { text: copy })
  ]);
}

function extensionId(record) {
  return record?.id || extensionLabel(record).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function extensionLabel(record) {
  return typeof record === "string" ? record : record?.label || "Extension";
}

function extensionEnabled(record) {
  return record?.enabled === true;
}

function extensionScope(record) {
  const scopes = Array.isArray(record?.scopes) ? record.scopes : [];
  if (scopes.includes("project")) return "Project";
  if (scopes.includes("workspace")) return "Workspace";
  return record?.projectScope ? "Project" : "Workspace";
}

function extensionSummary(record) {
  if (typeof record === "string") return "Imported extension record";
  const parts = [
    record.provenance,
    `${extensionScope(record)} scope`,
    `Shared data: ${extensionList(record.sharedData)}`,
    `Tool access: ${extensionList(record.toolAccess)}`,
    `Runtime ${record.runtimeAccess || "disabled"}`,
    `Audit: ${extensionList(record.audit)}`
  ];
  return parts.filter(Boolean).join(" - ");
}

function extensionMeta(record) {
  return h("span", { text: extensionSummary(record) });
}

function extensionList(values) {
  return Array.isArray(values) && values.length > 0 ? values.join(", ") : "none";
}

//--------------------------------------------------------------------//
// Settings dialogs
//--------------------------------------------------------------------//

/**
 * Render the plugin connection review dialog.
 */
function pluginDialog() {
  return h("div", { class: "plugin-modal", hidden: true, "data-plugin-modal": "connect" }, [
    h("div", { class: "plugin-modal-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "plugin-connect-title" }, [
      iconButton("Close plugin connection", "x", "plugin-modal-close plugin-connect-close", { "data-plugin-close": true }),
      h("div", { class: "plugin-connection" }, [
        h("span", { class: "plugin-node plugin-node-source", text: "AI" }),
        h("span", { class: "plugin-dots", "aria-hidden": "true", text: "..." }),
        h("span", { class: "plugin-node", "data-plugin-modal-target": true, text: "GH" })
      ]),
      h("h2", { id: "plugin-connect-title", "data-plugin-modal-title": true, text: "Connect GitHub" }),
      h("div", { class: "plugin-modal-status" }, [
        h("span", { class: "approval-check" }, [icon("check")]),
        h("span", { text: "Approved by your admin" })
      ]),
      h("div", { class: "plugin-risk-grid" }, [
        pluginReviewBlock("You're in control", "C4OS respects your project preferences and limits this plugin to permissions you explicitly set."),
        pluginReviewBlock("Plugins may introduce elevated risk", "Plugins can access scoped project context. Review permissions before connecting a workflow to your workspace."),
        pluginReviewBlock("Data shared with this plugin", "Adding this plugin allows access to basic workspace context and recent intent needed to respond to your request.")
      ]),
      h("div", { class: "plugin-modal-actions" }, [
        h("button", { class: "plugin-modal-continue", type: "button", "data-plugin-close": true }, [
          h("span", { "data-plugin-continue-label": true, text: "Continue to GitHub" }),
          icon("external")
        ]),
        button("Advanced settings", "plugin-advanced")
      ])
    ])
  ]);
}

/**
 * Render one block of plugin connection review copy.
 */
function pluginReviewBlock(title, copy) {
  return h("section", { class: "plugin-risk-block" }, [
    h("h3", { text: title }),
    h("p", { text: copy })
  ]);
}

/**
 * Convert a plugin name into the short mark shown in the connection graphic.
 */
function pluginInitials(name) {
  if (name === "GitHub") return "GH";
  return name
    .split(/\s+/)
    .map((part) => part[0])
    .join("")
    .slice(0, 2);
}

/**
 * Render the Add plugin marketplace dialog.
 */
function marketplaceDialog() {
  return h("div", { class: "plugin-modal", hidden: true, "data-marketplace-modal": true }, [
    h("div", { class: "marketplace-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "marketplace-title" }, [
      iconButton("Close marketplace dialog", "x", "plugin-modal-close", { "data-marketplace-close": true }),
      h("header", { class: "marketplace-head" }, [
        h("h2", { id: "marketplace-title", text: "Add plugin marketplace" }),
        h("p", { text: "Add from a GitHub repo, Git URL, or local folder." })
      ]),
      h("div", { class: "marketplace-form" }, [
        marketplaceField("Source", "marketplace-source", "openai/plugins or git@github.com:org/repo.git"),
        marketplaceField("Git ref", "marketplace-ref", "main"),
        marketplaceField("Sparse paths", "marketplace-paths", "plugins/codex", true)
      ]),
      h("div", { class: "marketplace-actions" }, [button("Cancel", "button secondary", null, { "data-marketplace-close": true }), button("Add marketplace", "button primary", null, { "data-marketplace-close": true })])
    ])
  ]);
}

/**
 * Render a marketplace form field.
 */
function marketplaceField(label, id, placeholder, multiline = false) {
  const control = multiline
    ? h("textarea", { id, name: id, placeholder, rows: "3", spellcheck: "false" })
    : h("input", { id, name: id, placeholder, type: "text", spellcheck: "false" });
  return h("div", { class: "marketplace-field" }, [h("label", { for: id, text: label }), control]);
}

/**
 * Render the skill details dialog.
 */
function skillDialog() {
  return h("div", { class: "plugin-modal", hidden: true, "data-skill-modal": true }, [
    h("div", { class: "skill-modal-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "skill-modal-title" }, [
      iconButton("Close skill details", "x", "plugin-modal-close", { "data-skill-close": true }),
      h("header", { class: "skill-modal-head" }, [
        h("span", { class: "skill-mark" }, [icon("file")]),
        h("div", {}, [
          h("h2", { id: "skill-modal-title", "data-skill-title": true, text: "ChrisAI Agents" }),
          h("p", { class: "type-label", text: "Skill" })
        ]),
        h("button", { class: "skill-switch", type: "button", "aria-label": "Skill enabled" }, [h("span")]),
        iconButton("More skill actions", "more")
      ]),
      h("p", { class: "skill-summary", text: "Project-local workflow skill detail fixture." }),
      h("div", { class: "skill-detail-grid" }, [
        h("section", {}, [h("h3", { text: "Scope" }), h("p", { text: "Project availability" })]),
        h("section", {}, [h("h3", { text: "Details" }), h("p", { text: "This frontend-local detail body previews how a selected skill record will explain what it can do before TASK-002 connects real data." })])
      ]),
      h("footer", { class: "skill-modal-actions" }, [button("Uninstall", "button secondary", null, { "data-skill-close": true }), button("Try in chat", "button primary", null, { "data-skill-close": true })])
    ])
  ]);
}

/**
 * Render the custom MCP connection dialog.
 */
function mcpDialog() {
  return h("div", { class: "plugin-modal", hidden: true, "data-mcp-modal": true }, [
    h("div", { class: "mcp-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "mcp-dialog-title" }, [
      h("header", { class: "mcp-dialog-head" }, [
        h("h2", { id: "mcp-dialog-title", text: "Connect to a custom MCP" }),
        iconButton("Close MCP server form", "x", "plugin-modal-close", { "data-mcp-close": true })
      ]),
      h("div", { class: "mcp-segment", role: "tablist", "aria-label": "MCP transport" }, [
        h("button", { class: "is-active", type: "button", role: "tab", "aria-selected": "true", "data-mcp-mode": "stdio" }, ["STDIO"]),
        h("button", { type: "button", role: "tab", "aria-selected": "false", "data-mcp-mode": "http" }, ["Streamable HTTP"])
      ]),
      h("div", { class: "mcp-mode-fields", "data-mcp-fields": "stdio" }, [
        mcpFormField("Command to launch", "mcp-command", "openai-dev-mcp serve-sqlite"),
        mcpInputGroup("Arguments", [""], "Add argument"),
        mcpInputGroup("Environment variables", ["Key", "Value"], "Add environment variable", true),
        mcpInputGroup("Environment variable passthrough", [""], "Add variable"),
        mcpFormField("Working directory", "mcp-working-directory", "~/code")
      ]),
      h("div", { class: "mcp-mode-fields", hidden: true, "data-mcp-fields": "http" }, [
        mcpFormField("URL", "mcp-url", "https://mcp.example.com/mcp"),
        mcpFormField("Bearer token env var", "mcp-bearer", "MCP_BEARER_TOKEN"),
        mcpInputGroup("Headers", ["Key", "Value"], "Add header", true),
        mcpInputGroup("Headers from environment variables", ["Key", "Value"], "Add variable", true)
      ]),
      h("footer", { class: "mcp-actions" }, [button("Cancel", "button secondary", null, { "data-mcp-close": true }), button("Save", "button primary", null, { "data-mcp-close": true })])
    ])
  ]);
}

/**
 * Render a single MCP text field.
 */
function mcpFormField(label, id, placeholder) {
  return h("label", { class: "mcp-field", for: id }, [
    h("span", { text: label }),
    h("input", { id, name: id, placeholder, spellcheck: "false", type: "text" })
  ]);
}

/**
 * Render an add/remove-capable MCP field group.
 */
function mcpInputGroup(title, placeholders, addLabel, twoColumn = false) {
  return h("section", { class: "mcp-group", "data-mcp-group": title, "data-mcp-columns": twoColumn ? "pair" : "single" }, [
    h("h3", { text: title }),
    h("div", { class: twoColumn ? "mcp-pair-row" : "mcp-single-row", "data-mcp-row": "1" }, [
      ...placeholders.map((placeholder, index) => h("input", {
        "aria-label": `${title} ${index + 1}`,
        placeholder,
        spellcheck: "false",
        type: "text"
      })),
      iconButton(`Remove ${title} row 1`, "trash", "mcp-trash", { "data-mcp-remove-row": true })
    ]),
    h("button", { class: "mcp-add-line", type: "button" }, [`+ ${addLabel}`])
  ]);
}

window.addEventListener("hashchange", render);
window.addEventListener("c4os:terminal-state-changed", updateAgentTerminalDom);
const boot = beginConnectorStateLoad();
render();
if (boot) {
  boot.then(() => {
    if (!routeBootstrappedWorkspace()) render();
  });
}
