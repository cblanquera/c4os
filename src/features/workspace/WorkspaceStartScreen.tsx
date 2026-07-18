import { useRef, useState } from "react";
import { Button } from "react-aria-components";

export type WorkspaceStartAction =
  | { readonly type: "openFolder" }
  | { readonly type: "openWorkspace" }
  | { readonly type: "cloneRepository" }
  | { readonly type: "openRecent"; readonly workspaceId: string };

export interface RecentWorkspace {
  readonly id: string;
  readonly name: string;
  readonly detail: string;
  readonly isMissing?: boolean;
}

export interface WorkspaceOpenResult {
  readonly workspaceName: string;
  readonly recovered: boolean;
}

interface WorkspaceStartScreenProps {
  readonly recents: readonly RecentWorkspace[];
  readonly openWorkspace: (
    action: WorkspaceStartAction,
  ) => Promise<WorkspaceOpenResult>;
}

type OpenState =
  | { readonly status: "idle" }
  | { readonly status: "opening"; readonly label: string }
  | { readonly status: "opened"; readonly result: WorkspaceOpenResult }
  | { readonly status: "error"; readonly message: string };

const primaryActions = [
  {
    type: "openFolder",
    label: "Open a folder",
    description: "Use a local folder as the project.",
    mark: "F",
  },
  {
    type: "openWorkspace",
    label: "Open a workspace",
    description: "Open a saved C4OS workspace.",
    mark: "W",
  },
  {
    type: "cloneRepository",
    label: "Clone Repository",
    description: "Clone from a Git repository URL.",
    mark: "G",
  },
] as const;

export function WorkspaceStartScreen({
  recents,
  openWorkspace,
}: WorkspaceStartScreenProps) {
  const [openState, setOpenState] = useState<OpenState>({ status: "idle" });
  const requestGeneration = useRef(0);
  const busy = openState.status === "opening";

  const beginOpen = (action: WorkspaceStartAction, label: string) => {
    const generation = ++requestGeneration.current;
    setOpenState({ status: "opening", label });
    void openWorkspace(action).then(
      (result) => {
        if (requestGeneration.current === generation) {
          setOpenState({ status: "opened", result });
        }
      },
      () => {
        if (requestGeneration.current === generation) {
          setOpenState({
            status: "error",
            message:
              "C4OS could not open that Workspace. Your existing state is unchanged.",
          });
        }
      },
    );
  };

  return (
    <main className="workspace-start" aria-labelledby="workspace-start-title">
      <section className="workspace-start__card">
        <header className="workspace-start__header">
          <div className="workspace-start__brand" aria-hidden="true">
            C4
          </div>
          <div>
            <h1 id="workspace-start-title">What would you like to open?</h1>
            <p>Start something new or continue where you left off.</p>
          </div>
        </header>

        <div
          className="workspace-start__actions"
          aria-label="Open options"
          role="group"
        >
          {primaryActions.map((action) => (
            <Button
              className="workspace-start__action"
              isDisabled={busy}
              key={action.type}
              onPress={() => beginOpen({ type: action.type }, action.label)}
            >
              <span className="workspace-start__action-mark" aria-hidden="true">
                {action.mark}
              </span>
              <span className="workspace-start__action-copy">
                <strong>{action.label}</strong>
                <span>{action.description}</span>
              </span>
            </Button>
          ))}
        </div>

        <section
          className="workspace-start__recents"
          aria-labelledby="recent-workspaces"
        >
          <h2 id="recent-workspaces">Recent Workspaces</h2>
          {recents.length === 0 ? (
            <p className="workspace-start__empty">No recent Workspaces yet.</p>
          ) : (
            <ul>
              {recents.map((recent) => (
                <li key={recent.id}>
                  <Button
                    className="workspace-start__recent"
                    isDisabled={busy}
                    onPress={() =>
                      beginOpen(
                        { type: "openRecent", workspaceId: recent.id },
                        recent.name,
                      )
                    }
                  >
                    <span
                      className="workspace-start__recent-icon"
                      aria-hidden="true"
                    >
                      {recent.name.slice(0, 1).toUpperCase()}
                    </span>
                    <span>
                      <strong>{recent.name}</strong>
                      <small>{recent.detail}</small>
                    </span>
                    <span className="workspace-start__recent-state">
                      {recent.isMissing ? "Locate" : "Open"}
                    </span>
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <div
          className={`workspace-start__notice workspace-start__notice--${openState.status}`}
          aria-live="polite"
          role="status"
        >
          {openState.status === "opening" && `Opening ${openState.label}…`}
          {openState.status === "opened" &&
            (openState.result.recovered
              ? `Recovered ${openState.result.workspaceName}. Review the recovery notice before the next save.`
              : `Opened ${openState.result.workspaceName}. Entering Chat…`)}
          {openState.status === "error" && openState.message}
        </div>
      </section>
    </main>
  );
}
