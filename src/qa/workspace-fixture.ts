import type {
  RecentWorkspace,
  WorkspaceOpenResult,
  WorkspaceStartAction,
} from "../features/workspace/WorkspaceStartScreen";
import type { CorrelationId, WorkspaceId } from "../platform/protocol";
import type { WorkspaceRecoveryNotice } from "../platform/workspace-start";

export const QA_RECENT_WORKSPACES: readonly RecentWorkspace[] = [
  {
    id: "workspace-qa-0001",
    name: "AI Desktop UI",
    detail: "3 Projects · opened today",
  },
  {
    id: "workspace-qa-0002",
    name: "Quotable AI",
    detail: "2 Projects · opened yesterday",
  },
  {
    id: "workspace-qa-0003",
    name: "Legacy UI",
    detail: "1 Project · recovery available",
    isMissing: true,
  },
];

export const QA_WORKSPACE_RECOVERY_NOTICE: WorkspaceRecoveryNotice = {
  recoveryId: "recovery:workspace-qa-0003:12:11",
  correlationId: "correlation:workspace-qa-0003" as CorrelationId,
  workspaceId: "workspace-qa-0003" as WorkspaceId,
  workspaceName: "Legacy UI",
  summary:
    "The working copy is newer than its saved archive and requires review.",
  action: "review_recovered_workspace_before_save",
  workingGeneration: 12,
  archiveGeneration: 11,
  mustNotifyBeforeNextSave: true,
};

let activeRecoveryNotice: WorkspaceRecoveryNotice | null = null;

export function openQaWorkspace(
  action: WorkspaceStartAction,
): Promise<WorkspaceOpenResult> {
  const recent =
    action.type === "openRecent"
      ? QA_RECENT_WORKSPACES.find((item) => item.id === action.workspaceId)
      : undefined;
  const recovered = recent?.id === "workspace-qa-0003";
  activeRecoveryNotice = recovered ? QA_WORKSPACE_RECOVERY_NOTICE : null;
  return Promise.resolve({
    workspaceName: recent?.name ?? "Untitled Workspace",
    recovered,
    recoveryNotice: activeRecoveryNotice,
  });
}

export function continueQaWorkspaceRecovery(): Promise<WorkspaceOpenResult> {
  if (activeRecoveryNotice === null) {
    return Promise.reject(new Error("No QA Workspace recovery is pending."));
  }
  const workspaceName = activeRecoveryNotice.workspaceName;
  activeRecoveryNotice = null;
  return Promise.resolve({
    workspaceName,
    recovered: true,
    recoveryNotice: null,
  });
}
