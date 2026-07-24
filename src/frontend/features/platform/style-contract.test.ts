/// <reference types="node" />

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const themeCss = readFileSync(
  resolve("src/frontend/features/platform/platform-theme.css"),
  "utf8",
);
const runtimeCss = readFileSync(
  resolve("src/frontend/features/runtime/runtime-qa.css"),
  "utf8",
);
const productCss = readFileSync(resolve("src/frontend/styles.css"), "utf8");

const literalColor = /#[0-9a-f]{3,8}\b|rgba?\(/iu;
const semanticTokens = [
  "surface-window",
  "surface-sidebar",
  "surface-raised",
  "surface-sunken",
  "surface-overlay",
  "text-primary",
  "text-secondary",
  "text-tertiary",
  "text-disabled",
  "text-link",
  "border-separator",
  "border-control",
  "border-strong",
  "accent",
  "accent-hover",
  "accent-pressed",
  "accent-text",
  "focus-ring",
  "selection",
  "danger",
  "warning",
  "success",
  "shadow-window",
  "shadow-menu",
  "shadow-dialog",
] as const;

function themeBlock(selector: "light" | "dark"): string {
  const expression =
    selector === "light"
      ? /:root,\s*:root\[data-color-scheme="light"\]\s*\{([^}]+)\}/u
      : /:root\[data-color-scheme="dark"\]\s*\{([^}]+)\}/u;
  const block = expression.exec(themeCss)?.[1];
  if (!block) throw new Error(`missing ${selector} semantic theme block`);
  return block;
}

function hexToken(block: string, token: string): string {
  const value = new RegExp(`--${token}:\\s*(#[0-9a-f]{6})`, "iu").exec(
    block,
  )?.[1];
  if (!value) throw new Error(`missing hexadecimal --${token} token`);
  return value;
}

function channel(value: number): number {
  const normalized = value / 255;
  return normalized <= 0.04045
    ? normalized / 12.92
    : ((normalized + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const value = Number.parseInt(hex.slice(1), 16);
  return (
    0.2126 * channel((value >> 16) & 255) +
    0.7152 * channel((value >> 8) & 255) +
    0.0722 * channel(value & 255)
  );
}

function contrast(first: string, second: string): number {
  const lighter = Math.max(luminance(first), luminance(second));
  const darker = Math.min(luminance(first), luminance(second));
  return (lighter + 0.05) / (darker + 0.05);
}

describe("semantic style contract", () => {
  it("defines every accepted semantic token exactly once per scheme", () => {
    const light = themeBlock("light");
    const dark = themeBlock("dark");

    for (const token of semanticTokens) {
      expect(light.match(new RegExp(`--${token}:`, "gu"))).toHaveLength(1);
      expect(dark.match(new RegExp(`--${token}:`, "gu"))).toHaveLength(1);
    }
  });

  it("keeps literal colors inside the one semantic token source", () => {
    for (const [label, css] of [
      ["src/frontend/styles.css", productCss],
      ["src/frontend/features/runtime/runtime-qa.css", runtimeCss],
    ] as const) {
      expect(
        css.match(literalColor),
        `${label} contains a product color literal`,
      ).toBeNull();
    }

    const literalLines = themeCss
      .split("\n")
      .filter((line) => literalColor.test(line));
    expect(literalLines.length).toBeGreaterThan(0);
    for (const line of literalLines) {
      expect(line.trimStart()).toMatch(/^--[a-z-]+:/u);
    }
    expect(themeCss).not.toMatch(
      /--(?:surface-canvas|surface-card|border-subtle|focus):/u,
    );
  });

  it("meets normal-text contrast for representative semantic pairs", () => {
    const pairs = [
      ["text-primary", "surface-window"],
      ["text-secondary", "surface-window"],
      ["text-tertiary", "surface-window"],
      ["text-link", "surface-window"],
      ["accent-text", "accent"],
      ["danger", "surface-raised"],
      ["warning", "surface-raised"],
      ["success", "surface-raised"],
    ] as const;

    for (const scheme of ["light", "dark"] as const) {
      const block = themeBlock(scheme);
      for (const [foreground, background] of pairs) {
        expect(
          contrast(hexToken(block, foreground), hexToken(block, background)),
          `${scheme} ${foreground} on ${background}`,
        ).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  it("keeps the native window hidden until reveal and permits narrow review", () => {
    const config = JSON.parse(
      readFileSync(resolve("src/backend/tauri.conf.json"), "utf8"),
    ) as {
      app: {
        windows: Array<{
          visible?: boolean;
          minWidth?: number;
          decorations?: boolean;
          transparent?: boolean;
        }>;
      };
    };
    const window = config.app.windows[0];
    expect(window).toMatchObject({
      visible: false,
      minWidth: 560,
      decorations: true,
      transparent: false,
    });
  });
});
