import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";
import { startMockServer } from "./server/mock-server.js";

describe("TASK-002 mock-backed frontend integration", () => {
  let browser;
  let frontend;
  let mock;
  let page;

  before(async () => {
    mock = await startMockServer();
    frontend = await startFrontendServer();
    browser = await chromium.launch();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
  });

  after(async () => {
    await browser?.close();
    await frontend?.close();
    await mock?.close();
  });

  it("renders loading and then replaces app-start fixtures with mock workspace state", async () => {
    await page.goto(`${frontend.origin}/?connector=server&api=${encodeURIComponent(mock.origin)}&delay=120#app-start`);
    await page.getByText("Loading workspace state").waitFor();
    await page.getByText("Mock Workspace Alpha").waitFor();
    await page.getByText("Mock Agent Lab").waitFor();
    assert.equal(await page.locator("[data-app-root]").count(), 1);
    assert.equal(await page.locator(".start-view").count(), 1);
  });

  it("renders failure state inside the accepted app-start surface when mock boot fails", async () => {
    await page.goto(`${frontend.origin}/?connector=server&api=${encodeURIComponent(mock.origin)}&scenario=boot-failure#app-start`);
    await page.getByText("Workspace state unavailable").waitFor();
    await page.getByText("Mock boot failure").waitFor();
    assert.equal(await page.locator(".start-view").count(), 1);
    assert.equal(await page.getByRole("button", { name: "Open Folder" }).count(), 1);
  });

  it("renders mock-backed waiting, success, and state transition behavior in the chat route", async () => {
    await page.goto(`${frontend.origin}/?connector=server&api=${encodeURIComponent(mock.origin)}&delay=90#chat-session`);
    await page.getByText("Mock Workspace Alpha").waitFor();
    await page.locator(".prompt-box").fill("Summarize the mock run state");
    await page.getByRole("button", { name: "Send Prompt" }).click();

    await page.getByText("Summarize the mock run state").waitFor();
    await page.getByText("Waiting on connector").waitFor();
    await page.getByText("Mock agent completed the requested transition.").waitFor();
    assert.equal(await page.getByText("Waiting on mock agent").count(), 0);
  });

  it("renders mock-backed failure state without adding route structure or controls", async () => {
    await page.goto(`${frontend.origin}/?connector=server&api=${encodeURIComponent(mock.origin)}&scenario=run-failure#chat-session`);
    await page.getByText("Mock Workspace Alpha").waitFor();
    const controlsBefore = await visibleControlNames(page);

    await page.locator(".prompt-box").fill("Trigger a failed mock run");
    await page.getByRole("button", { name: "Send Prompt" }).click();
    await page.getByText("Mock agent failed before producing output.").waitFor();

    assert.equal(await page.locator(".app-shell").count(), 1);
    assert.deepEqual(await visibleControlNames(page), controlsBefore);
  });
});

async function visibleControlNames(page) {
  return page.evaluate(() => {
    return Array.from(document.querySelectorAll("button, a"))
      .filter((node) => {
        const style = getComputedStyle(node);
        return style.display !== "none" && style.visibility !== "hidden" && !node.hidden && !node.closest(".work-log");
      })
      .map((node) => node.getAttribute("aria-label") || node.textContent.trim())
      .filter(Boolean);
  });
}
