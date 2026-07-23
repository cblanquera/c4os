import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Provider } from "react-redux";
import { createMemoryRouter, RouterProvider } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { createAppStore } from "../../app/store";
import type { ConversationSnapshot } from "../../platform/conversation-service";
import { WorkspaceStartRoute } from "./WorkspaceStartRoute";

const conversationMocks = vi.hoisted(() => ({
  readConversationSnapshot: vi.fn(),
}));
const platformMocks = vi.hoisted(() => ({
  pickNative: vi.fn(),
}));
const workspaceMocks = vi.hoisted(() => ({
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
});
