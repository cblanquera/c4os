import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-010B native Wry Browser surface", () => {
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

  it("renders public pages through the Browser tab native host instead of an iframe or external-open fallback", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010BTauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();

    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );
    assert.equal(await page.locator("[data-browser-preview='public']").count(), 1);
    assert.equal(await page.locator("[data-native-browser-frame]").count(), 1);
    assert.equal(await page.locator("[data-browser-frame]").count(), 0);
    assert.equal(await page.getByRole("button", { name: "Open externally" }).count(), 0);
    assert.equal(await page.getByRole("textbox", { name: "Browser address" }).inputValue(), "https://linkedin.com/");
    await page.waitForFunction(() => window.__task010BNativeRequests.length > 0);

    const [request] = await page.evaluate(() => window.__task010BNativeRequests);
    assert.equal(request.sessionId, "alpha");
    assert.equal(request.url, "https://linkedin.com/");
    assert.equal(request.visible, true);
    assert.ok(request.width > 300);
    assert.ok(request.height > 300);

    const browserBox = await page.locator(".browser-tool").boundingBox();
    const addressBox = await page.locator(".browser-tool .address-bar").boundingBox();
    const inputBox = await page.locator(".browser-tool [data-browser-address-input]").boundingBox();
    const nativeBox = await page.locator("[data-native-browser-frame]").boundingBox();
    assert.ok(browserBox && addressBox && inputBox && nativeBox);
    const nativeInset = Math.max(0, Math.round(inputBox.height - 2));
    assert.ok(Math.abs(request.y - (nativeBox.y + nativeInset)) <= 1);
    assert.ok(Math.abs(request.height - (nativeBox.height - nativeInset)) <= 1);
    assert.ok(Math.abs(nativeBox.x - browserBox.x) <= 1);
    assert.ok(Math.abs(nativeBox.width - browserBox.width) <= 1);
    assert.ok(Math.abs(nativeBox.y - (addressBox.y + addressBox.height)) <= 1);
    assert.ok(Math.abs((nativeBox.y + nativeBox.height) - (browserBox.y + browserBox.height)) <= 1);
  });

  it("resyncs the native Browser bounds when the right panel is resized", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010BTauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    await page.waitForFunction(() => window.__task010BNativeRequests.length > 0);
    const initial = await page.evaluate(() => window.__task010BNativeRequests.at(-1));

    await page.evaluate(() => {
      document.querySelector(".app-shell").style.setProperty("--right-panel", "520px");
    });
    await page.waitForFunction((initialWidth) => {
      const latest = window.__task010BNativeRequests.at(-1);
      return latest && Math.abs(latest.width - initialWidth) > 20;
    }, initial.width);

    const resized = await page.evaluate(() => window.__task010BNativeRequests.at(-1));
    const nativeBox = await page.locator("[data-native-browser-frame]").boundingBox();
    assert.ok(nativeBox);
    assert.ok(Math.abs(resized.x - nativeBox.x) <= 1);
    assert.ok(Math.abs(resized.width - nativeBox.width) <= 1);
  });

  it("hides the native Browser surface when switching to Files or Terminal without replacing those tabs", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010BTauri(page);

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    await page.waitForFunction(() => window.__task010BNativeRequests.length > 0);

    await page.getByRole("button", { name: "Files" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha\.md/);
    await page.waitForFunction(() => window.__task010BNativeRequests.some((request) => request.visible === false));

    await page.getByRole("button", { name: "Terminal" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha terminal/);
    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );
  });

  it("keeps artifact preview distinct while trusted local files use the native Browser host", async () => {
    const artifactPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010BTauri(artifactPage, { artifact: true });

    await artifactPage.goto(`${server.origin}/#chat-session`);
    await artifactPage.getByRole("button", { name: "Browser" }).click();
    assert.equal(await artifactPage.locator("[data-artifact-preview]").count(), 1);
    assert.equal(await artifactPage.locator("[data-native-browser-frame]").count(), 0);
    assert.equal(await artifactPage.getByRole("button", { name: "Open externally" }).count(), 0);
    await artifactPage.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").waitFor();
    assert.equal(await artifactPage.frameLocator(".tool-panel iframe[data-artifact-frame]").locator("h1").innerText(), "Generated artifact");

    const localPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010BTauri(localPage, { localFile: true });

    await localPage.goto(`${server.origin}/#chat-session`);
    await localPage.getByRole("button", { name: "Browser" }).click();
    await localPage.waitForFunction(() => window.__task010BNativeRequests.length > 0);
    assert.equal(await localPage.locator("[data-browser-preview='local-file']").count(), 1);
    assert.equal(await localPage.locator("[data-native-browser-frame]").count(), 1);
    assert.equal(await localPage.locator("[data-native-browser-frame]").getAttribute("data-native-browser-url"), "file:///Users/cblanquera/server/projects/cblanquera/mcp/docs/preview.html");
    assert.equal(await localPage.locator("[data-browser-frame]").count(), 0);
  });
});

async function installTask010BTauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const nativeRequests = [];
    window.__task010BNativeRequests = nativeRequests;

    function browserState(label, url, overrides = {}) {
      return {
        url,
        title: label,
        summary: `${label} rendered by C4OS Browser state`,
        previewMode: "public",
        profileId: "profile-task-010b",
        html: "",
        localPath: "",
        artifactId: "",
        ...overrides
      };
    }

    const artifact = {
      id: "artifact-task-010b",
      title: "Generated artifact",
      kind: "html",
      origin: "generated",
      safePreview: {
        url: "artifact://task-010b",
        title: "Generated artifact",
        summary: "Generated HTML artifact preview",
        html: "<!doctype html><html><body><h1>Generated artifact</h1></body></html>"
      }
    };

    const alphaBrowser = nextOptions.artifact
      ? browserState("Generated artifact", "artifact://task-010b", { previewMode: "artifact", artifactId: artifact.id })
      : nextOptions.localFile
        ? browserState("Trusted local file", "file:///Users/cblanquera/server/projects/cblanquera/mcp/docs/preview.html", {
            previewMode: "local-file",
            localPath: "/Users/cblanquera/server/projects/cblanquera/mcp/docs/preview.html",
            html: ""
          })
        : browserState("LinkedIn", "https://linkedin.com/");

    const session = {
      id: "alpha",
      project: "browser-repo",
      title: "Browser session",
      selectedModel: "model/browser",
      browser: alphaBrowser,
      artifacts: alphaBrowser.previewMode === "artifact" ? [artifact] : [],
      files: { roots: [["alpha.md", "file", "file-editor", "alpha.md", "0"]], breadcrumbs: ["browser-repo", "alpha.md"], lines: ["# alpha.md"], currentPath: "alpha.md", content: "# alpha.md", savedContent: "# alpha.md", dirty: false },
      terminal: { output: "alpha terminal", title: "Terminal", summary: "Terminal remains separate" },
      thread: { user: "Browser session", agent: "Browser session", extra: "", tool: "", run: "" },
      turns: [{ id: "alpha-turn", user: "Browser session", agent: "Browser session", extra: "", tool: "", run: "" }],
      browserActions: []
    };

    function payload() {
      return {
        workspace: { project: "browser-repo", session: session.title, branch: "main", model: "model/browser", sessionId: session.id },
        projects: [{ name: "browser-repo", sessions: [{ id: session.id, label: session.title }] }],
        providers: [],
        models: [],
        pluginCatalog: [],
        pluginMarketplaces: [{ label: "Built by C4OS", summary: "Default marketplace", active: true }],
        skillCatalog: [],
        mcpServers: [],
        browser: session.browser,
        artifacts: session.artifacts,
        files: session.files,
        terminal: session.terminal,
        thread: session.thread,
        sessions: [session]
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, invokePayload = {}) => {
          if (command === "load_workspace") return payload();
          if (command === "load_session") return session;
          if (command === "open_browser") {
            const request = invokePayload.request || {};
            session.browser = browserState("Accepted Browser", request.target);
            return { browser: session.browser, action: { actor: request.actor || "user", action: "open-public-url", target: request.target } };
          }
          if (command === "sync_native_browser") {
            nativeRequests.push(invokePayload.request || {});
            return { ok: true, ...(invokePayload.request || {}) };
          }
          return { ok: true };
        }
      },
      event: { listen: async () => () => {} }
    };
  }, options);
}
