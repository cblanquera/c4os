import { useEffect, useState, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router";

import { useAppDispatch, useAppSelector, useAppStore } from "../../app/hooks";
import {
  createSettingsVisit,
  settingsSectionForRoute,
} from "../../app/settings-visit";
import type { AppRoutePath } from "../../app/route-contract";
import { Notice } from "../../components/accessible";
import { NativePlatformSettingsContent } from "../platform";
import {
  selectComposerDraft,
  selectConversation,
  selectLaunch,
  selectSettingsProjection,
  selectSettingsReturnState,
  selectSessions,
  selectWorkspace,
  selectWorkspaceUiDraft,
  shellPanelBounds,
  shellDraftActions,
  UNINITIALIZED_GENERATION,
} from "./state";
import {
  ShellView,
  isSettingsRoute,
  type ShellFocusRestoreRequest,
  type ShellFocusTarget,
  type ShellRoutePath,
} from "./ui";

interface ShellRouteControllerProps {
  readonly route: ShellRoutePath;
}

type ShellNavigationState = {
  readonly focusRestoreRequest?: ShellFocusRestoreRequest;
};

let focusRequestSequence = 0;

/** Binds the accepted shell view to the single application router and store. */
export function ShellRouteController({ route }: ShellRouteControllerProps) {
  const dispatch = useAppDispatch();
  const store = useAppStore();
  const navigate = useNavigate();
  const location = useLocation();
  const workspace = useAppSelector(selectWorkspace);
  const launch = useAppSelector(selectLaunch);
  const sessions = useAppSelector(selectSessions);
  const conversation = useAppSelector(selectConversation);
  const settings = useAppSelector(selectSettingsProjection);
  const uiDraft = useAppSelector(selectWorkspaceUiDraft);
  const composerDraft = useAppSelector(selectComposerDraft);
  const settingsReturn = useAppSelector(selectSettingsReturnState);
  const qaEnabled = useAppSelector((state) => state.shellQa.enabled);
  const viewportWidth = useViewportWidth();
  const overlayPanel = viewportWidth <= 992;
  const navigationState = location.state as ShellNavigationState | null;
  const focusRestoreRequest = navigationState?.focusRestoreRequest ?? null;
  const panelOpen = overlayPanel
    ? !uiDraft.panel.collapsed && uiDraft.panel.overlayOpen
    : !uiDraft.panel.collapsed;
  const panelBounds = shellPanelBounds(viewportWidth);
  const panelWidth = Math.round(
    Math.min(
      panelBounds.maximum,
      Math.max(panelBounds.minimum, uiDraft.panel.width),
    ),
  );

  useEffect(() => {
    if (panelWidth === uiDraft.panel.width) return;
    dispatch(
      shellDraftActions.leftPanelResized({
        width: panelWidth,
        viewportWidth,
      }),
    );
  }, [dispatch, panelWidth, uiDraft.panel.width, viewportWidth]);

  const navigateWithinShell = (destination: ShellRoutePath) => {
    if (isSettingsRoute(destination)) {
      dispatch(
        shellDraftActions.settingsSectionChanged(
          settingsSectionForRoute(destination),
        ),
      );
    }
    void navigate(destination);
  };

  const visitSettings = (focusTarget: ShellFocusTarget) => {
    dispatch(
      shellDraftActions.settingsVisited(
        createSettingsVisit(store.getState(), route, focusTarget),
      ),
    );
    void navigate("/settings/providers");
  };

  const returnFromSettings = () => {
    const destination = settingsReturn?.route ?? "/start";
    const focusTarget = asShellFocusTarget(settingsReturn?.focusTarget ?? null);
    dispatch(shellDraftActions.settingsVisitEnded());
    if (focusTarget === null) {
      void navigate(destination);
      return;
    }
    focusRequestSequence += 1;
    void navigate(destination, {
      state: {
        focusRestoreRequest: {
          requestId: focusRequestSequence,
          target: focusTarget,
        },
      } satisfies ShellNavigationState,
    });
  };

  const changePanelOpen = (isOpen: boolean) => {
    if (overlayPanel) {
      dispatch(shellDraftActions.leftPanelOverlayChanged(isOpen));
    } else {
      dispatch(shellDraftActions.leftPanelCollapsed(!isOpen));
    }
  };

  return (
    <ShellView
      route={route}
      composer={{
        draft: composerDraft.text,
        mode: composerDraft.mode,
        isModeLocked: uiDraft.focusedArtifactId !== null,
      }}
      focusRestoreRequest={focusRestoreRequest}
      projectPanel={{
        mode: overlayPanel ? "overlay" : "docked",
        isOpen: panelOpen,
        width: panelWidth,
        minimumWidth: panelBounds.minimum,
        maximumWidth: panelBounds.maximum,
      }}
      projectPanelContent={<ProjectPanelContent projection={workspace} />}
      routeContent={
        <ProjectedRouteContent
          route={route}
          conversation={conversation}
          launch={launch}
          sessions={sessions}
          settings={settings}
          workspace={workspace}
        />
      }
      showReviewSettingsControl={qaEnabled}
      showContextualChat={uiDraft.focusedArtifactId !== null}
      onBackFromSettings={returnFromSettings}
      onComposerDraftChange={(draft) =>
        dispatch(shellDraftActions.composerTextChanged(draft))
      }
      onComposerModeChange={(mode) =>
        dispatch(shellDraftActions.composerModeChanged(mode))
      }
      onFocusRestored={() => undefined}
      onNavigate={navigateWithinShell}
      onPanelOpenChange={changePanelOpen}
      onPanelOverlayDismiss={() =>
        dispatch(shellDraftActions.leftPanelOverlayChanged(false))
      }
      onPanelWidthChange={(width) =>
        dispatch(
          shellDraftActions.leftPanelResized({
            width,
            viewportWidth,
          }),
        )
      }
      onVisitSettings={visitSettings}
    />
  );
}

function useViewportWidth(): number {
  const [viewportWidth, setViewportWidth] = useState(() => window.innerWidth);

  useEffect(() => {
    const update = () => setViewportWidth(window.innerWidth);
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  return viewportWidth;
}

type WorkspaceState = ReturnType<typeof selectWorkspace>;
type SessionsState = ReturnType<typeof selectSessions>;
type ConversationState = ReturnType<typeof selectConversation>;
type SettingsState = ReturnType<typeof selectSettingsProjection>;
type LaunchState = ReturnType<typeof selectLaunch>;

interface ProjectedRouteContentProps {
  readonly route: AppRoutePath;
  readonly launch: LaunchState;
  readonly workspace: WorkspaceState;
  readonly sessions: SessionsState;
  readonly conversation: ConversationState;
  readonly settings: SettingsState;
}

/** Renders only state already present in authoritative projections. */
function ProjectedRouteContent({
  route,
  launch,
  workspace,
  sessions,
  conversation,
  settings,
}: ProjectedRouteContentProps): ReactNode {
  if (route === "/settings/providers") {
    return <NativePlatformSettingsContent />;
  }

  if (route === "/onboarding") {
    return projectionNotice(
      launch.generation,
      "Provider setup is unavailable",
      "C4OS is waiting for the native launch projection.",
    );
  }

  if (route === "/start") {
    return projectionNotice(
      workspace.generation,
      "Workspace Start is unavailable",
      "C4OS is waiting for the native workspace projection.",
      workspace.value.displayName
        ? `Continue ${workspace.value.displayName}.`
        : "Choose a workspace to continue.",
    );
  }

  if (route === "/chat") {
    if (conversation.generation === UNINITIALIZED_GENERATION) {
      return projectionNotice(
        conversation.generation,
        "Conversation unavailable",
        "C4OS is waiting for an authoritative conversation projection.",
      );
    }
    return (
      <div className="shell-projection-list" aria-label="Conversation turns">
        {conversation.value.turns.map((turn) => (
          <article key={turn.id} data-author={turn.author}>
            <strong>{turn.author === "user" ? "You" : "C4OS"}</strong>
            <p>{turn.markdown}</p>
          </article>
        ))}
      </div>
    );
  }

  if (route === "/chat-search") {
    return projectionList(
      sessions.generation,
      "Saved chat sessions",
      sessions.value.sessions.map((session) => session.title),
    );
  }

  if (route.startsWith("/settings/")) {
    return projectionNotice(
      settings.generation,
      `${routeTitle(route)} unavailable`,
      "C4OS is waiting for the authoritative Settings projection.",
      "The Settings projection is ready for service-owned controls.",
    );
  }

  return projectionNotice(
    workspace.generation,
    `${routeTitle(route)} unavailable`,
    "C4OS is waiting for the authoritative workspace projection.",
    `${routeTitle(route)} is ready for its service-owned content.`,
  );
}

function ProjectPanelContent({
  projection,
}: {
  readonly projection: WorkspaceState;
}) {
  if (projection.generation === UNINITIALIZED_GENERATION) {
    return (
      <p>Projects are unavailable until native workspace state arrives.</p>
    );
  }
  if (projection.value.projects.length === 0) {
    return <p>No projects are open.</p>;
  }
  return (
    <ul className="shell-project-list">
      {projection.value.projects.map((project) => (
        <li key={project.id}>
          <strong>{project.name}</strong>
          <span>{project.pathState === "found" ? "Available" : "Missing"}</span>
        </li>
      ))}
    </ul>
  );
}

function projectionNotice(
  generation: number,
  unavailableTitle: string,
  unavailableDetail: string,
  readyDetail?: string,
) {
  const isReady = generation !== UNINITIALIZED_GENERATION;
  return (
    <Notice
      title={isReady ? "Ready" : unavailableTitle}
      tone={isReady ? "success" : "warning"}
    >
      {isReady ? readyDetail : unavailableDetail}
    </Notice>
  );
}

function projectionList(
  generation: number,
  label: string,
  items: readonly string[],
) {
  if (generation === UNINITIALIZED_GENERATION) {
    return projectionNotice(
      generation,
      `${label} unavailable`,
      "C4OS is waiting for authoritative session state.",
    );
  }
  return (
    <ul className="shell-projection-list" aria-label={label}>
      {items.map((item) => (
        <li key={item}>{item}</li>
      ))}
    </ul>
  );
}

function routeTitle(route: AppRoutePath): string {
  return route
    .split("/")
    .filter(Boolean)
    .at(-1)!
    .split("-")
    .map((word) => `${word[0]?.toLocaleUpperCase()}${word.slice(1)}`)
    .join(" ");
}

function asShellFocusTarget(value: string | null): ShellFocusTarget | null {
  return value === "workspace-settings" ||
    value === "project-panel-toggle" ||
    value === "composer-draft"
    ? value
    : null;
}
