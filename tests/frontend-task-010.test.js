import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-010 Browser slice frontend behavior", () => {
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

  it("renders public browsing inside the accepted Browser tab without adding surfaces", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010Tauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();

    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );
    assert.equal(await page.getByRole("button", { name: "Browser" }).getAttribute("aria-pressed"), "true");
    assert.equal(await page.locator("[data-browser-preview]").count(), 1);
    assert.equal(await page.locator("[data-browser-frame]").count(), 1);
    assert.equal(await page.getByRole("textbox", { name: "Browser address" }).inputValue(), "https://example.com/docs");
    await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").waitFor();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Example Docs");

    await page.goto(`${server.origin}/#browser`);
    assert.equal(await page.locator("[data-screen='app-start']").count(), 1);
    await page.goto(`${server.origin}/#chat-session`);
    assert.equal(await page.getByRole("button", { name: "Preview" }).count(), 0);
  });

  it("keeps local file browsing and artifact preview distinct in the Browser tab", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010Tauri(page, { localFile: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();

    assert.equal(await page.locator("[data-browser-preview][data-local-browser-file='docs/preview.html']").count(), 1);
    const localFrame = page.frameLocator(".tool-panel iframe[data-browser-frame]");
    await localFrame.locator("h1").waitFor();
    assert.equal(await localFrame.locator("h1").innerText(), "Trusted local file");

    const artifactPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010Tauri(artifactPage, { artifact: true });
    await artifactPage.goto(`${server.origin}/#chat-session`);
    await artifactPage.getByRole("button", { name: "Browser" }).click();
    assert.equal(await artifactPage.locator("[data-artifact-preview]").count(), 1);
    assert.equal(await artifactPage.locator("[data-browser-preview]").count(), 0);
    await artifactPage.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").waitFor();
    assert.equal(await artifactPage.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").innerText(), "Generated artifact");
  });

  it("restores each session-owned Browser state while preserving Files and Terminal tabs", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010Tauri(page, { sessions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Alpha Browser");

    await page.getByRole("button", { name: "Files" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha\.md/);
    await page.getByRole("button", { name: "Terminal" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha terminal/);

    await page.locator("[data-session-target='Beta Browser']").click();
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Beta Browser");
    await page.getByRole("button", { name: "Files" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /beta\.md/);
    await page.getByRole("button", { name: "Terminal" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /beta terminal/);
  });

  it("only records agent Browser actions when the request is explicit", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010Tauri(page, { actions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.evaluate(async () => {
      await window.__TAURI__.core.invoke("open_browser", {
        request: {
          sessionId: "alpha",
          target: "https://example.com/rejected",
          actor: "agent",
          clearRequest: false
        }
      }).catch((error) => {
        window.__task010Rejected = error.message;
      });
      await window.__TAURI__.core.invoke("open_browser", {
        request: {
          sessionId: "alpha",
          target: "https://example.com/accepted",
          actor: "agent",
          clearRequest: true
        }
      });
    });

    assert.match(await page.evaluate(() => window.__task010Rejected), /clearly requested/i);
    assert.deepEqual(await page.evaluate(() => window.__task010Actions), [
      { sessionId: "alpha", actor: "agent", action: "open-public-url", target: "https://example.com/accepted" }
    ]);
  });
});

async function installTask010Tauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const actions = [];
    window.__task010Actions = actions;

    function browserState(label, url, overrides = {}) {
      return {
        url,
        title: label,
        summary: `${label} rendered by C4OS Browser state`,
        previewMode: "public",
        profileId: `profile-${label.toLowerCase().replace(/\s+/g, "-")}`,
        html: `<!doctype html><html><body><h1>${label}</h1></body></html>`,
        localPath: "",
        artifactId: "",
        ...overrides
      };
    }

    const artifact = {
      id: "artifact-browser-boundary",
      title: "Generated artifact",
      kind: "html",
      origin: "generated",
      safePreview: {
        url: "artifact://browser-boundary",
        title: "Generated artifact",
        summary: "Generated HTML artifact preview",
        html: "<!doctype html><html><body><h1>Generated artifact</h1></body></html>"
      }
    };

    function sessionRecord(id, title, browser, file, terminal) {
      return {
        id,
        project: "browser-repo",
        title,
        selectedModel: "model/browser",
        browser,
        artifacts: browser.previewMode === "artifact" ? [artifact] : [],
        files: { roots: [[file, "file", "file-editor", file, "0"]], breadcrumbs: ["browser-repo", file], lines: [`# ${file}`], currentPath: file, content: `# ${file}`, savedContent: `# ${file}`, dirty: false },
        terminal: { output: terminal, title: "Terminal", summary: "Terminal remains separate" },
        thread: { user: title, agent: title, extra: "", tool: "", run: "" },
        turns: [{ id: `${id}-turn`, user: title, agent: title, extra: "", tool: "", run: "" }],
        browserActions: []
      };
    }

    const alphaBrowser = nextOptions.localFile
      ? browserState("Trusted local file", "file://project/docs/preview.html", {
          previewMode: "local-file",
          localPath: "docs/preview.html",
          html: "<!doctype html><html><body><h1>Trusted local file</h1></body></html>"
        })
      : nextOptions.artifact
        ? browserState("Generated artifact", "artifact://browser-boundary", {
            previewMode: "artifact",
            artifactId: artifact.id
          })
        : nextOptions.sessions
          ? browserState("Alpha Browser", "https://example.com/alpha")
          : browserState("Example Docs", "https://example.com/docs");
    const sessions = new Map([
      ["alpha", sessionRecord("alpha", nextOptions.sessions ? "Alpha Browser" : "Browser session", alphaBrowser, "alpha.md", "alpha terminal")],
      ["beta", sessionRecord("beta", "Beta Browser", browserState("Beta Browser", "https://example.com/beta"), "beta.md", "beta terminal")]
    ]);

    function payload(activeId = "alpha") {
      const active = sessions.get(activeId);
      return {
        workspace: { project: "browser-repo", session: active.title, branch: "main", model: "model/browser", sessionId: active.id },
        projects: [{ name: "browser-repo", sessions: Array.from(sessions.values()).map((session) => ({ id: session.id, label: session.title })) }],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Built by C4OS", summary: "Default marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: active.browser,
        artifacts: active.artifacts,
        files: active.files,
        terminal: active.terminal,
        thread: active.thread,
        sessions: Array.from(sessions.values())
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, invokePayload = {}) => {
          if (command === "load_workspace") return payload();
          if (command === "load_session") return sessions.get(invokePayload.request.sessionId);
          if (command === "open_browser") {
            const request = invokePayload.request || {};
            if (request.actor === "agent" && !request.clearRequest) {
              throw new Error("Agent Browser access must be clearly requested");
            }
            const action = {
              sessionId: request.sessionId,
              actor: request.actor || "user",
              action: "open-public-url",
              target: request.target
            };
            actions.push(action);
            const session = sessions.get(request.sessionId);
            session.browser = browserState("Accepted Browser", request.target);
            session.browserActions.push(action);
            return { browser: session.browser, action };
          }
          return { ok: true };
        }
      },
      event: { listen: async () => () => {} }
    };
  }, options);
}
