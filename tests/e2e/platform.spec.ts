import { expect, test, type Page } from "@playwright/test";

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

test.describe("native semantic platform foundation", () => {
  test("follows live system appearance without losing interaction state", async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await page.goto("/#/qa/platform");
    await expect(page.locator("html")).toHaveAttribute(
      "data-appearance-ready",
      "true",
    );
    await expect(page.locator("html")).toHaveAttribute(
      "data-platform",
      "macos",
    );
    await expect(page.locator("html")).toHaveAttribute(
      "data-color-scheme",
      "light",
    );

    await page.getByRole("button", { name: "Dirty" }).click();
    await expect(page.getByRole("status")).toContainText(
      "Last reviewed state: Dirty",
    );
    await page.emulateMedia({ colorScheme: "dark" });
    await expect(page.locator("html")).toHaveAttribute(
      "data-color-scheme",
      "dark",
    );
    await expect(page.locator("html")).toHaveAttribute(
      "data-appearance-source",
      "webviewPrefersColorScheme",
    );
    await expect(page.getByRole("status")).toContainText(
      "Last reviewed state: Dirty",
    );

    const tokens = await page.evaluate(() => {
      const styles = getComputedStyle(document.documentElement);
      return [
        "--surface-window",
        "--surface-raised",
        "--text-primary",
        "--focus-ring",
        "--danger",
      ].map((token) => styles.getPropertyValue(token).trim());
    });
    expect(tokens.every(Boolean)).toBe(true);
    await expectNoDocumentOverflow(page);
  });

  test("keeps modal focus, dismissal, and reduced motion intact across theme changes", async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: "light", reducedMotion: "reduce" });
    await page.goto("/#/qa/platform");
    const trigger = page.getByRole("button", { name: "Review dialog" });
    await trigger.focus();
    await trigger.click();
    await expect(
      page.getByRole("dialog", { name: "Keep unsaved appearance work?" }),
    ).toBeVisible();

    await page.emulateMedia({ colorScheme: "dark", reducedMotion: "reduce" });
    await expect(page.locator("html")).toHaveAttribute(
      "data-color-scheme",
      "dark",
    );
    const duration = await page
      .getByRole("button", { name: "Keep changes" })
      .evaluate((element) => getComputedStyle(element).transitionDuration);
    expect(Number.parseFloat(duration)).toBeLessThanOrEqual(0.00001);

    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(trigger).toBeFocused();
  });

  for (const width of [1180, 760, 680, 620, 560]) {
    test(`contains the semantic QA surface at ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width, height: 760 });
      await page.goto("/#/qa/platform");
      await expect(
        page.getByRole("heading", {
          name: "System appearance, one source at a time",
        }),
      ).toBeVisible();
      await expectNoDocumentOverflow(page);
    });
  }

  test("uses one production Settings route with compressed navigation and fail-closed picker feedback", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1100, height: 680 });
    await page.goto("/#/settings/providers");
    await expect(
      page.getByRole("heading", { name: "Providers" }),
    ).toBeVisible();
    await expect(
      page.getByRole("navigation", { name: "Settings" }),
    ).toContainText("MCP Servers");
    await expect(page.locator(".shell-settings__scroll")).toHaveCSS(
      "overflow-y",
      "auto",
    );
    await expectNoDocumentOverflow(page);

    await page.setViewportSize({ width: 620, height: 680 });
    await expect(page.locator(".shell-settings__body")).toHaveCSS(
      "grid-template-columns",
      "64px 556px",
    );
    await expect(
      page.getByRole("button", { name: "Back to C4OS" }),
    ).toBeVisible();
    await expectNoDocumentOverflow(page);

    await page.getByRole("button", { name: "Choose project folder…" }).click();
    await expect(page.getByRole("status")).toContainText(
      "Native folder access is unavailable",
    );
    expect(await page.locator("body").innerText()).not.toContain("/Users/");
  });
});
