const screen = document.body.dataset.screen || "hub";
const app = document.querySelector("#app");

const iconPaths = {
  "arrow-left": '<path d="m12 19-7-7 7-7"></path><path d="M19 12H5"></path>',
  "badge-check": '<path d="M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.78 4.78 4 4 0 0 1-6.74 0 4 4 0 0 1-4.78-4.78 4 4 0 0 1 0-6.75Z"></path><path d="m9 12 2 2 4-4"></path>',
  "bot": '<path d="M12 8V4H8"></path><rect width="16" height="12" x="4" y="8" rx="2"></rect><path d="M2 14h2"></path><path d="M20 14h2"></path><path d="M9 13v2"></path><path d="M15 13v2"></path>',
  "chevron-down": '<path d="m6 9 6 6 6-6"></path>',
  "chevron-left": '<path d="m15 18-6-6 6-6"></path>',
  "chevron-right": '<path d="m9 18 6-6-6-6"></path>',
  "database": '<ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"></path><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"></path>',
  "file": '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"></path><path d="M14 2v4a2 2 0 0 0 2 2h4"></path>',
  "folder": '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"></path>',
  "git-branch": '<line x1="6" x2="6" y1="3" y2="15"></line><circle cx="18" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><path d="M18 9a9 9 0 0 1-9 9"></path>',
  "globe": '<circle cx="12" cy="12" r="10"></circle><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"></path><path d="M2 12h20"></path>',
  "key-round": '<path d="M2 18v3h3l9.4-9.4"></path><circle cx="16" cy="8" r="6"></circle>',
  "mic": '<path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><path d="M12 19v3"></path>',
  "panel-left": '<rect width="18" height="18" x="3" y="3" rx="2"></rect><path d="M9 3v18"></path>',
  "panel-right": '<rect width="18" height="18" x="3" y="3" rx="2"></rect><path d="M15 3v18"></path>',
  "paperclip": '<path d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 1 1-2.83-2.83l8.49-8.48"></path>',
  "pencil": '<path d="M21.17 6.83a2.83 2.83 0 0 0-4-4L3 17v4h4Z"></path><path d="m15 5 4 4"></path>',
  "plug": '<path d="M12 22v-5"></path><path d="M9 8V2"></path><path d="M15 8V2"></path><path d="M18 8v5a6 6 0 0 1-12 0V8Z"></path>',
  "plus": '<path d="M5 12h14"></path><path d="M12 5v14"></path>',
  "search": '<path d="m21 21-4.34-4.34"></path><circle cx="11" cy="11" r="8"></circle>',
  "send-horizontal": '<path d="M3.71 3.71 21 12 3.71 20.29 6.5 12Z"></path><path d="M6.5 12H21"></path>',
  "server": '<rect width="20" height="8" x="2" y="2" rx="2"></rect><rect width="20" height="8" x="2" y="14" rx="2"></rect><path d="M6 6h.01"></path><path d="M6 18h.01"></path>',
  "settings": '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.38a2 2 0 0 0-.73-2.73l-.15-.09a2 2 0 0 1-1-1.74v-.51a2 2 0 0 1 1-1.72l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2Z"></path><circle cx="12" cy="12" r="3"></circle>',
  "shield": '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.68 0C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1Z"></path>',
  "terminal": '<polyline points="4 17 10 11 4 5"></polyline><line x1="12" x2="20" y1="19" y2="19"></line>',
  "trash-2": '<path d="M3 6h18"></path><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"></path><path d="M10 11v6"></path><path d="M14 11v6"></path>',
  "webhook": '<path d="M18 16.98h-5.99c-1.1 0-2.1-.45-2.83-1.17"></path><path d="m6 17 3-3-3-3"></path><path d="M14 7.02h5.99c1.1 0 2.1.45 2.83 1.17"></path><path d="m18 7-3 3 3 3"></path>'
};

function icon(name, label = name) {
  const path = iconPaths[name] || iconPaths.file;
  return `<svg class="lucide" aria-label="${label}" role="img" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${path}</svg>`;
}

const pages = [
  ["App start", "./app-start.html", "Folder-first entry state before a project is opened."],
  ["New session", "./new-session.html", "Trusted project shell with empty prompt composer."],
  ["Chat session", "./chat-session.html", "Messenger-style session, tool block, and approval block."],
  ["Providers popover", "./providers-popover.html", "Model chip opened to provider list."],
  ["Models popover", "./models-popover.html", "Model chip opened to active provider model list."],
  ["File explorer", "./file-explorer.html", "Right panel Files tab in tree state."],
  ["File editor", "./file-editor.html", "Right panel Files tab in breadcrumb/code state."],
  ["Terminal", "./terminal.html", "Right panel Terminal tab with read-only output and bottom panel."],
  ["Settings providers", "./settings-providers.html", "Provider connections and enable toggles."],
  ["Add provider", "./settings-add-provider.html", "Provider form with key under base URL."],
  ["Settings models", "./settings-models.html", "Fetched models with provider labels and toggles."],
  ["Settings configuration", "./settings-configuration.html", "Approval, sandbox, and config file access."]
];

const threads = [
`Basic message`,
`Message received.`,
`"Is the progressive page set complete?" - It's not suppose to expose levels passed level 1. Level 2-8 are suppose to be secret so dont mention it until they get to the end of level 1. 

"Does first visit feel focused enough?" - "Continue Level 1" should be "Getting Started". Move "Your docs progress" to a popup when they click badge on top right. Remove "Next Unlock". Badge on top right use icons from font awesome free. There's too much copy about unlocking levels, remove all of that just focus on 000-100 topics. Dont use the term "Level". These are just guides that unlock as they go along.

"Is the reading-ahead state clear without blocking deep-link readers?" - I think it's too clear. I want to make it seem like Stackpress is only 000 and 100 until they finish reading 160 debugging.

"Does API/search staying outside the course gate feel right?" - This is a static site so no search.

In guides, reduce the padding in the main section by like half. Also wireframes will not be complete until the wireframes are fully functional. Meaning you need to`,

`Updated the v2 wireframes.
Changes made:
Removed search.html and the reading-ahead page/state.
Removed visible “Level”, unlock, search, and later-guide copy from first-load pages.
Changed the homepage CTA to Getting Started.
Moved progress into a top-right badge popup using an embedded Font Awesome Free info icon.
Reduced the guide page/article padding.
Added [script.js](/Users/cblanquera/server/projects/stackpress/stackpress/packages/www/design/wireframes/stackpress-docs-v2/script.js) for cookie/localStorage state, completion buttons, badge popup, reset, and post-160 Debugging reveal.
Added a grayscale layout/theme shift once 160 Debugging is marked complete.
Verification passed:
index.html, guides.html, guide-detail.html, api.html, and mobile-menu.html all returned 200.
No visible Search, reading ahead, or Level copy on first-load pages.
Badge popup opens.
Completing 160-debugging writes the cookie and reveals the next guide group.
Temporary preview server was stopped.
Reload the in-app browser page to pick up the revised static files.`,

`Can you explain what the strategy of these wireframes are? because it looks like a proof of concept more than the final wireframes. Im afraid if we move to creative designs it will take the wireframes as is and just colorize it.`,

`Using \`chrisai-design-wireframes\`: you’re right. The current v2 is more of an interaction proof than a final wireframe set.

The strategy behind it was:

1. Prove the progressive docs behavior first:
   - 000/100 only on first visit
   - badge popup owns progress
   - API stays available
   - no search
   - later guide groups stay hidden
   - completing \`160 Debugging\` changes state and reveals the next group

2. Keep the visual language intentionally plain:
   - grayscale
   - obvious boxes
   - minimal layout styling
   - functional state controls

That means it is useful as a product-behavior prototype, but it is not yet safe as a creative-design handoff. If we moved directly to creative now, you’re right: the next phase could accidentally treat this layout as approved and just colorize it.

The better next step is another wireframe pass with two explicit tracks:

- **Behavior Prototype:** keep the current functional cookie/state prototype as proof that progression works.
- **Final Structural Wireframes:** create cleaner, final-review page compositions for home, guides, guide detail, API, badge popup, mobile drawer, and post-160 state, without exposing internal prototype controls as primary UI.

I would not move this to creative yet. I’d mark the current v2 as “behavior prototype” and make a \`v2-final\` or \`v2-round2\` wireframe pass that preserves the approved strategy but redesigns the actual page structure for handoff.`

];

function hub() {
  app.innerHTML = `
    <section class="hub">
      <h1>C4OS Functional Wireframes</h1>
      <p>Round 1 grayscale HTML/CSS/JS draft. These pages simulate structure and interaction only.</p>
      <div class="screen-grid">
        ${pages.map(([title, href, summary]) => `
          <a class="screen-link" href="${href}">
            <strong>${title}</strong>
            <span>${summary}</span>
          </a>
        `).join("")}
      </div>
    </section>
  `;
}

function sidebar(activeSession = false) {
  return `
    <aside class="sidebar">
      <div class="search-shell"><span class="icon">${icon("search")}</span><input class="search" aria-label="Search" value="Search" readonly></div>
      <div class="plugin-row"><span class="icon">${icon("plug")}</span><span>Plugins</span></div>
      <div>
        <div class="project-heading"><span>Projects</span><span>${icon("plus", "Add project")}</span></div>
        <a class="project-row" href="./new-session.html"><span class="icon">${icon("folder")}</span><span>suite</span><span>${icon("pencil")}</span><span>${icon("trash-2")}</span></a>
        <a class="project-row ${activeSession ? "" : "active"}" href="./new-session.html"><span class="icon">${icon("folder")}</span><span>c4os2</span><span>${icon("pencil")}</span><span>${icon("trash-2")}</span></a>
        <a class="chat-row ${activeSession ? "active" : ""}" href="./chat-session.html"><span>Locate Tauri integration</span></a>
        <a class="chat-row" href="./new-session.html"><span>Draft wireframes</span></a>
        <a class="project-row" href="./new-session.html"><span class="icon">${icon("folder")}</span><span>techops</span><span>${icon("pencil")}</span><span>${icon("trash-2")}</span></a>
        <a class="project-row" href="./new-session.html"><span class="icon">${icon("folder")}</span><span>ingest</span><span>${icon("pencil")}</span><span>${icon("trash-2")}</span></a>
        <a class="project-row" href="./new-session.html"><span class="icon">${icon("folder")}</span><span>chrisai</span><span>${icon("pencil")}</span><span>${icon("trash-2")}</span></a>
      </div>
      <a class="settings-link" href="./settings-providers.html"><span class="icon">${icon("settings")}</span><span>Settings</span></a>
    </aside>
  `;
}

function topbar(title) {
  return `
    <header class="topbar">
      <div class="container">
        <button aria-label="Collapse left panel">${icon("panel-left")}</button>
        <div class="topbar-label">${title}</div>
        <button aria-label="Collapse right panel">${icon("panel-right")}</button>
      </div>
    </header>
  `;
}

function composer(popover = "") {
  return `
    <div class="composer">
      <div class="composer-input" contenteditable="true" role="textbox" aria-label="Prompt" data-placeholder="Do anything"></div>
      <div class="composer-controls">
        <button class="control-chip icon-only attach" aria-label="Attach file">${icon("paperclip")}</button>
        <button class="control-chip with-icon approve">${icon("shield")}<span class="approve-label">Approve for me</span></button>
        <div class="divider"></div>
        <button class="control-chip icon-only" aria-label="Microphone">${icon("mic")}</button>
        <button class="send" aria-label="Send">${icon("send-horizontal")}</button>
      </div>
      <div class="context-row">
        <div class="branch">${icon("git-branch")}<span class="branch-label">main</span></div>
        <div class="divider"></div>
        <a class="control-chip model-chip with-icon model" href="./models-popover.html">
          <span class="model-label">
            ${icon("bot")}
            <span>gemini-flash-2.5</span>
          </span>
          ${icon("chevron-down")}
        </a>
      </div>
      ${popover}
    </div>
  `;
}

function browserPanel() {
  return `
    <section class="tool-body">
      <div class="browser-bar"><span>${icon("globe")}</span><div class="address">http://127.0.0.1:3000</div></div>
      <div class="preview-card">
        <h3>Rendered page mock</h3>
        <p>Single browser surface; tabs are out of scope.</p>
      </div>
    </section>
  `;
}

function filesPanel(editor = false) {
  if (editor) {
    return `
      <section class="tool-body code-view">
        <div class="breadcrumbs"><a href="./file-explorer.html">c4os2</a> <span>${icon("chevron-right")}</span> <a href="./file-explorer.html">frontend</a> <span>${icon("chevron-right")}</span> <span>main.js</span></div>
        <pre class="code-lines">1  import { start } from "./runtime";
2
3  const project = "c4os2";
4  const trustedRoot = true;
5
6  start({ project, trustedRoot });
7
8  // Wireframe code view fills the panel.</pre>
      </section>
    `;
  }
  return `
    <section class="tool-body file-panel">
      <div class="file-title">c4os2</div>
      <a class="tree-row" href="./file-explorer.html"><span>${icon("chevron-right")}</span><span>${icon("folder")}</span><span>backend</span></a>
      <div class="tree-row"><span>${icon("chevron-down")}</span><span>${icon("folder")}</span><span>frontend</span></div>
      <a class="tree-row indent active" href="./file-editor.html"><span></span><span>${icon("file")}</span><span>main.js</span></a>
      <a class="tree-row indent" href="./file-editor.html"><span></span><span>${icon("file")}</span><span>index.html</span></a>
      <a class="tree-row" href="./file-explorer.html"><span>${icon("chevron-right")}</span><span>${icon("folder")}</span><span>tests</span></a>
    </section>
  `;
}

function terminalPanel() {
  return `
    <section class="tool-body terminal-panel">
      <div class="term-output">
        <div>$ npm run dev</div>
        <div>ready in 614ms</div>
        <div>local preview available at 127.0.0.1:3000</div>
      </div>
      <div class="term-bottom">
        <div>AI command preview/results</div>
        <div>read-only bottom panel</div>
        <div>resize handle sits on this top boundary</div>
      </div>
    </section>
  `;
}

function rightPanel(active = "browser") {
  const body = active === "files" ? filesPanel(screen === "file-editor") : active === "terminal" ? terminalPanel() : browserPanel();
  return `
    <aside class="right-panel">
      <nav class="tabs" aria-label="Workspace tools">
        <a class="tab ${active === "browser" ? "active" : ""}" href="./new-session.html">${icon("globe", "Browser")}</a>
        <a class="tab ${active === "files" ? "active" : ""}" href="./file-explorer.html">${icon("file", "Files")}</a>
        <a class="tab ${active === "terminal" ? "active" : ""}" href="./terminal.html">${icon("terminal", "Terminal")}</a>
      </nav>
      ${body}
    </aside>
  `;
}

function providerPopover() {
  return `
    <div class="popover">
      <div class="popover-head"><strong>Providers</strong><span>Select source</span></div>
      <a class="popover-row" href="./models-popover.html"><span>OpenRouter</span><span>${icon("chevron-right")}</span></a>
      <a class="popover-row" href="./models-popover.html"><span>OpenAI</span><span>${icon("chevron-right")}</span></a>
      <a class="popover-row" href="./models-popover.html"><span>LiteLLM Local</span><span>${icon("chevron-right")}</span></a>
    </div>
  `;
}

function modelPopover() {
  return `
    <div class="popover">
      <div class="popover-head"><a class="with-icon" href="./providers-popover.html">${icon("chevron-left")}<span>OpenRouter</span></a></div>
      <a class="popover-row selected" href="./new-session.html"><span>Gemini 2.5 Flash</span><span></span></a>
      <a class="popover-row" href="./new-session.html"><span>ChatGPT o4</span><span></span></a>
      <a class="popover-row" href="./new-session.html"><span>Grok 2.0</span><span></span></a>
    </div>
  `;
}

function shell(state) {
  const activeSession = state === "chat-session";
  const activeTool = state === "file-explorer" || state === "file-editor" ? "files" : state === "terminal" ? "terminal" : "browser";
  const title = activeSession ? "Locate Tauri integration" : "c4os2";
  const popover = state === "providers-popover" ? providerPopover() : state === "models-popover" ? modelPopover() : "";
  const center = activeSession ? thread() : emptyWorkspace(popover);
  app.innerHTML = `
    <div class="shell sidebar-open right-open">
      ${sidebar(activeSession)}
      <section class="center">${topbar(title)}${center}</section>
      ${rightPanel(activeTool)}
    </div>
  `;
  bindThread();
}

function emptyWorkspace(popover) {
  return `
    <section class="workspace empty">
      <h2>What should we build in c4os2?</h2>
      ${composer(popover)}
    </section>
  `;
}

function message(id, text, type = "user") {
  //get lines of text
  const lines = text.split("\n").length;
  const length = text.length + (lines * 100);
  if (length > 1000) {
    return `<article class="message ${type} message-clipped">
      <input type="checkbox" class="message-visible" id="message-${id}" />
      <div class="inline-text">
        <pre>${text}</pre>
      </div>
      <label for="message-${id}" data-open="Show Less" data-close="Show More" class="message-visible-trigger"></label>
    </article>`;
  }
  return `<article class="message ${type}">
    <div class="inline-text">
      <pre>${text}</pre>
    </div>
  </article>`;
}

function thread() {
  return `
    <section class="workspace">
      <div class="thread">
        ${message(0, threads[0], "user")}
        ${message(1, threads[1], "agent")}
        ${message(2, threads[2], "user")}
        ${message(3, threads[3], "agent")}
        <article class="message agent">
          <div class="skeleton"></div>
          <div class="skeleton mid"></div>
          <div class="approval-row"><strong>Sensitive action waiting</strong><button class="secondary">Review Approval</button></div>
        </article>
        <article class="message user">
          <div class="skeleton"></div>
          <div class="skeleton mid"></div>
        </article>
        <article class="message agent" data-expandable>
          <div class="skeleton"></div>
          <div class="skeleton"></div>
          <div class="skeleton mid"></div>
          <button class="control-chip" data-show-more>Show more</button>
          <div class="show-extra">
            <div class="skeleton"></div>
            <div class="skeleton short"></div>
          </div>
        </article>
        <article class="message agent">
          <div class="skeleton"></div>
          <div class="skeleton mid"></div>
        </article>
      </div>
      <div class="floating-composer">${composer("")}</div>
    </section>
  `;
}

function settingsNav(active) {
  const nav = [
    ["Providers", "settings-providers", "./settings-providers.html", "key-round"],
    ["Models", "settings-models", "./settings-models.html", "database"],
    ["Configuration", "settings-configuration", "./settings-configuration.html", "settings"],
    ["Skills", "skills", "#", "file"],
    ["MCP Servers", "mcp", "#", "server"],
    ["Hooks", "hooks", "#", "webhook"]
  ];
  return `
    <aside class="settings-nav">
      <a class="settings-link" href="./new-session.html"><span class="icon">${icon("arrow-left")}</span><span>Back to app</span></a>
      <div class="project-heading"><span>Settings</span><span></span></div>
      ${nav.map(([label, key, href, iconName]) => `
        <a class="nav-row ${active === key ? "active" : ""}" href="${href}">
          <span class="icon">${icon(iconName)}</span><span>${label}</span><span></span>
        </a>
      `).join("")}
    </aside>
  `;
}

function settings(active) {
  app.innerHTML = `
    <div class="settings-shell">
      ${settingsNav(active)}
      <main class="settings-main">${settingsBody(active)}</main>
    </div>
  `;
}

function settingsBody(active) {
  if (active === "settings-add-provider") return addProvider();
  if (active === "settings-models") return models();
  if (active === "settings-configuration") return configuration();
  return providers();
}

function providers() {
  return `
    <div class="settings-title">
      <div><h1>Providers</h1><p>Manage OpenAI-compatible provider connections. Labels must be unique.</p></div>
      <a class="primary with-icon" href="./settings-add-provider.html">${icon("plus")}<span>Add provider</span></a>
    </div>
    <div class="list">
      ${providerRow("OpenRouter - Personal", "https://openrouter.ai/api/v1 - API key saved")}
      ${providerRow("OpenAI - Work", "https://api.openai.com/v1 - API key saved")}
      ${providerRow("LiteLLM Local", "http://localhost:4000/v1 - API key saved")}
    </div>
  `;
}

function providerRow(title, sub) {
  return `<div class="list-row"><div><div class="row-title">${title}</div><div class="row-sub">${sub}</div></div><button class="secondary">Edit</button><button class="toggle" aria-label="Enabled"></button></div>`;
}

function addProvider() {
  return `
    <div class="settings-title">
      <div><h1>Add Provider</h1><p>Save an OpenAI-compatible connection profile.</p></div>
    </div>
    <form class="form">
      <div class="field"><label>Provider type</label><select><option>OpenAI-compatible</option><option>OpenRouter</option><option>LiteLLM proxy</option></select></div>
      <div class="field"><label>Label</label><input value="OpenRouter - Personal"></div>
      <div class="field"><label>API Base URL</label><input value="https://openrouter.ai/api/v1"></div>
      <div class="field"><label>API key</label><input value="sk-****************"></div>
      <div class="field"><label>Auth</label><select><option>Bearer token</option><option>Custom header</option></select></div>
      <div class="field"><label>Headers</label><textarea rows="5">{}</textarea></div>
      <div class="form-actions"><a class="primary" href="./settings-providers.html">Save provider</a><a class="secondary" href="./settings-providers.html">Cancel</a></div>
    </form>
  `;
}

function models() {
  return `
    <div class="settings-title">
      <div><h1>Models</h1><p>Models are fetched from enabled provider connections when available.</p></div>
    </div>
    <div class="list">
      ${modelRow("Gemini 2.5 Flash", "OpenRouter - Personal")}
      ${modelRow("ChatGPT o4", "OpenRouter - Personal")}
      ${modelRow("gpt-4.1", "OpenAI - Work")}
      ${modelRow("local/coder-small", "LiteLLM Local")}
    </div>
  `;
}

function modelRow(title, provider) {
  return `<div class="list-row"><div><div class="row-title">${title}</div><div class="row-sub">${provider}</div></div><span class="pill">Enabled</span><button class="toggle" aria-label="Enabled"></button></div>`;
}

function configuration() {
  return `
    <div class="settings-title">
      <div><h1>Configuration</h1><p>Configure approval policy and sandbox settings.</p></div>
      <button class="secondary with-icon">${icon("file")}<span>Open config.toml externally</span></button>
    </div>
    <div class="config-grid">
      <section class="config-card"><div><h3>Approval policy</h3><p>Choose when the app asks before high-impact actions.</p></div><button class="secondary">On request</button></section>
      <section class="config-card"><div><h3>Sandbox settings</h3><p>Choose how much agents can do when running commands.</p></div><button class="secondary">Workspace write</button></section>
    </div>
  `;
}

function bindThread() {
  document.querySelectorAll("[data-show-more]").forEach((button) => {
    button.addEventListener("click", () => {
      const message = button.closest("[data-expandable]");
      const expanded = message.classList.toggle("expanded");
      button.textContent = expanded ? "Show less" : "Show more";
    });
  });
}

if (screen === "hub") hub();
else if (screen.startsWith("settings")) settings(screen);
else shell(screen);
