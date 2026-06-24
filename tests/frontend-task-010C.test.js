import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

describe("TASK-010C artifact preview type rendering", () => {
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

  it("renders supported artifact types inside the existing Browser/Preview surface", async () => {
    const cases = [
      ["html", async () => {
        const page = await pageForArtifactType(browser, server, "html");
        assert.equal(await page.locator("iframe[data-artifact-frame]").getAttribute("sandbox"), "allow-scripts");
        await page.frameLocator("iframe[data-artifact-frame]").locator("h1").waitFor();
        assert.equal(await page.frameLocator("iframe[data-artifact-frame]").locator("h1").innerText(), "Generated HTML");
        await page.close();
      }],
      ["markdown", async () => {
        const page = await pageForArtifactType(browser, server, "markdown");
        assert.equal(await page.locator("[data-artifact-renderer='markdown']").count(), 1);
        assert.equal(await page.locator("[data-artifact-renderer='markdown'] h2").innerText(), "Generated Notes");
        assert.match(await page.locator("[data-artifact-renderer='markdown']").innerText(), /rendered safely/);
        await page.close();
      }],
      ["image", async () => {
        const page = await pageForArtifactType(browser, server, "image");
        assert.equal(await page.locator("[data-artifact-renderer='image'] img").getAttribute("src"), pixelPngDataUrl);
        assert.equal(await page.locator("[data-artifact-renderer='image'] img").getAttribute("alt"), "Generated image");
        await page.close();
      }],
      ["pdf", async () => {
        const page = await pageForArtifactType(browser, server, "pdf");
        assert.equal(await page.locator("[data-artifact-renderer='pdf'] iframe").getAttribute("src"), tinyPdfDataUrl);
        await page.close();
      }],
      ["text", async () => {
        const page = await pageForArtifactType(browser, server, "text");
        assert.equal(await page.locator("[data-artifact-renderer='text'] pre").innerText(), "Plain generated notes\nSecond line");
        await page.close();
      }],
      ["json", async () => {
        const page = await pageForArtifactType(browser, server, "json");
        assert.equal(await page.locator("[data-artifact-renderer='json'] pre").innerText(), '{\n  "ok": true,\n  "count": 2\n}');
        await page.close();
      }],
      ["code", async () => {
        const page = await pageForArtifactType(browser, server, "code");
        assert.equal(await page.locator("[data-artifact-renderer='code'] code").innerText(), "export const answer = 42;");
        await page.close();
      }]
    ];

    for (const [kind, verify] of cases) {
      await verify();
    }
  });

  it("keeps untrusted HTML sandboxed and separate from general public browsing", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(page, { probe: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.locator("[data-artifact-preview]").count(), 1);
    assert.equal(await page.locator("[data-browser-preview='public']").count(), 0);
    assert.equal(await page.locator("[data-native-browser-frame]").count(), 0);
    assert.equal(await page.locator("iframe[data-artifact-frame]").getAttribute("sandbox"), "allow-scripts");

    await page.frameLocator("iframe[data-artifact-frame]").locator("#probe").waitFor();
    const probe = JSON.parse(await page.frameLocator("iframe[data-artifact-frame]").locator("#probe").innerText());
    assert.equal(probe.parentReachable, false);
    assert.equal(probe.tauriReachable, false);
    assert.equal(await page.evaluate(() => window.__task010CProbeLeak || ""), "");

    const publicPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(publicPage, { publicBrowser: true });
    await publicPage.goto(`${server.origin}/#chat-session`);
    await publicPage.getByRole("button", { name: "Browser" }).click();
    await publicPage.waitForFunction(() => window.__task010CNativeRequests.length > 0);
    assert.equal(await publicPage.locator("[data-browser-preview='public']").count(), 1);
    assert.equal(await publicPage.locator("[data-native-browser-frame]").count(), 1);
    assert.equal(await publicPage.locator("[data-artifact-preview]").count(), 0);
    await publicPage.close();
  });

  it("does not render artifacts through Terminal and preserves Files, local-file Browser preview, native public browsing, and sessions", async () => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(page, { sessions: true });

    await page.goto(`${server.origin}/#chat-session`);
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.locator("[data-artifact-renderer='markdown']").count(), 1);

    await page.getByRole("button", { name: "Terminal" }).click();
    await page.locator(".terminal-tool .xterm-screen").waitFor();
    assert.doesNotMatch(await page.locator(".terminal-tool .xterm-screen").innerText(), /alpha terminal/);
    assert.equal(await page.locator(".terminal-tool [data-artifact-renderer]").count(), 0);
    assert.equal(await page.locator(".terminal-tool iframe[data-artifact-frame]").count(), 0);

    await page.getByRole("button", { name: "Files" }).click();
    assert.match(await page.locator(".tool-panel").innerText(), /alpha\.md/);

    const localPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(localPage, { localFile: true });
    await localPage.goto(`${server.origin}/#chat-session`);
    await localPage.getByRole("button", { name: "Browser" }).click();
    await assertLocalFileUsesNativeWry(localPage);
    await localPage.close();

    const htmlPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(htmlPage, { localHtmlFile: true });
    await htmlPage.goto(`${server.origin}/#chat-session`);
    await htmlPage.getByRole("button", { name: "Browser" }).click();
    await assertLocalFileUsesNativeWry(htmlPage, "file:///workspace/wireframes/app-start.html");
    assert.equal(await htmlPage.locator("[data-local-file-renderer]").count(), 0);
    await htmlPage.close();

    const imagePage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(imagePage, { localImageFile: true });
    await imagePage.goto(`${server.origin}/#chat-session`);
    await imagePage.getByRole("button", { name: "Browser" }).click();
    await assertLocalFileUsesNativeWry(imagePage, "file:///workspace/backend/icons/icon.png");
    assert.equal(await imagePage.locator("[data-local-file-renderer='image']").count(), 0);
    await imagePage.close();

    const markdownPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(markdownPage, { localMarkdownFile: true });
    await markdownPage.goto(`${server.origin}/#chat-session`);
    await markdownPage.getByRole("button", { name: "Browser" }).click();
    assert.equal(await markdownPage.locator("[data-browser-preview='local-file']").count(), 1);
    assert.equal(await markdownPage.locator("[data-local-file-renderer='markdown']").count(), 1);
    assert.equal(await markdownPage.locator("[data-local-file-renderer='markdown'] h2").first().innerText(), "Chris MCP");
    assert.match(await markdownPage.locator("[data-local-file-renderer='markdown']").innerText(), /Install/);
    assert.equal(await markdownPage.locator("[data-browser-frame]").count(), 0);
    assert.equal(await markdownPage.locator("[data-artifact-preview]").count(), 0);
    assert.equal(await markdownPage.locator("[data-native-browser-frame]").count(), 0);
    await markdownPage.close();

    const publicPage = await browser.newPage({ viewport: { width: 1440, height: 920 } });
    await installTask010CTauri(publicPage, { publicBrowser: true });
    await publicPage.goto(`${server.origin}/#chat-session`);
    await publicPage.getByRole("button", { name: "Browser" }).click();
    await publicPage.waitForFunction(() => window.__task010CNativeRequests.length > 0);
    assert.equal(await publicPage.locator("[data-browser-preview='public']").count(), 1);
    assert.equal(await publicPage.locator("[data-native-browser-frame]").count(), 1);
    await publicPage.close();

    await page.locator("[data-session-target='Beta Artifact']").click();
    await page.getByRole("button", { name: "Browser" }).click();
    assert.equal(await page.locator("[data-artifact-renderer='json'] pre").innerText(), '{\n  "session": "beta"\n}');

    assert.deepEqual(
      await page.locator(".tool-tabs [data-tool-tab]").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim())),
      ["Browser", "Files", "Terminal"]
    );
  });
});

const pixelPngDataUrl = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgF8J9gWswAAAABJRU5ErkJggg==";
const tinyPdfDataUrl = "data:application/pdf;base64,JVBERi0xLjEKMSAwIG9iago8PC9UeXBlL0NhdGFsb2cvUGFnZXMgMiAwIFI+PgplbmRvYmoKMiAwIG9iago8PC9UeXBlL1BhZ2VzL0NvdW50IDAvS2lkc1tdPj4KZW5kb2JqCnRyYWlsZXIKPDwvUm9vdCAxIDAgUj4+CiUlRU9G";

async function pageForArtifactType(browser, server, kind) {
  const page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
  await installTask010CTauri(page, { kind });
  await page.goto(`${server.origin}/#chat-session`);
  await page.getByRole("button", { name: "Browser" }).click();
  assert.equal(await page.locator(`[data-artifact-preview="artifact-${kind}"]`).count(), 1);
  assert.equal(await page.locator("[data-browser-preview]").count(), 0);
  assert.equal(await page.locator("[data-native-browser-frame]").count(), 0);
  return page;
}

async function assertLocalFileUsesNativeWry(page, expectedUrl = null) {
  await page.waitForFunction(() => window.__task010CNativeRequests.length > 0);
  assert.equal(await page.locator("[data-browser-preview='local-file']").count(), 1);
  assert.equal(await page.locator("[data-native-browser-frame]").count(), 1);
  if (expectedUrl) {
    assert.equal(await page.locator("[data-native-browser-frame]").getAttribute("data-native-browser-url"), expectedUrl);
  }
  assert.equal(await page.locator("[data-browser-frame]").count(), 0);
  assert.equal(await page.locator("[data-artifact-preview]").count(), 0);
}

async function installTask010CTauri(page, options = {}) {
  await page.addInitScript((nextOptions) => {
    const nativeRequests = [];
    window.__task010CNativeRequests = nativeRequests;
    const parentFrame = window.parent === window;

    const probeHtml = `<!doctype html><html><body><h1>Generated HTML</h1><pre id="probe"></pre><script>
      const probe = {
        parentReachable: canRead(() => parent.__TASK010C_SECRET__),
        tauriReachable: typeof window.__TAURI__ !== "undefined"
      };
      try { parent.__task010CProbeLeak = "leaked"; } catch (error) {}
      function canRead(read) {
        try { return typeof read() !== "undefined"; } catch (error) { return false; }
      }
      document.querySelector("#probe").textContent = JSON.stringify(probe);
    </script></body></html>`;

    const artifacts = {
      html: artifact("html", "Generated HTML", "text/html", "report.html", {
        html: nextOptions.probe ? probeHtml : "<!doctype html><html><body><h1>Generated HTML</h1></body></html>"
      }),
      markdown: artifact("markdown", "Generated notes", "text/markdown", "notes.md", {
        content: "## Generated Notes\n\n- rendered safely"
      }),
      image: artifact("image", "Generated image", "image/png", "image.png", {
        dataUrl: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgF8J9gWswAAAABJRU5ErkJggg=="
      }),
      pdf: artifact("pdf", "Generated PDF", "application/pdf", "brief.pdf", {
        dataUrl: "data:application/pdf;base64,JVBERi0xLjEKMSAwIG9iago8PC9UeXBlL0NhdGFsb2cvUGFnZXMgMiAwIFI+PgplbmRvYmoKMiAwIG9iago8PC9UeXBlL1BhZ2VzL0NvdW50IDAvS2lkc1tdPj4KZW5kb2JqCnRyYWlsZXIKPDwvUm9vdCAxIDAgUj4+CiUlRU9G"
      }),
      text: artifact("text", "Plain generated notes", "text/plain", "notes.txt", {
        content: "Plain generated notes\nSecond line"
      }),
      json: artifact("json", "Generated JSON", "application/json", "data.json", {
        content: '{"ok":true,"count":2}'
      }),
      code: artifact("code", "Generated code", "application/javascript", "answer.js", {
        content: "export const answer = 42;"
      })
    };

    if (!parentFrame) return;

    const alpha = sessionRecord("alpha", "Alpha Artifact", artifacts[nextOptions.probe ? "html" : "markdown"]);
    const beta = sessionRecord("beta", "Beta Artifact", artifact("json", "Beta JSON", "application/json", "beta.json", {
      content: '{"session":"beta"}'
    }));
    let active = nextOptions.sessions ? alpha : sessionRecord("alpha", "Alpha Artifact", artifacts[nextOptions.probe ? "html" : nextOptions.kind || "html"]);
    if (nextOptions.localFile) {
      active.browser = browserState("Trusted local file", "file:///workspace/preview.html", {
        previewMode: "local-file",
        localPath: "/workspace/preview.html",
        html: ""
      });
      active.artifacts = [];
    }
    if (nextOptions.localHtmlFile) {
      active.browser = browserState("app-start.html", "file:///workspace/wireframes/app-start.html", {
        previewMode: "local-file",
        localPath: "/workspace/wireframes/app-start.html",
        html: ""
      });
      active.artifacts = [];
    }
    if (nextOptions.localImageFile) {
      active.browser = browserState("icon.png", "file:///workspace/backend/icons/icon.png", {
        previewMode: "local-file",
        localPath: "/workspace/backend/icons/icon.png",
        html: ""
      });
      active.artifacts = [];
    }
    if (nextOptions.localMarkdownFile) {
      active.browser = browserState("README.md", "file:///Users/cblanquera/server/projects/cblanquera/mcp/README.md", {
        previewMode: "local-file",
        localPath: "/Users/cblanquera/server/projects/cblanquera/mcp/README.md",
        html: "# Chris MCP\n\nA context provider on how I program.\n\n## Install\n\nUse Node version 22."
      });
      active.artifacts = [];
    }
    if (nextOptions.publicBrowser) {
      active.browser = browserState("Public page", "https://example.com/");
      active.artifacts = [];
    }

    window.__TASK010C_SECRET__ = "secret";
    window.__task010CSetArtifact = (kind) => {
      active = sessionRecord("alpha", "Alpha Artifact", artifacts[kind]);
      render();
    };
    window.__task010CSetLocalFile = () => {
      active.browser = browserState("Trusted local file", "file:///workspace/preview.html", {
        previewMode: "local-file",
        localPath: "/workspace/preview.html",
        html: ""
      });
      active.artifacts = [];
    };
    window.__task010CSetPublicBrowser = () => {
      active.browser = browserState("Public page", "https://example.com/");
      active.artifacts = [];
    };

    function artifact(kind, title, mimeType, filename, preview) {
      return {
        id: `artifact-${kind}`,
        title,
        kind,
        origin: "generated",
        mimeType,
        filename,
        safePreview: {
          url: `artifact://${kind}`,
          title,
          summary: `${title} preview`,
          ...preview
        }
      };
    }

    function browserState(label, url, overrides = {}) {
      return {
        url,
        title: label,
        summary: `${label} Browser state`,
        previewMode: "public",
        profileId: "profile-task-010c",
        localPath: "",
        artifactId: "",
        html: "",
        ...overrides
      };
    }

    function sessionRecord(id, title, previewArtifact) {
      return {
        id,
        project: "artifact-type-repo",
        title,
        selectedModel: "model/artifact",
        browser: browserState(previewArtifact.title, previewArtifact.safePreview.url, {
          previewMode: "artifact",
          artifactId: previewArtifact.id
        }),
        artifacts: [previewArtifact],
        files: { roots: [["alpha.md", "file", "file-editor", "alpha.md", "0"]], breadcrumbs: ["artifact-type-repo", "alpha.md"], lines: ["# alpha.md"], currentPath: "alpha.md", content: "# alpha.md", savedContent: "# alpha.md", dirty: false },
        terminal: { output: `${id} terminal`, title: "Terminal", summary: "Terminal remains separate" },
        thread: { user: title, agent: title, extra: "", tool: "", run: "" },
        turns: [{ id: `${id}-turn`, user: title, agent: title, extra: "", tool: "", run: "" }],
        browserActions: []
      };
    }

    function payload() {
      return {
        workspace: { project: "artifact-type-repo", session: active.title, branch: "main", model: "model/artifact", sessionId: active.id },
        projects: [{ name: "artifact-type-repo", sessions: nextOptions.sessions ? [{ id: alpha.id, label: alpha.title }, { id: beta.id, label: beta.title }] : [{ id: active.id, label: active.title }] }],
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
        sessions: nextOptions.sessions ? [alpha, beta] : [active]
      };
    }

    window.__TAURI__ = {
      core: {
        invoke: async (command, invokePayload = {}) => {
          if (command === "load_workspace") return payload();
          if (command === "load_session") {
            active = invokePayload.request.sessionId === "beta" ? beta : alpha;
            return active;
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
