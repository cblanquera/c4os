import { useEffect, useRef, useState } from "react";
import { Button } from "react-aria-components";

import { ControlledModalDialog } from "../../components/accessible";

export type WorkspaceStartAction =
  | { readonly type: "openFolder" }
  | { readonly type: "openWorkspace" }
  | { readonly type: "cloneRepository"; readonly repositoryUrl: string }
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
  readonly hydrationRequired?: boolean;
}

export interface WorkspaceCloneApprovalRequest {
  readonly promptId: string;
  readonly summary: string;
}

interface WorkspaceStartScreenProps {
  readonly recents: readonly RecentWorkspace[];
  readonly openWorkspace: (
    action: WorkspaceStartAction,
  ) => Promise<WorkspaceOpenResult | WorkspaceCloneApprovalRequest | null>;
  readonly answerCloneApproval?: (
    promptId: string,
    answer: "allow" | "deny",
  ) => Promise<WorkspaceOpenResult | null>;
}

type OpenState =
  | { readonly status: "idle" }
  | { readonly status: "opening"; readonly label: string }
  | {
      readonly status: "approval";
      readonly promptId: string;
      readonly summary: string;
    }
  | {
      readonly status: "opened";
      readonly result: WorkspaceOpenResult;
      readonly action: WorkspaceStartAction;
    }
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
  answerCloneApproval,
}: WorkspaceStartScreenProps) {
  const [openState, setOpenState] = useState<OpenState>({ status: "idle" });
  const [showCloneForm, setShowCloneForm] = useState(false);
  const [repositoryUrl, setRepositoryUrl] = useState("");
  const requestGeneration = useRef(0);
  const operationInFlight = useRef(false);
  const cloneSubmit = useRef<HTMLButtonElement>(null);
  const restoreCloneFocus = useRef(false);
  const busy =
    openState.status === "opening" || openState.status === "approval";

  const beginOpen = (action: WorkspaceStartAction, label: string) => {
    if (operationInFlight.current) return;
    operationInFlight.current = true;
    const generation = ++requestGeneration.current;
    setOpenState({ status: "opening", label });
    void openWorkspace(action).then(
      (result) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState(
            result === null
              ? { status: "idle" }
              : "promptId" in result
                ? {
                    status: "approval",
                    promptId: result.promptId,
                    summary: result.summary,
                  }
                : { status: "opened", result, action },
          );
        }
      },
      (error: unknown) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState({
            status: "error",
            message: workspaceOpenFailure(error),
          });
        }
      },
    );
  };

  const answerApproval = (answer: "allow" | "deny") => {
    if (openState.status !== "approval" || operationInFlight.current) return;
    restoreCloneFocus.current = true;
    if (answerCloneApproval === undefined) {
      setOpenState({
        status: "error",
        message: "Clone approval is unavailable in this surface.",
      });
      return;
    }
    const { promptId } = openState;
    const action: WorkspaceStartAction = {
      type: "cloneRepository",
      repositoryUrl,
    };
    operationInFlight.current = true;
    const generation = ++requestGeneration.current;
    setOpenState({ status: "opening", label: "Clone Repository" });
    void answerCloneApproval(promptId, answer).then(
      (result) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState(
            result === null
              ? { status: "idle" }
              : { status: "opened", result, action },
          );
        }
      },
      (error: unknown) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState({
            status: "error",
            message: workspaceOpenFailure(error),
          });
        }
      },
    );
  };

  useEffect(() => {
    if (
      restoreCloneFocus.current &&
      openState.status !== "approval" &&
      openState.status !== "opening"
    ) {
      restoreCloneFocus.current = false;
      window.queueMicrotask(() => cloneSubmit.current?.focus());
    }
  }, [openState.status]);

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
              onPress={() => {
                if (action.type === "cloneRepository") {
                  setShowCloneForm((visible) => !visible);
                  return;
                }
                beginOpen({ type: action.type }, action.label);
              }}
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

        {showCloneForm && (
          <form
            className="workspace-start__clone"
            onSubmit={(event) => {
              event.preventDefault();
              beginOpen(
                { type: "cloneRepository", repositoryUrl },
                "Clone Repository",
              );
            }}
          >
            <label htmlFor="workspace-start-repository-url">
              Repository URL
            </label>
            <div className="workspace-start__clone-controls">
              <input
                id="workspace-start-repository-url"
                disabled={busy}
                onChange={(event) =>
                  setRepositoryUrl(event.currentTarget.value)
                }
                placeholder="https://github.com/owner/repository.git"
                required
                type="url"
                value={repositoryUrl}
              />
              <Button isDisabled={busy} ref={cloneSubmit} type="submit">
                Clone
              </Button>
            </div>
            <p>C4OS will ask where to create the cloned repository.</p>
          </form>
        )}

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
            (openState.result.hydrationRequired
              ? `${openState.result.workspaceName} is active, but Chat still needs to be refreshed.`
              : openState.result.recovered
                ? `Recovered ${openState.result.workspaceName}. Review the recovery notice before the next save.`
                : `Opened ${openState.result.workspaceName}. Entering Chat…`)}
          {openState.status === "approval" && "Clone approval required."}
          {openState.status === "error" && openState.message}
        </div>
        {openState.status === "opened" && openState.result.hydrationRequired ? (
          <Button
            onPress={() => beginOpen(openState.action, "the active Workspace")}
          >
            Retry Chat recovery
          </Button>
        ) : null}
      </section>
      <ControlledModalDialog
        closeLabel="Cancel"
        isOpen={openState.status === "approval"}
        onDismiss={() => answerApproval("deny")}
        renderActions={() => (
          <>
            <Button onPress={() => answerApproval("deny")}>Cancel</Button>
            <Button onPress={() => answerApproval("allow")}>Allow clone</Button>
          </>
        )}
        title="Clone approval"
      >
        <p>{openState.status === "approval" ? openState.summary : ""}</p>
      </ControlledModalDialog>
    </main>
  );
}

function workspaceOpenFailure(error: unknown): string {
  void error;
  return "C4OS could not open that Workspace. Your existing state is unchanged.";
}
