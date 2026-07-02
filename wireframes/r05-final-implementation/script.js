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
  coverage: { title: "Wireframe Coverage", subtitle: "Specs 01 and 02", coverage: true }
};

const chats = [
  { title: "Locate Tauri integration", active: true },
  { title: "Draft wireframes", active: false },
  { title: "Plugin panel restore", active: false }
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
  return h("main", { class: "workbench", id: "main", tabindex: "-1" }, [
    h("section", { class: "settings-layout" }, [
      h("aside", { class: "settings-nav", "aria-label": "Settings navigation" }, [
        h("button", { class: "settings-back", type: "button", "data-route": "shell-foundation" }, [svgIcon("arrowLeft"), h("span", { text: "Back to app" })]),
        h("nav", { class: "settings-links" }, [
          settingsLink("Providers", "key"),
          settingsLink("Models", "bot"),
          settingsLink("Runtimes", "terminal"),
          settingsLink("Configuration", "settings"),
          settingsLink("Plugins", "plug", true),
          settingsLink("Skills", "file"),
          settingsLink("MCP Servers", "terminal")
        ])
      ]),
      h("div", { class: "settings-main" }, [
        h("div", { class: "settings-title" }, [
          h("h1", { text: mode === "repair" ? "Plugin repair" : "Plugins" }),
          h("p", { text: "Manage installed plugins, header icon placement, enablement, and shell-safe configuration." })
        ]),
        mode === "repair" ? null : h("div", { class: "plugin-controls" }, [
          h("label", { class: "plugin-search" }, [svgIcon("search"), h("input", { type: "search", placeholder: "Search plugins", "aria-label": "Search plugins" })]),
          h("button", { class: "source-filter", type: "button" }, [h("span", { text: "Built by C4OS" }), svgIcon("chevronDown")])
        ]),
        h("div", { class: "settings-section" }, mode === "repair" ? repairCards() : pluginCards())
      ])
    ])
  ]);
}

function settingsLink(label, icon, active = false) {
  return h("a", { class: `settings-link${active ? " is-active" : ""}`, href: "#settings" }, [svgIcon(icon), h("span", { text: label })]);
}

function pluginCards() {
  return [
    pluginCard("GitHub", "Installed from Built by C4OS...", "Available", "plug"),
    pluginCard("Browser", "Right side", "Enabled", "globe"),
    pluginCard("Files", "Left side", "Enabled", "folder"),
    pluginCard("Terminal", "Right side", "Enabled", "terminal"),
    pluginCard("Chat Debug", "Right side", "Disabled", "bug")
  ];
}

function pluginCard(name, side, status, icon) {
  return h("div", { class: "settings-card" }, [
    h("span", { class: "plugin-logo", text: name === "GitHub" ? "GH" : "" }, name === "GitHub" ? [] : [svgIcon(icon)]),
    h("div", {}, [h("strong", { text: name }), h("p", { text: `${side} - ${status}` })]),
    h("button", { class: "icon-button", type: "button", "aria-label": `${name} settings` }, [svgIcon("plus")])
  ]);
}

function repairCards() {
  return [
    h("div", { class: "repair-card" }, [
      h("strong", {}, [svgIcon("circleAlert"), " Ops Board"]),
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
      h("p", { text: "Review metadata is isolated here so the app shell routes stay usable and visually comparable to r04." }),
      h("table", { class: "matrix" }, [
        h("thead", {}, [h("tr", {}, ["r05 state", "Spec 01", "Spec 02", "Review intent"].map((text) => h("th", { text })))]),
        h("tbody", {}, [
          matrixRow("#shell-foundation", "REQ-001, REQ-006, AC-001", "REQ-001, REQ-002, AC-001, AC-004", "Default shell, no right panel"),
          matrixRow("#same-side-replacement", "REQ-005, AC-005", "REQ-003, REQ-004, AC-002", "Panel toggle and replacement"),
          matrixRow("#per-chat-restore", "REQ-007, AC-007", "REQ-005, AC-002", "Per-chat restore"),
          matrixRow("#settings", "REQ-006, AC-001", "REQ-007, AC-002, AC-007", "Settings center route"),
          matrixRow("#resize-collision", "REQ-010, AC-010", "REQ-006, AC-003", "640px center minimum"),
          matrixRow("#hidden-activity", "REQ-005, AC-005, AC-006", "REQ-008, AC-005, AC-006, AC-008", "Hidden plugin activity indicator"),
          matrixRow("#repair-state", "REQ-008, AC-004", "REQ-009, AC-007, AC-009", "Invalid layout repair")
        ])
      ])
    ])
  ]);
}

function matrixRow(...cells) {
  return h("tr", {}, cells.map((text) => h("td", { text })));
}

function render() {
  const route = routeStates[routeFromHash()] ? routeFromHash() : defaultRoute;
  const state = routeStates[route];
  document.body.dataset.route = route;
  if (state.coverage) {
    app.replaceChildren(coverageWorkbench());
    return;
  }
  const classes = ["app-shell"];
  if (state.left) classes.push("has-left");
  if (state.right) classes.push("has-right");

  const children = [headerFor(state)];
  if (state.left) children.push(pluginPanel(state.left), resizeHandle("left"));
  children.push(state.settings ? settingsWorkbench(route === "repair-state" ? "repair" : "plugins") : chatWorkbench(state));
  if (state.right) children.push(resizeHandle("right"), pluginPanel(state.right));
  app.replaceChildren(h("div", { class: classes.join(" ") }, children));
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
