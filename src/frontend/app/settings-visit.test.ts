import { describe, expect, it } from "vitest";

import {
  createSettingsVisit,
  readActiveShellFocusTarget,
  resolveSettingsReturnRoute,
  settingsSectionForRoute,
} from "./settings-visit";
import { createAppStore } from "./store";

describe("Settings visit integration", () => {
  it("keeps accepted workspace routes exact and fails closed elsewhere", () => {
    expect(resolveSettingsReturnRoute("/chat-search")).toBe("/chat-search");
    expect(resolveSettingsReturnRoute("/")).toBe("/start");
    expect(resolveSettingsReturnRoute("//external.invalid")).toBe("/start");
    expect(resolveSettingsReturnRoute("/settings/models")).toBe("/start");
    expect(settingsSectionForRoute("/settings/advanced-policies")).toBe(
      "advanced-policies",
    );
  });

  it("captures only shell-owned focus identifiers", () => {
    const composer = document.createElement("textarea");
    composer.dataset.shellFocusTarget = "composer-draft";
    document.body.append(composer);
    composer.focus();
    expect(readActiveShellFocusTarget()).toBe("composer-draft");

    composer.dataset.shellFocusTarget = "arbitrary-node";
    expect(readActiveShellFocusTarget()).toBeNull();
  });

  it("builds a complete return record from the authoritative store", () => {
    const store = createAppStore(undefined);
    expect(createSettingsVisit(store.getState(), "/chat", null)).toEqual({
      section: "providers",
      route: "/chat",
      workspaceId: null,
      sessionId: null,
      focusTarget: null,
    });
  });
});
