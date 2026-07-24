import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type PickerGrantId,
  type RequestId,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const PLATFORM_CONTRACT_VERSION = 1 as const;
export const PICKER_CONTRACT_VERSION = 1 as const;
export const PLATFORM_SETTINGS_EVENT = "c4os://platform/open-settings" as const;
export const OPEN_SETTINGS_COMMAND_ID = "c4os.command.openSettings" as const;
export const SETTINGS_ROUTE = "/settings/providers" as const;

export type PlatformCommand =
  "platform_snapshot" | "platform_reveal_main" | "platform_pick";
export type PickerPurpose =
  | "openProjectFolder"
  | "relocateProjectFolder"
  | "openWorkspaceArchive"
  | "saveWorkspaceArchive"
  | "attachChatFiles"
  | "openFile"
  | "openFolder";
export type PickerObjectKind = "file" | "folder";

export interface PlatformSnapshot {
  readonly contractVersion: typeof PLATFORM_CONTRACT_VERSION;
  readonly platform: "macos";
  readonly architecture: "aarch64";
  readonly initialTheme: {
    readonly scheme: "light" | "dark";
    readonly source:
      "macosAppearance" | "webviewPreferredColorScheme" | "semanticFallback";
  };
  readonly liveThemeSource: "webviewPrefersColorScheme";
  readonly window: {
    readonly decorations: "standard";
    readonly titlebarTransparent: false;
    readonly titlebarOverlay: false;
    readonly initiallyVisible: false;
    readonly revealFallbackTimeoutMs: number;
  };
  readonly vocabulary: {
    readonly revealAction: "Reveal in Finder";
    readonly primaryModifierSymbol: "⌘";
    readonly alternateModifierSymbol: "⌥";
    readonly shiftModifierSymbol: "⇧";
  };
  readonly capabilities: {
    readonly nativeApplicationMenu: boolean;
    readonly nativeSettingsShortcut: boolean;
    readonly nativeFilePicker: boolean;
    readonly nativeFolderPicker: boolean;
    readonly nativeWorkspacePicker: boolean;
    readonly standardWindowDecorations: boolean;
  };
  readonly settingsMenu: {
    readonly menuItemId: "c4os.menu.settings";
    readonly commandId: typeof OPEN_SETTINGS_COMMAND_ID;
    readonly route: typeof SETTINGS_ROUTE;
    readonly accelerator: "CmdOrCtrl+,";
    readonly keyboardLabel: "⌘,";
  };
}

export interface PickerGrantSnapshot {
  readonly grantId: PickerGrantId;
  readonly objectKind: PickerObjectKind;
  readonly displayName: string;
}

export type PickerOutcome =
  | {
      readonly type: "cancelled";
      readonly contractVersion: typeof PICKER_CONTRACT_VERSION;
      readonly requestId: RequestId;
    }
  | {
      readonly type: "selected";
      readonly contractVersion: typeof PICKER_CONTRACT_VERSION;
      readonly requestId: RequestId;
      readonly grants: readonly PickerGrantSnapshot[];
    };

interface PlatformEnvelope<Payload> {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly payload: Payload;
}

interface NativePickerRequest {
  readonly contractVersion: typeof PICKER_CONTRACT_VERSION;
  readonly requestId: RequestId;
  readonly purpose: PickerPurpose;
  readonly selection: {
    readonly objectKind: PickerObjectKind;
    readonly allowsMultiple: boolean;
    readonly allowedExtensions: readonly string[];
  };
}

export interface PlatformTransport {
  invoke(
    command: PlatformCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface PlatformAdapter {
  readSnapshot(): Promise<PlatformSnapshot>;
  revealMainWindow(): Promise<void>;
  pick(purpose: PickerPurpose): Promise<PickerOutcome>;
}

const nativeAdapter = createPlatformAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

let nativeSettingsReturnRoute: string | null = null;

export function readPlatformSnapshot(): Promise<PlatformSnapshot> {
  return nativeAdapter.readSnapshot();
}

export function revealMainWindow(): Promise<void> {
  return nativeAdapter.revealMainWindow();
}

export function pickNative(purpose: PickerPurpose): Promise<PickerOutcome> {
  return nativeAdapter.pick(purpose);
}

export async function listenForNativeSettings(
  navigate: (route: typeof SETTINGS_ROUTE) => void,
): Promise<UnlistenFn> {
  return listen<unknown>(PLATFORM_SETTINGS_EVENT, ({ payload }) => {
    const event = parseSettingsEvent(payload);
    const currentRoute = window.location.hash.slice(1) || "/";
    if (
      currentRoute.startsWith("/") &&
      !currentRoute.startsWith("//") &&
      currentRoute !== SETTINGS_ROUTE
    ) {
      nativeSettingsReturnRoute = currentRoute;
    }
    navigate(event.route);
  });
}

export function getNativeSettingsReturnRoute(): string | null {
  return nativeSettingsReturnRoute;
}

export function createPlatformAdapter(
  transport: PlatformTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): PlatformAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request ID") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation ID") as CorrelationId);

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: 0 as StateGeneration,
  });

  const invoke = async <Payload>(
    command: PlatformCommand,
    snapshotRequest: SnapshotRequest,
    args: Readonly<Record<string, unknown>>,
    parsePayload: (raw: unknown) => Payload,
  ): Promise<Payload> => {
    let raw: unknown;
    try {
      raw = await transport.invoke(command, args);
    } catch {
      throw new ProtocolBoundaryError(
        "unavailable",
        "The native platform service is unavailable.",
        true,
      );
    }
    const envelope = parseEnvelope(raw, parsePayload);
    if (
      envelope.requestId !== snapshotRequest.requestId ||
      envelope.correlationId !== snapshotRequest.correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The platform response identity did not match its request.",
      );
    }
    if (envelope.generation !== 0) {
      throw invalidPayload("The platform generation is invalid.");
    }
    return envelope.payload;
  };

  return {
    readSnapshot() {
      const snapshotRequest = request();
      return invoke(
        "platform_snapshot",
        snapshotRequest,
        { request: snapshotRequest },
        parsePlatformSnapshot,
      );
    },
    async revealMainWindow() {
      const snapshotRequest = request();
      await invoke(
        "platform_reveal_main",
        snapshotRequest,
        { request: snapshotRequest },
        parseRevealSnapshot,
      );
    },
    pick(purpose) {
      const snapshotRequest = request();
      const picker = createPickerRequest(snapshotRequest.requestId, purpose);
      return invoke(
        "platform_pick",
        snapshotRequest,
        { request: snapshotRequest, picker },
        (raw) => parsePickerOutcome(raw, snapshotRequest.requestId, picker),
      );
    },
  };
}

function createPickerRequest(
  requestId: RequestId,
  purpose: PickerPurpose,
): NativePickerRequest {
  const isFolder =
    purpose === "openProjectFolder" ||
    purpose === "relocateProjectFolder" ||
    purpose === "openFolder";
  return {
    contractVersion: PICKER_CONTRACT_VERSION,
    requestId,
    purpose,
    selection: {
      objectKind: isFolder ? "folder" : "file",
      allowsMultiple: purpose === "attachChatFiles",
      allowedExtensions:
        purpose === "openWorkspaceArchive" || purpose === "saveWorkspaceArchive"
          ? ["zip"]
          : [],
    },
  };
}

function parseEnvelope<Payload>(
  raw: unknown,
  parsePayload: (raw: unknown) => Payload,
): PlatformEnvelope<Payload> {
  const envelope = exactRecord(
    raw,
    ["protocolVersion", "requestId", "correlationId", "generation", "payload"],
    "platform envelope",
  );
  if (envelope.protocolVersion !== PROTOCOL_VERSION) {
    throw new ProtocolBoundaryError(
      "unknownProtocolVersion",
      "The platform protocol version is unsupported.",
    );
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: asIdentifier(envelope.requestId, "request ID") as RequestId,
    correlationId: asIdentifier(
      envelope.correlationId,
      "correlation ID",
    ) as CorrelationId,
    generation: asGeneration(envelope.generation),
    payload: parsePayload(envelope.payload),
  };
}

function parsePlatformSnapshot(raw: unknown): PlatformSnapshot {
  const snapshot = exactRecord(
    raw,
    [
      "contractVersion",
      "platform",
      "architecture",
      "initialTheme",
      "liveThemeSource",
      "window",
      "vocabulary",
      "capabilities",
      "settingsMenu",
    ],
    "platform snapshot",
  );
  if (
    snapshot.contractVersion !== PLATFORM_CONTRACT_VERSION ||
    snapshot.platform !== "macos" ||
    snapshot.architecture !== "aarch64" ||
    snapshot.liveThemeSource !== "webviewPrefersColorScheme"
  ) {
    throw invalidPayload("The platform snapshot target is invalid.");
  }
  const initialTheme = exactRecord(
    snapshot.initialTheme,
    ["scheme", "source"],
    "initial theme",
  );
  if (
    (initialTheme.scheme !== "light" && initialTheme.scheme !== "dark") ||
    ![
      "macosAppearance",
      "webviewPreferredColorScheme",
      "semanticFallback",
    ].includes(String(initialTheme.source))
  ) {
    throw invalidPayload("The initial platform theme is invalid.");
  }
  const window = exactRecord(
    snapshot.window,
    [
      "decorations",
      "titlebarTransparent",
      "titlebarOverlay",
      "initiallyVisible",
      "revealFallbackTimeoutMs",
    ],
    "window contract",
  );
  if (
    window.decorations !== "standard" ||
    window.titlebarTransparent !== false ||
    window.titlebarOverlay !== false ||
    window.initiallyVisible !== false ||
    !Number.isSafeInteger(window.revealFallbackTimeoutMs) ||
    (window.revealFallbackTimeoutMs as number) < 1
  ) {
    throw invalidPayload("The native window contract is invalid.");
  }
  const vocabulary = exactRecord(
    snapshot.vocabulary,
    [
      "revealAction",
      "primaryModifierSymbol",
      "alternateModifierSymbol",
      "shiftModifierSymbol",
    ],
    "platform vocabulary",
  );
  if (
    vocabulary.revealAction !== "Reveal in Finder" ||
    vocabulary.primaryModifierSymbol !== "⌘" ||
    vocabulary.alternateModifierSymbol !== "⌥" ||
    vocabulary.shiftModifierSymbol !== "⇧"
  ) {
    throw invalidPayload("The platform vocabulary is invalid.");
  }
  const capabilities = parseCapabilities(snapshot.capabilities);
  const settingsMenu = exactRecord(
    snapshot.settingsMenu,
    ["menuItemId", "commandId", "route", "accelerator", "keyboardLabel"],
    "Settings menu contract",
  );
  if (
    settingsMenu.menuItemId !== "c4os.menu.settings" ||
    settingsMenu.commandId !== OPEN_SETTINGS_COMMAND_ID ||
    settingsMenu.route !== SETTINGS_ROUTE ||
    settingsMenu.accelerator !== "CmdOrCtrl+," ||
    settingsMenu.keyboardLabel !== "⌘,"
  ) {
    throw invalidPayload("The native Settings menu contract is invalid.");
  }
  return {
    contractVersion: PLATFORM_CONTRACT_VERSION,
    platform: "macos",
    architecture: "aarch64",
    initialTheme: {
      scheme: initialTheme.scheme as "light" | "dark",
      source: initialTheme.source as PlatformSnapshot["initialTheme"]["source"],
    },
    liveThemeSource: "webviewPrefersColorScheme",
    window: {
      decorations: "standard",
      titlebarTransparent: false,
      titlebarOverlay: false,
      initiallyVisible: false,
      revealFallbackTimeoutMs: window.revealFallbackTimeoutMs as number,
    },
    vocabulary: {
      revealAction: "Reveal in Finder",
      primaryModifierSymbol: "⌘",
      alternateModifierSymbol: "⌥",
      shiftModifierSymbol: "⇧",
    },
    capabilities,
    settingsMenu: {
      menuItemId: "c4os.menu.settings",
      commandId: OPEN_SETTINGS_COMMAND_ID,
      route: SETTINGS_ROUTE,
      accelerator: "CmdOrCtrl+,",
      keyboardLabel: "⌘,",
    },
  };
}

function parseCapabilities(raw: unknown): PlatformSnapshot["capabilities"] {
  const keys = [
    "nativeApplicationMenu",
    "nativeSettingsShortcut",
    "nativeFilePicker",
    "nativeFolderPicker",
    "nativeWorkspacePicker",
    "standardWindowDecorations",
  ] as const;
  const capabilities = exactRecord(raw, keys, "platform capabilities");
  if (keys.some((key) => typeof capabilities[key] !== "boolean")) {
    throw invalidPayload("The platform capability map is invalid.");
  }
  return {
    nativeApplicationMenu: capabilities.nativeApplicationMenu as boolean,
    nativeSettingsShortcut: capabilities.nativeSettingsShortcut as boolean,
    nativeFilePicker: capabilities.nativeFilePicker as boolean,
    nativeFolderPicker: capabilities.nativeFolderPicker as boolean,
    nativeWorkspacePicker: capabilities.nativeWorkspacePicker as boolean,
    standardWindowDecorations:
      capabilities.standardWindowDecorations as boolean,
  };
}

function parseRevealSnapshot(raw: unknown): void {
  const snapshot = exactRecord(raw, ["revealed", "fallback"], "reveal result");
  if (snapshot.revealed !== true || snapshot.fallback !== false) {
    throw invalidPayload("The native reveal result is invalid.");
  }
}

function parsePickerOutcome(
  raw: unknown,
  expectedRequestId: RequestId,
  picker: NativePickerRequest,
): PickerOutcome {
  const record = requireRecord(raw, "picker outcome");
  if (record.type === "cancelled") {
    const cancelled = exactRecord(
      record,
      ["type", "contractVersion", "requestId"],
      "cancelled picker outcome",
    );
    requirePickerIdentity(cancelled, expectedRequestId);
    return {
      type: "cancelled",
      contractVersion: PICKER_CONTRACT_VERSION,
      requestId: expectedRequestId,
    };
  }
  if (record.type !== "selected") {
    throw invalidPayload("The native picker outcome type is invalid.");
  }
  const selected = exactRecord(
    record,
    ["type", "contractVersion", "requestId", "grants"],
    "selected picker outcome",
  );
  requirePickerIdentity(selected, expectedRequestId);
  if (
    !Array.isArray(selected.grants) ||
    selected.grants.length === 0 ||
    selected.grants.length > 64 ||
    (!picker.selection.allowsMultiple && selected.grants.length !== 1)
  ) {
    throw invalidPayload("The native picker grant count is invalid.");
  }
  const grantIds = new Set<string>();
  const grants = selected.grants.map((rawGrant) => {
    const grant = exactRecord(
      rawGrant,
      ["grantId", "objectKind", "displayName"],
      "picker grant",
    );
    const grantId = asIdentifier(
      grant.grantId,
      "picker grant ID",
    ) as PickerGrantId;
    if (
      grantIds.has(grantId) ||
      grant.objectKind !== picker.selection.objectKind ||
      typeof grant.displayName !== "string" ||
      grant.displayName.trim().length === 0 ||
      grant.displayName.length > 512 ||
      Array.from(grant.displayName).some((character) => {
        const codePoint = character.codePointAt(0) ?? 0;
        return (
          character === "/" ||
          character === "\\" ||
          codePoint < 32 ||
          codePoint === 127
        );
      })
    ) {
      throw invalidPayload("The native picker grant is invalid.");
    }
    grantIds.add(grantId);
    return {
      grantId,
      objectKind: grant.objectKind,
      displayName: grant.displayName,
    } as PickerGrantSnapshot;
  });
  return {
    type: "selected",
    contractVersion: PICKER_CONTRACT_VERSION,
    requestId: expectedRequestId,
    grants,
  };
}

function requirePickerIdentity(
  outcome: Record<string, unknown>,
  requestId: RequestId,
): void {
  if (
    outcome.contractVersion !== PICKER_CONTRACT_VERSION ||
    asIdentifier(outcome.requestId, "picker request ID") !== requestId
  ) {
    throw new ProtocolBoundaryError(
      "correlationMismatch",
      "The picker outcome did not match its request.",
    );
  }
}

export function parseSettingsEvent(raw: unknown): {
  readonly contractVersion: typeof PLATFORM_CONTRACT_VERSION;
  readonly commandId: typeof OPEN_SETTINGS_COMMAND_ID;
  readonly route: typeof SETTINGS_ROUTE;
} {
  const event = exactRecord(
    raw,
    ["contractVersion", "commandId", "route"],
    "native Settings event",
  );
  if (
    event.contractVersion !== PLATFORM_CONTRACT_VERSION ||
    event.commandId !== OPEN_SETTINGS_COMMAND_ID ||
    event.route !== SETTINGS_ROUTE
  ) {
    throw invalidPayload("The native Settings event is invalid.");
  }
  return {
    contractVersion: PLATFORM_CONTRACT_VERSION,
    commandId: OPEN_SETTINGS_COMMAND_ID,
    route: SETTINGS_ROUTE,
  };
}

function secureUuid(label: string): string {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw new ProtocolBoundaryError(
      "unavailable",
      `Secure ${label} generation is unavailable.`,
    );
  }
  return globalThis.crypto.randomUUID();
}

function exactRecord(
  value: unknown,
  expectedKeys: readonly string[],
  label: string,
): Record<string, unknown> {
  const record = requireRecord(value, label);
  const keys = Object.keys(record).sort();
  const expected = [...expectedKeys].sort();
  if (
    keys.length !== expected.length ||
    keys.some((key, index) => key !== expected[index])
  ) {
    throw invalidPayload(`The ${label} fields are invalid.`);
  }
  return record;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return value as Record<string, unknown>;
}

function asIdentifier(value: unknown, label: string): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > MAX_IDENTIFIER_BYTES ||
    !/^[A-Za-z0-9_.:@-]+$/u.test(value)
  ) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return value;
}

function asGeneration(value: unknown): StateGeneration {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw invalidPayload("The platform generation is invalid.");
  }
  return value as StateGeneration;
}

function invalidPayload(message: string): ProtocolBoundaryError {
  return new ProtocolBoundaryError("invalidPayload", message);
}
