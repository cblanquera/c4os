import type {
  RecentWorkspace,
  WorkspaceStartAction,
} from "../features/workspace/WorkspaceStartScreen";

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

export function openQaWorkspace(action: WorkspaceStartAction) {
  const recent =
    action.type === "openRecent"
      ? QA_RECENT_WORKSPACES.find((item) => item.id === action.workspaceId)
      : undefined;
  return Promise.resolve({
    workspaceName: recent?.name ?? "Untitled Workspace",
    recovered: recent?.id === "workspace-qa-0003",
  });
}
