const defaultRoute = "shell-foundation";
const app = document.querySelector("#app");

const icons = {
  arrowLeft: "M19 12H5m7 7-7-7 7-7",
  bot: "M12 8V4H8m0 4h8a4 4 0 0 1 4 4v4a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4v-4a4 4 0 0 1 4-4Zm1 5v2m6-2v2M2 14h2m16 0h2",
  bug: "m8 2 1.88 1.88M14.12 3.88 16 2M9 7.13v-1a3 3 0 0 1 6 0v1M12 20c-3.3 0-6-2.7-6-6v-3a6 6 0 0 1 12 0v3c0 3.3-2.7 6-6 6ZM4 13H2m20 0h-2M6.2 18 4.8 19.4m14.4 0L17.8 18M6.2 8 4.8 6.6m14.4 0L17.8 8",
  check: "m5 12 4 4L19 6",
  chevronDown: "m6 9 6 6 6-6",
  circleAlert: "M12 9v4m0 4h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z",
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Zm0 0v6h6",
  folder: "M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z",
  globe: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20Zm0-20a15 15 0 0 1 0 20m0-20a15 15 0 0 0 0 20M2 12h20",
  gitBranch: "M6 3v12m0 6a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm12-12a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm0 0a9 9 0 0 1-9 9",
  key: "M7 14a5 5 0 1 1 3.5-8.5A5 5 0 0 1 7 14Zm7-4 7-7m-3 3 3 3m-6 0 3 3",
  messages: "M21 15a4 4 0 0 1-4 4H7l-4 4V7a4 4 0 0 1 4-4h10a4 4 0 0 1 4 4Z",
  mic: "M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Zm7 8v2a7 7 0 0 1-14 0v-2m7 9v3",
  panelLeft: "M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Zm6-2v18",
  panelRight: "M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Zm12-2v18",
  paperclip: "m21 11-9 9a6 6 0 0 1-8-8l9-9a4 4 0 0 1 6 6l-9 9a2 2 0 0 1-3-3l8-8",
  pencil: "M21 6 7 20H3v-4L17 2a3 3 0 0 1 4 4Z",
  plug: "M9 2v6m6-6v6m3 0v5a6 6 0 0 1-12 0V8Zm-6 14v-5",
  plus: "M12 5v14M5 12h14",
  rotate: "M21 12a9 9 0 1 1-3-6.7M21 3v6h-6",
  search: "m21 21-4-4m2-6a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z",
  send: "M3 4 21 12 3 20l3-8Zm3 8h15",
  settings: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0-6v3m0 14v3M4.9 4.9 7 7m10 10 2.1 2.1M2 12h3m14 0h3M4.9 19.1 7 17m10-10 2.1-2.1",
  shield: "M12 3c3 2 5 3 8 3v6c0 5-3 8-8 10-5-2-8-5-8-10V6c3 0 5-1 8-3Z",
  terminal: "m4 17 6-5-6-5m8 10h8",
  x: "M18 6 6 18M6 6l12 12"
};

const plugins = {
  chats: { id: "chats", label: "Chats", icon: "messages", side: "left" },
  files: { id: "files", label: "Files", icon: "folder", side: "left" },
  browser: { id: "browser", label: "Browser", icon: "globe", side: "right" },
  terminal: { id: "terminal", label: "Terminal", icon: "terminal", side: "right" },
  debug: { id: "debug", label: "Debug", icon: "bug", side: "right" }
};

const routeStates = {
  "shell-foundation": { title: "Locate Tauri integration", subtitle: "c4os2", left: null, right: null },
  "same-side-replacement": { title: "Locate Tauri integration", subtitle: "c4os2", left: "files", right: "terminal" },
  "per-chat-restore": { title: "Draft wireframes", subtitle: "c4os2", left: "chats", right: "browser" },
  "resize-collision": { title: "Build shell state", subtitle: "c4os2", left: "files", right: "terminal", collision: true },
  "hidden-activity": { title: "Review hidden updates", subtitle: "c4os2", left: null, right: null, activity: "browser" },
  debug: { title: "Debug run history", subtitle: "c4os2", left: "chats", right: "debug" },
  "repair-state": { title: "Settings", subtitle: "Plugin repair", settings: "plugins" },
  settings: { title: "Settings", subtitle: "Plugins", settings: "plugins" },
  "settings-plugins": { title: "Settings", subtitle: "Plugins", settings: "plugins" },
  "settings-plugin-detail": { title: "Settings", subtitle: "Plugin detail", settings: "plugin-detail" },
  "settings-plugin-marketplace": { title: "Settings", subtitle: "Plugin marketplace", settings: "plugin-marketplace" },
  "settings-plugin-states": { title: "Settings", subtitle: "Plugin states", settings: "plugin-states" },
  "settings-plugin-uninstall": { title: "Settings", subtitle: "Uninstall plugin", settings: "plugin-uninstall" },
  "settings-configuration": { title: "Settings", subtitle: "Configuration", settings: "configuration" },
  "settings-config-error": { title: "Settings", subtitle: "Config parse error", settings: "config-error" },
  "settings-skills": { title: "Settings", subtitle: "Skills", settings: "skills" },
  "settings-skill-detail": { title: "Settings", subtitle: "Skill detail", settings: "skill-detail" },
  "settings-skill-customize": { title: "Settings", subtitle: "Customize skill", settings: "skill-customize" },
  "settings-skill-invalid": { title: "Settings", subtitle: "Invalid skill", settings: "skill-invalid" },
  coverage: { title: "Wireframe Coverage", subtitle: "Specs 03, 04, and 11", coverage: true }
};

const chats = [
  { title: "Locate Tauri integration", active: true },
  { title: "Draft wireframes", active: false },
  { title: "Plugin panel restore", active: false }
];

const pluginCatalog = [
  { name: "File system", description: "Activates Workspaces, Projects, and chat items per project.", logo: "FS", tone: "filesystem" },
  { name: "File editor", description: "Requires FS. Activates file explorer and editor.", logo: "IDE", tone: "editor" },
  { name: "Terminal", description: "Activates user terminal per chat session.", logo: "CLI", tone: "terminal" },
  { name: "Chat Debug", description: "Former agent terminal for command and tool-call history.", logo: "DBG", tone: "debug" },
  { name: "Browser", description: "Activates in-browser preview per chat session.", logo: "BR", tone: "browser" }
];

const skillCatalog = [
  { name: "Ab Testing", scope: "Personal", description: "When the user wants to plan, design, or implement an A/B test or experiment, or optimize a test plan.", detail: "Use this skill when a product, page, onboarding step, pricing flow, or campaign needs a clear experiment design. It helps define hypotheses, variants, success metrics, guardrails, and rollout decisions." },
  { name: "Ad Creative", scope: "Personal", description: "When the user wants to generate, iterate, or scale ad creative - headlines, hooks, variants, and angles.", detail: "Use this skill to turn campaign goals into compact creative directions, message variants, visual concepts, and testable ad copy." },
  { name: "Ads", scope: "Personal", description: "When the user wants help with paid advertising campaigns on Google Ads, Meta, or other acquisition channels.", detail: "Use this skill for campaign structure, targeting, bidding, budget allocation, and diagnosing paid acquisition performance." },
  { name: "Ai Seo", scope: "Personal", description: "When the user wants to optimize content for AI search engines, get cited by LLMs, and improve answer visibility.", detail: "Use this skill for answer-engine positioning, citation-worthy content, schema alignment, and source clarity." },
  { name: "Analytics", scope: "Personal", description: "When the user wants to set up, improve, or audit analytics tracking and measurement quality.", detail: "Use this skill to define events, funnels, attribution needs, dashboards, and data quality checks." },
  { name: "Aso", scope: "Personal", description: "When the user wants to audit or optimize an App Store or Google Play listing. Also useful for keyword and conversion work.", detail: "Use this skill for app listing positioning, screenshots, keywords, ratings strategy, and conversion review." },
  { name: "ChrisAI Agents", scope: "Project", description: "Set up project-local .agents workflows", detail: "Use this skill to set up or repair a project-local .agents/ operating surface.\n\nCore Jobs\n- Initialize or repair .agents/AGENTS.md.\n- Generate or repair local .agents/workflows/*.md files.\n- Establish the .agents folder contract without creating placeholder files before needed." }
];

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

function routeFromHash() {
  return String(window.location.hash || "").replace(/^#/, "") || defaultRoute;
}

function go(route) {
  window.location.hash = route;
}

function iconButton(plugin, active, activity) {
  return h("button", {
    class: `icon-button${active ? " is-active" : ""}`,
    type: "button",
    title: plugin.label,
    "aria-label": plugin.label,
    "aria-pressed": active ? "true" : "false",
    "data-plugin": plugin.id
  }, [
    svgIcon(plugin.icon),
    activity ? h("span", { class: "activity-dot", "aria-hidden": "true" }) : null
  ]);
}

function settingsButton(active) {
  return h("button", {
    class: `icon-button${active ? " is-active" : ""}`,
    type: "button",
    title: "Settings",
    "aria-label": "Settings",
    "data-route": "settings"
  }, [svgIcon("settings")]);
}

function headerFor(state) {
  return h("header", { class: "shell-header" }, [
    h("div", { class: "header-icons left", "aria-label": "Left plugin icons" },
      ["chats", "files"].map((id) => iconButton(plugins[id], state.left === id, state.activity === id))
    ),
    h("div", { class: "header-title" }, [
      h("strong", { text: state.title }),
      h("span", { text: state.subtitle })
    ]),
    h("div", { class: "header-icons right", "aria-label": "Right plugin icons" }, [
      ...["browser", "terminal", "debug"].map((id) => iconButton(plugins[id], state.right === id, state.activity === id)),
      settingsButton(Boolean(state.settings))
    ])
  ]);
}

function resizeHandle(side) {
  return h("div", {
    class: `resize-handle ${side}-resizer`,
    role: "separator",
    tabindex: "0",
    "aria-label": side === "left" ? "Resize left panel" : "Resize right panel",
    "aria-orientation": "vertical"
  });
}

function pluginPanel(id) {
  const plugin = plugins[id];
  if (!plugin) return null;
  return h("aside", { class: `plugin-panel ${plugin.side} ${id}`, "aria-label": `${plugin.label} panel` }, [
    h("div", { class: "panel-shell" }, [
      h("div", { class: "panel-body" }, panelBody(id))
    ])
  ]);
}

function panelBody(id) {
  if (id === "chats") {
    return [
      h("label", { class: "chat-search" }, [svgIcon("search"), h("input", { type: "search", placeholder: "Search chats", "aria-label": "Search chats" })]),
      h("div", { class: "panel-list" }, chats.map((chat) =>
        h("a", { class: `panel-row${chat.active ? " is-active" : ""}`, href: "#per-chat-restore" }, [svgIcon("messages"), h("span", { text: chat.title })])
      ))
    ];
  }
  if (id === "files") {
    return [h("div", { class: "file-tree" }, [
      fileRow("backend", "folder"),
      fileRow("frontend", "folder"),
      fileRow("main.js", "file", true),
      fileRow("index.html", "file"),
      fileRow("tests", "folder")
    ])];
  }
  if (id === "browser") {
    return [h("div", { class: "browser-frame" }, [
      h("div", { class: "browser-bar" }, [svgIcon("globe"), h("span", { text: "http://127.0.0.1:13000" })]),
      h("div", { class: "browser-page" }, [h("p", { text: "it's true." })])
    ])];
  }
  if (id === "terminal") {
    return [h("div", { class: "terminal-frame", text: "$ npm run dev\nready in 614ms\nlocal preview available at 127.0.0.1:3000" })];
  }
  return [debugPanel()];
}

function debugPanel() {
  return h("div", { class: "debug-shell" }, [
    h("div", { class: "debug-tabs" }, [
      h("span", { class: "debug-tab is-active", text: "Runs" }),
      h("span", { class: "debug-tab", text: "Tools" }),
      h("span", { class: "debug-tab", text: "Approvals" })
    ]),
    h("div", { class: "debug-terminal" }, [
      h("pre", { text: "Agent command terminal\n\n$ git log -1 && ls -la\ncommit 9b90fc07d24804e264df14106f40334f0fd1df68\nAuthor: Chris Blanquera <chris@incept.asia>\nDate:   Mon Sep 1 16:22:05 2025 +0800\n\n    converting human documents to AI rules\n\ntotal 336\ndrwxr-xr-x  25 cblanquera  staff    800 Jun 22 19:53 .\ndrwxr-xr-x  13 cblanquera  staff    416 Jun 25 19:29 ..\n-rw-r--r--   1 cblanquera  staff   6148 Sep  5  2025 .DS_Store\ndrwxr-xr-x   9 cblanquera  staff    288 Sep  1  2025 .build\ndrwxr-xr-x   3 cblanquera  staff     96 Jun 22 19:53 .c4os" })
    ]),
    h("div", { class: "debug-events" }, [
      debugEvent("terminal.run", "completed", "command: git log -1 && ls -la"),
      debugEvent("tool_call_requested", "recorded", "files.list cwd=."),
      debugEvent("tool_output_delta", "streamed", "336 directory entries"),
      debugEvent("approval_policy", "allowed", "workspace read")
    ])
  ]);
}

function debugEvent(kind, status, detail) {
  return h("div", { class: "debug-event" }, [
    h("strong", { text: kind }),
    h("span", { text: status }),
    h("p", { text: detail })
  ]);
}

function fileRow(name, icon, active = false) {
  return h("div", { class: `file-row${active ? " is-active" : ""}` }, [svgIcon(icon), h("span", { text: name })]);
}

function chatWorkbench(state) {
  if (state.collision) return collisionWorkbench();
  return h("main", { class: "workbench", id: "main", tabindex: "-1" }, [
    h("section", { class: "chat-surface" }, [
      h("div", { class: "thread-list", "aria-label": "Session messages" }, [
        h("article", { class: "message user" }, [
          h("p", { text: "run ls -al and tell me what the last commit was." })
        ]),
        h("div", { class: "run-summary" }, [
          h("button", { class: "text-button", type: "button" }, ["Worked for 1s ", svgIcon("chevronDown")]),
          h("ul", {}, [
            h("li", { text: "C4OS runtime adapter" }),
            h("li", { text: "OpenRouter stream complete" })
          ])
        ]),
        h("article", { class: "message agent" }, [
          h("p", {}, ["I'll run ", h("code", { text: "ls -al" }), " and check the latest Git commit with ", h("code", { text: "git log -1 --oneline" }), "."]),
          h("p", { text: "C4OS-owned session, run, message, and runtime-reference records persisted for this prompt." }),
          h("button", { class: "text-button show-less", type: "button", text: "Show Less" })
        ])
      ]),
      composer()
    ])
  ]);
}

function collisionWorkbench() {
  return h("main", { class: "workbench", id: "main", tabindex: "-1" }, [
    h("section", { class: "empty-workspace" }, [
      h("h1", { text: "What should we build in c4os2?" }),
      h("p", { text: "At the collision boundary, the center pane keeps a 640px minimum. Expanding one side closes the opposite side before clamping." }),
      composer()
    ])
  ]);
}

function composer() {
  return h("form", { class: "composer", "aria-label": "Prompt composer" }, [
    h("div", { class: "prompt-text", role: "textbox", "aria-label": "Prompt", "aria-multiline": "true", contenteditable: "true", text: "Do anything" }),
    h("div", { class: "composer-row" }, [
      h("button", { class: "icon-button", type: "button", "aria-label": "Attach file" }, [svgIcon("paperclip")]),
      h("span", { class: "chip" }, [svgIcon("shield"), h("span", { text: "Ask for approval" })]),
      h("span", { class: "spacer" }),
      h("button", { class: "icon-button", type: "button", "aria-label": "Use microphone" }, [svgIcon("mic")]),
      h("button", { class: "icon-button is-active", type: "button", "aria-label": "Send prompt" }, [svgIcon("send")])
    ]),
    h("div", { class: "context-strip" }, [
      h("span", { class: "readonly-chip" }, [svgIcon("gitBranch"), h("span", { text: "main" })]),
      h("span", { class: "readonly-chip" }, [svgIcon("bot"), h("span", { text: "sakana/fugu-l-ultra" })])
    ])
  ]);
}

function settingsWorkbench(mode = "plugins") {
  const titles = {
    plugins: ["Plugins", "Manage installed plugins, header icon placement, enablement, and shell-safe configuration."],
    "plugin-detail": ["File editor plugin", "Review rendered plugin configuration, repair states, and tool policy without exposing sensitive values."],
    "plugin-marketplace": ["Plugins", "Manage installed plugins and extension surfaces."],
    "plugin-states": ["Plugin states", "Review blocked, incompatible, pending restart, repair, and icon fallback states."],
    "plugin-uninstall": ["Uninstall plugin", "Choose whether plugin-owned user data is kept or deleted before uninstalling."],
    configuration: ["Configuration", "Review registered server-tool policy, remembered approval rules, and config source state."],
    "config-error": ["Configuration parse error", "Keep the last valid config active while the editable config source is repaired."],
    skills: ["Skills", "Show bundled, user-global, and plugin-provided skills without loading full skill instructions."],
    "skill-detail": ["Skill Creator", "Review metadata, source, validity, and $ suggestion status for one skill."],
    "skill-customize": ["Customize bundled skill", "Create a user-global copy because bundled skills are read-only."],
    "skill-invalid": ["Invalid skills", "Keep invalid skills visible in Settings while hiding them from $ suggestions."]
  };
  const [heading, summary] = titles[mode] || titles.plugins;
  const showPluginControls = mode === "plugins";
  return h("main", { class: "workbench", id: "main", tabindex: "-1" }, [
    h("section", { class: "settings-layout" }, [
      h("aside", { class: "settings-nav", "aria-label": "Settings navigation" }, [
        h("button", { class: "settings-back", type: "button", "data-route": "shell-foundation" }, [svgIcon("arrowLeft"), h("span", { text: "Back to app" })]),
        h("nav", { class: "settings-links" }, [
          settingsLink("Providers", "key", false, "#settings"),
          settingsLink("Models", "bot", false, "#settings"),
          settingsLink("Runtimes", "terminal", false, "#settings"),
          settingsLink("Configuration", "settings", mode === "configuration" || mode === "config-error", "#settings-configuration"),
          settingsLink("Plugins", "plug", mode.startsWith("plugin") || mode === "plugins", "#settings-plugins"),
          settingsLink("Skills", "file", mode.startsWith("skill") || mode === "skills", "#settings-skills"),
          settingsLink("MCP Servers", "terminal", false, "#settings")
        ])
      ]),
      h("div", { class: "settings-main" }, [
        h("div", { class: "settings-title" }, [
          h("h1", { text: heading }),
          h("p", { text: summary })
        ]),
        showPluginControls ? h("div", { class: "plugin-controls" }, [
          h("label", { class: "plugin-search" }, [svgIcon("search"), h("input", { type: "search", placeholder: "Search plugins", "aria-label": "Search plugins" })]),
          h("a", { class: "source-filter", href: "#settings-plugin-marketplace" }, [h("span", { text: "Add source" }), svgIcon("chevronDown")])
        ]) : null,
        h("div", { class: "settings-section" }, settingsContent(mode))
      ])
    ])
  ]);
}

function settingsLink(label, icon, active = false, href = "#settings") {
  return h("a", { class: `settings-link${active ? " is-active" : ""}`, href }, [svgIcon(icon), h("span", { text: label })]);
}

function settingsContent(mode) {
  if (mode === "repair") return repairCards();
  if (mode === "plugin-detail") return pluginDetail();
  if (mode === "plugin-marketplace") return pluginMarketplace();
  if (mode === "plugin-states") return pluginStateCards();
  if (mode === "plugin-uninstall") return pluginUninstall();
  if (mode === "configuration") return configurationSettings();
  if (mode === "config-error") return configErrorSettings();
  if (mode === "skills") return skillsSettings();
  if (mode === "skill-detail") return skillDetail();
  if (mode === "skill-customize") return skillCustomize();
  if (mode === "skill-invalid") return skillInvalid();
  return pluginCards();
}

function pluginCards() {
  return [
    pluginCard("File system", "Built by C4OS - enabled - activates Workspaces and Projects", "Enabled", "folder", "#settings-plugin-detail"),
    pluginCard("File editor", "Requires File system - activates file explorer and editor", "Available", "file", "#settings-plugin-detail"),
    pluginCard("Terminal", "Built by C4OS - user terminal per chat session", "Pending restart", "terminal", "#settings-plugin-states"),
    pluginCard("Chat Debug", "Former agent terminal - command and tool-call history", "Enabled", "bug", "#settings-plugin-states"),
    pluginCard("Browser", "Built by C4OS - in-browser preview per chat session", "Enabled", "globe", "#settings-plugin-detail"),
    h("div", { class: "inline-actions" }, [
      h("a", { class: "button secondary", href: "#settings-plugin-states" }, [svgIcon("circleAlert"), h("span", { text: "Review states" })]),
      h("a", { class: "button secondary", href: "#settings-plugin-uninstall" }, [svgIcon("x"), h("span", { text: "Uninstall flow" })])
    ])
  ];
}

function pluginCard(name, side, status, icon, href = "#settings-plugin-detail") {
  return h("div", { class: "settings-card" }, [
    h("span", { class: `plugin-logo${icon === "file" ? " is-fallback" : ""}` }, [svgIcon(icon)]),
    h("div", {}, [h("strong", { text: name }), h("p", { text: `${side} - ${status}` })]),
    h("a", { class: "icon-button", href, "aria-label": `${name} settings` }, [svgIcon("plus")])
  ]);
}

function pluginDetail() {
  return [
    h("section", { class: "settings-box detail-form" }, [
      h("div", { class: "detail-title-row" }, [
        h("div", {}, [
          h("h2", { text: "Rendered form from plugin config" }),
          h("p", { text: "These rows show possible field types. Labels and values are illustrative; the renderer is driven by the plugin settings schema." })
        ]),
        h("label", { class: "switch-line" }, [
          h("span", { text: "Enabled" }),
          h("span", { class: "switch is-on", role: "switch", "aria-checked": "true" }, [h("span")])
        ])
      ])
    ].concat(renderedFieldExamples())),
    h("section", { class: "below-form-grid" }, [
      h("div", { class: "settings-box compact-box" }, [
        h("h2", { text: "Repair states" }),
        stateRow("Reserved field validation", "`panel` and `iconOrder` validate from the same settings form before shell layout applies."),
        stateRow("Default-only setting", "A plugin may declare a default value without exposing a user-editable input."),
        h("a", { class: "button secondary", href: "#settings-plugin-states" }, [svgIcon("circleAlert"), h("span", { text: "Review repair states" })])
      ]),
      h("div", { class: "settings-box compact-box" }, [
        h("h2", { text: "Tool policy" }),
        stateRow("Gateway-owned tools", "Plugin-contributed tools are governed in Settings > Configuration."),
        stateRow("Secret use", "Sensitive settings stay redacted and raw use goes through governed calls."),
        h("a", { class: "button secondary", href: "#settings-configuration" }, [svgIcon("settings"), h("span", { text: "Open tool policy" })])
      ])
    ])
  ];
}

function renderedFieldExamples() {
  return [
    formField("Input", "Project root label", h("div", { class: "fake-input", text: "c4os workspace" })),
    formField("Number", "Max open editors", h("div", { class: "fake-input", text: "8" })),
    formField("Switch", "Auto save file drafts", h("span", { class: "switch is-on", role: "switch", "aria-checked": "true" }, [h("span")])),
    formField("Select", "Explorer density", h("div", { class: "fake-select" }, [h("span", { text: "Compact" }), svgIcon("chevronDown")])),
    formField("Input, sensitive", "Workspace token", h("div", { class: "fake-input", text: "Stored in keychain - redacted" })),
    formField("Default only", "Panel placement", h("div", { class: "default-value", text: "Left panel, not user editable" })),
    formField("Number", "Icon order", h("div", { class: "fake-input", text: "2" }))
  ];
}

function formField(type, label, control) {
  return h("div", { class: "rendered-field" }, [
    h("div", {}, [h("strong", { text: label }), h("span", { text: type })]),
    control
  ]);
}

function fieldRow(label, type, value) {
  return h("div", { class: "field-row" }, [
    h("div", {}, [h("strong", { text: label }), h("span", { text: type })]),
    h("code", { text: value })
  ]);
}

function pluginMarketplace() {
  return [
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
            h("div", { class: "plugin-filter-current", text: "Built by C4OS" }),
            h("div", { class: "plugin-filter-separator", role: "separator" }),
            h("button", { type: "button", "data-marketplace-open": "true" }, [h("span", { text: "+ Add Marketplace" })])
          ])
        ])
      ]),
      h("div", { class: "plugin-grid" }, pluginCatalog.map(pluginCatalogCard))
    ]),
    pluginConnectDialog(pluginCatalog[0]),
    marketplaceDialog()
  ];
}

function pluginCatalogCard(plugin) {
  return h("article", { class: "plugin-card" }, [
    h("div", { class: `plugin-logo plugin-logo-${plugin.tone}`, "aria-hidden": "true" }, [h("span", { text: plugin.logo })]),
    h("div", { class: "plugin-copy" }, [
      h("strong", { text: plugin.name }),
      h("span", { text: plugin.description })
    ]),
    h("button", { class: "plugin-add-button", type: "button", "aria-label": `Add ${plugin.name}`, "data-plugin-connect": plugin.name }, [svgIcon("plus")])
  ]);
}

function modalIconButton(label, icon, className, attrs = {}) {
  return h("button", { class: `icon-button ${className}`, type: "button", "aria-label": label, title: label, ...attrs }, [svgIcon(icon)]);
}

function pluginConnectDialog(plugin) {
  return h("div", { class: "plugin-modal", hidden: true, "data-plugin-modal": "connect" }, [
    h("div", { class: "plugin-modal-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "plugin-connect-title" }, [
      modalIconButton("Close plugin connection", "x", "plugin-modal-close", { "data-plugin-close": "true" }),
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
        svgIcon("globe")
      ]),
      h("a", { class: "plugin-modal-advanced", href: "#settings-plugin-detail", "data-plugin-advanced": "true" }, ["Advanced settings"])
    ])
  ]);
}

function modalCopyBlock(title, text) {
  return h("section", {}, [h("h3", { text: title }), h("p", { text })]);
}

function marketplaceDialog() {
  return h("div", { class: "plugin-modal", hidden: true, "data-marketplace-modal": "true" }, [
    h("div", { class: "marketplace-panel", role: "dialog", "aria-modal": "true", "aria-labelledby": "marketplace-title" }, [
      modalIconButton("Close marketplace dialog", "x", "plugin-modal-close", { "data-marketplace-close": "true" }),
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

function pluginStateCards() {
  return [
    stateCard("Dependency blocked", "File editor requires the File system plugin. Enable FS before the file explorer and editor can mount.", "Enable dependency"),
    stateCard("Incompatible", "A plugin built for an older C4OS settings schema is disabled before plugin code runs.", "Open source"),
    stateCard("Pending restart", "Terminal UI settings applied. New native backend registration waits for app restart; unavailable tools are labeled in commands and policy.", "Restart app"),
    stateCard("Repairable config", "File editor config failed validation and was reset to safe defaults. The plugin remains available with warning history.", "Review reset"),
    stateCard("Icon fallback", "SVG icon failed sanitization. Fixed shell slot shows fallback icon and preserves plugin row actions.", "Replace icon")
  ];
}

function stateCard(title, body, action) {
  return h("section", { class: "state-card" }, [
    h("div", {}, [h("strong", {}, [svgIcon("circleAlert"), h("span", { text: title })]), h("p", { text: body })]),
    h("button", { class: "button secondary", type: "button" }, [h("span", { text: action })])
  ]);
}

function pluginUninstall() {
  return [
    h("section", { class: "modal-sim" }, [
      h("h2", { text: "Uninstall File editor plugin?" }),
      h("p", { text: "The installed cache copy will be removed. Reinstall uses the configured marketplace source." }),
      h("label", { class: "check-row" }, [h("input", { type: "radio", name: "data", checked: "true" }), h("span", { text: "Keep plugin-owned user data and settings" })]),
      h("label", { class: "check-row" }, [h("input", { type: "radio", name: "data" }), h("span", { text: "Delete plugin-owned user data, cached state, and secrets" })]),
      h("div", { class: "inline-actions" }, [
        h("a", { class: "button secondary", href: "#settings-plugins" }, [h("span", { text: "Cancel" })]),
        h("button", { class: "button primary", type: "button" }, [svgIcon("x"), h("span", { text: "Uninstall" })])
      ])
    ])
  ];
}

function configurationSettings() {
  return [
    h("div", { class: "inline-actions" }, [
      h("a", { class: "button secondary", href: "#settings-config-error" }, [svgIcon("circleAlert"), h("span", { text: "Parse-error state" })])
    ]),
    toolPolicy("terminal.run", "Runs shell commands in the selected trusted project or explicit target.", "Ask", "Ask"),
    toolPolicy("files.read", "Reads files for user-directed or agent-initiated context.", "Allow", "Ask outside user-directed scope"),
    toolPolicy("git.worktree", "Creates or opens worktrees inside trusted projects.", "Allow", "Ask outside trusted project"),
    toolPolicy("credentials.use", "Allows governed use of stored provider or plugin secrets.", "Ask", "Ask")
  ];
}

function toolPolicy(id, explain, defaultPolicy, max) {
  return h("section", { class: "policy-row" }, [
    h("div", {}, [
      h("strong", { text: id }),
      h("p", { text: explain }),
      h("span", { class: "status-pill", text: `Default ${defaultPolicy}` }),
      h("span", { class: "status-pill", text: `Max ${max}` })
    ]),
    h("div", { class: "policy-actions" }, [
      h("div", { class: "icon-action-row" }, [
        h("button", { class: "icon-button", type: "button", "aria-label": `Edit ${id} remembered rule` }, [svgIcon("pencil")]),
        h("button", { class: "icon-button", type: "button", "aria-label": `Revoke ${id} remembered rule` }, [svgIcon("x")])
      ])
    ])
  ]);
}

function configErrorSettings() {
  return [
    h("section", { class: "state-card" }, [
      h("div", {}, [
        h("strong", {}, [svgIcon("circleAlert"), h("span", { text: "config.toml parse error" })]),
        h("p", { text: "Settings writes to config.toml, but the current file cannot parse. C4OS keeps the last valid config active until this is fixed." })
      ]),
      h("button", { class: "button secondary", type: "button" }, [h("span", { text: "Open config.toml" })])
    ]),
    h("section", { class: "settings-box" }, [
      h("h2", { text: "Last valid fallback" }),
      fieldRow("Loaded at", "timestamp", "2026-07-02 18:04"),
      fieldRow("Runtime", "config", "Pi"),
      fieldRow("Tool policy", "config", "terminal.run ask, browser.open allow"),
      h("p", { class: "note", text: "Edits are blocked from saving until the parse error is repaired or the user restores the last valid config." })
    ])
  ];
}

function skillsSettings() {
  return [
    h("div", { class: "skills-panel", "aria-label": "Skills" }, [
      h("label", { class: "skills-search" }, [
        svgIcon("search"),
        h("span", { class: "sr-only", text: "Search skills" }),
        h("input", { type: "search", name: "skill-search", autocomplete: "off", placeholder: "Search skills" })
      ]),
      h("div", { class: "skills-list" }, skillCatalog.map(skillRow))
    ]),
    skillDetailDialog(skillCatalog[6])
  ];
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
    h("button", { class: "skill-switch", type: "button", "aria-label": `${skill.name} enabled` }, [h("span")])
  ]);
}

function skillDetail() {
  return [
    h("div", { class: "skills-panel" }, [
      h("div", { class: "skills-list" }, skillCatalog.slice(0, 3).map(skillRow))
    ]),
    skillDetailPanel(skillCatalog[6], true, false)
  ];
}

function skillDetailDialog(skill) {
  return h("div", { class: "plugin-modal", hidden: true, "data-skill-modal": "true" }, [
    skillDetailPanel(skill, false, true)
  ]);
}

function skillDetailPanel(skill, open = false, modal = false) {
  return h("section", { class: `skill-modal-panel${open ? " is-route-preview" : ""}`, role: "dialog", "aria-modal": modal ? "true" : "false", "aria-labelledby": "skill-modal-title" }, [
    modal ? modalIconButton("Close skill details", "x", "plugin-modal-close", { "data-skill-close": "true" }) : null,
    h("header", { class: "skill-modal-head" }, [
      h("span", { class: "skill-modal-icon", "aria-hidden": "true" }, [svgIcon("file")]),
      h("div", { class: "skill-modal-title-row" }, [
        h("h2", { id: "skill-modal-title", "data-skill-title": modal ? "true" : null, text: skill.name }),
        h("span", { text: "Skill" })
      ]),
      h("p", { "data-skill-summary": modal ? "true" : null, text: skill.description })
    ]),
    h("div", { class: "skill-modal-controls" }, [
      h("button", { class: "skill-switch", type: "button", "aria-label": "Skill enabled" }, [h("span")]),
      h("button", { class: "icon-button skill-more", type: "button", "aria-label": "More skill actions" }, [svgIcon("settings")])
    ]),
    h("div", { class: "skill-detail-copy", "data-skill-detail": modal ? "true" : null }, renderSkillDetail(skill.detail)),
    h("footer", { class: "skill-modal-actions" }, [
      h("button", { class: "button secondary danger-action", type: "button", "data-skill-close": modal ? "true" : null }, ["Uninstall"]),
      h("button", { class: "button primary", type: "button", "data-skill-close": modal ? "true" : null }, ["Try in chat"])
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

function skillCustomize() {
  return [
    h("section", { class: "modal-sim" }, [
      h("h2", { text: "Create user-global copy?" }),
      h("p", { text: "Bundled skills are read-only. Customization creates an editable user-global copy and leaves the bundled source intact." }),
      fieldRow("Copy name", "text input", "skill-creator-custom"),
      fieldRow("Destination", "user-global", "$CODEX_HOME/skills/skill-creator-custom"),
      h("div", { class: "inline-actions" }, [
        h("a", { class: "button secondary", href: "#settings-skills" }, [h("span", { text: "Cancel" })]),
        h("button", { class: "button primary", type: "button" }, [svgIcon("plus"), h("span", { text: "Create copy" })])
      ])
    ])
  ];
}

function skillInvalid() {
  return [
    stateCard("Missing SKILL.md", "Source folder exists but no SKILL.md is present. Hidden from $ suggestions.", "Open folder"),
    stateCard("Invalid frontmatter", "YAML cannot parse. C4OS reads no full instructions and keeps runtime context clean.", "Repair file"),
    stateCard("Missing name or description", "Required display metadata is absent. The skill stays listed with repair reason only.", "Edit metadata"),
    stateCard("Duplicate name", "A user-global copy conflicts with bundled skill-creator. Choose the active source explicitly.", "Resolve duplicate"),
    stateCard("Unreadable or source unavailable", "The source path cannot be read now. The skill is hidden from suggestions until available.", "Retry")
  ];
}

function stateRow(label, text) {
  return h("div", { class: "field-row" }, [h("strong", { text: label }), h("p", { text })]);
}

function repairCards() {
  return [
    h("div", { class: "repair-card" }, [
      h("strong", {}, [svgIcon("circleAlert"), " File editor"]),
      h("p", { text: "Invalid shell field: panel must be left or right. The shell did not mount the panel." }),
      h("p", { text: "Action: repair plugin settings or disable the shell contribution." })
    ]),
    pluginCard("Browser", "Right side", "Enabled", "globe"),
    pluginCard("Files", "Left side", "Enabled", "folder")
  ];
}

function coverageWorkbench() {
  return h("main", { class: "workbench", id: "main", tabindex: "-1" }, [
    h("section", { class: "coverage-main" }, [
      h("h1", { text: "Wireframe coverage" }),
      h("p", { text: "Batch 2 adds focused Settings and Configuration routes for specs 03, 04, and 11 without recreating the full r04 Settings route set." }),
      h("table", { class: "matrix" }, [
        h("thead", {}, [h("tr", {}, ["r05 state", "Spec 03", "Spec 04", "Spec 11", "Review intent"].map((text) => h("th", { text })))]),
        h("tbody", {}, [
          matrixRow("#settings-plugins", "REQ-002, REQ-004, REQ-006, AC-001, AC-008", "-", "-", "Plugin list, panel/icon order, statuses, fallback icon"),
          matrixRow("#settings-plugin-detail", "REQ-003, REQ-008, AC-002, AC-009", "REQ-007", "-", "Settings renderer and sensitive redaction"),
          matrixRow("#settings-plugin-marketplace", "REQ-005, REQ-010, AC-006, AC-011", "-", "-", "Source add/install flow and metadata-first discovery"),
          matrixRow("#settings-plugin-states", "REQ-004, REQ-012, AC-001, AC-003", "REQ-008", "-", "Blocked, incompatible, pending-restart, repairable states"),
          matrixRow("#settings-plugin-uninstall", "REQ-005, AC-004", "-", "-", "Uninstall data-retention prompt"),
          matrixRow("#settings-configuration", "-", "REQ-003, REQ-008, AC-006, AC-007", "-", "Per server-tool policy and remembered rule review/edit/revoke"),
          matrixRow("#settings-config-error", "REQ-002", "REQ-004", "-", "Parse-error and last-valid config fallback"),
          matrixRow("#settings-skills", "-", "-", "REQ-001 through REQ-005, AC-001, AC-003, AC-004", "Skills list with source/status/suggestion boundaries"),
          matrixRow("#settings-skill-detail", "-", "-", "REQ-002, REQ-010, AC-003, AC-006", "Metadata-first skill detail"),
          matrixRow("#settings-skill-customize", "-", "-", "REQ-005, AC-004", "Bundled read-only to user-global copy"),
          matrixRow("#settings-skill-invalid", "-", "-", "REQ-006, REQ-008, AC-002, AC-005", "Invalid states and $ suggestion filtering")
        ])
      ])
    ])
  ]);
}

function matrixRow(...cells) {
  return h("tr", {}, cells.map((text) => h("td", { text })));
}

function bindPluginConnectDialog() {
  const modal = document.querySelector("[data-plugin-modal='connect']");
  if (!modal) return;
  const title = modal.querySelector("[data-plugin-modal-title]");
  const action = modal.querySelector("[data-plugin-modal-action]");
  const logo = modal.querySelector("[data-plugin-modal-logo]");
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
      logo.querySelector("span").textContent = plugin.logo;
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

function render() {
  const route = routeStates[routeFromHash()] ? routeFromHash() : defaultRoute;
  const state = routeStates[route];
  document.body.dataset.route = route;
  if (state.coverage) {
    app.replaceChildren(coverageWorkbench());
    bindPluginConnectDialog();
    bindMarketplaceControls();
    bindSkillDetails();
    return;
  }
  const classes = ["app-shell"];
  if (state.left) classes.push("has-left");
  if (state.right) classes.push("has-right");

  const children = [headerFor(state)];
  if (state.left) children.push(pluginPanel(state.left), resizeHandle("left"));
  children.push(state.settings ? settingsWorkbench(route === "repair-state" ? "repair" : state.settings) : chatWorkbench(state));
  if (state.right) children.push(resizeHandle("right"), pluginPanel(state.right));
  app.replaceChildren(h("div", { class: classes.join(" ") }, children));
  bindPluginConnectDialog();
  bindMarketplaceControls();
  bindSkillDetails();
}

document.addEventListener("click", (event) => {
  const pluginButton = event.target.closest("[data-plugin]");
  const routeButton = event.target.closest("[data-route]");
  const closeButton = event.target.closest("[data-close-side]");
  if (pluginButton) {
    const id = pluginButton.dataset.plugin;
    const current = routeStates[routeFromHash()] || routeStates[defaultRoute];
    const side = plugins[id].side;
    if (current[side] === id) go("shell-foundation");
    else if (side === "left") go(id === "files" ? "same-side-replacement" : "per-chat-restore");
    else go(id === "terminal" ? "same-side-replacement" : id === "debug" ? "debug" : "per-chat-restore");
  } else if (routeButton) {
    go(routeButton.dataset.route);
  } else if (closeButton) {
    go("shell-foundation");
  }
});

window.addEventListener("hashchange", render);
render();
