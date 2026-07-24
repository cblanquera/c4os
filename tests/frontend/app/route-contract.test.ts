import { describe, expect, it } from "vitest";

import {
  APP_ROUTE_DEFINITIONS,
  SETTINGS_NAVIGATION,
  findAppRoute,
  isSettingsRoute,
} from "../../../src/frontend/app/route-contract";

describe("application route contract", () => {
  it("keeps every accepted top-level surface directly addressable", () => {
    expect(APP_ROUTE_DEFINITIONS).toHaveLength(16);
    expect(new Set(APP_ROUTE_DEFINITIONS.map((route) => route.id)).size).toBe(
      16,
    );
    expect(new Set(APP_ROUTE_DEFINITIONS.map((route) => route.path)).size).toBe(
      16,
    );
    expect(APP_ROUTE_DEFINITIONS.map((route) => route.path)).toEqual([
      "/onboarding",
      "/start",
      "/chat",
      "/chat-search",
      "/chat-capabilities",
      "/files",
      "/browser",
      "/terminal",
      "/settings/providers",
      "/settings/models",
      "/settings/runtimes",
      "/settings/configuration",
      "/settings/plugins",
      "/settings/skills",
      "/settings/mcp",
      "/settings/advanced-policies",
    ]);
  });

  it("retains the accepted Settings order and Advanced Policies selection", () => {
    expect(SETTINGS_NAVIGATION.map((route) => route.title)).toEqual([
      "Providers",
      "Models",
      "Runtimes",
      "Configuration",
      "Plugins",
      "Skills",
      "MCP Servers",
    ]);
    expect(isSettingsRoute("/settings/providers")).toBe(true);
    expect(isSettingsRoute("/settings/advanced-policies")).toBe(true);
    expect(findAppRoute("/settings/advanced-policies")?.kind).toBe(
      "advancedPolicies",
    );
    expect(isSettingsRoute("/chat")).toBe(false);
  });

  it("fails closed for unaccepted routes", () => {
    expect(findAppRoute("/detached-chat")).toBeNull();
    expect(findAppRoute("/browser/tabs")).toBeNull();
    expect(findAppRoute("/terminal/fullscreen")).toBeNull();
  });
});
