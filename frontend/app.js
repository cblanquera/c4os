import {
  beginConnectorStateLoad,
  browserState,
  connectorState,
  filesState,
  mcpServers,
  models,
  openConnectorWorkspace,
  pluginCatalog,
  pluginMarketplaces,
  projects,
  providers,
  routes,
  sendConnectorPrompt,
  settingsItems,
  terminalState,
  threadTurns,
  skillCatalog,
  workspace
} from "./data.js";
import { h, icon } from "./dom.js";
import { bindInteractions, bindMessage, bindTerminal } from "./interactions.js";
import { renderMarkdown } from "./markdown.js";
import { appStore, sessionSurfaceKey } from "./state.js";

const app = document.querySelector("#app");

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
  appStore.setRoute(route);
  document.body.dataset.route = route;
  if (route === "app-start") app.replaceChildren(renderStart());
  else if (route.startsWith("settings")) app.replaceChildren(renderSettings(route));
  else app.replaceChildren(renderShell(route));
  bindInteractions(render, pluginInitials);
  bindConnectorRun();
  bindWorkspaceOpen();
  bindWorkLogDisclosure();
  bindToolTabs();
  bindPanelLinks();
  bindComposerPickers();
  bindSessionRows();
  applyShellState();
}

function bindConnectorRun() {
  if (!connectorState.connector.available) return;
  document.querySelectorAll(".send-button").forEach((control) => {
    control.addEventListener("click", async () => {
      const composerNode = control.closest(".composer");
      const prompt = composerNode?.querySelector(".prompt-box")?.textContent?.trim() || "";
      const shouldCreateSession = routeFromHash() !== "chat-session";
      appStore.composer.bySurface[composerSurfaceKey()] = { prompt: "" };
      const promptBox = composerNode?.querySelector(".prompt-box");
      if (promptBox) promptBox.textContent = "";
      const pending = sendConnectorPrompt(prompt, {
        createSession: shouldCreateSession,
        onTurnCreated: (turn) => {
          if (!shouldCreateSession) appendTurnToThread(turn);
        },
        onStateChange: (turn) => updateTurnDom(turn),
        beforeComplete: minimumPendingFrame
      });
      window.location.hash = "chat-session";
      if (shouldCreateSession) render();
      await minimumPendingFrame();
      await pending;
      updateTurnDom(threadTurns[threadTurns.length - 1]);
    });
  });
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
      bindTerminal();
      bindPanelLinks();
    });
  });
}

function bindSessionRows() {
  document.querySelectorAll("[data-session-target]").forEach((control) => {
    if (control.dataset.boundSessionTarget) return;
    control.dataset.boundSessionTarget = "true";
    control.addEventListener("click", (event) => {
      event.preventDefault();
      appStore.setActiveSession(control.dataset.projectTarget, control.dataset.sessionTarget);
      window.location.hash = "chat-session";
      updateShellSessionDom();
    });
  });
  document.querySelectorAll("[data-project-target]:not([data-session-target])").forEach((control) => {
    if (control.dataset.boundProjectTarget) return;
    control.dataset.boundProjectTarget = "true";
    control.addEventListener("click", (event) => {
      event.preventDefault();
      workspace.project = control.dataset.projectTarget;
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
      workspace.model = control.dataset.localModel;
      appStore.setComposerValue(surface, "openPicker", null);
      composerNode.querySelector("[data-local-picker-trigger='models'] span").textContent = workspace.model;
      composerNode.querySelector("[data-local-picker-trigger='models']").setAttribute("aria-label", `Model: ${workspace.model}`);
      updateComposerPicker(composerNode, surface);
    });
  });
}

function bindPanelLinks() {
  document.querySelectorAll(".tool-panel [data-file-target]").forEach((anchor) => {
    if (anchor.dataset.boundFileTarget) return;
    anchor.dataset.boundFileTarget = "true";
    anchor.addEventListener("click", (event) => {
      event.preventDefault();
      const route = routeFromHash();
      const surface = toolSurfaceKey(route);
      appStore.setFileView(surface, anchor.dataset.fileTarget);
      const panel = document.querySelector(".tool-panel");
      panel?.replaceWith(renderToolPanel("files", route));
      bindToolTabs();
      bindTerminal();
      bindPanelLinks();
    });
  });
}

function updateShellSessionDom() {
  const shell = document.querySelector(".app-shell");
  if (!shell) return render();
  shell.dataset.screen = "chat-session";
  document.body.dataset.route = "chat-session";
  shell.querySelector(".topbar strong").textContent = workspace.session;
  shell.querySelectorAll(".project-row").forEach((row) => {
    row.classList.toggle("is-active", row.dataset.projectTarget === workspace.project);
  });
  shell.querySelectorAll(".session-row").forEach((row) => {
    row.classList.toggle("is-active", row.dataset.sessionTarget === workspace.session);
  });
  const workbench = shell.querySelector(".workbench");
  const currentMain = workbench.querySelector(":scope > main");
  if (!currentMain?.classList.contains("thread-view")) {
    currentMain?.replaceWith(renderThread());
    bindWorkLogDisclosure();
    bindConnectorRun();
  }
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
    shell.querySelector(".terminal-tool")?.style.setProperty("--terminal-bottom", `${appStore.shell.terminalBottom}px`);
  }
}

function bindWorkspaceOpen() {
  if (!connectorState.connector.available) return;
  document.querySelectorAll("[data-open-workspace]").forEach((control) => {
    control.addEventListener("click", async () => {
      try {
        await openConnectorWorkspace();
        window.location.hash = "new-session";
        render();
      } catch {
        render();
      }
    });
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
    : ["Project Alpha", "Agent Lab", "Docs Workbench"].map((name) => ({ name }));
  const title = connectorState.loading
    ? "Loading workspace state"
    : connectorState.error
      ? "Workspace state unavailable"
      : "Open a folder to start working";
  const lead = connectorState.error
    ? connectorState.error
    : "Local files, instructions, skills, runtime policy, approvals, and workspace state stay unavailable until a trusted project folder scopes the session.";
  return h("main", { id: "main", class: "start-view", "data-screen": "app-start", tabindex: "-1" }, [
    h("section", { class: "start-copy" }, [
      h("p", { class: "kicker", text: "No trusted project folder" }),
      h("h1", { text: title }),
      h("p", { class: "lead", text: lead }),
      h("div", { class: "action-row" }, [
        button("Open Folder", "button primary", null, { "data-open-workspace": "" }),
        button("Clone Repository"),
        button("Open Workspace File")
      ])
    ]),
    h("section", { class: "recent-panel", "aria-labelledby": "recent-title" }, [
      h("h2", { id: "recent-title", text: "Recent Folder-Backed Workspaces" }),
      ...recentProjects.map((project, index) =>
        link("new-session", "workspace-row", [
          h("strong", { text: project.name }),
          h("span", { text: ["Trusted", "2 roots", "Saved"][index] })
        ])
      )
    ])
  ]);
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
  if (route === "chat-session") return sessionSurfaceKey(workspace.project, workspace.session || "untitled");
  if (route === "new-session" || route === "providers-popover" || route === "models-popover") return `new:${workspace.project}`;
  return `route:${route}`;
}

function defaultToolForRoute(route) {
  if (route === "file-explorer" || route === "file-editor") return "files";
  if (route === "terminal") return "terminal";
  return "browser";
}

function composerSurfaceKey() {
  return routeFromHash() === "chat-session" ? appStore.activeSessionKey() : `new:${workspace.project}`;
}

/**
 * Render project/session navigation for the left shell panel.
 */
function renderSidebar(chat) {
  return h("aside", { class: "sidebar", "aria-label": "Projects" }, [
    h("label", { class: "search-field" }, [icon("search"), h("span", { class: "sr-only", text: "Search projects" }), h("input", { type: "search", placeholder: "Search projects" })]),
    h("div", { class: "nav-section" }, [
      h("div", { class: "section-head" }, [h("span", { text: "Projects" }), iconButton("Add Project", "add")]),
      ...projects.flatMap((project) => [
        h("a", {
          class: `project-row${project.name === workspace.project && !chat ? " is-active" : ""}`,
          href: "#new-session",
          "data-project-target": project.name
        }, [icon("folder"), h("span", { text: project.name }), h("span", { class: "row-tools" }, [icon("pencil"), icon("trash")])]),
        ...project.sessions.map((session) => h("a", {
          class: `session-row${chat && session === workspace.session ? " is-active" : ""}`,
          href: "#chat-session",
          "data-project-target": project.name,
          "data-session-target": session
        }, [h("span", { text: session })]))
      ])
    ]),
    link("settings-providers", "settings-entry", [icon("settings"), h("span", { text: "Settings" })])
  ]);
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
      readonly ? h("span", { class: "chip readonly-chip", "aria-label": "Model locked for this chat" }, [icon("bot"), h("span", { text: workspace.model })]) : h("button", { class: "chip", type: "button", "aria-label": `Model: ${workspace.model}`, "data-local-picker-trigger": "models" }, [icon("bot"), h("span", { text: workspace.model })])
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
      h("header", { class: "popover-title" }, [h("strong", { text: "Providers" }), h("span", { text: "Select source" })]),
      ...["OpenRouter", "OpenAI", "LiteLLM Local"].map((name) =>
        h("button", { class: "popover-row", type: "button", "data-local-provider": name }, [h("span", { text: name }), icon("chevronRight")])
      )
    ]);
  }
  return h("aside", { class: "popover", "aria-label": "Model selector", "data-local-picker": "models" }, [
    h("header", { class: "popover-backbar" }, [
      h("button", { class: "popover-back", type: "button", "aria-label": "Back to providers", "data-local-picker-trigger": "providers" }, [icon("chevronLeft"), h("strong", { text: "OpenRouter" })])
    ]),
    ...models.slice(0, 3).map((model) => h("button", { class: `popover-row${model.active ? " is-selected" : ""}`, type: "button", "data-local-model": model.label }, [
      h("span", { text: model.label }),
      model.active ? h("span", { "aria-label": "Current model" }, [icon("check")]) : h("span")
    ]))
  ]);
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
    h("header", { class: "popover-title" }, [h("strong", { text: "Providers" }), h("span", { text: "Select source" })]),
    ...["OpenRouter", "OpenAI", "LiteLLM Local"].map((name) => link("models-popover", "popover-row", [h("span", { text: name }), icon("chevronRight")]))
  ]);
}

/**
 * Render the model selector route popover.
 */
function modelPopover() {
  return h("aside", { class: "popover", "aria-label": "Model selector" }, [
    h("header", { class: "popover-backbar" }, [link("providers-popover", "popover-back", [icon("chevronLeft"), h("strong", { text: "OpenRouter" })])]),
    ...models.slice(0, 3).map((model) => link("new-session", `popover-row${model.active ? " is-selected" : ""}`, [
      h("span", { text: model.label }),
      model.active ? h("span", { "aria-label": "Current model" }, [icon("check")]) : h("span")
    ]))
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
    active === "files" ? filesTool(fileView === "editor") : active === "terminal" ? terminalTool() : browserTool()
  ]);
}

/**
 * Render the browser preview fixture.
 */
function browserTool() {
  return h("section", { class: "tool-body" }, [
    h("div", { class: "address-bar" }, [icon("globe"), h("span", { text: browserState.url })]),
    h("div", { class: "preview-surface" }, [h("h2", { text: browserState.title }), h("p", { text: browserState.summary })])
  ]);
}

/**
 * Render either the file list or lightweight editor fixture.
 */
function filesTool(editor) {
  if (editor) {
    return h("section", { class: "tool-body editor-tool" }, [
      h("nav", { class: "breadcrumbs", "aria-label": "File breadcrumbs" }, [
        h("button", { type: "button", "data-file-target": "explorer" }, [filesState.breadcrumbs[0] || workspace.project]),
        h("span", { text: ">" }),
        h("button", { type: "button", "data-file-target": "explorer" }, [filesState.breadcrumbs[1] || "frontend"]),
        h("span", { text: ">" }),
        h("span", { text: filesState.breadcrumbs[2] || "main.js" })
      ]),
      h("div", { class: "code-pane", tabindex: "0" }, filesState.lines.map((line, index) =>
        h("div", { class: "code-line" }, [
          h("span", { class: "line-number", text: String(index + 1) }),
          h("code", { "aria-label": `Line ${index + 1} code`, class: "line-code", contenteditable: "true", spellcheck: "false", text: line })
        ])
      ))
    ]);
  }
  return h("section", { class: "tool-body file-tool" }, [
    h("h2", { text: workspace.project }),
    ...filesState.roots.map(([name, iconName, target], index) =>
      h("button", {
        class: `file-row${iconName === "file" ? " is-file" : ""}${index === 2 ? " is-active" : ""}`,
        type: "button",
        "data-file-target": target === "file-editor" ? "editor" : "explorer"
      }, [icon(iconName), h("span", { text: name })])
    )
  ]);
}

/**
 * Render the terminal fixture and its resizable result panel.
 */
function terminalTool() {
  return h("section", { class: "tool-body terminal-tool" }, [
    h("pre", { class: "terminal-output", text: terminalState.output }),
    h("div", { class: "vertical-resize-handle", role: "separator", tabindex: "0", "aria-label": "Resize terminal results panel", "aria-orientation": "horizontal", "data-resize-stack": "terminal" }),
    h("div", { class: "terminal-bottom" }, [h("strong", { text: terminalState.title }), h("p", { text: terminalState.summary })])
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
  return h("div", { class: "settings-shell", "data-screen": route }, [
    h("aside", { class: "settings-nav", "aria-label": "Settings navigation" }, [
      link("new-session", "settings-back", [icon("arrowLeft"), h("span", { text: "Back to app" })]),
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
  if (active === "models") return dataList("Models", "Models are fetched from enabled provider connections when available.", models.map((model) => [model.label, model.provider, model.active ? "Current" : "Enabled"]));
  if (active === "runtimes") return runtimes();
  if (active === "configuration") return configuration();
  if (active === "plugins") return plugins();
  if (active === "skills") return skills();
  if (active === "mcp") return mcp();
  return h("section", {}, [
    settingsTitle("Providers", "Manage OpenAI-compatible provider connections. Labels must be unique.", link("settings-add-provider", "button primary", [icon("add"), h("span", { text: "Add Provider" })])),
    h("div", { class: "data-list" }, providers.map((provider) => dataRow(provider.label, `${provider.endpoint} - ${provider.status}`, button("Edit"), true)))
  ]);
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
  return h("article", { class: "data-row" }, [
    h("div", {}, [h("strong", { text: name }), h("span", { text: meta })]),
    middle,
    toggle ? h("button", { class: "switch", type: "button", "aria-label": `${name} enabled` }, [h("span")]) : null
  ]);
}

/**
 * Render the Add Provider form with the documented provider type list.
 */
function providerForm() {
  const fields = [
    { id: "provider-type", label: "Provider Type", options: ["OpenAI", "OpenRouter", "Hugging Face router", "LiteLLM proxy", "Custom OpenAI-compatible endpoint"], type: "select", value: "Custom OpenAI-compatible endpoint" },
    { id: "provider-label", label: "Label", type: "text", value: "OpenRouter - Personal" },
    { id: "api-base-url", label: "API Base URL", type: "url", value: "https://openrouter.ai/api/v1" },
    { id: "api-key", label: "API Key", type: "password", value: "sk-****************" },
    { id: "auth-type", label: "Auth", options: ["Bearer token", "Custom header"], type: "select", value: "Bearer token" },
    { id: "provider-headers", label: "Headers", type: "textarea", value: "{}" }
  ];
  return h("section", {}, [
    settingsTitle("Add Provider", "Save an OpenAI-compatible connection profile."),
    h("form", { class: "provider-form" }, [
      ...fields.map(settingsField),
      h("div", { class: "form-actions" }, [link("settings-providers", "button primary", ["Save Provider"]), link("settings-providers", "button secondary", ["Cancel"])])
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
      : h("input", { id: field.id, name: field.id, type: field.type, value: field.value, spellcheck: field.type === "password" ? "false" : null });
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
      h("div", { class: "plugin-grid" }, pluginCatalog.map((name) => h("article", { class: "plugin-card" }, [
        h("div", { class: "plugin-logo" }, [h("span", { text: name.slice(0, 2) })]),
        h("div", { class: "plugin-copy" }, [h("strong", { text: name }), h("span", { text: "Frontend-local catalog fixture" })]),
        iconButton(`Add ${name}`, "add", "icon-button plugin-add-button", { "data-plugin-connect": name })
      ])))
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
      h("div", { class: "skills-list" }, skillCatalog.map((name) => h("article", { class: "skill-row" }, [
        h("button", { class: "skill-open", type: "button", "data-skill-open": name }, [h("span", { class: "skill-mark" }, [icon("file")]), h("span", { class: "skill-copy" }, [h("strong", { text: name }), h("span", { text: "Project availability fixture" })])]),
        h("span", { class: "skill-scope", text: name === "ChrisAI Agents" ? "Project" : "Personal" }),
        h("button", { class: "skill-switch", type: "button", "aria-label": "Skill enabled" }, [h("span")])
      ])))
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
    h("section", { class: "mcp-section" }, [h("h2", { text: "Servers" }), h("div", { class: "mcp-list" }, mcpServers.map((name) => dataRow(name, "External tool connection fixture", iconButton(`${name} settings`, "settings"), true)))]),
    mcpDialog()
  ]);
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
const boot = beginConnectorStateLoad();
render();
if (boot) {
  boot.then(render);
}
