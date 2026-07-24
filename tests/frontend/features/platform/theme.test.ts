import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  APPEARANCE_CHANGE_EVENT,
  bootstrapPlatformTheme,
  readRootAppearance,
  type NativeAppearanceSnapshot,
} from "../../../../src/frontend/features/platform/theme";

function createDarkSchemeQuery(initialMatches: boolean) {
  let listener: ((event: MediaQueryListEvent) => void) | undefined;
  const mediaQuery = {
    matches: initialMatches,
    media: "(prefers-color-scheme: dark)",
    onchange: null,
    addEventListener: vi.fn(
      (_type: string, nextListener: (event: MediaQueryListEvent) => void) => {
        listener = nextListener;
      },
    ),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  } as unknown as MediaQueryList;

  return {
    mediaQuery,
    change(matches: boolean) {
      listener?.({ matches } as MediaQueryListEvent);
    },
  };
}

describe("bootstrapPlatformTheme", () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="root"></div>';
  });

  it("guards reveal until a valid native snapshot initializes root state", async () => {
    let resolveSnapshot!: (snapshot: NativeAppearanceSnapshot) => void;
    const snapshot = new Promise<NativeAppearanceSnapshot>((resolve) => {
      resolveSnapshot = resolve;
    });
    const query = createDarkSchemeQuery(false);
    const appearanceEvents = vi.fn();
    document.addEventListener(APPEARANCE_CHANGE_EVENT, appearanceEvents);

    const bootstrap = bootstrapPlatformTheme({
      readNativeSnapshot: () => snapshot,
      matchMedia: () => query.mediaQuery,
    });

    expect(document.documentElement.dataset.appearanceReady).toBe("false");
    expect(document.getElementById("root")?.style.visibility).toBe("hidden");

    resolveSnapshot({
      platform: "macos",
      initialTheme: { scheme: "dark", source: "macosAppearance" },
    });
    const controller = await bootstrap;

    expect(controller.initialAppearance).toEqual({
      platform: "macos",
      colorScheme: "dark",
      source: "macosAppearance",
    });
    expect(document.documentElement.dataset.platform).toBe("macos");
    expect(document.documentElement.dataset.colorScheme).toBe("dark");
    expect(document.documentElement.style.colorScheme).toBe("dark");
    expect(document.documentElement.dataset.appearanceReady).toBe("true");
    expect(document.getElementById("root")?.style.visibility).toBe("");
    expect(query.mediaQuery.addEventListener).toHaveBeenCalledWith(
      "change",
      expect.any(Function),
    );
    expect(appearanceEvents).toHaveBeenCalledTimes(1);

    document.removeEventListener(APPEARANCE_CHANGE_EVENT, appearanceEvents);
    controller.dispose();
  });

  it("observes webview scheme changes independently and never persists an override", async () => {
    const query = createDarkSchemeQuery(false);
    const storageWrite = vi.spyOn(Storage.prototype, "setItem");
    const controller = await bootstrapPlatformTheme({
      readNativeSnapshot: () =>
        Promise.resolve({
          platform: "macos",
          initialTheme: { scheme: "light", source: "macosAppearance" },
        }),
      matchMedia: () => query.mediaQuery,
    });

    query.change(true);

    expect(readRootAppearance()).toEqual({
      platform: "macos",
      colorScheme: "dark",
      source: "webviewPrefersColorScheme",
    });
    expect(storageWrite).not.toHaveBeenCalled();

    controller.dispose();
    expect(query.mediaQuery.removeEventListener).toHaveBeenCalledWith(
      "change",
      expect.any(Function),
    );
    storageWrite.mockRestore();
  });

  it("falls back to the current webview preference when native input fails validation", async () => {
    const query = createDarkSchemeQuery(true);

    const controller = await bootstrapPlatformTheme({
      readNativeSnapshot: () =>
        Promise.resolve({
          platform: "macos",
          initialTheme: { scheme: "sepia", source: "macosAppearance" },
        }),
      matchMedia: () => query.mediaQuery,
    });

    expect(controller.initialAppearance).toEqual({
      platform: "macos",
      colorScheme: "dark",
      source: "webviewPreferredColorScheme",
    });
    controller.dispose();
  });

  it("cannot strand the renderer when the native snapshot never settles", async () => {
    vi.useFakeTimers();
    const query = createDarkSchemeQuery(true);
    const controllerPromise = bootstrapPlatformTheme({
      readNativeSnapshot: () => new Promise(() => undefined),
      nativeSnapshotTimeoutMs: 25,
      matchMedia: () => query.mediaQuery,
    });

    expect(document.getElementById("root")?.style.visibility).toBe("hidden");
    await vi.advanceTimersByTimeAsync(25);
    const controller = await controllerPromise;

    expect(controller.initialAppearance.source).toBe(
      "webviewPreferredColorScheme",
    );
    expect(document.documentElement.dataset.appearanceReady).toBe("true");
    expect(document.getElementById("root")?.style.visibility).toBe("");
    controller.dispose();
    vi.useRealTimers();
  });
});
