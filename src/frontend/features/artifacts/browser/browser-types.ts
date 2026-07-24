import type { ArtifactShellStatus } from "../types";

export const BROWSER_PHASES = [
  "queued",
  "loading",
  "ready",
  "error",
  "recovery",
] as const;

export type BrowserPhase = (typeof BROWSER_PHASES)[number];

export interface BrowserArtifactNotice {
  readonly id: string;
  readonly kind: "information" | "permission" | "warning" | "error";
  readonly message: string;
  readonly title: string;
}

export interface BrowserPendingApproval {
  readonly approvalId: string;
  readonly message: string;
  readonly origin: string;
  readonly permission: string;
}

/** The bounded Browser state projected by the Rust-owned native controller. */
export interface BrowserArtifactModel {
  readonly artifactId: string;
  readonly canGoBack: boolean;
  readonly canGoForward: boolean;
  readonly controllerGeneration: number;
  readonly currentUrl: string;
  readonly environmentScope:
    "all-browsers" | "per-project" | "per-chat-session" | "none";
  readonly mountGeneration: number;
  readonly notices: readonly BrowserArtifactNotice[];
  readonly pageTitle: string;
  readonly pendingApproval: BrowserPendingApproval | null;
  readonly phase: BrowserPhase;
  readonly recordRevision: number;
  readonly refreshing: boolean;
  readonly status?: ArtifactShellStatus;
  readonly title: string;
}

export type BrowserViewportPresentation = "focused";

export interface BrowserViewportIdentity {
  readonly artifactId: string;
  readonly baseRecordRevision: number;
  readonly controllerGeneration: number;
  readonly mountGeneration: number;
  readonly presentation: BrowserViewportPresentation;
}

export interface BrowserViewportRect {
  readonly height: number;
  readonly width: number;
  readonly x: number;
  readonly y: number;
}

export type BrowserViewportLifecycleEvent = BrowserViewportIdentity &
  (
    | {
        readonly kind: "geometry";
        readonly rect: BrowserViewportRect;
        readonly visible: true;
      }
    | {
        readonly kind: "hidden";
        readonly rect: BrowserViewportRect;
        readonly visible: false;
      }
    | {
        readonly kind: "detach";
      }
  );

export interface BrowserViewportFocusIntent extends BrowserViewportIdentity {
  readonly input: "keyboard" | "pointer";
}
