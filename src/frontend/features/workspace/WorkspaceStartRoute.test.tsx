import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Provider } from "react-redux";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppStore } from "../../app/store";
import type { ConversationSnapshot } from "../../platform/conversation-service";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";
import { WorkspaceStartRoute } from "./WorkspaceStartRoute";

const conversationMocks = vi.hoisted(() => ({
  readConversationSnapshot: vi.fn(),
}));
const platformMocks = vi.hoisted(() => ({
  pickNative: vi.fn(),
}));
const workspaceMocks = vi.hoisted(() => ({
  acknowledgeWorkspaceRecovery: vi.fn(),
  answerWorkspaceCloneApproval: vi.fn(),
  cloneWorkspaceRepository: vi.fn(),
  openRecentWorkspace: vi.fn(),
  openWorkspaceArchive: vi.fn(),
  openWorkspaceFolder: vi.fn(),
  readWorkspaceStartSnapshot: vi.fn(),
}));

vi.mock("../../platform/conversation-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/conversation-service")
  >("../../platform/conversation-service");
  return { ...actual, ...conversationMocks };
});

vi.mock("../../platform/platform-service", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/platform-service")
  >("../../platform/platform-service");
  return { ...actual, ...platformMocks };
});

vi.mock("../../platform/workspace-start", async () => {
  const actual = await vi.importActual<
    typeof import("../../platform/workspace-start")
  >("../../platform/workspace-start");
  return { ...actual, ...workspaceMocks };
});

function conversationSnapshot(): ConversationSnapshot {
  return {
    protocolVersion: 1,
    generation: 12 as never,
    authority: "rust-core",
    workspaceId: "workspace:activated" as never,
    workspaceName: "Activated Workspace",
    activeProjectId: "project:activated" as never,
    activeSessionId: "session:activated" as never,
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
        projectId: "project:activated" as never,
        displayName: "Activated Project",
        pathState: "found",
        position: 0,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "session:activated" as never,
        projectId: "project:activated" as never,
        title: "Activated Chat",
        updatedAtMs: 12,
      },
    ],
    activeConversation: {
      sessionId: "session:activated" as never,
      title: "Activated Chat",
      turns: [],
      attempts: [],
      activeAttemptId: null,
    },
    models: [],
    branchControl: null,
  };
}

describe("WorkspaceStartRoute", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    workspaceMocks.readWorkspaceStartSnapshot.mockResolvedValue({
      protocolVersion: 1,
      generation: 11,
      authority: "rust-workspace-service",
      activeRecoveryNotice: null,
      recents: [],
    });
    platformMocks.pickNative.mockResolvedValue({
      type: "selected",
      contractVersion: 1,
      requestId: "request:00000000-0000-4000-8000-000000000412",
      grants: [
        {
          grantId: "picker-grant:activated",
          objectKind: "folder",
          displayName: "Activated Workspace",
        },
      ],
    });
    workspaceMocks.openWorkspaceFolder.mockResolvedValue({
      authority: "rust-workspace-service",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      recovered: false,
      recoveryNotice: null,
    });
    workspaceMocks.acknowledgeWorkspaceRecovery.mockResolvedValue({
      protocolVersion: 1,
      generation: 14,
      authority: "rust-core",
      activeRecoveryNotice: null,
      recents: [],
    });
  });

  it("retries only Chat hydration after native Workspace activation succeeds", async () => {
    conversationMocks.readConversationSnapshot
      .mockRejectedValueOnce(new Error("temporary Chat hydration failure"))
      .mockResolvedValueOnce(conversationSnapshot());
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: /Open a folder/u }),
    );
    expect(
      await screen.findByText(
        /Activated Workspace is active, but Chat still needs to be refreshed/u,
      ),
    ).toBeVisible();
    expect(workspaceMocks.openWorkspaceFolder).toHaveBeenCalledTimes(1);
    expect(platformMocks.pickNative).toHaveBeenCalledTimes(1);

    fireEvent.click(
      screen.getByRole("button", { name: "Retry Chat recovery" }),
    );
    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    await waitFor(() =>
      expect(conversationMocks.readConversationSnapshot).toHaveBeenCalledTimes(
        2,
      ),
    );
    expect(workspaceMocks.openWorkspaceFolder).toHaveBeenCalledTimes(1);
    expect(platformMocks.pickNative).toHaveBeenCalledTimes(1);
  });

  it("blocks Chat navigation until a structured recovery notice is reviewed", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Recovered generation 13 after an interrupted archive save.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    workspaceMocks.openWorkspaceFolder.mockResolvedValueOnce({
      authority: "rust-workspace-service",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      recovered: true,
      recoveryNotice,
    });
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: /Open a folder/u }),
    );
    const notice = await screen.findByText(
      /Recovered Activated Workspace. Review before the next save/u,
    );
    expect(notice).toBeVisible();
    expect(screen.queryByText("Hydrated Chat")).not.toBeInTheDocument();
    await waitFor(() => expect(notice.parentElement).toHaveFocus());

    workspaceMocks.acknowledgeWorkspaceRecovery.mockClear();
    conversationMocks.readConversationSnapshot.mockClear();
    fireEvent.click(screen.getByRole("button", { name: "Continue to Chat" }));
    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledWith(
      recoveryNotice,
    );
    const acknowledgementOrder =
      workspaceMocks.acknowledgeWorkspaceRecovery.mock.invocationCallOrder[0];
    const hydrationOrder =
      conversationMocks.readConversationSnapshot.mock.invocationCallOrder[0];
    if (acknowledgementOrder === undefined || hydrationOrder === undefined) {
      throw new Error("Expected acknowledgement and hydration calls.");
    }
    expect(acknowledgementOrder).toBeLessThan(hydrationOrder);
    expect(workspaceMocks.openWorkspaceFolder).toHaveBeenCalledTimes(1);
    expect(platformMocks.pickNative).toHaveBeenCalledTimes(1);
  });

  it("delivers an automatic startup recovery notice before native resume", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Startup restored generation 13 from validated state.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    workspaceMocks.readWorkspaceStartSnapshot.mockResolvedValueOnce({
      protocolVersion: 1,
      generation: 13,
      authority: "rust-workspace-service",
      activeRecoveryNotice: recoveryNotice,
      recents: [],
    });
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    expect(
      await screen.findByText(
        /Recovered Activated Workspace. Review before the next save/u,
      ),
    ).toBeVisible();
    expect(screen.queryByText("Hydrated Chat")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Continue to Chat" }));
    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledWith(
      recoveryNotice,
    );
    expect(platformMocks.pickNative).not.toHaveBeenCalled();
    expect(workspaceMocks.openWorkspaceFolder).not.toHaveBeenCalled();
  });

  it("keeps Chat blocked and offers a bounded retry when recovery acknowledgement fails", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Startup restored generation 13 from validated state.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    workspaceMocks.readWorkspaceStartSnapshot.mockResolvedValueOnce({
      protocolVersion: 1,
      generation: 13,
      authority: "rust-core",
      activeRecoveryNotice: recoveryNotice,
      recents: [
        {
          workspaceId: "workspace:other",
          displayName: "Other Workspace",
          lastOpenedAt: 1_784_476_800,
          isMissing: false,
        },
      ],
    });
    workspaceMocks.acknowledgeWorkspaceRecovery
      .mockRejectedValueOnce(new Error("stale recovery identity"))
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 14,
        authority: "rust-core",
        activeRecoveryNotice: null,
        recents: [],
      });
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    await screen.findByRole("button", { name: "Continue to Chat" });
    fireEvent.click(screen.getByRole("button", { name: "Continue to Chat" }));

    expect(
      await screen.findByText(/could not record that recovery review/u),
    ).toBeVisible();
    expect(screen.queryByText("Hydrated Chat")).not.toBeInTheDocument();
    expect(conversationMocks.readConversationSnapshot).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: /Open a folder/u }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: /Other Workspace/u }),
    ).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: /Open a folder/u }));
    fireEvent.click(screen.getByRole("button", { name: /Other Workspace/u }));
    expect(platformMocks.pickNative).not.toHaveBeenCalled();
    expect(workspaceMocks.openRecentWorkspace).not.toHaveBeenCalled();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledTimes(
      1,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Retry recovery review" }),
    );

    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledTimes(
      2,
    );
  });

  it("continues after a stale acknowledgement refresh finds no active notice", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Startup restored generation 13 from validated state.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    workspaceMocks.readWorkspaceStartSnapshot
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 13,
        authority: "rust-core",
        activeRecoveryNotice: recoveryNotice,
        recents: [],
      })
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 14,
        authority: "rust-core",
        activeRecoveryNotice: null,
        recents: [],
      });
    workspaceMocks.acknowledgeWorkspaceRecovery.mockRejectedValueOnce(
      new ProtocolBoundaryError(
        "staleGeneration",
        "Recovery state changed.",
        true,
      ),
    );
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Continue to Chat" }),
    );

    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.readWorkspaceStartSnapshot).toHaveBeenCalledTimes(2);
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledTimes(
      1,
    );
  });

  it("refreshes and retries the exact same recovery identity after a stale acknowledgement", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Startup restored generation 13 from validated state.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    const refreshedNotice = {
      ...recoveryNotice,
      correlationId: "correlation:recovery-13-refreshed",
      summary: "The same recovered generations still require review.",
    };
    workspaceMocks.readWorkspaceStartSnapshot
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 13,
        authority: "rust-core",
        activeRecoveryNotice: recoveryNotice,
        recents: [],
      })
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 14,
        authority: "rust-core",
        activeRecoveryNotice: refreshedNotice,
        recents: [],
      });
    workspaceMocks.acknowledgeWorkspaceRecovery
      .mockRejectedValueOnce(
        new ProtocolBoundaryError(
          "invalidGeneration",
          "Recovery state changed.",
          true,
        ),
      )
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 15,
        authority: "rust-core",
        activeRecoveryNotice: null,
        recents: [],
      });
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Continue to Chat" }),
    );

    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenNthCalledWith(
      2,
      refreshedNotice,
    );
  });

  it("replaces a stale recovery notice and requires an exact new Continue", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-activated:13:12",
      correlationId: "correlation:recovery-13",
      workspaceId: "workspace:activated",
      workspaceName: "Activated Workspace",
      summary: "Startup restored generation 13 from validated state.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 13,
      archiveGeneration: 12,
      mustNotifyBeforeNextSave: true,
    };
    const replacement = {
      ...recoveryNotice,
      recoveryId: "recovery:workspace-activated:14:13",
      correlationId: "correlation:recovery-14",
      summary: "A newer recovery replaced the notice under review.",
      workingGeneration: 14,
      archiveGeneration: 13,
    };
    workspaceMocks.readWorkspaceStartSnapshot
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 13,
        authority: "rust-core",
        activeRecoveryNotice: recoveryNotice,
        recents: [],
      })
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 14,
        authority: "rust-core",
        activeRecoveryNotice: replacement,
        recents: [],
      });
    workspaceMocks.acknowledgeWorkspaceRecovery
      .mockRejectedValueOnce(
        new ProtocolBoundaryError(
          "conflict",
          "Recovery identity changed.",
          true,
        ),
      )
      .mockResolvedValueOnce({
        protocolVersion: 1,
        generation: 15,
        authority: "rust-core",
        activeRecoveryNotice: null,
        recents: [],
      });
    conversationMocks.readConversationSnapshot.mockResolvedValue(
      conversationSnapshot(),
    );
    const router = createMemoryRouter(
      [
        { path: "/start", element: <WorkspaceStartRoute /> },
        { path: "/chat", element: <main>Hydrated Chat</main> },
      ],
      { initialEntries: ["/start"] },
    );
    render(
      <Provider store={createAppStore(undefined)}>
        <RouterProvider router={router} />
      </Provider>,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Continue to Chat" }),
    );

    expect(await screen.findByText(replacement.summary)).toBeVisible();
    expect(screen.queryByText("Hydrated Chat")).not.toBeInTheDocument();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenCalledTimes(
      1,
    );
    fireEvent.click(screen.getByRole("button", { name: "Continue to Chat" }));

    expect(await screen.findByText("Hydrated Chat")).toBeVisible();
    expect(workspaceMocks.acknowledgeWorkspaceRecovery).toHaveBeenNthCalledWith(
      2,
      replacement,
    );
  });
});
