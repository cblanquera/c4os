import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-012 settings IA and extension records frontend slice", () => {
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

  it("preserves the accepted Settings navigation order while rendering app-owned extension records", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask012Tauri(page);

    await page.goto(`${server.origin}/?connector=tauri#settings-plugins`);

    const nav = await page.locator(".settings-link span").evaluateAll((nodes) => nodes.map((node) => node.textContent));
    assert.deepEqual(nav, ["Providers", "Models", "Runtimes", "Configuration", "Plugins", "Skills", "MCP Servers"]);

    const pluginText = await page.locator(".settings-main").innerText();
    assert.match(pluginText, /GitHub/);
    assert.match(pluginText, /Installed from Built by C4OS/);
    assert.match(pluginText, /Project scope/);
    assert.match(pluginText, /Runtime disabled/);
    assert.match(pluginText, /Audit: Installed by workspace owner/);
    assert.doesNotMatch(pluginText, /Frontend-local catalog fixture|Mock GitHub/);

    await page.goto(`${server.origin}/#settings-skills`);
    const skillText = await page.locator(".settings-main").innerText();
    assert.match(skillText, /ChrisAI Agents/);
    assert.match(skillText, /Shared data: current transcript, selected files/);
    assert.match(skillText, /Runtime disabled/);
    assert.doesNotMatch(skillText, /Project availability fixture|Mock Skill/);

    await page.goto(`${server.origin}/#settings-mcp`);
    const mcpText = await page.locator(".settings-main").innerText();
    assert.match(mcpText, /Docs MCP/);
    assert.match(mcpText, /Tool access: read_docs/);
    assert.match(mcpText, /Runtime disabled/);
    assert.doesNotMatch(mcpText, /External tool connection fixture|mock_docs_mcp/);

    await page.close();
  });

  it("keeps extension records inert when prompt tags are submitted before runtime enablement", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask012Tauri(page);

    await page.goto(`${server.origin}/?connector=tauri#new-session`);
    await page.getByText("task-012-repo").first().waitFor();
    await page.locator(".prompt-box").fill("$code-review @github-reviews ^docs-mcp summarize this repo");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Extension records stayed inert.").waitFor();

    assert.deepEqual(await page.evaluate(() => window.__task012Calls), {
      createSession: 1,
      sendPrompt: 1,
      extensionInvocations: 0,
      terminalCommands: 0
    });

    await page.close();
  });

  it("renders empty states when no plugins, skills, or MCP servers are installed", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask012Tauri(page, { emptyExtensions: true });

    await page.goto(`${server.origin}/?connector=tauri#settings-plugins`);
    await page.getByText("No plugins installed").waitFor();
    await page.getByText("Add a marketplace when plugin installation is available for this workspace.").waitFor();
    assert.equal(await page.locator("[data-extension-record]").count(), 0);

    await page.goto(`${server.origin}/#settings-skills`);
    await page.getByText("No skills installed").waitFor();
    await page.getByText("Skills will appear here after a project or workspace skill folder is connected.").waitFor();
    assert.equal(await page.locator("[data-extension-record]").count(), 0);

    await page.goto(`${server.origin}/#settings-mcp`);
    await page.getByText("No MCP servers connected").waitFor();
    await page.getByText("Connect a server when MCP connection persistence is available.").waitFor();
    assert.equal(await page.locator("[data-extension-record]").count(), 0);

    await page.close();
  });
});

async function installTask012Tauri(page, options = {}) {
  await page.addInitScript((task012Options) => {
    const extensionRecord = (kind, id, label, overrides = {}) => ({
      id,
      kind,
      label,
      summary: `${label} app-owned extension record`,
      provenance: overrides.provenance,
      scopes: overrides.scopes || ["project"],
      workspaceScope: "task-012-repo",
      projectScope: "task-012-repo",
      sharedData: overrides.sharedData || ["workspace metadata"],
      runtimeAccess: "disabled",
      toolAccess: overrides.toolAccess || [],
      enabled: false,
      audit: overrides.audit || ["Record created"]
    });
    window.__task012Calls = {
      createSession: 0,
      sendPrompt: 0,
      extensionInvocations: 0,
      terminalCommands: 0
    };
    const extensionPayload = {
      pluginCatalog: task012Options.emptyExtensions ? [] : [
        extensionRecord("plugin", "github", "GitHub", {
          provenance: "Installed from Built by C4OS",
          scopes: ["repo:read", "pull_request:read"],
          toolAccess: ["review_threads"],
          audit: ["Installed by workspace owner"]
        })
      ],
      skillCatalog: task012Options.emptyExtensions ? [] : [
        extensionRecord("skill", "chrisai-agents", "ChrisAI Agents", {
          provenance: "Installed from project .agents skills",
          scopes: ["project"],
          sharedData: ["current transcript", "selected files"],
          audit: ["Enabled state reviewed"]
        })
      ],
      mcpServers: task012Options.emptyExtensions ? [] : [
        extensionRecord("mcp", "docs-mcp", "Docs MCP", {
          provenance: "Connected from workspace settings",
          scopes: ["project"],
          toolAccess: ["read_docs"],
          audit: ["Connection recorded"]
        })
      ]
    };
    const shellPayload = {
      workspace: {
        project: "task-012-repo",
        session: "Extension record review",
        branch: "main",
        model: "google/gemini-2.5-flash-lite",
        sessionId: "task-012-session"
      },
      projects: [{ name: "task-012-repo", sessions: [{ id: "task-012-session", label: "Extension record review" }] }],
      providers: [],
      models: [],
      pluginMarketplaces: [{ label: "Built by C4OS", summary: "App-owned extension source", active: true }],
      ...extensionPayload,
      browser: { url: "about:blank", title: "Browser", summary: "Ready" },
      artifacts: [],
      files: { roots: [], breadcrumbs: ["task-012-repo"], lines: [] },
      terminal: {
        output: "",
        title: "User terminal",
        summary: "Ready",
        userTerminal: { output: "", title: "User terminal", summary: "Ready", cwd: "", running: false },
        agentTerminal: { output: "", title: "Agent command terminal", summary: "Ready", cwd: "", running: false },
        actions: []
      },
      thread: { user: "Existing turn", agent: "Existing response", extra: "", tool: "Idle", run: "Complete" },
      sessions: []
    };
    window.__TAURI__ = {
      core: {
        invoke: async (command, payload = {}) => {
          if (command === "load_workspace") return shellPayload;
          if (command === "list_extensions") return extensionPayload;
          if (command === "create_session") {
            window.__task012Calls.createSession += 1;
            return { id: "task-012-new-session", project: payload.project, title: payload.label, terminal: shellPayload.terminal };
          }
          if (command === "send_prompt") {
            window.__task012Calls.sendPrompt += 1;
            return {
              prompt: payload.prompt,
              agent: "Extension records stayed inert.",
              run: "Prompt completed without extension invocation.",
              model: payload.model || "",
              events: [{ kind: "activity", text: "No extension invocation requested by runtime gateway.", sequence: 1 }]
            };
          }
          if (command === "run_terminal_command") {
            window.__task012Calls.terminalCommands += 1;
            return { terminal: shellPayload.terminal, action: { status: "blocked", output: "" } };
          }
          if (command?.includes?.("extension")) window.__task012Calls.extensionInvocations += 1;
          return { ok: true };
        }
      },
      event: { listen: async () => () => {} }
    };
  }, options);
}
