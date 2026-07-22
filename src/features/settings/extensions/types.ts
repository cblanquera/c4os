/** Serializable projections consumed by the controlled Plugin and Skill views. */

export type ExtensionActionResult = void | Promise<void>;

export type ExtensionOperation =
  | "installing"
  | "enabling"
  | "disabling"
  | "stagingUpdate"
  | "activatingUpdate"
  | "rollingBack"
  | "uninstalling";

export type PluginLifecycle =
  | "available"
  | "quarantined"
  | "installedDisabled"
  | "enabled"
  | "updateStaged"
  | "executing"
  | "revoked"
  | "failed"
  | "rolledBack";

export type ExtensionTrustState =
  "verified" | "quarantined" | "revoked" | "invalid";

export interface ExtensionSignatureView {
  readonly contentKeyId: string;
  readonly contentSignature: "verified" | "invalid";
  readonly originKeyId: string;
  readonly originSignature: "verified" | "invalid";
  readonly verifiedAt: string | null;
}

export interface MarketplaceView {
  readonly detail: string;
  readonly gitRef: string | null;
  readonly id: string;
  readonly label: string;
  readonly lastCheckedAt: string | null;
  readonly packageCount: number;
  readonly resolvedCommit: string | null;
  readonly source: string;
  readonly sparsePaths: readonly string[];
  readonly status: "ready" | "checking" | "unavailable" | "invalid";
  readonly trustedOrigin: string;
}

export interface PluginSettingView {
  readonly choices: readonly string[];
  readonly description: string;
  readonly id: string;
  readonly kind:
    "boolean" | "integer" | "string" | "select" | "credentialReference";
  readonly label: string;
  readonly required: boolean;
}

export interface PluginAppView {
  readonly id: string;
  readonly settingIds: readonly string[];
  readonly summary: string;
  readonly title: string;
}

export interface PluginMcpServerView {
  readonly id: string;
  readonly name: string;
  readonly settingIds: readonly string[];
  readonly transport: "stdio" | "streamableHttp";
}

export interface PluginHookView {
  readonly arguments: readonly string[];
  readonly grants: readonly string[];
  readonly id: string;
  readonly lastResult: string | null;
  readonly name: string;
  readonly review: string;
  readonly status: "notReviewed" | "ready" | "blocked" | "failed";
}

export interface PluginUpdateView {
  readonly availableVersion: string | null;
  readonly currentDigest: string;
  readonly stagedDigest: string | null;
  readonly stagedVersion: string | null;
}

export interface PluginView {
  readonly apps: readonly PluginAppView[];
  readonly capabilities: readonly string[];
  readonly compatibility: string;
  readonly failureCode: string | null;
  readonly hooks: readonly PluginHookView[];
  readonly id: string;
  readonly isInstalled: boolean;
  readonly lastKnownGoodVersion: string | null;
  readonly lifecycle: PluginLifecycle;
  readonly marketplaceId: string;
  readonly mcpServers: readonly PluginMcpServerView[];
  readonly name: string;
  readonly operation: ExtensionOperation | null;
  readonly packageKind: "plugin" | "skill";
  readonly privacyPolicyUrl: string | null;
  readonly publisher: string;
  readonly revocationReason: string | null;
  readonly rollbackAvailable: boolean;
  readonly signature: ExtensionSignatureView;
  readonly source: string;
  readonly settings: readonly PluginSettingView[];
  readonly summary: string;
  readonly termsUrl: string | null;
  readonly trustState: ExtensionTrustState;
  readonly update: PluginUpdateView;
  readonly version: string;
  readonly websiteUrl: string | null;
}

export type PluginSettingsSnapshot =
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
      readonly catalogDetail: string;
      readonly catalogStatus: "current" | "checking" | "degraded";
      readonly generation: number;
      readonly marketplaces: readonly MarketplaceView[];
      readonly plugins: readonly PluginView[];
      readonly status: "ready";
    };

export interface AddMarketplaceRequest {
  readonly gitRef: string | null;
  readonly publicKeySha256: string;
  readonly signingKeyId: string;
  readonly source: string;
  readonly sparsePaths: readonly string[];
  readonly trustedOrigin: string;
}

export interface PluginSettingsActions {
  readonly onActivateUpdate: (pluginId: string) => ExtensionActionResult;
  readonly onAddMarketplace: (
    request: AddMarketplaceRequest,
  ) => ExtensionActionResult;
  readonly onDisable: (pluginId: string) => ExtensionActionResult;
  readonly onEnableForNextTurn: (pluginId: string) => ExtensionActionResult;
  readonly onInstallDisabled: (pluginId: string) => ExtensionActionResult;
  readonly onOpenPublisherLink: (
    pluginId: string,
    link: "privacy" | "terms" | "website",
  ) => ExtensionActionResult;
  readonly onRefreshCatalog: () => ExtensionActionResult;
  readonly onRetry: () => ExtensionActionResult;
  readonly onReviewHook: (
    pluginId: string,
    hookId: string,
  ) => ExtensionActionResult;
  readonly onRollback: (pluginId: string) => ExtensionActionResult;
  readonly onStageUpdate: (pluginId: string) => ExtensionActionResult;
  readonly onUninstall: (pluginId: string) => ExtensionActionResult;
}

export type SkillSourceKind =
  "project" | "workspace" | "user" | "plugin" | "bundled";

export type SkillEffectiveState =
  "active" | "disabled" | "ineligible" | "invalid" | "shadowed";

export interface SkillCollisionView {
  readonly sourceLabel: string;
  readonly sourcePrecedence: number;
  readonly sourceQualifiedId: string;
}

export type SkillInstructionsView =
  | { readonly state: "notLoaded" }
  | { readonly state: "loading" }
  | { readonly error: string; readonly state: "error" }
  | { readonly instructions: string; readonly state: "loaded" };

export interface SkillView {
  readonly collisions: readonly SkillCollisionView[];
  readonly effectiveState: SkillEffectiveState;
  readonly eligibilityDetail: string;
  readonly enabled: boolean;
  readonly frontmatter:
    | { readonly state: "valid" }
    | { readonly diagnostic: string; readonly state: "invalid" };
  readonly id: string;
  readonly instructions: SkillInstructionsView;
  readonly isExplicitSelection: boolean;
  readonly isInstalled: boolean;
  readonly name: string;
  readonly sourceKind: SkillSourceKind;
  readonly sourceLabel: string;
  readonly sourcePrecedence: number;
  readonly sourceQualifiedId: string;
  readonly summary: string;
  readonly version: string;
}

export type SkillSettingsSnapshot =
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
      readonly discoveryDetail: string;
      readonly discoveryStatus: "current" | "discovering" | "degraded";
      readonly generation: number;
      readonly skills: readonly SkillView[];
      readonly status: "ready";
    };

export interface SkillSettingsActions {
  readonly onCustomize: (skillId: string) => ExtensionActionResult;
  readonly onLoadInstructions: (skillId: string) => ExtensionActionResult;
  readonly onRetry: () => ExtensionActionResult;
  readonly onSelectExplicitly: (skillId: string) => ExtensionActionResult;
  readonly onSetEnabled: (
    skillId: string,
    enabled: boolean,
  ) => ExtensionActionResult;
  readonly onTryInChat: (skillId: string) => ExtensionActionResult;
  readonly onUninstall: (skillId: string) => ExtensionActionResult;
}
