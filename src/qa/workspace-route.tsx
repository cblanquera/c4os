import { WorkspaceStartScreen } from "../features/workspace/WorkspaceStartScreen";
import { isQaFixtureBuildEnabled } from "./fixture";
import {
  continueQaWorkspaceRecovery,
  openQaWorkspace,
  QA_RECENT_WORKSPACES,
} from "./workspace-fixture";

export const QA_WORKSPACE_PATH = "/qa/workspace";

export function QaWorkspaceRoute() {
  if (!isQaFixtureBuildEnabled()) {
    return (
      <main className="qa-foundation" data-qa-fixture-mode="disabled">
        <h1>C4OS QA Workspace</h1>
        <p role="status">Fixture mode unavailable</p>
      </main>
    );
  }

  return (
    <div data-qa-fixture-mode="enabled">
      <WorkspaceStartScreen
        continueRecovery={continueQaWorkspaceRecovery}
        recents={QA_RECENT_WORKSPACES}
        openWorkspace={openQaWorkspace}
      />
    </div>
  );
}
