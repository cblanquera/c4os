import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-005A scoped frontend state and DOM updates", () => {
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

  it("changes right-panel tabs without replacing the chat transcript", async () => {
    await page.goto(`${server.origin}/#chat-session`);

    assert.equal(await markNode(".thread-list", "task005aTranscript"), true);
    await page.getByRole("button", { name: "Files" }).click();

    assert.equal(
      await page.evaluate(() => document.querySelector(".thread-list").__task005aTranscript === true),
      true
    );
  });

  it("expands one work log without replacing message bubbles", async () => {
    await page.goto(`${server.origin}/#chat-session`);

    assert.equal(await markNode(".message.agent", "task005aAgentBubble"), true);
    await page.getByRole("button", { name: /Worked for/ }).first().click();

    assert.equal(
      await page.evaluate(() => document.querySelector(".message.agent").__task005aAgentBubble === true),
      true
    );
  });

  it("streams progress into only the active turn", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.addInitScript(() => {
      const shellPayload = {
        workspace: {
          project: "scoped-update-repo",
          session: "Scoped update review",
          branch: "main",
          model: "google/gemini-2.5-flash-lite"
        },
        projects: [{ name: "scoped-update-repo", sessions: ["Scoped update review"] }],
        providers: [{ label: "OpenRouter Review", endpoint: "https://openrouter.ai/api/v1", status: "API key from runtime" }],
        models: [{ label: "google/gemini-2.5-flash-lite", provider: "OpenRouter Review", active: true }],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: { url: "http://127.0.0.1/mock", title: "Mock rendered page", summary: "Mock Browser state" },
        artifacts: [],
        files: { roots: [], breadcrumbs: ["scoped-update-repo"], lines: [] },
        terminal: { output: "mock terminal", title: "Mock terminal", summary: "Mock output" },
        thread: {
          user: "Existing accepted turn.",
          agent: "Existing assistant answer.",
          extra: "Existing extra detail.",
          tool: "Previous reasoning",
          run: "Previous run complete"
        }
      };
      const listeners = new Map();
      window.__TAURI__ = {
        core: {
          invoke: async (command, payload) => {
            if (command === "load_workspace") return shellPayload;
            if (command === "send_prompt") {
              await new Promise((resolve) => setTimeout(resolve, 20));
              listeners.get("c4os://runtime-event")?.({
                payload: {
                  kind: "reasoning",
                  text: "Scoped reasoning update.",
                  sequence: 1
                }
              });
              await new Promise((resolve) => setTimeout(resolve, 20));
              return {
                agent: "Scoped final response.",
                prompt: payload.prompt,
                run: "Scoped run complete"
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

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByText("Existing assistant answer.").waitFor();
    assert.equal(await markNode(".message.agent", "task005aPriorAgentBubble"), true);

    await page.locator(".composer-dock .prompt-box").fill("Stream a scoped update");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Connector is processing the request.").waitFor();
    assert.equal(await markNode(".message.agent:last-of-type", "task005aActiveAgentBubble"), true);

    await page.getByText("Scoped reasoning update.").waitFor();
    assert.equal(
      await page.evaluate(() => document.querySelector(".message.agent").__task005aPriorAgentBubble === true),
      true
    );
    assert.equal(
      await page.evaluate(() => document.querySelectorAll(".message.agent").item(1).__task005aActiveAgentBubble === true),
      true
    );
  });

  it("opens file rows inside the right panel without changing the active chat screen", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Files" }).click();

    assert.equal(await markNode(".thread-list", "task005aFileClickTranscript"), true);
    await page.locator(".tool-panel .file-row.is-file").first().click();

    assert.deepEqual(
      await page.evaluate(() => ({
        hash: location.hash,
        screen: document.querySelector("[data-screen]")?.dataset.screen,
        transcriptKept: document.querySelector(".thread-list")?.__task005aFileClickTranscript === true,
        editorVisible: document.querySelector(".tool-panel .editor-tool") !== null
      })),
      {
        hash: "#chat-session",
        screen: "chat-session",
        transcriptKept: true,
        editorVisible: true
      }
    );
  });

  it("keeps file-panel buttons visually styled like panel rows and breadcrumbs", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#file-explorer`);

    const fileRowStyles = await page.evaluate(() => {
      const row = document.querySelector(".tool-panel .file-row");
      const style = getComputedStyle(row);
      return {
        appearance: style.appearance,
        background: style.backgroundColor,
        borderStyle: style.borderStyle,
        display: style.display
      };
    });
    assert.deepEqual(fileRowStyles, {
      appearance: "none",
      background: "rgba(0, 0, 0, 0)",
      borderStyle: "none",
      display: "grid"
    });

    await page.locator(".tool-panel .file-row.is-file").first().click();
    const breadcrumbStyles = await page.evaluate(() => {
      const crumb = document.querySelector(".tool-panel .breadcrumbs button");
      const style = getComputedStyle(crumb);
      return {
        appearance: style.appearance,
        background: style.backgroundColor,
        borderStyle: style.borderStyle,
        display: style.display
      };
    });
    assert.deepEqual(breadcrumbStyles, {
      appearance: "none",
      background: "rgba(0, 0, 0, 0)",
      borderStyle: "none",
      display: "flex"
    });
  });

  it("opens composer model/provider pickers without replacing the workspace shell", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#new-session`);

    assert.equal(await markNode(".app-shell", "task005aPickerShell"), true);
    assert.equal(await markNode(".composer", "task005aPickerComposer"), true);
    await page.getByRole("button", { name: /Model:/ }).click();

    assert.deepEqual(
      await page.evaluate(() => ({
        hash: location.hash,
        screen: document.querySelector("[data-screen]")?.dataset.screen,
        shellKept: document.querySelector(".app-shell")?.__task005aPickerShell === true,
        composerKept: document.querySelector(".composer")?.__task005aPickerComposer === true,
        modelPickerVisible: document.querySelector("[data-local-picker='models']") !== null
      })),
      {
        hash: "#new-session",
        screen: "new-session",
        shellKept: true,
        composerKept: true,
        modelPickerVisible: true
      }
    );

    await page.getByRole("button", { name: "Back to providers" }).click();
    assert.equal(
      await page.evaluate(() => document.querySelector("[data-local-picker='providers']") !== null),
      true
    );
  });

  it("switches sessions from the sidebar without replacing the shell", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#chat-session`);

    assert.equal(await markNode(".app-shell", "task005aSessionShell"), true);
    await page.getByText("UI handoff review").click();

    assert.deepEqual(
      await page.evaluate(() => ({
        hash: location.hash,
        screen: document.querySelector("[data-screen]")?.dataset.screen,
        shellKept: document.querySelector(".app-shell")?.__task005aSessionShell === true,
        topbarTitle: document.querySelector(".topbar strong")?.textContent,
        activeSession: document.querySelector(".session-row.is-active")?.textContent
      })),
      {
        hash: "#chat-session",
        screen: "chat-session",
        shellKept: true,
        topbarTitle: "UI handoff review",
        activeSession: "UI handoff review"
      }
    );
  });

  it("keeps shell collapse state across route renders", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#chat-session`);

    await page.getByRole("button", { name: "Collapse left panel" }).click();
    await page.evaluate(() => {
      window.location.hash = "settings-providers";
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    });
    await page.getByText("Back to app").click();

    assert.equal(
      await page.evaluate(() => document.querySelector(".app-shell")?.classList.contains("is-left-collapsed")),
      true
    );
  });

  it("ignores accidental route links inside scoped tool panels", async () => {
    await page.close();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await page.goto(`${server.origin}/#chat-session`);

    await page.evaluate(() => {
      const panel = document.querySelector(".tool-panel");
      const link = document.createElement("a");
      link.href = "#file-editor";
      link.dataset.link = "file-editor";
      link.textContent = "Scoped accidental route";
      panel.append(link);
    });
    await page.getByText("Scoped accidental route").click();

    assert.deepEqual(
      await page.evaluate(() => ({
        hash: location.hash,
        screen: document.querySelector("[data-screen]")?.dataset.screen
      })),
      {
        hash: "#chat-session",
        screen: "chat-session"
      }
    );
  });

  async function markNode(selector, property) {
    return page.evaluate(({ selector, property }) => {
      const node = document.querySelector(selector);
      if (!node) return false;
      node[`__${property}`] = true;
      return true;
    }, { selector, property });
  }
});
