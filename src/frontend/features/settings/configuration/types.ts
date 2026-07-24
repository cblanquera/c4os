export type ConfigurationApprovalPreset =
  "askForApproval" | "approveSafeActions" | "approveForMe" | "custom";

export type BrowserEnvironmentScope =
  "appWide" | "workspaceProject" | "chat" | "none";

export interface ConfigurationValues {
  readonly approvalPreset: ConfigurationApprovalPreset;
  readonly browserEnvironment: BrowserEnvironmentScope;
  readonly inheritShellEnvironment: boolean;
  readonly restoreLastWorkspace: boolean;
}

export type ConfigurationOperationView =
  | { readonly message: string; readonly status: "idle" }
  | { readonly message: string; readonly status: "pending" }
  | { readonly message: string; readonly status: "success" }
  | { readonly message: string; readonly status: "error" };

export interface LiveConfigurationView {
  readonly detail: string;
  readonly displayPath: string;
  readonly source: "default" | "lastKnownGood" | "external";
}

export type ConfigurationSettingsSnapshot =
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
      readonly live: LiveConfigurationView;
      readonly openConfiguration: ConfigurationOperationView;
      readonly save: ConfigurationOperationView;
      readonly saved: ConfigurationValues;
      readonly status: "ready";
    };

export interface ConfigurationSettingsActions {
  readonly onNavigateAdvanced: () => Promise<void>;
  readonly onOpenConfigurationFile: () => Promise<void>;
  readonly onRetry: () => Promise<void>;
  readonly onSaveConfiguration: (
    values: ConfigurationValues,
    expectedGeneration: number,
  ) => Promise<void>;
}
