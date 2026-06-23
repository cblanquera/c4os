import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-009 Artifact and safe HTML preview frontend behavior", () => {
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

  it("renders generated HTML artifacts in the existing Browser tab sandbox", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask009Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();

    assert.equal(await page.getByRole("button", { name: "Artifacts" }).count(), 0);
    assert.equal(await page.locator(".tool-panel [data-artifact-preview]").count(), 1);
    assert.equal(await page.locator(".tool-panel iframe[data-artifact-frame]").getAttribute("sandbox"), "allow-scripts");
    assert.match(await page.locator(".tool-panel .address-bar").innerText(), /artifact:\/\/task-009-preview/);
    assert.match(await page.locator(".tool-panel").innerText(), /Generated preview/);

    const frame = page.frameLocator(".tool-panel iframe[data-artifact-frame]");
    await frame.locator("h1").waitFor();
    assert.equal(await frame.locator("h1").innerText(), "Generated preview");
  });

  it("does not introduce an Artifacts route, panel, tab, or Terminal rendering", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask009Tauri(page);

    await page.goto(`${server.origin}/#artifacts`);
    assert.equal(await page.locator("[data-screen='app-start']").count(), 1);

    await page.goto(`${server.origin}/#chat-session`);
    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );

    await page.getByRole("button", { name: "Terminal" }).click();
    assert.equal(await page.locator(".terminal-tool iframe[data-artifact-frame]").count(), 0);
    assert.doesNotMatch(await page.locator(".terminal-tool").innerText(), /Generated preview|artifact:\/\/task-009-preview/);
  });

  it("keeps untrusted preview HTML away from credentials, files, terminal state, and app APIs", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask009Tauri(page, { probe: true });

    await page.goto(`${server.origin}/#chat-session`);
    const frame = page.frameLocator(".tool-panel iframe[data-artifact-frame]");
    await frame.locator("#probe").waitFor();

    const probe = JSON.parse(await frame.locator("#probe").innerText());
    assert.equal(probe.credential, "blocked");
    assert.equal(probe.file, "blocked");
    assert.equal(probe.terminal, "blocked");
    assert.equal(probe.privilegedApi, "blocked");
    assert.equal(await page.evaluate(() => window.__task009ProbeLeak || ""), "");
  });

  it("restores each session-owned artifact preview when switching sessions", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask009Tauri(page, { sessions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").waitFor();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").innerText(), "Alpha artifact");

    await page.locator("[data-session-target='Beta Artifact']").click();
    await page.getByRole("button", { name: "Browser" }).click();
    await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").waitFor();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").innerText(), "Beta artifact");
  });
});

async function installTask009Tauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const probeHtml = `<!doctype html><html><body><h1>Generated preview</h1><pre id="probe"></pre><script>
      const probe = {
        credential: canRead(() => parent.__TASK009_PROVIDER_KEY__) ? "leaked" : "blocked",
        file: canRead(() => parent.__TASK009_FILE_CONTENT__) ? "leaked" : "blocked",
        terminal: canRead(() => parent.__TASK009_TERMINAL_STATE__) ? "leaked" : "blocked",
        privilegedApi: typeof window.__TAURI__ === "undefined" ? "blocked" : "leaked"
      };
      try { parent.__task009ProbeLeak = "parent reachable"; } catch (error) {}
      function canRead(read) {
        try { return typeof read() !== "undefined"; } catch (error) { return false; }
      }
      document.querySelector("#probe").textContent = JSON.stringify(probe);
    </script></body></html>`;

    const parentFrame = window.parent === window;
    if (parentFrame) {
      window.__TASK009_PROVIDER_KEY__ = "provider-secret";
      window.__TASK009_FILE_CONTENT__ = "workspace-secret";
      window.__TASK009_TERMINAL_STATE__ = "terminal-secret";
    }

    const artifact = (id, title, html) => ({
      id,
      title,
      kind: "html",
      origin: "generated",
      safePreview: {
        url: `artifact://${id}`,
        title,
        summary: "Generated HTML artifact preview",
        html
      }
    });
    const alpha = artifact("alpha-artifact", "Alpha artifact", "<!doctype html><html><body><h1>Alpha artifact</h1></body></html>");
    const beta = artifact("beta-artifact", "Beta artifact", "<!doctype html><html><body><h1>Beta artifact</h1></body></html>");
    const baseArtifact = artifact(
      "task-009-preview",
      "Generated preview",
      nextOptions.probe ? probeHtml : "<!doctype html><html><body><h1>Generated preview</h1></body></html>"
    );

    const sessions = new Map();
    if (nextOptions.sessions) {
      sessions.set("alpha", sessionRecord("alpha", "Alpha Artifact", alpha));
      sessions.set("beta", sessionRecord("beta", "Beta Artifact", beta));
    }

    function sessionRecord(id, title, previewArtifact) {
      return {
        id,
        project: "artifact-repo",
        title,
        selectedModel: "model/artifact",
        browser: {
          url: previewArtifact.safePreview.url,
          title: previewArtifact.safePreview.title,
          summary: previewArtifact.safePreview.summary,
          artifactId: previewArtifact.id,
          previewMode: "artifact"
        },
        artifacts: [previewArtifact],
        files: { roots: [], breadcrumbs: ["artifact-repo"], lines: [] },
        terminal: { output: "terminal state remains separate", title: "Terminal", summary: "No artifact rendering" },
        thread: { user: title, agent: title, extra: "", tool: "", run: "" },
        turns: [{ id: `${id}-turn`, user: title, agent: title, extra: "", tool: "", run: "" }]
      };
    }

    function payload(activeId = nextOptions.sessions ? "alpha" : "") {
      const active = sessions.get(activeId);
      return {
        workspace: { project: "artifact-repo", session: active?.title || "Artifact review", branch: "main", model: "model/artifact", sessionId: active?.id || "" },
        projects: [{ name: "artifact-repo", sessions: nextOptions.sessions ? Array.from(sessions.values()).map((session) => ({ id: session.id, label: session.title })) : ["Artifact review"] }],
        providers: [{ id: "openrouter", label: "OpenRouter", kind: "OpenAI-compatible", baseUrl: "https://openrouter.ai/api/v1", endpoint: "https://openrouter.ai/api/v1", status: "Key configured", keyStatus: { state: "present", source: "session" }, enabled: true, supportsDiscovery: true }],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Built by C4OS", summary: "Default marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: active?.browser || { url: baseArtifact.safePreview.url, title: baseArtifact.safePreview.title, summary: baseArtifact.safePreview.summary, artifactId: baseArtifact.id, previewMode: "artifact" },
        artifacts: active?.artifacts || [baseArtifact],
        files: active?.files || { roots: [], breadcrumbs: ["artifact-repo"], lines: [] },
        terminal: active?.terminal || { output: "terminal state remains separate", title: "Terminal", summary: "No artifact rendering" },
        thread: active?.thread || { user: "Artifact", agent: "Artifact ready", extra: "", tool: "", run: "" },
        sessions: Array.from(sessions.values())
      };
    }

    if (!parentFrame) return;

    window.__TAURI__ = {
      core: {
        invoke: async (command, requestPayload = {}) => {
          if (command === "load_workspace") return payload();
          if (command === "load_session") return sessions.get(requestPayload.request.sessionId);
          return { ok: true };
        }
      },
      event: { listen: async () => () => {} }
    };
  }, options);
}
