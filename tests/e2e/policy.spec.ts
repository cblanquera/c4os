import { expect, test } from "@playwright/test";

test.describe.configure({ timeout: 60_000 });

test("Advanced Policies preserves the seven-group and concrete-exception contract", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  await page.goto("/#/qa/policy");

  await expect(
    page.getByRole("heading", { name: "Advanced Policies" }),
  ).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Policy groups" }).getByRole("button"),
  ).toHaveCount(7);
  await page
    .getByRole("combobox", { name: "workspace.modify policy" })
    .selectOption("ask");
  await expect(
    page.getByRole("button", { name: "Save Policies" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Save Policies" }).click();
  await expect(page.getByRole("status")).toContainText("Policies saved");

  await page.getByRole("tab", { name: /Exceptions 2/i }).click();
  await expect(
    page.getByRole("heading", { name: "Saved exceptions" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Revoke" }).first().click();
  await expect(page.getByRole("tab", { name: /Exceptions 1/i })).toBeVisible();

  await page.screenshot({
    path: "output/playwright/task-00003-advanced-policies.png",
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

test("approval activity exposes lifecycle and denial-before-effect", async ({
  page,
}) => {
  await page.goto("/#/qa/policy");
  await page.getByRole("button", { name: "Approval activity" }).click();

  await expect(
    page.getByRole("heading", { name: "Approval activity" }),
  ).toBeVisible();
  for (const state of ["Pending", "Queued", "Expired", "Denied", "Completed"]) {
    await expect(page.getByText(state, { exact: true })).toBeVisible();
  }
  await expect(page.getByText(/1 pending · 1 queued/i)).toBeVisible();
  await page.getByRole("button", { name: "Deny" }).click();
  await expect(page.getByRole("status")).toContainText(
    "No side effect was released",
  );
  await expect(page.getByText(/0 pending · 1 queued/i)).toBeVisible();
  await page.screenshot({
    path: "output/playwright/task-00003-approval-lifecycle.png",
    fullPage: true,
  });
});
