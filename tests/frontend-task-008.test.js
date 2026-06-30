import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-008 Files slice frontend behavior", () => {
  let browser;
  let server;

  before(async () => {
    server = await startFrontendServer();
    browser = await chromium.launch();
  });

  after(async () => {
    await browser?.close();
    await server?.close();
  });

  it("browses trusted project files without repeating the root as the first row", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#file-explorer`);
    await page.getByRole("button", { name: "Files" }).click();

    const rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["frontend", "README.md"]);
  });

  it("opens a trusted file in the accepted editor view, tracks dirty state, saves, and reverts", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByRole("button", { name: "README.md" }).click();

    const editor = page.locator(".tool-panel .code-editor");
    await expectEditorValue(editor, "Initial file contents\nSecond line");
    assert.doesNotMatch(await page.locator(".tool-panel").innerText(), /\bOpened\b/);
    assert.deepEqual(await page.locator(".tool-panel .line-number").evaluateAll((nodes) => nodes.map((node) => node.textContent)), ["1", "2"]);
    assert.equal(await page.getByRole("button", { name: "Save file" }).count(), 0);
    assert.equal(await page.getByRole("button", { name: "Revert file" }).count(), 0);

    await editor.fill("Edited file contents");
    await page.getByText("README.md *").waitFor();
    assert.equal(await page.locator(".tool-panel .file-editor-toolbar").count(), 0);
    assert.doesNotMatch(await page.locator(".tool-panel").innerText(), /Unsaved changes|Saved/);
    await page.evaluate(() => window.__task008EmitMenu("file.saveFile"));
    await page.getByText("README.md *").waitFor({ state: "detached" });
    assert.equal(await page.evaluate(() => window.__task008Files.get("README.md")), "Edited file contents");

    await editor.fill("Unsaved second edit");
    await page.getByText("README.md *").waitFor();
    await page.evaluate(() => window.__task008EmitMenu("file.revertFile"));
    assert.equal(await editor.inputValue(), "Edited file contents");
    assert.doesNotMatch(await page.locator(".tool-panel").innerText(), /Unsaved second edit|Unsaved changes|Saved/);
  });

  it("renders syntax-highlighted editor text without showing the opened status row", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByRole("button", { name: "frontend" }).click();
    await page.getByRole("button", { name: "app.js" }).click();
    await expectEditorValue(page.locator(".tool-panel .code-editor"), "// comment\nimport path from 'node:path';\nconst label = 'frontend';\nconsole.log(label)");

    assert.equal(await page.locator(".tool-panel .file-editor-toolbar").count(), 0);
    assert.doesNotMatch(await page.locator(".tool-panel").innerText(), /\bOpened\b|Unsaved changes|Saved/);
    assert.ok(await page.locator(".tool-panel .syntax-highlight").isVisible());
    assert.ok(await page.locator(".tool-panel .syntax-token.keyword").count() > 0);
    assert.ok(await page.locator(".tool-panel .syntax-token.string").count() > 0);
    assert.ok(await page.locator(".tool-panel .syntax-token.comment").count() > 0);
  });

  it("colors common file-type icons and renders git and ignored row states", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page, { fileMeta: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();

    await expectRowClass(page, "app.js", "file-type-js");
    await expectRowClass(page, "package.json", "file-type-json");
    await expectRowClass(page, "styles.css", "file-type-css");
    await expectRowClass(page, "README.md", "file-type-md");
    await expectRowClass(page, "sync.sh", "file-type-shell");
    await expectRowClass(page, "created.ts", "git-added");
    await expectRowClass(page, "changed.ts", "git-modified");
    await expectRowClass(page, "removed.ts", "git-deleted");
    await expectRowClass(page, "node_modules", "is-ignored");
    await expectRowClass(page, "mcp.md", "is-folder");
    await expectNoRowClass(page, "mcp.md", "is-ignored");
  });

  it("expands and collapses folders in place without replacing the root explorer", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    const promptBox = await page.locator(".composer").boundingBox();
    await page.getByRole("button", { name: "frontend" }).click();
    await page.getByRole("button", { name: "app.js" }).waitFor();

    let rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["frontend", "app.js", "README.md"]);
    assert.equal(await page.locator(".tool-panel .file-row.is-active").count(), 0);
    assert.deepEqual(await page.locator(".composer").boundingBox(), await promptBox);

    await page.getByRole("button", { name: "frontend" }).click();
    rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["frontend", "README.md"]);
  });

  it("keeps the file explorer independently scrollable and marks only the open editor file active", async () => {
    const page = await browser.newPage({ viewport: { width: 1180, height: 520 } });
    await installTask008Tauri(page, { manyFiles: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    const promptBox = await page.locator(".composer").boundingBox();
    await page.getByRole("button", { name: "frontend" }).click();
    await page.getByRole("button", { name: "file-35.js" }).waitFor();

    const listMetrics = await page.locator(".tool-panel .file-list").evaluate((node) => ({
      clientHeight: node.clientHeight,
      scrollHeight: node.scrollHeight
    }));
    assert.ok(listMetrics.scrollHeight > listMetrics.clientHeight, "expanded explorer should scroll inside the Files panel");
    assert.deepEqual(await page.locator(".composer").boundingBox(), promptBox);
    assert.equal(await page.locator(".tool-panel .file-row.is-active").count(), 0);

    await page.getByRole("button", { name: "file-35.js" }).click();
    await expectEditorValue(page.locator(".tool-panel .code-editor"), "console.log('file 35')");
    await page.getByRole("button", { name: "trusted-files-repo" }).click();

    const rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["frontend", "README.md"]);
    assert.equal(await page.locator(".tool-panel .file-row.is-active").count(), 0);
  });

  it("updates native file menu state for an open editor and uses breadcrumbs to collapse explorer scope", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page, { breadcrumbTree: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByRole("button", { name: "docs" }).click();
    assert.ok(await page.getByRole("button", { name: "guide.md" }).isVisible());
    await page.getByRole("button", { name: "src" }).click();
    await page.getByRole("button", { name: "nested" }).click();
    await page.locator("[data-file-path='src/nested/index.ts']").click();

    await page.evaluate(() => { window.__task008MenuStates.length = 0; });
    await page.locator(".tool-panel .code-editor").focus();
    await page.waitForFunction(() => window.__task008MenuStates.at(-1)?.focusState?.fileEditorOpen === true);
    assert.deepEqual(await page.evaluate(() => window.__task008MenuStates.at(-1)?.focusState), {
      editable: true,
      fileEditorOpen: true,
      fileCanSave: true
    });
    await page.locator(".composer-dock .prompt-box").focus();
    await page.waitForFunction(() => window.__task008MenuStates.at(-1)?.focusState?.fileEditorOpen === false);
    assert.deepEqual(await page.evaluate(() => window.__task008MenuStates.at(-1)?.focusState), {
      editable: true,
      fileEditorOpen: false,
      fileCanSave: false
    });

    await page.getByRole("button", { name: "src" }).click();

    let rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["docs", "src", "nested", "index.ts", "README.md"]);
    assert.equal(await page.getByRole("button", { name: "guide.md" }).count(), 0);

    await page.getByRole("button", { name: "nested" }).click();
    await page.locator("[data-file-path='src/nested/index.ts']").click();
    await page.getByRole("button", { name: "trusted-files-repo" }).click();

    rows = await page.locator(".tool-panel .file-row span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(rows, ["docs", "src", "README.md"]);
    assert.equal(await page.getByRole("button", { name: "nested" }).count(), 0);
  });

  it("saves the open editor file from the keyboard shortcut", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByRole("button", { name: "README.md" }).click();

    const editor = page.locator(".tool-panel .code-editor");
    await editor.fill("Saved by keyboard");
    await editor.evaluate((node) => {
      node.dataset.identity = "keyboard-save-editor";
      node.setSelectionRange(8, 8);
    });
    await page.keyboard.press(process.platform === "darwin" ? "Meta+S" : "Control+S");

    await page.waitForFunction(() => window.__task008Files.get("README.md") === "Saved by keyboard");
    await page.getByText("README.md *").waitFor({ state: "detached" });
    assert.equal(await page.locator(".tool-panel .code-editor").evaluate((node) => node.dataset.identity), "keyboard-save-editor");
    assert.equal(await page.locator(".tool-panel .code-editor").evaluate((node) => node.selectionStart), 8);
  });

  it("saves and reverts the focused editor file from the native OS menu command hook", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByRole("button", { name: "README.md" }).click();

    const editor = page.locator(".tool-panel .code-editor");
    await editor.focus();
    await editor.fill("Saved from OS menu");
    await page.evaluate(() => window.__c4osNativeMenuCommand("file.saveFile"));
    await page.waitForFunction(() => window.__task008Files.get("README.md") === "Saved from OS menu");

    await editor.fill("Reverted from OS menu");
    await page.evaluate(() => window.__c4osNativeMenuCommand("file.revertFile"));
    assert.equal(await editor.inputValue(), "Saved from OS menu");
  });

  it("surfaces trusted-root and .git guard rejections before file mutation", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.evaluate(async () => {
      await window.__TAURI__.core.invoke("read_file", { request: { path: "../outside.txt" } }).catch((error) => {
        window.__task008TraversalRead = error.message;
      });
      await window.__TAURI__.core.invoke("save_file", { request: { path: ".git/config", content: "bad" } }).catch((error) => {
        window.__task008GitSave = error.message;
      });
    });

    assert.match(await page.evaluate(() => window.__task008TraversalRead), /outside trusted root/i);
    assert.match(await page.evaluate(() => window.__task008GitSave), /\.git/i);
    assert.equal(await page.evaluate(() => window.__task008Files.has(".git/config")), false);
  });

  it("restores each session-owned Files and File Editor state when switching sessions", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask008Tauri(page, { sessions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByText("alpha.md").waitFor();
    await expectEditorValue(page.locator(".tool-panel .code-editor"), "Alpha persisted");

    await page.locator("[data-session-target='Beta Files']").click();
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByText("beta.md").waitFor();
    await expectEditorValue(page.locator(".tool-panel .code-editor"), "Beta persisted");
  });
});

async function expectEditorValue(editor, expected) {
  await editor.waitFor();
  assert.equal(await editor.inputValue(), expected);
}

async function expectRowClass(page, name, className) {
  const classes = await page.getByRole("button", { name }).getAttribute("class");
  assert.match(classes, new RegExp(`\\b${className}\\b`));
}

async function expectNoRowClass(page, name, className) {
  const classes = await page.getByRole("button", { name }).getAttribute("class");
  assert.doesNotMatch(classes, new RegExp(`\\b${className}\\b`));
}

async function installTask008Tauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const files = new Map([
      ["README.md", "Initial file contents\nSecond line"],
      ["frontend/app.js", "// comment\nimport path from 'node:path';\nconst label = 'frontend';\nconsole.log(label)"],
      ["alpha.md", "Alpha persisted"],
      ["beta.md", "Beta persisted"]
    ]);
    if (nextOptions.manyFiles) {
      for (let index = 1; index <= 40; index += 1) {
        files.set(`frontend/file-${index}.js`, `console.log('file ${index}')`);
      }
    }
    const listeners = new Map();
    const menuStates = [];
    window.__task008Files = files;
    window.__task008MenuStates = menuStates;
    window.__task008EmitMenu = (id) => listeners.get("c4os://native-menu")?.({ payload: id });

    function fileState(path = "") {
      if (nextOptions.breadcrumbTree) {
        const tree = {
          "": [["docs", "folder", "file-explorer", "docs", "0"], ["src", "folder", "file-explorer", "src", "0"], ["README.md", "file", "file-editor", "README.md", "0"]],
          docs: [["guide.md", "file", "file-editor", "guide.md", "1"]],
          src: [["nested", "folder", "file-explorer", "nested", "1"], ["index.ts", "file", "file-editor", "index.ts", "1"]],
          "src/nested": [["index.ts", "file", "file-editor", "index.ts", "2"]]
        };
        if (path === "docs" || path === "src" || path === "src/nested") {
          return {
            roots: tree[path],
            breadcrumbs: ["trusted-files-repo", ...path.split("/")],
            lines: [],
            currentPath: path,
            content: "",
            savedContent: "",
            dirty: false,
            status: "Ready"
          };
        }
        const content = path.endsWith("index.ts") ? "export const ok = true;" : files.get(path) || "";
        return {
          roots: tree[""],
          breadcrumbs: path ? ["trusted-files-repo", ...path.split("/")] : ["trusted-files-repo"],
          lines: content ? content.split("\n") : [],
          currentPath: path,
          content,
          savedContent: content,
          dirty: false,
          status: path ? "Opened" : "Ready"
        };
      }
      if (path === "frontend") {
        const children = Array.from(files.keys())
          .filter((file) => file.startsWith("frontend/"))
          .map((file) => [file.replace("frontend/", ""), "file", "file-editor"]);
        return {
          roots: children,
          breadcrumbs: ["trusted-files-repo", "frontend"],
          lines: [],
          currentPath: "frontend",
          content: "",
          savedContent: "",
          dirty: false,
          status: "Ready"
        };
      }
      const content = path && files.has(path) ? files.get(path) : "";
      const roots = nextOptions.fileMeta
        ? [
            ["app.js", "file", "file-editor", "app.js", "0", "modified"],
            ["package.json", "file", "file-editor", "package.json", "0"],
            ["styles.css", "file", "file-editor", "styles.css", "0"],
            ["README.md", "file", "file-editor", "README.md", "0"],
            ["sync.sh", "file", "file-editor", "sync.sh", "0"],
            ["created.ts", "file", "file-editor", "created.ts", "0", "added"],
            ["changed.ts", "file", "file-editor", "changed.ts", "0", "modified"],
            ["removed.ts", "file", "file-editor", "removed.ts", "0", "deleted"],
            ["mcp.md", "folder", "file-explorer", "mcp.md", "0", "", ""],
            ["node_modules", "folder", "file-explorer", "node_modules", "0", "", "ignored"]
          ]
        : path
          ? [["README.md", "file", "file-editor"], ["frontend", "folder", "file-explorer"]]
          : [["frontend", "folder", "file-explorer"], ["README.md", "file", "file-editor"]];
      return {
        roots,
        breadcrumbs: path ? ["trusted-files-repo", path] : ["trusted-files-repo"],
        lines: content ? content.split("\n") : [],
        currentPath: path,
        content,
        savedContent: content,
        dirty: false,
        status: path ? "Opened" : "Ready"
      };
    }

    const sessions = new Map();
    if (nextOptions.sessions) {
      sessions.set("alpha", {
        id: "alpha",
        project: "trusted-files-repo",
        title: "Alpha Files",
        selectedModel: "model/alpha",
        browser: { url: "http://alpha.local", title: "Alpha", summary: "Alpha" },
        artifacts: [],
        files: { ...fileState("alpha.md"), roots: [["alpha.md", "file", "file-editor"]], content: "Alpha persisted", savedContent: "Alpha persisted", lines: ["Alpha persisted"] },
        terminal: { output: "alpha", title: "Alpha terminal", summary: "Alpha" },
        thread: { user: "Alpha", agent: "Alpha", extra: "", tool: "", run: "" },
        turns: [{ id: "alpha-turn", user: "Alpha", agent: "Alpha", extra: "", tool: "", run: "" }]
      });
      sessions.set("beta", {
        id: "beta",
        project: "trusted-files-repo",
        title: "Beta Files",
        selectedModel: "model/beta",
        browser: { url: "http://beta.local", title: "Beta", summary: "Beta" },
        artifacts: [],
        files: { ...fileState("beta.md"), roots: [["beta.md", "file", "file-editor"]], content: "Beta persisted", savedContent: "Beta persisted", lines: ["Beta persisted"] },
        terminal: { output: "beta", title: "Beta terminal", summary: "Beta" },
        thread: { user: "Beta", agent: "Beta", extra: "", tool: "", run: "" },
        turns: [{ id: "beta-turn", user: "Beta", agent: "Beta", extra: "", tool: "", run: "" }]
      });
    }

    function workspacePayload(activeId = nextOptions.sessions ? "alpha" : "") {
      const active = sessions.get(activeId);
      return {
        workspace: { project: "trusted-files-repo", session: active?.title || "Files review", branch: "main", model: "model/alpha", sessionId: active?.id || "" },
        projects: [{ name: "trusted-files-repo", sessions: nextOptions.sessions ? Array.from(sessions.values()).map((session) => ({ id: session.id, label: session.title })) : ["Files review"] }],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: active?.browser || { url: "http://files.local", title: "Files", summary: "Files" },
        artifacts: active?.artifacts || [],
        files: active?.files || fileState(),
        terminal: active?.terminal || { output: "", title: "Terminal", summary: "" },
        thread: active?.thread || { user: "Files", agent: "Files ready", extra: "", tool: "", run: "" },
        sessions: Array.from(sessions.values())
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, payload = {}) => {
          const request = payload.request || {};
          if (command === "load_workspace") return workspacePayload();
          if (command === "load_session") return sessions.get(request.sessionId);
          if (command === "read_file") {
            if (request.path.includes("..")) throw new Error("Path is outside trusted root");
            if (request.path.startsWith(".git")) throw new Error("Casual .git mutation is rejected");
            return fileState(request.path);
          }
          if (command === "save_file") {
            if (request.path.includes("..")) throw new Error("Path is outside trusted root");
            if (request.path.startsWith(".git")) throw new Error("Casual .git mutation is rejected");
            files.set(request.path, request.content);
            return { ...fileState(request.path), saved: true, status: "Saved" };
          }
          if (command === "native_menu_state") {
            menuStates.push(payload);
            return {
              commands: {
                "file.saveFile": { enabled: Boolean(payload.focusState?.fileEditorOpen) },
                "file.revertFile": { enabled: Boolean(payload.focusState?.fileEditorOpen) }
              }
            };
          }
          return { ok: true };
        }
      },
      event: {
        listen: async (event, handler) => {
          listeners.set(event, handler);
          return () => listeners.delete(event);
        }
      }
    };
  }, options);
}
