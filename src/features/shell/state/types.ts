import type {
  ApprovalId,
  ArtifactId,
  AttachmentId,
  ProjectId,
  RuntimeId,
  SessionId,
  StateGeneration,
  WorkspaceId,
} from "../../../platform/protocol";
import type { AppRoutePath } from "../../../app/route-contract";

export const UNINITIALIZED_GENERATION = -1 as const;

export type ProjectionGeneration =
  StateGeneration | typeof UNINITIALIZED_GENERATION;

export type AuthoritativeProjection<Value> = {
  readonly generation: ProjectionGeneration;
  readonly value: Value;
};

export type PlatformProjection = {
  readonly appearance: "light" | "dark";
  readonly appearanceSource:
    | "native-snapshot"
    | "macosAppearance"
    | "webviewPreferredColorScheme"
    | "semanticFallback"
    | "webview-live-preference";
  readonly reducedMotion: boolean;
};

export type LaunchProjection = {
  readonly destination: "onboarding" | "workspace-start" | "workspace";
  readonly providerConfigured: boolean;
  readonly onboardingReady: boolean;
  readonly recentWorkspaceIds: readonly WorkspaceId[];
};

export type ProjectProjection = {
  readonly id: ProjectId;
  readonly name: string;
  readonly pathState: "found" | "missing";
  readonly gitVersioned: boolean;
};

export type WorkspaceProjection = {
  readonly activeWorkspaceId: WorkspaceId | null;
  readonly displayName: string | null;
  readonly projects: readonly ProjectProjection[];
  readonly activeProjectId: ProjectId | null;
};

export type SessionProjection = {
  readonly id: SessionId;
  readonly projectId: ProjectId;
  readonly title: string;
  readonly lifecycle: "pending" | "saved" | "inactive";
};

export type SessionsProjection = {
  readonly activeSessionId: SessionId | null;
  readonly sessions: readonly SessionProjection[];
};

export type ConversationTurnProjection = {
  readonly id: string;
  readonly author: "user" | "assistant";
  readonly markdown: string;
  readonly status: "streaming" | "completed" | "failed";
  readonly attachments?: readonly {
    readonly id: string;
    readonly name: string;
    readonly mediaType: string;
    readonly byteLength: number;
    readonly stableReference: string;
    readonly referenceNumber: number;
  }[];
  readonly modelLabel?: string;
  readonly runtimeId?: string;
  readonly runtimeLabel?: string;
  readonly adapterLabel?: string;
  readonly environmentLabel?: string;
  readonly inputTokens?: number;
  readonly outputTokens?: number;
  readonly durationMs?: number;
  readonly activities?: readonly {
    readonly id: string;
    readonly kind: string;
    readonly label: string;
    readonly detail?: string;
    readonly state: "running" | "completed" | "failed";
  }[];
};

export type ConversationProjection = {
  readonly sessionId: SessionId | null;
  readonly title: string | null;
  readonly turns: readonly ConversationTurnProjection[];
  readonly activeAttemptId: string | null;
};

export type ComposerMode = "chat" | "files" | "browser" | "terminal";

export type ComposerProjection = {
  readonly activeModelId: string | null;
  readonly activeReasoningEffort: "off" | "low" | "medium" | "high" | null;
  readonly models: readonly {
    readonly providerId: string;
    readonly providerName: string;
    readonly modelId: string;
    readonly selected: boolean;
    readonly available: boolean;
    readonly supportsVision: boolean;
    readonly supportsTools: boolean;
    readonly supportsReasoning: boolean;
    readonly supportsAudio: boolean;
    readonly contextTokens: number;
  }[];
  readonly allowedModes: readonly ComposerMode[];
  readonly reasoningEfforts: readonly ("off" | "low" | "medium" | "high")[];
  readonly activeBranch: string | null;
  readonly branches: readonly {
    readonly name: string;
    readonly targetOid: string;
  }[];
  readonly branchPendingApprovalId: string | null;
  readonly branchOperationStatus:
    "pending" | "switched" | "created" | "blocked" | "denied" | null;
  readonly branchOperationMessage: string | null;
};

export type ArtifactKind = "browser" | "file" | "folder" | "terminal";

export type ArtifactProjection = {
  readonly id: ArtifactId;
  readonly kind: ArtifactKind;
  readonly title: string;
  readonly focusSupported: boolean;
  readonly sourceTurnId: string;
};

export type ArtifactsProjection = {
  readonly artifacts: readonly ArtifactProjection[];
};

export type SettingsProjection = {
  readonly savedRuntimeId: RuntimeId | null;
  readonly approvalPreset: "ask" | "approve-safe" | "approve-for-me" | "custom";
  readonly restoreLastWorkspace: boolean;
  readonly shellEnvironmentEnabled: boolean;
  readonly browserEnvironment:
    "all-browsers" | "per-project" | "per-chat-session" | "none";
};

export type ApprovalProjection = {
  readonly id: ApprovalId;
  readonly summary: string;
  readonly state: "pending" | "expired" | "denied" | "completed";
};

export type ApprovalsProjection = {
  readonly approvals: readonly ApprovalProjection[];
};

export type RuntimeProjection = {
  readonly id: RuntimeId;
  readonly kind: "open-code" | "pi";
  readonly lifecycle:
    | "stopped"
    | "starting"
    | "ready"
    | "degraded"
    | "incompatible"
    | "stopping"
    | "failed";
  readonly health: "unknown" | "healthy" | "degraded" | "unhealthy";
};

export type RuntimesProjection = {
  readonly runtimes: readonly RuntimeProjection[];
};

export type ExtensionKind = "plugin" | "skill" | "mcp";

export type ExtensionProjection = {
  readonly id: string;
  readonly kind: ExtensionKind;
  readonly name: string;
  readonly state: "available" | "installed-disabled" | "enabled" | "revoked";
};

export type ExtensionsProjection = {
  readonly extensions: readonly ExtensionProjection[];
};

export type NoticeProjection = {
  readonly id: string;
  readonly tone: "info" | "success" | "warning" | "error";
  readonly message: string;
};

export type NoticesProjection = {
  readonly notices: readonly NoticeProjection[];
};

export type AuthorityProjectionMap = {
  readonly platform: PlatformProjection;
  readonly launch: LaunchProjection;
  readonly workspace: WorkspaceProjection;
  readonly sessions: SessionsProjection;
  readonly conversation: ConversationProjection;
  readonly composer: ComposerProjection;
  readonly artifacts: ArtifactsProjection;
  readonly settings: SettingsProjection;
  readonly approvals: ApprovalsProjection;
  readonly runtime: RuntimesProjection;
  readonly extensions: ExtensionsProjection;
  readonly notices: NoticesProjection;
};

export type AuthorityDomain = keyof AuthorityProjectionMap;

export type ShellAuthorityState = {
  readonly [Domain in AuthorityDomain]: AuthoritativeProjection<
    AuthorityProjectionMap[Domain]
  >;
};

export type AuthoritativePublication = {
  [Domain in AuthorityDomain]: {
    readonly source: "snapshot" | "event";
    readonly domain: Domain;
    readonly generation: StateGeneration;
    readonly value: AuthorityProjectionMap[Domain];
  };
}[AuthorityDomain];

export type DraftAttachment = {
  readonly id: AttachmentId;
  readonly name: string;
  readonly byteLength?: number;
  readonly mediaType?: string;
  readonly stableReference?: string;
  readonly referenceNumber: number;
  readonly compatibility:
    "ready" | "needs-vision" | "needs-audio" | "converted" | "incompatible";
};

export type ComposerDraft = {
  readonly mode: ComposerMode;
  readonly modeBeforeFocus: ComposerMode | null;
  readonly text: string;
  readonly attachments: readonly DraftAttachment[];
  readonly nextAttachmentReference: number;
  readonly replyTargetId: string | null;
};

export type LeftPanelDraft = {
  readonly collapsed: boolean;
  readonly width: number;
  readonly overlayOpen: boolean;
};

export type WorkspaceUiDraft = {
  readonly panel: LeftPanelDraft;
  readonly focusedArtifactId: ArtifactId | null;
  readonly focusRestoreTarget: string | null;
};

export type SettingsSection =
  | "providers"
  | "models"
  | "runtimes"
  | "configuration"
  | "plugins"
  | "skills"
  | "mcp"
  | "advanced-policies";

export type SettingsReturnState = {
  readonly route: AppRoutePath;
  readonly workspaceId: WorkspaceId | null;
  readonly sessionId: SessionId | null;
  readonly focusTarget: string | null;
};

export type SettingsDraft = {
  readonly activeSection: SettingsSection;
  readonly visit: SettingsReturnState | null;
  readonly runtimeId: RuntimeId | null;
  readonly dirty: boolean;
};

export type ShellDraftState = {
  readonly workspace: WorkspaceUiDraft;
  readonly composer: ComposerDraft;
  readonly settings: SettingsDraft;
  readonly dismissedNoticeIds: readonly string[];
};

export type QaState = {
  readonly enabled: boolean;
  readonly fixtureId: string | null;
  readonly activeWorkflow: string | null;
};

export type ShellState = {
  readonly shellAuthority: ShellAuthorityState;
  readonly shellDrafts: ShellDraftState;
  readonly shellQa: QaState;
};

export type ShellStateOwner = {
  readonly shellAuthority: ShellAuthorityState;
  readonly shellDrafts: ShellDraftState;
  readonly shellQa: QaState;
};

export type ShellPreloadedState = ShellState;
