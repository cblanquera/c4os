import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";
import { chromium } from "playwright";
import { startFrontendServer } from "./support/browser-harness/server.js";

const routes = [
  "app-start",
  "new-session",
  "chat-session",
  "providers-popover",
  "models-popover",
  "file-explorer",
  "file-editor",
  "terminal",
  "settings-providers",
  "settings-add-provider",
  "settings-models",
  "settings-runtimes",
  "settings-configuration",
  "settings-plugins",
  "settings-skills",
  "settings-mcp"
];

describe("TASK-001 r04 frontend parity", () => {
  let browser;
  let page;
  let server;

  before(async () => {
    server = await startFrontendServer();
    browser = await chromium.launch();
    page = await browser.newPage({ viewport: { width: 1440, height: 920 } });
  });

  after(async () => {
    await browser?.close();
    await server?.close();
  });

  async function goto(route) {
    await page.goto(`${server.origin}/#${route}`);
    await page.waitForSelector(`[data-route="${route}"]`, { timeout: 1500 });
  }

  it("renders every r04 route with the expected top-level surface", async () => {
    for (const route of routes) {
      await goto(route);
      assert.equal(await page.locator("[data-app-root]").count(), 1, route);
      await assertRouteStructure(page, route);
    }
  });

  it("preserves shell collapse and resize interactions", async () => {
    await goto("new-session");

    const shell = page.locator(".app-shell");
    const leftPanel = page.locator(".sidebar");
    const rightPanel = page.locator(".tool-panel");
    const leftBefore = await leftPanel.boundingBox();
    const rightBefore = await rightPanel.boundingBox();

    await page.getByRole("button", { name: "Collapse left panel" }).click();
    assert.equal(await shell.evaluate((node) => node.classList.contains("is-left-collapsed")), true);
    await page.getByRole("button", { name: "Collapse right panel" }).click();
    assert.equal(await shell.evaluate((node) => node.classList.contains("is-right-collapsed")), true);
    await page.getByRole("button", { name: "Collapse left panel" }).click();
    await page.getByRole("button", { name: "Collapse right panel" }).click();

    await drag(page.locator("[data-resize-panel='left']"), 70, 0);
    await drag(page.locator("[data-resize-panel='right']"), -70, 0);

    const leftAfter = await leftPanel.boundingBox();
    const rightAfter = await rightPanel.boundingBox();
    assert.notEqual(Math.round(leftBefore.width), Math.round(leftAfter.width));
    assert.notEqual(Math.round(rightBefore.width), Math.round(rightAfter.width));
  });

  it("keeps resized right panel within the visible viewport", async () => {
    try {
      await page.setViewportSize({ width: 1200, height: 820 });
      await goto("new-session");

      await drag(page.locator("[data-resize-panel='right']"), -520, 0);
      const rightPanel = await page.locator(".tool-panel").boundingBox();
      assert.equal(rightPanel.x + rightPanel.width <= 1200, true);
    } finally {
      await page.setViewportSize({ width: 1440, height: 920 });
    }
  });

  it("lets side panels overlay instead of clipping the workbench on narrow windows", async () => {
    try {
      await page.setViewportSize({ width: 900, height: 760 });
      await goto("new-session");

      const layout = await page.evaluate(() => {
        const read = (selector) => {
          const box = document.querySelector(selector).getBoundingClientRect();
          return {
            left: Math.round(box.left),
            right: Math.round(box.right),
            width: Math.round(box.width)
          };
        };
        return {
          app: read(".app-shell"),
          composer: read(".composer"),
          left: read(".sidebar"),
          right: read(".tool-panel"),
          viewport: window.innerWidth,
          workbench: read(".workbench")
        };
      });

      assert.equal(layout.app.width, 900);
      assert.equal(layout.workbench.left, 0);
      assert.equal(layout.workbench.width, 900);
      assert.equal(layout.composer.width >= 620, true);
      assert.equal(layout.left.right > layout.composer.left, true);
      assert.equal(layout.right.left < layout.composer.right, true);

      await page.getByRole("button", { name: "Collapse left panel" }).click();
      await page.getByRole("button", { name: "Collapse right panel" }).click();
      const visibleComposer = await page.locator(".composer").boundingBox();
      assert.equal(visibleComposer.x >= 0, true);
      assert.equal(visibleComposer.x + visibleComposer.width <= 900, true);
    } finally {
      await page.setViewportSize({ width: 1440, height: 920 });
    }
  });

  it("preserves composer attachments and popovers", async () => {
    await goto("new-session");

    await page.locator(".attachment-input").setInputFiles({
      name: "brief.md",
      mimeType: "text/markdown",
      buffer: Buffer.from("# Frontend context")
    });
    await page.getByText("brief.md").waitFor();
    await page.getByRole("button", { name: "Remove attachment: brief.md" }).click();
    assert.equal(await page.getByText("brief.md").count(), 0);

    await page.getByRole("button", { name: /Approval policy/ }).click();
    await page.getByRole("button", { name: "Ask for approval" }).click();
    await page.getByRole("button", { name: /Ask for approval/ }).waitFor();

    await page.getByRole("button", { name: /Branch:/ }).click();
    await page.getByRole("button", { name: "feature/trust-shell" }).click();
    await page.getByRole("button", { name: /feature\/trust-shell/ }).waitFor();
  });

  it("preserves provider and model popover routes", async () => {
    await goto("providers-popover");
    assert.equal(await page.getByRole("complementary", { name: "Provider selector" }).isVisible(), true);
    await page.getByText("Providers").waitFor();

    await goto("models-popover");
    assert.equal(await page.getByRole("complementary", { name: "Model selector" }).isVisible(), true);
    assert.equal(await page.getByRole("complementary", { name: "Model selector" }).isVisible(), true);
  });

  it("preserves chat message disclosure", async () => {
    await goto("chat-session");
    const extra = page.locator(".message-extra");
    const workBefore = await page.locator(".work-log").first().boundingBox();
    assert.equal(await extra.isVisible(), false);
    await page.getByRole("button", { name: "Show More" }).click();
    assert.equal(await extra.isVisible(), true);
    const messageAfter = await page.locator(".message.agent").boundingBox();
    const workAfter = await page.locator(".work-log").first().boundingBox();
    assert.equal(messageAfter.y > workAfter.y + workAfter.height + 14, true);
    assert.equal(Math.abs(workAfter.y - workBefore.y) <= 10, true);
    await page.getByRole("button", { name: "Show Less" }).click();
    assert.equal(await extra.isVisible(), false);
  });

  it("prevents expanded chat bubbles from clipping or overlapping activity cards", async () => {
    try {
      await page.setViewportSize({ width: 900, height: 720 });
      await goto("chat-session");
      const shell = page.locator(".app-shell");
      if (!(await shell.evaluate((node) => node.classList.contains("is-left-collapsed")))) {
        await page.getByRole("button", { name: "Collapse left panel" }).click();
      }
      if (!(await shell.evaluate((node) => node.classList.contains("is-right-collapsed")))) {
        await page.getByRole("button", { name: "Collapse right panel" }).click();
      }
      await page.getByRole("button", { name: "Show More" }).click();

      const layout = await page.evaluate(() => {
        const message = document.querySelector(".message.agent").getBoundingClientRect();
        const extra = document.querySelector(".message-extra").getBoundingClientRect();
        const work = document.querySelector(".work-log").getBoundingClientRect();
        return {
          workBottom: work.bottom,
          extraBottom: extra.bottom,
          messageTop: message.top,
          messageBottom: message.bottom
        };
      });

      assert.equal(layout.extraBottom <= layout.messageBottom - 16, true);
      assert.equal(layout.messageTop >= layout.workBottom + 18, true);
    } finally {
      await page.setViewportSize({ width: 1440, height: 920 });
    }
  });

  it("aligns the chat prompt with the outside edges of the chat bubbles", async () => {
    await goto("chat-session");
    await page.getByRole("button", { name: "Collapse left panel" }).click();
    await page.getByRole("button", { name: "Collapse right panel" }).click();

    const edges = await page.evaluate(() => {
      const read = (selector) => {
        const box = document.querySelector(selector).getBoundingClientRect();
        return {
          left: Math.round(box.left),
          right: Math.round(box.right)
        };
      };
      return {
        agent: read(".message.agent"),
        composer: read(".composer-dock .composer"),
        user: read(".message.user")
      };
    });

    assert.equal(edges.composer.left, edges.agent.left);
    assert.equal(edges.composer.right, edges.user.right);
  });

  it("preserves the Terminal tab inside the accepted right panel", async () => {
    await goto("terminal");
    await page.locator("[data-terminal-emulator]").waitFor();
    await page.locator(".tool-panel [data-tool-tab].is-active", { hasText: "Terminal" }).waitFor();
    assert.equal(await page.locator("[data-terminal-form]").count(), 0);
    assert.equal(await page.locator(".terminal-tool [data-browser-preview]").count(), 0);
    assert.equal(await page.locator(".terminal-tool .files-list").count(), 0);
  });

  it("preserves plugin, skill, and MCP dialogs plus transport switching", async () => {
    await goto("settings-plugins");
    await page.getByRole("button", { name: "Add GitHub" }).click();
    const pluginDialog = page.getByRole("dialog", { name: "Connect GitHub" });
    await pluginDialog.waitFor();
    await pluginDialog.getByText("AI", { exact: true }).waitFor();
    await pluginDialog.getByText("GH", { exact: true }).waitFor();
    await pluginDialog.getByText("You're in control").waitFor();
    await pluginDialog.getByText("Plugins may introduce elevated risk").waitFor();
    await pluginDialog.getByText("Data shared with this plugin").waitFor();
    assert.equal(await pluginDialog.locator(".plugin-risk-block").count(), 3);
    const riskPanelColumns = await pluginDialog.locator(".plugin-risk-grid").evaluate((node) => {
      return getComputedStyle(node).gridTemplateColumns.split(" ").length;
    });
    assert.equal(riskPanelColumns, 1);
    const continueBox = await pluginDialog.getByRole("button", { name: "Continue to GitHub" }).boundingBox();
    const panelBox = await pluginDialog.boundingBox();
    assert.equal(continueBox.width / panelBox.width > 0.8, true);
    const modalScale = await pluginDialog.evaluate((dialog) => {
      const title = dialog.querySelector("#plugin-connect-title");
      const body = dialog.querySelector(".plugin-risk-block p");
      const button = dialog.querySelector(".plugin-modal-continue");
      return {
        buttonFont: Number.parseFloat(getComputedStyle(button).fontSize),
        bodyFont: Number.parseFloat(getComputedStyle(body).fontSize),
        titleFont: Number.parseFloat(getComputedStyle(title).fontSize)
      };
    });
    assert.equal(modalScale.titleFont <= 24, true);
    assert.equal(modalScale.bodyFont <= 14, true);
    assert.equal(modalScale.buttonFont <= 15, true);
    await pluginDialog.getByRole("button", { name: "Advanced settings" }).waitFor();
    await page.keyboard.press("Escape");

    const marketplaceTrigger = page.getByRole("button", { name: /Built by C4OS/ });
    await marketplaceTrigger.click();
    assert.equal(await marketplaceTrigger.getAttribute("aria-expanded"), "true");
    const marketplaceMenu = page.getByRole("menu", { name: "Plugin marketplaces" });
    await marketplaceMenu.waitFor();
    const defaultMarketplace = marketplaceMenu.getByRole("menuitemradio", { name: /Built by C4OS/ });
    await defaultMarketplace.waitFor();
    assert.equal(await defaultMarketplace.getAttribute("aria-checked"), "true");
    await marketplaceMenu.getByRole("menuitem", { name: "Add Marketplace" }).click();
    await page.getByRole("dialog", { name: "Add plugin marketplace" }).waitFor();
    await page.keyboard.press("Escape");

    await goto("settings-skills");
    await page.getByRole("button", { name: /ChrisAI Agents/ }).click();
    const skillDialog = page.getByRole("dialog", { name: "ChrisAI Agents" });
    await skillDialog.waitFor();
    await skillDialog.getByText("Skill", { exact: true }).waitFor();
    await skillDialog.getByRole("button", { name: "Skill enabled" }).waitFor();
    await skillDialog.getByRole("button", { name: "More skill actions" }).waitFor();
    await skillDialog.getByText("Scope").waitFor();
    await skillDialog.getByText("Project availability").waitFor();
    await page.keyboard.press("Escape");

    await goto("settings-mcp");
    await page.getByRole("button", { name: "Learn more about MCP Servers" }).waitFor();
    await page.getByRole("button", { name: "Add server" }).click();
    await page.getByRole("dialog", { name: "Connect to a custom MCP" }).waitFor();
    await page.getByRole("tab", { name: "Streamable HTTP" }).click();
    assert.equal(await page.locator("[data-mcp-fields='http']").isVisible(), true);
    await page.keyboard.press("Escape");
  });

  it("preserves r04 settings form controls and field structures", async () => {
    await goto("settings-add-provider");
    const providerForm = await page.evaluate(() => {
      const labels = Array.from(document.querySelectorAll(".provider-form label"));
      return labels.map((label) => ({
        control: label.querySelector("select, input, textarea")?.tagName.toLowerCase(),
        text: label.querySelector("span, label")?.textContent?.trim() || label.textContent.trim()
      }));
    });
    assert.deepEqual(providerForm, [
      { control: "select", text: "Provider Type" },
      { control: "input", text: "Label" },
      { control: "input", text: "API Base URL" },
      { control: "input", text: "API Key" },
      { control: "select", text: "Auth" },
      { control: "textarea", text: "Headers" }
    ]);

    await goto("settings-plugins");
    await page.getByRole("button", { name: /Built by C4OS/ }).click();
    await page.getByRole("menuitem", { name: "Add Marketplace" }).click();
    const marketplaceForm = await page.evaluate(() => {
      return Array.from(document.querySelectorAll(".marketplace-field")).map((field) => ({
        control: field.querySelector("input, textarea")?.tagName.toLowerCase(),
        text: field.querySelector("label, span")?.textContent?.trim()
      }));
    });
    assert.deepEqual(marketplaceForm, [
      { control: "input", text: "Source" },
      { control: "input", text: "Git ref" },
      { control: "textarea", text: "Sparse paths" }
    ]);
    await page.keyboard.press("Escape");

    await goto("settings-mcp");
    await page.getByRole("button", { name: "Add server" }).click();
    const stdioForm = await page.evaluate(() => ({
      addButtons: Array.from(document.querySelectorAll("[data-mcp-fields='stdio'] .mcp-add-line")).map((button) => button.textContent.trim()),
      fields: Array.from(document.querySelectorAll("[data-mcp-fields='stdio'] .mcp-field > span")).map((node) => node.textContent.trim()),
      groups: Array.from(document.querySelectorAll("[data-mcp-fields='stdio'] .mcp-group h3")).map((node) => node.textContent.trim())
    }));
    assert.deepEqual(stdioForm.fields, ["Command to launch", "Working directory"]);
    assert.deepEqual(stdioForm.groups, ["Arguments", "Environment variables", "Environment variable passthrough"]);
    assert.deepEqual(stdioForm.addButtons, ["+ Add argument", "+ Add environment variable", "+ Add variable"]);

    await page.getByRole("tab", { name: "Streamable HTTP" }).click();
    const httpForm = await page.evaluate(() => ({
      addButtons: Array.from(document.querySelectorAll("[data-mcp-fields='http'] .mcp-add-line")).map((button) => button.textContent.trim()),
      fields: Array.from(document.querySelectorAll("[data-mcp-fields='http'] .mcp-field > span")).map((node) => node.textContent.trim()),
      groups: Array.from(document.querySelectorAll("[data-mcp-fields='http'] .mcp-group h3")).map((node) => node.textContent.trim())
    }));
    assert.deepEqual(httpForm.fields, ["URL", "Bearer token env var"]);
    assert.deepEqual(httpForm.groups, ["Headers", "Headers from environment variables"]);
    assert.deepEqual(httpForm.addButtons, ["+ Add header", "+ Add variable"]);
    await page.keyboard.press("Escape");
  });

  it("supports provider-specific fields and dynamic settings form rows", async () => {
    await goto("settings-add-provider");
    const providerTypes = await page.locator("#provider-type option").evaluateAll((options) => {
      return options.map((option) => option.textContent.trim());
    });
    assert.deepEqual(providerTypes, [
      "OpenAI",
      "OpenRouter",
      "Hugging Face router",
      "LiteLLM proxy",
      "Custom OpenAI-compatible endpoint"
    ]);

    await page.locator("#provider-type").selectOption("OpenRouter");
    let visibleFields = await visibleFieldLabels(page, ".provider-form .field");
    assert.deepEqual(visibleFields, ["Provider Type", "Label", "API Key"]);

    await page.locator("#provider-type").selectOption("Hugging Face router");
    visibleFields = await visibleFieldLabels(page, ".provider-form .field");
    assert.deepEqual(visibleFields, ["Provider Type", "Label", "API Key"]);

    await page.locator("#provider-type").selectOption("LiteLLM proxy");
    visibleFields = await visibleFieldLabels(page, ".provider-form .field");
    assert.deepEqual(visibleFields, [
      "Provider Type",
      "Label",
      "API Base URL",
      "API Key",
      "Auth",
      "Headers"
    ]);

    await page.locator("#provider-type").selectOption("Custom OpenAI-compatible endpoint");
    visibleFields = await visibleFieldLabels(page, ".provider-form .field");
    assert.deepEqual(visibleFields, [
      "Provider Type",
      "Label",
      "API Base URL",
      "API Key",
      "Auth",
      "Headers"
    ]);

    await goto("settings-mcp");
    await page.getByRole("button", { name: "Add server" }).click();
    const mcpDialog = page.getByRole("dialog", { name: "Connect to a custom MCP" });
    assert.equal(await mcpDialog.getByText("Docs", { exact: true }).count(), 0);

    await page.getByRole("tab", { name: "Streamable HTTP" }).click();
    assert.equal(await page.locator("[data-mcp-fields='stdio']").isVisible(), false);
    assert.equal(await page.locator("[data-mcp-fields='http']").isVisible(), true);

    const headerRows = page.locator("[data-mcp-group='Headers'] .mcp-pair-row");
    assert.equal(await headerRows.count(), 1);
    await page.getByRole("button", { name: "+ Add header" }).click();
    assert.equal(await headerRows.count(), 2);
    await page.getByRole("button", { name: "Remove Headers row 2" }).click();
    assert.equal(await headerRows.count(), 1);
    await page.keyboard.press("Escape");
  });

  it("renders the plugin marketplace picker as a source list", async () => {
    await goto("settings-plugins");

    const trigger = page.getByRole("button", { name: /Built by C4OS/ });
    await trigger.click();

    const menu = page.getByRole("menu", { name: "Plugin marketplaces" });
    const marketplace = menu.getByRole("menuitemradio", { name: /Built by C4OS/ });
    const addMarketplace = menu.getByRole("menuitem", { name: "Add Marketplace" });
    await marketplace.waitFor();
    await addMarketplace.waitFor();

    const menuState = await menu.evaluate((node) => {
      const option = node.querySelector(".marketplace-option");
      const action = node.querySelector(".marketplace-add-option");
      const optionStyles = getComputedStyle(option);
      const actionStyles = getComputedStyle(action);
      return {
        actionBorder: actionStyles.borderStyle,
        actionBorderWidth: actionStyles.borderTopWidth,
        actionDisplay: actionStyles.display,
        actionWidth: Math.round(action.getBoundingClientRect().width),
        checked: option.getAttribute("aria-checked"),
        menuWidth: Math.round(node.getBoundingClientRect().width),
        optionClass: option.className,
        optionDisplay: optionStyles.display
      };
    });

    assert.equal(menuState.checked, "true");
    assert.match(menuState.optionClass, /is-selected/);
    assert.equal(menuState.optionDisplay, "flex");
    assert.equal(menuState.actionDisplay, "flex");
    assert.equal(menuState.actionBorder, "none");
    assert.equal(menuState.actionBorderWidth, "0px");
    assert.equal(menuState.actionWidth >= menuState.menuWidth - 14, true);
  });

  it("normalizes form typography, sticky modal headers, and editable file editor", async () => {
    await goto("settings-mcp");
    await page.getByRole("button", { name: "Add server" }).click();
    await page.getByRole("tab", { name: "Streamable HTTP" }).click();

    const formStyles = await page.evaluate(() => {
      const input = document.querySelector("#mcp-url");
      const header = document.querySelector(".mcp-dialog-head");
      const panel = document.querySelector(".mcp-panel");
      return {
        headerPosition: getComputedStyle(header).position,
        inputWeight: getComputedStyle(input).fontWeight,
        panelPaddingTop: getComputedStyle(panel).paddingTop
      };
    });
    assert.equal(formStyles.headerPosition, "sticky");
    assert.equal(Number(formStyles.inputWeight) <= 450, true);
    assert.equal(formStyles.panelPaddingTop, "0px");
    await page.keyboard.press("Escape");

    await goto("settings-providers");
    const navSpacing = await page.evaluate(() => {
      const back = document.querySelector(".settings-back").getBoundingClientRect();
      const kicker = document.querySelector(".settings-nav .kicker").getBoundingClientRect();
      return Math.round(kicker.top - back.bottom);
    });
    assert.equal(navSpacing >= 18, true);

    await goto("file-editor");
    const editorState = await page.evaluate(() => {
      const lineNumber = document.querySelector(".line-number");
      const code = document.querySelector(".line-code");
      const lineBox = lineNumber.getBoundingClientRect();
      const paneBox = document.querySelector(".code-pane").getBoundingClientRect();
      return {
        codeEditable: code.getAttribute("contenteditable"),
        gutterOffset: Math.round(lineBox.left - paneBox.left),
        numberPosition: getComputedStyle(lineNumber).position
      };
    });
    assert.equal(editorState.codeEditable, "true");
    assert.equal(editorState.numberPosition, "sticky");
    assert.equal(editorState.gutterOffset <= 8, true);
  });

  it("keeps primary button hover and modal close controls visually stable", async () => {
    await goto("settings-mcp");

    const primary = page.getByRole("button", { name: "Add server" });
    const primaryBefore = await primary.evaluate((node) => getComputedStyle(node).backgroundColor);
    await primary.hover();
    const primaryAfter = await primary.evaluate((node) => getComputedStyle(node).backgroundColor);
    assert.equal(primaryAfter, primaryBefore);

    await primary.click();
    const close = page.getByRole("button", { name: "Close MCP server form" });
    const closeMetrics = await close.evaluate((node) => {
      const styles = getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return {
        background: styles.backgroundColor,
        borderRadius: styles.borderRadius,
        height: Math.round(rect.height),
        width: Math.round(rect.width)
      };
    });

    assert.equal(closeMetrics.height, 32);
    assert.equal(closeMetrics.width, 32);
    assert.equal(closeMetrics.borderRadius, "999px");
    assert.match(closeMetrics.background, /^rgb/);
  });

  it("renders production treatment instead of grayscale wireframe styling", async () => {
    await goto("new-session");
    const tokens = await page.evaluate(() => {
      const styles = getComputedStyle(document.documentElement);
      return {
        canvas: styles.getPropertyValue("--canvas").trim(),
        accent: styles.getPropertyValue("--accent").trim(),
        rail: styles.getPropertyValue("--rail").trim()
      };
    });

    assert.notEqual(tokens.canvas, "#ffffff");
    assert.notEqual(tokens.rail, "#eeeeee");
    assert.match(tokens.accent, /^#/);
  });

  it("captures screenshot evidence for every r04 route", async () => {
    const outputDir = join(process.cwd(), "tests", "support", "task-001-screenshots");
    await mkdir(outputDir, { recursive: true });
    for (const route of routes) {
      await goto(route);
      await page.screenshot({ path: join(outputDir, `${route}.png`), fullPage: false });
    }
  });
});

async function assertRouteStructure(page, route) {
  if (route === "app-start") {
    assert.equal(await page.locator("[data-screen='app-start']").count(), 1);
    assert.equal(await page.locator("text=Open Folder").count(), 1);
    return;
  }
  if (route.startsWith("settings")) {
    assert.equal(await page.locator(".settings-shell").count(), 1);
    assert.equal(await page.locator(".settings-link").count(), 7);
    return;
  }

  assert.equal(await page.locator(".app-shell").count(), 1);
  assert.equal(await page.locator(".sidebar").count(), 1);
  assert.equal(await page.locator(".workbench").count(), 1);
  assert.equal(await page.locator(".tool-panel").count(), 1);
}

async function visibleFieldLabels(page, selector) {
  return page.evaluate((fieldSelector) => {
    return Array.from(document.querySelectorAll(fieldSelector))
      .filter((field) => getComputedStyle(field).display !== "none")
      .map((field) => field.querySelector("span, label")?.textContent?.trim());
  }, selector);
}

async function drag(locator, deltaX, deltaY) {
  const box = await locator.boundingBox();
  assert.ok(box, "expected draggable element to have a box");
  const x = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await locator.page().mouse.move(x, y);
  await locator.page().mouse.down();
  await locator.page().mouse.move(x + deltaX, y + deltaY, { steps: 8 });
  await locator.page().mouse.up();
}
