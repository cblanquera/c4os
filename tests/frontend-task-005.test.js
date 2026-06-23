import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-005 OpenRouter streaming chat experience", () => {
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

  it("renders streamed reasoning activity before the final assistant response", async () => {
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "openrouter-review-repo",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "openrouter-review-repo", sessions: [] }],
        providers: [{ label: "OpenRouter Review", endpoint: "https://openrouter.ai/api/v1", status: "API key from runtime" }],
        models: [{ label: "google/gemini-2.5-flash-lite", provider: "OpenRouter Review", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["openrouter-review-repo"], lines: [] },
        terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
        thread: {
          user: "Open trusted local workspace.",
          agent: "Workspace is ready.",
          extra: "Only workspace identity is real.",
          tool: "Workspace descriptor loaded",
          run: "Ready for first session"
        }
      };
      const listeners = new Map();
      window.__TASK_005_CALLS__ = [];
      window.__TAURI__ = {
        core: {
          invoke: async (command, payload) => {
            window.__TASK_005_CALLS__.push({ command, payload });
            if (command === "load_workspace" || command === "open_workspace") {
              return command === "open_workspace"
                ? { path: "/tmp/openrouter-review-repo", trusted: true, workspace: shellPayload.workspace, payload: shellPayload }
                : shellPayload;
            }
            if (command === "create_session") {
              return { id: "session-openrouter-review", project: "openrouter-review-repo", status: "created" };
            }
            if (command === "send_prompt") {
              await new Promise((resolve) => setTimeout(resolve, 20));
              listeners.get("c4os://runtime-event")?.({
                payload: {
                  kind: "reasoning",
                  text: "Evaluating the project request before answering.",
                  sequence: 1
                }
              });
              await new Promise((resolve) => setTimeout(resolve, 20));
              listeners.get("c4os://runtime-event")?.({
                payload: {
                  kind: "activity",
                  text: "Streaming OpenRouter response.",
                  sequence: 2
                }
              });
              await new Promise((resolve) => setTimeout(resolve, 20));
              return {
                agent: "The **real** OpenRouter-backed answer is ready.",
                prompt: payload.prompt,
                run: "OpenRouter stream complete"
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
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByRole("button", { name: "Open Folder" }).click();
    await page.waitForURL(/#new-session$/);

    await page.locator(".prompt-box").fill("Explain the next implementation slice");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.getByText("Evaluating the project request before answering.").waitFor();
    await page.getByText("Streaming OpenRouter response.").waitFor();
    await page.getByText("The real OpenRouter-backed answer is ready.").waitFor();
    await page.locator(".message.agent strong", { hasText: "real" }).waitFor();
    assert.equal(await page.getByText("Agent Run").count(), 0);
    assert.equal(await page.getByText("Tool Call").count(), 0);
    assert.equal(await page.getByRole("button", { name: /Worked for|Working for/ }).count() > 0, true);
    await page.getByRole("button", { name: /Worked for/ }).click();
    assert.deepEqual(
      await page.evaluate(() => {
        const text = document.querySelector(".thread-list").textContent;
        return [
          text.indexOf("Explain the next implementation slice"),
          text.indexOf("Evaluating the project request before answering."),
          text.indexOf("Streaming OpenRouter response."),
          text.indexOf("The real OpenRouter-backed answer is ready.")
        ].map((index) => index >= 0);
      }),
      [true, true, true, true]
    );
    assert.equal(
      await page.evaluate(() => {
        const text = document.querySelector(".thread-list").textContent;
        return text.indexOf("Evaluating the project request before answering.") < text.indexOf("The real OpenRouter-backed answer is ready.");
      }),
      true
    );

    await page.locator(".composer-dock .prompt-box").fill("Explain a second calculation");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Explain a second calculation").waitFor();
    assert.equal(await page.locator(".message.user").getByText("Explain the next implementation slice").count(), 1);
    assert.equal(await page.locator(".message.user").getByText("Explain a second calculation").count(), 1);
    await page.locator(".message.agent").getByText("The real OpenRouter-backed answer is ready.").nth(1).waitFor();
    assert.equal(await page.locator(".message.agent").getByText("The real OpenRouter-backed answer is ready.").count(), 2);
    const firstWorkLog = page.locator(".work-log").first();
    if (await firstWorkLog.locator(".work-log-body").count() > 0) {
      await firstWorkLog.getByRole("button", { name: /Worked for/ }).click();
    }
    await firstWorkLog.getByRole("button", { name: /Worked for/ }).click();
    await firstWorkLog.getByText("Streaming OpenRouter response.").waitFor();
    assert.deepEqual(
      await page.evaluate(() => window.__TASK_005_CALLS__.map((call) => call.command).slice(0, 5)),
      ["load_workspace", "open_workspace", "create_session", "send_prompt", "send_prompt"]
    );
  });

  it("replays returned runtime events when live desktop event delivery is unavailable", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "openrouter-review-repo",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "openrouter-review-repo", sessions: [] }],
        providers: [{ label: "OpenRouter Review", endpoint: "https://openrouter.ai/api/v1", status: "API key from runtime" }],
        models: [{ label: "google/gemini-2.5-flash-lite", provider: "OpenRouter Review", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["openrouter-review-repo"], lines: [] },
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
                ? { path: "/tmp/openrouter-review-repo", trusted: true, workspace: shellPayload.workspace, payload: shellPayload }
                : shellPayload;
            }
            if (command === "create_session") {
              return { id: "session-openrouter-review", project: "openrouter-review-repo", status: "created" };
            }
            if (command === "send_prompt") {
              return {
                agent: "The final fallback response is visible.",
                prompt: payload.prompt,
                run: "OpenRouter stream complete",
                events: [
                  { kind: "reasoning", text: "Payload reasoning fallback.", sequence: 1 },
                  { kind: "activity", text: "Payload activity fallback.", sequence: 2 }
                ]
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

    await page.locator(".prompt-box").fill("Explain fallback event replay");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.getByRole("button", { name: /Worked for|Working for/ }).click();
    await page.getByText("Payload reasoning fallback.").waitFor();
    await page.getByText("OpenRouter stream complete").waitFor();
    await page.getByText("The final fallback response is visible.").waitFor();
  });

  it("still renders returned events when desktop event subscription fails", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "openrouter-review-repo",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "openrouter-review-repo", sessions: [] }],
        providers: [{ label: "OpenRouter Review", endpoint: "https://openrouter.ai/api/v1", status: "API key from runtime" }],
        models: [{ label: "google/gemini-2.5-flash-lite", provider: "OpenRouter Review", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["openrouter-review-repo"], lines: [] },
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
                ? { path: "/tmp/openrouter-review-repo", trusted: true, workspace: shellPayload.workspace, payload: shellPayload }
                : shellPayload;
            }
            if (command === "create_session") {
              return { id: "session-openrouter-review", project: "openrouter-review-repo", status: "created" };
            }
            if (command === "send_prompt") {
              return {
                agent: "The command result survived a listener failure.",
                prompt: payload.prompt,
                run: "OpenRouter stream complete",
                events: [
                  { kind: "activity", text: "Returned activity after listener failure.", sequence: 1 }
                ]
              };
            }
            return { ok: true };
          }
        },
        event: {
          listen: async () => {
            throw new Error("event listen denied");
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByRole("button", { name: "Open Folder" }).click();
    await page.waitForURL(/#new-session$/);

    await page.locator(".prompt-box").fill("Explain listener fallback");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.getByText("The command result survived a listener failure.").waitFor();
    assert.equal(await page.getByText("The connector run did not complete.").count(), 0);
  });

  it("shows actionable connector errors instead of blank activity cards", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "openrouter-review-repo",
          session: "",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "openrouter-review-repo", sessions: [] }],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["openrouter-review-repo"], lines: [] },
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
          invoke: async (command) => {
            if (command === "load_workspace" || command === "open_workspace") {
              return command === "open_workspace"
                ? { path: "/tmp/openrouter-review-repo", trusted: true, workspace: shellPayload.workspace, payload: shellPayload }
                : shellPayload;
            }
            if (command === "create_session") {
              return { id: "session-openrouter-review", project: "openrouter-review-repo", status: "created" };
            }
            if (command === "send_prompt") {
              throw {};
            }
            return { ok: true };
          }
        }
      };
    });

    await page.goto(`${server.origin}/#app-start`);
    await page.getByRole("button", { name: "Open Folder" }).click();
    await page.waitForURL(/#new-session$/);

    await page.locator(".prompt-box").fill("Explain blank failure handling");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.getByText("The connector run did not complete.").waitFor();
    await page.locator(".work-log").last().getByText("The connector failed before returning error details. Check the desktop app terminal for the backend error.").waitFor();
  });

  it("keeps the chat session active when switching right-panel tools", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "openrouter-review-repo",
          session: "Panel switching review",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "openrouter-review-repo", sessions: ["Panel switching review"] }],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [["frontend", "folder", "file-explorer"]], breadcrumbs: ["openrouter-review-repo"], lines: ["# mock"] },
        terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
        thread: {
          user: "Keep me in chat.",
          agent: "The chat session should stay active.",
          extra: "Tool tabs only change the right panel.",
          tool: "Workspace descriptor loaded",
          run: "Ready"
        }
      };
      window.__TAURI__ = {
        core: {
          invoke: async (command) => {
            if (command === "load_workspace") return shellPayload;
            return { ok: true };
          }
        }
      };
    });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByText("The chat session should stay active.").waitFor();
    await page.evaluate(() => {
      document.querySelector(".thread-list").dataset.stabilityProbe = "kept";
    });

    await page.getByRole("button", { name: "Files" }).click();
    await page.getByText("frontend").waitFor();
    assert.match(page.url(), /#chat-session$/);
    assert.equal(await page.locator("[data-screen='chat-session']").count(), 1);
    await page.getByText("The chat session should stay active.").waitFor();
    assert.equal(await page.locator(".thread-list").evaluate((node) => node.dataset.stabilityProbe), "kept");

    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator(".terminal-output", { hasText: "mock terminal" }).waitFor();
    assert.match(page.url(), /#chat-session$/);
    assert.equal(await page.locator("[data-screen='chat-session']").count(), 1);
    assert.equal(await page.locator(".thread-list").evaluate((node) => node.dataset.stabilityProbe), "kept");

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: "Files" }).click();
    await page.getByText("frontend").waitFor();
    await page.goto(`${server.origin}/#chat-session`);
    await page.locator(".terminal-output", { hasText: "mock terminal" }).waitFor();
  });
});
