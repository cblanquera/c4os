import { afterEach, describe, expect, it, vi } from "vitest";

import { shouldSubscribeToNativeShellEvents } from "./build-native-transport";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("QA native shell event boundary", () => {
  it("subscribes only inside a real Tauri QA shell", () => {
    vi.stubGlobal("__TAURI_INTERNALS__", { invoke: vi.fn() });

    expect(shouldSubscribeToNativeShellEvents()).toBe(true);
  });

  it("keeps the explicit Playwright transport isolated from native events", () => {
    vi.stubGlobal("__C4OS_QA_NATIVE_FIXTURE__", "deterministic-e2e");
    vi.stubGlobal("__TAURI_INTERNALS__", { invoke: vi.fn() });

    expect(shouldSubscribeToNativeShellEvents()).toBe(false);
  });

  it("does not subscribe in a standalone browser QA build", () => {
    expect(shouldSubscribeToNativeShellEvents()).toBe(false);
  });
});
