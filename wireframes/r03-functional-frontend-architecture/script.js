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
  ["Settings Configuration", "./settings-configuration.html", "Approval policy, sandbox settings, and config access."],
  ["Settings Plugins", "./settings-plugins.html", "Installed plugin controls grouped below configuration."],
  ["Settings Skills", "./settings-skills.html", "Installed skill controls and detail state."],
  ["Settings MCP Servers", "./settings-mcp.html", "MCP server connections and custom server form."]
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
  ["sidebar.projects", "Projects"],
  ["sidebar.addProject", "Add Project"],
  ["sidebar.settings", "Settings"],
  ["row.newChat", "New chat"],
  ["row.removeProject", "Remove project"],
  ["topbar.collapseLeft", "Collapse left panel"],
  ["topbar.collapseRight", "Collapse right panel"],
  ["panel.resizeLeft", "Resize left panel"],
  ["panel.resizeRight", "Resize right panel"],
  ["panel.resizeTerminal", "Resize terminal results panel"],
  ["empty.title", "What should we build in c4os2?"],
  ["composer.label", "Prompt composer"],
  ["composer.promptLabel", "Prompt"],
  ["composer.emptyPlaceholder", "Do anything"],
  ["composer.threadPlaceholder", "Ask for follow-up changes"],
  ["composer.attach", "Attach File"],
  ["composer.attachments", "Attached files"],
  ["composer.removeAttachment", "Remove attachment"],
  ["composer.approvalAria", "Approval policy: Approve for me"],
  ["composer.approvalLabel", "Approve for me"],
  ["composer.approvalAsk", "Ask for approval"],
  ["composer.branchAria", "Branch: main"],
  ["composer.branchReadonly", "Branch locked for this chat"],
  ["composer.modelReadonly", "Model locked for this chat"],
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
  ["settings.openConfig", "Open config.toml externally"],
  ["settings.pluginsSummary", "Manage installed plugins and extension surfaces."]
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

const approvalOptions = [
  { label: "Ask for approval", value: "ask" },
  { label: "Approve for me", value: "auto" }
];

const branchOptions = [
  { label: "main", value: "main" },
  { label: "feature/trust-shell", value: "feature/trust-shell" },
  { label: "+ Create branch", value: "create" }
];

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
  lines: [
    'import { startWorkspace } from "./runtime";',
    "",
    'const project = "c4os2";',
    "const trustedRoot = true;",
    "",
    "startWorkspace({ project, trustedRoot });",
    "",
    "// Code view fills the full tool body."
  ]
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
  { label: "Plugins", key: "plugins", href: "./settings-plugins.html", icon: "plug", dividerBefore: true },
  { label: "Skills", key: "skills", href: "./settings-skills.html", icon: "file" },
  { label: "MCP Servers", key: "mcp", href: "./settings-mcp.html", icon: "server" }
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

const pluginCatalog = [
  { name: "GitHub", description: "Triage PRs, issues, CI, and publish flows", logo: "GH", tone: "github" },
  { name: "Slack", description: "Read and manage Slack", logo: "S", tone: "slack" },
  { name: "Notion", description: "Notion workflows for specs, research,...", logo: "N", tone: "notion" },
  { name: "Linear", description: "Find and reference issues and projects.", logo: "L", tone: "linear" },
  { name: "Gmail", description: "Read and manage Gmail", logo: "M", tone: "gmail" },
  { name: "Google Calendar", description: "Manage Google Calendar events and...", logo: "31", tone: "calendar" },
  { name: "Google Drive", description: "Work across Drive, Docs, Sheets, and...", logo: "D", tone: "drive" },
  { name: "Figma", description: "Design-to-code workflows powered by th...", logo: "F", tone: "figma" },
  { name: "Vercel", description: "Vercel ecosystem guidance for Codex", logo: "V", tone: "vercel" }
];

const skillCatalog = [
  { name: "Ab Testing", scope: "Personal", description: "When the user wants to plan, design, or implement an A/B test or experiment, or optimize a test plan.", detail: "Use this skill when a product, page, onboarding step, pricing flow, or campaign needs a clear experiment design. It helps define hypotheses, variants, success metrics, guardrails, and rollout decisions." },
  { name: "Ad Creative", scope: "Personal", description: "When the user wants to generate, iterate, or scale ad creative - headlines, hooks, variants, and angles.", detail: "Use this skill to turn campaign goals into compact creative directions, message variants, visual concepts, and testable ad copy." },
  { name: "Ads", scope: "Personal", description: "When the user wants help with paid advertising campaigns on Google Ads, Meta, or other acquisition channels.", detail: "Use this skill for campaign structure, targeting, bidding, budget allocation, and diagnosing paid acquisition performance." },
  { name: "Ai Seo", scope: "Personal", description: "When the user wants to optimize content for AI search engines, get cited by LLMs, and improve answer visibility.", detail: "Use this skill for answer-engine positioning, citation-worthy content, schema alignment, and source clarity." },
  { name: "Analytics", scope: "Personal", description: "When the user wants to set up, improve, or audit analytics tracking and measurement quality.", detail: "Use this skill to define events, funnels, attribution needs, dashboards, and data quality checks." },
  { name: "Aso", scope: "Personal", description: "When the user wants to audit or optimize an App Store or Google Play listing. Also useful for keyword and conversion work.", detail: "Use this skill for app listing positioning, screenshots, keywords, ratings strategy, and conversion review." },
  { name: "ChrisAI Agents", scope: "Project", description: "Set up project-local .agents workflows", detail: "Use this skill to set up or repair a project-local .agents/ operating surface. The deliverable is the folder contract itself: .agents/AGENTS.md, local workflow files, and only the files or records needed for the current setup or repair. Future agents should be able to work from .agents/AGENTS.md and .agents/workflows/ without loading this skill.\n\nCore Jobs\n- Initialize or repair .agents/AGENTS.md.\n- Generate or repair local .agents/workflows/*.md files, including context-ingestion.md for future knowledge-base intake.\n- Establish the .agents folder contract without creating empty folders or placeholder files before they are needed.\n- Seed only the minimal .agents/context/, research, spec, progress, or handoff records needed for the current setup or repair task.\n\nTask Routing\nRead only the references needed for the task." }
];

const mcpServers = [
  "node_repl",
  "openaiDeveloperDocs",
  "stackpress_blog_mcp"
];

const icons = {
  add: "M12 5v14M5 12h14",
  arrowLeft: "M19 12H5m7 7-7-7 7-7",
  bot: "M12 8V4H8m0 4h8a4 4 0 0 1 4 4v4a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4v-4a4 4 0 0 1 4-4Zm1 5v2m6-2v2M2 14h2m16 0h2",
  check: "m5 12 4 4L19 6",
  chevronDown: "m6 9 6 6 6-6",
  chevronLeft: "m15 18-6-6 6-6",
  chevronRight: "m9 18 6-6-6-6",
  external: "M7 17 17 7m0 0h-7m7 0v7",
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Zm0 0v6h6",
  folder: "M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z",
  globe: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Zm0-20a15 15 0 0 1 0 20m0-20a15 15 0 0 0 0 20M2 12h20",
  gitBranch: "M6 3v12m0 6a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm12-12a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm0 0a9 9 0 0 1-9 9",
  key: "M7 14a5 5 0 1 1 3.5-8.5A5 5 0 0 1 7 14Zm7-4 7-7m-3 3 3 3m-6 0 3 3",
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
  webhook: "M6 3v6a4 4 0 0 0 4 4h4a4 4 0 0 1 4 4v4M6 9l-3-3m3 3 3-3m9 9 3 3m-3-3-3 3",
  x: "M18 6 6 18M6 6l12 12"
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
  bindTerminalResize();
  bindComposerControls();
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
    composer(copy["composer.emptyPlaceholder"], { popover })
  ]);
}

function composer(placeholder, options = {}) {
  const readonlyContext = Boolean(options.readonlyContext);
  return h("section", { class: "composer", "aria-label": copy["composer.label"] }, [
    h("input", { class: "attachment-input", type: "file", multiple: "true", tabindex: "-1", "aria-hidden": "true" }),
    h("div", { class: "attachment-preview", "data-attachments": "", "aria-label": copy["composer.attachments"], hidden: "true" }),
    h("div", { class: "prompt-box", role: "textbox", "aria-label": copy["composer.promptLabel"], "aria-multiline": "true", contenteditable: "true", spellcheck: "true", "data-placeholder": placeholder }),
    h("div", { class: "composer-controls" }, [
      iconButton(copy["composer.attach"], "paperclip", "", { "data-attach-button": "" }),
      h("button", { class: "chip", type: "button", "aria-label": copy["composer.approvalAria"], "data-popover-trigger": "approval" }, [svgIcon("shield"), h("span", { text: copy["composer.approvalLabel"] })]),
      h("span", { class: "spacer" }),
      iconButton(copy["composer.microphone"], "mic"),
      iconButton(copy["composer.send"], "send", "send-button")
    ]),
    h("div", { class: "context-strip" }, [
      branchChip(readonlyContext),
      h("span", { class: "spacer" }),
      modelChip(readonlyContext)
    ]),
    composerPopover("approval", approvalOptions),
    readonlyContext ? null : composerPopover("branch", branchOptions),
    options.popover
  ]);
}

function branchChip(readonlyContext) {
  const label = composerContextChips[0].label;
  const children = [svgIcon(composerContextChips[0].icon), h("span", { text: label })];
  return readonlyContext
    ? h("span", { class: "chip readonly-chip", "aria-label": copy["composer.branchReadonly"] }, children)
    : h("button", { class: "chip", type: "button", "aria-label": copy["composer.branchAria"], "data-popover-trigger": "branch" }, children);
}

function modelChip(readonlyContext) {
  const children = [svgIcon(composerModelChip.icon), h("span", { text: composerModelChip.label })];
  return readonlyContext
    ? h("span", { class: "chip readonly-chip", "aria-label": copy["composer.modelReadonly"] }, children)
    : link("chip", composerModelChip.href, children);
}

function composerPopover(kind, options) {
  return h("div", { class: `composer-popover ${kind}-popover`, hidden: "true", "data-popover": kind }, options.map((option) =>
    h("button", { class: "popover-row", type: "button", "data-popover-option": option.value }, [h("span", { text: option.label })])
  ));
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
    h("div", { class: "composer-dock" }, [composer(copy["composer.threadPlaceholder"], { readonlyContext: true })])
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
      h("div", { class: "code-pane", role: "region", tabindex: "0", "aria-label": copy["files.breadcrumbs"] }, editorPreview.lines.map((line, index) =>
        h("div", { class: "code-line" }, [
          h("span", { class: "line-number", text: String(index + 1) }),
          h("code", { class: "line-code", text: line })
        ])
      ))
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
    h("div", {
      class: "vertical-resize-handle",
      role: "separator",
      tabindex: "0",
      "aria-label": copy["panel.resizeTerminal"],
      "aria-orientation": "horizontal",
      "data-resize-stack": "terminal"
    }),
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
  bindPluginConnect();
  bindMarketplaceControls();
  bindSkillDetails();
  bindMcpControls();
}

function settingsNav(active) {
  return h("aside", { class: "settings-nav", "aria-label": copy["settings.navLabel"] }, [
    link("settings-back", "./new-session.html", [svgIcon("arrowLeft"), h("span", { text: copy["settings.back"] })]),
    h("p", { class: "kicker", text: copy["settings.kicker"] }),
    ...settingsNavItems.flatMap((item) => [
      item.dividerBefore ? h("div", { class: "settings-divider", role: "separator" }) : null,
      link(`settings-link${active === item.key ? " is-active" : ""}`, item.href, [svgIcon(item.icon), h("span", { text: item.label })], { "aria-current": active === item.key ? "page" : null })
    ])
  ]);
}

function settingsBody(active) {
  if (active === "add-provider") return providerForm();
  if (active === "models") return modelsSettings();
  if (active === "configuration") return configurationSettings();
  if (active === "plugins") return pluginsSettings();
  if (active === "skills") return skillsSettings();
  if (active === "mcp") return mcpSettings();
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

function pluginsSettings() {
  return h("section", {}, [
    settingsTitle(settingsNavItems[3].label, copy["settings.pluginsSummary"]),
    h("div", { class: "plugin-store", "aria-label": "Plugin catalog" }, [
      h("div", { class: "plugin-toolbar" }, [
        h("label", { class: "plugin-search" }, [
          svgIcon("search"),
          h("span", { class: "sr-only", text: "Search plugins" }),
          h("input", { type: "search", name: "plugin-search", autocomplete: "off", placeholder: "Search plugins" })
        ]),
        h("div", { class: "plugin-filter-wrap" }, [
          h("button", { class: "plugin-filter", type: "button", "aria-expanded": "false", "data-marketplace-menu-trigger": "true" }, [
            h("span", { text: "Built by C4OS" }),
            svgIcon("chevronDown")
          ]),
          h("div", { class: "plugin-filter-menu", hidden: true, "data-marketplace-menu": "true" }, [
            h("button", { type: "button", "data-marketplace-open": "true" }, [h("span", { text: "+ Add Marketplace" })])
          ])
        ])
      ]),
      h("div", { class: "plugin-grid" }, pluginCatalog.map(pluginCard))
    ]),
    pluginConnectDialog(pluginCatalog[0]),
    marketplaceDialog()
  ]);
}

function skillsSettings() {
  return h("section", {}, [
    settingsTitle(settingsNavItems[4].label, "Manage installed skills and per-project availability."),
    h("div", { class: "skills-panel", "aria-label": "Skills" }, [
      h("label", { class: "skills-search" }, [
        svgIcon("search"),
        h("span", { class: "sr-only", text: "Search skills" }),
        h("input", { type: "search", name: "skill-search", autocomplete: "off", placeholder: "Search skills" })
      ]),
      h("div", { class: "skills-list" }, skillCatalog.map(skillRow))
    ]),
    skillDetailDialog(skillCatalog[6])
  ]);
}

function mcpSettings() {
  return h("section", {}, [
    h("header", { class: "mcp-head" }, [
      h("div", {}, [
        h("h1", { text: "MCP Servers" }),
        h("p", {}, [
          "Connect external tools and data sources. ",
          h("button", { class: "marketplace-inline-link", type: "button" }, ["Learn more."])
        ])
      ]),
      h("button", { class: "button primary", type: "button", "data-mcp-add": "true" }, [svgIcon("add"), h("span", { text: "Add server" })])
    ]),
    h("section", { class: "mcp-section", "aria-labelledby": "mcp-servers-title" }, [
      h("h2", { id: "mcp-servers-title", text: "Servers" }),
      h("div", { class: "mcp-list" }, mcpServers.map((name) => mcpRow(name)))
    ]),
    mcpDialog()
  ]);
}

function mcpRow(name) {
  return h("article", { class: "mcp-row" }, [
    h("strong", { text: name }),
    iconButton(`${name} settings`, "settings", "mcp-row-settings"),
    h("button", { class: "skill-switch is-on", type: "button", "aria-label": `${name} enabled` }, [h("span")])
  ]);
}

function mcpDialog() {
  return h("div", { class: "plugin-modal mcp-modal", hidden: true, "data-mcp-modal": "true" }, [
    h("div", { class: "mcp-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "mcp-dialog-title" }, [
      iconButton("Close MCP server form", "x", "plugin-modal-close", { "data-mcp-close": "true" }),
      h("header", { class: "mcp-dialog-head" }, [
        h("h2", { id: "mcp-dialog-title", text: "Connect to a custom MCP" }),
        h("button", { class: "marketplace-inline-link", type: "button" }, ["Docs"])
      ]),
      h("form", { class: "mcp-form" }, [
        mcpFormField("Name", "mcp-name", "MCP server name"),
        h("div", { class: "mcp-segment", role: "tablist", "aria-label": "MCP transport" }, [
          h("button", { class: "is-active", type: "button", role: "tab", "aria-selected": "true", "data-mcp-mode": "stdio" }, ["STDIO"]),
          h("button", { type: "button", role: "tab", "aria-selected": "false", "data-mcp-mode": "http" }, ["Streamable HTTP"])
        ]),
        h("div", { class: "mcp-mode-fields", "data-mcp-fields": "stdio" }, [
          mcpFormField("Command to launch", "mcp-command", "openai-dev-mcp serve-sqlite"),
          mcpInputGroup("Arguments", [[""]], "Add argument"),
          mcpInputGroup("Environment variables", [["Key", "Value"]], "Add environment variable", true),
          mcpInputGroup("Environment variable passthrough", [[""]], "Add variable"),
          mcpFormField("Working directory", "mcp-working-directory", "~/code")
        ]),
        h("div", { class: "mcp-mode-fields", hidden: true, "data-mcp-fields": "http" }, [
          mcpFormField("URL", "mcp-url", "https://mcp.example.com/mcp"),
          mcpFormField("Bearer token env var", "mcp-bearer", "MCP_BEARER_TOKEN"),
          mcpInputGroup("Headers", [["Key", "Value"]], "Add header", true),
          mcpInputGroup("Headers from environment variables", [["Key", "Value"]], "Add variable", true)
        ]),
        h("footer", { class: "mcp-actions" }, [
          h("button", { class: "button secondary", type: "button", "data-mcp-close": "true" }, ["Cancel"]),
          h("button", { class: "button primary", type: "button", "data-mcp-close": "true" }, ["Save"])
        ])
      ])
    ])
  ]);
}

function mcpFormField(label, id, placeholder) {
  return h("label", { class: "mcp-field", for: id }, [
    h("span", { text: label }),
    h("input", { id, name: id, type: "text", placeholder, autocomplete: "off", spellcheck: "false" })
  ]);
}

function mcpInputGroup(title, placeholders, addLabel, twoColumn = false) {
  return h("section", { class: "mcp-group" }, [
    h("h3", { text: title }),
    h("div", { class: twoColumn ? "mcp-pair-row" : "mcp-single-row" }, [
      ...placeholders[0].map((placeholder, index) =>
        h("input", { type: "text", placeholder, "aria-label": `${title} ${index + 1}`, autocomplete: "off", spellcheck: "false" })
      ),
      iconButton(`Remove ${title}`, "trash", "mcp-trash")
    ]),
    h("button", { class: "mcp-add-line", type: "button" }, [h("span", { text: `+ ${addLabel}` })])
  ]);
}

function skillRow(skill) {
  return h("article", { class: "skill-row" }, [
    h("button", { class: "skill-open", type: "button", "data-skill-open": skill.name }, [
      h("span", { class: "skill-mark", "aria-hidden": "true" }, [svgIcon("file")]),
      h("span", { class: "skill-copy" }, [
        h("strong", { text: skill.name }),
        h("span", { text: skill.description })
      ])
    ]),
    h("span", { class: "skill-scope", text: skill.scope }),
    h("button", { class: "skill-switch is-on", type: "button", "aria-label": `${skill.name} enabled` }, [h("span")])
  ]);
}

function skillDetailDialog(skill) {
  return h("div", { class: "plugin-modal skill-modal", hidden: true, "data-skill-modal": "true" }, [
    h("div", { class: "skill-modal-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "skill-modal-title" }, [
      iconButton("Close skill details", "x", "plugin-modal-close", { "data-skill-close": "true" }),
      h("header", { class: "skill-modal-head" }, [
        h("span", { class: "skill-modal-icon", "aria-hidden": "true" }, [svgIcon("file")]),
        h("div", { class: "skill-modal-title-row" }, [
          h("h2", { id: "skill-modal-title", "data-skill-title": "true", text: skill.name }),
          h("span", { text: "Skill" })
        ]),
        h("p", { "data-skill-summary": "true", text: skill.description })
      ]),
      h("div", { class: "skill-modal-controls" }, [
        h("button", { class: "skill-switch is-on", type: "button", "aria-label": "Skill enabled" }, [h("span")]),
        iconButton("More skill actions", "settings", "skill-more")
      ]),
      h("div", { class: "skill-detail-copy", "data-skill-detail": "true" }, renderSkillDetail(skill.detail)),
      h("footer", { class: "skill-modal-actions" }, [
        h("button", { class: "button secondary danger-action", type: "button", "data-skill-close": "true" }, ["Uninstall"]),
        h("button", { class: "button primary", type: "button", "data-skill-close": "true" }, ["Try in chat"])
      ])
    ])
  ]);
}

function renderSkillDetail(detail) {
  return detail.split("\n").map((line) => {
    if (!line.trim()) return h("div", { class: "skill-detail-gap" });
    if (line.startsWith("- ")) return h("p", { class: "skill-detail-bullet", text: line });
    if (!line.includes(".") && line.length < 40) return h("h3", { text: line });
    return h("p", { text: line });
  });
}

function pluginCard(plugin) {
  return h("article", { class: "plugin-card" }, [
    h("div", { class: `plugin-logo plugin-logo-${plugin.tone}`, "aria-hidden": "true" }, [h("span", { text: plugin.logo })]),
    h("div", { class: "plugin-copy" }, [
      h("strong", { text: plugin.name }),
      h("span", { text: plugin.description })
    ]),
    iconButton(`Add ${plugin.name}`, "add", "plugin-add-button", { "data-plugin-connect": plugin.name })
  ]);
}

function pluginConnectDialog(plugin) {
  return h("div", { class: "plugin-modal", hidden: true, "data-plugin-modal": "connect" }, [
    h("div", { class: "plugin-modal-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "plugin-connect-title" }, [
      iconButton("Close plugin connection", "x", "plugin-modal-close", { "data-plugin-close": "true" }),
      h("div", { class: "plugin-modal-brand" }, [
        h("div", { class: "plugin-modal-app plugin-modal-source" }, [h("span", { text: "AI" })]),
        h("div", { class: "plugin-modal-dots", "aria-hidden": "true" }, [h("span"), h("span"), h("span")]),
        h("div", { class: `plugin-logo plugin-modal-app plugin-logo-${plugin.tone}`, "data-plugin-modal-logo": "true", "aria-hidden": "true" }, [h("span", { text: plugin.logo })])
      ]),
      h("h2", { id: "plugin-connect-title", "data-plugin-modal-title": "true", text: `Connect ${plugin.name}` }),
      h("div", { class: "plugin-modal-status" }, [svgIcon("check"), h("span", { text: "Approved by your admin" })]),
      h("div", { class: "plugin-modal-copy" }, [
        modalCopyBlock("You're in control", "C4OS respects your project preferences and limits this plugin to permissions you explicitly set."),
        modalCopyBlock("Plugins may introduce elevated risk", "Plugins can access scoped project context. Review permissions before connecting a workflow to your workspace."),
        modalCopyBlock("Data shared with this plugin", "Adding this plugin allows access to basic workspace context and recent intent needed to respond to your request.")
      ]),
      h("button", { class: "plugin-modal-continue", type: "button", "data-plugin-continue": "true" }, [
        h("span", { "data-plugin-modal-action": "true", text: `Continue to ${plugin.name}` }),
        svgIcon("external")
      ]),
      h("button", { class: "plugin-modal-advanced", type: "button" }, ["Advanced settings"])
    ])
  ]);
}

function modalCopyBlock(title, text) {
  return h("section", {}, [h("h3", { text: title }), h("p", { text })]);
}

function marketplaceDialog() {
  return h("div", { class: "plugin-modal marketplace-modal", hidden: true, "data-marketplace-modal": "true" }, [
    h("div", { class: "marketplace-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "marketplace-title" }, [
      iconButton("Close marketplace dialog", "x", "plugin-modal-close", { "data-marketplace-close": "true" }),
      h("header", { class: "marketplace-head" }, [
        h("h2", { id: "marketplace-title", text: "Add plugin marketplace" }),
        h("p", {}, [
          "Add from a GitHub repo, Git URL, or local folder. ",
          h("button", { class: "marketplace-inline-link", type: "button" }, ["Learn more"])
        ])
      ]),
      h("div", { class: "marketplace-form" }, [
        marketplaceField("Source", "marketplace-source", "openai/plugins or git@github.com:org/repo.git"),
        marketplaceField("Git ref", "marketplace-ref", "main"),
        marketplaceField("Sparse paths", "marketplace-paths", "plugins/codex", true)
      ]),
      h("div", { class: "marketplace-actions" }, [
        h("button", { class: "button secondary", type: "button", "data-marketplace-close": "true" }, ["Cancel"]),
        h("button", { class: "button primary", type: "button", "data-marketplace-close": "true" }, ["Add marketplace"])
      ])
    ])
  ]);
}

function marketplaceField(label, id, placeholder, multiline = false) {
  const control = multiline
    ? h("textarea", { id, name: id, rows: "3", placeholder, autocomplete: "off", spellcheck: "false" })
    : h("input", { id, name: id, type: "text", placeholder, autocomplete: "off", spellcheck: "false" });
  return h("div", { class: "marketplace-field" }, [h("label", { for: id, text: label }), control]);
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

function bindPluginConnect() {
  const modal = document.querySelector("[data-plugin-modal='connect']");
  if (!modal) return;
  const title = modal.querySelector("[data-plugin-modal-title]");
  const action = modal.querySelector("[data-plugin-modal-action]");
  const logo = modal.querySelector("[data-plugin-modal-logo]");
  const logoText = logo?.querySelector("span");
  let lastTrigger = null;

  const closeModal = () => {
    modal.hidden = true;
    document.removeEventListener("keydown", onKeydown);
    lastTrigger?.focus();
  };

  const onKeydown = (event) => {
    if (event.key === "Escape") closeModal();
  };

  document.querySelectorAll("[data-plugin-connect]").forEach((trigger) => {
    trigger.addEventListener("click", () => {
      const plugin = pluginCatalog.find((item) => item.name === trigger.dataset.pluginConnect) || pluginCatalog[0];
      lastTrigger = trigger;
      title.textContent = `Connect ${plugin.name}`;
      action.textContent = `Continue to ${plugin.name}`;
      logo.className = `plugin-logo plugin-modal-app plugin-logo-${plugin.tone}`;
      logoText.textContent = plugin.logo;
      modal.hidden = false;
      modal.querySelector("[data-plugin-close]")?.focus();
      document.addEventListener("keydown", onKeydown);
    });
  });

  modal.querySelectorAll("[data-plugin-close], [data-plugin-continue]").forEach((control) => {
    control.addEventListener("click", closeModal);
  });

  modal.addEventListener("click", (event) => {
    if (event.target === modal) closeModal();
  });
}

function bindMarketplaceControls() {
  const menu = document.querySelector("[data-marketplace-menu]");
  const trigger = document.querySelector("[data-marketplace-menu-trigger]");
  const modal = document.querySelector("[data-marketplace-modal]");
  if (!menu || !trigger || !modal) return;
  let lastTrigger = null;

  const closeMenu = () => {
    menu.hidden = true;
    trigger.setAttribute("aria-expanded", "false");
  };

  const closeModal = () => {
    modal.hidden = true;
    document.removeEventListener("keydown", onKeydown);
    lastTrigger?.focus();
  };

  const onKeydown = (event) => {
    if (event.key === "Escape") {
      closeMenu();
      if (!modal.hidden) closeModal();
    }
  };

  trigger.addEventListener("click", () => {
    const expanded = menu.hidden;
    menu.hidden = !expanded;
    trigger.setAttribute("aria-expanded", String(expanded));
    document.addEventListener("keydown", onKeydown);
  });

  document.querySelector("[data-marketplace-open]")?.addEventListener("click", () => {
    lastTrigger = trigger;
    closeMenu();
    modal.hidden = false;
    modal.querySelector("[data-marketplace-close]")?.focus();
    document.addEventListener("keydown", onKeydown);
  });

  modal.querySelectorAll("[data-marketplace-close]").forEach((control) => {
    control.addEventListener("click", closeModal);
  });

  modal.addEventListener("click", (event) => {
    if (event.target === modal) closeModal();
  });

  document.addEventListener("click", (event) => {
    if (!event.target.closest(".plugin-filter-wrap")) closeMenu();
  });
}

function bindSkillDetails() {
  const modal = document.querySelector("[data-skill-modal]");
  if (!modal) return;
  const title = modal.querySelector("[data-skill-title]");
  const summary = modal.querySelector("[data-skill-summary]");
  const detail = modal.querySelector("[data-skill-detail]");
  let lastTrigger = null;

  const closeModal = () => {
    modal.hidden = true;
    document.removeEventListener("keydown", onKeydown);
    lastTrigger?.focus();
  };

  const onKeydown = (event) => {
    if (event.key === "Escape") closeModal();
  };

  document.querySelectorAll("[data-skill-open]").forEach((trigger) => {
    trigger.addEventListener("click", () => {
      const skill = skillCatalog.find((item) => item.name === trigger.dataset.skillOpen) || skillCatalog[0];
      lastTrigger = trigger;
      title.textContent = skill.name;
      summary.textContent = skill.description;
      detail.replaceChildren(...renderSkillDetail(skill.detail));
      modal.hidden = false;
      modal.querySelector("[data-skill-close]")?.focus();
      document.addEventListener("keydown", onKeydown);
    });
  });

  modal.querySelectorAll("[data-skill-close]").forEach((control) => {
    control.addEventListener("click", closeModal);
  });

  modal.addEventListener("click", (event) => {
    if (event.target === modal) closeModal();
  });
}

function bindMcpControls() {
  const modal = document.querySelector("[data-mcp-modal]");
  const trigger = document.querySelector("[data-mcp-add]");
  if (!modal || !trigger) return;
  let lastTrigger = null;

  const closeModal = () => {
    modal.hidden = true;
    document.removeEventListener("keydown", onKeydown);
    lastTrigger?.focus();
  };

  const onKeydown = (event) => {
    if (event.key === "Escape") closeModal();
  };

  trigger.addEventListener("click", () => {
    lastTrigger = trigger;
    modal.hidden = false;
    modal.querySelector("[data-mcp-close]")?.focus();
    document.addEventListener("keydown", onKeydown);
  });

  modal.querySelectorAll("[data-mcp-close]").forEach((control) => {
    control.addEventListener("click", closeModal);
  });

  modal.querySelectorAll("[data-mcp-mode]").forEach((control) => {
    control.addEventListener("click", () => {
      const mode = control.dataset.mcpMode;
      modal.querySelectorAll("[data-mcp-mode]").forEach((item) => {
        const active = item === control;
        item.classList.toggle("is-active", active);
        item.setAttribute("aria-selected", String(active));
      });
      modal.querySelectorAll("[data-mcp-fields]").forEach((fields) => {
        fields.hidden = fields.dataset.mcpFields !== mode;
      });
    });
  });

  modal.addEventListener("click", (event) => {
    if (event.target === modal) closeModal();
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
      const shellWidth = shell.getBoundingClientRect().width;
      const leftWidth = shell.querySelector(".sidebar")?.getBoundingClientRect().width || 0;
      const rightWidth = shell.querySelector(".tool-panel")?.getBoundingClientRect().width || 0;
      const handleWidth = Array.from(shell.querySelectorAll(".resize-handle"))
        .filter((node) => getComputedStyle(node).display !== "none")
        .reduce((total, node) => total + node.getBoundingClientRect().width, 0);
      const minWorkbench = parseFloat(getComputedStyle(shell).getPropertyValue("--workbench-min")) || 520;
      const availableMax = side === "left"
        ? shellWidth - rightWidth - handleWidth - minWorkbench
        : shellWidth - leftWidth - handleWidth - minWorkbench;
      const configured = side === "left" ? { min: 220, max: 460, prop: "--left-panel" } : { min: 280, max: 560, prop: "--right-panel" };
      const limits = { ...configured, max: Math.max(configured.min, Math.min(configured.max, availableMax)) };

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

function bindTerminalResize() {
  document.querySelectorAll("[data-resize-stack='terminal']").forEach((handle) => {
    handle.addEventListener("pointerdown", (event) => {
      const panel = handle.closest(".terminal-tool");
      const bottom = panel.querySelector(".terminal-bottom");
      const startY = event.clientY;
      const startHeight = bottom.getBoundingClientRect().height;
      const limits = { min: 120, max: 420 };

      handle.setPointerCapture(event.pointerId);
      panel.classList.add("is-resizing");

      const onPointerMove = (moveEvent) => {
        const delta = startY - moveEvent.clientY;
        const height = Math.min(limits.max, Math.max(limits.min, startHeight + delta));
        panel.style.setProperty("--terminal-bottom", `${Math.round(height)}px`);
      };

      const onPointerUp = () => {
        panel.classList.remove("is-resizing");
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

function bindComposerControls() {
  document.querySelectorAll(".composer").forEach((composerNode) => {
    const input = composerNode.querySelector(".attachment-input");
    const preview = composerNode.querySelector("[data-attachments]");
    const attachButton = composerNode.querySelector("[data-attach-button]");
    let attachments = [];

    attachButton?.addEventListener("click", () => input.click());
    input?.addEventListener("change", () => {
      const files = Array.from(input.files || []);
      attachments = attachments.concat(files);
      input.value = "";
      renderAttachments();
    });

    const renderAttachments = () => {
      preview.hidden = attachments.length === 0;
      preview.replaceChildren(...attachments.map((file, index) =>
        h("span", { class: "attachment-chip" }, [
          svgIcon("file"),
          h("span", { text: file.name }),
          h("small", { text: formatBytes(file.size) }),
          h("button", { class: "attachment-remove", type: "button", "aria-label": `${copy["composer.removeAttachment"]}: ${file.name}`, "data-remove-attachment": String(index) }, ["x"])
        ])
      ));
      preview.querySelectorAll("[data-remove-attachment]").forEach((remove) => {
        remove.addEventListener("click", () => {
          attachments.splice(Number(remove.dataset.removeAttachment), 1);
          renderAttachments();
        });
      });
    };

    composerNode.querySelectorAll("[data-popover-trigger]").forEach((trigger) => {
      trigger.addEventListener("click", () => {
        const target = composerNode.querySelector(`[data-popover='${trigger.dataset.popoverTrigger}']`);
        composerNode.querySelectorAll("[data-popover]").forEach((popover) => {
          if (popover !== target) popover.hidden = true;
        });
        if (target) target.hidden = !target.hidden;
      });
    });

    composerNode.querySelectorAll("[data-popover-option]").forEach((option) => {
      option.addEventListener("click", () => {
        const popover = option.closest("[data-popover]");
        const kind = popover.dataset.popover;
        const label = option.textContent.trim();
        if (kind === "approval") {
          const approval = composerNode.querySelector("[data-popover-trigger='approval'] span");
          if (approval) approval.textContent = label;
        }
        if (kind === "branch" && option.dataset.popoverOption !== "create") {
          const branch = composerNode.querySelector("[data-popover-trigger='branch'] span");
          if (branch) branch.textContent = label;
        }
        popover.hidden = true;
      });
    });
  });
}

function formatBytes(size) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${Math.round(size / 1024)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

if (route === "hub") renderHub();
else if (route === "app-start") renderStart();
else if (route.startsWith("settings")) renderSettings(route);
else renderShell(route);
