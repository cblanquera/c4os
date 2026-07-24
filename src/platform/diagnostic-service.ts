import type { DiagnosticCategory as GeneratedDiagnosticCategory } from "../generated/DiagnosticCategory";
import type { DiagnosticSeverity as GeneratedDiagnosticSeverity } from "../generated/DiagnosticSeverity";
import type { DiagnosticsExportSnapshot as GeneratedDiagnosticsExportSnapshot } from "../generated/DiagnosticsExportSnapshot";
import type { DiagnosticsSnapshot as GeneratedDiagnosticsSnapshot } from "../generated/DiagnosticsSnapshot";
import type { UpdateDiagnosticRecord as GeneratedDiagnosticRecord } from "../generated/UpdateDiagnosticRecord";
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
import type { UpdateRecoveryAction } from "./update-service";

export type DiagnosticSeverity = "info" | "warning" | "error";
export type DiagnosticCategory =
  | "update"
  | "recovery"
  | "configuration"
  | "runtime"
  | "plugin"
  | "security"
  | "persistence";

export interface DiagnosticRecord {
  readonly diagnosticId: string;
  readonly correlationId: string;
  readonly category: DiagnosticCategory;
  readonly severity: DiagnosticSeverity;
  readonly componentBoundary: string;
  readonly message: string;
  readonly recoveryAction: UpdateRecoveryAction | null;
  readonly createdAtMs: number;
}

export interface DiagnosticsSnapshot {
  readonly schemaVersion: number;
  readonly generation: number;
  readonly records: readonly DiagnosticRecord[];
  readonly truncated: boolean;
}

export interface DiagnosticsExportSnapshot extends DiagnosticsSnapshot {
  readonly exportId: string;
  readonly createdAtMs: number;
  readonly sha256: string;
}

type Assert<Condition extends true> = Condition;
type KeysMatch<Left, Right> = keyof Left extends keyof Right
  ? keyof Right extends keyof Left
    ? true
    : false
  : false;
type DiagnosticsSnapshotKeysMatch = Assert<
  KeysMatch<DiagnosticsSnapshot, GeneratedDiagnosticsSnapshot>
>;
type DiagnosticsExportKeysMatch = Assert<
  KeysMatch<DiagnosticsExportSnapshot, GeneratedDiagnosticsExportSnapshot>
>;
type DiagnosticRecordKeysMatch = Assert<
  KeysMatch<DiagnosticRecord, GeneratedDiagnosticRecord>
>;
type DiagnosticCategoryMatches = Assert<
  DiagnosticCategory extends GeneratedDiagnosticCategory
    ? GeneratedDiagnosticCategory extends DiagnosticCategory
      ? true
      : false
    : false
>;
type DiagnosticSeverityMatches = Assert<
  DiagnosticSeverity extends GeneratedDiagnosticSeverity
    ? GeneratedDiagnosticSeverity extends DiagnosticSeverity
      ? true
      : false
    : false
>;

export type DiagnosticSchemaCompatibilityChecks =
  | DiagnosticsSnapshotKeysMatch
  | DiagnosticsExportKeysMatch
  | DiagnosticRecordKeysMatch
  | DiagnosticCategoryMatches
  | DiagnosticSeverityMatches;

type DiagnosticCommand = "diagnostics_snapshot" | "diagnostics_export";

export interface DiagnosticTransport {
  invoke(
    command: DiagnosticCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface DiagnosticAdapter {
  readSnapshot(): Promise<DiagnosticsSnapshot>;
  exportSnapshot(): Promise<DiagnosticsExportSnapshot>;
}

const nativeAdapter = createDiagnosticAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readDiagnosticsSnapshot = () => nativeAdapter.readSnapshot();
export const exportDiagnosticsSnapshot = () => nativeAdapter.exportSnapshot();

export function createDiagnosticAdapter(
  transport: DiagnosticTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): DiagnosticAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invokeSnapshot = async (
    command: DiagnosticCommand,
  ): Promise<DiagnosticsSnapshot | DiagnosticsExportSnapshot> => {
    const identity = request();
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        command === "diagnostics_export"
          ? {
              request: identity,
              input: { expectedGeneration: identity.expectedGeneration },
            }
          : { request: identity },
      );
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = parseEnvelope(raw, identity);
    const payload =
      command === "diagnostics_export"
        ? parseExport(envelope.payload)
        : parseSnapshot(envelope.payload);
    if (payload.generation !== envelope.generation) {
      throw invalidPayload("The Diagnostics generations do not match.");
    }
    if (envelope.generation < generation) {
      throw invalidPayload(
        "The Diagnostics response generation regressed behind the live cursor.",
      );
    }
    generation = envelope.generation;
    return payload;
  };

  return {
    readSnapshot: async () =>
      (await invokeSnapshot("diagnostics_snapshot")) as DiagnosticsSnapshot,
    exportSnapshot: async () =>
      (await invokeSnapshot("diagnostics_export")) as DiagnosticsExportSnapshot,
  };
}

function parseEnvelope(
  raw: unknown,
  request: SnapshotRequest,
): { readonly generation: number; readonly payload: unknown } {
  const value = asObject(raw, "Diagnostics response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Diagnostics protocol version is invalid.");
  }
  if (
    asIdentifier(value.requestId, "response request ID") !==
      request.requestId ||
    asIdentifier(value.correlationId, "response correlation ID") !==
      request.correlationId
  ) {
    throw boundary(
      "correlationMismatch",
      "The Diagnostics response identity did not match its request.",
    );
  }
  const responseGeneration = asGeneration(
    value.generation,
    "response generation",
  );
  if (responseGeneration < request.expectedGeneration) {
    throw invalidPayload("The Diagnostics response generation regressed.");
  }
  return { generation: responseGeneration, payload: value.payload };
}

function parseSnapshot(raw: unknown): DiagnosticsSnapshot {
  const value = asObject(raw, "Diagnostics snapshot");
  if (value.schemaVersion !== 1) {
    throw invalidPayload("The Diagnostics schema version is unsupported.");
  }
  return {
    schemaVersion: 1,
    generation: asGeneration(value.generation, "diagnostics generation"),
    records: asArray(value.records, "diagnostic records", 256).map(parseRecord),
    truncated: asBoolean(value.truncated, "diagnostic truncation state"),
  };
}

function parseExport(raw: unknown): DiagnosticsExportSnapshot {
  const value = asObject(raw, "Diagnostics export");
  const snapshot = parseSnapshot(value);
  return {
    ...snapshot,
    exportId: asIdentifier(value.exportId, "diagnostics export ID"),
    createdAtMs: asTimestamp(value.createdAtMs, "export timestamp"),
    sha256: asSha256(value.sha256, "export digest"),
  };
}

function parseRecord(raw: unknown): DiagnosticRecord {
  const value = asObject(raw, "diagnostic record");
  return {
    diagnosticId: asIdentifier(value.diagnosticId, "diagnostic ID"),
    correlationId: asIdentifier(value.correlationId, "correlation ID"),
    category: asEnum(
      value.category,
      [
        "update",
        "recovery",
        "configuration",
        "runtime",
        "plugin",
        "security",
        "persistence",
      ] as const,
      "diagnostic category",
    ),
    severity: asEnum(
      value.severity,
      ["info", "warning", "error"] as const,
      "diagnostic severity",
    ),
    componentBoundary: asIdentifier(
      value.componentBoundary,
      "component boundary",
    ),
    message: asText(value.message, "diagnostic message", MAX_DIAGNOSTIC_BYTES),
    recoveryAction: optionalEnum(
      value.recoveryAction,
      [
        "retry_stage",
        "activate_staged",
        "rollback_last_known_good",
        "recover_interrupted",
        "remove_revocation",
        "review_plugin_update",
        "rebuild_application",
        "wait_for_active_runs",
        "rebuild_runtime",
        "rebuild_last_known_good",
        "review_runtime_crash_loop",
      ] as const,
      "diagnostic recovery action",
    ),
    createdAtMs: asTimestamp(value.createdAtMs, "diagnostic timestamp"),
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
  return asGeneration(raw, label);
}

function asSha256(raw: unknown, label: string): string {
  if (typeof raw !== "string" || !/^sha256:[0-9a-f]{64}$/.test(raw)) {
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
    "The native Diagnostics service is unavailable.",
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
