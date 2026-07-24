import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
  type SafeDetailValue,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export type ApprovalPreset =
  "ask_for_approval" | "approve_safe_actions" | "approve_for_me" | "custom";

export type BrowserEnvironment =
  "app_wide" | "workspace_project" | "chat" | "none";

export type ConfigurationSettingsSnapshot = {
  readonly authority: "rust-configuration-service";
  readonly generation: number;
  readonly coordinatorGeneration: number;
  readonly policyVersion: number;
  readonly defaultApprovalPreset: ApprovalPreset;
  readonly restoreLastWorkspace: boolean;
  readonly inheritShellEnvironment: boolean;
  readonly shellEnvironmentAllowlist: readonly string[];
  readonly browserEnvironment: BrowserEnvironment;
  readonly defaultRuntime: "opencode" | "pi" | null;
  readonly defaultEnvironment: "local" | null;
  readonly modelRoute: string | null;
  readonly hasExternalError: boolean;
};

export type ConfigurationSettingsDraft = Pick<
  ConfigurationSettingsSnapshot,
  | "defaultApprovalPreset"
  | "restoreLastWorkspace"
  | "inheritShellEnvironment"
  | "browserEnvironment"
  | "defaultRuntime"
  | "defaultEnvironment"
>;

type ConfigurationCommand =
  | "configuration_snapshot"
  | "configuration_save_settings"
  | "configuration_open_external";

export interface ConfigurationTransport {
  invoke(
    command: ConfigurationCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface ConfigurationAdapter {
  readSnapshot(): Promise<ConfigurationSettingsSnapshot>;
  saveSettings(
    draft: ConfigurationSettingsDraft,
  ): Promise<ConfigurationSettingsSnapshot>;
  openExternal(): Promise<void>;
}

const nativeAdapter = createConfigurationAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readConfigurationSettings = () => nativeAdapter.readSnapshot();
export const saveConfigurationSettings = (draft: ConfigurationSettingsDraft) =>
  nativeAdapter.saveSettings(draft);
export const openConfigurationExternal = () => nativeAdapter.openExternal();

export function createConfigurationAdapter(
  transport: ConfigurationTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): ConfigurationAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;
  let coordinatorGeneration = 0;
  let policyVersion = 0;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invokeSnapshot = async (
    command: "configuration_snapshot" | "configuration_save_settings",
    input?: Readonly<Record<string, unknown>>,
  ): Promise<ConfigurationSettingsSnapshot> => {
    const identity = request();
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        input === undefined
          ? { request: identity }
          : { request: identity, input },
      );
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = parseEnvelope(raw, identity);
    const payload = parseSnapshot(envelope.payload);
    if (payload.generation !== envelope.generation) {
      throw invalidPayload("The Configuration generations do not match.");
    }
    generation = envelope.generation;
    coordinatorGeneration = payload.coordinatorGeneration;
    policyVersion = payload.policyVersion;
    return payload;
  };

  return {
    readSnapshot: () => invokeSnapshot("configuration_snapshot"),
    saveSettings(draft) {
      return invokeSnapshot("configuration_save_settings", {
        expectedConfigurationGeneration: generation,
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedPolicyVersion: policyVersion,
        defaultApprovalPreset: asEnum(
          draft.defaultApprovalPreset,
          [
            "ask_for_approval",
            "approve_safe_actions",
            "approve_for_me",
            "custom",
          ] as const,
          "approval preset",
        ),
        restoreLastWorkspace: asBoolean(
          draft.restoreLastWorkspace,
          "restore-last-Workspace state",
        ),
        inheritShellEnvironment: asBoolean(
          draft.inheritShellEnvironment,
          "shell-environment state",
        ),
        browserEnvironment: asEnum(
          draft.browserEnvironment,
          ["app_wide", "workspace_project", "chat", "none"] as const,
          "Browser Environment",
        ),
        defaultRuntime:
          draft.defaultRuntime === null
            ? null
            : asEnum(
                draft.defaultRuntime,
                ["opencode", "pi"] as const,
                "default runtime",
              ),
        defaultEnvironment:
          draft.defaultEnvironment === null
            ? null
            : asEnum(
                draft.defaultEnvironment,
                ["local"] as const,
                "default environment",
              ),
      });
    },
    async openExternal() {
      const identity = request();
      let raw: unknown;
      try {
        raw = await transport.invoke("configuration_open_external", {
          request: identity,
        });
      } catch (error) {
        throw normalizeFailure(error);
      }
      const envelope = parseEnvelope(raw, identity);
      if (envelope.generation < generation) {
        throw invalidPayload(
          "The Configuration response generation regressed.",
        );
      }
      const payload = asObject(envelope.payload, "Configuration open result");
      if (payload.opened !== true) {
        throw invalidPayload("The Configuration file was not opened.");
      }
      generation = envelope.generation;
    },
  };
}

function parseEnvelope(
  raw: unknown,
  request: SnapshotRequest,
): { readonly generation: number; readonly payload: unknown } {
  const value = asObject(raw, "Configuration response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Configuration protocol version is invalid.");
  }
  if (
    asIdentifier(value.requestId, "response request ID") !==
      request.requestId ||
    asIdentifier(value.correlationId, "response correlation ID") !==
      request.correlationId
  ) {
    throw boundary(
      "correlationMismatch",
      "The Configuration response identity did not match its request.",
    );
  }
  const responseGeneration = asGeneration(
    value.generation,
    "response generation",
  );
  if (responseGeneration < request.expectedGeneration) {
    throw invalidPayload("The Configuration response generation regressed.");
  }
  return { generation: responseGeneration, payload: value.payload };
}

function parseSnapshot(raw: unknown): ConfigurationSettingsSnapshot {
  const value = asObject(raw, "Configuration snapshot");
  if (value.authority !== "rust-configuration-service") {
    throw invalidPayload("The Configuration authority is invalid.");
  }
  return {
    authority: "rust-configuration-service",
    generation: asGeneration(value.generation, "configuration generation"),
    coordinatorGeneration: asGeneration(
      value.coordinatorGeneration,
      "coordinator generation",
    ),
    policyVersion: asGeneration(value.policyVersion, "policy version"),
    defaultApprovalPreset: asEnum(
      value.defaultApprovalPreset,
      [
        "ask_for_approval",
        "approve_safe_actions",
        "approve_for_me",
        "custom",
      ] as const,
      "approval preset",
    ),
    restoreLastWorkspace: asBoolean(
      value.restoreLastWorkspace,
      "restore-last-Workspace state",
    ),
    inheritShellEnvironment: asBoolean(
      value.inheritShellEnvironment,
      "shell-environment state",
    ),
    shellEnvironmentAllowlist: asArray(
      value.shellEnvironmentAllowlist,
      "shell environment allowlist",
    ).map((name) => asIdentifier(name, "environment variable name")),
    browserEnvironment: asEnum(
      value.browserEnvironment,
      ["app_wide", "workspace_project", "chat", "none"] as const,
      "Browser Environment",
    ),
    defaultRuntime: optionalEnum(
      value.defaultRuntime,
      ["opencode", "pi"] as const,
      "default runtime",
    ),
    defaultEnvironment: optionalEnum(
      value.defaultEnvironment,
      ["local"] as const,
      "default environment",
    ),
    modelRoute: optionalText(value.modelRoute, "model route", 512),
    hasExternalError: asBoolean(
      value.hasExternalError,
      "external configuration error state",
    ),
  };
}

function asObject(
  raw: unknown,
  label: string,
): Readonly<Record<string, unknown>> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) {
    throw invalidPayload(`${label} must be an object.`);
  }
  return raw as Readonly<Record<string, unknown>>;
}

function asArray(raw: unknown, label: string): readonly unknown[] {
  if (!Array.isArray(raw) || raw.length > 256) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw;
}

function asBoolean(raw: unknown, label: string): boolean {
  if (typeof raw !== "boolean") throw invalidPayload(`${label} is invalid.`);
  return raw;
}

function asIdentifier(raw: unknown, label: string): string {
  const value = asText(raw, label, MAX_IDENTIFIER_BYTES);
  if (!/^[0-9A-Za-z@_.:-]+$/.test(value)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function asText(raw: unknown, label: string, maximum: number): string {
  if (
    typeof raw !== "string" ||
    raw.trim().length === 0 ||
    raw.length > maximum ||
    hasControlCharacter(raw)
  ) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw;
}

function hasControlCharacter(value: string): boolean {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return codePoint < 32 || codePoint === 127;
  });
}

function optionalText(
  raw: unknown,
  label: string,
  maximum: number,
): string | null {
  return raw === null || raw === undefined ? null : asText(raw, label, maximum);
}

function asGeneration(raw: unknown, label: string): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < 0) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as number;
}

function asEnum<const Value extends string>(
  raw: unknown,
  values: readonly Value[],
  label: string,
): Value {
  if (typeof raw !== "string" || !values.includes(raw as Value)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as Value;
}

function optionalEnum<const Value extends string>(
  raw: unknown,
  values: readonly Value[],
  label: string,
): Value | null {
  return raw === null || raw === undefined ? null : asEnum(raw, values, label);
}

function normalizeFailure(error: unknown): ProtocolBoundaryError {
  if (error instanceof ProtocolBoundaryError) return error;
  if (typeof error === "object" && error !== null) {
    const value = error as {
      code?: unknown;
      details?: unknown;
      message?: unknown;
      retryable?: unknown;
    };
    if (typeof value.code === "string" && typeof value.message === "string") {
      return new ProtocolBoundaryError(
        value.code as never,
        value.message,
        value.retryable === true,
        safeBoundaryDetails(value.details),
      );
    }
  }
  return boundary(
    "unavailable",
    "The native Configuration service is unavailable.",
    true,
  );
}

function safeBoundaryDetails(
  raw: unknown,
): Readonly<Record<string, SafeDetailValue>> {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return {};
  const details: Record<string, SafeDetailValue> = {};
  for (const [key, candidate] of Object.entries(raw)) {
    if (
      typeof candidate !== "object" ||
      candidate === null ||
      Array.isArray(candidate)
    ) {
      continue;
    }
    const value = candidate as { kind?: unknown; value?: unknown };
    if (value.kind === "text" && typeof value.value === "string") {
      details[key] = { kind: "text", value: value.value };
    } else if (
      (value.kind === "integer" || value.kind === "unsigned") &&
      Number.isSafeInteger(value.value)
    ) {
      details[key] = { kind: value.kind, value: value.value as number };
    } else if (value.kind === "boolean" && typeof value.value === "boolean") {
      details[key] = { kind: "boolean", value: value.value };
    }
  }
  return details;
}

function secureUuid(prefix: string): string {
  const uuid = globalThis.crypto?.randomUUID?.();
  if (!uuid) {
    throw boundary(
      "unavailable",
      "Secure request identity generation is unavailable.",
      true,
    );
  }
  return `${prefix}:${uuid}`;
}

function invalidPayload(message: string): ProtocolBoundaryError {
  return boundary("invalidPayload", message);
}

function boundary(
  code: string,
  message: string,
  retryable = false,
): ProtocolBoundaryError {
  return new ProtocolBoundaryError(code as never, message, retryable);
}
