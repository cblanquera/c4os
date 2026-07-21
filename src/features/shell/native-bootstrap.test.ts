import { describe, expect, it } from "vitest";

import { createAppStore } from "../../app/store";
import type { PlatformSnapshot } from "../../platform/platform-service";
import type { RuntimeCoreSnapshot } from "../../platform/runtime-core";
import type { ConversationSnapshot } from "../../platform/conversation-service";
import type {
  StateGeneration,
  WorkspaceId,
  WorkspaceStartSnapshot,
} from "../../platform/protocol";
import { shellDraftActions, UNINITIALIZED_GENERATION } from "./state";
import {
  ingestNativeShellProjections,
  nativeResumeRoute,
  type NativeShellReaders,
} from "./native-bootstrap";

const generation = 7 as StateGeneration;

function readers(
  overrides: Partial<NativeShellReaders> = {},
): NativeShellReaders {
  return {
    readPlatform: () => Promise.resolve(platformSnapshot()),
    readRuntime: () => Promise.resolve(runtimeSnapshot()),
    readWorkspaceStart: () => Promise.resolve(workspaceStartSnapshot()),
    readConversation: () => Promise.resolve(conversationSnapshot()),
    readReducedMotion: () => true,
    ...overrides,
  };
}

describe("native shell projection ingestion", () => {
  it("publishes validated platform, launch, workspace, runtime, and approval domains", async () => {
    const store = createAppStore(undefined);
    const result = await ingestNativeShellProjections(
      store.dispatch,
      readers(),
    );

    expect(result).toEqual({
      publishedDomains: [
        "platform",
        "runtime",
        "approvals",
        "launch",
        "workspace",
        "sessions",
        "conversation",
        "composer",
      ],
      unavailableSources: [],
    });
    expect(store.getState().shellAuthority.platform).toMatchObject({
      generation: 0,
      value: {
        appearance: "dark",
        appearanceSource: "macosAppearance",
        reducedMotion: true,
      },
    });
    expect(store.getState().shellAuthority.launch).toMatchObject({
      generation,
      value: {
        destination: "workspace-start",
        providerConfigured: true,
        onboardingReady: true,
        recentWorkspaceIds: ["workspace:native"],
      },
    });
    expect(store.getState().shellAuthority.runtime.value.runtimes).toEqual([
      expect.objectContaining({ id: "runtime:native", lifecycle: "ready" }),
    ]);
    expect(store.getState().shellAuthority.approvals.value.approvals).toEqual([
      expect.objectContaining({ id: "approval:native", state: "pending" }),
    ]);
    expect(nativeResumeRoute(result, store.getState(), "/start")).toBe("/chat");
    expect(
      nativeResumeRoute(result, store.getState(), "/settings/models"),
    ).toBe(null);
  });

  it("leaves unsupported native sources fail-closed without rejecting available domains", async () => {
    const store = createAppStore(undefined);
    const result = await ingestNativeShellProjections(
      store.dispatch,
      readers({
        readRuntime: () => Promise.reject(new Error("runtime unavailable")),
      }),
    );

    expect(result.publishedDomains).toEqual([
      "platform",
      "workspace",
      "sessions",
      "conversation",
      "composer",
    ]);
    expect(result.unavailableSources).toEqual(["runtime"]);
    expect(store.getState().shellAuthority.runtime.generation).toBe(
      UNINITIALIZED_GENERATION,
    );
    expect(store.getState().shellAuthority.launch.generation).toBe(
      UNINITIALIZED_GENERATION,
    );
    expect(nativeResumeRoute(result, store.getState(), "/start")).toBe("/chat");
  });

  it("does not resume when the authoritative Conversation source is unavailable", async () => {
    const store = createAppStore(undefined);
    const result = await ingestNativeShellProjections(
      store.dispatch,
      readers({
        readConversation: () =>
          Promise.reject(new Error("conversation unavailable")),
      }),
    );

    expect(nativeResumeRoute(result, store.getState(), "/start")).toBeNull();
  });

  it("hydrates a persisted Reply target before autosave can clear it", async () => {
    const store = createAppStore(undefined);
    const base = conversationSnapshot();
    const restored: ConversationSnapshot = {
      ...base,
      draft: {
        ...base.draft,
        attachments: [
          {
            attachmentId: "attachment:one" as never,
            displayName: "one.txt",
            mediaType: "text/plain",
            byteLength: 1,
            stableReference: "reference:one",
            originalReference: 1,
          },
          {
            attachmentId: "attachment:three" as never,
            displayName: "three.txt",
            mediaType: "text/plain",
            byteLength: 3,
            stableReference: "reference:three",
            originalReference: 3,
          },
        ],
        nextAttachmentReference: 4,
        replyTargetId: "turn:reply",
      },
      activeConversation: {
        sessionId: "session:native" as never,
        title: "Native Chat",
        turns: [
          {
            turnId: "turn:reply" as never,
            prompt: "Keep this persisted Reply target",
            attachments: [],
            submittedAtMs: 1_721_312_001,
          },
        ],
        attempts: [],
        activeAttemptId: null,
      },
    };

    await ingestNativeShellProjections(
      store.dispatch,
      readers({ readConversation: () => Promise.resolve(restored) }),
    );

    expect(store.getState().shellDrafts.composer.replyTargetId).toBe(
      "turn:reply",
    );
    expect(
      store.getState().shellDrafts.composer.attachments.map((attachment) => ({
        id: attachment.id,
        referenceNumber: attachment.referenceNumber,
      })),
    ).toEqual([
      { id: "attachment:one", referenceNumber: 1 },
      { id: "attachment:three", referenceNumber: 3 },
    ]);
    expect(store.getState().shellDrafts.composer.nextAttachmentReference).toBe(
      4,
    );
  });

  it("does not let delayed bootstrap replace a newer local Reply target", async () => {
    const store = createAppStore(undefined);
    store.dispatch(shellDraftActions.composerReplyChanged("turn:newer-local"));
    const base = conversationSnapshot();

    await ingestNativeShellProjections(
      store.dispatch,
      readers({
        readConversation: () =>
          Promise.resolve({
            ...base,
            draft: { ...base.draft, replyTargetId: "turn:persisted" },
          }),
      }),
    );

    expect(store.getState().shellDrafts.composer.replyTargetId).toBe(
      "turn:newer-local",
    );
  });

  it("preserves failed and retried attempts in authoritative order", async () => {
    const store = createAppStore(undefined);
    const snapshot = conversationSnapshot();
    const retriedSnapshot: ConversationSnapshot = {
      ...snapshot,
      activeConversation: {
        sessionId: "session:native" as never,
        title: "Native Chat",
        turns: [
          {
            turnId: "turn:native" as never,
            prompt: "Retry this request",
            attachments: [],
            submittedAtMs: 1_721_312_001,
          },
        ],
        attempts: [
          conversationAttempt({
            attemptId: "attempt:failed",
            status: "failed",
            assistantMarkdown: "The first attempt failed after partial work.",
          }),
          conversationAttempt({
            attemptId: "attempt:completed-retry",
            status: "completed",
            assistantMarkdown: "The first retry completed.",
          }),
          conversationAttempt({
            attemptId: "attempt:active-retry",
            status: "working",
            assistantMarkdown: "The active retry has new output.",
          }),
        ],
        activeAttemptId: "attempt:active-retry" as never,
      },
    };

    await ingestNativeShellProjections(
      store.dispatch,
      readers({
        readConversation: () => Promise.resolve(retriedSnapshot),
      }),
    );

    const conversation = store.getState().shellAuthority.conversation.value;
    expect(conversation.activeAttemptId).toBe("attempt:active-retry");
    expect(conversation.turns).toEqual([
      expect.objectContaining({
        id: "turn:native",
        author: "user",
        markdown: "Retry this request",
        status: "completed",
      }),
      expect.objectContaining({
        id: "attempt:failed",
        author: "assistant",
        markdown: "The first attempt failed after partial work.",
        status: "failed",
        activities: [expect.objectContaining({ state: "failed" })],
      }),
      expect.objectContaining({
        id: "attempt:completed-retry",
        author: "assistant",
        markdown: "The first retry completed.",
        status: "completed",
        activities: [expect.objectContaining({ state: "completed" })],
      }),
      expect.objectContaining({
        id: "attempt:active-retry",
        author: "assistant",
        markdown: "The active retry has new output.",
        status: "streaming",
        activities: [expect.objectContaining({ state: "running" })],
      }),
    ]);
  });
});

function platformSnapshot(): PlatformSnapshot {
  return {
    contractVersion: 1,
    platform: "macos",
    architecture: "aarch64",
    initialTheme: { scheme: "dark", source: "macosAppearance" },
    liveThemeSource: "webviewPrefersColorScheme",
    window: {
      decorations: "standard",
      titlebarTransparent: false,
      titlebarOverlay: false,
      initiallyVisible: false,
      revealFallbackTimeoutMs: 4_000,
    },
    vocabulary: {
      revealAction: "Reveal in Finder",
      primaryModifierSymbol: "⌘",
      alternateModifierSymbol: "⌥",
      shiftModifierSymbol: "⇧",
    },
    capabilities: {
      nativeApplicationMenu: true,
      nativeSettingsShortcut: true,
      nativeFilePicker: true,
      nativeFolderPicker: true,
      nativeWorkspacePicker: true,
      standardWindowDecorations: true,
    },
    settingsMenu: {
      menuItemId: "c4os.menu.settings",
      commandId: "c4os.command.openSettings",
      route: "/settings/providers",
      accelerator: "CmdOrCtrl+,",
      keyboardLabel: "⌘,",
    },
  };
}

function conversationSnapshot(): ConversationSnapshot {
  return {
    protocolVersion: 1,
    generation,
    authority: "rust-core",
    workspaceId: "workspace:native" as never,
    workspaceName: "Native Workspace",
    activeProjectId: "project:native" as never,
    activeSessionId: "session:native" as never,
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: null,
      modelId: null,
      reasoningMode: null,
      mode: "chat",
      replyTargetId: null,
    },
    projects: [
      {
        projectId: "project:native" as never,
        displayName: "Native Project",
        pathState: "found",
        position: 0,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "session:native" as never,
        projectId: "project:native" as never,
        title: "Native Chat",
        updatedAtMs: 1_721_312_000,
      },
    ],
    activeConversation: {
      sessionId: "session:native" as never,
      title: "Native Chat",
      turns: [],
      attempts: [],
      activeAttemptId: null,
    },
    models: [],
    branchControl: null,
  };
}

/** Builds one native attempt while each retry-focused assertion varies its outcome. */
function conversationAttempt({
  attemptId,
  status,
  assistantMarkdown,
}: {
  attemptId: string;
  status: "completed" | "failed" | "working";
  assistantMarkdown: string;
}): NonNullable<
  ConversationSnapshot["activeConversation"]
>["attempts"][number] {
  return {
    attemptId: attemptId as never,
    turnId: "turn:native" as never,
    status,
    assistantMarkdown,
    activities: [
      {
        sequence: 1,
        kind: status === "failed" ? "error" : "work",
        label: `${attemptId} activity`,
        detail: null,
      },
    ],
    runtimeId: "runtime:native" as never,
    runtimeKind: "open-code",
    environmentId: "environment:native" as never,
    providerId: "provider:native",
    modelId: "model:native",
    adapterId: "adapter:native",
    inputTokens: 12,
    outputTokens: 24,
    durationMs: status === "working" ? null : 150,
  };
}

function runtimeSnapshot(): RuntimeCoreSnapshot {
  return {
    authority: "rust-core",
    generation,
    providerGeneration: 4,
    capabilityGeneration: 5,
    runtimeGeneration: 6,
    onboardingReady: true,
    providers: [
      {
        providerId: "provider:native",
        displayName: "Native Provider",
        enabled: true,
        testStatus: {},
        modelCount: 1,
        selectedModelId: "model:native",
      },
    ],
    runtimes: [
      {
        runtimeId: "runtime:native",
        runtimeKind: "open-code",
        nativeVersion: "1.18.3",
        lifecycle: "ready",
        health: "healthy",
        processGeneration: 2,
      },
    ],
    pendingApprovals: [
      {
        runtimeId: "runtime:native" as never,
        correlationId: "correlation:native" as never,
        promptId: "approval:native" as never,
      },
    ],
  };
}

function workspaceStartSnapshot(): WorkspaceStartSnapshot {
  return {
    protocolVersion: 1,
    generation,
    authority: "rust-core",
    recents: [
      {
        workspaceId: "workspace:native" as WorkspaceId,
        displayName: "Native Workspace",
        lastOpenedAt: 1_721_312_000,
        isMissing: false,
      },
    ],
  };
}
