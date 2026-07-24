import { describe, expect, it, vi } from "vitest";

import {
  PROTOCOL_VERSION,
  type CorrelationId,
  type PickerGrantId,
  type RequestId,
  type StateGeneration,
  type WorkspaceId,
} from "../../../src/frontend/platform/protocol";
import {
  createWorkspaceStartAdapter,
  WORKSPACE_RECOVERY_ACKNOWLEDGE_COMMAND,
  WORKSPACE_START_CLONE_REPOSITORY_COMMAND,
  WORKSPACE_START_OPEN_RECENT_COMMAND,
  WORKSPACE_START_SNAPSHOT_COMMAND,
  type WorkspaceStartTransport,
} from "../../../src/frontend/platform/workspace-start";

const requestId = "request-workspace-1" as RequestId;
const correlationId = "corr-workspace-1" as CorrelationId;

function response(overrides: Record<string, unknown> = {}) {
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId,
    correlationId,
    generation: 4,
    payload: {
      protocolVersion: PROTOCOL_VERSION,
      generation: 4,
      authority: "rust-core",
      activeRecoveryNotice: null,
      recents: [
        {
          workspaceId: "workspace-1",
          displayName: "First Workspace",
          lastOpenedAt: 1_721_312_000,
          isMissing: false,
        },
      ],
    },
    ...overrides,
  };
}

function openResponse(overrides: Record<string, unknown> = {}) {
  return response({
    payload: {
      authority: "rust-workspace-service",
      workspaceId: "workspace-opened",
      workspaceName: "Opened Workspace",
      recovered: false,
      recoveryNotice: null,
    },
    ...overrides,
  });
}

function adapter(raw: unknown, initialGeneration = 3) {
  const transport: WorkspaceStartTransport = {
    invoke: vi.fn().mockResolvedValue(raw),
  };
  return {
    transport,
    adapter: createWorkspaceStartAdapter(transport, {
      requestIdFactory: () => requestId,
      correlationIdFactory: () => correlationId,
      initialGeneration: initialGeneration as StateGeneration,
    }),
  };
}

describe("createWorkspaceStartAdapter", () => {
  it("uses only the allowlisted command and advances its validated cursor", async () => {
    const fixture = adapter(response());

    await expect(fixture.adapter.readSnapshot()).resolves.toMatchObject({
      authority: "rust-core",
      generation: 4,
      recents: [{ displayName: "First Workspace" }],
    });
    expect(fixture.adapter.currentGeneration).toBe(4);
    expect(fixture.transport.invoke).toHaveBeenCalledWith(
      WORKSPACE_START_SNAPSHOT_COMMAND,
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 3,
        },
      },
    );
  });

  it("rejects stale, mismatched, unknown, and oversized responses", async () => {
    await expect(
      adapter(response(), 5).adapter.readSnapshot(),
    ).rejects.toMatchObject({
      code: "staleGeneration",
    });
    await expect(
      adapter(response({ correlationId: "corr-other" })).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "correlationMismatch" });
    await expect(
      adapter(response({ protocolVersion: 2 })).adapter.readSnapshot(),
    ).rejects.toMatchObject({ code: "unknownProtocolVersion" });

    const raw = response();
    raw.payload.recents = Array.from({ length: 4 }, (_, index) => ({
      workspaceId: `workspace-${index}`,
      displayName: `Workspace ${index}`,
      lastOpenedAt: index,
      isMissing: false,
    }));
    await expect(adapter(raw).adapter.readSnapshot()).rejects.toMatchObject({
      code: "invalidPayload",
    });
  });

  it("maps structured failures and hides arbitrary transport failures", async () => {
    const structured: WorkspaceStartTransport = {
      invoke: vi.fn().mockRejectedValue({
        code: "unavailable",
        message: "Workspace state is unavailable",
        retryable: true,
        correlationId,
        details: {},
      }),
    };
    await expect(
      createWorkspaceStartAdapter(structured, {
        requestIdFactory: () => requestId,
        correlationIdFactory: () => correlationId,
      }).readSnapshot(),
    ).rejects.toMatchObject({
      code: "unavailable",
      message: "Workspace state is unavailable",
    });

    const arbitrary: WorkspaceStartTransport = {
      invoke: vi.fn().mockRejectedValue(new Error("/secret/user/path")),
    };
    await expect(
      createWorkspaceStartAdapter(arbitrary, {
        requestIdFactory: () => requestId,
        correlationIdFactory: () => correlationId,
      }).readSnapshot(),
    ).rejects.toMatchObject({
      code: "unavailable",
      message: "Workspace Start is unavailable.",
    });
  });

  it("opens a recent Workspace with the exact CAS request and advances the cursor", async () => {
    const fixture = adapter(openResponse());

    await expect(
      fixture.adapter.openRecent("workspace-1" as WorkspaceId),
    ).resolves.toEqual({
      authority: "rust-workspace-service",
      workspaceId: "workspace-opened",
      workspaceName: "Opened Workspace",
      recovered: false,
      recoveryNotice: null,
    });
    expect(fixture.adapter.currentGeneration).toBe(4);
    expect(fixture.transport.invoke).toHaveBeenCalledWith(
      WORKSPACE_START_OPEN_RECENT_COMMAND,
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 3,
        },
        input: { workspaceId: "workspace-1" },
      },
    );
  });

  it("clones only through the native command and validates its bounded result", async () => {
    const fixture = adapter(
      openResponse({
        payload: {
          state: "pendingApproval",
          promptId: "clone-approval-1",
          summary: "Clone the repository into the selected folder?",
        },
      }),
    );

    await expect(
      fixture.adapter.cloneRepository(
        "picker-grant-1" as PickerGrantId,
        "https://github.com/example/repository.git",
      ),
    ).resolves.toEqual({
      state: "pendingApproval",
      promptId: "clone-approval-1",
      summary: "Clone the repository into the selected folder?",
    });
    expect(fixture.transport.invoke).toHaveBeenCalledWith(
      WORKSPACE_START_CLONE_REPOSITORY_COMMAND,
      expect.objectContaining({
        input: {
          pickerGrantId: "picker-grant-1",
          repositoryUrl: "https://github.com/example/repository.git",
        },
      }),
    );

    await expect(
      adapter(
        openResponse({ payload: { authority: "renderer" } }),
      ).adapter.openRecent("workspace-1" as WorkspaceId),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    expect(() =>
      fixture.adapter.cloneRepository(
        "picker-grant-1" as PickerGrantId,
        "https://example.com/repo\nname",
      ),
    ).toThrowError(/repository URL is invalid/);
  });

  it("validates a structured recovery notice and rejects mismatched recovery authority", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-opened:5:4",
      correlationId: "correlation:recovery-1" as CorrelationId,
      workspaceId: "workspace-opened",
      workspaceName: "Opened Workspace",
      summary: "Recovered the working generation after an interrupted save.",
      action: "review_recovered_workspace_before_save",
      workingGeneration: 5,
      archiveGeneration: 4,
      mustNotifyBeforeNextSave: true,
    };
    await expect(
      adapter(
        openResponse({
          payload: {
            authority: "rust-workspace-service",
            workspaceId: "workspace-opened",
            workspaceName: "Opened Workspace",
            recovered: true,
            recoveryNotice,
          },
        }),
      ).adapter.openRecent("workspace-1" as WorkspaceId),
    ).resolves.toMatchObject({
      recovered: true,
      recoveryNotice: {
        recoveryId: "recovery:workspace-opened:5:4",
        workingGeneration: 5,
        archiveGeneration: 4,
      },
    });

    await expect(
      adapter(
        openResponse({
          payload: {
            authority: "rust-workspace-service",
            workspaceId: "workspace-opened",
            workspaceName: "Opened Workspace",
            recovered: true,
            recoveryNotice: {
              ...recoveryNotice,
              workspaceId: "workspace:other",
            },
          },
        }),
      ).adapter.openRecent("workspace-1" as WorkspaceId),
    ).rejects.toMatchObject({ code: "invalidPayload" });
    await expect(
      adapter(
        openResponse({
          payload: {
            authority: "rust-workspace-service",
            workspaceId: "workspace-opened",
            workspaceName: "Opened Workspace",
            recovered: true,
            recoveryNotice: {
              ...recoveryNotice,
              action: "skip_recovery_review",
            },
          },
        }),
      ).adapter.openRecent("workspace-1" as WorkspaceId),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });

  it("acknowledges only the exact recovery identity and requires native suppression", async () => {
    const recoveryNotice = {
      recoveryId: "recovery:workspace-opened:5:4",
      correlationId: "correlation:recovery-1" as CorrelationId,
      workspaceId: "workspace-opened" as WorkspaceId,
      workspaceName: "Opened Workspace",
      summary: "Recovered the working generation after an interrupted save.",
      action: "review_recovered_workspace_before_save" as const,
      workingGeneration: 5,
      archiveGeneration: 4,
      mustNotifyBeforeNextSave: true,
    };
    const fixture = adapter(response());

    await expect(
      fixture.adapter.acknowledgeRecovery(recoveryNotice),
    ).resolves.toMatchObject({
      generation: 4,
      activeRecoveryNotice: null,
    });
    expect(fixture.transport.invoke).toHaveBeenCalledWith(
      WORKSPACE_RECOVERY_ACKNOWLEDGE_COMMAND,
      {
        request: {
          protocolVersion: PROTOCOL_VERSION,
          requestId,
          correlationId,
          expectedGeneration: 3,
        },
        input: {
          expectedGeneration: 3,
          recoveryId: "recovery:workspace-opened:5:4",
          workspaceId: "workspace-opened",
          workingGeneration: 5,
          archiveGeneration: 4,
        },
      },
    );

    const unsuppressed = response({
      payload: {
        protocolVersion: PROTOCOL_VERSION,
        generation: 4,
        authority: "rust-core",
        activeRecoveryNotice: recoveryNotice,
        recents: [],
      },
    });
    await expect(
      adapter(unsuppressed).adapter.acknowledgeRecovery(recoveryNotice),
    ).rejects.toMatchObject({ code: "invalidPayload" });
  });
});
