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

test("retired foundation route enters production Workspace Start", async ({
  page,
}) => {
  await page.goto("/#/foundation");

  await expect(
    page.getByRole("heading", { name: "Workspace Start" }),
  ).toBeVisible();
  await expect(page).toHaveURL(/#\/$/);
});

test("production Workspace Start fails closed without native authority", async ({
  page,
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "Workspace Start" }),
  ).toBeVisible();
  await expect(page.getByRole("status")).toContainText(
    "Workspace Start is unavailable",
  );
  await expect(page.getByRole("button")).toHaveCount(0);
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
  await expect(page.getByRole("status")).toContainText("Recovered Legacy UI");
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

  await page.waitForLoadState("networkidle");
  expect(consoleErrors).toEqual([]);
});
