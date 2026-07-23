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
export type PolicyValue = "default" | PolicyDecision;
export type PolicyPreset =
  "ask-for-approval" | "approve-safe-actions" | "approve-for-me" | "custom";

export type PolicyCategoryValues = Readonly<
  Record<PolicySettingKey, PolicyDecision | null>
>;

export interface PolicySettingDefinition {
  readonly description: string;
  readonly key: PolicySettingKey;
  readonly label: string;
}

export interface PolicyGroupDefinition {
  readonly id: string;
  readonly items: readonly PolicySettingDefinition[];
  readonly label: string;
}

export interface PolicyExceptionView {
  readonly action: string;
  readonly decision: PolicyDecision;
  readonly duration: string;
  readonly exceptionId: string;
  readonly scope: string;
  readonly source: string;
}

export interface ReadyPolicySettingsSnapshot {
  readonly authority: string;
  readonly basePreset: Exclude<PolicyPreset, "custom">;
  readonly categoryValues: PolicyCategoryValues;
  readonly coordinatorGeneration: number;
  readonly effectiveCategoryValues: PolicyCategoryValues;
  readonly exceptions: readonly PolicyExceptionView[];
  readonly managedRequirementCount: number;
  readonly maximumAuthorityRuleCount: number;
  readonly operationError?: string;
  readonly policyVersion: number;
  readonly preset: PolicyPreset;
  readonly revocationEpoch: number;
  readonly status: "ready";
}

export type PolicySettingsSnapshot =
  | {
      readonly message: string;
      readonly status: "loading";
    }
  | {
      readonly message: string;
      readonly retryable: boolean;
      readonly status: "error";
    }
  | ReadyPolicySettingsSnapshot;

export interface SavePolicySettingsInput {
  readonly categoryValues: PolicyCategoryValues;
  readonly expectedCoordinatorGeneration: number;
  readonly expectedPolicyVersion: number;
}

export interface RevokePolicyExceptionInput {
  readonly exceptionId: string;
  readonly expectedCoordinatorGeneration: number;
  readonly expectedPolicyVersion: number;
}

export interface PolicySettingsActions {
  readonly onRetry?: () => Promise<void>;
  readonly onRevokeException: (
    input: RevokePolicyExceptionInput,
  ) => Promise<ReadyPolicySettingsSnapshot>;
  readonly onSave: (
    input: SavePolicySettingsInput,
  ) => Promise<ReadyPolicySettingsSnapshot>;
}
