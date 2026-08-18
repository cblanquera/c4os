/// <reference types="node" />

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const shellCss = readFileSync(
  resolve("src/frontend/features/shell/ui/shell-view.css"),
  "utf8",
);
const platformCss = readFileSync(
  resolve("src/frontend/features/platform/platform-theme.css"),
  "utf8",
);
const geometrySource = readFileSync(
  resolve("src/frontend/features/shell/ui/shell-geometry.ts"),
  "utf8",
);

describe("shell view style contract", () => {
  it("uses shared r013 geometry, a contained composer dock, and no right panel", () => {
    expect(platformCss).toMatch(/--shell-header-height:\s*58px;/u);
    expect(platformCss).toMatch(/--shell-composer-maximum:\s*760px;/u);
    expect(geometrySource).toMatch(/projectPanelInitialWidth:\s*228/u);
    expect(geometrySource).toMatch(/settingsNavigationWidth:\s*224/u);
    expect(geometrySource).toMatch(/settingsContentMaximumWidth:\s*920/u);
    expect(shellCss).toMatch(
      /\.shell-title-header\s*\{[^}]*block-size:\s*var\(--shell-header-height\);/su,
    );
    expect(shellCss).toMatch(
      /\.shell-composer-dock\s*\{[^}]*border-block-start:\s*1px solid var\(--border-separator\);/su,
    );
    expect(shellCss).toMatch(
      /\.shell-composer-dock\s*>\s*\*\s*\{[^}]*var\(--shell-composer-maximum\)/su,
    );
    expect(shellCss).not.toMatch(/right-panel/iu);
  });

  it("contains the accepted overlay and compressed Settings breakpoints", () => {
    expect(shellCss).toMatch(/@media\s*\(max-width:\s*992px\)/u);
    expect(shellCss).toMatch(/@media\s*\(max-width:\s*680px\)/u);
    expect(shellCss).toMatch(
      /\.shell-settings__nav-label,\s*\.shell-settings__back-label,\s*\.shell-settings__navigation-label\s*\{[^}]*clip-path:\s*inset\(50%\)/su,
    );
  });

  it("allocates a retained contextual Chat row with a visible resize boundary", () => {
    expect(shellCss).toMatch(
      /\.shell-project-panel\[data-contextual-chat="true"\]\s*\{[^}]*var\(--shell-contextual-chat-height,\s*40vh\)/su,
    );
    expect(shellCss).toMatch(
      /\.shell-project-panel__context-resizer\s*\{[^}]*cursor:\s*row-resize;/su,
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
