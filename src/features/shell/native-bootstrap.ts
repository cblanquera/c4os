import type { AppDispatch, RootState } from "../../app/store";
import {
  readPlatformSnapshot,
  type PlatformSnapshot,
} from "../../platform/platform-service";
import {
  readRuntimeCoreSnapshot,
  type RuntimeCoreSnapshot,
} from "../../platform/runtime-core";
import {
  readWorkspaceStartSnapshot,
  type WorkspaceStartSnapshot,
} from "../../platform/workspace-start";
import {
  readConversationSnapshot,
  type ConversationSnapshot,
} from "../../platform/conversation-service";
import type { RuntimeId, StateGeneration } from "../../platform/protocol";
import {
  shellAuthorityActions,
  shellDraftActions,
  type AuthoritativePublication,
} from "./state";

export interface NativeShellReaders {
  readonly readPlatform: () => Promise<PlatformSnapshot>;
  readonly readRuntime: () => Promise<RuntimeCoreSnapshot>;
  readonly readWorkspaceStart: () => Promise<WorkspaceStartSnapshot>;
  readonly readConversation: () => Promise<ConversationSnapshot>;
  readonly readReducedMotion: () => boolean;
}

export interface NativeShellIngestionResult {
  readonly activeRecoveryNoticePending: boolean;
  readonly publishedDomains: readonly AuthoritativePublication["domain"][];
  readonly unavailableSources: readonly (
    "platform" | "runtime" | "workspaceStart" | "conversation"
  )[];
}

/** Resumes only an already-active native Chat and never overrides user navigation. */
export function nativeResumeRoute(
  result: NativeShellIngestionResult,
  state: RootState,
  currentRoute: string,
): "/chat" | null {
  if (currentRoute !== "/" && currentRoute !== "/start") return null;
  if (result.activeRecoveryNoticePending) return null;
  if (result.unavailableSources.includes("conversation")) return null;
  const authority = state.shellAuthority;
  return authority.workspace.value.activeWorkspaceId !== null &&
    authority.sessions.value.activeSessionId !== null &&
    authority.conversation.value.sessionId !== null
    ? "/chat"
    : null;
}

const defaultReaders: NativeShellReaders = {
  readPlatform: readPlatformSnapshot,
  readRuntime: readRuntimeCoreSnapshot,
  readWorkspaceStart: readWorkspaceStartSnapshot,
  readConversation: readConversationSnapshot,
  readReducedMotion: () =>
    globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ??
    false,
};

/**
 * Publishes only projections derived from validated, allowlisted native
 * snapshots. Unsupported domains stay uninitialized until their owning task
 * supplies a real service projection.
 */
export async function ingestNativeShellProjections(
  dispatch: AppDispatch,
  readers: NativeShellReaders = defaultReaders,
): Promise<NativeShellIngestionResult> {
  const [platform, runtime, workspaceStart, conversation] =
    await Promise.allSettled([
      readers.readPlatform(),
      readers.readRuntime(),
      readers.readWorkspaceStart(),
      readers.readConversation(),
    ]);
  const publications: AuthoritativePublication[] = [];
  const unavailableSources: NativeShellIngestionResult["unavailableSources"][number][] =
    [];

  if (platform.status === "fulfilled") {
    publications.push(platformPublication(platform.value, readers));
  } else {
    unavailableSources.push("platform");
  }

  if (runtime.status === "fulfilled") {
    publications.push(...runtimePublications(runtime.value));
  } else {
    unavailableSources.push("runtime");
  }

  if (workspaceStart.status === "fulfilled") {
    if (conversation.status !== "fulfilled") {
      publications.push(workspacePublication(workspaceStart.value));
    }
  } else {
    unavailableSources.push("workspaceStart");
  }

  if (runtime.status === "fulfilled" && workspaceStart.status === "fulfilled") {
    publications.push(launchPublication(runtime.value, workspaceStart.value));
  }

  if (conversation.status === "fulfilled") {
    publications.push(...conversationPublications(conversation.value));
  } else {
    unavailableSources.push("conversation");
  }

  for (const publication of publications) {
    dispatch(shellAuthorityActions.publicationReceived(publication));
  }
  if (conversation.status === "fulfilled") {
    reconcileConversationDraft(dispatch, conversation.value);
  }

  return {
    activeRecoveryNoticePending:
      workspaceStart.status === "fulfilled" &&
      workspaceStart.value.activeRecoveryNotice !== null,
    publishedDomains: publications.map(({ domain }) => domain),
    unavailableSources,
  };
}

/** Converts one validated native Conversation snapshot into store projections. */
export function conversationPublications(
  snapshot: ConversationSnapshot,
): AuthoritativePublication[] {
  const sessions: {
    id: (typeof snapshot.sessions)[number]["sessionId"];
    projectId: (typeof snapshot.sessions)[number]["projectId"];
    title: string;
    lifecycle: "saved" | "pending";
  }[] = snapshot.sessions.map((session) => ({
    id: session.sessionId,
    projectId: session.projectId,
    title: session.title,
    lifecycle: "saved" as const,
  }));
  if (snapshot.pending) {
    sessions.unshift({
      id: snapshot.pending.sessionId,
      projectId: snapshot.pending.projectId,
      title: snapshot.pending.title,
      lifecycle: "pending" as const,
    });
  }
  const activeConversation = snapshot.activeConversation;
  const turns = activeConversation
    ? activeConversation.turns.flatMap((turn) => {
        const user = {
          id: turn.turnId,
          author: "user" as const,
          markdown: turn.prompt ?? "",
          status: "completed" as const,
          attachments: turn.attachments.map((attachment) => ({
            id: attachment.attachmentId,
            name: attachment.displayName,
            mediaType: attachment.mediaType,
            byteLength: attachment.byteLength,
            stableReference: attachment.stableReference,
            referenceNumber: attachment.originalReference,
          })),
          ...(turn.artifactContext === null
            ? {}
            : { artifactContext: turn.artifactContext }),
        };
        const attempts = activeConversation.attempts.filter(
          (candidate) => candidate.turnId === turn.turnId,
        );
        return [
          user,
          ...attempts.map((attempt) => ({
            id: attempt.attemptId,
            author: "assistant" as const,
            markdown: attempt.assistantMarkdown,
            status:
              attempt.status === "working" ||
              attempt.status === "starting" ||
              attempt.status === "cancelling"
                ? ("streaming" as const)
                : attempt.status === "failed" ||
                    attempt.status === "interrupted" ||
                    attempt.status === "cancelled"
                  ? ("failed" as const)
                  : ("completed" as const),
            modelLabel: attempt.modelId,
            runtimeId: attempt.runtimeId,
            runtimeLabel: attempt.runtimeKind,
            adapterLabel: attempt.adapterId,
            environmentLabel: attempt.environmentId,
            inputTokens: attempt.inputTokens,
            outputTokens: attempt.outputTokens,
            ...(attempt.durationMs === null
              ? {}
              : { durationMs: attempt.durationMs }),
            ...(turn.mcpProvenance === null
              ? {}
              : {
                  mcpProvenance: {
                    snapshotId: turn.mcpProvenance.snapshotId,
                    serverCount: turn.mcpProvenance.serverCount,
                    toolCount: turn.mcpProvenance.toolCount,
                    omittedToolCount: turn.mcpProvenance.omittedToolCount,
                    truncated: turn.mcpProvenance.truncated,
                    tools: turn.mcpProvenance.tools,
                  },
                }),
            activities: attempt.activities.map((activity) => ({
              id: `${attempt.attemptId}:${activity.sequence}`,
              kind: activity.kind,
              label: activity.label,
              ...(activity.detail === null ? {} : { detail: activity.detail }),
              state:
                attempt.status === "working" ||
                attempt.status === "starting" ||
                attempt.status === "cancelling"
                  ? ("running" as const)
                  : attempt.status === "failed" ||
                      attempt.status === "interrupted" ||
                      attempt.status === "cancelled"
                    ? ("failed" as const)
                    : ("completed" as const),
            })),
          })),
        ];
      })
    : [];
  const activeModel = snapshot.models.find(({ selected }) => selected) ?? null;
  return [
    {
      source: "snapshot",
      domain: "workspace",
      generation: snapshot.generation,
      value: {
        activeWorkspaceId: snapshot.workspaceId,
        displayName: snapshot.workspaceName,
        activeProjectId: snapshot.activeProjectId,
        projects: snapshot.projects.map((project) => ({
          id: project.projectId,
          name: project.displayName,
          pathState: project.pathState === "missing" ? "missing" : "found",
          gitVersioned: project.gitVersioned,
        })),
      },
    },
    {
      source: "snapshot",
      domain: "sessions",
      generation: snapshot.generation,
      value: {
        activeSessionId: snapshot.activeSessionId,
        sessions,
      },
    },
    {
      source: "snapshot",
      domain: "conversation",
      generation: snapshot.generation,
      value: {
        sessionId: snapshot.activeSessionId,
        title:
          snapshot.pending?.title ?? snapshot.activeConversation?.title ?? null,
        activeAttemptId: snapshot.activeConversation?.activeAttemptId ?? null,
        turns,
      },
    },
    {
      source: "snapshot",
      domain: "composer",
      generation: snapshot.generation,
      value: {
        activeModelId: activeModel?.modelId ?? null,
        activeReasoningEffort: snapshot.draft.reasoningMode,
        models: snapshot.models.map((model) => ({
          providerId: model.providerId,
          providerName: model.providerName,
          modelId: model.modelId,
          selected: model.selected,
          available: model.available,
          supportsVision: model.supportsVision,
          supportsTools: model.supportsTools,
          supportsReasoning: model.supportsReasoning,
          supportsAudio: model.supportsAudio,
          contextTokens: model.contextTokens,
        })),
        allowedModes: ["chat", "files", "browser", "terminal"],
        reasoningEfforts:
          activeModel?.available && activeModel.supportsReasoning
            ? ["off", "low", "medium", "high"]
            : [],
        activeBranch: snapshot.branchControl?.currentBranch ?? null,
        branches:
          snapshot.branchControl?.branches.map(({ name, targetOid }) => ({
            name,
            targetOid,
          })) ?? [],
        branchPendingApprovalId:
          snapshot.branchControl?.pendingApprovalId ?? null,
        branchOperationStatus: snapshot.branchControl?.operationStatus ?? null,
        branchOperationMessage:
          snapshot.branchControl?.operationMessage ?? null,
      },
    },
  ];
}

export function publishConversationSnapshot(
  dispatch: AppDispatch,
  snapshot: ConversationSnapshot,
  options: { readonly reconcileDraft?: boolean } = {},
): void {
  for (const publication of conversationPublications(snapshot)) {
    dispatch(shellAuthorityActions.publicationReceived(publication));
  }
  if (options.reconcileDraft === true) {
    reconcileConversationDraft(dispatch, snapshot);
  }
}

function reconcileConversationDraft(
  dispatch: AppDispatch,
  snapshot: ConversationSnapshot,
): void {
  dispatch(shellDraftActions.composerTextChanged(snapshot.draft.prompt));
  dispatch(shellDraftActions.composerModeChanged(snapshot.draft.mode));
  dispatch(
    shellDraftActions.composerReplyReconciled({
      expectedReplyTargetId: null,
      authoritativeReplyTargetId: snapshot.draft.replyTargetId,
    }),
  );
  dispatch(
    shellDraftActions.composerAttachmentsReconciled({
      attachments: snapshot.draft.attachments.map((attachment) => ({
        id: attachment.attachmentId,
        name: attachment.displayName,
        byteLength: attachment.byteLength,
        mediaType: attachment.mediaType,
        stableReference: attachment.stableReference,
        referenceNumber: attachment.originalReference,
        compatibility: "ready",
      })),
      nextAttachmentReference: snapshot.draft.nextAttachmentReference,
    }),
  );
}

function platformPublication(
  snapshot: PlatformSnapshot,
  readers: NativeShellReaders,
): AuthoritativePublication {
  return {
    source: "snapshot",
    domain: "platform",
    generation: 0 as StateGeneration,
    value: {
      appearance: snapshot.initialTheme.scheme,
      appearanceSource: snapshot.initialTheme.source,
      reducedMotion: readers.readReducedMotion(),
    },
  };
}

function runtimePublications(
  snapshot: RuntimeCoreSnapshot,
): AuthoritativePublication[] {
  return [
    {
      source: "snapshot",
      domain: "runtime",
      generation: snapshot.generation,
      value: {
        runtimes: snapshot.runtimes.map((runtime) => ({
          id: runtime.runtimeId as RuntimeId,
          kind: runtime.runtimeKind,
          lifecycle: runtime.lifecycle,
          health: runtime.health,
        })),
      },
    },
    {
      source: "snapshot",
      domain: "approvals",
      generation: snapshot.generation,
      value: {
        approvals: snapshot.pendingApprovals.map((approval) => ({
          id: approval.promptId,
          summary:
            approval.disclosureScope === null
              ? approval.summary
              : `${approval.summary} ${approval.disclosureScope}`,
          state: "pending",
        })),
      },
    },
  ];
}

function workspacePublication(
  snapshot: WorkspaceStartSnapshot,
): AuthoritativePublication {
  return {
    source: "snapshot",
    domain: "workspace",
    generation: snapshot.generation,
    value: {
      activeWorkspaceId: null,
      displayName: null,
      projects: [],
      activeProjectId: null,
    },
  };
}

function launchPublication(
  runtime: RuntimeCoreSnapshot,
  workspaceStart: WorkspaceStartSnapshot,
): AuthoritativePublication {
  const generation = Math.max(
    runtime.generation,
    workspaceStart.generation,
  ) as StateGeneration;
  return {
    source: "snapshot",
    domain: "launch",
    generation,
    value: {
      destination: runtime.onboardingReady ? "workspace-start" : "onboarding",
      providerConfigured: runtime.providers.some(({ enabled }) => enabled),
      onboardingReady: runtime.onboardingReady,
      recentWorkspaceIds: workspaceStart.recents.map(
        ({ workspaceId }) => workspaceId,
      ),
    },
  };
}
