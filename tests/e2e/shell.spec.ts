import { expect, test, type Page } from "@playwright/test";

const routes = [
  ["/onboarding", "Connect your AI provider"],
  ["/start", "What would you like to open?"],
  ["/chat", "Chat"],
  ["/chat-search", "Search Chat Sessions"],
  ["/chat-capabilities", "Capability-aware Chat"],
  ["/files", "Files"],
  ["/browser", "Browser"],
  ["/terminal", "Terminal"],
  ["/settings/providers", "Providers"],
  ["/settings/models", "Models"],
  ["/settings/runtimes", "Runtimes"],
  ["/settings/configuration", "Configuration"],
  ["/settings/plugins", "Plugins"],
  ["/settings/skills", "Skills"],
  ["/settings/mcp", "MCP Servers"],
  ["/settings/advanced-policies", "Advanced Policies"],
] as const;

test("QA adapters exercise the production onboarding and Workspace Start controllers", async ({
  page,
}) => {
  await page.goto("/#/onboarding");
  await expect(
    page.locator('[data-qa-product-adapter="deterministic"]'),
  ).toHaveAttribute("data-route", "/onboarding");

  await page.getByRole("textbox", { name: "Profile label" }).fill("QA OpenAI");
  await page
    .getByRole("textbox", { name: "API key" })
    .fill("qa-renderer-only-key");
  await page.getByRole("button", { name: "Test Connection" }).click();
  await expect(page.getByText("Connection passed")).toBeVisible();
  await page.getByRole("button", { name: "Continue" }).click();

  await expect(page).toHaveURL(/#\/start$/);
  await expect(
    page.locator('[data-qa-product-adapter="deterministic"]'),
  ).toHaveAttribute("data-route", "/start");
  await page.getByRole("button", { name: /AI Desktop UI/ }).click();
  await expect(page).toHaveURL(/#\/chat$/);
  await expect(page.getByRole("region", { name: "Chat" })).toBeVisible();
});

async function expectNoDocumentOverflow(page: Page) {
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          document.documentElement.scrollWidth <=
          document.documentElement.clientWidth,
      ),
    )
    .toBe(true);
}

test("all accepted product routes are directly addressable", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  for (const [path, title] of routes) {
    await page.goto(`/#${path}`);
    if (path === "/chat") {
      await expect(page.getByRole("region", { name: title })).toBeVisible();
    } else {
      await expect(
        page.getByRole("heading", { name: title, level: 1 }),
      ).toBeVisible();
    }
    await expect(page.locator(`[data-route="${path}"]`)).toBeVisible();
    await expectNoDocumentOverflow(page);
  }

  expect(consoleErrors).toEqual([]);
});

test("Settings Back restores exact workspace drafts, panel state, and focus", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/chat");

  const composer = page.getByRole("textbox", { name: "Message" });
  await composer.fill("Round-trip draft remains exact");
  await page.getByRole("button", { name: "Collapse project panel" }).click();
  await page.getByRole("button", { name: "Settings" }).click();
  await expect(page).toHaveURL(/#\/settings\/providers$/);

  await page.getByRole("button", { name: "Back to C4OS" }).click();
  await expect(page).toHaveURL(/#\/chat$/);
  await expect(page.getByRole("textbox", { name: "Message" })).toHaveValue(
    "Round-trip draft remains exact",
  );
  await expect(
    page.getByRole("button", { name: "Show project panel" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Settings" })).toBeFocused();

  await page.getByRole("button", { name: "Show project panel" }).click();
  const resizer = page.getByRole("separator", {
    name: "Resize project panel",
  });
  await resizer.focus();
  await expect(resizer).toHaveAttribute("aria-valuenow", "228");
  await expect(resizer).toHaveAttribute("aria-valuemax", "704");
  await page.keyboard.press("ArrowRight");
  await expect(resizer).toHaveAttribute("aria-valuenow", "240");

  await page.screenshot({
    path: "output/native/task-00006-chat-settings-roundtrip.png",
    fullPage: true,
  });
});

test("responsive overlay, compressed Settings, and deferred gates stay explicit", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto("/#/chat");
  const panel = page.locator("aside.shell-project-panel");
  let resizer = page.getByRole("separator", {
    name: "Resize project panel",
  });
  await resizer.focus();
  await page.keyboard.press("End");
  await expect(resizer).toHaveAttribute("aria-valuenow", "704");
  await expect(resizer).toHaveAttribute("aria-valuemax", "704");
  await page.getByRole("button", { name: "Collapse project panel" }).click();
  await page.setViewportSize({ width: 760, height: 720 });
  await page.getByRole("button", { name: "Show project panel" }).click();
  await expect(panel).toHaveAttribute("aria-hidden", "false");
  resizer = page.getByRole("separator", { name: "Resize project panel" });
  await expect(resizer).toHaveAttribute("aria-valuenow", "340");
  await expect(resizer).toHaveAttribute("aria-valuemax", "340");
  await page.setViewportSize({ width: 700, height: 720 });
  await expect(resizer).toHaveAttribute("aria-valuenow", "280");
  await expect(resizer).toHaveAttribute("aria-valuemax", "280");
  await page.locator("main.shell-workspace__stage").click({
    position: { x: 600, y: 300 },
  });
  await expect(panel).toHaveAttribute("aria-hidden", "true");

  await page.goto("/#/browser");
  await expect(page.getByRole("button", { name: "New tab" })).toBeDisabled();
  await page.goto("/#/terminal");
  await expect(
    page.getByRole("button", { name: "Enter full screen" }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Enter password" }),
  ).toBeDisabled();
  await page.goto("/#/chat");
  await expect(page.getByRole("button", { name: "Detach Chat" })).toHaveCount(
    0,
  );
  await page.setViewportSize({ width: 620, height: 720 });
  await page.goto("/#/settings/providers");
  await expect(page.locator(".shell-settings__body")).toHaveCSS(
    "grid-template-columns",
    "64px 556px",
  );
  await expectNoDocumentOverflow(page);
  await page.screenshot({
    path: "output/native/task-00006-settings-compressed.png",
    fullPage: true,
  });
});

for (const width of [1440, 993, 992, 680, 620, 390]) {
  test(`contains shell content and fixed composer at ${width}px`, async ({
    page,
  }) => {
    await page.setViewportSize({ width, height: 760 });
    await page.goto("/#/chat");
    await expect(page.getByRole("region", { name: "Chat" })).toBeVisible();
    await expect(
      page.getByRole("form", { name: "Message composer" }),
    ).toBeVisible();
    await expect(page.locator('[data-shell-region="right-panel"]')).toHaveCount(
      0,
    );
    await expectNoDocumentOverflow(page);
  });
}

test("reduced motion and long projected content remain contained", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/#/chat");
  await expect(page.locator("html")).toHaveAttribute(
    "data-color-scheme",
    "dark",
  );
  const duration = await page
    .getByRole("button", { name: "Settings" })
    .evaluate((element) => getComputedStyle(element).transitionDuration);
  expect(Number.parseFloat(duration)).toBeLessThanOrEqual(0.00001);
  await expectNoDocumentOverflow(page);
});
