import { describe, expect, it } from "vitest";

import { createAppStore } from "../../app/store";
import type { PlatformSnapshot } from "../../platform/platform-service";
import type { RuntimeCoreSnapshot } from "../../platform/runtime-core";
import type {
  StateGeneration,
  WorkspaceId,
  WorkspaceStartSnapshot,
} from "../../platform/protocol";
import { UNINITIALIZED_GENERATION } from "./state";
import {
  ingestNativeShellProjections,
  type NativeShellReaders,
} from "./native-bootstrap";

const generation = 7 as StateGeneration;

function readers(
  overrides: Partial<NativeShellReaders> = {},
): NativeShellReaders {
  return {
    readPlatform: () => Promise.resolve(platformSnapshot()),
    readRuntime: () => Promise.resolve(runtimeSnapshot()),
    readWorkspaceStart: () => Promise.resolve(workspaceStartSnapshot()),
    readReducedMotion: () => true,
    ...overrides,
  };
}

describe("native shell projection ingestion", () => {
  it("publishes validated platform, launch, workspace, runtime, and approval domains", async () => {
    const store = createAppStore(undefined);
    const result = await ingestNativeShellProjections(
      store.dispatch,
      readers(),
    );

    expect(result).toEqual({
      publishedDomains: [
        "platform",
        "runtime",
        "approvals",
        "workspace",
        "launch",
      ],
      unavailableSources: [],
    });
    expect(store.getState().shellAuthority.platform).toMatchObject({
      generation: 0,
      value: {
        appearance: "dark",
        appearanceSource: "macosAppearance",
        reducedMotion: true,
      },
    });
    expect(store.getState().shellAuthority.launch).toMatchObject({
      generation,
      value: {
        destination: "workspace-start",
        providerConfigured: true,
        onboardingReady: true,
        recentWorkspaceIds: ["workspace:native"],
      },
    });
    expect(store.getState().shellAuthority.runtime.value.runtimes).toEqual([
      expect.objectContaining({ id: "runtime:native", lifecycle: "ready" }),
    ]);
    expect(store.getState().shellAuthority.approvals.value.approvals).toEqual([
      expect.objectContaining({ id: "approval:native", state: "pending" }),
    ]);
  });

  it("leaves unsupported native sources fail-closed without rejecting available domains", async () => {
    const store = createAppStore(undefined);
    const result = await ingestNativeShellProjections(
      store.dispatch,
      readers({
        readRuntime: () => Promise.reject(new Error("runtime unavailable")),
      }),
    );

    expect(result.publishedDomains).toEqual(["platform", "workspace"]);
    expect(result.unavailableSources).toEqual(["runtime"]);
    expect(store.getState().shellAuthority.runtime.generation).toBe(
      UNINITIALIZED_GENERATION,
    );
    expect(store.getState().shellAuthority.launch.generation).toBe(
      UNINITIALIZED_GENERATION,
    );
  });
});

function platformSnapshot(): PlatformSnapshot {
  return {
    contractVersion: 1,
    platform: "macos",
    architecture: "aarch64",
    initialTheme: { scheme: "dark", source: "macosAppearance" },
    liveThemeSource: "webviewPrefersColorScheme",
    window: {
      decorations: "standard",
      titlebarTransparent: false,
      titlebarOverlay: false,
      initiallyVisible: false,
      revealFallbackTimeoutMs: 4_000,
    },
    vocabulary: {
      revealAction: "Reveal in Finder",
      primaryModifierSymbol: "⌘",
      alternateModifierSymbol: "⌥",
      shiftModifierSymbol: "⇧",
    },
    capabilities: {
      nativeApplicationMenu: true,
      nativeSettingsShortcut: true,
      nativeFilePicker: true,
      nativeFolderPicker: true,
      nativeWorkspacePicker: true,
      standardWindowDecorations: true,
    },
    settingsMenu: {
      menuItemId: "c4os.menu.settings",
      commandId: "c4os.command.openSettings",
      route: "/settings/providers",
      accelerator: "CmdOrCtrl+,",
      keyboardLabel: "⌘,",
    },
  };
}

function runtimeSnapshot(): RuntimeCoreSnapshot {
  return {
    authority: "rust-core",
    generation,
    providerGeneration: 4,
    capabilityGeneration: 5,
    runtimeGeneration: 6,
    onboardingReady: true,
    providers: [
      {
        providerId: "provider:native",
        displayName: "Native Provider",
        enabled: true,
        testStatus: {},
        modelCount: 1,
        selectedModelId: "model:native",
      },
    ],
    runtimes: [
      {
        runtimeId: "runtime:native",
        runtimeKind: "open-code",
        nativeVersion: "1.18.3",
        lifecycle: "ready",
        health: "healthy",
        processGeneration: 2,
      },
    ],
    pendingApprovals: [
      {
        runtimeId: "runtime:native" as never,
        correlationId: "correlation:native" as never,
        promptId: "approval:native" as never,
      },
    ],
  };
}

function workspaceStartSnapshot(): WorkspaceStartSnapshot {
  return {
    protocolVersion: 1,
    generation,
    authority: "rust-core",
    recents: [
      {
        workspaceId: "workspace:native" as WorkspaceId,
        displayName: "Native Workspace",
        lastOpenedAt: 1_721_312_000,
        isMissing: false,
      },
    ],
  };
}
