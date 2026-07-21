import { useRef } from "react";
import type { ChangeEvent, ReactNode, RefObject } from "react";

import { ArtifactShell } from "../ui/ArtifactShell";
import type {
  ArtifactBreadcrumb,
  ArtifactContext,
  ArtifactShellStatus,
} from "../types";
import "./file-artifact.css";

interface FileArtifactBaseState {
  readonly content: string;
}

export type FileArtifactState =
  | (FileArtifactBaseState & { readonly phase: "read" })
  | (FileArtifactBaseState & {
      readonly draft: string;
      readonly phase: "edit" | "dirty";
    })
  | (FileArtifactBaseState & {
      readonly phase: "proposed";
      readonly proposedContent: string;
      readonly proposalDiff?: string;
      readonly proposalSummary: string;
    })
  | (FileArtifactBaseState & {
      readonly approvalId: string;
      readonly approvalSummary: string;
      readonly phase: "approval";
      readonly proposedContent: string;
      readonly proposalDiff?: string;
    })
  | (FileArtifactBaseState & {
      readonly conflictMessage: string;
      readonly currentVersionLabel: string;
      readonly draft: string;
      readonly phase: "conflict";
    })
  | (FileArtifactBaseState & {
      readonly draft: string;
      readonly phase: "recovery";
      readonly recoveryMessage: string;
    });

export interface FileArtifactModel {
  readonly artifactId: string;
  readonly breadcrumbs: readonly ArtifactBreadcrumb[];
  readonly languageLabel?: string;
  readonly state: FileArtifactState;
  readonly status?: ArtifactShellStatus;
  readonly title: string;
  readonly versionLabel?: string;
}

export type FileConflictResolution = "reload-current" | "keep-draft";

export interface FileArtifactProps {
  readonly context: ArtifactContext;
  readonly model: FileArtifactModel;
  readonly onAllowApproval?: (artifactId: string, approvalId: string) => void;
  readonly onApproveProposal?: (artifactId: string) => void;
  readonly onBreadcrumbSelect?: (
    artifactId: string,
    breadcrumbId: string,
  ) => void;
  readonly onClose?: (artifactId: string) => void;
  readonly onCopy?: (artifactId: string, content: string) => void;
  readonly onDiscard?: (artifactId: string) => void;
  readonly onDenyApproval?: (artifactId: string, approvalId: string) => void;
  readonly onDraftChange?: (artifactId: string, content: string) => void;
  readonly onEdit?: (artifactId: string) => void;
  readonly onExpand?: (artifactId: string) => void;
  readonly onRecoverDraft?: (artifactId: string) => void;
  readonly onRejectProposal?: (artifactId: string) => void;
  readonly onReply?: (
    artifactId: string,
    selection?: { readonly selectedText: string },
  ) => void;
  readonly onResolveConflict?: (
    artifactId: string,
    resolution: FileConflictResolution,
  ) => void;
  readonly onSave?: (artifactId: string, content: string) => void;
}

/** Renders one controlled File provider state through the shared artifact shell. */
export function FileArtifact({
  context,
  model,
  onAllowApproval,
  onApproveProposal,
  onBreadcrumbSelect,
  onClose,
  onCopy,
  onDiscard,
  onDenyApproval,
  onDraftChange,
  onEdit,
  onExpand,
  onRecoverDraft,
  onRejectProposal,
  onReply,
  onResolveConflict,
  onSave,
}: FileArtifactProps) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const visibleContent = fileVisibleContent(model.state);
  const isContextual = context === "contextual";
  const controls = isContextual
    ? null
    : fileControls({
        model,
        onAllowApproval,
        onApproveProposal,
        onDiscard,
        onDenyApproval,
        onEdit,
        onRecoverDraft,
        onRejectProposal,
        onResolveConflict,
        onSave,
      });
  const footer = [
    model.versionLabel,
    model.languageLabel,
    filePhaseLabel(model.state.phase),
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <ArtifactShell
      context={context}
      focusSupported
      footerContent={footer}
      headerContent={
        <div className="artifact-file__header">
          <ArtifactBreadcrumbs
            artifactId={model.artifactId}
            breadcrumbs={model.breadcrumbs}
            isReadOnly={isContextual}
            onSelect={onBreadcrumbSelect}
          />
          <div className="artifact-file__controls">{controls}</div>
        </div>
      }
      identity={{
        accessibleLabel: `File response artifact: ${model.title}`,
        id: model.artifactId,
        title: model.title,
        typeLabel: "File",
      }}
      onClose={onClose ? () => onClose(model.artifactId) : undefined}
      onCopy={
        onCopy ? () => onCopy(model.artifactId, visibleContent) : undefined
      }
      onExpand={onExpand ? () => onExpand(model.artifactId) : undefined}
      onReply={
        onReply
          ? () => {
              const selection = window.getSelection();
              const selectedText = selection?.toString();
              const selectionRoot =
                selection && selection.rangeCount > 0
                  ? selection.getRangeAt(0).commonAncestorContainer
                  : null;
              if (
                selectedText &&
                selectedText.length <= 1_048_576 &&
                selectionRoot !== null &&
                bodyRef.current?.contains(selectionRoot) === true &&
                visibleContent.includes(selectedText)
              ) {
                onReply(model.artifactId, { selectedText });
              } else {
                onReply(model.artifactId);
              }
            }
          : undefined
      }
      status={model.status}
    >
      <FileStateBody
        artifactId={model.artifactId}
        bodyRef={bodyRef}
        context={context}
        onDraftChange={onDraftChange}
        state={model.state}
        title={model.title}
      />
    </ArtifactShell>
  );
}

interface ArtifactBreadcrumbsProps {
  readonly artifactId: string;
  readonly breadcrumbs: readonly ArtifactBreadcrumb[];
  readonly isReadOnly: boolean;
  readonly onSelect: FileArtifactProps["onBreadcrumbSelect"];
}

/** Renders stable path segments while suppressing navigation in contextual Chat. */
function ArtifactBreadcrumbs({
  artifactId,
  breadcrumbs,
  isReadOnly,
  onSelect,
}: ArtifactBreadcrumbsProps) {
  return (
    <nav aria-label="File breadcrumbs" className="artifact-file__breadcrumbs">
      <ol>
        {breadcrumbs.map((breadcrumb) => (
          <li key={breadcrumb.id}>
            <button
              aria-current={breadcrumb.isCurrent ? "page" : undefined}
              disabled={
                isReadOnly || breadcrumb.isCurrent || onSelect === undefined
              }
              onClick={() => onSelect?.(artifactId, breadcrumb.id)}
              type="button"
            >
              {breadcrumb.label}
            </button>
          </li>
        ))}
      </ol>
    </nav>
  );
}

interface FileControlProps {
  readonly model: FileArtifactModel;
  readonly onAllowApproval: FileArtifactProps["onAllowApproval"];
  readonly onApproveProposal: FileArtifactProps["onApproveProposal"];
  readonly onDiscard: FileArtifactProps["onDiscard"];
  readonly onDenyApproval: FileArtifactProps["onDenyApproval"];
  readonly onEdit: FileArtifactProps["onEdit"];
  readonly onRecoverDraft: FileArtifactProps["onRecoverDraft"];
  readonly onRejectProposal: FileArtifactProps["onRejectProposal"];
  readonly onResolveConflict: FileArtifactProps["onResolveConflict"];
  readonly onSave: FileArtifactProps["onSave"];
}

/** Selects phase-specific controls without taking ownership of provider state. */
function fileControls({
  model,
  onAllowApproval,
  onApproveProposal,
  onDiscard,
  onDenyApproval,
  onEdit,
  onRecoverDraft,
  onRejectProposal,
  onResolveConflict,
  onSave,
}: FileControlProps): ReactNode {
  const { artifactId, state } = model;
  if (state.phase === "read") {
    return (
      <button
        disabled={onEdit === undefined}
        onClick={() => onEdit?.(artifactId)}
        type="button"
      >
        Edit
      </button>
    );
  }
  if (state.phase === "edit" || state.phase === "dirty") {
    return (
      <>
        <button
          disabled={onDiscard === undefined}
          onClick={() => onDiscard?.(artifactId)}
          type="button"
        >
          Discard
        </button>
        <button
          disabled={state.phase !== "dirty" || onSave === undefined}
          onClick={() => onSave?.(artifactId, state.draft)}
          type="button"
        >
          Save
        </button>
      </>
    );
  }
  if (state.phase === "proposed") {
    return (
      <>
        <button
          disabled={onRejectProposal === undefined}
          onClick={() => onRejectProposal?.(artifactId)}
          type="button"
        >
          Reject
        </button>
        <button
          disabled={onApproveProposal === undefined}
          onClick={() => onApproveProposal?.(artifactId)}
          type="button"
        >
          Approve
        </button>
      </>
    );
  }
  if (state.phase === "approval") {
    return (
      <>
        <button
          disabled={onDenyApproval === undefined}
          onClick={() => onDenyApproval?.(artifactId, state.approvalId)}
          type="button"
        >
          Deny
        </button>
        <button
          disabled={onAllowApproval === undefined}
          onClick={() => onAllowApproval?.(artifactId, state.approvalId)}
          type="button"
        >
          Allow
        </button>
      </>
    );
  }
  if (state.phase === "conflict") {
    return (
      <>
        <button
          disabled={onResolveConflict === undefined}
          onClick={() => onResolveConflict?.(artifactId, "reload-current")}
          type="button"
        >
          Reload current
        </button>
        <button
          disabled={onResolveConflict === undefined}
          onClick={() => onResolveConflict?.(artifactId, "keep-draft")}
          type="button"
        >
          Keep draft
        </button>
      </>
    );
  }
  if (state.phase === "recovery") {
    return (
      <>
        <button
          disabled={onDiscard === undefined}
          onClick={() => onDiscard?.(artifactId)}
          type="button"
        >
          Discard recovered draft
        </button>
        <button
          disabled={onRecoverDraft === undefined}
          onClick={() => onRecoverDraft?.(artifactId)}
          type="button"
        >
          Recover draft
        </button>
      </>
    );
  }
  return null;
}

interface FileStateBodyProps {
  readonly artifactId: string;
  readonly bodyRef: RefObject<HTMLDivElement | null>;
  readonly context: ArtifactContext;
  readonly onDraftChange: FileArtifactProps["onDraftChange"];
  readonly state: FileArtifactState;
  readonly title: string;
}

/** Renders read, edit, proposal, approval, conflict, and recovery bodies. */
function FileStateBody({
  artifactId,
  bodyRef,
  context,
  onDraftChange,
  state,
  title,
}: FileStateBodyProps) {
  const editable =
    context !== "contextual" &&
    (state.phase === "edit" || state.phase === "dirty");
  const visibleContent = fileVisibleContent(state);

  return (
    <div
      className="artifact-file__body"
      data-file-phase={state.phase}
      ref={bodyRef}
    >
      <FileStateNotice state={state} />
      {(state.phase === "proposed" || state.phase === "approval") &&
      state.proposalDiff ? (
        <pre
          className="artifact-file__proposal-diff"
          aria-label="Proposed File diff"
        >
          {state.proposalDiff}
        </pre>
      ) : null}
      {editable ? (
        <LineNumberedEditor
          artifactId={artifactId}
          content={visibleContent}
          onDraftChange={onDraftChange}
          title={title}
        />
      ) : (
        <LineNumberedReadView content={visibleContent} title={title} />
      )}
    </div>
  );
}

interface FileStateNoticeProps {
  readonly state: FileArtifactState;
}

/** Describes provider state explicitly so color is never the only signal. */
function FileStateNotice({ state }: FileStateNoticeProps) {
  if (state.phase === "proposed") {
    return <p role="status">Proposed change · {state.proposalSummary}</p>;
  }
  if (state.phase === "approval") {
    return <p role="status">Approval pending · {state.approvalSummary}</p>;
  }
  if (state.phase === "conflict") {
    return (
      <p role="alert">
        Save conflict · {state.conflictMessage} · {state.currentVersionLabel}
      </p>
    );
  }
  if (state.phase === "recovery") {
    return <p role="status">Recovered draft · {state.recoveryMessage}</p>;
  }
  if (state.phase === "dirty") {
    return <p role="status">Unsaved changes</p>;
  }
  return null;
}

interface LineNumberedContentProps {
  readonly content: string;
  readonly title: string;
}

/** Renders source-preserving file text with a fixed semantic line sequence. */
function LineNumberedReadView({ content, title }: LineNumberedContentProps) {
  const lines = fileLines(content);
  return (
    <ol
      aria-label={`Contents of ${title}`}
      className="artifact-file__read-lines"
    >
      {lines.map((line, index) => (
        <li key={`${index}-${line}`}>
          <code>{line.length === 0 ? " " : line}</code>
        </li>
      ))}
    </ol>
  );
}

interface LineNumberedEditorProps extends LineNumberedContentProps {
  readonly artifactId: string;
  readonly onDraftChange: FileArtifactProps["onDraftChange"];
}

/** Keeps line numbers beside the controlled source textarea while editing. */
function LineNumberedEditor({
  artifactId,
  content,
  onDraftChange,
  title,
}: LineNumberedEditorProps) {
  const lines = fileLines(content);

  /** Publishes source changes without retaining an independent component draft. */
  const handleChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    onDraftChange?.(artifactId, event.currentTarget.value);
  };

  return (
    <div className="artifact-file__editor">
      <ol aria-hidden="true" className="artifact-file__line-numbers">
        {lines.map((_line, index) => (
          <li key={index}>{index + 1}</li>
        ))}
      </ol>
      <textarea
        aria-label={`Edit ${title}`}
        onChange={handleChange}
        spellCheck={false}
        value={content}
      />
    </div>
  );
}

/** Selects the exact content represented by each controlled provider phase. */
function fileVisibleContent(state: FileArtifactState): string {
  if (
    state.phase === "edit" ||
    state.phase === "dirty" ||
    state.phase === "conflict" ||
    state.phase === "recovery"
  ) {
    return state.draft;
  }
  if (state.phase === "proposed" || state.phase === "approval") {
    return state.proposedContent;
  }
  return state.content;
}

/** Preserves an empty final line so line numbers match the source exactly. */
function fileLines(content: string): readonly string[] {
  return content.split("\n");
}

/** Provides compact, non-color state copy for the static artifact footer. */
function filePhaseLabel(phase: FileArtifactState["phase"]): string {
  const labels: Record<FileArtifactState["phase"], string> = {
    read: "Read only",
    edit: "Editing",
    dirty: "Unsaved changes",
    proposed: "Proposed change",
    approval: "Approval pending",
    conflict: "Save conflict",
    recovery: "Recovery available",
  };
  return labels[phase];
}
