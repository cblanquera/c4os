/// <reference types="node" />

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const policyCss = readFileSync(
  resolve(
    "src/frontend/features/settings/policies/advanced-policy-settings.css",
  ),
  "utf8",
);

describe("advanced policy settings style contract", () => {
  it("stacks the guardrail before the expanded Settings sidebar clips it", () => {
    expect(policyCss).toMatch(
      /@media\s*\(max-width:\s*900px\)\s*\{[\s\S]*?\.advanced-policy-guardrail\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+auto;/u,
    );
  });

  it("uses a horizontally scrollable policy-group rail at narrow widths", () => {
    expect(policyCss).toMatch(
      /@media\s*\(max-width:\s*760px\)[\s\S]*?\.advanced-policy-groups\s*\{[^}]*display:\s*flex;[^}]*overflow-x:\s*auto;/u,
    );
    expect(policyCss).toMatch(
      /@media\s*\(max-width:\s*760px\)[\s\S]*?\.advanced-policy-groups button\s*\{[^}]*flex:\s*0 0 min\(180px,\s*72vw\);/u,
    );
  });
});
