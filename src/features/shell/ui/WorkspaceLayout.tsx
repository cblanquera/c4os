import type { CSSProperties, ChangeEvent, ReactNode } from "react";

import {
  Button,
  Icon,
  IconButton,
  Resizer,
} from "../../../components/accessible";
import type { ShellComposerMode, WorkspaceRoutePath } from "./shell-routes";
import type {
  ShellComposerState,
  ShellFocusTarget,
  ShellProjectPanelState,
} from "./shell-view.types";
import { getShellRouteCopy } from "./shell-routes";
import { RouteSurface, UnavailableGate } from "./RouteSurface";

interface WorkspaceLayoutProps {
  readonly route: WorkspaceRoutePath;
  readonly children?: ReactNode;
  readonly composerContent?: ReactNode;
  readonly workspaceTitle?: string;
  readonly workspaceTitleAccessory?: ReactNode;
  readonly projectPanelContent?: ReactNode;
  readonly contextualChatContent?: ReactNode;
  readonly projectPanelContentOwnsHeading?: boolean;
  readonly composer: ShellComposerState;
  readonly projectPanel: ShellProjectPanelState;
  readonly showReviewSettingsControl?: boolean;
  readonly showContextualChat?: boolean;
  readonly onVisitSettings: (returnFocusTarget: ShellFocusTarget) => void;
  readonly onPanelWidthChange: (width: number) => void;
  readonly onPanelOpenChange: (isOpen: boolean) => void;
  readonly onPanelOverlayDismiss: () => void;
  readonly onComposerDraftChange: (draft: string) => void;
  readonly onComposerModeChange: (mode: ShellComposerMode) => void;
}

const COMPOSER_MODES = ["chat", "files", "browser", "terminal"] as const;

/** Renders the stateful workspace geometry without owning product state. */
export function WorkspaceLayout({
  route,
  children,
  composerContent,
  workspaceTitle,
  workspaceTitleAccessory,
  projectPanelContent,
  contextualChatContent,
  projectPanelContentOwnsHeading = false,
  composer,
  projectPanel,
  showReviewSettingsControl = false,
  showContextualChat = false,
  onVisitSettings,
  onPanelWidthChange,
  onPanelOpenChange,
  onPanelOverlayDismiss,
  onComposerDraftChange,
  onComposerModeChange,
}: WorkspaceLayoutProps) {
  const hasContextualChat =
    contextualChatContent !== undefined && contextualChatContent !== null;
  const exposesContextualChat = showContextualChat || hasContextualChat;
  const minimumWidth = projectPanel.minimumWidth ?? 180;
  const maximumWidth = Math.max(minimumWidth, projectPanel.maximumWidth ?? 640);

  /** Constrains view-level resize intent to the accepted panel range. */
  const requestPanelWidth = (width: number) => {
    onPanelWidthChange(Math.min(maximumWidth, Math.max(minimumWidth, width)));
  };

  /** Emits controlled composer drafts without claiming submission behavior. */
  const handleDraftChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    onComposerDraftChange(event.currentTarget.value);
  };

  /** Emits controlled composer mode changes allowed by the shell projection. */
  const handleModeChange = (event: ChangeEvent<HTMLSelectElement>) => {
    onComposerModeChange(event.currentTarget.value as ShellComposerMode);
  };

  /** Dismisses only an open overlay while preserving the intended main action. */
  const handleStagePointerDown = () => {
    if (projectPanel.mode === "overlay" && projectPanel.isOpen) {
      onPanelOverlayDismiss();
    }
  };

  const title = workspaceTitle ?? getShellRouteCopy(route).title;
  const panelStyle = {
    "--shell-project-panel-width": `${projectPanel.width}px`,
  } as CSSProperties;

  return (
    <div
      className="shell-view shell-workspace"
      data-panel-mode={projectPanel.mode}
      data-panel-open={projectPanel.isOpen}
      data-shell-layout="workspace"
      style={panelStyle}
    >
      <header className="shell-title-header" aria-label="C4OS window title">
        <IconButton
          className="shell-title-header__panel-toggle"
          label={
            projectPanel.isOpen
              ? "Collapse project panel"
              : "Show project panel"
          }
          aria-expanded={projectPanel.isOpen}
          data-shell-focus-target="project-panel-toggle"
          icon={<Icon name="project" />}
          onPress={() => onPanelOpenChange(!projectPanel.isOpen)}
        />
        <span className="shell-title-header__route">{title}</span>
        {workspaceTitleAccessory || showReviewSettingsControl ? (
          <span className="shell-title-header__accessories">
            {workspaceTitleAccessory}
            {showReviewSettingsControl ? (
              <Button
                className="shell-title-header__settings"
                data-shell-focus-target="workspace-settings"
                onPress={() => onVisitSettings("workspace-settings")}
                variant="quiet"
              >
                Settings
              </Button>
            ) : null}
          </span>
        ) : null}
      </header>

      <div className="shell-workspace__body">
        <aside
          className="shell-project-panel"
          aria-label={
            exposesContextualChat ? "Projects and contextual chat" : "Projects"
          }
          aria-hidden={!projectPanel.isOpen}
          data-contextual-chat={exposesContextualChat}
          inert={!projectPanel.isOpen ? true : undefined}
        >
          <div className="shell-project-panel__projects">
            <div
              className="shell-project-panel__heading"
              data-content-heading={projectPanelContentOwnsHeading}
            >
              {projectPanelContentOwnsHeading ? null : <h2>Projects</h2>}
              <IconButton
                icon={<Icon name="chevron-right" />}
                label="Hide project panel"
                onPress={() => onPanelOpenChange(false)}
              />
            </div>
            {projectPanelContent ?? <p>No project is open.</p>}
          </div>
          {hasContextualChat ? (
            <div className="shell-project-panel__context">
              {contextualChatContent}
            </div>
          ) : showContextualChat ? (
            <section
              className="shell-project-panel__context"
              aria-labelledby="shell-context-chat-title"
            >
              <div className="shell-project-panel__heading">
                <h2 id="shell-context-chat-title">Chat</h2>
                <UnavailableGate feature="Detached native windows">
                  Detach Chat
                </UnavailableGate>
              </div>
              <p>Conversation context stays in this application window.</p>
            </section>
          ) : null}
        </aside>

        {projectPanel.isOpen ? (
          <Resizer
            className="shell-project-panel__resizer"
            label="Resize project panel"
            max={maximumWidth}
            min={minimumWidth}
            onValueChange={requestPanelWidth}
            orientation="vertical"
            step={12}
            value={projectPanel.width}
          />
        ) : null}

        <div className="shell-workspace__center">
          <main
            className="shell-workspace__stage"
            onPointerDown={handleStagePointerDown}
          >
            <RouteSurface
              compact={route === "/chat" && children !== undefined}
              route={route}
            >
              {children}
              <RouteNegativeGates route={route} />
            </RouteSurface>
          </main>

          {composerContent ?? (
            <form
              className="shell-composer"
              aria-label="Message composer"
              onSubmit={(event) => event.preventDefault()}
            >
              <label className="shell-composer__mode">
                <span>Mode</span>
                <select
                  aria-label="Composer mode"
                  value={composer.mode}
                  disabled={composer.isModeLocked}
                  onChange={handleModeChange}
                >
                  {COMPOSER_MODES.map((mode) => (
                    <option key={mode} value={mode}>
                      {mode[0]?.toLocaleUpperCase()}
                      {mode.slice(1)}
                    </option>
                  ))}
                </select>
              </label>
              <label className="shell-composer__draft">
                <span className="shell-visually-hidden">Message</span>
                <textarea
                  rows={2}
                  value={composer.draft}
                  placeholder="Ask C4OS"
                  data-shell-focus-target="composer-draft"
                  onChange={handleDraftChange}
                />
              </label>
            </form>
          )}
        </div>
      </div>
    </div>
  );
}

interface RouteNegativeGatesProps {
  readonly route: WorkspaceRoutePath;
}

/** Places deferred controls only on the surfaces where users would expect them. */
function RouteNegativeGates({ route }: RouteNegativeGatesProps) {
  if (route === "/browser") {
    return (
      <div className="shell-route-gates">
        <UnavailableGate feature="Browser sub-tabs">New tab</UnavailableGate>
      </div>
    );
  }

  if (route === "/terminal") {
    return (
      <div className="shell-route-gates">
        <UnavailableGate feature="Full-screen terminal programs">
          Enter full screen
        </UnavailableGate>
        <UnavailableGate feature="Terminal password entry">
          Enter password
        </UnavailableGate>
      </div>
    );
  }

  return null;
}
