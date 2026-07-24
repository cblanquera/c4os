import { invokeNative } from "./native-transport";
import {
  MAX_DIAGNOSTIC_BYTES,
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
  type SafeDetailValue,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export type StartupRecoveryBoundary =
  | "database"
  | "configuration"
  | "browserRegistry"
  | "extension"
  | "workspace"
  | "runtime"
  | "mcp"
  | "update";
export type StartupRecoveryLifecycle =
  "healthy" | "degraded" | "retrying" | "recovered";
export type StartupRecoveryAction =
  "retry" | "restoreValidatedBackup" | "openRecoveryLocation";
export type StartupRecoveryHistoryKind =
  | "failureReported"
  | "actionStarted"
  | "actionFailed"
  | "actionSucceeded"
  | "recoveryLocationOpened";

export interface StartupRecoveryFailure {
  readonly boundary: StartupRecoveryBoundary;
  readonly correlationId: string;
  readonly diagnosticCode: string;
  readonly message: string;
  readonly failedAtMs: number;
  readonly validatedBackupAvailable: boolean;
  readonly recoveryLocationAvailable: boolean;
}

export interface StartupRecoveryActiveAction {
  readonly boundary: StartupRecoveryBoundary;
  readonly action: StartupRecoveryAction;
  readonly startedAtMs: number;
}

export interface StartupRecoveryHistoryRecord {
  readonly generation: number;
  readonly boundary: StartupRecoveryBoundary;
  readonly lifecycle: StartupRecoveryLifecycle;
  readonly kind: StartupRecoveryHistoryKind;
  readonly action: StartupRecoveryAction | null;
  readonly correlationId: string;
  readonly diagnosticCode: string;
  readonly message: string;
  readonly occurredAtMs: number;
}

export interface StartupRecoverySnapshot {
  readonly schemaVersion: 1;
  readonly authority: "rust-startup-recovery";
  readonly generation: number;
  readonly lifecycle: StartupRecoveryLifecycle;
  readonly normalWorkAuthorized: boolean;
  readonly failure: StartupRecoveryFailure | null;
  readonly availableActions: readonly StartupRecoveryAction[];
  readonly activeAction: StartupRecoveryActiveAction | null;
  readonly history: readonly StartupRecoveryHistoryRecord[];
  readonly historyTruncated: number;
}

type StartupRecoveryCommand =
  "startup_recovery_snapshot" | "startup_recovery_action";

export interface StartupRecoveryTransport {
  invoke(
    command: StartupRecoveryCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface StartupRecoveryAdapter {
  readSnapshot(): Promise<StartupRecoverySnapshot>;
  performAction(
    action: StartupRecoveryAction,
  ): Promise<StartupRecoverySnapshot>;
}

const nativeAdapter = createStartupRecoveryAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readStartupRecoverySnapshot = () => nativeAdapter.readSnapshot();
export const performStartupRecoveryAction = (action: StartupRecoveryAction) =>
  nativeAdapter.performAction(action);

export function createStartupRecoveryAdapter(
  transport: StartupRecoveryTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
    readonly now?: () => number;
  } = {},
): StartupRecoveryAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  const now = options.now ?? Date.now;
  let generation = 0;
  let currentSnapshot: StartupRecoverySnapshot | null = null;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invokeSnapshot = async (
    command: StartupRecoveryCommand,
    input?: Readonly<Record<string, unknown>>,
  ): Promise<StartupRecoverySnapshot> => {
    const identity = request();
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        input === undefined
          ? { request: identity }
          : {
              request: identity,
              input: {
                ...input,
                expectedGeneration: identity.expectedGeneration,
              },
            },
      );
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = parseEnvelope(raw, identity);
    const payload = parseSnapshot(envelope.payload);
    if (payload.generation !== envelope.generation) {
      throw invalidPayload("The Startup Recovery generations do not match.");
    }
    if (envelope.generation < generation) {
      throw invalidPayload(
        "The Startup Recovery response generation regressed behind the live cursor.",
      );
    }
    generation = envelope.generation;
    currentSnapshot = payload;
    return payload;
  };

  return {
    readSnapshot: () => invokeSnapshot("startup_recovery_snapshot"),
    performAction(action) {
      if (
        currentSnapshot === null ||
        currentSnapshot.lifecycle !== "degraded" ||
        currentSnapshot.normalWorkAuthorized ||
        currentSnapshot.activeAction !== null ||
        !currentSnapshot.availableActions.includes(action)
      ) {
        throw invalidPayload(
          "The recovery action is not available in the latest native snapshot.",
        );
      }
      return invokeSnapshot("startup_recovery_action", {
        expectedGeneration: generation,
        action: asAction(action),
        requestedAtMs: asTimestamp(now(), "recovery request timestamp"),
      });
    },
  };
}

function parseEnvelope(
  raw: unknown,
  request: SnapshotRequest,
): { readonly generation: number; readonly payload: unknown } {
  const value = asObject(raw, "Startup Recovery response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Startup Recovery protocol version is invalid.");
  }
  if (
    asIdentifier(value.requestId, "response request ID") !==
      request.requestId ||
    asIdentifier(value.correlationId, "response correlation ID") !==
      request.correlationId
  ) {
    throw boundary(
      "correlationMismatch",
      "The Startup Recovery response identity did not match its request.",
    );
  }
  const responseGeneration = asGeneration(
    value.generation,
    "response generation",
  );
  if (responseGeneration < request.expectedGeneration) {
    throw invalidPayload("The Startup Recovery response generation regressed.");
  }
  return { generation: responseGeneration, payload: value.payload };
}

function parseSnapshot(raw: unknown): StartupRecoverySnapshot {
  const value = asObject(raw, "Startup Recovery snapshot");
  if (value.schemaVersion !== 1) {
    throw invalidPayload("The Startup Recovery schema version is unsupported.");
  }
  if (value.authority !== "rust-startup-recovery") {
    throw invalidPayload("The Startup Recovery authority is invalid.");
  }
  const snapshot: StartupRecoverySnapshot = {
    schemaVersion: 1,
    authority: "rust-startup-recovery",
    generation: asGeneration(value.generation, "recovery generation"),
    lifecycle: asLifecycle(value.lifecycle),
    normalWorkAuthorized: asBoolean(
      value.normalWorkAuthorized,
      "normal-work authority",
    ),
    failure: value.failure === null ? null : parseFailure(value.failure),
    availableActions: parseActions(value.availableActions),
    activeAction:
      value.activeAction === null
        ? null
        : parseActiveAction(value.activeAction),
    history: asArray(value.history, "recovery history", 64).map(parseHistory),
    historyTruncated: asGeneration(
      value.historyTruncated,
      "truncated history count",
    ),
  };
  assertSnapshotInvariants(snapshot);
  return snapshot;
}

function parseFailure(raw: unknown): StartupRecoveryFailure {
  const value = asObject(raw, "Startup Recovery failure");
  return {
    boundary: asBoundary(value.boundary),
    correlationId: asIdentifier(value.correlationId, "correlation ID"),
    diagnosticCode: asIdentifier(value.diagnosticCode, "diagnostic code"),
    message: asText(value.message, "recovery diagnostic", MAX_DIAGNOSTIC_BYTES),
    failedAtMs: asTimestamp(value.failedAtMs, "failure timestamp"),
    validatedBackupAvailable: asBoolean(
      value.validatedBackupAvailable,
      "validated backup state",
    ),
    recoveryLocationAvailable: asBoolean(
      value.recoveryLocationAvailable,
      "recovery location state",
    ),
  };
}

function parseActiveAction(raw: unknown): StartupRecoveryActiveAction {
  const value = asObject(raw, "active Startup Recovery action");
  return {
    boundary: asBoundary(value.boundary),
    action: asAction(value.action),
    startedAtMs: asTimestamp(value.startedAtMs, "action timestamp"),
  };
}

function parseHistory(raw: unknown): StartupRecoveryHistoryRecord {
  const value = asObject(raw, "Startup Recovery history record");
  return {
    generation: asGeneration(value.generation, "history generation"),
    boundary: asBoundary(value.boundary),
    lifecycle: asLifecycle(value.lifecycle),
    kind: asHistoryKind(value.kind),
    action: value.action === null ? null : asAction(value.action),
    correlationId: asIdentifier(value.correlationId, "history correlation ID"),
    diagnosticCode: asIdentifier(
      value.diagnosticCode,
      "history diagnostic code",
    ),
    message: asText(
      value.message,
      "recovery history message",
      MAX_DIAGNOSTIC_BYTES,
    ),
    occurredAtMs: asTimestamp(value.occurredAtMs, "history timestamp"),
  };
}

function parseActions(raw: unknown): readonly StartupRecoveryAction[] {
  const actions = asArray(raw, "available recovery actions", 3).map(asAction);
  if (new Set(actions).size !== actions.length) {
    throw invalidPayload("The available recovery actions contain duplicates.");
  }
  return actions;
}

function assertSnapshotInvariants(snapshot: StartupRecoverySnapshot): void {
  const authorizedLifecycle =
    snapshot.lifecycle === "healthy" || snapshot.lifecycle === "recovered";
  if (
    snapshot.normalWorkAuthorized !==
      (authorizedLifecycle &&
        snapshot.failure === null &&
        snapshot.activeAction === null) ||
    (snapshot.lifecycle === "degraded" &&
      (snapshot.failure === null || snapshot.activeAction !== null)) ||
    (snapshot.lifecycle === "retrying" &&
      (snapshot.failure === null || snapshot.activeAction === null))
  ) {
    throw invalidPayload("The Startup Recovery authority is inconsistent.");
  }
  if (
    authorizedLifecycle &&
    (snapshot.failure !== null ||
      snapshot.activeAction !== null ||
      snapshot.availableActions.length > 0)
  ) {
    throw invalidPayload(
      "Authorized startup state cannot expose failure or recovery actions.",
    );
  }
  if (
    snapshot.lifecycle === "retrying" &&
    snapshot.availableActions.length > 0
  ) {
    throw invalidPayload(
      "Retrying startup state cannot expose competing recovery actions.",
    );
  }
  if (
    snapshot.activeAction !== null &&
    snapshot.failure !== null &&
    snapshot.activeAction.boundary !== snapshot.failure.boundary
  ) {
    throw invalidPayload(
      "The active recovery action boundary does not match the failure.",
    );
  }
  if (snapshot.failure === null && snapshot.availableActions.length > 0) {
    throw invalidPayload(
      "Recovery actions cannot be available without a failure.",
    );
  }
  if (snapshot.failure !== null) {
    const expected = new Set<StartupRecoveryAction>(["retry"]);
    if (snapshot.failure.validatedBackupAvailable) {
      expected.add("restoreValidatedBackup");
    }
    if (snapshot.failure.recoveryLocationAvailable) {
      expected.add("openRecoveryLocation");
    }
    if (
      snapshot.availableActions.some((action) => !expected.has(action)) ||
      (snapshot.lifecycle === "degraded" &&
        Array.from(expected).some(
          (action) => !snapshot.availableActions.includes(action),
        ))
    ) {
      throw invalidPayload(
        "The available recovery actions contradict native failure state.",
      );
    }
  }
  let previousGeneration = -1;
  for (const record of snapshot.history) {
    if (
      record.generation > snapshot.generation ||
      record.generation < previousGeneration
    ) {
      throw invalidPayload(
        "The Startup Recovery history generation order is invalid.",
      );
    }
    previousGeneration = record.generation;
  }
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

function asArray(
  raw: unknown,
  label: string,
  maximum: number,
): readonly unknown[] {
  if (!Array.isArray(raw) || raw.length > maximum) {
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
    new TextEncoder().encode(raw).length > maximum ||
    hasControlCharacter(raw)
  ) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw;
}

function asGeneration(raw: unknown, label: string): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < 0) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as number;
}

function asTimestamp(raw: unknown, label: string): number {
  const value = asGeneration(raw, label);
  if (value === 0) throw invalidPayload(`${label} is invalid.`);
  return value;
}

function asBoundary(raw: unknown): StartupRecoveryBoundary {
  return asEnum(
    raw,
    [
      "database",
      "configuration",
      "browserRegistry",
      "extension",
      "workspace",
      "runtime",
      "mcp",
      "update",
    ] as const,
    "recovery boundary",
  );
}

function asLifecycle(raw: unknown): StartupRecoveryLifecycle {
  return asEnum(
    raw,
    ["healthy", "degraded", "retrying", "recovered"] as const,
    "recovery lifecycle",
  );
}

function asAction(raw: unknown): StartupRecoveryAction {
  return asEnum(
    raw,
    ["retry", "restoreValidatedBackup", "openRecoveryLocation"] as const,
    "recovery action",
  );
}

function asHistoryKind(raw: unknown): StartupRecoveryHistoryKind {
  return asEnum(
    raw,
    [
      "failureReported",
      "actionStarted",
      "actionFailed",
      "actionSucceeded",
      "recoveryLocationOpened",
    ] as const,
    "recovery history kind",
  );
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

function hasControlCharacter(value: string): boolean {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return codePoint < 32 || codePoint === 127;
  });
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
    "The native Startup Recovery service is unavailable.",
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
