const route = document.body.dataset.route || "hub";
const app = document.querySelector("#app");

const routes = [
  ["App Start", "./app-start.html", "Folder-first entry before any local authority is granted."],
  ["New Session", "./new-session.html", "Trusted project shell with the empty composer state."],
  ["Chat Session", "./chat-session.html", "Messenger ownership, agent activity, tool call, and approval state."],
  ["Providers Popover", "./providers-popover.html", "Provider picker opened from the model chip."],
  ["Models Popover", "./models-popover.html", "OpenRouter model list with the current model marked."],
  ["File Explorer", "./file-explorer.html", "Right panel file tree state."],
  ["File Editor", "./file-editor.html", "Right panel code view with breadcrumbs."],
  ["Terminal", "./terminal.html", "Read-only terminal output and agent command result panel."],
  ["Settings Providers", "./settings-providers.html", "Provider connection list."],
  ["Add Provider", "./settings-add-provider.html", "Provider form with key below base URL."],
  ["Settings Models", "./settings-models.html", "Fetched models grouped by provider labels."],
  ["Settings Configuration", "./settings-configuration.html", "Approval policy, sandbox settings, and config access."]
];

const projects = [
  { name: "suite", trusted: false },
  { name: "c4os2", trusted: true, sessions: ["Locate Tauri integration", "Draft wireframes"] },
  { name: "techops", trusted: false },
  { name: "ingest", trusted: false },
  { name: "chrisai", trusted: false }
];

const providers = [
  { label: "OpenRouter - Personal", endpoint: "https://openrouter.ai/api/v1", status: "API key saved" },
  { label: "OpenAI - Work", endpoint: "https://api.openai.com/v1", status: "API key saved" },
  { label: "LiteLLM Local", endpoint: "http://localhost:4000/v1", status: "API key saved" }
];

const models = [
  { label: "Gemini 2.5 Flash", provider: "OpenRouter - Personal", active: true },
  { label: "ChatGPT o4", provider: "OpenRouter - Personal", active: false },
  { label: "Grok 2.0", provider: "OpenRouter - Personal", active: false },
  { label: "gpt-4.1", provider: "OpenAI - Work", active: false },
  { label: "local/coder-small", provider: "LiteLLM Local", active: false }
];

const threadItems = [
  {
    type: "message",
    id: "trusted-state-question",
    role: "user",
    body: ["Locate the Tauri integration path and check whether the desktop shell can own trusted project state."]
  },
  {
    type: "message",
    id: "shell-boundary-answer",
    role: "agent",
    body: ["I found the shell boundary and the trusted root checks. The next step is to wire the visible approval state to the file and terminal surfaces."],
    disclosure: {
      body: ["This draft keeps file writes, shell commands, provider calls, and browser previews as separate product surfaces so approval state can be audited."],
      collapsedLabel: "Show More",
      expandedLabel: "Show Less"
    }
  },
  {
    type: "activity",
    id: "read-project-instructions",
    title: "Tool Call",
    label: "Read project instructions",
    status: "Completed"
  },
  {
    type: "activity",
    id: "sensitive-action",
    title: "Agent Run",
    label: "Sensitive action waiting",
    action: "Review Approval"
  }
];

const labels = [
  ["hub.kicker", "C4OS r03"],
  ["hub.title", "Functional Frontend Architecture Draft"],
  ["hub.lead", "A fresh static review artifact built from product requirements, not copied from earlier wireframe code."],
  ["hub.routeList", "Review screens"],
  ["start.kicker", "No trusted project folder"],
  ["start.title", "Open a folder to start working"],
  ["start.lead", "The first usable state requires a local project folder so file access, instructions, skills, runtime policy, approvals, and workspace persistence are scoped before prompting."],
  ["start.actions", "Project entry actions"],
  ["start.recentTitle", "Recent Folder-Backed Workspaces"],
  ["sidebar.search", "Search projects"],
  ["sidebar.plugins", "Plugins"],
  ["sidebar.pluginPlaceholder", "Placeholder"],
  ["sidebar.projects", "Projects"],
  ["sidebar.addProject", "Add Project"],
  ["sidebar.settings", "Settings"],
  ["row.newChat", "New chat"],
  ["row.removeProject", "Remove project"],
  ["topbar.collapseLeft", "Collapse left panel"],
  ["topbar.collapseRight", "Collapse right panel"],
  ["panel.resizeLeft", "Resize left panel"],
  ["panel.resizeRight", "Resize right panel"],
  ["empty.title", "What should we build in c4os2?"],
  ["composer.label", "Prompt composer"],
  ["composer.promptLabel", "Prompt"],
  ["composer.emptyPlaceholder", "Do anything"],
  ["composer.threadPlaceholder", "Ask for follow-up changes"],
  ["composer.attach", "Attach File"],
  ["composer.approvalAria", "Approval policy: Approve for me"],
  ["composer.approvalLabel", "Approve for me"],
  ["composer.microphone", "Use Microphone"],
  ["composer.send", "Send Prompt"],
  ["popover.providerSelector", "Provider selector"],
  ["popover.providersTitle", "Providers"],
  ["popover.providersSubtitle", "Select source"],
  ["popover.modelSelector", "Model selector"],
  ["popover.currentModel", "Current model"],
  ["thread.list", "Session messages"],
  ["message.collapse", "Collapse message"],
  ["message.showMore", "Show More"],
  ["message.showLess", "Show Less"],
  ["toolPanel.label", "Workspace tools"],
  ["toolPanel.tabs", "Workspace tool tabs"],
  ["browser.previewTitle", "Rendered page mock"],
  ["browser.previewText", "Single browser surface; tabs are out of scope."],
  ["files.breadcrumbs", "File breadcrumbs"],
  ["terminal.resultTitle", "AI command preview/results"],
  ["terminal.resultText", "Read-only bottom panel. Resize handle sits on this top boundary."],
  ["settings.navLabel", "Settings navigation"],
  ["settings.back", "Back to app"],
  ["settings.kicker", "Settings"],
  ["settings.providerSummary", "Manage OpenAI-compatible provider connections. Labels must be unique."],
  ["settings.addProvider", "Add Provider"],
  ["settings.edit", "Edit"],
  ["settings.enabledSuffix", "enabled"],
  ["settings.addProviderSummary", "Save an OpenAI-compatible connection profile."],
  ["settings.saveProvider", "Save Provider"],
  ["settings.cancel", "Cancel"],
  ["settings.modelsSummary", "Models are fetched from enabled provider connections when available."],
  ["settings.modelCurrent", "Current"],
  ["settings.modelEnabled", "Enabled"],
  ["settings.configurationSummary", "Configure approval policy and sandbox settings."],
  ["settings.openConfig", "Open config.toml externally"]
];

const copy = Object.fromEntries(labels);

const activeWorkspace = {
  project: "c4os2",
  session: "Locate Tauri integration"
};

const startActions = [
  { label: "Open Folder", className: "button primary" },
  { label: "Clone Repository", className: "button secondary" },
  { label: "Open Workspace File", className: "button secondary" }
];

const recentWorkspaces = [
  { name: "c4os2", badge: "Trusted" },
  { name: "agent-lab", badge: "2 roots" },
  { name: "docs-workbench", badge: "Saved" }
];

const composerContextChips = [
  { label: "main", icon: "gitBranch" }
];

const composerModelChip = {
  label: "gemini-flash-2.5",
  icon: "bot",
  href: "./models-popover.html"
};

const providerChoices = ["OpenRouter", "OpenAI", "LiteLLM Local"];

const toolTabs = [
  { key: "browser", label: "Browser", icon: "globe", href: "./new-session.html" },
  { key: "files", label: "Files", icon: "file", href: "./file-explorer.html" },
  { key: "terminal", label: "Terminal", icon: "terminal", href: "./terminal.html" }
];

const browserPreview = {
  address: "http://127.0.0.1:13000"
};

const editorPreview = {
  breadcrumbs: ["c4os2", "frontend", "main.js"],
  code: `1  import { startWorkspace } from "./runtime";\n2\n3  const project = "c4os2";\n4  const trustedRoot = true;\n5\n6  startWorkspace({ project, trustedRoot });\n7\n8  // Code view fills the full tool body.`
};

const fileRows = [
  { name: "backend", icon: "folder", href: "./file-explorer.html" },
  { name: "frontend", icon: "folder", href: "./file-explorer.html" },
  { name: "main.js", icon: "file", href: "./file-editor.html", active: true },
  { name: "index.html", icon: "file", href: "./file-editor.html" },
  { name: "tests", icon: "folder", href: "./file-explorer.html" }
];

const terminalPreview = {
  output: "$ npm run dev\nready in 614ms\nlocal preview available at 127.0.0.1:3000"
};

const settingsNavItems = [
  { label: "Providers", key: "providers", href: "./settings-providers.html", icon: "key" },
  { label: "Models", key: "models", href: "./settings-models.html", icon: "bot" },
  { label: "Configuration", key: "configuration", href: "./settings-configuration.html", icon: "settings" },
  { label: "Skills", key: "skills", href: "./settings-providers.html", icon: "file" },
  { label: "MCP Servers", key: "mcp", href: "./settings-providers.html", icon: "server" },
  { label: "Hooks", key: "hooks", href: "./settings-providers.html", icon: "webhook" }
];

const providerFormFields = [
  { label: "Provider Type", id: "provider-type", type: "select", value: "OpenAI-compatible", options: ["OpenAI-compatible", "OpenRouter", "LiteLLM proxy"] },
  { label: "Label", id: "provider-label", type: "text", value: "OpenRouter - Personal" },
  { label: "API Base URL", id: "api-base-url", type: "url", value: "https://openrouter.ai/api/v1" },
  { label: "API Key", id: "api-key", type: "password", value: "sk-****************" },
  { label: "Auth", id: "auth-type", type: "select", value: "Bearer token", options: ["Bearer token", "Custom header"] },
  { label: "Headers", id: "provider-headers", type: "textarea", value: "{}" }
];

const configCards = [
  { title: "Approval Policy", text: "Choose when the app asks before high-impact actions.", action: "On request" },
  { title: "Sandbox Settings", text: "Choose how much agents can do when running commands.", action: "Workspace write" }
];

const icons = {
  add: "M12 5v14M5 12h14",
  arrowLeft: "M19 12H5m7 7-7-7 7-7",
  bot: "M12 8V4H8m0 4h8a4 4 0 0 1 4 4v4a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4v-4a4 4 0 0 1 4-4Zm1 5v2m6-2v2M2 14h2m16 0h2",
  check: "m5 12 4 4L19 6",
  chevronDown: "m6 9 6 6 6-6",
  chevronLeft: "m15 18-6-6 6-6",
  chevronRight: "m9 18 6-6-6-6",
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Zm0 0v6h6",
  folder: "M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z",
  globe: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Zm0-20a15 15 0 0 1 0 20m0-20a15 15 0 0 0 0 20M2 12h20",
  gitBranch: "M6 3v12m0 6a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm12-12a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm0 0a9 9 0 0 1-9 9",
  key: "M2 18v3h3l9-9m-1-4a6 6 0 1 0 12 0 6 6 0 0 0-12 0Z",
  mic: "M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Zm7 8v2a7 7 0 0 1-14 0v-2m7 9v3",
  paperclip: "m21 11-9 9a6 6 0 0 1-8-8l9-9a4 4 0 0 1 6 6l-9 9a2 2 0 0 1-3-3l8-8",
  pencil: "M21 6 7 20H3v-4L17 2a3 3 0 0 1 4 4Z",
  panelLeft: "M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Zm6-2v18",
  panelRight: "M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Zm12-2v18",
  plug: "M9 2v6m6-6v6m3 0v5a6 6 0 0 1-12 0V8Zm-6 14v-5",
  search: "m21 21-4-4m2-6a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z",
  send: "M3 4 21 12 3 20l3-8Zm3 8h15",
  server: "M3 3h18v7H3Zm0 11h18v7H3ZM7 6h.01M7 17h.01",
  settings: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0-6v3m0 14v3M4.9 4.9 7 7m10 10 2.1 2.1M2 12h3m14 0h3M4.9 19.1 7 17m10-10 2.1-2.1",
  shield: "M12 3c3 2 5 3 8 3v6c0 5-3 8-8 10-5-2-8-5-8-10V6c3 0 5-1 8-3Z",
  terminal: "m4 17 6-5-6-5m8 10h8",
  trash: "M3 6h18M8 6V4h8v2m3 0v14H5V6m5 5v6m4-6v6",
  webhook: "M18 16h-6a4 4 0 0 1-4-4m-2 4 3-3-3-3m0-2h6a4 4 0 0 1 4 4m2-4-3 3 3 3"
};

function h(tag, props = {}, children = []) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (value === false || value == null) continue;
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else node.setAttribute(key, String(value));
  }
  for (const child of [].concat(children)) {
    if (child == null) continue;
    node.append(child.nodeType ? child : document.createTextNode(String(child)));
  }
  return node;
}

function svgIcon(name, label = "") {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("class", "icon");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "2");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  if (label) {
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", label);
  } else {
    svg.setAttribute("aria-hidden", "true");
  }
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", icons[name] || icons.file);
  svg.append(path);
  return svg;
}

function link(className, href, children, attrs = {}) {
  return h("a", { class: className, href, ...attrs }, children);
}

function button(className, label, iconName) {
  const children = iconName ? [svgIcon(iconName), h("span", { text: label })] : [label];
  return h("button", { class: className, type: "button", "aria-label": iconName ? label : null }, children);
}

function iconButton(label, iconName, extraClass = "", attrs = {}) {
  return h("button", { class: `icon-button ${extraClass}`.trim(), type: "button", "aria-label": label, ...attrs }, [svgIcon(iconName)]);
}

function chip(label, iconName, href) {
  const children = [svgIcon(iconName), h("span", { text: label })];
  return href ? link("chip", href, children) : h("button", { class: "chip", type: "button" }, children);
}

function renderHub() {
  const cards = routes.map(([title, hrefValue, summary]) =>
    link("hub-card", hrefValue, [
      h("strong", { text: title }),
      h("span", { text: summary })
    ])
  );
  app.replaceChildren(
    h("main", { id: "main", class: "hub", tabindex: "-1" }, [
      h("p", { class: "kicker", text: copy["hub.kicker"] }),
      h("h1", { text: copy["hub.title"] }),
      h("p", { class: "lead", text: copy["hub.lead"] }),
      h("section", { class: "hub-grid", "aria-label": copy["hub.routeList"] }, cards)
    ])
  );
}

function renderStart() {
  app.replaceChildren(
    h("main", { id: "main", class: "start-view", tabindex: "-1" }, [
      h("section", { class: "start-copy" }, [
        h("p", { class: "kicker", text: copy["start.kicker"] }),
        h("h1", { text: copy["start.title"] }),
        h("p", { class: "lead", text: copy["start.lead"] }),
        h("div", { class: "action-row", "aria-label": copy["start.actions"] }, startActions.map((action) => button(action.className, action.label)))
      ]),
      h("section", { class: "recent-panel", "aria-labelledby": "recent-workspaces" }, [
        h("h2", { id: "recent-workspaces", text: copy["start.recentTitle"] }),
        ...recentWorkspaces.map((workspace) => workspaceRow(workspace.name, workspace.badge))
      ])
    ])
  );
}

function workspaceRow(name, badge) {
  return link("workspace-row", "./new-session.html", [
    h("strong", { text: name, translate: "no" }),
    h("span", { text: badge })
  ]);
}

function renderShell(screen) {
  const activeSession = screen === "chat-session";
  const activeTool = screen === "file-explorer" || screen === "file-editor" ? "files" : screen === "terminal" ? "terminal" : "browser";
  app.replaceChildren(
    h("div", { class: "app-shell" }, [
      renderSidebar(activeSession),
      resizeHandle("left"),
      h("section", { class: "workbench", id: "main", tabindex: "-1" }, [
        renderTopbar(activeSession ? activeWorkspace.session : activeWorkspace.project),
        activeSession ? renderThread() : renderEmptyWorkspace(screen)
      ]),
      resizeHandle("right"),
      renderToolPanel(activeTool, screen)
    ])
  );
  bindShowMore();
  bindPanelControls();
}

function resizeHandle(side) {
  return h("div", {
    class: `resize-handle ${side}-resizer`,
    role: "separator",
    tabindex: "0",
    "aria-label": side === "left" ? copy["panel.resizeLeft"] : copy["panel.resizeRight"],
    "aria-orientation": "vertical",
    "data-resize-panel": side
  });
}

function renderSidebar(activeSession) {
  const rows = [];
  projects.forEach((project) => {
    rows.push(projectRow(project, activeSession));
    (project.sessions || []).forEach((session) => rows.push(sessionRow(session, activeSession && session === activeWorkspace.session)));
  });
  return h("aside", { class: "sidebar", "aria-label": copy["sidebar.projects"] }, [
    h("label", { class: "search-field" }, [
      svgIcon("search"),
      h("span", { class: "sr-only", text: copy["sidebar.search"] }),
      h("input", { type: "search", name: "project-search", autocomplete: "off", placeholder: copy["sidebar.search"] })
    ]),
    h("div", { class: "plugin-line" }, [svgIcon("plug"), h("span", { text: copy["sidebar.plugins"] }), h("span", { text: copy["sidebar.pluginPlaceholder"] })]),
    h("div", { class: "nav-section" }, [
      h("div", { class: "section-head" }, [h("span", { text: copy["sidebar.projects"] }), iconButton(copy["sidebar.addProject"], "add")]),
      ...rows
    ]),
    link("settings-entry", "./settings-providers.html", [svgIcon("settings"), h("span", { text: copy["sidebar.settings"] })])
  ]);
}

function projectRow(project, activeSession) {
  const selected = project.name === activeWorkspace.project && !activeSession;
  return link(`project-row${selected ? " is-active" : ""}`, "./new-session.html", [
    svgIcon("folder"),
    h("span", { text: project.name, translate: "no" }),
    h("span", { class: "row-tools" }, [svgIcon("pencil", copy["row.newChat"]), svgIcon("trash", copy["row.removeProject"])])
  ], { "aria-current": selected ? "page" : null });
}

function sessionRow(label, active) {
  return link(`session-row${active ? " is-active" : ""}`, "./chat-session.html", [h("span", { text: label })], { "aria-current": active ? "page" : null });
}

function renderTopbar(title) {
  return h("header", { class: "topbar" }, [
    iconButton(copy["topbar.collapseLeft"], "panelLeft", "", { "data-panel-toggle": "left", "aria-pressed": "false" }),
    h("strong", { text: title, translate: "no" }),
    iconButton(copy["topbar.collapseRight"], "panelRight", "", { "data-panel-toggle": "right", "aria-pressed": "false" })
  ]);
}

function renderEmptyWorkspace(screen) {
  const popover = screen === "providers-popover" ? providerPopover() : screen === "models-popover" ? modelPopover() : null;
  return h("main", { class: "empty-workspace" }, [
    h("h1", { text: copy["empty.title"] }),
    composer(copy["composer.emptyPlaceholder"], popover)
  ]);
}

function composer(placeholder, popover) {
  return h("section", { class: "composer", "aria-label": copy["composer.label"] }, [
    h("div", { class: "prompt-box", role: "textbox", "aria-label": copy["composer.promptLabel"], "aria-multiline": "true", contenteditable: "true", spellcheck: "true", "data-placeholder": placeholder }),
    h("div", { class: "composer-controls" }, [
      iconButton(copy["composer.attach"], "paperclip"),
      h("button", { class: "chip", type: "button", "aria-label": copy["composer.approvalAria"] }, [svgIcon("shield"), h("span", { text: copy["composer.approvalLabel"] })]),
      h("span", { class: "spacer" }),
      chip(composerModelChip.label, composerModelChip.icon, composerModelChip.href),
      iconButton(copy["composer.microphone"], "mic"),
      iconButton(copy["composer.send"], "send", "send-button")
    ]),
    h("div", { class: "context-strip" }, composerContextChips.map((item) => chip(item.label, item.icon))),
    popover
  ]);
}

function providerPopover() {
  return h("aside", { class: "popover", "aria-label": copy["popover.providerSelector"] }, [
    h("header", { class: "popover-title" }, [h("strong", { text: copy["popover.providersTitle"] }), h("span", { text: copy["popover.providersSubtitle"] })]),
    ...providerChoices.map((provider) =>
      link("popover-row", "./models-popover.html", [h("span", { text: provider }), svgIcon("chevronRight")])
    )
  ]);
}

function modelPopover() {
  return h("aside", { class: "popover", "aria-label": copy["popover.modelSelector"] }, [
    h("header", { class: "popover-backbar" }, [link("popover-back", "./providers-popover.html", [svgIcon("chevronLeft"), h("strong", { text: providerChoices[0] })])]),
    ...models.slice(0, 3).map((model) =>
      link(`popover-row${model.active ? " is-selected" : ""}`, "./new-session.html", [
        h("span", { text: model.label }),
        model.active ? svgIcon("check", copy["popover.currentModel"]) : h("span")
      ])
    )
  ]);
}

function renderThread() {
  return h("main", { class: "thread-view" }, [
    h("div", { class: "thread-list", "aria-label": copy["thread.list"] }, threadItems.map(renderThreadItem)),
    h("div", { class: "composer-dock" }, [composer(copy["composer.threadPlaceholder"])])
  ]);
}

function renderThreadItem(item) {
  return item.type === "activity" ? activityCard(item) : message(item);
}

function message(item) {
  const disclosureLines = item.disclosure?.body || [];
  const hasDisclosure = disclosureLines.length > 0;
  const disclosureId = `${item.id}-disclosure`;
  const article = h("article", { class: `message ${item.role}${hasDisclosure ? " has-actions" : ""}`, "data-message-id": item.id }, [
    hasDisclosure ? h("div", { class: "message-actions" }, [iconButton(copy["message.collapse"], "chevronDown")]) : null,
    ...item.body.map((line) => h("p", { text: line }))
  ]);
  if (hasDisclosure) {
    const collapsedLabel = item.disclosure.collapsedLabel || copy["message.showMore"];
    const expandedLabel = item.disclosure.expandedLabel || copy["message.showLess"];
    article.append(
      h("div", { class: "message-extra", id: disclosureId }, disclosureLines.map((line) => h("p", { text: line }))),
      h("button", {
        class: "text-button",
        type: "button",
        "data-toggle": "message",
        "data-collapsed-label": collapsedLabel,
        "data-expanded-label": expandedLabel,
        "aria-controls": disclosureId,
        "aria-expanded": "false"
      }, [collapsedLabel])
    );
  }
  return article;
}

function activityCard(item) {
  return h("article", { class: "activity-card" }, [
    h("header", {}, [h("strong", { text: item.title }), iconButton(`${copy["message.collapse"]}: ${item.title}`, "chevronDown")]),
    h("div", { class: "activity-row" }, [
      h("span", { text: item.label }),
      item.action ? button("button secondary", item.action) : h("span", { class: "status-pill", text: item.status })
    ])
  ]);
}

function renderToolPanel(active, screen) {
  return h("aside", { class: "tool-panel", "aria-label": copy["toolPanel.label"] }, [
    h("nav", { class: "tool-tabs", "aria-label": copy["toolPanel.tabs"] }, toolTabs.map((tab) =>
      link(`tool-tab${active === tab.key ? " is-active" : ""}`, tab.href, [svgIcon(tab.icon), h("span", { text: tab.label })])
    )),
    active === "files" ? filesTool(screen === "file-editor") : active === "terminal" ? terminalTool() : browserTool()
  ]);
}

function browserTool() {
  return h("section", { class: "tool-body browser-tool" }, [
    h("div", { class: "address-bar" }, [svgIcon("globe"), h("span", { text: browserPreview.address })]),
    h("div", { class: "preview-surface" }, [h("h2", { text: copy["browser.previewTitle"] }), h("p", { text: copy["browser.previewText"] })])
  ]);
}

function filesTool(editorOpen) {
  if (editorOpen) {
    return h("section", { class: "tool-body editor-tool" }, [
      h("nav", { class: "breadcrumbs", "aria-label": copy["files.breadcrumbs"] }, editorPreview.breadcrumbs.flatMap((crumb, index) =>
        index < editorPreview.breadcrumbs.length - 1
          ? [link("", "./file-explorer.html", [crumb]), h("span", { text: ">" })]
          : [h("span", { text: crumb })]
      )),
      h("pre", { class: "code-pane", text: editorPreview.code })
    ]);
  }
  return h("section", { class: "tool-body file-tool" }, [
    h("h2", { text: activeWorkspace.project, translate: "no" }),
    ...fileRows.map((row) =>
      link(`file-row${row.active ? " is-active" : ""}${row.icon === "file" ? " is-file" : ""}`, row.href, [svgIcon(row.icon), h("span", { text: row.name, translate: "no" })])
    )
  ]);
}

function terminalTool() {
  return h("section", { class: "tool-body terminal-tool" }, [
    h("pre", { class: "terminal-output", text: terminalPreview.output }),
    h("div", { class: "terminal-bottom" }, [
      h("strong", { text: copy["terminal.resultTitle"] }),
      h("p", { text: copy["terminal.resultText"] })
    ])
  ]);
}

function renderSettings(screen) {
  const section = screen.replace("settings-", "");
  app.replaceChildren(
    h("div", { class: "settings-shell" }, [
      settingsNav(section),
      h("main", { id: "main", class: "settings-main", tabindex: "-1" }, [settingsBody(section)])
    ])
  );
}

function settingsNav(active) {
  return h("aside", { class: "settings-nav", "aria-label": copy["settings.navLabel"] }, [
    link("settings-back", "./new-session.html", [svgIcon("arrowLeft"), h("span", { text: copy["settings.back"] })]),
    h("p", { class: "kicker", text: copy["settings.kicker"] }),
    ...settingsNavItems.map((item) =>
      link(`settings-link${active === item.key ? " is-active" : ""}`, item.href, [svgIcon(item.icon), h("span", { text: item.label })], { "aria-current": active === item.key ? "page" : null })
    )
  ]);
}

function settingsBody(active) {
  if (active === "add-provider") return providerForm();
  if (active === "models") return modelsSettings();
  if (active === "configuration") return configurationSettings();
  return providersSettings();
}

function settingsTitle(title, text, action) {
  return h("header", { class: "settings-title" }, [
    h("div", {}, [h("h1", { text: title }), h("p", { text })]),
    action
  ]);
}

function providersSettings() {
  return h("section", {}, [
    settingsTitle(settingsNavItems[0].label, copy["settings.providerSummary"], link("button primary", "./settings-add-provider.html", [svgIcon("add"), h("span", { text: copy["settings.addProvider"] })])),
    h("div", { class: "data-list" }, providers.map((provider) =>
      h("article", { class: "data-row" }, [
        h("div", {}, [h("strong", { text: provider.label }), h("span", { text: `${provider.endpoint} - ${provider.status}` })]),
        button("button secondary", copy["settings.edit"]),
        h("button", { class: "switch is-on", type: "button", "aria-label": `${provider.label} ${copy["settings.enabledSuffix"]}` }, [h("span")])
      ])
    ))
  ]);
}

function providerForm() {
  return h("section", {}, [
    settingsTitle(copy["settings.addProvider"], copy["settings.addProviderSummary"]),
    h("form", { class: "provider-form" }, [
      ...providerFormFields.map((item) => field(item.label, item.id, item.type, item.value, item.options)),
      h("div", { class: "form-actions" }, [
        link("button primary", "./settings-providers.html", [copy["settings.saveProvider"]]),
        link("button secondary", "./settings-providers.html", [copy["settings.cancel"]])
      ])
    ])
  ]);
}

function field(label, id, type, value, options = []) {
  const control = type === "select"
    ? h("select", { id, name: id, autocomplete: "off" }, options.map((option) => h("option", { text: option, selected: option === value })))
    : type === "textarea"
      ? h("textarea", { id, name: id, rows: "5", spellcheck: "false", autocomplete: "off" }, [value])
      : h("input", { id, name: id, type, value, autocomplete: "off", spellcheck: type === "password" ? "false" : null, inputmode: type === "url" ? "url" : null });
  return h("div", { class: "field" }, [h("label", { for: id, text: label }), control]);
}

function modelsSettings() {
  return h("section", {}, [
    settingsTitle(settingsNavItems[1].label, copy["settings.modelsSummary"]),
    h("div", { class: "data-list" }, models.map((model) =>
      h("article", { class: "data-row" }, [
        h("div", {}, [h("strong", { text: model.label }), h("span", { text: model.provider })]),
        h("span", { class: "status-pill", text: model.active ? copy["settings.modelCurrent"] : copy["settings.modelEnabled"] }),
        h("button", { class: "switch is-on", type: "button", "aria-label": `${model.label} ${copy["settings.enabledSuffix"]}` }, [h("span")])
      ])
    ))
  ]);
}

function configurationSettings() {
  return h("section", {}, [
    settingsTitle(settingsNavItems[2].label, copy["settings.configurationSummary"], button("button secondary", copy["settings.openConfig"], "file")),
    h("div", { class: "config-grid" }, configCards.map((card) => configCard(card.title, card.text, card.action)))
  ]);
}

function configCard(title, text, action) {
  return h("article", { class: "config-card" }, [
    h("div", {}, [h("h2", { text: title }), h("p", { text })]),
    button("button secondary", action)
  ]);
}

function bindShowMore() {
  document.querySelectorAll("[data-toggle='message']").forEach((control) => {
    control.addEventListener("click", () => {
      const card = control.closest(".message");
      const expanded = card.classList.toggle("is-expanded");
      control.setAttribute("aria-expanded", String(expanded));
      control.textContent = expanded ? control.dataset.expandedLabel : control.dataset.collapsedLabel;
    });
  });
}

function bindPanelControls() {
  const shell = document.querySelector(".app-shell");
  if (!shell) return;

  shell.querySelectorAll("[data-panel-toggle]").forEach((control) => {
    control.addEventListener("click", () => {
      const side = control.dataset.panelToggle;
      const collapsed = !shell.classList.contains(`is-${side}-collapsed`);
      shell.classList.toggle(`is-${side}-collapsed`, collapsed);
      control.setAttribute("aria-pressed", String(collapsed));
    });
  });

  shell.querySelectorAll("[data-resize-panel]").forEach((handle) => {
    handle.addEventListener("pointerdown", (event) => {
      const side = handle.dataset.resizePanel;
      if (shell.classList.contains(`is-${side}-collapsed`)) return;
      const panel = side === "left" ? shell.querySelector(".sidebar") : shell.querySelector(".tool-panel");
      const startX = event.clientX;
      const startWidth = panel.getBoundingClientRect().width;
      const limits = side === "left" ? { min: 220, max: 460, prop: "--left-panel" } : { min: 280, max: 560, prop: "--right-panel" };

      handle.setPointerCapture(event.pointerId);
      shell.classList.add("is-resizing");

      const onPointerMove = (moveEvent) => {
        const delta = side === "left" ? moveEvent.clientX - startX : startX - moveEvent.clientX;
        const width = Math.min(limits.max, Math.max(limits.min, startWidth + delta));
        shell.style.setProperty(limits.prop, `${Math.round(width)}px`);
      };

      const onPointerUp = () => {
        shell.classList.remove("is-resizing");
        handle.removeEventListener("pointermove", onPointerMove);
        handle.removeEventListener("pointerup", onPointerUp);
        handle.removeEventListener("pointercancel", onPointerUp);
      };

      handle.addEventListener("pointermove", onPointerMove);
      handle.addEventListener("pointerup", onPointerUp);
      handle.addEventListener("pointercancel", onPointerUp);
    });
  });
}

if (route === "hub") renderHub();
else if (route === "app-start") renderStart();
else if (route.startsWith("settings")) renderSettings(route);
else renderShell(route);
