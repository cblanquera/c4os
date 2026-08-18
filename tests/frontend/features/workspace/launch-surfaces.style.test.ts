/// <reference types="node" />

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const providerCss = readFileSync(
  resolve("src/frontend/features/settings/providers/provider-settings.css"),
  "utf8",
);
const workspaceCss = readFileSync(resolve("src/frontend/styles.css"), "utf8");

describe("r013 launch surface style contract", () => {
  it("keeps onboarding compact, one-column, and one-form", () => {
    expect(providerCss).toMatch(
      /\.provider-onboarding\s*\{[^}]*width:\s*min\(480px,\s*100%\);/su,
    );
    expect(providerCss).toMatch(
      /\.provider-form--onboarding\s+\.provider-form__grid\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\);/su,
    );
    expect(providerCss).toMatch(
      /\.provider-form__actions\s*\{[^}]*grid-template-columns:\s*1fr 1\.1fr;/su,
    );
    expect(providerCss).not.toMatch(
      /\.provider-connection-status\s*\{[^}]*(?:background|border):/su,
    );
  });

  it("keeps Workspace Start bounded and stacks icon-plus-copy without excess height", () => {
    expect(workspaceCss).toMatch(
      /\.workspace-start__card\s*\{[^}]*width:\s*min\(100%,\s*940px\);/su,
    );
    expect(workspaceCss).toMatch(
      /\.workspace-start__content\s*\{[^}]*width:\s*min\(100%,\s*820px\);/su,
    );
    expect(workspaceCss).toMatch(/@media\s*\(max-width:\s*760px\)/u);
    const workspaceNarrow = workspaceCss.slice(
      workspaceCss.lastIndexOf("@media (max-width: 760px)"),
    );
    const narrowAction = workspaceNarrow.match(
      /\.workspace-start__action\s*\{(?<rule>[^}]*)\}/u,
    )?.groups?.rule;
    expect(narrowAction).toContain("min-height: 0;");
    expect(narrowAction).toContain(
      "grid-template-columns: auto minmax(0, 1fr);",
    );
    expect(narrowAction).toContain("grid-template-rows: auto auto;");
    expect(narrowAction).toContain("row-gap: 5px;");
  });
});
