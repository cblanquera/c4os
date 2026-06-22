import {
  mcpServers,
  models,
  pluginCatalog,
  pluginMarketplaces,
  projects,
  providers,
  routes,
  settingsItems,
  skillCatalog,
  workspace
} from "./data.js";
import { h, icon } from "./dom.js";
import { bindInteractions } from "./interactions.js";

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
  document.body.dataset.route = route;
  if (route === "app-start") app.replaceChildren(renderStart());
  else if (route.startsWith("settings")) app.replaceChildren(renderSettings(route));
  else app.replaceChildren(renderShell(route));
  bindInteractions(render, pluginInitials);
}

//--------------------------------------------------------------------//
// Start and workspace shell routes
//--------------------------------------------------------------------//

/**
 * Render the trusted-folder entry screen.
 */
function renderStart() {
  return h("main", { id: "main", class: "start-view", "data-screen": "app-start", tabindex: "-1" }, [
    h("section", { class: "start-copy" }, [
      h("p", { class: "kicker", text: "No trusted project folder" }),
      h("h1", { text: "Open a folder to start working" }),
      h("p", { class: "lead", text: "Local files, instructions, skills, runtime policy, approvals, and workspace state stay unavailable until a trusted project folder scopes the session." }),
      h("div", { class: "action-row" }, [
        button("Open Folder", "button primary"),
        button("Clone Repository"),
        button("Open Workspace File")
      ])
    ]),
    h("section", { class: "recent-panel", "aria-labelledby": "recent-title" }, [
      h("h2", { id: "recent-title", text: "Recent Folder-Backed Workspaces" }),
      ...["Project Alpha", "Agent Lab", "Docs Workbench"].map((name, index) =>
        link("new-session", "workspace-row", [
          h("strong", { text: name }),
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
  const tool = route === "file-explorer" || route === "file-editor" ? "files" : route === "terminal" ? "terminal" : "browser";
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

/**
 * Render project/session navigation for the left shell panel.
 */
function renderSidebar(chat) {
  return h("aside", { class: "sidebar", "aria-label": "Projects" }, [
    h("label", { class: "search-field" }, [icon("search"), h("span", { class: "sr-only", text: "Search projects" }), h("input", { type: "search", placeholder: "Search projects" })]),
    h("div", { class: "nav-section" }, [
      h("div", { class: "section-head" }, [h("span", { text: "Projects" }), iconButton("Add Project", "add")]),
      ...projects.flatMap((project) => [
        link("new-session", `project-row${project.name === workspace.project && !chat ? " is-active" : ""}`, [icon("folder"), h("span", { text: project.name }), h("span", { class: "row-tools" }, [icon("pencil"), icon("trash")])]),
        ...project.sessions.map((session) => link("chat-session", `session-row${chat && session === workspace.session ? " is-active" : ""}`, [h("span", { text: session })]))
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
    h("h1", { text: "What should we build in this project?" }),
    composer("Do anything", { popover })
  ]);
}

/**
 * Render the shared prompt composer surface.
 */
function composer(placeholder, options = {}) {
  const readonly = options.readonlyContext;
  return h("section", { class: "composer", "aria-label": "Prompt composer" }, [
    h("input", { class: "attachment-input", type: "file", multiple: true, tabindex: "-1", "aria-hidden": "true" }),
    h("div", { class: "attachment-preview", "data-attachments": "", hidden: true, "aria-label": "Attached files" }),
    h("div", { class: "prompt-box", role: "textbox", "aria-label": "Prompt", "aria-multiline": "true", contenteditable: "true", "data-placeholder": placeholder }),
    h("div", { class: "composer-controls" }, [
      iconButton("Attach File", "paperclip", "icon-button", { "data-attach-button": "" }),
      h("button", { class: "chip", type: "button", "aria-label": "Approval policy: Approve for me", "data-popover-trigger": "approval" }, [icon("shield"), h("span", { text: "Approve for me" })]),
      h("span", { class: "spacer" }),
      iconButton("Use Microphone", "mic"),
      iconButton("Send Prompt", "send", "icon-button send-button")
    ]),
    h("div", { class: "context-strip" }, [
      readonly ? h("span", { class: "chip readonly-chip", "aria-label": "Branch locked for this chat" }, [icon("gitBranch"), h("span", { text: workspace.branch })]) : h("button", { class: "chip", type: "button", "aria-label": `Branch: ${workspace.branch}`, "data-popover-trigger": "branch" }, [icon("gitBranch"), h("span", { text: workspace.branch })]),
      h("span", { class: "spacer" }),
      readonly ? h("span", { class: "chip readonly-chip", "aria-label": "Model locked for this chat" }, [icon("bot"), h("span", { text: workspace.model })]) : link("models-popover", "chip", [icon("bot"), h("span", { text: workspace.model })])
    ]),
    composerPopover("approval", ["Ask for approval", "Approve for me"]),
    readonly ? null : composerPopover("branch", ["main", "feature/trust-shell", "+ Create branch"]),
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
  return h("main", { class: "thread-view" }, [
    h("div", { class: "thread-list", "aria-label": "Session messages" }, [
      h("article", { class: "message user" }, [h("p", { text: "Locate the desktop integration path and confirm the trusted project state." })]),
      h("article", { class: "message agent has-actions" }, [
        h("div", { class: "message-actions" }, [iconButton("Collapse message", "chevronDown")]),
        h("p", { text: "The shell boundary is visible. The next step is to connect state without changing the r04 surface." }),
        h("div", { class: "message-extra" }, [h("p", { text: "This fixture is frontend-local and does not imply backend, filesystem, terminal, browser, approval, or persistence behavior." })]),
        h("button", { class: "text-button", type: "button", "data-toggle": "message", "aria-expanded": "false" }, ["Show More"])
      ]),
      activityCard("Tool Call", "Read project instructions", "Completed"),
      activityCard("Agent Run", "Sensitive action waiting", null, "Review Approval")
    ]),
    h("div", { class: "composer-dock" }, [composer("Ask for follow-up changes", { readonlyContext: true })])
  ]);
}

/**
 * Render an agent activity card in the transcript.
 */
function activityCard(title, label, status, action) {
  return h("article", { class: "activity-card" }, [
    h("header", {}, [h("strong", { text: title }), iconButton(`Collapse message: ${title}`, "chevronDown")]),
    h("div", { class: "activity-row" }, [
      h("span", { text: label }),
      action ? button(action) : h("span", { class: "status-pill", text: status })
    ])
  ]);
}

/**
 * Render the right-side Browser, Files, or Terminal tool panel.
 */
function renderToolPanel(active, route) {
  const tabs = [
    ["Browser", "browser", "new-session", "globe"],
    ["Files", "files", "file-explorer", "file"],
    ["Terminal", "terminal", "terminal", "terminal"]
  ];
  return h("aside", { class: "tool-panel", "aria-label": "Workspace tools" }, [
    h("nav", { class: "tool-tabs", "aria-label": "Workspace tool tabs" }, tabs.map(([label, key, target, iconName]) =>
      link(target, `tool-tab${active === key ? " is-active" : ""}`, [icon(iconName), h("span", { text: label })])
    )),
    active === "files" ? filesTool(route === "file-editor") : active === "terminal" ? terminalTool() : browserTool()
  ]);
}

/**
 * Render the browser preview fixture.
 */
function browserTool() {
  return h("section", { class: "tool-body" }, [
    h("div", { class: "address-bar" }, [icon("globe"), h("span", { text: "http://127.0.0.1:13000" })]),
    h("div", { class: "preview-surface" }, [h("h2", { text: "Rendered page mock" }), h("p", { text: "Single browser surface; tabs are out of scope. This is frontend-local fixture state." })])
  ]);
}

/**
 * Render either the file list or lightweight editor fixture.
 */
function filesTool(editor) {
  if (editor) {
    return h("section", { class: "tool-body editor-tool" }, [
      h("nav", { class: "breadcrumbs", "aria-label": "File breadcrumbs" }, [link("file-explorer", "", ["Project Alpha"]), h("span", { text: ">" }), link("file-explorer", "", ["frontend"]), h("span", { text: ">" }), h("span", { text: "main.js" })]),
      h("div", { class: "code-pane", tabindex: "0" }, ["import { startWorkspace } from './runtime';", "", "const trustedRoot = true;", "", "startWorkspace({ trustedRoot });"].map((line, index) =>
        h("div", { class: "code-line" }, [
          h("span", { class: "line-number", text: String(index + 1) }),
          h("code", { "aria-label": `Line ${index + 1} code`, class: "line-code", contenteditable: "true", spellcheck: "false", text: line })
        ])
      ))
    ]);
  }
  return h("section", { class: "tool-body file-tool" }, [
    h("h2", { text: workspace.project }),
    ...[
      ["backend", "folder", "file-explorer"],
      ["frontend", "folder", "file-explorer"],
      ["main.js", "file", "file-editor"],
      ["index.html", "file", "file-editor"],
      ["tests", "folder", "file-explorer"]
    ].map(([name, iconName, target]) => link(target, `file-row${iconName === "file" ? " is-file" : ""}${name === "main.js" ? " is-active" : ""}`, [icon(iconName), h("span", { text: name })]))
  ]);
}

/**
 * Render the terminal fixture and its resizable result panel.
 */
function terminalTool() {
  return h("section", { class: "tool-body terminal-tool" }, [
    h("pre", { class: "terminal-output", text: "$ npm run dev\nready in 614ms\nlocal preview available at 127.0.0.1:3000" }),
    h("div", { class: "vertical-resize-handle", role: "separator", tabindex: "0", "aria-label": "Resize terminal results panel", "aria-orientation": "horizontal", "data-resize-stack": "terminal" }),
    h("div", { class: "terminal-bottom" }, [h("strong", { text: "AI command preview/results" }), h("p", { text: "Read-only frontend fixture panel. No command execution is implied." })])
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
render();
