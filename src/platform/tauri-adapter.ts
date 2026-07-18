import {
  MAX_DIAGNOSTIC_BYTES,
  MAX_IDENTIFIER_BYTES,
  MAX_REDACTION_MARKERS,
  MAX_SAFE_DETAILS,
  PROTOCOL_VERSION,
  type ApprovalId,
  type CorrelationId,
  type CoreEvent,
  type EventEnvelope,
  type FoundationSnapshot,
  type FoundationSnapshotEnvelope,
  type ProcessGeneration,
  type ProtocolError,
  type ProtocolErrorCode,
  type RedactionMarker,
  type RedactionReason,
  type RequestId,
  type RunPhase,
  type RunScope,
  type SafeDetailValue,
  type SnapshotRequest,
  type StateGeneration,
  type StateSnapshot,
  type RuntimeHealth,
} from "./protocol";

export const FOUNDATION_SNAPSHOT_COMMAND = "foundation_snapshot" as const;

/** The injected transport is intentionally unable to name another command. */
export interface FoundationTauriTransport {
  invoke(
    command: typeof FOUNDATION_SNAPSHOT_COMMAND,
    args: { readonly request: SnapshotRequest },
  ): Promise<unknown>;
}

export type RequestIdFactory = () => RequestId;
export type CorrelationIdFactory = () => CorrelationId;

export interface FoundationTauriAdapter {
  readonly currentGeneration: StateGeneration;
  readFoundationSnapshot(): Promise<FoundationSnapshot>;
  acceptCoreEvent(rawEvent: unknown): EventEnvelope;
}

export class ProtocolBoundaryError extends Error {
  readonly name = "ProtocolBoundaryError";

  constructor(
    readonly code: ProtocolErrorCode,
    message: string,
    readonly retryable = false,
    readonly details: Readonly<Record<string, SafeDetailValue>> = {},
  ) {
    super(message);
  }
}

export function createTauriAdapter(
  transport: FoundationTauriTransport,
  options: {
    readonly requestIdFactory?: RequestIdFactory;
    readonly correlationIdFactory?: CorrelationIdFactory;
    readonly initialGeneration?: StateGeneration;
    readonly expectedProcessGeneration?: ProcessGeneration;
  } = {},
): FoundationTauriAdapter {
  let currentGeneration = options.initialGeneration ?? asStateGeneration(0);
  const requestIdFactory = options.requestIdFactory ?? defaultRequestIdFactory;
  const correlationIdFactory =
    options.correlationIdFactory ?? defaultCorrelationIdFactory;

  const requireFreshGeneration = (candidate: StateGeneration): void => {
    if (candidate < currentGeneration) {
      throw generationError("staleGeneration", candidate, currentGeneration);
    }
  };

  return {
    get currentGeneration() {
      return currentGeneration;
    },

    async readFoundationSnapshot(): Promise<FoundationSnapshot> {
      const requestId = requestIdFactory();
      const correlationId = correlationIdFactory();
      const request: SnapshotRequest = {
        protocolVersion: PROTOCOL_VERSION,
        requestId,
        correlationId,
        expectedGeneration: currentGeneration,
      };

      let rawResponse: unknown;
      try {
        rawResponse = await transport.invoke(FOUNDATION_SNAPSHOT_COMMAND, {
          request,
        });
      } catch (error) {
        const coreError = tryParseProtocolError(error);
        if (coreError !== null) {
          throw boundaryErrorFromCore(coreError);
        }
        throw error;
      }

      const response = parseFoundationSnapshotEnvelope(rawResponse);
      if (
        response.requestId !== requestId ||
        response.correlationId !== correlationId
      ) {
        throw new ProtocolBoundaryError(
          "correlationMismatch",
          "The foundation snapshot response identity did not match its request.",
          false,
          {
            expectedRequestId: textDetail(requestId),
            actualRequestId: textDetail(response.requestId),
            expectedCorrelationId: textDetail(correlationId),
            actualCorrelationId: textDetail(response.correlationId),
          },
        );
      }

      requireFreshGeneration(response.generation);
      if (response.payload.protocolVersion !== PROTOCOL_VERSION) {
        throw unknownVersion(response.payload.protocolVersion);
      }
      if (response.payload.generation !== response.generation) {
        throw generationError(
          "staleGeneration",
          response.payload.generation,
          response.generation,
        );
      }

      currentGeneration = response.generation;
      return response.payload;
    },

    acceptCoreEvent(rawEvent: unknown): EventEnvelope {
      const envelope = parseEventEnvelope(rawEvent);
      requireFreshGeneration(envelope.generation);

      if (envelope.runScope !== null) {
        if (envelope.runScope.correlationId !== envelope.correlationId) {
          throw new ProtocolBoundaryError(
            "correlationMismatch",
            "The event and run-scope correlations differ.",
          );
        }
        const expected = options.expectedProcessGeneration;
        if (
          expected !== undefined &&
          envelope.runScope.processGeneration !== expected
        ) {
          throw generationError(
            envelope.runScope.processGeneration < expected
              ? "staleGeneration"
              : "futureGeneration",
            envelope.runScope.processGeneration,
            expected,
          );
        }
      }

      currentGeneration = envelope.generation;
      return envelope;
    },
  };
}

function defaultRequestIdFactory(): RequestId {
  return secureUuid("request ID") as RequestId;
}

function defaultCorrelationIdFactory(): CorrelationId {
  return secureUuid("correlation ID") as CorrelationId;
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

function parseFoundationSnapshotEnvelope(
  raw: unknown,
): FoundationSnapshotEnvelope {
  const envelope = parseBaseEnvelope(raw);
  return { ...envelope, payload: parseFoundationSnapshot(envelope.payload) };
}

function parseBaseEnvelope(raw: unknown) {
  const envelope = requireRecord(raw, "protocol envelope");
  requireProtocolVersion(envelope.protocolVersion);
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: asRequestId(envelope.requestId),
    correlationId: asCorrelationId(envelope.correlationId),
    generation: asStateGeneration(envelope.generation),
    payload: envelope.payload,
  };
}

function parseFoundationSnapshot(raw: unknown): FoundationSnapshot {
  const snapshot = requireRecord(raw, "foundation snapshot");
  requireProtocolVersion(snapshot.protocolVersion);
  if (snapshot.authority !== "rust-core") {
    throw invalidPayload("The foundation snapshot has an unknown authority.");
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    generation: asStateGeneration(snapshot.generation),
    authority: "rust-core",
    redactions: parseRedactions(snapshot.redactions),
  };
}

function parseEventEnvelope(raw: unknown): EventEnvelope {
  const envelope = requireRecord(raw, "event envelope");
  requireProtocolVersion(envelope.protocolVersion);
  const event = parseCoreEvent(envelope.event);
  const runScope =
    envelope.runScope === null ? null : parseRunScope(envelope.runScope);

  if (
    (event.type === "runChanged" || event.type === "runtimeHealthChanged") &&
    runScope === null
  ) {
    throw invalidPayload("A run-scoped event is missing its run identity.");
  }

  return {
    protocolVersion: PROTOCOL_VERSION,
    eventId: asRequestId(envelope.eventId),
    correlationId: asCorrelationId(envelope.correlationId),
    generation: asStateGeneration(envelope.generation),
    runScope,
    event,
  };
}

function parseCoreEvent(raw: unknown): CoreEvent {
  const event = requireRecord(raw, "core event");
  const type = requireString(event.type, "event type");
  const payload = event.payload;

  switch (type) {
    case "stateSnapshot":
      return { type, payload: parseStateSnapshot(payload) };
    case "stateChanged": {
      const record = requireRecord(payload, "state change payload");
      if (
        !Array.isArray(record.areas) ||
        record.areas.length > MAX_SAFE_DETAILS
      ) {
        throw invalidPayload(
          "The changed state areas are invalid or unbounded.",
        );
      }
      return {
        type,
        payload: {
          areas: record.areas.map((area) => asIdentifier(area, "state area")),
        },
      };
    }
    case "runChanged": {
      const record = requireRecord(payload, "run change payload");
      return {
        type,
        payload: {
          sequence: requireNonNegativeInteger(record.sequence, "run sequence"),
          phase: parseRunPhase(record.phase),
        },
      };
    }
    case "runtimeHealthChanged": {
      const record = requireRecord(payload, "runtime health payload");
      return { type, payload: { health: parseRuntimeHealth(record.health) } };
    }
    case "approvalChanged": {
      const record = requireRecord(payload, "approval change payload");
      return {
        type,
        payload: {
          approvalId: asIdentifier(
            record.approvalId,
            "approval ID",
          ) as ApprovalId,
        },
      };
    }
    case "error":
      return { type, payload: parseProtocolError(payload) };
    default:
      throw invalidPayload(`Unsupported core event type: ${type}.`);
  }
}

function parseStateSnapshot(raw: unknown): StateSnapshot {
  const snapshot = requireRecord(raw, "state snapshot");
  return {
    snapshotId: asIdentifier(
      snapshot.snapshotId,
      "snapshot ID",
    ) as StateSnapshot["snapshotId"],
    generation: asStateGeneration(snapshot.generation),
    activeWorkspaceId:
      snapshot.activeWorkspaceId === null
        ? null
        : (asIdentifier(
            snapshot.activeWorkspaceId,
            "Workspace ID",
          ) as StateSnapshot["activeWorkspaceId"]),
    activeSessionId:
      snapshot.activeSessionId === null
        ? null
        : (asIdentifier(
            snapshot.activeSessionId,
            "Session ID",
          ) as StateSnapshot["activeSessionId"]),
    redactions: parseRedactions(snapshot.redactions),
  };
}

function parseRunScope(raw: unknown): RunScope {
  const scope = requireRecord(raw, "run scope");
  return {
    workspaceId: asIdentifier(
      scope.workspaceId,
      "Workspace ID",
    ) as RunScope["workspaceId"],
    sessionId: asIdentifier(
      scope.sessionId,
      "session ID",
    ) as RunScope["sessionId"],
    turnId: asIdentifier(scope.turnId, "turn ID") as RunScope["turnId"],
    attemptId: asIdentifier(
      scope.attemptId,
      "attempt ID",
    ) as RunScope["attemptId"],
    runtimeId: asIdentifier(
      scope.runtimeId,
      "runtime ID",
    ) as RunScope["runtimeId"],
    environmentId: asIdentifier(
      scope.environmentId,
      "environment ID",
    ) as RunScope["environmentId"],
    correlationId: asCorrelationId(scope.correlationId),
    processGeneration: asPositiveGeneration(scope.processGeneration),
  };
}

function tryParseProtocolError(raw: unknown): ProtocolError | null {
  try {
    return parseProtocolError(raw);
  } catch {
    return null;
  }
}

function parseProtocolError(raw: unknown): ProtocolError {
  const error = requireRecord(raw, "structured core error");
  const code = parseProtocolErrorCode(error.code);
  const message = requireString(error.message, "error message");
  if (new TextEncoder().encode(message).length > MAX_DIAGNOSTIC_BYTES) {
    throw invalidPayload("The structured core error message is unbounded.");
  }
  const detailsRecord = requireRecord(error.details, "error details");
  if (Object.keys(detailsRecord).length > MAX_SAFE_DETAILS) {
    throw invalidPayload("The structured core error has too many details.");
  }
  const details: Record<string, SafeDetailValue> = {};
  for (const [key, value] of Object.entries(detailsRecord)) {
    asIdentifier(key, "error detail key");
    details[key] = parseSafeDetail(value);
  }
  return {
    code,
    message,
    retryable: requireBoolean(error.retryable, "retryable error flag"),
    correlationId:
      error.correlationId === null
        ? null
        : asCorrelationId(error.correlationId),
    details,
  };
}

function parseSafeDetail(raw: unknown): SafeDetailValue {
  const detail = requireRecord(raw, "safe diagnostic detail");
  switch (detail.kind) {
    case "text":
      return {
        kind: "text",
        value: requireString(detail.value, "detail text"),
      };
    case "integer":
      return {
        kind: "integer",
        value: requireSafeInteger(detail.value, "detail integer"),
      };
    case "boolean":
      return {
        kind: "boolean",
        value: requireBoolean(detail.value, "detail boolean"),
      };
    case "redacted":
      return { kind: "redacted", value: parseRedactionMarker(detail.value) };
    default:
      throw invalidPayload("The safe diagnostic detail kind is invalid.");
  }
}

function parseRedactions(raw: unknown): readonly RedactionMarker[] {
  if (!Array.isArray(raw) || raw.length > MAX_REDACTION_MARKERS) {
    throw invalidPayload("The redaction marker list is invalid or unbounded.");
  }
  return raw.map(parseRedactionMarker);
}

function parseRedactionMarker(raw: unknown): RedactionMarker {
  const marker = requireRecord(raw, "redaction marker");
  return {
    fieldPath: asIdentifier(marker.fieldPath, "redacted field path"),
    reason: parseRedactionReason(marker.reason),
  };
}

function parseRedactionReason(raw: unknown): RedactionReason {
  const reasons: readonly RedactionReason[] = [
    "credential",
    "authorization",
    "environmentValue",
    "websiteStorage",
    "sensitiveField",
    "policy",
  ];
  if (!reasons.includes(raw as RedactionReason)) {
    throw invalidPayload("The redaction reason is invalid.");
  }
  return raw as RedactionReason;
}

function parseProtocolErrorCode(raw: unknown): ProtocolErrorCode {
  const codes: readonly ProtocolErrorCode[] = [
    "unknownProtocolVersion",
    "invalidIdentifier",
    "invalidGeneration",
    "staleGeneration",
    "futureGeneration",
    "correlationMismatch",
    "invalidPayload",
    "payloadTooLarge",
    "notFound",
    "conflict",
    "unauthorized",
    "forbidden",
    "unavailable",
    "internal",
  ];
  if (!codes.includes(raw as ProtocolErrorCode)) {
    throw invalidPayload("The structured core error code is invalid.");
  }
  return raw as ProtocolErrorCode;
}

function parseRunPhase(raw: unknown): RunPhase {
  const phases: readonly RunPhase[] = [
    "queued",
    "running",
    "awaitingApproval",
    "cancelling",
    "completed",
    "failed",
    "interrupted",
  ];
  if (!phases.includes(raw as RunPhase)) {
    throw invalidPayload("The run phase is invalid.");
  }
  return raw as RunPhase;
}

function parseRuntimeHealth(raw: unknown): RuntimeHealth {
  const states: readonly RuntimeHealth[] = [
    "starting",
    "ready",
    "degraded",
    "unavailable",
    "stopped",
  ];
  if (!states.includes(raw as RuntimeHealth)) {
    throw invalidPayload("The runtime health is invalid.");
  }
  return raw as RuntimeHealth;
}

function requireProtocolVersion(raw: unknown): void {
  if (raw !== PROTOCOL_VERSION) {
    throw unknownVersion(raw);
  }
}

function unknownVersion(actual: unknown): ProtocolBoundaryError {
  return new ProtocolBoundaryError(
    "unknownProtocolVersion",
    `Unsupported protocol version: ${String(actual)}.`,
  );
}

function boundaryErrorFromCore(error: ProtocolError): ProtocolBoundaryError {
  return new ProtocolBoundaryError(
    error.code,
    error.message,
    error.retryable,
    error.details,
  );
}

function generationError(
  code: "staleGeneration" | "futureGeneration",
  actual: number,
  expected: number,
): ProtocolBoundaryError {
  return new ProtocolBoundaryError(code, "Generation check failed.", false, {
    actual: { kind: "integer", value: actual },
    expected: { kind: "integer", value: expected },
  });
}

function textDetail(value: string): SafeDetailValue {
  return { kind: "text", value };
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw invalidPayload(`The ${label} must be an object.`);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw invalidPayload(`The ${label} must be a non-empty string.`);
  }
  return value;
}

function requireBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw invalidPayload(`The ${label} must be a boolean.`);
  }
  return value;
}

function requireSafeInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value)) {
    throw invalidPayload(`The ${label} must be a safe integer.`);
  }
  return value as number;
}

function requireNonNegativeInteger(value: unknown, label: string): number {
  const integer = requireSafeInteger(value, label);
  if (integer < 0) {
    throw invalidPayload(`The ${label} must be non-negative.`);
  }
  return integer;
}

function asIdentifier(value: unknown, label: string): string {
  const identifier = requireString(value, label);
  if (
    new TextEncoder().encode(identifier).length > MAX_IDENTIFIER_BYTES ||
    !/^[A-Za-z0-9_.:@/-]+$/.test(identifier)
  ) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return identifier;
}

function asRequestId(value: unknown): RequestId {
  return asIdentifier(value, "request ID") as RequestId;
}

function asCorrelationId(value: unknown): CorrelationId {
  return asIdentifier(value, "correlation ID") as CorrelationId;
}

function asStateGeneration(value: unknown): StateGeneration {
  return requireNonNegativeInteger(
    value,
    "state generation",
  ) as StateGeneration;
}

function asPositiveGeneration(value: unknown): ProcessGeneration {
  const generation = requireNonNegativeInteger(value, "process generation");
  if (generation === 0) {
    throw invalidPayload("The process generation must be positive.");
  }
  return generation as ProcessGeneration;
}

function invalidPayload(message: string): ProtocolBoundaryError {
  return new ProtocolBoundaryError("invalidPayload", message);
}
