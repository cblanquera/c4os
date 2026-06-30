import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-013A desktop QA bootstrap frontend routing", () => {
  let browser;
  let page;
  let server;

  before(async () => {
    server = await startFrontendServer();
    browser = await chromium.launch();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
  });

  after(async () => {
    await browser?.close();
    await server?.close();
  });

  it("routes a bootstrapped trusted workspace from app start into new session", async () => {
    await page.addInitScript(() => {
      const payload = {
        workspace: {
          project: "qa-bootstrap-repo",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite",
          sessionId: ""
        },
        projects: [{
          id: "c4os-ws-qa-bootstrap",
          name: "qa-bootstrap-repo",
          rootPath: "/tmp/qa-bootstrap-repo",
          sessions: []
        }],
        sessions: [],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "", title: "", summary: "", previewMode: "public" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["qa-bootstrap-repo"], lines: [] },
        terminal: { output: "", title: "Terminal", summary: "", userTerminal: {}, agentTerminal: {}, actions: [] },
        thread: {
          user: "Open trusted local workspace.",
          agent: "Workspace 'qa-bootstrap-repo' is active from a real local descriptor.",
          extra: "Bootstrapped for QA.",
          tool: "Workspace descriptor loaded",
          run: "Ready for first session"
        }
      };
      window.__TASK_013A_CALLS__ = [];
      window.__TAURI__ = {
        core: {
          invoke: async (command, request) => {
            window.__TASK_013A_CALLS__.push({ command, request });
            if (command === "load_workspace") return payload;
            return payload;
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.waitForURL(/#new-session$/);
    await page.locator(".topbar strong", { hasText: "qa-bootstrap-repo" }).waitFor();
    await page.getByText("What should we build in qa-bootstrap-repo?").waitFor();

    assert.equal(await page.locator(".start-view").count(), 0);
    assert.deepEqual(
      await page.evaluate(() => window.__TASK_013A_CALLS__.map((call) => call.command)),
      ["load_workspace"]
    );
  });

  it("keeps normal human launches on app start when no trusted root is bootstrapped", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const payload = {
        workspace: {
          project: "Mock Workspace Alpha",
          session: "Mock integration run",
          branch: "mock/task-002",
          model: "",
          sessionId: ""
        },
        projects: [{
          id: "mock-workspace-alpha",
          name: "Mock Workspace Alpha",
          rootPath: "",
          sessions: [{ id: "mock-integration-run", label: "Mock integration run" }]
        }],
        sessions: [],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "", title: "", summary: "", previewMode: "public" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["Mock Workspace Alpha"], lines: [] },
        terminal: { output: "", title: "Terminal", summary: "", userTerminal: {}, agentTerminal: {}, actions: [] },
        thread: { user: "mock", agent: "mock", extra: "mock", tool: "mock", run: "mock" }
      };
      window.__TAURI__ = {
        core: {
          invoke: async (command) => {
            if (command === "load_workspace") return payload;
            return payload;
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.waitForTimeout(100);

    assert.equal(new URL(page.url()).hash, "#app-start");
    assert.equal(await page.locator(".start-view").count(), 1);
    await page.getByRole("heading", { name: "Open a folder to start working" }).waitFor();
  });

  it("shows real recent workspaces on app start without auto-opening one", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const recentPayload = {
        workspace: {
          project: "",
          session: "",
          branch: "",
          model: "google/gemini-2.5-flash-lite",
          sessionId: ""
        },
        projects: [
          {
            id: "c4os-file-workspace",
            name: "workspace",
            rootPath: "/tmp/recent-one",
            workspaceFilePath: "/tmp/workspace.c4os.json",
            sessions: []
          },
          {
            id: "c4os-ws-recent-two",
            name: "recent-two",
            rootPath: "/tmp/recent-two",
            sessions: []
          }
        ],
        sessions: [],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "", title: "", summary: "", previewMode: "public" },
        artifacts: [],
        files: { roots: [], breadcrumbs: [], lines: [] },
        terminal: { output: "", title: "Terminal", summary: "", userTerminal: {}, agentTerminal: {}, actions: [] },
        thread: { user: "", agent: "", extra: "", tool: "", run: "" }
      };
      const openedPayload = {
        ...recentPayload,
        workspace: {
          project: "project-a",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite",
          sessionId: ""
        }
      };
      window.__TASK_013A_CALLS__ = [];
      window.__TAURI__ = {
        core: {
          invoke: async (command, request) => {
            window.__TASK_013A_CALLS__.push({ command, request });
            if (command === "open_workspace_file") return { payload: openedPayload };
            return recentPayload;
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.waitForTimeout(100);

    assert.equal(new URL(page.url()).hash, "#app-start");
    assert.equal(await page.locator(".recent-panel").count(), 1);
    assert.equal(await page.locator(".workspace-row").count(), 2);
    await page.getByRole("heading", { name: "Recent Workspaces" }).waitFor();
    await page.getByRole("link", { name: /workspace/ }).waitFor();
    await page.getByText("recent-two").waitFor();

    await page.getByRole("link", { name: /workspace/ }).click();
    await page.waitForURL(/#new-session$/);
    await page.locator(".topbar strong", { hasText: "project-a" }).waitFor();
    assert.deepEqual(
      await page.evaluate(() => window.__TASK_013A_CALLS__.filter((call) => call.command !== "native_menu_state")),
      [
        { command: "load_workspace", request: undefined },
        {
          command: "open_workspace_file",
          request: { request: { path: "/tmp/workspace.c4os.json" } }
        }
      ]
    );
  });

  it("does not render fallback fake recent workspaces when no connector is available", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByRole("heading", { name: "Open a folder to start working" }).waitFor();

    assert.equal(await page.getByText("Project Alpha").count(), 0);
    assert.equal(await page.getByText("Agent Lab").count(), 0);
    assert.equal(await page.getByText("Docs Workbench").count(), 0);
    assert.equal(await page.locator(".recent-panel").count(), 0);
    assert.equal(await page.locator(".workspace-row").count(), 0);

    const startCopyCenterDelta = await page.evaluate(() => {
      const startCopy = document.querySelector(".start-copy");
      if (!startCopy) return Number.POSITIVE_INFINITY;
      const rect = startCopy.getBoundingClientRect();
      const copyCenter = rect.left + rect.width / 2;
      return Math.abs(copyCenter - window.innerWidth / 2);
    });
    assert.ok(
      startCopyCenterDelta < 80,
      `expected empty start content to be centered, got ${startCopyCenterDelta}px offset`
    );
  });

  it("routes OS menu workspace open and save commands to workspace file actions", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const listeners = new Map();
      const payload = {
        workspace: {
          project: "menu-workspace",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite",
          sessionId: ""
        },
        projects: [{
          id: "c4os-ws-menu",
          name: "menu-workspace",
          rootPath: "/tmp/menu-workspace",
          sessions: []
        }],
        sessions: [],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "", title: "", summary: "", previewMode: "public" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["menu-workspace"], lines: [] },
        terminal: { output: "", title: "Terminal", summary: "", userTerminal: {}, agentTerminal: {}, actions: [] },
        thread: {
          user: "Open trusted local workspace.",
          agent: "Workspace 'menu-workspace' is active from a real local descriptor.",
          extra: "Loaded from the OS menu.",
          tool: "Workspace descriptor loaded",
          run: "Ready for first session"
        }
      };
      window.__TASK_013A_CALLS__ = [];
      window.__TASK_013A_EMIT_MENU__ = (id) => listeners.get("c4os://native-menu")?.({ payload: id });
      window.__TAURI__ = {
        core: {
          invoke: async (command, request) => {
            window.__TASK_013A_CALLS__.push({ command, request });
            if (command === "open_workspace_file") return { payload };
            if (command === "save_workspace_file") return { rootPath: "/tmp/menu-workspace" };
            return payload;
          }
        },
        event: {
          listen: async (name, callback) => {
            listeners.set(name, callback);
            return () => listeners.delete(name);
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.waitForTimeout(100);

    await page.evaluate(() => window.__TASK_013A_EMIT_MENU__("file.openWorkspace"));
    await page.waitForURL(/#new-session$/);
    await page.locator(".topbar strong", { hasText: "menu-workspace" }).waitFor();

    await page.evaluate(() => window.__TASK_013A_EMIT_MENU__("file.saveWorkspace"));
    await page.waitForFunction(() =>
      window.__TASK_013A_CALLS__.some((call) => call.command === "save_workspace_file")
    );

    assert.deepEqual(
      await page.evaluate(() => window.__TASK_013A_CALLS__.map((call) => call.command)),
      ["load_workspace", "open_workspace_file", "save_workspace_file"]
    );
  });
});
