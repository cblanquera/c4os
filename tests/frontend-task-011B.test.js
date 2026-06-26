import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-011B chat session transition polish", () => {
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

  it("creates and selects a new chat before the first connector response returns", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.getByText("Prior stale assistant answer.").waitFor({ state: "detached" });

    await page.locator(".prompt-box").fill("Build the transition polish");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.waitForURL(/#chat-session$/);
    await page.locator(".session-row.is-active", { hasText: "Build the transition polish" }).waitFor();
    await page.locator(".message.user").first().getByText("Build the transition polish").waitFor();
    await page.locator(".message.agent").first().getByText("Connector is processing the request.").waitFor();
    assert.equal(await page.getByText("Prior stale assistant answer.").count(), 0);
    assert.equal((await visibleThreadText(page))[0], "Build the transition polish");

    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.getByText("Transition response complete.").waitFor();
    await page.close();
  });

  it("streams runtime activity into the new transcript before the final response returns", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("Stream before final response");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.waitForURL(/#chat-session$/);
    await page.evaluate(() => window.__task011BResolveCreate());
    await page.waitForFunction(() => window.__task011BPromptStarted === true);
    await page.evaluate(() => window.__task011BEmitRuntime({
      kind: "activity",
      text: "Realtime activity before final response.",
      sequence: 1
    }));

    await page.locator(".work-log").first().getByText("Realtime activity before final response.").waitFor();
    assert.equal(await page.getByText("Transition response complete.").count(), 0);

    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.getByText("Transition response complete.").waitFor();
    await page.close();
  });

  it("keeps the completed work log expandable after backend session reconciliation", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("Expand reconciled work log");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);

    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.getByText("Transition response complete.").waitFor();

    const toggle = page.getByRole("button", { name: /Worked for/ }).first();
    await toggle.click();
    await page.locator(".work-log").first().locator(".work-log-body").waitFor();
    await page.locator(".work-log").first().getByText("Waiting on connector").waitFor();
    assert.equal(await toggle.getAttribute("aria-expanded"), "true");

    await page.close();
  });

  it("shows setup-required state inside the new transcript when the selected model cannot run", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page, { setupRequired: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("Use the unconfigured model");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.waitForURL(/#chat-session$/);
    await page.locator(".session-row.is-active", { hasText: "Use the unconfigured model" }).waitFor();
    await page.locator(".message.user").first().getByText("Use the unconfigured model").waitFor();
    await page.locator(".message.agent").first().getByText("Provider setup required.").waitFor();
    await page.locator(".work-log").first().getByText("Provider setup required: OpenRouter Review is missing an API key.").waitFor();
    assert.equal(await page.getByText("Prior stale assistant answer.").count(), 0);

    await page.close();
  });

  it("keeps agent command terminal output scoped to the new chat session", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator("[data-agent-terminal]").waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run git status");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator("[data-agent-terminal]").waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.locator("[data-agent-terminal]", { hasText: "$ git status" }).waitFor();
    await page.locator("[data-agent-terminal]", { hasText: "On branch main" }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.close();
  });

  it("puts the newest chat session first in project navigation", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run pwd");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.getByText("Transition response complete.").waitFor();

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run git status");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.getByText("Transition response complete.").waitFor();

    assert.deepEqual(await page.locator(".session-row").evaluateAll((rows) => rows.map((row) => row.textContent.trim()).slice(0, 2)), [
      "run git status",
      "run pwd"
    ]);

    await page.close();
  });

  it("switches the agent terminal to the active command while the command is running", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page, { delayTerminalCommand: true, stalePromptTerminal: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run git status");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.evaluate(() => window.__task011BResolveCreate());

    await page.locator("[data-agent-terminal]", { hasText: "$ git status" }).waitFor();
    await page.locator("[data-agent-terminal]", { hasText: "Running..." }).waitFor();
    await page.waitForFunction(() => window.__task011BTerminalCommandStarted === true);
    assert.equal(await page.getByText("Transition response complete.").count(), 0);
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.evaluate(() => window.__task011BResolvePrompt());

    await page.locator("[data-agent-terminal]", { hasText: "$ git status" }).waitFor();
    await page.locator("[data-agent-terminal]", { hasText: "Running..." }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.evaluate(() => window.__task011BResolveTerminalCommand());
    await page.locator("[data-agent-terminal]", { hasText: "On branch main" }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.close();
  });

  it("ignores late agent terminal responses from a previously selected chat", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page, { delayTerminalCommand: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.evaluate(async () => {
      const data = await import("/data.js");
      data.workspace.sessionId = "prior";
      window.__task011BLatePriorCommand = data.runConnectorTerminalCommand("pwd", "agent");
      await new Promise((resolve) => setTimeout(resolve, 0));
      data.workspace.sessionId = "active-git-status";
      data.terminalState.agentTerminal.output = "$ git status\nRunning...";
    });

    await page.waitForFunction(() => window.__task011BTerminalCommandStarted === true);
    await page.evaluate(() => window.__task011BResolveTerminalCommand());

    const output = await page.evaluate(async () => {
      const data = await import("/data.js");
      await window.__task011BLatePriorCommand;
      return data.terminalState.agentTerminal.output;
    });
    assert.match(output, /\$ git status/);
    assert.doesNotMatch(output, /\$ pwd/);

    await page.close();
  });

  it("keeps an existing thread explicit command visible when the prompt session snapshot is stale", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page, { delayTerminalCommand: true, stalePromptTerminal: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator("[data-agent-terminal]", { hasText: "$ pwd" }).waitFor();

    await page.locator(".composer-dock .prompt-box").fill("run git status");
    await page.locator(".composer-dock .send-button").click();
    await page.locator("[data-agent-terminal]", { hasText: "$ git status" }).waitFor();
    await page.locator("[data-agent-terminal]", { hasText: "Running..." }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.waitForFunction(() => window.__task011BPromptStarted === true);
    await page.waitForTimeout(50);
    assert.match(await page.locator("[data-agent-terminal]").innerText(), /\$ git status/);
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.evaluate(() => window.__task011BResolveTerminalCommand());
    await page.locator("[data-agent-terminal]", { hasText: "On branch main" }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ pwd/);

    await page.close();
  });

  it("normalizes two explicit commands in one message before starting the agent terminal bridge", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run pwd and git status");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());
    await page.locator("[data-agent-terminal]", { hasText: "$ pwd && git status" }).waitFor();

    assert.deepEqual(await page.evaluate(() => window.__task011BTerminalCommands), ["pwd && git status"]);

    await page.close();
  });

  it("selects the Terminal tool for a new-session explicit command prompt", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page);

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("run git status and ls -al");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);

    await page.getByRole("button", { name: "Terminal" }).waitFor();
    assert.equal(await page.getByRole("button", { name: "Terminal" }).getAttribute("aria-pressed"), "true");
    await page.locator("[data-agent-terminal]", { hasText: "$ git status && ls -al" }).waitFor();

    await page.close();
  });

  it("renders fenced markdown code blocks as block code instead of inline fragments", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask011BTauri(page, { markdownResponse: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.locator(".prompt-box").fill("Show fenced code");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.waitForURL(/#chat-session$/);
    await page.evaluate(() => window.__task011BResolveCreate());
    await page.evaluate(() => window.__task011BResolvePrompt());

    const block = page.locator(".message.agent pre code").first();
    await block.waitFor();
    assert.match(await block.innerText(), /README\.md/);
    assert.equal(await page.locator(".message.agent p code", { hasText: "bash" }).count(), 0);

    await page.close();
  });
});

async function visibleThreadText(page) {
  return page.locator(".thread-list > .message").evaluateAll((nodes) =>
    nodes.map((node) => node.textContent.trim())
  );
}

async function installTask011BTauri(page, options = {}) {
  await page.addInitScript((initOptions) => {
    const sessions = new Map([
      ["prior", {
        id: "prior",
        project: "Transition Repo",
        title: "Prior stale session",
        selectedModel: "model/ready",
        browser: { url: "https://example.com/prior", title: "Prior Browser", summary: "Browser", previewMode: "public", profileId: "prior", localPath: "", html: "" },
        artifacts: [],
        files: { roots: [["README.md"]], breadcrumbs: ["Transition Repo"], lines: ["README.md"] },
        terminal: { output: "prior terminal", title: "Terminal", summary: "Terminal", userTerminal: { output: "prior terminal" }, agentTerminal: { output: "$ pwd\n/previous/session\n" }, actions: [] },
        thread: {
          user: "Prior stale prompt.",
          agent: "Prior stale assistant answer.",
          extra: "Prior extra.",
          tool: "Prior tool",
          run: "Prior complete"
        },
        turns: [{
          id: "prior-turn",
          user: "Prior stale prompt.",
          agent: "Prior stale assistant answer.",
          extra: "Prior extra.",
          tool: "Prior tool",
          run: "Prior complete"
        }]
      }]
    ]);

    let activeSessionId = "prior";
    const listeners = new Map();
    let pendingCreateResolve = null;
    let pendingPromptResolve = null;
    let pendingTerminalResolve = null;
    window.__task011BPromptStarted = false;
    window.__task011BTerminalCommandStarted = false;
    window.__task011BTerminalCommands = [];
    window.__task011BEmitRuntime = (payload) => {
      listeners.get("c4os://runtime-event")?.({ payload });
    };
    window.__task011BResolveCreate = () => pendingCreateResolve?.();
    window.__task011BResolvePrompt = () => pendingPromptResolve?.();
    window.__task011BResolveTerminalCommand = () => pendingTerminalResolve?.();

    function activeSession() {
      return sessions.get(activeSessionId);
    }

    function payload() {
      const session = activeSession();
      return {
        workspace: {
          project: "Transition Repo",
          session: session.title,
          branch: "main",
          model: initOptions.setupRequired ? "model/missing-key" : "model/ready",
          sessionId: session.id
        },
        projects: [{ name: "Transition Repo", sessions: Array.from(sessions.values()).map((record) => ({ id: record.id, label: record.title })) }],
        providers: [{
          id: "openrouter",
          label: "OpenRouter Review",
          endpoint: "https://openrouter.ai/api/v1",
          status: initOptions.setupRequired ? "Key not configured" : "Key configured",
          keyStatus: { state: initOptions.setupRequired ? "missing" : "present", source: "session" },
          enabled: true
        }],
        models: [{
          id: initOptions.setupRequired ? "missing-key" : "ready",
          label: initOptions.setupRequired ? "model/missing-key" : "model/ready",
          provider: "OpenRouter Review",
          providerId: "openrouter",
          enabled: true
        }],
        pluginCatalog: [],
        pluginMarketplaces: [],
        skillCatalog: [],
        mcpServers: [],
        browser: session.browser,
        artifacts: session.artifacts,
        files: session.files,
        terminal: session.terminal,
        thread: session.thread,
        sessions: Array.from(sessions.values())
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, invokePayload = {}) => {
          if (command === "load_workspace") return payload();
          if (command === "create_session") {
            if (!initOptions.setupRequired) {
              await new Promise((resolve) => {
                pendingCreateResolve = resolve;
              });
            }
            const id = `session-${sessions.size + 1}`;
            const session = {
              id,
              project: invokePayload.project,
              title: invokePayload.label,
              selectedModel: invokePayload.model,
              browser: { url: "about:blank", title: "Browser", summary: "Browser", previewMode: "public", profileId: id, localPath: "", html: "" },
              artifacts: [],
              files: { roots: [["README.md"]], breadcrumbs: ["Transition Repo"], lines: ["README.md"] },
              terminal: { output: "", title: "Terminal", summary: "Terminal", userTerminal: { output: "" }, agentTerminal: { output: "" }, actions: [] },
              thread: { user: "", agent: "", extra: "", tool: "", run: "" },
              turns: []
            };
            sessions.delete(id);
            sessions.set(id, session);
            activeSessionId = id;
            return { id, project: invokePayload.project, title: invokePayload.label, status: "created" };
          }
          if (command === "send_prompt") {
            if (initOptions.setupRequired) {
              throw new Error("Provider setup required: OpenRouter Review is missing an API key.");
            }
            window.__task011BPromptStarted = true;
            await new Promise((resolve) => {
              pendingPromptResolve = resolve;
            });
            const session = sessions.get(invokePayload.sessionId);
            session.thread = {
              user: invokePayload.prompt,
              agent: initOptions.markdownResponse
                ? "Here is the listing:\n\n```bash\nREADME.md\npackage.json\n```"
                : "Transition response complete.",
              extra: "Connector run events completed successfully.",
              tool: "Thinking complete",
              run: "Completed"
            };
            session.turns = [{ id: `${session.id}-turn`, ...session.thread }];
            if (initOptions.stalePromptTerminal) {
              session.terminal.agentTerminal.output = "$ pwd\n/previous/session\n";
            }
            return {
              prompt: invokePayload.prompt,
              agent: session.thread.agent,
              run: "Completed",
              session
            };
          }
          if (command === "run_terminal_command") {
            window.__task011BTerminalCommandStarted = true;
            window.__task011BTerminalCommands.push(invokePayload.request.command);
            if (initOptions.delayTerminalCommand) {
              await new Promise((resolve) => {
                pendingTerminalResolve = resolve;
              });
            }
            const session = sessions.get(invokePayload.request.sessionId);
            session.terminal.agentTerminal.output = `$ ${invokePayload.request.command}\nOn branch main\nnothing to commit\n`;
            return {
              command: invokePayload.request.command,
              output: session.terminal.agentTerminal.output,
              terminal: session.terminal,
              action: {
                command: invokePayload.request.command,
                terminalKind: "agent",
                status: "completed",
                output: session.terminal.agentTerminal.output,
                exitCode: 0
              }
            };
          }
          return {};
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
