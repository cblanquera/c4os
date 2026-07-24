export type SettingsActionResult = void | Promise<void>;

export type ModelCapabilityKey = "vision" | "tools" | "reasoning" | "audio";

export type ModelCapabilityState =
  "supported" | "degraded" | "unsupported" | "unknown";

export interface ModelCapabilityEvidenceView {
  readonly checkedAt: string;
  readonly detail: string;
  readonly source: string;
}

export interface ModelCapabilityView {
  readonly evidence: readonly ModelCapabilityEvidenceView[];
  readonly key: ModelCapabilityKey;
  readonly state: ModelCapabilityState;
  readonly summary: string;
}

export interface ModelRouteView {
  readonly available: boolean;
  readonly availabilityDetail: string;
  readonly capabilities: readonly ModelCapabilityView[];
  readonly contextTokens: number | null;
  readonly enabled: boolean;
  readonly id: string;
  readonly modelId: string;
  readonly modelName: string;
  readonly operationPending?: boolean;
  readonly providerId: string;
  readonly providerName: string;
  readonly revision: string;
  readonly runtimeLabel: string;
}

export type ModelRefreshView =
  | { readonly message: string; readonly status: "idle" }
  | { readonly message: string; readonly status: "pending" }
  | { readonly message: string; readonly status: "success" }
  | { readonly message: string; readonly status: "error" };

export type ModelSettingsSnapshot =
  | {
      readonly message: string;
      readonly status: "loading";
    }
  | {
      readonly message: string;
      readonly retryable: boolean;
      readonly status: "error";
    }
  | {
      readonly generation: number;
      readonly models: readonly ModelRouteView[];
      readonly operationError?: string;
      readonly refresh: ModelRefreshView;
      readonly status: "ready";
      readonly writesDisabled: boolean;
    };

export interface ModelSettingsActions {
  readonly onRefresh: () => SettingsActionResult;
  readonly onRetry: () => SettingsActionResult;
  readonly onSetEnabled: (
    modelRouteId: string,
    enabled: boolean,
  ) => SettingsActionResult;
  readonly onSetVisibleEnabled: (
    modelRouteIds: readonly string[],
    enabled: boolean,
  ) => SettingsActionResult;
}

export type RuntimeHealth = "healthy" | "degraded" | "unavailable";

export interface RuntimeOptionView {
  readonly detail: string;
  readonly health: RuntimeHealth;
  readonly id: string;
  readonly kind: "open-code" | "pi";
  readonly label: "OpenCode" | "Pi";
  readonly version: string;
}

export type RuntimeSaveView =
  | { readonly message: string; readonly status: "idle" }
  | { readonly message: string; readonly status: "pending" }
  | { readonly message: string; readonly status: "success" }
  | { readonly message: string; readonly status: "error" };

export type RuntimeSettingsSnapshot =
  | {
      readonly message: string;
      readonly status: "loading";
    }
  | {
      readonly message: string;
      readonly retryable: boolean;
      readonly status: "error";
    }
  | {
      readonly generation: number;
      readonly runtimes: readonly RuntimeOptionView[];
      readonly save: RuntimeSaveView;
      readonly savedRuntimeId: string | null;
      readonly status: "ready";
    };

export interface RuntimeSettingsActions {
  readonly onRetry: () => SettingsActionResult;
  readonly onSaveRuntime: (
    runtimeId: string,
    expectedGeneration: number,
  ) => SettingsActionResult;
}
