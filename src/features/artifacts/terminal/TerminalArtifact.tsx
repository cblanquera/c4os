import { useEffect, useRef } from "react";
import type { FormEvent } from "react";

import type { ArtifactContext } from "../types";
import { ArtifactShell } from "../ui/ArtifactShell";
import { TerminalViewport } from "./TerminalViewport";
import {
  boundedTerminalSelection,
  type TerminalArtifactModel,
  type TerminalReplySelection,
  type TerminalResizeIntent,
  type TerminalRunIntent,
  type TerminalStdinIntent,
} from "./terminal-types";
import "./terminal-artifact.css";

export interface TerminalArtifactProps {
  readonly context: ArtifactContext;
  readonly model: TerminalArtifactModel;
  readonly onAllowApproval?: (artifactId: string, approvalId: string) => void;
  readonly onClose?: (artifactId: string) => void;
  readonly onCopy?: (artifactId: string, value: string) => void;
  readonly onExpand?: (artifactId: string) => void;
  readonly onOutputWritten?: (outputSequence: number) => void;
  readonly onPromptValueChange: (value: string) => void;
  readonly onDenyApproval?: (artifactId: string, approvalId: string) => void;
  readonly onReply?: (
    artifactId: string,
    selection?: TerminalReplySelection,
  ) => void;
  readonly onResize?: (intent: TerminalResizeIntent) => void;
  readonly onRunNext?: (intent: TerminalRunIntent) => void;
  readonly onStdinValueChange: (value: string) => void;
  readonly onStop?: (artifactId: string, commandId: string) => void;
  readonly onSubmitStdin?: (intent: TerminalStdinIntent) => void;
  readonly promptValue: string;
  readonly reducedMotion: boolean;
  readonly stdinValue: string;
}

/** Renders one controlled immutable command card and its focused shell view. */
export function TerminalArtifact({
  context,
  model,
  onAllowApproval,
  onClose,
  onCopy,
  onExpand,
  onOutputWritten,
  onPromptValueChange,
  onDenyApproval,
  onReply,
  onResize,
  onRunNext,
  onStdinValueChange,
  onStop,
  onSubmitStdin,
  promptValue,
  reducedMotion,
  stdinValue,
}: TerminalArtifactProps) {
  const outputHostRef = useRef<HTMLPreElement>(null);
  const focusedSelectionRef = useRef("");
  const outputText = model.outputText;
  const copyValue = [model.command, outputText]
    .filter((value) => value.length > 0)
    .join("\n");
  const isFocused = context === "focused";
  const isContextual = context === "contextual";
  const approvalId = model.pendingApprovalId;
  const hasPendingApproval = approvalId !== null;
  const showStop =
    !isContextual &&
    !hasPendingApproval &&
    model.stopAvailable &&
    (model.phase === "running" || model.phase === "stdinReady");
  const showStdin =
    isFocused &&
    !hasPendingApproval &&
    model.phase === "stdinReady" &&
    model.stdinReady;
  const showPrompt =
    isFocused &&
    !hasPendingApproval &&
    model.promptReady &&
    (model.phase === "completed" ||
      model.phase === "interrupted" ||
      model.phase === "failed" ||
      model.phase === "recovery");
  useEffect(() => {
    focusedSelectionRef.current = "";
  }, [model.commandId]);

  const submitStdin = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!showStdin || onSubmitStdin === undefined) return;
    onSubmitStdin({
      artifactId: model.artifactId,
      commandId: model.commandId,
      input: stdinValue,
    });
  };
  const runNext = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (
      !showPrompt ||
      promptValue.trim().length === 0 ||
      onRunNext === undefined
    ) {
      return;
    }
    onRunNext({
      artifactId: model.artifactId,
      terminalSessionId: model.terminalSessionId,
      command: promptValue,
    });
  };

  return (
    <ArtifactShell
      context={context}
      focusSupported
      footerContent={
        <span className="artifact-terminal__footer-status">
          {terminalPhaseLabel(model)}
        </span>
      }
      headerContent={
        <TerminalMetadata
          approvalId={approvalId}
          model={model}
          onAllowApproval={
            approvalId && onAllowApproval
              ? () => onAllowApproval(model.artifactId, approvalId)
              : undefined
          }
          onDenyApproval={
            approvalId && onDenyApproval
              ? () => onDenyApproval(model.artifactId, approvalId)
              : undefined
          }
          onStop={
            showStop && onStop
              ? () => onStop(model.artifactId, model.commandId)
              : undefined
          }
        />
      }
      identity={{
        accessibleLabel: `Terminal response artifact: ${model.command}`,
        id: model.artifactId,
        title: model.command,
        typeLabel: "Terminal",
      }}
      onClose={onClose ? () => onClose(model.artifactId) : undefined}
      onCopy={onCopy ? () => onCopy(model.artifactId, copyValue) : undefined}
      onExpand={onExpand ? () => onExpand(model.artifactId) : undefined}
      onReply={
        onReply
          ? () => {
              const selectedText = isFocused
                ? focusedSelectionRef.current
                : selectionInside(outputHostRef.current);
              const selection = boundedTerminalSelection(
                selectedText,
                outputText,
              );
              if (selection === undefined) onReply(model.artifactId);
              else onReply(model.artifactId, selection);
            }
          : undefined
      }
      {...(model.status === undefined ? {} : { status: model.status })}
    >
      <div
        className="artifact-terminal"
        data-terminal-context={context}
        data-terminal-phase={model.phase}
        data-terminal-session-id={model.terminalSessionId}
      >
        {isFocused ? (
          <TerminalViewport
            accessibleLabel={`Output for ${model.command}`}
            columns={model.columns}
            droppedBytes={model.droppedBytes}
            onOutputWritten={onOutputWritten}
            onResize={
              onResize
                ? (columns, rows) =>
                    onResize({
                      artifactId: model.artifactId,
                      terminalSessionId: model.terminalSessionId,
                      columns,
                      rows,
                    })
                : undefined
            }
            onSelectionChange={(selectedText) => {
              focusedSelectionRef.current = selectedText;
            }}
            outputBase64={model.outputBase64}
            outputSequence={model.outputSequence}
            reducedMotion={reducedMotion}
            retainedBytes={model.retainedBytes}
            rows={model.rows}
            snapshotKey={model.commandId}
          />
        ) : (
          <pre
            aria-label={`Output for ${model.command}`}
            className="artifact-terminal__output"
            data-terminal-output-sequence={model.outputSequence}
            ref={outputHostRef}
            tabIndex={0}
          >
            {outputText}
          </pre>
        )}

        {model.droppedBytes > 0 ? (
          <p className="artifact-terminal__retention" role="status">
            {`Earlier output omitted · ${model.droppedBytes.toLocaleString()} bytes dropped`}
          </p>
        ) : null}

        {model.statusMessage ? (
          <p
            className="artifact-terminal__message"
            role={model.phase === "failed" ? "alert" : "status"}
          >
            {model.statusMessage}
          </p>
        ) : null}

        <p
          aria-atomic="true"
          aria-live="polite"
          className="artifact-terminal__phase"
          role="status"
        >
          {terminalPhaseLabel(model)}
        </p>

        {showStdin ? (
          <form
            aria-label={`Input for ${model.command}`}
            className="artifact-terminal__input"
            onSubmit={submitStdin}
          >
            <label>
              <span>Process input</span>
              <input
                autoComplete="off"
                onChange={(event) =>
                  onStdinValueChange(event.currentTarget.value)
                }
                type="text"
                value={stdinValue}
              />
            </label>
            <button disabled={onSubmitStdin === undefined} type="submit">
              Send input
            </button>
          </form>
        ) : null}

        {showPrompt ? (
          <form
            aria-label="Terminal prompt"
            className="artifact-terminal__input artifact-terminal__prompt"
            onSubmit={runNext}
          >
            <label>
              <span aria-hidden="true">$</span>
              <span className="artifact-terminal__visually-hidden">
                Next command
              </span>
              <input
                autoComplete="off"
                onChange={(event) =>
                  onPromptValueChange(event.currentTarget.value)
                }
                type="text"
                value={promptValue}
              />
            </label>
            <button
              disabled={
                onRunNext === undefined || promptValue.trim().length === 0
              }
              type="submit"
            >
              Run
            </button>
          </form>
        ) : null}
      </div>
    </ArtifactShell>
  );
}

interface TerminalMetadataProps {
  readonly approvalId: string | null;
  readonly model: TerminalArtifactModel;
  readonly onAllowApproval?: (() => void) | undefined;
  readonly onDenyApproval?: (() => void) | undefined;
  readonly onStop?: (() => void) | undefined;
}

function TerminalMetadata({
  approvalId,
  model,
  onAllowApproval,
  onDenyApproval,
  onStop,
}: TerminalMetadataProps) {
  return (
    <div className="artifact-terminal__metadata">
      <span className="artifact-terminal__location">
        <strong>{model.workingDirectoryDisplay}</strong>
        <span>{model.environmentId}</span>
      </span>
      <details>
        <summary>Terminal details</summary>
        <dl>
          <dt>Session</dt>
          <dd>{model.terminalSessionId}</dd>
          <dt>Shell</dt>
          <dd>{model.shellPath}</dd>
          <dt>Environment generation</dt>
          <dd>{model.environmentGeneration}</dd>
          <dt>Process generation</dt>
          <dd>{model.processGeneration}</dd>
          <dt>Shell process</dt>
          <dd>{model.shellProcessId ?? "Unavailable"}</dd>
          <dt>Foreground process group</dt>
          <dd>{model.foregroundProcessGroupId ?? "None"}</dd>
          <dt>Dimensions</dt>
          <dd>
            {model.columns} × {model.rows}
          </dd>
        </dl>
      </details>
      {approvalId ? (
        <span
          aria-label={`Approval for ${model.command}`}
          className="artifact-terminal__approval"
          role="group"
        >
          <button
            disabled={onDenyApproval === undefined}
            onClick={onDenyApproval}
            type="button"
          >
            Deny
          </button>
          <button
            disabled={onAllowApproval === undefined}
            onClick={onAllowApproval}
            type="button"
          >
            Allow
          </button>
        </span>
      ) : null}
      {onStop ? (
        <button
          aria-label={`Stop ${model.command}`}
          onClick={onStop}
          type="button"
        >
          Stop
        </button>
      ) : null}
    </div>
  );
}

function terminalPhaseLabel(model: TerminalArtifactModel): string {
  const phaseLabels = {
    queued: "Queued",
    approvalWaiting: "Waiting for approval",
    running: "Running",
    stdinReady: "Waiting for input",
    stopping: "Stopping…",
    completed:
      model.exitCode === null
        ? "Completed"
        : `Completed · exit ${model.exitCode}`,
    interrupted:
      model.exitCode === null
        ? "Interrupted"
        : `Interrupted · exit ${model.exitCode}`,
    failed:
      model.exitCode === null ? "Failed" : `Failed · exit ${model.exitCode}`,
    recovery: "Recovery required",
  } satisfies Record<TerminalArtifactModel["phase"], string>;
  return `${phaseLabels[model.phase]}${model.shellReplaced ? " · shell replaced" : ""}`;
}

function selectionInside(host: HTMLElement | null): string {
  if (host === null) return "";
  const selection = window.getSelection();
  if (selection === null || selection.rangeCount === 0) return "";
  const range = selection.getRangeAt(0);
  if (!host.contains(range.commonAncestorContainer)) return "";
  return selection.toString();
}
