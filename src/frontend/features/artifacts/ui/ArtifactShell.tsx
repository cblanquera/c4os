import type { ReactNode } from "react";

import type {
  ArtifactContext,
  ArtifactIdentity,
  ArtifactShellActionCallbacks,
  ArtifactShellStatus,
} from "../types";
import "./artifact-shell.css";

export interface ArtifactShellProps extends ArtifactShellActionCallbacks {
  readonly children: ReactNode;
  readonly context: ArtifactContext;
  readonly focusSupported: boolean;
  readonly footerContent?: ReactNode | undefined;
  readonly headerContent?: ReactNode | undefined;
  readonly identity: ArtifactIdentity;
  readonly status?: ArtifactShellStatus | undefined;
}

/** Renders static artifact chrome around the only internally scrolling body. */
export function ArtifactShell({
  children,
  context,
  focusSupported,
  footerContent,
  headerContent,
  identity,
  onClose,
  onCopy,
  onExpand,
  onReply,
  status = { kind: "ready" },
}: ArtifactShellProps) {
  // Non-focusable providers always render through the inline-safe shell, even
  // if a stale caller asks for a focused or contextual presentation.
  const effectiveContext = focusSupported ? context : "inline";
  const showsClose = effectiveContext === "focused";
  const showsExpand =
    focusSupported && effectiveContext !== "focused" && onExpand !== undefined;
  const isBlockingStatus = status.kind === "loading" || status.kind === "error";

  return (
    <article
      aria-busy={status.kind === "loading"}
      aria-label={identity.accessibleLabel}
      className="artifact-shell"
      data-artifact-context={effectiveContext}
      data-artifact-id={identity.id}
      data-artifact-status={status.kind}
    >
      <header className="artifact-shell__header" data-artifact-static="header">
        <div className="artifact-shell__identity">
          <span aria-hidden="true" className="artifact-shell__icon">
            {identity.icon ?? "◇"}
          </span>
          <span className="artifact-shell__identity-copy">
            <span>{identity.typeLabel}</span>
            <strong>{identity.title}</strong>
          </span>
          <ArtifactStatusLabel status={status} />
          <span className="artifact-shell__focus-action">
            {showsClose ? (
              <button
                disabled={onClose === undefined}
                onClick={onClose}
                type="button"
              >
                Close
              </button>
            ) : showsExpand ? (
              <button onClick={onExpand} type="button">
                Expand
              </button>
            ) : (
              <span
                aria-hidden="true"
                className="artifact-shell__action-space"
              />
            )}
          </span>
        </div>
        {headerContent ? (
          <div className="artifact-shell__provider-header">{headerContent}</div>
        ) : null}
      </header>

      <div
        className="artifact-shell__body"
        data-artifact-scroll-region="body"
        tabIndex={0}
      >
        <ArtifactStatusBody status={status} />
        {isBlockingStatus ? null : children}
      </div>

      <footer className="artifact-shell__footer" data-artifact-static="footer">
        <div className="artifact-shell__footer-content">{footerContent}</div>
        <div aria-label="Artifact actions" className="artifact-shell__actions">
          <button
            disabled={onCopy === undefined}
            onClick={onCopy}
            type="button"
          >
            Copy
          </button>
          <button
            disabled={onReply === undefined}
            onClick={onReply}
            type="button"
          >
            Reply
          </button>
        </div>
      </footer>
    </article>
  );
}

interface ArtifactStatusProps {
  readonly status: ArtifactShellStatus;
}

/** Keeps a concise non-color status label visible in the static header. */
function ArtifactStatusLabel({ status }: ArtifactStatusProps) {
  if (status.kind === "ready" && !status.message) return null;
  const label = status.message ?? "Ready";
  return (
    <span className="artifact-shell__status-label" data-state={status.kind}>
      {label}
    </span>
  );
}

/** Announces operational states without moving the shared shell chrome. */
function ArtifactStatusBody({ status }: ArtifactStatusProps) {
  if (status.kind === "ready") return null;
  const role = status.kind === "error" ? "alert" : "status";
  return (
    <div
      className="artifact-shell__status"
      data-state={status.kind}
      role={role}
    >
      <strong>{statusTitle(status.kind)}</strong>
      <p>{status.message}</p>
    </div>
  );
}

/** Converts wire-safe status names into explicit user-facing state labels. */
function statusTitle(kind: ArtifactShellStatus["kind"]): string {
  const titles: Record<ArtifactShellStatus["kind"], string> = {
    ready: "Ready",
    loading: "Loading artifact",
    error: "Artifact unavailable",
    degraded: "Limited artifact",
    recovery: "Recovery available",
  };
  return titles[kind];
}
