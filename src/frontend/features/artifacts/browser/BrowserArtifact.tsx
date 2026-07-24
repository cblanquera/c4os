import type { ReactNode } from "react";

import { ArtifactShell } from "../ui/ArtifactShell";
import type { ArtifactContext, ArtifactShellStatus } from "../types";
import { NativeBrowserViewport } from "./NativeBrowserViewport";
import type {
  BrowserArtifactModel,
  BrowserPendingApproval,
  BrowserViewportFocusIntent,
  BrowserViewportLifecycleEvent,
} from "./browser-types";
import "./browser-artifact.css";

export interface BrowserArtifactProps {
  readonly context: ArtifactContext;
  readonly model: BrowserArtifactModel;
  readonly onAllowApproval?: (artifactId: string, approvalId: string) => void;
  readonly onBack?: (artifactId: string) => void;
  readonly onClose?: (artifactId: string) => void;
  readonly onClearData?: (artifactId: string) => void;
  readonly onCopy?: (artifactId: string, currentUrl: string) => void;
  readonly onDenyApproval?: (artifactId: string, approvalId: string) => void;
  readonly onExpand?: (artifactId: string) => void;
  readonly onForward?: (artifactId: string) => void;
  readonly onRefresh?: (artifactId: string) => void;
  readonly onReply?: (artifactId: string) => void;
  readonly onViewportFocusIntent?: (intent: BrowserViewportFocusIntent) => void;
  readonly onViewportLifecycle?: (event: BrowserViewportLifecycleEvent) => void;
}

/** Renders one controlled Browser provider without rendering website DOM. */
export function BrowserArtifact({
  context,
  model,
  onAllowApproval,
  onBack,
  onClose,
  onClearData,
  onCopy,
  onDenyApproval,
  onExpand,
  onForward,
  onRefresh,
  onReply,
  onViewportFocusIntent,
  onViewportLifecycle,
}: BrowserArtifactProps) {
  const isContextual = context === "contextual";
  const isFocused = context === "focused";
  const status = browserShellStatus(model);

  return (
    <ArtifactShell
      context={context}
      focusSupported
      footerContent={browserFooter(model)}
      headerContent={
        <BrowserNavigation
          contextual={isContextual}
          model={model}
          onBack={onBack}
          onClearData={onClearData}
          onForward={onForward}
          onRefresh={onRefresh}
        />
      }
      identity={{
        accessibleLabel: `Browser response artifact: ${model.title}`,
        id: model.artifactId,
        title: model.title,
        typeLabel: "Browser",
      }}
      onClose={onClose ? () => onClose(model.artifactId) : undefined}
      onCopy={
        onCopy ? () => onCopy(model.artifactId, model.currentUrl) : undefined
      }
      onExpand={onExpand ? () => onExpand(model.artifactId) : undefined}
      onReply={
        isFocused && onReply ? () => onReply(model.artifactId) : undefined
      }
      status={status}
    >
      <div
        aria-busy={model.phase === "loading" || model.refreshing}
        className="artifact-browser__body"
        data-browser-phase={model.phase}
      >
        {model.phase === "loading" ? (
          <p className="artifact-browser__loading" role="status">
            Loading web page
          </p>
        ) : null}
        {model.pendingApproval !== null ? (
          <BrowserApproval
            artifactId={model.artifactId}
            approval={model.pendingApproval}
            onAllow={onAllowApproval}
            onDeny={onDenyApproval}
            readOnly={isContextual}
          />
        ) : null}
        <BrowserNotices notices={model.notices} />
        {isFocused ? (
          <NativeBrowserViewport
            accessibleTitle={model.pageTitle || model.title}
            artifactId={model.artifactId}
            baseRecordRevision={model.recordRevision}
            controllerGeneration={model.controllerGeneration}
            mountGeneration={model.mountGeneration}
            {...(onViewportFocusIntent === undefined
              ? {}
              : { onFocusIntent: onViewportFocusIntent })}
            {...(onViewportLifecycle === undefined
              ? {}
              : { onLifecycle: onViewportLifecycle })}
            presentation="focused"
          />
        ) : (
          <BrowserMetadataPreview model={model} contextual={isContextual} />
        )}
      </div>
    </ArtifactShell>
  );
}

interface BrowserNavigationProps {
  readonly contextual: boolean;
  readonly model: BrowserArtifactModel;
  readonly onBack: BrowserArtifactProps["onBack"];
  readonly onClearData: BrowserArtifactProps["onClearData"];
  readonly onForward: BrowserArtifactProps["onForward"];
  readonly onRefresh: BrowserArtifactProps["onRefresh"];
}

function BrowserNavigation({
  contextual,
  model,
  onBack,
  onClearData,
  onForward,
  onRefresh,
}: BrowserNavigationProps) {
  return (
    <nav
      aria-label="Browser navigation"
      className="artifact-browser__navigation"
    >
      {contextual ? null : (
        <span className="artifact-browser__navigation-actions">
          <button
            aria-label="Back"
            disabled={
              model.pendingApproval !== null ||
              !model.canGoBack ||
              onBack === undefined
            }
            onClick={() => onBack?.(model.artifactId)}
            type="button"
          >
            Back
          </button>
          <button
            aria-label="Forward"
            disabled={
              model.pendingApproval !== null ||
              !model.canGoForward ||
              onForward === undefined
            }
            onClick={() => onForward?.(model.artifactId)}
            type="button"
          >
            Forward
          </button>
          <button
            aria-label="Refresh"
            disabled={
              model.pendingApproval !== null ||
              model.refreshing ||
              onRefresh === undefined
            }
            onClick={() => onRefresh?.(model.artifactId)}
            type="button"
          >
            {model.refreshing ? "Refreshing…" : "Refresh"}
          </button>
          <button
            aria-label="Clear browser data"
            disabled={
              model.pendingApproval !== null ||
              model.phase === "queued" ||
              model.phase === "loading" ||
              onClearData === undefined
            }
            onClick={() => onClearData?.(model.artifactId)}
            title={`Clear ${browserEnvironmentLabel(model.environmentScope)} data`}
            type="button"
          >
            Clear data
          </button>
        </span>
      )}
      <label className="artifact-browser__address">
        <span className="artifact-browser__visually-hidden">
          Current web address
        </span>
        <input readOnly type="url" value={model.currentUrl} />
      </label>
    </nav>
  );
}

interface BrowserApprovalProps {
  readonly approval: BrowserPendingApproval;
  readonly artifactId: string;
  readonly onAllow: BrowserArtifactProps["onAllowApproval"];
  readonly onDeny: BrowserArtifactProps["onDenyApproval"];
  readonly readOnly: boolean;
}

function BrowserApproval({
  approval,
  artifactId,
  onAllow,
  onDeny,
  readOnly,
}: BrowserApprovalProps) {
  return (
    <section
      aria-label={`Browser permission request from ${approval.origin}`}
      className="artifact-browser__approval"
      role={readOnly ? "status" : "alert"}
    >
      <div>
        <strong>{approval.permission}</strong>
        <p>{approval.message}</p>
        <small>{approval.origin}</small>
      </div>
      {readOnly ? null : (
        <span className="artifact-browser__approval-actions">
          <button
            disabled={onDeny === undefined}
            onClick={() => onDeny?.(artifactId, approval.approvalId)}
            type="button"
          >
            Deny
          </button>
          <button
            disabled={onAllow === undefined}
            onClick={() => onAllow?.(artifactId, approval.approvalId)}
            type="button"
          >
            Allow
          </button>
        </span>
      )}
    </section>
  );
}

function BrowserNotices({
  notices,
}: {
  readonly notices: BrowserArtifactModel["notices"];
}) {
  if (notices.length === 0) return null;
  return (
    <div aria-label="Browser notices" className="artifact-browser__notices">
      {notices.map((notice) => (
        <section
          className="artifact-browser__notice"
          data-notice-kind={notice.kind}
          key={notice.id}
          role={notice.kind === "error" ? "alert" : "status"}
        >
          <strong>{notice.title}</strong>
          <p>{notice.message}</p>
        </section>
      ))}
    </div>
  );
}

function BrowserMetadataPreview({
  contextual,
  model,
}: {
  readonly contextual: boolean;
  readonly model: BrowserArtifactModel;
}) {
  return (
    <section
      aria-label="Browser page metadata"
      className="artifact-browser__metadata"
      data-browser-preview={contextual ? "contextual" : "compact"}
    >
      <strong>{model.pageTitle || model.title}</strong>
      <dl>
        <div>
          <dt>Page</dt>
          <dd>{model.currentUrl}</dd>
        </div>
        <div>
          <dt>Navigation</dt>
          <dd>{historyLabel(model)}</dd>
        </div>
      </dl>
    </section>
  );
}

function browserShellStatus(model: BrowserArtifactModel): ArtifactShellStatus {
  if (model.status !== undefined) return model.status;
  if (model.phase === "queued") {
    // Queued is the Browser's pre-controller phase. The native viewport must
    // remain composed so its first measured rectangle can trigger the mount.
    return { kind: "ready", message: "Preparing Browser" };
  }
  if (model.phase === "loading") {
    // Once the controller has a mount, loading is non-blocking: the native view
    // must retain its rectangle while this component announces busy state.
    return { kind: "ready", message: "Loading" };
  }
  if (model.phase === "error") {
    return { kind: "error", message: "The web page could not be loaded." };
  }
  if (model.phase === "recovery") {
    return {
      kind: "recovery",
      message: "The Browser controller is ready to recover.",
    };
  }
  return model.refreshing
    ? { kind: "ready", message: "Refreshing" }
    : { kind: "ready" };
}

function browserFooter(model: BrowserArtifactModel): ReactNode {
  const state =
    model.phase === "ready" && model.refreshing
      ? "Refreshing"
      : phaseLabel(model);
  return `${state} · ${historyLabel(model)}`;
}

function phaseLabel(model: BrowserArtifactModel): string {
  const labels: Record<BrowserArtifactModel["phase"], string> = {
    queued: "Queued",
    loading: "Loading",
    ready: "Ready",
    error: "Unavailable",
    recovery: "Recovery available",
  };
  return labels[model.phase];
}

function historyLabel(model: BrowserArtifactModel): string {
  if (model.canGoBack && model.canGoForward) return "Middle of history";
  if (model.canGoBack) return "Latest history entry";
  if (model.canGoForward) return "First history entry";
  return "Only history entry";
}

function browserEnvironmentLabel(
  environmentScope: BrowserArtifactModel["environmentScope"],
): string {
  const labels: Record<BrowserArtifactModel["environmentScope"], string> = {
    "all-browsers": "All browsers",
    "per-project": "Per project",
    "per-chat-session": "Per chat session",
    none: "this ephemeral Browser",
  };
  return labels[environmentScope];
}
