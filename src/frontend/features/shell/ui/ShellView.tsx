import { useEffect, useRef } from "react";

import { isLaunchRoute, isSettingsRoute } from "./shell-routes";
import type {
  ShellFocusRestoreRequest,
  ShellViewProps,
} from "./shell-view.types";
import { LaunchLayout } from "./LaunchLayout";
import { SettingsLayout } from "./SettingsLayout";
import { WorkspaceLayout } from "./WorkspaceLayout";
import "./shell-view.css";

/** Composes every accepted route into one controller-owned application root. */
export function ShellView(props: ShellViewProps) {
  const root = useRef<HTMLDivElement>(null);
  const handledFocusRequest = useRef<number | null>(null);
  const { focusRestoreRequest, onFocusRestored, route } = props;

  // Focus restoration runs after the workspace route has committed. Stable
  // target ids avoid retaining a DOM node that Settings unmounted.
  useEffect(() => {
    const request = focusRestoreRequest;
    if (!request || handledFocusRequest.current === request.requestId) return;
    const target = root.current?.querySelector<HTMLElement>(
      `[data-shell-focus-target="${request.target}"]`,
    );
    if (!target) return;
    target.focus();
    handledFocusRequest.current = request.requestId;
    onFocusRestored(request);
  }, [focusRestoreRequest, onFocusRestored, route]);

  return (
    <div ref={root} className="shell-document" data-shell-document="primary">
      <ShellRouteView {...props} />
    </div>
  );
}

/** Selects one layout without introducing view-owned routing or persistence. */
function ShellRouteView(props: ShellViewProps) {
  if (isLaunchRoute(props.route)) {
    return (
      <LaunchLayout route={props.route}>{props.routeContent}</LaunchLayout>
    );
  }

  if (isSettingsRoute(props.route)) {
    return (
      <SettingsLayout
        route={props.route}
        onNavigate={props.onNavigate}
        onBack={props.onBackFromSettings}
      >
        {props.routeContent}
      </SettingsLayout>
    );
  }

  return (
    <WorkspaceLayout
      route={props.route}
      composer={props.composer}
      composerContent={props.composerContent}
      {...(props.workspaceTitle === undefined
        ? {}
        : { workspaceTitle: props.workspaceTitle })}
      {...(props.workspaceTitleAccessory === undefined
        ? {}
        : { workspaceTitleAccessory: props.workspaceTitleAccessory })}
      projectPanel={props.projectPanel}
      projectPanelContent={props.projectPanelContent}
      {...(props.contextualChatContent === undefined
        ? {}
        : { contextualChatContent: props.contextualChatContent })}
      {...(props.projectPanelContentOwnsHeading === undefined
        ? {}
        : {
            projectPanelContentOwnsHeading:
              props.projectPanelContentOwnsHeading,
          })}
      {...(props.showReviewSettingsControl === undefined
        ? {}
        : { showReviewSettingsControl: props.showReviewSettingsControl })}
      {...(props.showContextualChat === undefined
        ? {}
        : { showContextualChat: props.showContextualChat })}
      onVisitSettings={props.onVisitSettings}
      onPanelWidthChange={props.onPanelWidthChange}
      onPanelOpenChange={props.onPanelOpenChange}
      onPanelOverlayDismiss={props.onPanelOverlayDismiss}
      onComposerDraftChange={props.onComposerDraftChange}
      onComposerModeChange={props.onComposerModeChange}
    >
      {props.routeContent}
    </WorkspaceLayout>
  );
}

// Re-exporting the request type here gives adapters one import for the view API.
export type { ShellFocusRestoreRequest };
