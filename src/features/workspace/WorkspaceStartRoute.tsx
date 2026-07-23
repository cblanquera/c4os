import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router";

import { useAppDispatch } from "../../app/hooks";
import { readConversationSnapshot } from "../../platform/conversation-service";
import { pickNative } from "../../platform/platform-service";
import type { WorkspaceId } from "../../platform/protocol";
import {
  answerWorkspaceCloneApproval,
  cloneWorkspaceRepository,
  openRecentWorkspace,
  openWorkspaceArchive,
  openWorkspaceFolder,
  readWorkspaceStartSnapshot,
} from "../../platform/workspace-start";
import { publishConversationSnapshot } from "../shell/native-bootstrap";
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

type ActivatedWorkspace = {
  readonly workspaceId: string;
  readonly workspaceName: string;
  readonly recovered: boolean;
};

export function WorkspaceStartRoute() {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const [state, setState] = useState<StartRouteState>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);
  const pendingActivation = useRef<ActivatedWorkspace | null>(null);

  const hydrateActivatedWorkspace = async (
    result: ActivatedWorkspace,
  ): Promise<WorkspaceOpenResult> => {
    try {
      const conversation = await readConversationSnapshot();
      if (conversation.workspaceId !== result.workspaceId) {
        throw new Error(
          "The active Workspace and authoritative Chat snapshot do not match yet.",
        );
      }
      publishConversationSnapshot(dispatch, conversation, {
        reconcileDraft: true,
      });
      pendingActivation.current = null;
      void navigate("/chat", { replace: true });
      return {
        workspaceName: result.workspaceName,
        recovered: result.recovered,
      };
    } catch {
      pendingActivation.current = result;
      return {
        workspaceName: result.workspaceName,
        recovered: result.recovered,
        hydrationRequired: true,
      };
    }
  };

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
  }, [attempt]);

  if (state.status === "loading") {
    return <WorkspaceStartStatus message="Loading recent Workspaces…" />;
  }
  if (state.status === "error") {
    return (
      <WorkspaceStartStatus
        message="Workspace Start is unavailable. Retry before opening a Workspace."
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }
  const openWorkspace = async (
    action: WorkspaceStartAction,
  ): Promise<
    WorkspaceOpenResult | { promptId: string; summary: string } | null
  > => {
    if (pendingActivation.current !== null) {
      return hydrateActivatedWorkspace(pendingActivation.current);
    }
    const recent =
      action.type === "openRecent"
        ? state.recents.find(({ id }) => id === action.workspaceId)
        : undefined;
    let result;
    if (action.type === "openRecent" && !recent?.isMissing) {
      result = await openRecentWorkspace(action.workspaceId as WorkspaceId);
    } else {
      const purpose =
        action.type === "openFolder" || action.type === "cloneRepository"
          ? "openProjectFolder"
          : "openWorkspaceArchive";
      const outcome = await pickNative(purpose);
      if (outcome.type === "cancelled") return null;
      const grant = outcome.grants.at(0);
      if (grant === undefined) throw new Error("Picker selection is empty.");
      if (action.type === "openFolder") {
        result = await openWorkspaceFolder(grant.grantId);
      } else if (action.type === "cloneRepository") {
        const clone = await cloneWorkspaceRepository(
          grant.grantId,
          action.repositoryUrl,
        );
        if (clone.state === "pendingApproval") {
          return { promptId: clone.promptId, summary: clone.summary };
        }
        if (clone.state === "denied") return null;
        result = clone;
      } else {
        result = await openWorkspaceArchive(grant.grantId);
      }
    }
    pendingActivation.current = result;
    return hydrateActivatedWorkspace(result);
  };
  const answerCloneApproval = async (
    promptId: string,
    answer: "allow" | "deny",
  ): Promise<WorkspaceOpenResult | null> => {
    const result = await answerWorkspaceCloneApproval(promptId, answer);
    if (result.state !== "opened") return null;
    pendingActivation.current = result;
    return hydrateActivatedWorkspace(result);
  };
  return (
    <WorkspaceStartScreen
      answerCloneApproval={answerCloneApproval}
      recents={state.recents}
      openWorkspace={openWorkspace}
    />
  );
}

function WorkspaceStartStatus({
  message,
  onRetry,
}: {
  readonly message: string;
  readonly onRetry?: () => void;
}) {
  return (
    <main className="workspace-start" aria-labelledby="workspace-start-title">
      <section className="workspace-start__card workspace-start__card--status">
        <div className="workspace-start__brand" aria-hidden="true">
          C4
        </div>
        <h1 id="workspace-start-title">Workspace Start</h1>
        <p role="status">{message}</p>
        {onRetry ? (
          <button onClick={onRetry} type="button">
            Try again
          </button>
        ) : null}
      </section>
    </main>
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
