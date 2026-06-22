import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-004 first real user-flow frontend activation", () => {
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

  it("uses the existing Open Folder action to enter the accepted shell with real workspace state", async () => {
    await page.addInitScript(() => {
      const mockWorkspace = {
        workspace: {
          project: "Mock Workspace Alpha",
          session: "Mock integration run",
          branch: "mock/task-002",
          model: "mock-fast-coder"
        },
        projects: [{ name: "Mock Workspace Alpha", sessions: ["Mock integration run"] }],
        providers: [],
        models: [{ label: "mock-fast-coder", provider: "Mock OpenRouter", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        files: { roots: [], breadcrumbs: ["Mock Workspace Alpha"], lines: [] },
        terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
        thread: { user: "mock", agent: "mock", extra: "mock", tool: "mock", run: "mock" }
      };
      const realPayload = {
        ...mockWorkspace,
        workspace: {
          project: "real-product-repo",
          session: "",
          branch: "main",
          model: "mock-fast-coder"
        },
        projects: [{ name: "real-product-repo", sessions: [] }],
        files: { roots: [], breadcrumbs: ["real-product-repo"], lines: [] },
        thread: {
          user: "Open trusted local workspace.",
          agent: "Workspace 'real-product-repo' is active from a real local descriptor.",
          extra: "Only workspace identity is real.",
          tool: "Workspace descriptor loaded",
          run: "Ready for first session"
        }
      };
      window.__TASK_004_CALLS__ = [];
      window.__TAURI__ = {
        core: {
          invoke: async (command, payload) => {
            window.__TASK_004_CALLS__.push({ command, payload });
            if (command === "load_workspace") return mockWorkspace;
            if (command === "open_workspace") {
              return {
                path: "/tmp/real-product-repo",
                trusted: true,
                workspace: realPayload.workspace,
                descriptor: {
                  schemaVersion: 1,
                  id: "c4os-ws-real",
                  name: "real-product-repo",
                  rootPath: "/tmp/real-product-repo",
                  trusted: true,
                  createdAt: 1,
                  updatedAt: 1
                },
                payload: realPayload
              };
            }
            return { run: "mock", agent: "mock" };
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByText("Mock Workspace Alpha").waitFor();
    const controlsBefore = await visibleControlNames(page);

    await page.getByRole("button", { name: "Open Folder" }).click();
    await page.waitForURL(/#new-session$/);
    await page.locator(".topbar strong", { hasText: "real-product-repo" }).waitFor();
    await page.getByText("What should we build in real-product-repo?").waitFor();

    assert.equal(await page.locator(".app-shell").count(), 1);
    assert.equal(await page.locator(".start-view").count(), 0);
    assert.equal(await page.getByText("Mock Workspace Alpha").count(), 0);
    assert.equal(await page.locator(".session-row").count(), 0);
    assert.deepEqual(
      (await page.evaluate(() => window.__TASK_004_CALLS__.map((call) => call.command))).slice(0, 2),
      ["load_workspace", "open_workspace"]
    );
    assert.equal(controlsBefore.includes("Open Folder"), true);
  });

  it("keeps first-session prompt styling stable and creates a visible chat session on send", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "style-review-repo",
          session: "",
          branch: "main",
          model: "mock-fast-coder"
        },
        projects: [{ name: "style-review-repo", sessions: [] }],
        providers: [],
        models: [{ label: "mock-fast-coder", provider: "Mock OpenRouter", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        files: { roots: [], breadcrumbs: ["style-review-repo"], lines: [] },
        terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
        thread: {
          user: "Open trusted local workspace.",
          agent: "Workspace is ready.",
          extra: "Only workspace identity is real.",
          tool: "Workspace descriptor loaded",
          run: "Ready for first session"
        }
      };
      window.__TAURI__ = {
        core: {
          invoke: async (command, payload) => {
            if (command === "load_workspace" || command === "open_workspace") {
              return command === "open_workspace"
                ? { path: "/tmp/style-review-repo", trusted: true, workspace: shellPayload.workspace, payload: shellPayload }
                : shellPayload;
            }
            if (command === "create_session") {
              return { id: "session-review-first-prompt", project: "style-review-repo", status: "mock-created" };
            }
            if (command === "send_prompt") {
              await new Promise((resolve) => setTimeout(resolve, 120));
              return {
                agent: "Mock agent completed the requested transition.",
                prompt: payload.prompt,
                run: "Mock agent completed the requested transition."
              };
            }
            return { ok: true };
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByRole("button", { name: "Open Folder" }).click();
    await page.waitForURL(/#new-session$/);

    const prompt = page.locator(".empty-workspace .prompt-box");
    await prompt.click();
    const promptFocusStyle = await prompt.evaluate((node) => {
      const style = getComputedStyle(node);
      return { outlineStyle: style.outlineStyle, outlineWidth: style.outlineWidth };
    });
    assert.equal(promptFocusStyle.outlineStyle, "none");
    assert.equal(promptFocusStyle.outlineWidth, "0px");

    const sendButton = page.getByRole("button", { name: "Send Prompt" });
    const beforeHover = await sendButton.evaluate((node) => getComputedStyle(node).backgroundColor);
    await sendButton.hover();
    const afterHover = await sendButton.evaluate((node) => getComputedStyle(node).backgroundColor);
    assert.equal(afterHover, beforeHover);

    await prompt.fill("Build the onboarding test harness");
    await sendButton.click();

    await page.getByText("Connector is processing the request.").waitFor();
    await page.getByRole("button", { name: /Working for|Worked for/ }).waitFor();
    await page.waitForURL(/#chat-session$/);
    await page.getByRole("link", { name: /Build the onboarding test harness/ }).waitFor();
    await page.getByText("Mock agent completed the requested transition.").waitFor();
  });
});

async function visibleControlNames(page) {
  return page.evaluate(() => {
    return Array.from(document.querySelectorAll("button, a"))
      .filter((node) => {
        const style = getComputedStyle(node);
        return style.display !== "none" && style.visibility !== "hidden" && !node.hidden;
      })
      .map((node) => node.getAttribute("aria-label") || node.textContent.trim())
      .filter(Boolean);
  });
}
