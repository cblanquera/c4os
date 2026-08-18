import { useEffect, useRef, useState } from "react";
import { Button } from "react-aria-components";

import { BrandMark, ControlledModalDialog } from "../../components/accessible";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";
import type { WorkspaceRecoveryNotice } from "../../platform/workspace-start";

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
  readonly recoveryNotice?: WorkspaceRecoveryNotice | null;
  readonly hydrationRequired?: boolean;
}

export interface WorkspaceCloneApprovalRequest {
  readonly promptId: string;
  readonly summary: string;
}

interface WorkspaceStartScreenProps {
  readonly initialRecovery?: WorkspaceOpenResult | null;
  readonly recents: readonly RecentWorkspace[];
  readonly openWorkspace: (
    action: WorkspaceStartAction,
  ) => Promise<WorkspaceOpenResult | WorkspaceCloneApprovalRequest | null>;
  readonly continueRecovery?: () => Promise<WorkspaceOpenResult>;
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
  | {
      readonly status: "error";
      readonly message: string;
      readonly retryRecovery?: {
        readonly action: WorkspaceStartAction;
        readonly label: string;
      };
    };

const primaryActions = [
  {
    type: "openFolder",
    label: "Open a folder",
    description: "Use a local folder as the project.",
    icon: "folder",
  },
  {
    type: "openWorkspace",
    label: "Open a workspace",
    description: "Open a saved C4OS workspace.",
    icon: "workspace",
  },
  {
    type: "cloneRepository",
    label: "Clone Repository",
    description: "Clone from a Git repository URL.",
    icon: "clone",
  },
] as const;

const workspaceActivationBoundaries: Readonly<Record<string, string>> = {
  "active-workspace-lock": "Workspace activation",
  "app-configuration-lock": "app configuration restoration",
  "app-configuration-read": "app configuration restoration",
  "app-configuration-resolution": "app configuration restoration",
  "chat-selection-restore": "Chat selection restoration",
  "conversation-publication": "Chat hydration publication",
  "conversation-restore": "Chat restoration",
  "conversation-state-lock": "Chat activation",
  "policy-transition-lock": "policy preparation",
  "picker-display-name": "Project selection validation",
  "picker-grant": "Project selection validation",
  "project-selection-restore": "Project selection restoration",
  "request-validation": "Workspace request validation",
  "runtime-activation": "runtime activation",
  "runtime-binding": "runtime binding",
  "runtime-bootstrap": "runtime bootstrap",
  "runtime-installations": "runtime installation discovery",
  "runtime-policy": "runtime policy restoration",
  "runtime-publication": "runtime publication",
  "runtime-selection": "runtime selection",
  "runtime-snapshot": "runtime state restoration",
  "workspace-configuration-observer": "Workspace configuration observation",
  "workspace-configuration-resolution": "Workspace configuration restoration",
  "workspace-clock": "Workspace preparation",
  "workspace-create": "Workspace preparation",
  "workspace-created-at": "Workspace preparation",
  "workspace-generation": "Workspace state synchronization",
  "workspace-identity": "Workspace identity restoration",
  "workspace-operation-lock": "Workspace operation preparation",
  "workspace-publication-floor": "Workspace publication synchronization",
  "workspace-query": "Workspace state lookup",
  "workspace-recovery-notice": "Workspace recovery restoration",
  "workspace-retry": "Workspace retry recovery",
  "workspace-snapshot": "Workspace state restoration",
};

export function WorkspaceStartScreen({
  initialRecovery = null,
  recents,
  openWorkspace,
  continueRecovery,
  answerCloneApproval,
}: WorkspaceStartScreenProps) {
  const [openState, setOpenState] = useState<OpenState>(() =>
    initialRecovery?.recoveryNotice == null
      ? { status: "idle" }
      : {
          status: "opened",
          result: initialRecovery,
          action: {
            type: "openRecent",
            workspaceId: initialRecovery.recoveryNotice.workspaceId,
          },
        },
  );
  const [showCloneForm, setShowCloneForm] = useState(false);
  const [repositoryUrl, setRepositoryUrl] = useState("");
  const requestGeneration = useRef(0);
  const operationInFlight = useRef(false);
  const cloneSubmit = useRef<HTMLButtonElement>(null);
  const recoveryNotice = useRef<HTMLDivElement>(null);
  const restoreCloneFocus = useRef(false);
  const recoveryReviewPending =
    (openState.status === "opened" &&
      !openState.result.hydrationRequired &&
      openState.result.recoveryNotice != null) ||
    (openState.status === "error" && openState.retryRecovery !== undefined);
  const busy =
    openState.status === "opening" ||
    openState.status === "approval" ||
    recoveryReviewPending;

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

  const beginRecoveryReview = (action: WorkspaceStartAction, label: string) => {
    if (operationInFlight.current) return;
    operationInFlight.current = true;
    const generation = ++requestGeneration.current;
    setOpenState({ status: "opening", label });
    const review =
      continueRecovery === undefined
        ? Promise.reject(
            new Error("Recovery review is unavailable in this surface."),
          )
        : continueRecovery();
    void review.then(
      (result) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState({ status: "opened", result, action });
        }
      },
      (error: unknown) => {
        operationInFlight.current = false;
        if (requestGeneration.current === generation) {
          setOpenState({
            status: "error",
            message: workspaceOpenFailure(error, true),
            retryRecovery: { action, label },
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

  useEffect(() => {
    if (
      openState.status === "opened" &&
      !openState.result.hydrationRequired &&
      openState.result.recoveryNotice != null
    ) {
      window.queueMicrotask(() => recoveryNotice.current?.focus());
    }
  }, [openState]);

  const recoveryRetry =
    openState.status === "error" ? openState.retryRecovery : undefined;

  return (
    <main className="workspace-start" aria-labelledby="workspace-start-title">
      <section className="workspace-start__card">
        <header className="workspace-start__bar">
          <div className="workspace-start__brand" aria-label="C4OS">
            <BrandMark size={28} />
            <strong>C4OS</strong>
          </div>
        </header>

        <div className="workspace-start__content">
          <header className="workspace-start__intro">
            <p>Workspace</p>
            <h1 id="workspace-start-title">What would you like to open?</h1>
            <span>Start something new or continue where you left off.</span>
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
                <span
                  className="workspace-start__action-mark"
                  aria-hidden="true"
                >
                  <WorkspaceStartIcon name={action.icon} />
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
            <h2 id="recent-workspaces">Recent workspaces</h2>
            {recents.length === 0 ? (
              <p className="workspace-start__empty">
                No recent Workspaces yet.
              </p>
            ) : (
              <ul>
                {recents.slice(0, 3).map((recent) => (
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
                        <WorkspaceStartIcon name="folder" />
                      </span>
                      <span>
                        <strong>{recent.name}</strong>
                        <small>{recent.detail}</small>
                      </span>
                      <span className="workspace-start__recent-state">
                        {recent.isMissing ? "Locate" : "Open"}
                      </span>
                      <WorkspaceStartIcon
                        className="workspace-start__recent-chevron"
                        name="chevron"
                      />
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </section>

          <div
            className={`workspace-start__notice workspace-start__notice--${openState.status}`}
            aria-atomic="true"
            aria-live="polite"
            ref={recoveryNotice}
            role={openState.status === "error" ? "alert" : "status"}
            tabIndex={-1}
          >
            {openState.status === "opening" && `Opening ${openState.label}…`}
            {openState.status === "opened" &&
              (openState.result.hydrationRequired ? (
                `${openState.result.workspaceName} is active, but Chat still needs to be refreshed.`
              ) : openState.result.recoveryNotice != null ? (
                <>
                  <strong>
                    Recovered {openState.result.workspaceName}. Review before
                    the next save.
                  </strong>
                  <span>{openState.result.recoveryNotice.summary}</span>
                  <small>
                    Working generation{" "}
                    {openState.result.recoveryNotice.workingGeneration}; archive
                    generation{" "}
                    {openState.result.recoveryNotice.archiveGeneration};
                    correlation {openState.result.recoveryNotice.correlationId}.
                  </small>
                </>
              ) : (
                `Opened ${openState.result.workspaceName}. Entering Chat…`
              ))}
            {openState.status === "approval" && "Clone approval required."}
            {openState.status === "error" && openState.message}
          </div>
          {openState.status === "opened" &&
          openState.result.hydrationRequired ? (
            <Button
              onPress={() =>
                beginOpen(openState.action, "the active Workspace")
              }
            >
              Retry Chat recovery
            </Button>
          ) : openState.status === "opened" &&
            openState.result.recoveryNotice != null ? (
            <Button
              onPress={() =>
                beginRecoveryReview(openState.action, "the active Workspace")
              }
            >
              Continue to Chat
            </Button>
          ) : recoveryRetry !== undefined ? (
            <Button
              onPress={() =>
                beginRecoveryReview(recoveryRetry.action, recoveryRetry.label)
              }
            >
              Retry recovery review
            </Button>
          ) : null}
        </div>
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

function WorkspaceStartIcon({
  className,
  name,
}: {
  readonly className?: string;
  readonly name: "chevron" | "clone" | "folder" | "workspace";
}) {
  const paths = {
    chevron: <path d="m9 18 6-6-6-6" />,
    clone: (
      <>
        <circle cx="6" cy="5" r="2" />
        <circle cx="18" cy="7" r="2" />
        <circle cx="6" cy="19" r="2" />
        <path d="M6 7v10M8 5h4a6 6 0 0 1 6 6V9" />
      </>
    ),
    folder: <path d="M3 7h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />,
    workspace: (
      <>
        <rect x="3" y="4" width="18" height="16" rx="2" />
        <path d="M8 4v16M8 9h13" />
      </>
    ),
  } as const;
  return (
    <svg
      aria-hidden="true"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
      viewBox="0 0 24 24"
    >
      {paths[name]}
    </svg>
  );
}

/** Returns a bounded recovery message without exposing native failure details. */
function workspaceOpenFailure(error: unknown, recoveryReview = false): string {
  if (recoveryReview) {
    return "C4OS could not record that recovery review. Chat remains blocked and your existing state is unchanged.";
  }

  const stage =
    error instanceof ProtocolBoundaryError &&
    error.details.stage?.kind === "text"
      ? error.details.stage.value
      : null;
  const boundary =
    stage === null ? undefined : workspaceActivationBoundaries[stage];
  if (boundary !== undefined) {
    return `C4OS could not open that Workspace during ${boundary}. Your Workspace state was preserved. Retry the original Project folder.`;
  }

  if (error instanceof ProtocolBoundaryError) {
    return `${error.message} Your existing Workspace state was preserved.`;
  }

  return "C4OS could not open that Workspace. Your existing state is unchanged.";
}
