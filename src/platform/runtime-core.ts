import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type ApprovalId,
  type CorrelationId,
  type ProcessGeneration,
  type RequestId,
  type RuntimeId,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const RUNTIME_CORE_SNAPSHOT_COMMAND = "runtime_core_snapshot" as const;
export const RUNTIME_PRODUCTION_ACTIVATE_COMMAND =
  "runtime_production_activate" as const;
export const RUNTIME_PRODUCTION_SHUTDOWN_COMMAND =
  "runtime_production_shutdown" as const;
export const RUNTIME_PRODUCTION_PUMP_COMMAND =
  "runtime_production_pump" as const;
export const RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND =
  "runtime_production_answer_approval" as const;

const MAX_PROCESS_ID = 4_294_967_295;

export type RuntimeProviderSummary = {
  readonly providerId: string;
  readonly displayName: string;
  readonly enabled: boolean;
  readonly testStatus: Readonly<Record<string, unknown>>;
  readonly modelCount: number;
  readonly selectedModelId: string | null;
};

export type RuntimeProcessSummary = {
  readonly runtimeId: string;
  readonly runtimeKind: "open-code" | "pi";
  readonly nativeVersion: string;
  readonly lifecycle:
    | "stopped"
    | "starting"
    | "ready"
    | "degraded"
    | "incompatible"
    | "stopping"
    | "failed";
  readonly health: "unknown" | "healthy" | "degraded" | "unhealthy";
  readonly processGeneration: number;
};

export type RuntimeCoreSnapshot = {
  readonly authority: "rust-core";
  readonly generation: StateGeneration;
  readonly providerGeneration: number;
  readonly capabilityGeneration: number;
  readonly runtimeGeneration: number;
  readonly onboardingReady: boolean;
  readonly providers: readonly RuntimeProviderSummary[];
  readonly runtimes: readonly RuntimeProcessSummary[];
  readonly pendingApprovals: readonly ProductionRuntimePendingApproval[];
};

export type ProductionRuntimePendingApproval = {
  readonly runtimeId: RuntimeId;
  readonly correlationId: CorrelationId;
  readonly promptId: ApprovalId;
};

export type ActivatedProductionRuntime = {
  readonly runtimeId: RuntimeId;
  readonly coordinatorGeneration: StateGeneration;
  readonly capabilityGeneration: number;
  readonly authorityGeneration: number;
  readonly processGeneration: ProcessGeneration;
  readonly processId: number;
};

export type ProductionRuntimeShutdown = {
  readonly runtimeId: RuntimeId;
  readonly coordinatorGeneration: StateGeneration;
};

export type ProductionRuntimePump = {
  readonly runtimeId: RuntimeId;
  readonly pumpedEvents: number;
  readonly coordinatorGeneration: StateGeneration;
};

export type RuntimeProductionApprovalAnswer = "allow" | "deny";

export type ProductionRuntimeApprovalRequest = {
  readonly runtimeId: RuntimeId;
  readonly correlationId: CorrelationId;
  readonly promptId: ApprovalId;
  readonly answer: RuntimeProductionApprovalAnswer;
};

export type ProductionRuntimeApprovalSettlement = {
  readonly runtimeId: RuntimeId;
  readonly correlationId: CorrelationId;
  readonly promptId: ApprovalId;
};

export type RuntimeCoreCommand =
  | typeof RUNTIME_CORE_SNAPSHOT_COMMAND
  | typeof RUNTIME_PRODUCTION_ACTIVATE_COMMAND
  | typeof RUNTIME_PRODUCTION_SHUTDOWN_COMMAND
  | typeof RUNTIME_PRODUCTION_PUMP_COMMAND
  | typeof RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND;

export type RuntimeCoreTransportArguments = {
  readonly [RUNTIME_CORE_SNAPSHOT_COMMAND]: {
    readonly request: SnapshotRequest;
  };
  readonly [RUNTIME_PRODUCTION_ACTIVATE_COMMAND]: {
    readonly request: SnapshotRequest;
    readonly runtimeId: RuntimeId;
  };
  readonly [RUNTIME_PRODUCTION_SHUTDOWN_COMMAND]: {
    readonly request: SnapshotRequest;
    readonly runtimeId: RuntimeId;
  };
  readonly [RUNTIME_PRODUCTION_PUMP_COMMAND]: {
    readonly request: SnapshotRequest;
    readonly runtimeId: RuntimeId;
  };
  readonly [RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND]: {
    readonly request: SnapshotRequest;
    readonly runtimeId: RuntimeId;
    readonly correlationId: CorrelationId;
    readonly promptId: ApprovalId;
    readonly answer: RuntimeProductionApprovalAnswer;
  };
};

export interface RuntimeCoreTransport {
  invoke<Command extends RuntimeCoreCommand>(
    command: Command,
    args: RuntimeCoreTransportArguments[Command],
  ): Promise<unknown>;
}

export interface RuntimeCoreAdapter {
  readonly currentGeneration: StateGeneration;
  readSnapshot(): Promise<RuntimeCoreSnapshot>;
  activateRuntime(runtimeId: RuntimeId): Promise<ActivatedProductionRuntime>;
  shutdownRuntime(runtimeId: RuntimeId): Promise<ProductionRuntimeShutdown>;
  pumpRuntime(runtimeId: RuntimeId): Promise<ProductionRuntimePump>;
  answerApproval(
    approval: ProductionRuntimeApprovalRequest,
  ): Promise<ProductionRuntimeApprovalSettlement>;
}

const nativeAdapter = createRuntimeCoreAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

/** Reads the bounded Rust-owned runtime projection through an allowlisted command. */
export function readRuntimeCoreSnapshot(): Promise<RuntimeCoreSnapshot> {
  return nativeAdapter.readSnapshot();
}

/** Activates one Rust-owned production runtime under the shared CAS cursor. */
export function activateProductionRuntime(
  runtimeId: RuntimeId,
): Promise<ActivatedProductionRuntime> {
  return nativeAdapter.activateRuntime(runtimeId);
}

/** Shuts down one Rust-owned production runtime under the shared CAS cursor. */
export function shutdownProductionRuntime(
  runtimeId: RuntimeId,
): Promise<ProductionRuntimeShutdown> {
  return nativeAdapter.shutdownRuntime(runtimeId);
}

/** Pumps one Rust-owned production runtime under the shared CAS cursor. */
export function pumpProductionRuntime(
  runtimeId: RuntimeId,
): Promise<ProductionRuntimePump> {
  return nativeAdapter.pumpRuntime(runtimeId);
}

/** Answers one Rust-owned runtime approval under the shared CAS cursor. */
export function answerProductionRuntimeApproval(
  approval: ProductionRuntimeApprovalRequest,
): Promise<ProductionRuntimeApprovalSettlement> {
  return nativeAdapter.answerApproval(approval);
}

/**
 * Uses the native core in Tauri and a command-shaped deterministic transport
 * only in an explicitly enabled browser QA build. Both paths cross the same
 * strict envelope parser, so rendered acceptance cannot bypass correlation,
 * generation, authority, or payload validation.
 */
export function readRuntimeReviewSnapshot(): Promise<RuntimeCoreSnapshot> {
  if (
    import.meta.env.VITE_C4OS_QA_FIXTURES === "1" &&
    !("__TAURI_INTERNALS__" in globalThis)
  ) {
    return createRuntimeCoreAdapter({
      async invoke(command, { request }) {
        if (command !== RUNTIME_CORE_SNAPSHOT_COMMAND) {
          throw boundary(
            "unavailable",
            "The QA transport rejected an unknown command.",
          );
        }
        return {
          protocolVersion: PROTOCOL_VERSION,
          requestId: request.requestId,
          correlationId: request.correlationId,
          generation: 26,
          payload: {
            authority: "rust-core",
            providerGeneration: 12,
            capabilityGeneration: 15,
            runtimeGeneration: 18,
            onboardingReady: true,
            providers: [],
            runtimes: [
              {
                runtimeId: "opencode-primary",
                runtimeKind: "open-code",
                nativeVersion: "1.18.3",
                lifecycle: "ready",
                health: "healthy",
                processGeneration: 17,
              },
              {
                runtimeId: "pi-primary",
                runtimeKind: "pi",
                nativeVersion: "0.80.10",
                lifecycle: "degraded",
                health: "degraded",
                processGeneration: 9,
              },
            ],
            pendingApprovals: [
              {
                runtimeId: "opencode-primary",
                correlationId: "correlation-approval-review",
                promptId: "approval:runtime-review",
              },
            ],
          },
        };
      },
    }).readSnapshot();
  }
  return readRuntimeCoreSnapshot();
}

export function createRuntimeCoreAdapter(
  transport: RuntimeCoreTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
    readonly initialGeneration?: StateGeneration;
  } = {},
): RuntimeCoreAdapter {
  let currentGeneration = options.initialGeneration ?? (0 as StateGeneration);
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request ID") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation ID") as CorrelationId);

  async function invokeCommand<Command extends RuntimeCoreCommand, Result>(
    command: Command,
    createArguments: (
      request: SnapshotRequest,
    ) => RuntimeCoreTransportArguments[Command],
    parsePayload: (payload: unknown, generation: StateGeneration) => Result,
  ): Promise<Result> {
    const requestId = identifier(requestIdFactory(), "request ID") as RequestId;
    const correlationId = identifier(
      correlationIdFactory(),
      "correlation ID",
    ) as CorrelationId;
    const request: SnapshotRequest = {
      protocolVersion: PROTOCOL_VERSION,
      requestId,
      correlationId,
      expectedGeneration: currentGeneration,
    };
    const raw = await transport.invoke(command, createArguments(request));
    const envelope = parseEnvelope(raw);
    if (
      envelope.requestId !== requestId ||
      envelope.correlationId !== correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "The runtime response identity did not match its request.",
      );
    }
    if (envelope.generation < currentGeneration) {
      throw boundary("staleGeneration", "The runtime response was stale.");
    }
    const payload = parsePayload(envelope.payload, envelope.generation);
    currentGeneration = envelope.generation;
    return payload;
  }

  return {
    get currentGeneration() {
      return currentGeneration;
    },
    async readSnapshot(): Promise<RuntimeCoreSnapshot> {
      return invokeCommand(
        RUNTIME_CORE_SNAPSHOT_COMMAND,
        (request) => ({ request }),
        parseSnapshot,
      );
    },
    async activateRuntime(runtimeId): Promise<ActivatedProductionRuntime> {
      const expectedRuntimeId = identifier(
        runtimeId,
        "runtime ID",
      ) as RuntimeId;
      return invokeCommand(
        RUNTIME_PRODUCTION_ACTIVATE_COMMAND,
        (request) => ({ request, runtimeId: expectedRuntimeId }),
        (payload, generation) =>
          parseActivatedRuntime(payload, generation, expectedRuntimeId),
      );
    },
    async shutdownRuntime(runtimeId): Promise<ProductionRuntimeShutdown> {
      const expectedRuntimeId = identifier(
        runtimeId,
        "runtime ID",
      ) as RuntimeId;
      return invokeCommand(
        RUNTIME_PRODUCTION_SHUTDOWN_COMMAND,
        (request) => ({ request, runtimeId: expectedRuntimeId }),
        (payload, generation) =>
          parseRuntimeShutdown(payload, generation, expectedRuntimeId),
      );
    },
    async pumpRuntime(runtimeId): Promise<ProductionRuntimePump> {
      const expectedRuntimeId = identifier(
        runtimeId,
        "runtime ID",
      ) as RuntimeId;
      return invokeCommand(
        RUNTIME_PRODUCTION_PUMP_COMMAND,
        (request) => ({ request, runtimeId: expectedRuntimeId }),
        (payload, generation) =>
          parseRuntimePump(payload, generation, expectedRuntimeId),
      );
    },
    async answerApproval(
      approval,
    ): Promise<ProductionRuntimeApprovalSettlement> {
      const expectedRuntimeId = identifier(
        approval.runtimeId,
        "runtime ID",
      ) as RuntimeId;
      const expectedCorrelationId = identifier(
        approval.correlationId,
        "approval correlation ID",
      ) as CorrelationId;
      const expectedPromptId = identifier(
        approval.promptId,
        "approval prompt ID",
      ) as ApprovalId;
      const answer = enumValue(approval.answer, ["allow", "deny"]);
      return invokeCommand(
        RUNTIME_PRODUCTION_ANSWER_APPROVAL_COMMAND,
        (request) => ({
          request,
          runtimeId: expectedRuntimeId,
          correlationId: expectedCorrelationId,
          promptId: expectedPromptId,
          answer,
        }),
        (payload) =>
          parseApprovalSettlement(payload, {
            runtimeId: expectedRuntimeId,
            correlationId: expectedCorrelationId,
            promptId: expectedPromptId,
          }),
      );
    },
  };
}

function parseEnvelope(raw: unknown) {
  const envelope = exactRecord(raw, "runtime envelope", [
    "protocolVersion",
    "requestId",
    "correlationId",
    "generation",
    "payload",
  ]);
  if (envelope.protocolVersion !== PROTOCOL_VERSION) {
    throw boundary(
      "unknownProtocolVersion",
      "Unsupported runtime protocol version.",
    );
  }
  return {
    requestId: identifier(envelope.requestId, "response request ID"),
    correlationId: identifier(
      envelope.correlationId,
      "response correlation ID",
    ),
    generation: generationValue(envelope.generation, "runtime generation"),
    payload: envelope.payload,
  };
}

function parseSnapshot(
  raw: unknown,
  generation: StateGeneration,
): RuntimeCoreSnapshot {
  const value = exactRecord(raw, "runtime snapshot", [
    "authority",
    "providerGeneration",
    "capabilityGeneration",
    "runtimeGeneration",
    "onboardingReady",
    "providers",
    "runtimes",
    "pendingApprovals",
  ]);
  if (value.authority !== "rust-core") {
    throw boundary("invalidPayload", "The runtime authority is invalid.");
  }
  if (!Array.isArray(value.providers) || value.providers.length > 128) {
    throw boundary("invalidPayload", "The provider projection is invalid.");
  }
  if (!Array.isArray(value.runtimes) || value.runtimes.length > 128) {
    throw boundary("invalidPayload", "The runtime projection is invalid.");
  }
  if (
    !Array.isArray(value.pendingApprovals) ||
    value.pendingApprovals.length > 4_096
  ) {
    throw boundary("invalidPayload", "The approval projection is invalid.");
  }
  return {
    authority: "rust-core",
    generation,
    providerGeneration: generationValue(
      value.providerGeneration,
      "provider generation",
    ),
    capabilityGeneration: generationValue(
      value.capabilityGeneration,
      "capability evidence generation",
    ),
    runtimeGeneration: generationValue(
      value.runtimeGeneration,
      "supervisor generation",
    ),
    onboardingReady: booleanValue(
      value.onboardingReady,
      "onboarding readiness",
    ),
    providers: value.providers.map(parseProvider),
    runtimes: value.runtimes.map(parseRuntime),
    pendingApprovals: value.pendingApprovals.map(parsePendingApproval),
  };
}

function parsePendingApproval(raw: unknown): ProductionRuntimePendingApproval {
  const value = exactRecord(raw, "pending runtime approval", [
    "runtimeId",
    "correlationId",
    "promptId",
  ]);
  return {
    runtimeId: identifier(value.runtimeId, "runtime ID") as RuntimeId,
    correlationId: identifier(
      value.correlationId,
      "approval correlation ID",
    ) as CorrelationId,
    promptId: identifier(value.promptId, "approval prompt ID") as ApprovalId,
  };
}

function parseProvider(raw: unknown): RuntimeProviderSummary {
  const value = exactRecord(raw, "provider summary", [
    "providerId",
    "displayName",
    "enabled",
    "testStatus",
    "modelCount",
    "selectedModelId",
  ]);
  return {
    providerId: identifier(value.providerId, "provider ID"),
    displayName: textValue(value.displayName, "provider display name", 512),
    enabled: booleanValue(value.enabled, "provider enabled state"),
    testStatus: record(value.testStatus, "provider test status"),
    modelCount: boundedInteger(value.modelCount, "model count", 512),
    selectedModelId:
      value.selectedModelId === null
        ? null
        : identifier(value.selectedModelId, "selected model ID"),
  };
}

function parseRuntime(raw: unknown): RuntimeProcessSummary {
  const value = exactRecord(raw, "runtime summary", [
    "runtimeId",
    "runtimeKind",
    "nativeVersion",
    "lifecycle",
    "health",
    "processGeneration",
  ]);
  return {
    runtimeId: identifier(value.runtimeId, "runtime ID"),
    runtimeKind: enumValue(value.runtimeKind, ["open-code", "pi"]),
    nativeVersion: textValue(value.nativeVersion, "native version", 64),
    lifecycle: enumValue(value.lifecycle, [
      "stopped",
      "starting",
      "ready",
      "degraded",
      "incompatible",
      "stopping",
      "failed",
    ]),
    health: enumValue(value.health, [
      "unknown",
      "healthy",
      "degraded",
      "unhealthy",
    ]),
    processGeneration: processGenerationValue(
      value.processGeneration,
      "process generation",
    ),
  };
}

function parseActivatedRuntime(
  raw: unknown,
  generation: StateGeneration,
  expectedRuntimeId: RuntimeId,
): ActivatedProductionRuntime {
  const value = exactRecord(raw, "runtime activation", [
    "coordinatorGeneration",
    "capabilityGeneration",
    "authorityGeneration",
    "runtimeId",
    "processGeneration",
    "processId",
  ]);
  const runtimeId = matchingIdentifier(
    value.runtimeId,
    expectedRuntimeId,
    "runtime ID",
  ) as RuntimeId;
  const coordinatorGeneration = matchingGeneration(
    value.coordinatorGeneration,
    generation,
    "coordinator generation",
  );
  return {
    runtimeId,
    coordinatorGeneration,
    capabilityGeneration: generationValue(
      value.capabilityGeneration,
      "capability generation",
    ),
    authorityGeneration: generationValue(
      value.authorityGeneration,
      "authority generation",
    ),
    processGeneration: processGenerationValue(
      value.processGeneration,
      "process generation",
    ),
    processId: boundedInteger(value.processId, "process ID", MAX_PROCESS_ID, 1),
  };
}

function parseRuntimeShutdown(
  raw: unknown,
  generation: StateGeneration,
  expectedRuntimeId: RuntimeId,
): ProductionRuntimeShutdown {
  const value = exactRecord(raw, "runtime shutdown", [
    "runtimeId",
    "coordinatorGeneration",
  ]);
  return {
    runtimeId: matchingIdentifier(
      value.runtimeId,
      expectedRuntimeId,
      "runtime ID",
    ) as RuntimeId,
    coordinatorGeneration: matchingGeneration(
      value.coordinatorGeneration,
      generation,
      "coordinator generation",
    ),
  };
}

function parseRuntimePump(
  raw: unknown,
  generation: StateGeneration,
  expectedRuntimeId: RuntimeId,
): ProductionRuntimePump {
  const value = exactRecord(raw, "runtime pump", [
    "runtimeId",
    "pumpedEvents",
    "coordinatorGeneration",
  ]);
  return {
    runtimeId: matchingIdentifier(
      value.runtimeId,
      expectedRuntimeId,
      "runtime ID",
    ) as RuntimeId,
    pumpedEvents: boundedInteger(
      value.pumpedEvents,
      "pumped event count",
      Number.MAX_SAFE_INTEGER,
    ),
    coordinatorGeneration: matchingGeneration(
      value.coordinatorGeneration,
      generation,
      "coordinator generation",
    ),
  };
}

function parseApprovalSettlement(
  raw: unknown,
  expected: Pick<
    ProductionRuntimeApprovalRequest,
    "runtimeId" | "correlationId" | "promptId"
  >,
): ProductionRuntimeApprovalSettlement {
  const value = exactRecord(raw, "runtime approval settlement", [
    "runtimeId",
    "correlationId",
    "promptId",
  ]);
  return {
    runtimeId: matchingIdentifier(
      value.runtimeId,
      expected.runtimeId,
      "runtime ID",
    ) as RuntimeId,
    correlationId: matchingIdentifier(
      value.correlationId,
      expected.correlationId,
      "approval correlation ID",
    ) as CorrelationId,
    promptId: matchingIdentifier(
      value.promptId,
      expected.promptId,
      "approval prompt ID",
    ) as ApprovalId,
  };
}

function secureUuid(label: string): string {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw boundary("unavailable", `Secure ${label} generation is unavailable.`);
  }
  return globalThis.crypto.randomUUID();
}

function record(value: unknown, label: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw boundary("invalidPayload", `The ${label} is invalid.`);
  }
  return value as Record<string, unknown>;
}

function exactRecord(
  value: unknown,
  label: string,
  keys: readonly string[],
): Record<string, unknown> {
  const candidate = record(value, label);
  const actualKeys = Object.keys(candidate);
  if (
    actualKeys.length !== keys.length ||
    actualKeys.some((key) => !keys.includes(key))
  ) {
    throw boundary("invalidPayload", `The ${label} shape is invalid.`);
  }
  return candidate;
}

function generationValue(value: unknown, label: string): StateGeneration {
  return boundedInteger(
    value,
    label,
    Number.MAX_SAFE_INTEGER,
  ) as StateGeneration;
}

function processGenerationValue(
  value: unknown,
  label: string,
): ProcessGeneration {
  return boundedInteger(
    value,
    label,
    Number.MAX_SAFE_INTEGER,
  ) as ProcessGeneration;
}

function boundedInteger(
  value: unknown,
  label: string,
  maximum: number,
  minimum = 0,
): number {
  if (
    !Number.isSafeInteger(value) ||
    (value as number) < minimum ||
    (value as number) > maximum
  ) {
    throw boundary("invalidPayload", `The ${label} is invalid.`);
  }
  return value as number;
}

function matchingGeneration(
  value: unknown,
  expected: StateGeneration,
  label: string,
): StateGeneration {
  const candidate = generationValue(value, label);
  if (candidate !== expected) {
    throw boundary(
      "staleGeneration",
      `The ${label} did not match its envelope.`,
    );
  }
  return candidate;
}

function identifier(value: unknown, label: string): string {
  const candidate = textValue(value, label, MAX_IDENTIFIER_BYTES);
  if (!/^[A-Za-z0-9_.:@-]+$/u.test(candidate)) {
    throw boundary("invalidPayload", `The ${label} is invalid.`);
  }
  return candidate;
}

function matchingIdentifier(
  value: unknown,
  expected: string,
  label: string,
): string {
  const candidate = identifier(value, label);
  if (candidate !== expected) {
    throw boundary(
      "correlationMismatch",
      `The ${label} did not match its request.`,
    );
  }
  return candidate;
}

function textValue(
  value: unknown,
  label: string,
  maximumBytes: number,
): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    new TextEncoder().encode(value).length > maximumBytes
  ) {
    throw boundary("invalidPayload", `The ${label} is invalid.`);
  }
  return value;
}

function booleanValue(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw boundary("invalidPayload", `The ${label} is invalid.`);
  }
  return value;
}

function enumValue<const T extends readonly string[]>(
  value: unknown,
  values: T,
): T[number] {
  if (typeof value !== "string" || !values.includes(value)) {
    throw boundary("invalidPayload", "An enum value is invalid.");
  }
  return value as T[number];
}

function boundary(
  code:
    | "invalidPayload"
    | "unknownProtocolVersion"
    | "correlationMismatch"
    | "staleGeneration"
    | "unavailable",
  message: string,
) {
  return new ProtocolBoundaryError(code, message);
}
