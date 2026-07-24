import { describe, expect, it, vi } from "vitest";

import type { CorrelationId, RequestId } from "./protocol";
import {
  OPEN_SETTINGS_COMMAND_ID,
  PICKER_CONTRACT_VERSION,
  PLATFORM_CONTRACT_VERSION,
  SETTINGS_ROUTE,
  createPlatformAdapter,
  parseSettingsEvent,
  type PlatformTransport,
} from "./platform-service";

const requestId = "platform-request" as RequestId;
const correlationId = "platform-correlation" as CorrelationId;

function envelope(payload: unknown) {
  return {
    protocolVersion: 1,
    requestId,
    correlationId,
    generation: 0,
    payload,
  };
}

function platformSnapshot() {
  return {
    contractVersion: PLATFORM_CONTRACT_VERSION,
    platform: "macos",
    architecture: "aarch64",
    initialTheme: { scheme: "dark", source: "macosAppearance" },
    liveThemeSource: "webviewPrefersColorScheme",
    window: {
      decorations: "standard",
      titlebarTransparent: false,
      titlebarOverlay: false,
      initiallyVisible: false,
      revealFallbackTimeoutMs: 5_000,
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
      commandId: OPEN_SETTINGS_COMMAND_ID,
      route: SETTINGS_ROUTE,
      accelerator: "CmdOrCtrl+,",
      keyboardLabel: "⌘,",
    },
  };
}

function adapter(transport: PlatformTransport) {
  return createPlatformAdapter(transport, {
    requestIdFactory: () => requestId,
    correlationIdFactory: () => correlationId,
  });
}

describe("PlatformService renderer boundary", () => {
  it("accepts the exact target-qualified platform snapshot", async () => {
    const invoke = vi.fn(() => Promise.resolve(envelope(platformSnapshot())));
    const snapshot = await adapter({ invoke }).readSnapshot();

    expect(invoke).toHaveBeenCalledWith("platform_snapshot", {
      request: {
        protocolVersion: 1,
        requestId,
        correlationId,
        expectedGeneration: 0,
      },
    });
    expect(snapshot.initialTheme).toEqual({
      scheme: "dark",
      source: "macosAppearance",
    });
    expect(snapshot.settingsMenu.keyboardLabel).toBe("⌘,");
    expect(snapshot.vocabulary.revealAction).toBe("Reveal in Finder");
  });

  it("rejects extra target fields and mismatched response identities", async () => {
    await expect(
      adapter({
        invoke: () =>
          Promise.resolve(
            envelope({ ...platformSnapshot(), manualThemeOverride: "dark" }),
          ),
      }).readSnapshot(),
    ).rejects.toMatchObject({ code: "invalidPayload" });

    await expect(
      adapter({
        invoke: () =>
          Promise.resolve({
            ...envelope(platformSnapshot()),
            requestId: "another-request",
          }),
      }).readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });
  });

  it("returns only correlated opaque picker grants", async () => {
    const invoke: PlatformTransport["invoke"] = (command, args) => {
      expect(command).toBe("platform_pick");
      const picker = args.picker as { requestId: RequestId };
      return Promise.resolve(
        envelope({
          type: "selected",
          contractVersion: PICKER_CONTRACT_VERSION,
          requestId: picker.requestId,
          grants: [
            {
              grantId: "picker-grant-1",
              objectKind: "folder",
              displayName: "project",
            },
          ],
        }),
      );
    };

    await expect(
      adapter({ invoke }).pick("openProjectFolder"),
    ).resolves.toEqual({
      type: "selected",
      contractVersion: PICKER_CONTRACT_VERSION,
      requestId,
      grants: [
        {
          grantId: "picker-grant-1",
          objectKind: "folder",
          displayName: "project",
        },
      ],
    });
  });

  it("rejects any raw path added to a picker grant", async () => {
    await expect(
      adapter({
        invoke: () =>
          Promise.resolve(
            envelope({
              type: "selected",
              contractVersion: PICKER_CONTRACT_VERSION,
              requestId,
              grants: [
                {
                  grantId: "picker-grant-1",
                  objectKind: "file",
                  displayName: "private.txt",
                  path: "/Users/example/private.txt",
                },
              ],
            }),
          ),
      }).pick("openFile"),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("requests one opaque folder grant for Files Browse", async () => {
    const invoke = vi.fn(() =>
      Promise.resolve(
        envelope({
          type: "cancelled",
          contractVersion: PICKER_CONTRACT_VERSION,
          requestId,
        }),
      ),
    );

    await adapter({ invoke }).pick("openFolder");

    expect(invoke).toHaveBeenCalledWith(
      "platform_pick",
      expect.objectContaining({
        picker: expect.objectContaining({
          purpose: "openFolder",
          selection: {
            objectKind: "folder",
            allowsMultiple: false,
            allowedExtensions: [],
          },
        }),
      }),
    );
  });

  it("validates the typed native Settings event", () => {
    expect(
      parseSettingsEvent({
        contractVersion: PLATFORM_CONTRACT_VERSION,
        commandId: OPEN_SETTINGS_COMMAND_ID,
        route: SETTINGS_ROUTE,
      }),
    ).toEqual({
      contractVersion: PLATFORM_CONTRACT_VERSION,
      commandId: OPEN_SETTINGS_COMMAND_ID,
      route: SETTINGS_ROUTE,
    });
    expect(() =>
      parseSettingsEvent({
        contractVersion: PLATFORM_CONTRACT_VERSION,
        commandId: OPEN_SETTINGS_COMMAND_ID,
        route: "/qa/platform",
      }),
    ).toThrowError(/native Settings event is invalid/u);
  });
});
