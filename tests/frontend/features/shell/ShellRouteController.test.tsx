import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { Provider } from "react-redux";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { APP_ROUTE_DEFINITIONS } from "../../../../src/frontend/app/route-contract";
import { createAppStore } from "../../../../src/frontend/app/store";
import type { ConversationSnapshot } from "../../../../src/frontend/platform/conversation-service";
import {
  initialQaState,
  initialShellAuthorityState,
  initialShellDraftState,
  shellAuthorityActions,
  shellDraftActions,
} from "../../../../src/frontend/features/shell/state";
import type {
  SessionId,
  StateGeneration,
} from "../../../../src/frontend/platform/protocol";
import {
  activeSessionCanOwnArtifacts,
  artifactWorkspaceForActiveSession,
} from "../../../../src/frontend/features/shell/artifact-session";
import { ShellRouteController } from "../../../../src/frontend/features/shell/ShellRouteController";

const conversationServiceMocks = vi.hoisted(() => ({
  readConversationSnapshot: vi.fn(),
  submitConversation: vi.fn(),
}));
const extensionServiceMocks = vi.hoisted(() => ({
  loadExtensionSkill: vi.fn(),
  readExtensionSnapshot: vi.fn(),
  selectExtensionSkill: vi.fn(),
}));

vi.mock("../../../../src/frontend/platform/conversation-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../../../src/frontend/platform/conversation-service")
  >("../../../../src/frontend/platform/conversation-service");
  return { ...actual, ...conversationServiceMocks };
});

vi.mock("../../../../src/frontend/platform/extension-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../../../src/frontend/platform/extension-service")
  >("../../../../src/frontend/platform/extension-service");
  return { ...actual, ...extensionServiceMocks };
});

function extensionSnapshot(generation = 7) {
  return {
    schemaVersion: 1,
    generation,
    marketplaces: [],
    plugins: [],
    skills: [
      {
        identity: {
          sourceKind: "pluginProvided" as const,
          sourceId: "sample-plugin",
          skillId: "review",
        },
        name: "Review",
        summary: "Review the active Project.",
        packageId: "sample-plugin",
        sourceLabel: "Plugin · sample-plugin",
        enabled: true,
        eligible: true,
        valid: true,
        active: true,
        shadowed: false,
        instructionsLoaded: false,
        collisionSources: [],
        diagnostic: null,
        version: "1.0.0",
        isInstalled: true,
        eligibilityDetail: "Eligible for the next turn.",
      },
    ],
    selectedSkill: null,
    activeWorkers: 0,
    lastEventId: generation,
  } satisfies import("../../../../src/frontend/platform/extension-service").ExtensionServiceSnapshot;
}

function renderShellAt(path: string, qaEnabled = false) {
  const store = createAppStore({
    shellAuthority: initialShellAuthorityState,
    shellDrafts: initialShellDraftState,
    shellQa: qaEnabled
      ? { enabled: true, fixtureId: "test", activeWorkflow: "chat" }
      : initialQaState,
  });
  const router = createMemoryRouter(
    APP_ROUTE_DEFINITIONS.map((route) => ({
      path: route.path,
      element: <ShellRouteController route={route.path} />,
    })),
    { initialEntries: [path] },
  );
  return {
    store,
    router,
    ...render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    ),
  };
}

function deferred<Value>() {
  let resolve!: (value: Value) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<Value>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, reject, resolve };
}

function conversationSubmitSnapshot(
  generation: number,
  replyTargetId: string | null,
): ConversationSnapshot {
  return {
    protocolVersion: 1,
    generation: generation as StateGeneration,
    authority: "rust-core",
    workspaceId: "workspace:reply-race" as never,
    workspaceName: "Reply Race Workspace",
    activeProjectId: "project:reply-race" as never,
    activeSessionId: "session:reply-race" as never,
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: "provider:test",
      modelId: "model:test",
      reasoningMode: null,
      mode: "chat",
      replyTargetId,
    },
    projects: [
      {
        projectId: "project:reply-race" as never,
        displayName: "Reply Race Project",
        pathState: "found",
        position: 0,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "session:reply-race" as never,
        projectId: "project:reply-race" as never,
        title: "Reply Race Chat",
        updatedAtMs: 1,
      },
    ],
    activeConversation: {
      sessionId: "session:reply-race" as never,
      title: "Reply Race Chat",
      turns: [
        {
          turnId: "turn:submitted" as never,
          prompt: "The Reply target active when submit began",
          attachments: [],
          artifactContext: null,
          mcpProvenance: null,
          submittedAtMs: 1,
        },
        {
          turnId: "turn:newer" as never,
          prompt: "The newer local Reply target",
          attachments: [],
          artifactContext: null,
          mcpProvenance: null,
          submittedAtMs: 2,
        },
      ],
      attempts: [],
      activeAttemptId: null,
    },
    models: [
      {
        providerId: "provider:test",
        providerName: "Test Provider",
        modelId: "model:test",
        selected: true,
        available: true,
        supportsVision: false,
        supportsTools: true,
        supportsReasoning: false,
        supportsAudio: false,
        contextTokens: 8_192,
      },
    ],
    branchControl: null,
  };
}

function renderConversationSubmitRace() {
  const store = createAppStore({
    shellAuthority: {
      ...initialShellAuthorityState,
      workspace: {
        generation: 1 as StateGeneration,
        value: {
          activeWorkspaceId: "workspace:reply-race" as never,
          displayName: "Reply Race Workspace",
          projects: [
            {
              id: "project:reply-race" as never,
              name: "Reply Race Project",
              pathState: "found",
              gitVersioned: false,
            },
          ],
          activeProjectId: "project:reply-race" as never,
        },
      },
      sessions: {
        generation: 1 as StateGeneration,
        value: {
          activeSessionId: "session:reply-race" as never,
          sessions: [
            {
              id: "session:reply-race" as never,
              projectId: "project:reply-race" as never,
              title: "Reply Race Chat",
              lifecycle: "saved",
            },
          ],
        },
      },
      conversation: {
        generation: 1 as StateGeneration,
        value: {
          sessionId: "session:reply-race" as never,
          title: "Reply Race Chat",
          activeAttemptId: null,
          turns: [
            {
              id: "turn:submitted",
              author: "user",
              markdown: "The Reply target active when submit began",
              status: "completed",
            },
            {
              id: "turn:newer",
              author: "user",
              markdown: "The newer local Reply target",
              status: "completed",
            },
          ],
        },
      },
      composer: {
        generation: 1 as StateGeneration,
        value: {
          ...initialShellAuthorityState.composer.value,
          activeModelId: "model:test",
          models: [
            {
              providerId: "provider:test",
              providerName: "Test Provider",
              modelId: "model:test",
              selected: true,
              available: true,
              supportsVision: false,
              supportsTools: true,
              supportsReasoning: false,
              supportsAudio: false,
              contextTokens: 8_192,
            },
          ],
        },
      },
    },
    shellDrafts: {
      ...initialShellDraftState,
      composer: {
        ...initialShellDraftState.composer,
        text: "Send the submitted Reply",
        replyTargetId: "turn:submitted",
      },
    },
    shellQa: initialQaState,
  });
  const router = createMemoryRouter(
    [{ path: "/chat", element: <ShellRouteController route="/chat" /> }],
    { initialEntries: ["/chat"] },
  );
  return {
    store,
    ...render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    ),
  };
}

function selectNewerReplyTarget() {
  const newerTurn = screen.getAllByRole("article", {
    name: "User message",
  })[1];
  if (newerTurn === undefined) throw new Error("Newer Reply turn is missing.");
  fireEvent.click(within(newerTurn).getByRole("button", { name: "Reply" }));
}

describe("ShellRouteController", () => {
  beforeEach(() => {
    conversationServiceMocks.readConversationSnapshot.mockReset();
    conversationServiceMocks.submitConversation.mockReset();
    extensionServiceMocks.loadExtensionSkill.mockReset();
    extensionServiceMocks.readExtensionSnapshot.mockReset();
    extensionServiceMocks.selectExtensionSkill.mockReset();
    extensionServiceMocks.readExtensionSnapshot.mockResolvedValue(
      extensionSnapshot(),
    );
  });

  it("never projects a cached Artifact Workspace into a different Chat", () => {
    const cached = {
      protocolVersion: 1,
      generation: 7 as StateGeneration,
      authority: "rust-core",
      workspaceId: "workspace:cached" as never,
      activeProjectId: "project:cached" as never,
      activeSessionId: "session:cached" as never,
      focusedArtifactId: null,
      artifacts: [],
    } satisfies import("../../../../src/frontend/platform/artifact-service").ArtifactWorkspaceSnapshot;

    expect(
      artifactWorkspaceForActiveSession(cached, "session:next" as SessionId),
    ).toBeNull();
    expect(
      artifactWorkspaceForActiveSession(cached, "session:cached" as SessionId),
    ).toBe(cached);
    expect(artifactWorkspaceForActiveSession(cached, null)).toBeNull();
  });

  it("never requests Artifact state for a provisional blank Chat", () => {
    const pendingSessionId = "session:pending" as SessionId;
    expect(
      activeSessionCanOwnArtifacts({
        activeSessionId: pendingSessionId,
        sessions: [
          {
            id: pendingSessionId,
            projectId: "project:pending" as never,
            title: "New Chat",
            lifecycle: "pending",
          },
        ],
      }),
    ).toBe(false);

    expect(
      activeSessionCanOwnArtifacts({
        activeSessionId: pendingSessionId,
        sessions: [
          {
            id: pendingSessionId,
            projectId: "project:pending" as never,
            title: "Saved Chat",
            lifecycle: "saved",
          },
        ],
      }),
    ).toBe(true);
  });

  it("keeps every accepted route directly addressable in one composed shell", () => {
    for (const definition of APP_ROUTE_DEFINITIONS) {
      const rendered = renderShellAt(definition.path);
      if (
        [
          "/chat",
          "/chat-search",
          "/chat-capabilities",
          "/files",
          "/browser",
          "/terminal",
        ].includes(definition.path)
      ) {
        expect(
          screen.getByRole("region", { name: definition.title }),
        ).toHaveAttribute("data-route-surface", "compact");
      } else {
        expect(
          screen.getByRole("heading", { name: definition.title, level: 1 }),
        ).toBeVisible();
      }
      expect(
        document.querySelector(`[data-route="${definition.path}"]`),
      ).toBeInTheDocument();
      rendered.unmount();
    }
  });

  it("round-trips through Settings without losing panel, composer, or focus state", async () => {
    conversationServiceMocks.readConversationSnapshot.mockResolvedValueOnce(
      conversationSubmitSnapshot(2, null),
    );
    const { store } = renderShellAt("/chat", true);
    const composer = screen.getByRole("textbox", { name: "Message" });
    fireEvent.change(composer, { target: { value: "Keep this exact draft" } });
    fireEvent.click(
      screen.getByRole("button", { name: "Collapse project panel" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(
      await screen.findByRole("heading", { name: "Providers", level: 1 }),
    ).toBeVisible();
    expect(store.getState().shellDrafts.settings.visit?.route).toBe("/chat");

    fireEvent.click(screen.getByRole("button", { name: "Back to C4OS" }));
    const restoredComposer = await screen.findByRole("textbox", {
      name: "Message",
    });
    expect(restoredComposer).toHaveValue("Keep this exact draft");
    expect(
      screen.getByRole("button", { name: "Show project panel" }),
    ).toBeVisible();
    expect(
      conversationServiceMocks.readConversationSnapshot,
    ).toHaveBeenCalledOnce();
    expect(store.getState().shellAuthority.composer.value.activeModelId).toBe(
      "model:test",
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Settings" })).toHaveFocus(),
    );
    expect(store.getState().shellDrafts.settings.visit).toBeNull();
  });

  it("returns to Workspace Start without restoring focus when Settings identities no longer match", async () => {
    const { router, store } = renderShellAt("/chat", true);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(
      await screen.findByRole("heading", { name: "Providers", level: 1 }),
    ).toBeVisible();
    expect(store.getState().shellDrafts.settings.visit).toMatchObject({
      route: "/chat",
      workspaceId: null,
      sessionId: null,
      focusTarget: "workspace-settings",
    });

    act(() => {
      store.dispatch(
        shellAuthorityActions.publicationReceived({
          source: "snapshot",
          domain: "workspace",
          generation: 1 as StateGeneration,
          value: {
            activeWorkspaceId: "workspace:replacement" as never,
            displayName: "Replacement Workspace",
            projects: [],
            activeProjectId: null,
          },
        }),
      );
      store.dispatch(
        shellAuthorityActions.publicationReceived({
          source: "snapshot",
          domain: "sessions",
          generation: 1 as StateGeneration,
          value: {
            activeSessionId: "session:replacement" as never,
            sessions: [],
          },
        }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: "Back to C4OS" }));

    await waitFor(() => expect(router.state.location.pathname).toBe("/start"));
    expect(router.state.location.state).toBeNull();
    expect(store.getState().shellDrafts.settings.visit).toBeNull();
    expect(
      await screen.findByRole("heading", {
        name: "Workspace Start",
        level: 1,
      }),
    ).toBeVisible();
  });

  it("prepares the explicitly selected Skill visibly before Try in Chat navigates", async () => {
    const snapshot = extensionSnapshot();
    extensionServiceMocks.loadExtensionSkill.mockResolvedValue({
      identity: snapshot.skills[0]?.identity,
      packageId: "sample-plugin",
      entrypointDigest: `sha256:${"a".repeat(64)}`,
      instructions: "Review this change carefully.",
      referencedResources: [],
    });
    extensionServiceMocks.selectExtensionSkill.mockResolvedValue({
      ...snapshot,
      generation: 8,
      selectedSkill: snapshot.skills[0]?.identity ?? null,
      lastEventId: 8,
    });
    const { router, store } = renderShellAt("/settings/skills");

    expect(await screen.findByText("Review the active Project.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Details" }));
    const dialog = await screen.findByRole("dialog", { name: "Review" });
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Load instructions" }),
    );
    await waitFor(() =>
      expect(
        within(dialog).getByRole("button", { name: "Try in Chat" }),
      ).toBeEnabled(),
    );
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Try in Chat" }),
    );

    await waitFor(() => expect(router.state.location.pathname).toBe("/chat"));
    expect(extensionServiceMocks.selectExtensionSkill).toHaveBeenCalledWith(
      "plugin:sample-plugin:review",
    );
    expect(store.getState().shellDrafts.composer.mode).toBe("chat");
    expect(store.getState().shellDrafts.composer.text).toBe(
      "[Selected Skill: plugin:sample-plugin:review]\n\n",
    );
  });

  it("does not expose the review Settings control in production state", () => {
    renderShellAt("/chat");
    expect(screen.queryByRole("button", { name: "Settings" })).toBeNull();
  });

  it("keeps synthetic unknown run activity inline-only", () => {
    const store = createAppStore({
      shellAuthority: {
        ...initialShellAuthorityState,
        conversation: {
          generation: 1 as StateGeneration,
          value: {
            sessionId: null,
            title: "Focused run activity",
            activeAttemptId: null,
            turns: [
              {
                id: "turn:assistant",
                author: "assistant",
                markdown: "The safe result is ready.",
                status: "completed",
                activities: [
                  {
                    id: "activity:safe-summary",
                    kind: "reasoning-summary",
                    label: "Checked the bounded result",
                    detail: "No private model reasoning is included.",
                    state: "completed",
                  },
                ],
              },
            ],
          },
        },
      },
      shellDrafts: initialShellDraftState,
      shellQa: initialQaState,
    });
    const router = createMemoryRouter(
      [{ path: "/chat", element: <ShellRouteController route="/chat" /> }],
      { initialEntries: ["/chat"] },
    );
    render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    );

    expect(screen.getByRole("complementary")).toHaveAccessibleName("Projects");
    expect(screen.queryByRole("button", { name: "Expand" })).toBeNull();
    expect(store.getState().shellDrafts.workspace.focusedArtifactId).toBeNull();
    expect(screen.queryByLabelText("Contextual conversation")).toBeNull();
  });

  it("renders and clears a Reply target restored in the composer draft", () => {
    const store = createAppStore({
      shellAuthority: {
        ...initialShellAuthorityState,
        conversation: {
          generation: 1 as StateGeneration,
          value: {
            sessionId: null,
            title: "Restored Reply",
            activeAttemptId: null,
            turns: [
              {
                id: "turn:reply",
                author: "user",
                markdown: "Keep this exact persisted reply context",
                status: "completed",
              },
            ],
          },
        },
      },
      shellDrafts: {
        ...initialShellDraftState,
        composer: {
          ...initialShellDraftState.composer,
          replyTargetId: "turn:reply",
        },
      },
      shellQa: initialQaState,
    });
    const router = createMemoryRouter(
      [{ path: "/chat", element: <ShellRouteController route="/chat" /> }],
      { initialEntries: ["/chat"] },
    );
    render(
      <Provider store={store}>
        <RouterProvider router={router} />
      </Provider>,
    );

    expect(
      screen.getByRole("region", { name: "Reply reference" }),
    ).toHaveTextContent("Keep this exact persisted reply context");
    fireEvent.click(
      screen.getByRole("button", { name: "Remove reply reference" }),
    );
    expect(store.getState().shellDrafts.composer.replyTargetId).toBeNull();
    expect(
      screen.queryByRole("region", { name: "Reply reference" }),
    ).toBeNull();
  });

  it("does not clear a newer Reply target when submit succeeds", async () => {
    const submission = deferred<ConversationSnapshot>();
    conversationServiceMocks.submitConversation.mockReturnValueOnce(
      submission.promise,
    );
    const { store } = renderConversationSubmitRace();

    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(conversationServiceMocks.submitConversation).toHaveBeenCalledOnce();
    selectNewerReplyTarget();
    expect(store.getState().shellDrafts.composer.replyTargetId).toBe(
      "turn:newer",
    );

    await act(async () => {
      submission.resolve(conversationSubmitSnapshot(2, null));
      await submission.promise;
    });

    expect(store.getState().shellDrafts.composer.replyTargetId).toBe(
      "turn:newer",
    );
  });

  it("does not clear a newer Reply target after submit failure refresh", async () => {
    const submission = deferred<ConversationSnapshot>();
    const refresh = deferred<ConversationSnapshot>();
    conversationServiceMocks.submitConversation.mockReturnValueOnce(
      submission.promise,
    );
    conversationServiceMocks.readConversationSnapshot.mockReturnValueOnce(
      refresh.promise,
    );
    const { store } = renderConversationSubmitRace();

    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    selectNewerReplyTarget();

    await act(async () => {
      submission.reject(new Error("Submit failed safely."));
      await Promise.resolve();
    });
    await waitFor(() =>
      expect(
        conversationServiceMocks.readConversationSnapshot,
      ).toHaveBeenCalledOnce(),
    );

    await act(async () => {
      refresh.resolve(conversationSubmitSnapshot(2, null));
      await refresh.promise;
    });

    expect(store.getState().shellDrafts.composer.replyTargetId).toBe(
      "turn:newer",
    );
  });

  it("reconciles retained panel width and ARIA bounds on every viewport resize", async () => {
    const originalWidth = window.innerWidth;
    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 1280,
    });
    const rendered = renderShellAt("/chat");

    act(() => {
      rendered.store.dispatch(
        shellDraftActions.leftPanelResized({
          width: 900,
          viewportWidth: 1280,
        }),
      );
    });
    let resizer = screen.getByRole("separator", {
      name: "Resize project panel",
    });
    expect(resizer).toHaveAttribute("aria-valuenow", "704");
    expect(resizer).toHaveAttribute("aria-valuemax", "704");

    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 760,
    });
    fireEvent(window, new Event("resize"));
    await waitFor(() =>
      expect(
        document.querySelector("[data-shell-layout='workspace']"),
      ).toHaveAttribute("data-panel-mode", "overlay"),
    );
    fireEvent.click(screen.getByRole("button", { name: "Show project panel" }));
    resizer = await screen.findByRole("separator", {
      name: "Resize project panel",
    });
    await waitFor(() => {
      expect(resizer).toHaveAttribute("aria-valuemax", "340");
      expect(resizer).toHaveAttribute("aria-valuenow", "340");
      expect(rendered.store.getState().shellDrafts.workspace.panel.width).toBe(
        340,
      );
    });

    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: 700,
    });
    fireEvent(window, new Event("resize"));
    await waitFor(() => {
      expect(resizer).toHaveAttribute("aria-valuemax", "280");
      expect(resizer).toHaveAttribute("aria-valuenow", "280");
    });

    rendered.unmount();
    Object.defineProperty(window, "innerWidth", {
      configurable: true,
      value: originalWidth,
    });
  });
});
