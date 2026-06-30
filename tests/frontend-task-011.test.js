import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-011 terminal slice", () => {
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

  it("renders a session-scoped xterm transcript in the existing Terminal tab", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator("[data-terminal-emulator]").waitFor();
    await page.locator(".xterm-screen").waitFor();
    assert.doesNotMatch(await page.locator(".xterm-screen").innerText(), /npm run mock:task-002/);
    assert.equal(await page.locator("[data-terminal-form]").count(), 0);
    assert.equal(await page.locator("[data-terminal-pane='user']").count(), 0);
    assert.equal(await page.locator("[data-terminal-pane='agent']").count(), 0);
    await page.locator("[data-agent-terminal]").waitFor();
    assert.match(await page.locator("[data-agent-terminal]").innerText(), /agent transcript/);
    assert.equal(await page.locator(".terminal-tool [data-artifact-renderer]").count(), 0);
    assert.equal(await page.locator(".terminal-tool iframe[data-artifact-frame]").count(), 0);
    assert.equal(await page.locator(".terminal-tool [data-browser-preview]").count(), 0);
    assert.equal(await page.locator(".terminal-tool .files-list").count(), 0);

    await page.evaluate(() => document.activeElement?.blur?.());
    await page.locator("[data-terminal-emulator]").click();
    assert.equal(await page.evaluate(() => {
      const host = document.querySelector("[data-terminal-emulator]");
      return Boolean(host?.contains(document.activeElement));
    }), true);
    await page.keyboard.type("printf user-task-011");
    await page.keyboard.press("Enter");
    await page.locator(".xterm-screen", { hasText: "user-task-011" }).waitFor();

    await page.waitForFunction(() => window.__task011TerminalWrites.join("").includes("user-task-011"));
    const terminalCommands = calls
      .filter((call) => !["load_workspace", "native_menu_state"].includes(call.command))
      .map((call) => call.command);
    assert.equal(terminalCommands[0], "start_terminal_session");
    assert.ok(terminalCommands.includes("resize_terminal_session"));
    assert.ok(terminalCommands.slice(1).every((command) => [
      "read_terminal_output",
      "resize_terminal_session",
      "write_terminal_input"
    ].includes(command)));
    assert.deepEqual(calls.find((call) => call.command === "start_terminal_session").payload.request, {
      sessionId: "alpha",
      terminalKind: "user"
    });

    await page.locator("[data-session-target='Beta Terminal']").click();
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator(".xterm-screen", { hasText: "beta prompt" }).waitFor();
    assert.doesNotMatch(await page.locator(".xterm-screen").innerText(), /user-task-011/);

    await page.close();
  });

  it("uses the real xterm renderer for click-to-type input and agent-pane resizing", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { disableTerminalEcho: true, useRealXterm: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator("[data-terminal-emulator] .xterm-helper-textarea").waitFor();
    await page.locator("[data-agent-terminal]").waitFor();
    const before = await page.locator("[data-agent-terminal]").boundingBox();

    await page.locator("[data-terminal-emulator]").click();
    await page.keyboard.type("real-xterm-input");
    await page.waitForFunction(() => window.__task011TerminalWrites.join("").includes("real-xterm-input"));
    await page.locator(".xterm-screen", { hasText: "real-xterm-input" }).waitFor();

    await drag(page, page.locator("[data-resize-stack='terminal-agent']"), 0, -80);
    const after = await page.locator("[data-agent-terminal]").boundingBox();
    assert.notEqual(Math.round(before.height), Math.round(after.height));

    await page.close();
  });

  it("does not pretend local echo can execute commands when PTY startup fails", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { failTerminalStart: true, useRealXterm: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator(".xterm-screen", { hasText: "Terminal unavailable: No trusted workspace root is active" }).waitFor();
    await page.locator("[data-terminal-emulator]").click();
    await page.keyboard.type("ls");
    await page.keyboard.press("Enter");

    await page.locator(".xterm-screen", { hasText: "ls" }).waitFor();
    assert.equal((await page.locator(".xterm-screen").innerText()).includes("README.md"), false);

    await page.close();
  });

  it("accepts backend terminal output when the frontend workspace session id is not hydrated", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { blankWorkspaceSessionId: true, useRealXterm: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator(".xterm-screen", { hasText: "alpha prompt" }).waitFor();
    await page.locator("[data-terminal-emulator]").click();
    await page.keyboard.type("ls");
    await page.keyboard.press("Enter");
    await page.locator(".xterm-screen", { hasText: "ls-result-from-backend" }).waitFor();

    await page.close();
  });

  it("starts the trusted workspace terminal from #new-session before a chat session exists", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { blankWorkspaceSessionId: true, useRealXterm: true });

    await page.goto(`${server.origin}/#new-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator(".xterm-screen", { hasText: "alpha prompt" }).waitFor();
    const terminalText = await page.locator(".xterm-screen").innerText();
    assert.doesNotMatch(terminalText, /A C4OS session is required before using Terminal/);
    assert.deepEqual(calls.find((call) => call.command === "start_terminal_session").payload.request, {
      sessionId: "",
      terminalKind: "user"
    });

    await page.close();
  });

  it("polls backend-owned PTY output when native terminal events are unavailable", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, {
      omitEventListener: true,
      pollTerminalOutput: true,
      useRealXterm: true
    });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator("[data-terminal-emulator]").click();
    await page.keyboard.type("ls");
    await page.keyboard.press("Enter");
    await page.locator(".xterm-screen", { hasText: "ls-result-from-backend" }).waitFor();

    assert.ok(calls.some((call) => call.command === "read_terminal_output"));

    await page.close();
  });

  it("does not parse chat prompts into agent terminal commands before runtime tool requests", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { useRealXterm: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.locator(".composer-dock .prompt-box").fill("run ls -al and tell me what changed");
    await page.locator(".composer-dock .send-button").click();
    await page.getByRole("button", { name: "Terminal" }).click();

    await page.locator(".xterm-screen", { hasText: "alpha prompt" }).waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ ls -al/);
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /README\.md/);
    await page.locator(".xterm-screen", { hasText: "alpha prompt" }).waitFor();
    assert.equal(await page.locator("[data-agent-terminal] textarea, [data-agent-terminal] input, [data-agent-terminal] [contenteditable='true']").count(), 0);
    assert.equal(calls.some((call) => call.command === "run_terminal_command"), false);

    await page.close();
  });

  it("preserves the mounted agent terminal until runtime emits a tool request", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { useRealXterm: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator("[data-agent-terminal]").waitFor();
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /README\.md/);

    await page.locator(".composer-dock .prompt-box").fill("run ls");
    await page.locator(".composer-dock .send-button").click();

    await page.waitForFunction(() => window.__task011Calls.some((call) => call.command === "send_prompt"));
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /\$ ls/);
    assert.doesNotMatch(await page.locator("[data-agent-terminal]").innerText(), /README\.md/);
    await page.locator("[data-agent-terminal]", { hasText: "No structured agent terminal calls for this run." }).waitFor();
    assert.doesNotMatch(await page.locator(".xterm-screen").innerText(), /README\.md/);
    assert.equal(calls.some((call) => call.command === "run_terminal_command"), false);

    await page.close();
  });

  it("visually separates the agent console and keeps the latest agent output visible", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    const calls = [];
    await installTask011Tauri(page, calls, { useRealXterm: true, longAgentOutput: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Terminal" }).click();

    const contrast = await page.evaluate(() => {
      const user = getComputedStyle(document.querySelector(".terminal-tool")).backgroundColor;
      const agent = getComputedStyle(document.querySelector("[data-agent-terminal]")).backgroundColor;
      const rgb = (value) => value.match(/\d+/g).slice(0, 3).map(Number);
      const luminance = (value) => rgb(value).reduce((sum, channel) => sum + channel, 0);
      return { agent, agentLuminance: luminance(agent), user, userLuminance: luminance(user) };
    });
    assert.ok(contrast.agentLuminance > contrast.userLuminance, JSON.stringify(contrast));

    await page.evaluate(async () => {
      const data = await import("/data.js");
      await data.runConnectorTerminalCommand("ls", "agent");
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    });
    await page.locator("[data-agent-terminal]", { hasText: "final-agent-line" }).waitFor();

    const scrollState = await page.locator("[data-agent-terminal]").evaluate((node) => ({
      clientHeight: node.clientHeight,
      scrollHeight: node.scrollHeight,
      scrollTop: node.scrollTop
    }));
    assert.ok(scrollState.scrollHeight > scrollState.clientHeight, JSON.stringify(scrollState));
    assert.ok(
      scrollState.scrollTop + scrollState.clientHeight >= scrollState.scrollHeight - 2,
      JSON.stringify(scrollState)
    );

    await page.close();
  });

});

async function installTask011Tauri(page, calls, options = {}) {
  await page.addInitScript((initOptions) => {
    window.__task011TerminalWrites = [];

    if (!initOptions.useRealXterm) {
      class MockTerminal {
        constructor(options = {}) {
          this.options = options;
          this.cols = options.cols || 102;
          this.rows = options.rows || 31;
          this._onData = null;
          this._text = "";
        }

        open(node) {
          this.node = node;
          this.screen = document.createElement("div");
          this.screen.className = "xterm-screen";
          this.screen.tabIndex = 0;
          node.classList.add("xterm");
          node.append(this.screen);
          this.screen.addEventListener("keydown", (event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              this._onData?.("\r");
              return;
            }
            if (event.key.length === 1) {
              event.preventDefault();
              this._onData?.(event.key);
            }
          });
        }

        focus() {
          this.screen?.focus();
        }

        onData(callback) {
          this._onData = callback;
        }

        write(value) {
          this._text += value;
          this.screen.textContent = this._text;
        }

        dispose() {
          this.node?.replaceChildren();
        }
      }

      window.Terminal = MockTerminal;
    }

    const sessions = new Map([
      ["alpha", sessionRecord("alpha", "Alpha Terminal", "alpha prompt")],
      ["beta", sessionRecord("beta", "Beta Terminal", "beta prompt")]
    ]);

    function active() {
      return sessions.get(window.__task011ActiveSession || "alpha");
    }

    function payload() {
      const session = active();
      return {
        workspace: {
          project: "Terminal Repo",
          session: session.title,
          branch: "task-011",
          model: "model/alpha",
          sessionId: initOptions.blankWorkspaceSessionId ? "" : session.id
        },
        projects: [{ name: "Terminal Repo", sessions: Array.from(sessions.values()).map((item) => ({ id: item.id, label: item.title })) }],
        providers: [],
        models: [],
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
      event: initOptions.omitEventListener ? undefined : {
        listen: async (event, callback) => {
          window.__task011TerminalListener = { event, callback };
          return () => {
            window.__task011TerminalListener = null;
          };
        }
      },
      core: {
        invoke: async (command, invokePayload = {}) => {
          window.__task011Calls.push({ command, payload: invokePayload });
          if (command === "load_workspace") return payload();
          if (command === "load_session") {
            window.__task011ActiveSession = invokePayload.request.sessionId;
            return sessions.get(invokePayload.request.sessionId);
          }
          if (command === "send_prompt") {
            const session = sessions.get(invokePayload.sessionId || "alpha");
            session.thread = {
              user: invokePayload.prompt,
              agent: "Explicit command request recorded.",
              extra: "Connector run events completed successfully.",
              tool: "Agent command terminal",
              run: "completed"
            };
            session.turns.push({ id: `${session.id}-turn-${session.turns.length + 1}`, ...session.thread });
            return {
              prompt: invokePayload.prompt,
              run: "completed",
              agent: session.thread.agent,
              model: "model/alpha",
              session,
              events: []
            };
          }
          if (command === "run_terminal_command") {
            const request = invokePayload.request;
            const session = sessions.get(request.sessionId || "alpha");
            const output = initOptions.longAgentOutput
              ? Array.from({ length: 80 }, (_, index) => `agent-output-line-${index + 1}`).join("\n") + "\nfinal-agent-line\n"
              : "README.md\nfrontend\n";
            session.terminal.agentTerminal.output = `$ ${request.command}\n${output}`;
            session.terminal.agentTerminal.cwd = "/workspace";
            session.terminal.actions.push({
              id: `terminal-action-${session.terminal.actions.length + 1}`,
              sessionId: session.id,
              terminalKind: request.terminalKind,
              command: request.command,
              cwd: "/workspace",
              status: "completed",
              output: session.terminal.agentTerminal.output,
              exitCode: 0
            });
            return {
              command: request.command,
              output: session.terminal.agentTerminal.output,
              terminal: session.terminal,
              action: session.terminal.actions.at(-1)
            };
          }
          if (command === "start_terminal_session") {
            if (initOptions.failTerminalStart) throw new Error("No trusted workspace root is active");
            const session = sessions.get(invokePayload.request.sessionId || "alpha");
            queueMicrotask(() => {
              window.__task011TerminalListener?.callback({
                payload: {
                  sessionId: session.id,
                  terminalKind: "user",
                  text: `${session.terminal.userTerminal.output}\n`
                }
              });
            });
            return { terminal: session.terminal };
          }
          if (command === "write_terminal_input") {
            const request = invokePayload.request;
            const session = sessions.get(request.sessionId || "alpha");
            window.__task011TerminalWrites.push(request.input);
            session.terminal.userTerminal.output += request.input === "\r"
              ? "\nuser-task-011\nalpha prompt "
              : request.input;
            if (!initOptions.disableTerminalEcho) {
              queueMicrotask(() => {
                window.__task011TerminalListener?.callback({
                  payload: {
                    sessionId: session.id,
                    terminalKind: "user",
                    text: request.input === "\r" ? "\nuser-task-011\nls-result-from-backend\nalpha prompt " : request.input
                  }
                });
              });
            }
            return { terminal: session.terminal };
          }
          if (command === "resize_terminal_session") {
            return { ok: true };
          }
          if (command === "read_terminal_output") {
            const request = invokePayload.request;
            const session = sessions.get(request.sessionId || "alpha");
            const output = initOptions.pollTerminalOutput && window.__task011TerminalWrites.join("").includes("ls\r")
              ? "ls-result-from-backend\nalpha prompt "
              : "";
            if (output) session.terminal.userTerminal.output += output;
            return {
              sessionId: session.id,
              terminalKind: request.terminalKind || "user",
              text: output
            };
          }
          return {};
        }
      }
    };

    function sessionRecord(id, title, output) {
      return {
        id,
        workspaceId: "terminal-repo",
        project: "Terminal Repo",
        title,
        selectedModel: "model/alpha",
        browser: { url: "https://example.com", title: "Browser", summary: "Browser remains separate", artifactId: "", previewMode: "public", profileId: `browser-${id}`, localPath: "", html: "" },
        artifacts: [{ id: `artifact-${id}`, title: "Artifact", kind: "markdown", origin: "generated", mimeType: "text/markdown", filename: "artifact.md", safePreview: { url: `artifact://${id}`, title: "Artifact", summary: "Not terminal", html: "", content: "# Not Terminal", dataUrl: "" } }],
        files: { roots: [["README.md"]], breadcrumbs: ["Terminal Repo"], lines: ["README.md"], currentPath: "README.md", content: "files remain separate", savedContent: "files remain separate", dirty: false, status: "clean" },
        terminal: {
          output,
          title: "Terminal",
          summary: "Backend-owned terminal state",
          userTerminal: { title: "User terminal", summary: "Interactive user command surface", output, cwd: "/workspace", running: false },
          agentTerminal: { title: "Agent command terminal", summary: "Read-only agent command output", output: "agent transcript", cwd: "/workspace", running: false },
          actions: []
        },
        thread: { user: "", agent: "", extra: "", tool: "", run: "" },
        browserActions: [],
        turns: [],
        messages: [],
        runs: [],
        runtimeReference: { id: `${id}-runtime`, label: "C4OS runtime adapter", adapter: "c4os-runtime-adapter" }
      };
    }
  }, options);
  await page.exposeFunction("__task011RecordCall", (call) => calls.push(call));
  await page.addInitScript(() => {
    window.__task011Calls = [];
    const originalPush = window.__task011Calls.push.bind(window.__task011Calls);
    window.__task011Calls.push = (call) => {
      window.__task011RecordCall(call);
      return originalPush(call);
    };
  });
}

async function drag(page, locator, deltaX, deltaY) {
  const box = await locator.boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + deltaX, box.y + box.height / 2 + deltaY, { steps: 6 });
  await page.mouse.up();
}
