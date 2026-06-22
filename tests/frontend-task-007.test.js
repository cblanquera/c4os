import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-007 persistent session frontend behavior", () => {
  let browser;
  let page;
  let server;

  before(async () => {
    server = await startFrontendServer();
    browser = await chromium.launch();
  });

  after(async () => {
    await browser?.close();
    await server?.close();
  });

  it("targets new-session creation and prompt append to the correct C4OS session records", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask007Tauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.locator("[data-local-model='model/alpha']").click();
    await page.locator(".prompt-box").fill("First persistent chat");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Reply for First persistent chat").waitFor();

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: /Model:/ }).click();
    await page.locator("[data-local-model='model/beta']").click();
    await page.locator(".prompt-box").fill("Second persistent chat");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Reply for Second persistent chat").waitFor();

    await page.locator(".composer-dock .prompt-box").fill("Append to second chat");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Reply for Append to second chat").waitFor();

    assert.deepEqual(await page.evaluate(() => window.__task007Calls), [
      { command: "create_session", session: undefined, label: "First persistent chat", model: "model/alpha" },
      { command: "send_prompt", session: "c4os-session-first-persistent-chat", prompt: "First persistent chat", model: "model/alpha" },
      { command: "create_session", session: undefined, label: "Second persistent chat", model: "model/beta" },
      { command: "send_prompt", session: "c4os-session-second-persistent-chat", prompt: "Second persistent chat", model: "model/beta" },
      { command: "send_prompt", session: "c4os-session-second-persistent-chat", prompt: "Append to second chat", model: "model/beta" }
    ]);
  });

  it("restores session-owned model, thread, Browser, Terminal, Files, and File Editor state when switching sessions", async () => {
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask007Tauri(page, { resumed: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByText("Alpha restored answer").waitFor();
    await expectSessionSurface(page, {
      model: "model/alpha",
      thread: /Alpha restored answer/,
      browser: /alpha\.local/,
      terminal: /alpha session terminal/,
      file: /alpha-file\.js/
    });

    await page.locator("[data-session-target='Beta restored chat']").click();
    await page.getByText("Beta restored answer").waitFor();
    await expectSessionSurface(page, {
      model: "model/beta",
      thread: /Beta restored answer/,
      browser: /beta\.local/,
      terminal: /beta session terminal/,
      file: /beta-file\.js/
    });
  });
});

async function expectSessionSurface(page, expected) {
  const model = await page.locator(".readonly-chip[aria-label='Model locked for this chat'] span").last().textContent();
  assert.equal(model, expected.model);
  assert.match(await page.locator(".thread-list").innerText(), expected.thread);

  await page.getByRole("button", { name: "Browser" }).click();
  assert.match(await page.locator(".tool-panel").innerText(), expected.browser);

  await page.getByRole("button", { name: "Terminal" }).click();
  assert.match(await page.locator(".tool-panel").innerText(), expected.terminal);

  await page.getByRole("button", { name: "Files" }).click();
  assert.match(await page.locator(".tool-panel").innerText(), expected.file);
}

async function installTask007Tauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const models = [
      { id: "alpha", label: "model/alpha", providerId: "provider", provider: "Provider", enabled: true, active: true, source: "manual" },
      { id: "beta", label: "model/beta", providerId: "provider", provider: "Provider", enabled: true, active: false, source: "manual" }
    ];
    const sessions = new Map();
    const calls = [];
    window.__task007Calls = calls;

    function stateFor(id, title, model, prompt) {
      return {
        id,
        project: "runtime-session-repo",
        title,
        selectedModel: model,
        browser: { url: `http://${title.split(" ")[0].toLowerCase()}.local`, title, summary: `${title} browser` },
        files: { roots: [[`${title.split(" ")[0].toLowerCase()}-file.js`, "file", "file-editor"]], breadcrumbs: ["runtime-session-repo", "frontend", `${title.split(" ")[0].toLowerCase()}-file.js`], lines: [`console.log("${title}")`] },
        terminal: { output: `${title.split(" ")[0].toLowerCase()} session terminal`, title: `${title} Terminal`, summary: `${title} output` },
        thread: { user: prompt, agent: `${title} answer`, extra: "Restored C4OS session record.", tool: "Runtime restored", run: "Run restored" },
        turns: [{ id: `${id}-turn`, user: prompt, agent: `${title} answer`, extra: "Restored C4OS session record.", tool: "Runtime restored", run: "Run restored" }]
      };
    }

    if (nextOptions.resumed) {
      sessions.set("c4os-session-alpha", stateFor("c4os-session-alpha", "Alpha restored", "model/alpha", "Alpha restored prompt"));
      sessions.set("c4os-session-beta", stateFor("c4os-session-beta", "Beta restored", "model/beta", "Beta restored prompt"));
    }

    function workspacePayload(activeId = nextOptions.resumed ? "c4os-session-alpha" : "") {
      const active = sessions.get(activeId);
      return {
        workspace: {
          project: "runtime-session-repo",
          session: active?.title ? `${active.title} chat` : "",
          branch: "main",
          model: active?.selectedModel || "model/alpha",
          sessionId: active?.id || ""
        },
        projects: [{ name: "runtime-session-repo", sessions: Array.from(sessions.values()).map((session) => ({ id: session.id, label: `${session.title} chat` })) }],
        providers: [{ id: "provider", label: "Provider", kind: "OpenAI-compatible", baseUrl: "https://provider.test/v1", endpoint: "https://provider.test/v1", status: "Key configured", keyStatus: { state: "present", source: "session" }, enabled: true, supportsDiscovery: true }],
        models,
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Mock C4OS Marketplace", summary: "Mock configured marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: active?.browser || { url: "http://empty.local", title: "Empty", summary: "Empty browser" },
        files: active?.files || { roots: [], breadcrumbs: ["runtime-session-repo"], lines: [] },
        terminal: active?.terminal || { output: "", title: "Terminal", summary: "" },
        thread: active?.thread || { user: "", agent: "", extra: "", tool: "", run: "" },
        sessions: Array.from(sessions.values())
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, payload = {}) => {
          if (command === "load_workspace") return workspacePayload();
          if (command === "load_session") return sessions.get(payload.request.sessionId);
          if (command === "create_session") {
            const label = payload.label || payload.project;
            const id = `c4os-session-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
            calls.push({ command, session: payload.sessionId, label, model: payload.model });
            sessions.set(id, stateFor(id, label, payload.model, label));
            return sessions.get(id);
          }
          if (command === "send_prompt") {
            calls.push({ command, session: payload.sessionId, prompt: payload.prompt, model: payload.model });
            const session = sessions.get(payload.sessionId);
            session.thread = { user: payload.prompt, agent: `Reply for ${payload.prompt}`, extra: "C4OS runtime reference recorded.", tool: "C4OS runtime adapter", run: "Run complete" };
            session.turns.push({ id: `${payload.sessionId}-${session.turns.length}`, ...session.thread });
            return { session, prompt: payload.prompt, agent: session.thread.agent, run: "Run complete", events: [{ kind: "activity", text: "C4OS runtime adapter", sequence: 1 }] };
          }
          if (command === "select_session_model") return { session: payload.request.session, model: payload.request.model };
          return { ok: true };
        }
      },
      event: {
        listen: async () => () => {}
      }
    };
  }, options);
}
