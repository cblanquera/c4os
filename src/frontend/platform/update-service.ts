import type { UpdateCandidateSnapshot as GeneratedUpdateCandidateSnapshot } from "../generated/UpdateCandidateSnapshot";
import type { UpdateChannel as GeneratedUpdateChannel } from "../generated/UpdateChannel";
import type { UpdateComponentSnapshot as GeneratedUpdateComponentSnapshot } from "../generated/UpdateComponentSnapshot";
import type { UpdateCoordinatorSnapshot as GeneratedUpdateCoordinatorSnapshot } from "../generated/UpdateCoordinatorSnapshot";
import type { UpdateLifecycleState as GeneratedUpdateLifecycleState } from "../generated/UpdateLifecycleState";
import type { UpdatePendingOperation as GeneratedUpdatePendingOperation } from "../generated/UpdatePendingOperation";
import type { UpdateRecoveryAction as GeneratedUpdateRecoveryAction } from "../generated/UpdateRecoveryAction";
import type { UpdateRecoveryNotice as GeneratedUpdateRecoveryNotice } from "../generated/UpdateRecoveryNotice";
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

export type UpdateChannel = "application" | "runtime" | "plugin";
export type UpdateLifecycleState =
  | "current"
  | "staged"
  | "activating"
  | "activated"
  | "rolled_back"
  | "revoked"
  | "failed";
export type UpdateRecoveryAction =
  | "retry_stage"
  | "activate_staged"
  | "rollback_last_known_good"
  | "recover_interrupted"
  | "remove_revocation"
  | "review_plugin_update"
  | "rebuild_application"
  | "wait_for_active_runs"
  | "rebuild_runtime"
  | "rebuild_last_known_good"
  | "review_runtime_crash_loop";

export interface UpdateIdentity {
  readonly channel: UpdateChannel;
  readonly componentId: string;
}

export interface UpdateComponentSnapshot extends UpdateIdentity {
  readonly state: UpdateLifecycleState;
  readonly currentVersion: string;
  readonly candidateVersion: string | null;
  readonly lastKnownGoodVersion: string | null;
  readonly stagedArtifactSha256: string | null;
  readonly revoked: boolean;
  readonly recoveryAction: UpdateRecoveryAction | null;
  readonly failureCode: string | null;
  readonly updatedAtMs: number;
}

export interface PendingUpdateOperation extends UpdateIdentity {
  readonly operationId: string;
  readonly correlationId: string;
  readonly fromVersion: string;
  readonly toVersion: string;
  readonly state: "activating";
  readonly startedAtMs: number;
}

export interface UpdateRecoveryNotice extends UpdateIdentity {
  readonly recoveryId: string;
  readonly correlationId: string;
  readonly summary: string;
  readonly action: UpdateRecoveryAction;
  readonly createdAtMs: number;
}

export interface UpdateDiscoveredCandidate extends UpdateIdentity {
  readonly candidateId: string;
  readonly version: string;
  readonly artifactSha256: string;
  readonly compatibilitySha256: string;
  readonly discoveredAtMs: number;
}

export interface UpdateCoordinatorSnapshot {
  readonly schemaVersion: 1;
  readonly generation: number;
  readonly channels: readonly UpdateComponentSnapshot[];
  readonly candidates: readonly UpdateDiscoveredCandidate[];
  readonly pendingOperations: readonly PendingUpdateOperation[];
  readonly recoveryNotices: readonly UpdateRecoveryNotice[];
}

export interface LocalUpdateStageInput extends UpdateIdentity {
  readonly candidateId: string;
}

export interface UpdateRecoveryInput extends UpdateIdentity {
  readonly recoveryId: string;
}

type Assert<Condition extends true> = Condition;
type KeysMatch<Left, Right> = keyof Left extends keyof Right
  ? keyof Right extends keyof Left
    ? true
    : false
  : false;
type UpdateCoordinatorKeysMatch = Assert<
  KeysMatch<UpdateCoordinatorSnapshot, GeneratedUpdateCoordinatorSnapshot>
>;
type UpdateComponentKeysMatch = Assert<
  KeysMatch<UpdateComponentSnapshot, GeneratedUpdateComponentSnapshot>
>;
type UpdateCandidateKeysMatch = Assert<
  KeysMatch<UpdateDiscoveredCandidate, GeneratedUpdateCandidateSnapshot>
>;
type UpdatePendingKeysMatch = Assert<
  KeysMatch<PendingUpdateOperation, GeneratedUpdatePendingOperation>
>;
type UpdateRecoveryNoticeKeysMatch = Assert<
  KeysMatch<UpdateRecoveryNotice, GeneratedUpdateRecoveryNotice>
>;
type UpdateChannelMatches = Assert<
  UpdateChannel extends GeneratedUpdateChannel
    ? GeneratedUpdateChannel extends UpdateChannel
      ? true
      : false
    : false
>;
type UpdateLifecycleMatches = Assert<
  UpdateLifecycleState extends GeneratedUpdateLifecycleState
    ? GeneratedUpdateLifecycleState extends UpdateLifecycleState
      ? true
      : false
    : false
>;
type UpdateRecoveryActionMatches = Assert<
  UpdateRecoveryAction extends GeneratedUpdateRecoveryAction
    ? GeneratedUpdateRecoveryAction extends UpdateRecoveryAction
      ? true
      : false
    : false
>;

export type UpdateSchemaCompatibilityChecks =
  | UpdateCoordinatorKeysMatch
  | UpdateComponentKeysMatch
  | UpdateCandidateKeysMatch
  | UpdatePendingKeysMatch
  | UpdateRecoveryNoticeKeysMatch
  | UpdateChannelMatches
  | UpdateLifecycleMatches
  | UpdateRecoveryActionMatches;

type UpdateCommand =
  | "update_snapshot"
  | "update_stage_local"
  | "update_activate"
  | "update_rollback"
  | "update_revoke"
  | "update_recover";

export interface UpdateTransport {
  invoke(
    command: UpdateCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface UpdateAdapter {
  readSnapshot(): Promise<UpdateCoordinatorSnapshot>;
  stageLocal(input: LocalUpdateStageInput): Promise<UpdateCoordinatorSnapshot>;
  activate(input: UpdateIdentity): Promise<UpdateCoordinatorSnapshot>;
  rollback(input: UpdateIdentity): Promise<UpdateCoordinatorSnapshot>;
  revoke(input: UpdateIdentity): Promise<UpdateCoordinatorSnapshot>;
  recover(input: UpdateRecoveryInput): Promise<UpdateCoordinatorSnapshot>;
}

const nativeAdapter = createUpdateAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readUpdateSnapshot = () => nativeAdapter.readSnapshot();
export const stageLocalUpdate = (input: LocalUpdateStageInput) =>
  nativeAdapter.stageLocal(input);
export const activateUpdate = (input: UpdateIdentity) =>
  nativeAdapter.activate(input);
export const rollbackUpdate = (input: UpdateIdentity) =>
  nativeAdapter.rollback(input);
export const revokeUpdate = (input: UpdateIdentity) =>
  nativeAdapter.revoke(input);
export const recoverUpdate = (input: UpdateRecoveryInput) =>
  nativeAdapter.recover(input);

export function createUpdateAdapter(
  transport: UpdateTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): UpdateAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;
  let currentSnapshot: UpdateCoordinatorSnapshot | null = null;

  const request = (): SnapshotRequest => ({
    protocolVersion: PROTOCOL_VERSION,
    requestId: requestIdFactory(),
    correlationId: correlationIdFactory(),
    expectedGeneration: generation as StateGeneration,
  });

  const invokeSnapshot = async (
    command: UpdateCommand,
    input?: Readonly<Record<string, unknown>>,
  ): Promise<UpdateCoordinatorSnapshot> => {
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
                expectedGeneration: identity.expectedGeneration,
                ...input,
              },
            },
      );
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = parseEnvelope(raw, identity);
    const payload = parseSnapshot(envelope.payload);
    if (payload.generation !== envelope.generation) {
      throw invalidPayload("The Update generations do not match.");
    }
    if (envelope.generation < generation) {
      throw invalidPayload(
        "The Update response generation regressed behind the live cursor.",
      );
    }
    generation = envelope.generation;
    currentSnapshot = payload;
    return payload;
  };

  const identityInput = (input: UpdateIdentity) => ({
    channel: asChannel(input.channel),
    componentId: asIdentifier(input.componentId, "component ID"),
  });

  return {
    readSnapshot: () => invokeSnapshot("update_snapshot"),
    stageLocal(input) {
      const candidate = currentSnapshot?.candidates.find(
        (entry) => entry.candidateId === input.candidateId,
      );
      const component = currentSnapshot?.channels.find(
        (entry) =>
          entry.channel === input.channel &&
          entry.componentId === input.componentId,
      );
      if (
        candidate === undefined ||
        candidate.channel !== input.channel ||
        candidate.componentId !== input.componentId
      ) {
        throw invalidPayload(
          "The update candidate is not present in the latest native snapshot.",
        );
      }
      if (
        component === undefined ||
        component.revoked ||
        component.state === "staged" ||
        component.state === "activating" ||
        component.state === "revoked" ||
        (component.recoveryAction === null
          ? component.state === "failed"
          : component.recoveryAction !== "retry_stage")
      ) {
        throw invalidPayload(
          "The update candidate cannot be staged from the latest native component state.",
        );
      }
      return invokeSnapshot("update_stage_local", {
        ...identityInput(input),
        candidateId: asIdentifier(input.candidateId, "update candidate ID"),
      });
    },
    activate(input) {
      requireGenericActivation(currentSnapshot, input);
      return invokeSnapshot("update_activate", identityInput(input));
    },
    rollback(input) {
      requireComponentState(currentSnapshot, input, [
        "failed",
        "activated",
        "revoked",
      ]);
      return invokeSnapshot("update_rollback", identityInput(input));
    },
    revoke(input) {
      requireComponentState(currentSnapshot, input, ["staged"]);
      return invokeSnapshot("update_revoke", {
        ...identityInput(input),
        reasonCode: "user_requested",
      });
    },
    recover(input) {
      const notice = currentSnapshot?.recoveryNotices.find(
        (entry) => entry.recoveryId === input.recoveryId,
      );
      if (
        notice === undefined ||
        notice.channel !== input.channel ||
        notice.componentId !== input.componentId
      ) {
        throw invalidPayload(
          "The recovery ID is not present in the latest native snapshot.",
        );
      }
      return invokeSnapshot("update_recover", {
        ...identityInput(input),
        recoveryId: asIdentifier(input.recoveryId, "update recovery ID"),
      });
    },
  };
}

function requireComponentState(
  snapshot: UpdateCoordinatorSnapshot | null,
  identity: UpdateIdentity,
  allowed: readonly UpdateLifecycleState[],
): void {
  const component = snapshot?.channels.find(
    (entry) =>
      entry.channel === identity.channel &&
      entry.componentId === identity.componentId,
  );
  if (
    component === undefined ||
    (component.revoked && !allowed.includes("revoked")) ||
    !allowed.includes(component.state)
  ) {
    throw invalidPayload(
      "The update action is not valid for the latest native component state.",
    );
  }
}

function requireGenericActivation(
  snapshot: UpdateCoordinatorSnapshot | null,
  identity: UpdateIdentity,
): void {
  const component = snapshot?.channels.find(
    (entry) =>
      entry.channel === identity.channel &&
      entry.componentId === identity.componentId,
  );
  if (
    component === undefined ||
    component.revoked ||
    component.state !== "staged" ||
    (component.recoveryAction !== null &&
      component.recoveryAction !== "activate_staged")
  ) {
    throw invalidPayload(
      "The update activation is not valid for the latest native component state.",
    );
  }
}

function parseEnvelope(
  raw: unknown,
  request: SnapshotRequest,
): { readonly generation: number; readonly payload: unknown } {
  const value = asObject(raw, "Update response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Update protocol version is invalid.");
  }
  if (
    asIdentifier(value.requestId, "response request ID") !==
      request.requestId ||
    asIdentifier(value.correlationId, "response correlation ID") !==
      request.correlationId
  ) {
    throw boundary(
      "correlationMismatch",
      "The Update response identity did not match its request.",
    );
  }
  const responseGeneration = asGeneration(
    value.generation,
    "response generation",
  );
  if (responseGeneration < request.expectedGeneration) {
    throw invalidPayload("The Update response generation regressed.");
  }
  return { generation: responseGeneration, payload: value.payload };
}

function parseSnapshot(raw: unknown): UpdateCoordinatorSnapshot {
  const value = asObject(raw, "Update snapshot");
  if (value.schemaVersion !== 1) {
    throw invalidPayload("The Update schema version is unsupported.");
  }
  return {
    schemaVersion: 1,
    generation: asGeneration(value.generation, "update generation"),
    channels: asArray(value.channels, "update channels", 128).map(
      parseComponent,
    ),
    candidates: asArray(
      value.candidates,
      "discovered update candidates",
      128,
    ).map(parseCandidate),
    pendingOperations: asArray(
      value.pendingOperations,
      "pending update operations",
      128,
    ).map(parsePendingOperation),
    recoveryNotices: asArray(
      value.recoveryNotices,
      "update recovery notices",
      128,
    ).map(parseRecoveryNotice),
  };
}

function parseCandidate(raw: unknown): UpdateDiscoveredCandidate {
  const value = asObject(raw, "discovered update candidate");
  return {
    ...parseIdentity(value),
    candidateId: asIdentifier(value.candidateId, "update candidate ID"),
    version: asVersion(value.version, "candidate version"),
    artifactSha256: asSha256(value.artifactSha256, "artifact digest"),
    compatibilitySha256: asSha256(
      value.compatibilitySha256,
      "compatibility digest",
    ),
    discoveredAtMs: asTimestamp(
      value.discoveredAtMs,
      "candidate discovery timestamp",
    ),
  };
}

function parseComponent(raw: unknown): UpdateComponentSnapshot {
  const value = asObject(raw, "update component");
  return {
    ...parseIdentity(value),
    state: asEnum(
      value.state,
      [
        "current",
        "staged",
        "activating",
        "activated",
        "rolled_back",
        "revoked",
        "failed",
      ] as const,
      "update state",
    ),
    currentVersion: asVersion(value.currentVersion, "current version"),
    candidateVersion: optionalVersion(
      value.candidateVersion,
      "candidate version",
    ),
    lastKnownGoodVersion: optionalVersion(
      value.lastKnownGoodVersion,
      "last-known-good version",
    ),
    stagedArtifactSha256: optionalSha256(
      value.stagedArtifactSha256,
      "staged artifact digest",
    ),
    revoked: asBoolean(value.revoked, "revocation state"),
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
      "recovery action",
    ),
    failureCode: optionalIdentifier(value.failureCode, "failure code"),
    updatedAtMs: asTimestamp(value.updatedAtMs, "update timestamp"),
  };
}

function parsePendingOperation(raw: unknown): PendingUpdateOperation {
  const value = asObject(raw, "pending update operation");
  if (value.state !== "activating") {
    throw invalidPayload("The pending update operation state is invalid.");
  }
  return {
    ...parseIdentity(value),
    operationId: asIdentifier(value.operationId, "update operation ID"),
    correlationId: asIdentifier(value.correlationId, "correlation ID"),
    fromVersion: asVersion(value.fromVersion, "source version"),
    toVersion: asVersion(value.toVersion, "target version"),
    state: "activating",
    startedAtMs: asTimestamp(value.startedAtMs, "operation timestamp"),
  };
}

function parseRecoveryNotice(raw: unknown): UpdateRecoveryNotice {
  const value = asObject(raw, "update recovery notice");
  return {
    ...parseIdentity(value),
    recoveryId: asIdentifier(value.recoveryId, "update recovery ID"),
    correlationId: asIdentifier(value.correlationId, "correlation ID"),
    summary: asText(
      value.summary,
      "update recovery summary",
      MAX_DIAGNOSTIC_BYTES,
    ),
    action: asEnum(
      value.action,
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
      "update recovery action",
    ),
    createdAtMs: asTimestamp(value.createdAtMs, "recovery timestamp"),
  };
}

function parseIdentity(
  value: Readonly<Record<string, unknown>>,
): UpdateIdentity {
  return {
    channel: asChannel(value.channel),
    componentId: asIdentifier(value.componentId, "component ID"),
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

function optionalIdentifier(raw: unknown, label: string): string | null {
  return raw === null || raw === undefined ? null : asIdentifier(raw, label);
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

function asVersion(raw: unknown, label: string): string {
  return asText(raw, label, 256);
}

function optionalVersion(raw: unknown, label: string): string | null {
  return raw === null || raw === undefined ? null : asVersion(raw, label);
}

function asChannel(raw: unknown): UpdateChannel {
  if (raw === "application" || raw === "runtime" || raw === "plugin") {
    return raw;
  }
  throw invalidPayload("The update channel is invalid.");
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

function optionalSha256(raw: unknown, label: string): string | null {
  return raw === null || raw === undefined ? null : asSha256(raw, label);
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
    "The native Update service is unavailable.",
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
