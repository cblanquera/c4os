import { expect, test } from "@playwright/test";

test.describe.configure({ timeout: 60_000 });

test("runtime QA keeps capability conflicts explicit and responsive", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  await page.goto("/#/qa/runtime");
  await expect(
    page.getByRole("heading", {
      name: "Providers, models, and recoverable Chats",
    }),
  ).toBeVisible();
  await expect(
    page.getByLabel("Rust runtime authority").getByText("rust-core"),
  ).toBeVisible();
  await expect(
    page.getByLabel("Rust runtime authority").getByText("26"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Models & preflight" }).click();
  await expect(page.getByText("Needs Vision")).toBeVisible();
  await page.getByRole("button", { name: "Use compatible model" }).click();
  await expect(page.getByRole("status")).toContainText(
    "attachment ready for submission",
  );
  await page.screenshot({
    path: "tests/results/playwright/task-00004-provider-capability-preflight.png",
    fullPage: true,
  });

  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
  await page.keyboard.press("Tab");
  await expect(page.locator(":focus-visible")).toHaveCount(1);
  expect(consoleErrors).toEqual([]);
});

test("runtime QA preserves immutable turns through recovery and cancel", async ({
  page,
}) => {
  await page.goto("/#/qa/runtime");
  await page.getByRole("button", { name: "Session recovery" }).click();

  await expect(
    page.getByRole("button", { name: "Retry immutable turn" }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "Reload current state" }).click();
  await page.getByRole("button", { name: "Review effect" }).click();
  await page.getByRole("button", { name: "Retry immutable turn" }).click();
  await expect(page.getByText("Run Attempt 2", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Cancel active attempt" }).click();

  await expect(page.getByText("Cancelled", { exact: true })).toBeVisible();
  await expect(page.getByText("Interrupted", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Immutable prompt snapshot" }),
  ).toBeVisible();
  await page.screenshot({
    path: "tests/results/playwright/task-00004-session-recovery.png",
    fullPage: true,
  });
});
