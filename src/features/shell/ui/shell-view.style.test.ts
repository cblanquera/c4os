/// <reference types="node" />

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const shellCss = readFileSync(
  resolve("src/features/shell/ui/shell-view.css"),
  "utf8",
);

describe("shell view style contract", () => {
  it("uses a 58px title header, fixed composer, and no right-panel selector", () => {
    expect(shellCss).toMatch(
      /\.shell-title-header\s*\{[^}]*block-size:\s*58px;/su,
    );
    expect(shellCss).toMatch(
      /\.shell-composer\s*\{[^}]*position:\s*sticky;[^}]*inset-block-end:\s*0;/su,
    );
    expect(shellCss).not.toMatch(/right-panel/iu);
  });

  it("contains the accepted overlay and compressed Settings breakpoints", () => {
    expect(shellCss).toMatch(/@media\s*\(max-width:\s*992px\)/u);
    expect(shellCss).toMatch(/@media\s*\(max-width:\s*680px\)/u);
    expect(shellCss).toMatch(
      /\.shell-settings__nav-label,\s*\.shell-settings__back-label\s*\{[^}]*clip-path:\s*inset\(50%\)/su,
    );
  });

  it("uses semantic colors without embedding product color literals", () => {
    expect(shellCss).not.toMatch(/#[0-9a-f]{3,8}\b|rgba?\(/iu);
    expect(shellCss).not.toMatch(
      /(?:color|background|border-color):\s*(?:white|black|gray|grey|red|blue)\b/iu,
    );
    expect(shellCss).toContain("var(--surface-window)");
    expect(shellCss).toContain("var(--text-primary)");
    expect(shellCss).toContain("var(--border-separator)");
    expect(shellCss).toContain("var(--focus-ring)");
  });
});
