import { useEffect, useState } from "react";

import { readWorkspaceStartSnapshot } from "../../platform/workspace-start";
import {
  WorkspaceStartScreen,
  type RecentWorkspace,
  type WorkspaceOpenResult,
  type WorkspaceStartAction,
} from "./WorkspaceStartScreen";

type StartRouteState =
  | { readonly status: "loading" }
  | { readonly status: "ready"; readonly recents: readonly RecentWorkspace[] }
  | { readonly status: "error" };

export function WorkspaceStartRoute() {
  const [state, setState] = useState<StartRouteState>({ status: "loading" });

  useEffect(() => {
    let active = true;
    void readWorkspaceStartSnapshot().then(
      (snapshot) => {
        if (active) {
          setState({
            status: "ready",
            recents: snapshot.recents.map((recent) => ({
              id: recent.workspaceId,
              name: recent.displayName,
              detail: recent.isMissing
                ? "Saved archive needs to be located"
                : formatLastOpened(recent.lastOpenedAt),
              isMissing: recent.isMissing,
            })),
          });
        }
      },
      () => {
        if (active) setState({ status: "error" });
      },
    );
    return () => {
      active = false;
    };
  }, []);

  if (state.status === "loading") {
    return <WorkspaceStartStatus message="Loading recent Workspaces…" />;
  }
  if (state.status === "error") {
    return (
      <WorkspaceStartStatus message="Workspace Start is unavailable. Restart C4OS and try again." />
    );
  }
  return (
    <WorkspaceStartScreen
      recents={state.recents}
      openWorkspace={unavailablePlatformAction}
    />
  );
}

function WorkspaceStartStatus({ message }: { readonly message: string }) {
  return (
    <main className="workspace-start" aria-labelledby="workspace-start-title">
      <section className="workspace-start__card workspace-start__card--status">
        <div className="workspace-start__brand" aria-hidden="true">
          C4
        </div>
        <h1 id="workspace-start-title">Workspace Start</h1>
        <p role="status">{message}</p>
      </section>
    </main>
  );
}

function unavailablePlatformAction(
  action: WorkspaceStartAction,
): Promise<WorkspaceOpenResult> {
  // Native picker/clone effects are installed by the PlatformService and
  // Action Gateway tasks. Until then the production route fails closed.
  return Promise.reject(
    new Error(`platform action unavailable: ${action.type}`),
  );
}

function formatLastOpened(unixSeconds: number): string {
  const date = new Date(unixSeconds * 1_000);
  if (Number.isNaN(date.valueOf())) return "Previously opened";
  return `Last opened ${new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).format(date)}`;
}
