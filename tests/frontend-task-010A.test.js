import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-010A Browser address bar and local target UI", () => {
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

  it("submits public URLs through open_browser and renders the result in the accepted Browser tab", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    const address = page.getByRole("textbox", { name: "Browser address" });
    await address.fill("https://example.com/manual");
    await address.press("Enter");

    await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").waitFor();
    assert.equal(await page.locator("[data-browser-preview='public']").count(), 1);
    assert.equal(await address.inputValue(), "https://example.com/manual");
    assert.equal(await page.locator(".tool-panel iframe[data-browser-frame]").getAttribute("sandbox"), null);
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Manual Browser");
    const browserBox = await page.locator(".browser-tool").boundingBox();
    const addressBox = await page.locator(".browser-tool .address-bar").boundingBox();
    const frameBox = await page.locator(".browser-tool iframe[data-browser-frame]").boundingBox();
    assert.ok(browserBox && addressBox && frameBox);
    assert.ok(Math.abs(frameBox.x - browserBox.x) <= 1, `frame x ${frameBox.x} should align to browser x ${browserBox.x}`);
    assert.ok(Math.abs(frameBox.width - browserBox.width) <= 1, `frame width ${frameBox.width} should fill browser width ${browserBox.width}`);
    assert.ok(Math.abs(frameBox.y - (addressBox.y + addressBox.height)) <= 1, "frame should start directly below address bar");
    assert.ok(Math.abs((frameBox.y + frameBox.height) - (browserBox.y + browserBox.height)) <= 1, "frame should fill remaining browser height");
    assert.deepEqual(await page.evaluate(() => window.__task010AOpenRequests), [
      { sessionId: "alpha", target: "https://example.com/manual", actor: "user", clearRequest: false }
    ]);
    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );
  });

  it("opens project-local targets through Browser authority and renders local-file state", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    const address = page.getByRole("textbox", { name: "Browser address" });
    await address.fill("docs/local-preview.html");
    await address.press("Enter");

    await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").waitFor();
    assert.equal(
      await page.locator("[data-browser-preview='local-file'][data-local-browser-file='/Users/cblanquera/server/projects/cblanquera/mcp/docs/local-preview.html']").count(),
      1
    );
    assert.equal(await address.inputValue(), "file:///Users/cblanquera/server/projects/cblanquera/mcp/docs/local-preview.html");
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Trusted local target");
  });

  it("lets backend resolve bare public hostnames from the editable address bar", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    const address = page.getByRole("textbox", { name: "Browser address" });
    await address.fill("iamawesome.com");
    await address.press("Enter");

    await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").waitFor();
    assert.equal(await address.inputValue(), "https://iamawesome.com/");
    assert.deepEqual(await page.evaluate(() => window.__task010AOpenRequests.at(-1)), {
      sessionId: "alpha",
      target: "iamawesome.com",
      actor: "user",
      clearRequest: false
    });
  });

  it("renders address-bar failures inside Browser without replacing chat, Files, or Terminal", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    const originalChat = await page.locator(".thread-list").innerText();
    await page.getByRole("button", { name: "Browser" }).click();
    const address = page.getByRole("textbox", { name: "Browser address" });
    await address.fill("../secret.html");
    await address.press("Enter");

    assert.match(await page.locator(".browser-status").innerText(), /outside trusted root/i);
    await address.fill(".git/config");
    await address.press("Enter");
    assert.match(await page.locator(".browser-status").innerText(), /\.git|outside trusted root/i);
    assert.equal(await page.locator("[data-screen='chat-session']").count(), 1);
    assert.equal(await page.locator(".thread-list").innerText(), originalChat);

    await page.getByRole("button", { name: "Files" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha\.md/);
    await page.getByRole("button", { name: "Terminal" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha terminal/);
  });

  it("keeps TASK-009 artifact preview distinct from general browsing", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page, { artifact: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();

    assert.equal(await page.locator("[data-artifact-preview]").count(), 1);
    assert.equal(await page.locator("[data-browser-preview]").count(), 0);
    assert.equal(await page.getByRole("textbox", { name: "Browser address" }).count(), 0);
    await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").waitFor();
    assert.equal(await page.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").innerText(), "Generated artifact");
  });

  it("restores each session Browser address and rendered state after switching sessions", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010ATauri(page, { sessions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.getByRole("textbox", { name: "Browser address" }).inputValue(), "https://example.com/alpha");
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Alpha Browser");

    await page.locator("[data-session-target='Beta Browser']").click();
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.getByRole("textbox", { name: "Browser address" }).inputValue(), "https://example.com/beta");
    assert.equal(await page.frameLocator(".tool-panel iframe[data-browser-frame]").locator("h1").innerText(), "Beta Browser");
  });
});

async function installTask010ATauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const openRequests = [];
    window.__task010AOpenRequests = openRequests;

    function browserState(label, url, overrides = {}) {
      return {
        url,
        title: label,
        summary: `${label} rendered by Browser state`,
        previewMode: "public",
        profileId: "profile-task-010a",
        localPath: "",
        artifactId: "",
        html: `<!doctype html><html><body><h1>${label}</h1></body></html>`,
        ...overrides
      };
    }

    const artifact = {
      id: "artifact-task-010a",
      title: "Generated artifact",
      kind: "html",
      origin: "generated",
      safePreview: {
        url: "artifact://task-010a",
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

    const alphaBrowser = nextOptions.artifact
      ? browserState("Generated artifact", "artifact://task-010a", { previewMode: "artifact", artifactId: artifact.id })
      : browserState(nextOptions.sessions ? "Alpha Browser" : "Initial Browser", nextOptions.sessions ? "https://example.com/alpha" : "https://example.com/start");
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
            openRequests.push({
              sessionId: request.sessionId,
              target: request.target,
              actor: request.actor,
              clearRequest: request.clearRequest
            });
            if (request.target.includes("..") || request.target.startsWith(".git")) {
              throw new Error("Browser local target is outside trusted root");
            }
            const session = sessions.get(request.sessionId);
            const publicTarget = request.target.startsWith("http://") || request.target.startsWith("https://") || request.target === "iamawesome.com";
            const resolvedPublicUrl = request.target === "iamawesome.com" ? "https://iamawesome.com/" : request.target;
            const browser = publicTarget
              ? browserState("Manual Browser", resolvedPublicUrl)
              : browserState("Trusted local target", `file:///Users/cblanquera/server/projects/cblanquera/mcp/${request.target}`, {
                  previewMode: "local-file",
                  localPath: `/Users/cblanquera/server/projects/cblanquera/mcp/${request.target}`,
                  html: "<!doctype html><html><body><h1>Trusted local target</h1></body></html>"
                });
            session.browser = browser;
            session.artifacts = [];
            return { browser, action: { actor: "user", action: browser.previewMode === "local-file" ? "open-local-file" : "open-public-url", target: request.target } };
          }
          return { ok: true };
        }
      },
      event: { listen: async () => () => {} }
    };
  }, options);
}
