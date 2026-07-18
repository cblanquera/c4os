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

test("browser renderer fails closed without native authority", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "C4OS foundation is running" }),
  ).toBeVisible();
  await expect(page.getByText("Unavailable · fail closed")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Start a Workspace" }),
  ).toBeDisabled();

  await page.waitForLoadState("networkidle");

  expect(consoleErrors).toEqual([]);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(
    await page.evaluate(() => document.documentElement.clientWidth),
  );
});
