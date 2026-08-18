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
import {
  isAcceptedQaAwareAuthority,
  type QaAwareAuthority,
} from "../qa/authority";

export type ProviderKind =
  | "open-ai"
  | "anthropic"
  | "gemini"
  | "open-router"
  | "hugging-face"
  | "custom";

export type ConfigurableProviderKind = Exclude<
  ProviderKind,
  "anthropic" | "gemini"
>;

export type ProviderAuthentication =
  | { readonly type: "bearer" }
  | { readonly type: "apiKeyHeader"; readonly headerName: string }
  | { readonly type: "none" };

export type ProviderEndpoint = {
  readonly endpointId: string;
  readonly baseUrl: string;
  readonly apiKind: "openai" | "openai-compatible";
};

export type ProviderTestStatus =
  | { readonly state: "untested" }
  | { readonly state: "succeeded"; readonly checkedAtMs: number }
  | {
      readonly state: "succeededNoUsableModels";
      readonly checkedAtMs: number;
    }
  | {
      readonly state: "failed";
      readonly checkedAtMs: number;
      readonly code:
        | "authentication"
        | "network"
        | "rate-limited"
        | "incompatible"
        | "cancelled"
        | "internal";
    };

export type ProviderCredentialProtection = "installation-key" | "session-only";

export type ProviderPendingApproval = {
  readonly promptId: string;
  readonly operation:
    | "save-profile"
    | "test-connection"
    | "complete-onboarding"
    | "delete-profile";
  readonly providerId: string;
  readonly providerName: string;
  readonly expiresAtMs: number;
};

export type ProviderModel = {
  readonly modelId: string;
  readonly displayName: string;
  readonly recommendationRank: number;
  readonly availability: "available" | "degraded" | "unavailable" | "unknown";
  readonly checkedAtMs: number;
  readonly lifecycle: "active" | "preview" | "deprecated" | "unavailable";
  readonly features: Readonly<Record<string, string>>;
  readonly contextTokens: number | null;
  readonly outputTokens: number | null;
  readonly productionReady: boolean;
};

export type ProviderRecord = {
  readonly providerId: string;
  readonly kind: ProviderKind;
  readonly displayName: string;
  readonly endpoint: ProviderEndpoint;
  readonly authentication: ProviderAuthentication;
  readonly headers: Readonly<Record<string, string>>;
  readonly hasCredential: boolean;
  readonly enabled: boolean;
  readonly testStatus: ProviderTestStatus;
  readonly models: readonly ProviderModel[];
  readonly disabledModelIds: readonly string[];
  readonly selectedModelId: string | null;
  readonly generation: number;
};

export type ProviderSettingsSnapshot = {
  readonly authority: QaAwareAuthority<"rust-provider-service">;
  readonly coordinatorGeneration: number;
  readonly configurationGeneration: number;
  readonly credentialProtection: ProviderCredentialProtection;
  readonly credentialFallbackRequired: boolean;
  readonly generation: number;
  readonly onboardingCompleted: boolean;
  readonly providers: readonly ProviderRecord[];
  readonly modelRoute: string | null;
  readonly defaultRuntime: string | null;
  readonly defaultEnvironment: string | null;
  readonly pendingApproval: ProviderPendingApproval | null;
  readonly transientTest: {
    readonly testToken: string;
    readonly provider: ProviderRecord;
  } | null;
};

export type ProviderProfileDraft = {
  readonly providerId: string;
  readonly kind: ConfigurableProviderKind;
  readonly displayName: string;
  readonly endpoint: ProviderEndpoint;
  readonly authentication: ProviderAuthentication;
  readonly headers: Readonly<Record<string, string>>;
  readonly secret?: string;
  readonly enabled: boolean;
};

export type ProviderCommand =
  | "provider_snapshot"
  | "provider_accept_session_credentials"
  | "provider_retry_secure_storage"
  | "provider_save_profile"
  | "provider_test_connection"
  | "provider_refresh_connection"
  | "provider_answer_approval"
  | "provider_select_model"
  | "provider_set_models_enabled"
  | "provider_complete_onboarding"
  | "provider_delete_profile";

export interface ProviderTransport {
  invoke(
    command: ProviderCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface ProviderAdapter {
  readSnapshot(): Promise<ProviderSettingsSnapshot>;
  acceptSessionCredentials(): Promise<ProviderSettingsSnapshot>;
  retrySecureStorage(): Promise<ProviderSettingsSnapshot>;
  saveProfile(draft: ProviderProfileDraft): Promise<ProviderSettingsSnapshot>;
  testConnection(
    draft: ProviderProfileDraft,
  ): Promise<ProviderSettingsSnapshot>;
  refreshConnection(providerId: string): Promise<ProviderSettingsSnapshot>;
  answerApproval(
    promptId: string,
    answer: "allow" | "deny",
  ): Promise<ProviderSettingsSnapshot>;
  selectModel(
    providerId: string,
    modelId: string,
  ): Promise<ProviderSettingsSnapshot>;
  setModelsEnabled(
    providerId: string,
    modelIds: readonly string[],
    enabled: boolean,
  ): Promise<ProviderSettingsSnapshot>;
  completeOnboarding(
    testToken: string,
    modelId: string,
  ): Promise<ProviderSettingsSnapshot>;
  deleteProfile(providerId: string): Promise<ProviderSettingsSnapshot>;
}

const nativeAdapter = createProviderAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readProviderSnapshot = () => nativeAdapter.readSnapshot();
export const acceptProviderSessionCredentials = () =>
  nativeAdapter.acceptSessionCredentials();
export const retryProviderSecureStorage = () =>
  nativeAdapter.retrySecureStorage();
export const saveProviderProfile = (draft: ProviderProfileDraft) =>
  nativeAdapter.saveProfile(draft);
export const testProviderConnection = (draft: ProviderProfileDraft) =>
  nativeAdapter.testConnection(draft);
export const refreshProviderConnection = (providerId: string) =>
  nativeAdapter.refreshConnection(providerId);
export const answerProviderApproval = (
  promptId: string,
  answer: "allow" | "deny",
) => nativeAdapter.answerApproval(promptId, answer);
export const selectProviderModel = (providerId: string, modelId: string) =>
  nativeAdapter.selectModel(providerId, modelId);
export const setProviderModelsEnabled = (
  providerId: string,
  modelIds: readonly string[],
  enabled: boolean,
) => nativeAdapter.setModelsEnabled(providerId, modelIds, enabled);
export const completeProviderOnboarding = (
  testToken: string,
  modelId: string,
) => nativeAdapter.completeOnboarding(testToken, modelId);
export const deleteProviderProfile = (providerId: string) =>
  nativeAdapter.deleteProfile(providerId);

export function createProviderAdapter(
  transport: ProviderTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
  } = {},
): ProviderAdapter {
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation") as CorrelationId);
  let generation = 0;
  let coordinatorGeneration = 0;
  let configurationGeneration = 0;

  const invoke = async (
    command: ProviderCommand,
    input?: Readonly<Record<string, unknown>>,
    retryStalePreEffectOperation = true,
  ): Promise<ProviderSettingsSnapshot> => {
    const request: SnapshotRequest = {
      protocolVersion: PROTOCOL_VERSION,
      requestId: requestIdFactory(),
      correlationId: correlationIdFactory(),
      expectedGeneration: generation as StateGeneration,
    };
    let raw: unknown;
    try {
      raw = await transport.invoke(
        command,
        input === undefined ? { request } : { request, input },
      );
    } catch (error) {
      const failure = normalizeFailure(error);
      if (
        command !== "provider_snapshot" &&
        failure.code === "staleGeneration" &&
        failure.retryable
      ) {
        // Runtime composition can legitimately advance the coordinator after
        // a Provider surface reads its snapshot. Refresh the private CAS
        // cursors, and retry only operations whose stale-generation rejection
        // happens before any durable or network effect. Other operations keep
        // an explicit user retry so an effect is never replayed implicitly.
        try {
          await invoke("provider_snapshot", undefined, false);
          if (
            retryStalePreEffectOperation &&
            (command === "provider_save_profile" ||
              command === "provider_test_connection")
          ) {
            return invoke(
              command,
              {
                ...input,
                expectedCoordinatorGeneration: coordinatorGeneration,
                expectedProviderGeneration: generation,
              },
              false,
            );
          }
        } catch {
          // Preserve the original operation failure. A later explicit surface
          // refresh still owns recovery when the snapshot is unavailable.
        }
      }
      throw failure;
    }
    const envelope = parseEnvelope(raw);
    if (
      envelope.requestId !== request.requestId ||
      envelope.correlationId !== request.correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "The Provider response identity did not match its request.",
      );
    }
    if (
      envelope.generation < generation ||
      envelope.payload.generation !== envelope.generation ||
      envelope.payload.coordinatorGeneration < coordinatorGeneration
    ) {
      throw invalidPayload("The Provider response generation regressed.");
    }
    generation = envelope.generation;
    coordinatorGeneration = envelope.payload.coordinatorGeneration;
    configurationGeneration = envelope.payload.configurationGeneration;
    return envelope.payload;
  };

  const identity = (providerId: string) => ({
    expectedCoordinatorGeneration: coordinatorGeneration,
    expectedProviderGeneration: generation,
    providerId: asIdentifier(providerId, "provider ID"),
  });

  return {
    readSnapshot: () => invoke("provider_snapshot"),
    acceptSessionCredentials: () =>
      invoke("provider_accept_session_credentials"),
    retrySecureStorage: () => invoke("provider_retry_secure_storage"),
    saveProfile(draft) {
      return invoke("provider_save_profile", {
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedProviderGeneration: generation,
        providerId: asIdentifier(draft.providerId, "provider ID"),
        kind: asEnum(
          draft.kind,
          ["open-ai", "open-router", "hugging-face", "custom"] as const,
          "provider kind",
        ),
        displayName: asText(draft.displayName, "display name", 256),
        endpoint: validateEndpoint(draft.endpoint),
        authentication: validateAuthentication(draft.authentication),
        headers: validateHeaders(draft.headers),
        secret: draft.secret,
        enabled: draft.enabled,
      });
    },
    testConnection: (draft) =>
      invoke("provider_test_connection", {
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedProviderGeneration: generation,
        providerId: asIdentifier(draft.providerId, "provider ID"),
        kind: asEnum(
          draft.kind,
          ["open-ai", "open-router", "hugging-face", "custom"] as const,
          "provider kind",
        ),
        displayName: asText(draft.displayName, "display name", 256),
        endpoint: validateEndpoint(draft.endpoint),
        authentication: validateAuthentication(draft.authentication),
        headers: validateHeaders(draft.headers),
        secret: draft.secret,
        enabled: draft.enabled,
      }),
    refreshConnection: (providerId) =>
      invoke("provider_refresh_connection", identity(providerId)),
    answerApproval: (promptId, answer) =>
      invoke("provider_answer_approval", {
        promptId: asIdentifier(promptId, "Provider approval ID"),
        answer: asEnum(answer, ["allow", "deny"] as const, "approval answer"),
      }),
    selectModel: (providerId, modelId) =>
      invoke("provider_select_model", {
        ...identity(providerId),
        modelId: asModelIdentifier(modelId),
      }),
    setModelsEnabled: (providerId, modelIds, enabled) =>
      invoke("provider_set_models_enabled", {
        ...identity(providerId),
        modelIds: modelIds.map(asModelIdentifier),
        enabled,
      }),
    completeOnboarding: (testToken, modelId) =>
      invoke("provider_complete_onboarding", {
        expectedCoordinatorGeneration: coordinatorGeneration,
        expectedProviderGeneration: generation,
        expectedConfigurationGeneration: configurationGeneration,
        testToken: asIdentifier(testToken, "Provider test token"),
        modelId: asModelIdentifier(modelId),
        runtimeId: "opencode",
        environmentId: "local",
      }),
    deleteProfile: (providerId) =>
      invoke("provider_delete_profile", identity(providerId)),
  };
}

function parseEnvelope(raw: unknown): {
  readonly requestId: string;
  readonly correlationId: string;
  readonly generation: number;
  readonly payload: ProviderSettingsSnapshot;
} {
  const value = asObject(raw, "Provider response");
  if (value.protocolVersion !== PROTOCOL_VERSION) {
    throw invalidPayload("The Provider protocol version is invalid.");
  }
  return {
    requestId: asIdentifier(value.requestId, "response request ID"),
    correlationId: asIdentifier(value.correlationId, "response correlation ID"),
    generation: asGeneration(value.generation, "response generation"),
    payload: parseSnapshot(value.payload),
  };
}

function parseSnapshot(raw: unknown): ProviderSettingsSnapshot {
  const value = asObject(raw, "Provider snapshot");
  if (!isAcceptedQaAwareAuthority(value.authority, "rust-provider-service")) {
    throw invalidPayload("The Provider authority is invalid.");
  }
  const providerState = asObject(value.providers, "provider state");
  const generation = asGeneration(
    providerState.generation,
    "provider generation",
  );
  const records = asArray(providerState.providers, "providers").map(
    parseRecord,
  );
  const completedAt = optionalGeneration(
    providerState.onboardingCompletedAtMs,
    "onboarding completion",
  );
  const onboardingCompleted = asBoolean(
    value.onboardingCompleted,
    "onboarding completion state",
  );
  if (onboardingCompleted !== (completedAt !== null && records.length > 0)) {
    throw invalidPayload("The Provider onboarding state is inconsistent.");
  }
  return {
    authority: value.authority,
    coordinatorGeneration: asGeneration(
      value.coordinatorGeneration,
      "coordinator generation",
    ),
    configurationGeneration: asGeneration(
      value.configurationGeneration,
      "configuration generation",
    ),
    credentialProtection: asEnum(
      value.credentialProtection,
      ["installation-key", "session-only"] as const,
      "credential protection",
    ),
    credentialFallbackRequired: asBoolean(
      value.credentialFallbackRequired,
      "credential fallback requirement",
    ),
    generation,
    onboardingCompleted,
    providers: records,
    modelRoute: optionalText(value.modelRoute, "model route", 512),
    defaultRuntime: optionalText(value.defaultRuntime, "default runtime", 128),
    defaultEnvironment: optionalText(
      value.defaultEnvironment,
      "default environment",
      128,
    ),
    pendingApproval: parsePendingApproval(value.pendingApproval),
    transientTest: parseTransientTest(value.transientTest),
  };
}

function parseTransientTest(
  raw: unknown,
): ProviderSettingsSnapshot["transientTest"] {
  if (raw === null) return null;
  const value = asObject(raw, "Provider transient test");
  return {
    testToken: asIdentifier(value.testToken, "Provider test token"),
    provider: parseRecord(value.provider),
  };
}

function parsePendingApproval(raw: unknown): ProviderPendingApproval | null {
  if (raw === null) return null;
  const value = asObject(raw, "Provider pending approval");
  return {
    promptId: asIdentifier(value.promptId, "Provider approval ID"),
    operation: asEnum(
      value.operation,
      [
        "save-profile",
        "test-connection",
        "complete-onboarding",
        "delete-profile",
      ] as const,
      "Provider approval operation",
    ),
    providerId: asIdentifier(value.providerId, "Provider approval provider ID"),
    providerName: asText(
      value.providerName,
      "Provider approval provider name",
      256,
    ),
    expiresAtMs: positiveInteger(value.expiresAtMs, "Provider approval expiry"),
  };
}

function parseRecord(raw: unknown): ProviderRecord {
  const value = asObject(raw, "provider record");
  const profile = asObject(value.profile, "provider profile");
  const models = asObject(value.models, "provider models");
  const selectedModelId = optionalText(
    value.selectedModelId,
    "selected model ID",
    160,
  );
  const parsedModels = Object.entries(models).map(([key, model]) => {
    const parsed = parseModel(model);
    if (parsed.modelId !== key) {
      throw invalidPayload("A Provider model key did not match its identity.");
    }
    return parsed;
  });
  const disabledModelIds = asArray(
    value.disabledModelIds ?? [],
    "disabled model IDs",
  ).map(asModelIdentifier);
  if (disabledModelIds.some((modelId) => !models[modelId])) {
    throw invalidPayload("A disabled Provider model is unavailable.");
  }
  if (
    selectedModelId !== null &&
    !parsedModels.some((model) => model.modelId === selectedModelId)
  ) {
    throw invalidPayload("The selected Provider model is unavailable.");
  }
  return {
    providerId: asIdentifier(profile.providerId, "provider ID"),
    kind: asEnum(
      profile.kind,
      [
        "open-ai",
        "anthropic",
        "gemini",
        "open-router",
        "hugging-face",
        "custom",
      ] as const,
      "provider kind",
    ),
    displayName: asText(profile.displayName, "provider display name", 256),
    endpoint: validateEndpoint(profile.endpoint),
    authentication: validateAuthentication(profile.authentication),
    headers: validateHeaders(asObject(profile.headers, "provider headers")),
    hasCredential: profile.credentialReference !== null,
    enabled: asBoolean(profile.enabled, "provider enabled state"),
    testStatus: parseTestStatus(value.testStatus),
    models: parsedModels.sort(
      (left, right) =>
        left.recommendationRank - right.recommendationRank ||
        left.displayName.localeCompare(right.displayName),
    ),
    disabledModelIds,
    selectedModelId,
    generation: asGeneration(value.generation, "provider record generation"),
  };
}

function parseTestStatus(raw: unknown): ProviderTestStatus {
  const value = asObject(raw, "provider test status");
  const state = asEnum(
    value.state,
    ["untested", "succeeded", "succeededNoUsableModels", "failed"] as const,
    "provider test state",
  );
  if (state === "untested") return { state };
  const checkedAtMs = positiveInteger(value.checkedAtMs, "test timestamp");
  if (state !== "failed") return { state, checkedAtMs };
  return {
    state,
    checkedAtMs,
    code: asEnum(
      value.code,
      [
        "authentication",
        "network",
        "rate-limited",
        "incompatible",
        "cancelled",
        "internal",
      ] as const,
      "provider failure code",
    ),
  };
}

function parseModel(raw: unknown): ProviderModel {
  const value = asObject(raw, "provider model");
  const capabilities = asObject(value.capabilities, "model capabilities");
  const features = asObject(capabilities.features, "model features");
  const numeric = asObject(capabilities.numericLimits, "model numeric limits");
  const declaration = value.providerDeclaration;
  return {
    modelId: asModelIdentifier(value.modelId),
    displayName: asText(value.displayName, "model display name", 256),
    recommendationRank: asGeneration(
      value.recommendationRank,
      "recommendation rank",
    ),
    availability: asEnum(
      value.availability,
      ["available", "degraded", "unavailable", "unknown"] as const,
      "model availability",
    ),
    checkedAtMs: positiveInteger(value.checkedAtMs, "model timestamp"),
    lifecycle: asEnum(
      capabilities.lifecycle,
      ["active", "preview", "deprecated", "unavailable"] as const,
      "model lifecycle",
    ),
    features: Object.fromEntries(
      Object.entries(features).map(([key, evidence]) => [
        key,
        asText(
          asObject(evidence, "feature evidence").state,
          "feature state",
          32,
        ),
      ]),
    ),
    contextTokens: numericMaximum(
      numeric["context-tokens"] ?? numeric.contextTokens,
    ),
    outputTokens: numericMaximum(
      numeric["output-tokens"] ?? numeric.outputTokens,
    ),
    productionReady:
      declaration !== null &&
      (value.availability === "available" ||
        value.availability === "degraded") &&
      capabilities.lifecycle !== "unavailable",
  };
}

function numericMaximum(raw: unknown): number | null {
  if (raw === undefined) return null;
  return optionalGeneration(
    asObject(raw, "numeric capability").maximum,
    "numeric maximum",
  );
}

function validateEndpoint(raw: unknown): ProviderEndpoint {
  const value = asObject(raw, "provider endpoint");
  return {
    endpointId: asIdentifier(value.endpointId, "endpoint ID"),
    baseUrl: asText(value.baseUrl, "provider URL", 2_048),
    apiKind: asEnum(
      value.apiKind,
      ["openai", "openai-compatible"] as const,
      "provider API kind",
    ),
  };
}

function validateAuthentication(raw: unknown): ProviderAuthentication {
  const value = asObject(raw, "provider authentication");
  const type = asEnum(
    value.type,
    ["bearer", "apiKeyHeader", "none"] as const,
    "authentication type",
  );
  if (type === "apiKeyHeader") {
    return {
      type,
      headerName: asHeaderName(value.headerName),
    };
  }
  return { type };
}

function validateHeaders(
  raw: Readonly<Record<string, unknown>>,
): Record<string, string> {
  if (Object.keys(raw).length > 32) {
    throw invalidPayload("Provider headers exceed their bound.");
  }
  return Object.fromEntries(
    Object.entries(raw).map(([name, value]) => [
      asHeaderName(name),
      asText(value, "header value", 2_048),
    ]),
  );
}

function asHeaderName(raw: unknown): string {
  const value = asText(raw, "header name", 128);
  if (!/^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/.test(value)) {
    throw invalidPayload("A Provider header name is invalid.");
  }
  return value;
}

function asModelIdentifier(raw: unknown): string {
  const value = asText(raw, "model ID", 160);
  if (!/^[0-9A-Za-z@_.:/-]+$/.test(value)) {
    throw invalidPayload("The model ID is invalid.");
  }
  return value;
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
  if (!Array.isArray(raw)) throw invalidPayload(`${label} must be an array.`);
  return raw;
}

function asBoolean(raw: unknown, label: string): boolean {
  if (typeof raw !== "boolean") {
    throw invalidPayload(`${label} must be a boolean.`);
  }
  return raw;
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

function asIdentifier(raw: unknown, label: string): string {
  const value = asText(raw, label, MAX_IDENTIFIER_BYTES);
  if (!/^[0-9A-Za-z@_.:-]+$/.test(value)) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return value;
}

function asGeneration(raw: unknown, label: string): number {
  if (!Number.isSafeInteger(raw) || (raw as number) < 0) {
    throw invalidPayload(`${label} is invalid.`);
  }
  return raw as number;
}

function positiveInteger(raw: unknown, label: string): number {
  const value = asGeneration(raw, label);
  if (value === 0) throw invalidPayload(`${label} is invalid.`);
  return value;
}

function optionalGeneration(raw: unknown, label: string): number | null {
  return raw === null || raw === undefined ? null : asGeneration(raw, label);
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
    "The native Provider service is unavailable.",
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
