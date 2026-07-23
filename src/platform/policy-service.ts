import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type RequestId,
  type SnapshotRequest,
  type StateGeneration,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const POLICY_SETTING_KEYS = [
  "workspace.read",
  "workspace.modify",
  "workspace.delete",
  "workspace.outside",
  "command.inspect",
  "command.workspace",
  "command.system",
  "process.control",
  "git.local.read",
  "git.local.change",
  "git.remote.read",
  "git.remote.publish",
  "network.retrieve",
  "network.publish",
  "network.listen",
  "network.upload",
  "browser.view",
  "browser.interact",
  "browser.authenticated",
  "desktop.control",
  "credential.use",
  "credential.add",
  "credential.reveal",
  "extension.read",
  "extension.use",
  "extension.configure",
  "c4os.policy",
  "artifact.export",
] as const;

export type PolicySettingKey = (typeof POLICY_SETTING_KEYS)[number];
export type PolicyDecision = "allow" | "ask" | "deny";
export type RuntimeApprovalPreset =
  "ask-for-approval" | "approve-safe-actions" | "approve-for-me" | "custom";

export type PolicyException = {
  readonly exceptionId: string;
  readonly decision: PolicyDecision;
  readonly action: string;
  readonly scope: string;
  readonly source: string;
  readonly duration: string;
};

export type PolicySettingsSnapshot = {
  readonly authority: "rust-policy-service";
  readonly generation: number;
  readonly coordinatorGeneration: number;
  readonly policyVersion: number;
  readonly revocationEpoch: number;
  readonly preset: RuntimeApprovalPreset;
  readonly basePreset: Exclude<RuntimeApprovalPreset, "custom">;
  readonly categoryValues: Readonly<
    Record<PolicySettingKey, PolicyDecision | null>
  >;
  readonly effectiveCategoryValues: Readonly<
    Record<PolicySettingKey, PolicyDecision | null>
  >;
  readonly exceptions: readonly PolicyException[];
  readonly maximumAuthorityRuleCount: number;
  readonly managedRequirementCount: number;
};

type PolicyCommand =
  "policy_snapshot" | "policy_save" | "policy_revoke_exception";

export interface PolicyTransport {
  invoke(
    command: PolicyCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface PolicyAdapter {
  readSnapshot(): Promise<PolicySettingsSnapshot>;
  save(
    categoryValues: Readonly<Record<PolicySettingKey, PolicyDecision | null>>,
  ): Promise<PolicySettingsSnapshot>;
  revokeException(exceptionId: string): Promise<PolicySettingsSnapshot>;
}

const nativeAdapter = createPolicyAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readPolicySettings = () => nativeAdapter.readSnapshot();
export const savePolicySettings = (
  categoryValues: Readonly<Record<PolicySettingKey, PolicyDecision | null>>,
) => nativeAdapter.save(categoryValues);
export const revokePolicyException = (exceptionId: string) =>
  nativeAdapter.revokeException(exceptionId);

export function createPolicyAdapter(
  transport: PolicyTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): PolicyAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let policyVersion = 0;
  let coordinatorGeneration = 0;

  async function invoke(
    command: PolicyCommand,
    input?: Readonly<Record<string, unknown>>,
  ): Promise<PolicySettingsSnapshot> {
    const request: SnapshotRequest = {
      protocolVersion: PROTOCOL_VERSION,
      requestId: requestIdFactory(),
      correlationId: correlationIdFactory(),
      expectedGeneration: policyVersion as StateGeneration,
    };
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        input === undefined ? { request } : { request, input },
      );
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = parseEnvelope(raw, request);
    const payload = parseSnapshot(envelope.payload);
    if (
      payload.policyVersion !== envelope.generation ||
      payload.policyVersion < policyVersion ||
      payload.coordinatorGeneration < coordinatorGeneration
    ) {
      throw invalidPayload("The Policy response generation regressed.");
    }
    policyVersion = payload.policyVersion;
    coordinatorGeneration = payload.coordinatorGeneration;
    return payload;
  }

  return {
    readSnapshot: () => invoke("policy_snapshot"),
    save(categoryValues) {
      return invoke("policy_save", {
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedPolicyVersion: policyVersion,
        categoryValues: parseCategoryValues(categoryValues),
      });
    },
    revokeException(exceptionId) {
      return invoke("policy_revoke_exception", {
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedPolicyVersion: policyVersion,
        exceptionId: asIdentifier(exceptionId, "Policy exception ID"),
      });
    },
  };
}

function parseEnvelope(
  raw: unknown,
  request: SnapshotRequest,
): { readonly generation: number; readonly payload: unknown } {
  const value = asObject(raw, "Policy response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Policy protocol version is invalid.");
  }
  if (
    asIdentifier(value.requestId, "response request ID") !==
      request.requestId ||
    asIdentifier(value.correlationId, "response correlation ID") !==
      request.correlationId
  ) {
    throw boundary(
      "correlationMismatch",
      "The Policy response identity did not match its request.",
    );
  }
  const generation = asGeneration(value.generation, "response generation");
  if (generation < request.expectedGeneration) {
    throw invalidPayload("The Policy response generation regressed.");
  }
  return { generation, payload: value.payload };
}

function parseSnapshot(raw: unknown): PolicySettingsSnapshot {
  const value = asObject(raw, "Policy snapshot");
  if (value.authority !== "rust-policy-service") {
    throw invalidPayload("The Policy authority is invalid.");
  }
  const categoryValues = parseCategoryValues(value.categoryValues);
  const effectiveCategoryValues = parseCategoryValues(
    value.effectiveCategoryValues,
  );
  const policyVersion = asGeneration(value.policyVersion, "policy version");
  return {
    authority: "rust-policy-service",
    generation: policyVersion,
    coordinatorGeneration: asGeneration(
      value.coordinatorGeneration,
      "coordinator generation",
    ),
    policyVersion,
    revocationEpoch: asGeneration(value.revocationEpoch, "revocation epoch"),
    preset: asEnum(
      value.preset,
      [
        "ask-for-approval",
        "approve-safe-actions",
        "approve-for-me",
        "custom",
      ] as const,
      "approval preset",
    ),
    basePreset: asEnum(
      value.basePreset,
      ["ask-for-approval", "approve-safe-actions", "approve-for-me"] as const,
      "base approval preset",
    ),
    categoryValues,
    effectiveCategoryValues,
    exceptions: asArray(value.exceptions, "Policy exceptions", 512).map(
      (rawException) => {
        const exception = asObject(rawException, "Policy exception");
        return {
          exceptionId: asIdentifier(exception.exceptionId, "exception ID"),
          decision: asEnum(
            exception.decision,
            ["allow", "ask", "deny"] as const,
            "exception decision",
          ),
          action: asText(exception.action, "exception action", 512),
          scope: asText(exception.scope, "exception scope", 1024),
          source: asText(exception.source, "exception source", 512),
          duration: asText(exception.duration, "exception duration", 512),
        };
      },
    ),
    maximumAuthorityRuleCount: asGeneration(
      value.maximumAuthorityRuleCount,
      "maximum-authority rule count",
    ),
    managedRequirementCount: asGeneration(
      value.managedRequirementCount,
      "managed requirement count",
    ),
  };
}

function parseCategoryValues(
  raw: unknown,
): Record<PolicySettingKey, PolicyDecision | null> {
  const value = asObject(raw, "Policy category values");
  const keys = Object.keys(value);
  if (
    keys.length !== POLICY_SETTING_KEYS.length ||
    POLICY_SETTING_KEYS.some((key) => !Object.hasOwn(value, key))
  ) {
    throw invalidPayload("The Policy category schema is invalid.");
  }
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [
      key,
      value[key] === null
        ? null
        : asEnum(
            value[key],
            ["allow", "ask", "deny"] as const,
            `${key} decision`,
          ),
    ]),
  ) as Record<PolicySettingKey, PolicyDecision | null>;
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

function asGeneration(raw: unknown, label: string): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < 0) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as number;
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

function normalizeFailure(error: unknown): ProtocolBoundaryError {
  if (error instanceof ProtocolBoundaryError) return error;
  if (typeof error === "object" && error !== null) {
    const value = error as {
      code?: unknown;
      message?: unknown;
      retryable?: unknown;
    };
    if (typeof value.code === "string" && typeof value.message === "string") {
      return boundary(value.code, value.message, value.retryable === true);
    }
  }
  return boundary(
    "unavailable",
    "The native Policy service is unavailable.",
    true,
  );
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
