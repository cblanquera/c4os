import { describe, expect, it } from "vitest";

import type {
  CorrelationId,
  PickerGrantId,
  RequestId,
} from "../platform/protocol";
import {
  createWorkspaceStartAdapter,
  type WorkspaceStartTransport,
} from "../platform/workspace-start";
import { invokeQaProductRoute } from "./product-route-native";
import {
  continueQaWorkspaceRecovery,
  openQaWorkspace,
  QA_WORKSPACE_RECOVERY_NOTICE,
} from "./workspace-fixture";

describe("QA Workspace recovery fixtures", () => {
  it("clears direct recovery only through explicit Continue", async () => {
    const opened = await openQaWorkspace({
      type: "openRecent",
      workspaceId: QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
    });
    expect(opened.recoveryNotice).toEqual(QA_WORKSPACE_RECOVERY_NOTICE);

    await expect(continueQaWorkspaceRecovery()).resolves.toMatchObject({
      workspaceName: "Legacy UI",
      recovered: true,
      recoveryNotice: null,
    });
    await expect(continueQaWorkspaceRecovery()).rejects.toThrow(
      /No QA Workspace recovery is pending/,
    );
  });

  it("implements the complete production acknowledgement protocol", async () => {
    const transport: WorkspaceStartTransport = {
      async invoke(command, args) {
        const result = invokeQaProductRoute(command, args);
        if (result === null)
          throw new Error(`Unsupported QA command ${command}`);
        return result;
      },
    };
    const adapter = createWorkspaceStartAdapter(transport, {
      requestIdFactory: () => "request:qa-workspace" as RequestId,
      correlationIdFactory: () => "correlation:qa-workspace" as CorrelationId,
    });

    await expect(adapter.readSnapshot()).resolves.toMatchObject({
      generation: 51,
      activeRecoveryNotice: null,
    });
    const opened = await adapter.openArchive(
      "picker-grant:legacy-workspace" as PickerGrantId,
    );
    expect(opened).toMatchObject({
      workspaceId: "workspace-qa-0003",
      workspaceName: "Legacy UI",
      recovered: true,
      recoveryNotice: QA_WORKSPACE_RECOVERY_NOTICE,
    });

    await expect(
      adapter.acknowledgeRecovery(opened.recoveryNotice!),
    ).resolves.toMatchObject({
      generation: 53,
      activeRecoveryNotice: null,
    });
  });
});
