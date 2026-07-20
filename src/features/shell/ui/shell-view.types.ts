import type { ReactNode } from "react";

import type { ShellComposerMode, ShellRoutePath } from "./shell-routes";

// Focus targets survive route replacement as stable identifiers rather than
// references to DOM nodes that Settings temporarily removes.
export type ShellFocusTarget =
  "workspace-settings" | "project-panel-toggle" | "composer-draft";

// The monotonically increasing request id lets a controller request the same
// target more than once without introducing view-owned navigation state.
export interface ShellFocusRestoreRequest {
  readonly requestId: number;
  readonly target: ShellFocusTarget;
}

// Panel mode is projected by the responsive controller; CSS provides the
// geometry while these values keep open/collapse behavior deterministic.
export interface ShellProjectPanelState {
  readonly mode: "docked" | "overlay";
  readonly isOpen: boolean;
  readonly width: number;
  readonly minimumWidth?: number;
  readonly maximumWidth?: number;
}

// Composer state is a controlled draft. Submitting and runtime binding remain
// outside the structural shell and belong to later implementation tasks.
export interface ShellComposerState {
  readonly draft: string;
  readonly mode: ShellComposerMode;
  readonly isModeLocked?: boolean;
}

// ShellViewProps is the integration seam between the coordinator-owned store,
// router, and this task-owned, prop-driven view layer.
export interface ShellViewProps {
  readonly route: ShellRoutePath;
  readonly routeContent?: ReactNode;
  readonly projectPanelContent?: ReactNode;
  readonly composer: ShellComposerState;
  readonly projectPanel: ShellProjectPanelState;
  readonly focusRestoreRequest?: ShellFocusRestoreRequest | null;
  /** QA/review builds may expose Settings without relying on the native menu. */
  readonly showReviewSettingsControl?: boolean;
  /** Contextual Chat exists only while an artifact has explicit focus. */
  readonly showContextualChat?: boolean;
  readonly onNavigate: (route: ShellRoutePath) => void;
  readonly onVisitSettings: (returnFocusTarget: ShellFocusTarget) => void;
  readonly onBackFromSettings: () => void;
  readonly onPanelWidthChange: (width: number) => void;
  readonly onPanelOpenChange: (isOpen: boolean) => void;
  readonly onPanelOverlayDismiss: () => void;
  readonly onComposerDraftChange: (draft: string) => void;
  readonly onComposerModeChange: (mode: ShellComposerMode) => void;
  readonly onFocusRestored: (request: ShellFocusRestoreRequest) => void;
}
