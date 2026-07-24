import { expect, test } from "@playwright/test";

test("direct QA foundation route is visibly deterministic and fixture-only", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  await page.goto("/#/qa/foundation");

  await expect(
    page.getByRole("heading", { name: "C4OS QA Foundation" }),
  ).toBeVisible();
  await expect(page.getByLabel("Fixture mode")).toBeVisible();
  await expect(
    page.getByText("Deterministic QA data only — not production state."),
  ).toBeVisible();
  await expect(page.getByText("workspace-qa-0001")).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Accepted deterministic routes" }),
  ).toHaveCount(1);
  await expect(
    page
      .getByRole("navigation", { name: "Accepted deterministic routes" })
      .getByRole("link"),
  ).toHaveCount(16);
  await expect(page.getByText("spec-00003-integrated-r013")).toBeVisible();

  await page.getByRole("button", { name: "Reset fixture" }).click();
  await expect(
    page.getByText("Fixture reset to 2026-07-18T00:00:00.000Z"),
  ).toBeVisible();

  await page.waitForLoadState("networkidle");

  expect(consoleErrors).toEqual([]);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
});

test("QA direct route launcher enters a production-composed destination", async ({
  page,
}) => {
  await page.goto("/#/qa/foundation");

  await page
    .getByRole("navigation", { name: "Accepted deterministic routes" })
    .getByRole("link", { name: "/settings/models", exact: true })
    .click();

  await expect(page).toHaveURL(/#\/settings\/models$/);
  await expect(page.getByRole("heading", { name: "Models" })).toBeVisible();
  await expect(page.getByLabel("QA fixture identity")).toContainText(
    "not production state",
  );
  await page.goto("/#/qa/foundation");
  await expect(page).toHaveURL(/#\/qa\/foundation$/);
  await expect(
    page.getByRole("heading", { name: "Direct route launcher" }),
  ).toBeVisible();
});

test("retired foundation route enters production Workspace Start", async ({
  page,
}) => {
  await page.goto("/#/foundation");

  await expect(
    page.getByRole("heading", { name: "What would you like to open?" }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#\/start$/);
});

test("QA Workspace Start uses the production controller with a build-gated adapter", async ({
  page,
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "What would you like to open?" }),
  ).toBeVisible();
  await expect(
    page.locator('[data-qa-product-adapter="deterministic"]'),
  ).toHaveAttribute("data-qa-route", "/start");
  await expect(page.getByRole("listitem")).toHaveCount(3);
  await page.getByRole("button", { name: /Legacy UI/ }).click();
  await expect(page.getByRole("status")).toContainText("Recovered Legacy UI");
  await page.getByRole("button", { name: "Continue to Chat" }).click();
  await expect(page).toHaveURL(/#\/chat$/);
  await expect(page.getByRole("region", { name: "Chat" })).toBeVisible();
});

test("workspace QA route preserves the three-row start contract", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  await page.goto("/#/qa/workspace");

  await expect(
    page.getByRole("heading", { name: "What would you like to open?" }),
  ).toBeVisible();
  await expect(page.getByRole("group", { name: "Open options" })).toContainText(
    "Open a folder",
  );
  await expect(page.getByRole("listitem")).toHaveCount(3);
  await page.screenshot({
    path: "output/playwright/task-00002-qa-workspace-start.png",
    fullPage: true,
  });

  await page.getByRole("button", { name: /Legacy UI/ }).click();
  const recoveryStatus = page.getByRole("status");
  await expect(recoveryStatus).toContainText("Recovered Legacy UI");
  await expect(recoveryStatus).toHaveAttribute("aria-atomic", "true");
  await expect(recoveryStatus).toBeFocused();
  await expect(
    page.getByRole("button", { name: "Continue to Chat" }),
  ).toBeEnabled();
  for (const primaryAction of await page
    .getByRole("group", { name: "Open options" })
    .getByRole("button")
    .all()) {
    await expect(primaryAction).toBeDisabled();
  }
  for (const recentAction of await page
    .getByRole("listitem")
    .getByRole("button")
    .all()) {
    await expect(recentAction).toBeDisabled();
  }
  await page.screenshot({
    path: "output/playwright/task-00002-qa-workspace-recovery.png",
    fullPage: true,
  });

  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
  await page.getByRole("button", { name: "Continue to Chat" }).click();
  await expect(recoveryStatus).toContainText(
    "Opened Legacy UI. Entering Chat…",
  );
  await expect(
    page.getByRole("button", { name: /Open a folder/ }),
  ).toBeEnabled();

  await page.waitForLoadState("networkidle");
  expect(consoleErrors).toEqual([]);
});
