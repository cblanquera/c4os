import { useLayoutEffect, useRef, useState, type ReactNode } from "react";

import { SafeMarkdown } from "./SafeMarkdown";
import type {
  AssistantTranscriptTurn,
  ConversationTranscriptPlacement,
  ConversationTranscriptTurn,
  TranscriptArtifactPresentation,
} from "./types";
import "./conversation-ui.css";

export interface ConversationTranscriptProps {
  readonly focusedArtifactId?: string | null;
  readonly onArtifactFocusRequest?: (artifactId: string) => void;
  readonly onCancelAttempt?: (attemptId: string) => void;
  readonly onCopy: (turnId: string, markdownSource: string) => void;
  readonly onLinkActivate?: (href: string) => void;
  readonly onProvenanceExpandedChange: (
    turnId: string,
    isExpanded: boolean,
  ) => void;
  readonly onReply: (turnId: string, markdownSource: string) => void;
  readonly onRetryAttempt?: (attemptId: string) => void;
  readonly onWorkExpandedChange: (turnId: string, isExpanded: boolean) => void;
  readonly placement?: ConversationTranscriptPlacement;
  readonly reducedMotion: boolean;
  readonly turns: readonly ConversationTranscriptTurn[];
}

/** Renders immutable turns and delegates every operation to authoritative owners. */
export function ConversationTranscript({
  focusedArtifactId = null,
  onArtifactFocusRequest,
  onCancelAttempt,
  onCopy,
  onLinkActivate,
  onProvenanceExpandedChange,
  onReply,
  onRetryAttempt,
  onWorkExpandedChange,
  placement = "center",
  reducedMotion,
  turns,
}: ConversationTranscriptProps) {
  const transcript = useRef<HTMLElement>(null);
  const scrollHost = useRef<HTMLElement | null>(null);
  const [scrollState, setScrollState] = useState({
    distanceFromBottom: 0,
    isOverflowing: false,
  });

  useLayoutEffect(() => {
    const transcriptElement = transcript.current;
    if (transcriptElement === null) return;
    const host =
      transcriptElement.closest<HTMLElement>(".shell-workspace__stage") ??
      transcriptElement.closest<HTMLElement>(".shell-project-panel__context") ??
      transcriptElement;
    scrollHost.current = host;

    const measure = () => {
      const distanceFromBottom = Math.max(
        0,
        host.scrollHeight - host.clientHeight - host.scrollTop,
      );
      const next = {
        distanceFromBottom,
        isOverflowing: host.scrollHeight > host.clientHeight,
      };
      setScrollState((current) =>
        current.distanceFromBottom === next.distanceFromBottom &&
        current.isOverflowing === next.isOverflowing
          ? current
          : next,
      );
    };

    measure();
    host.addEventListener("scroll", measure, { passive: true });
    window.addEventListener("resize", measure);
    const resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(measure);
    resizeObserver?.observe(host);
    resizeObserver?.observe(transcriptElement);
    return () => {
      host.removeEventListener("scroll", measure);
      window.removeEventListener("resize", measure);
      resizeObserver?.disconnect();
      if (scrollHost.current === host) scrollHost.current = null;
    };
  }, [placement, turns]);

  const { distanceFromBottom, isOverflowing } = scrollState;
  const showScrollToLatest = isOverflowing && distanceFromBottom > 24;

  const scrollToLatest = () => {
    const host = scrollHost.current;
    if (host === null) return;
    host.scrollTo({
      behavior: reducedMotion ? "auto" : "smooth",
      top: host.scrollHeight,
    });
  };

  return (
    <section
      ref={transcript}
      className="conversation-transcript"
      aria-label={
        placement === "context-pane"
          ? "Contextual conversation"
          : "Conversation"
      }
      data-focused-artifact-id={focusedArtifactId ?? "none"}
      data-placement={placement}
      data-reduced-motion={reducedMotion}
    >
      <div className="conversation-transcript__turns" role="feed">
        {turns.map((turn) =>
          turn.author === "assistant" ? (
            <AssistantTurn
              key={turn.id}
              onArtifactFocusRequest={onArtifactFocusRequest}
              onCancelAttempt={onCancelAttempt}
              onCopy={onCopy}
              onLinkActivate={onLinkActivate}
              onProvenanceExpandedChange={onProvenanceExpandedChange}
              onReply={onReply}
              onRetryAttempt={onRetryAttempt}
              onWorkExpandedChange={onWorkExpandedChange}
              placement={placement}
              turn={turn}
            />
          ) : (
            <article
              key={turn.id}
              className="conversation-turn conversation-turn--user"
              aria-label="User message"
              data-status={turn.status}
            >
              <div className="conversation-turn__bubble">
                <SafeMarkdown
                  source={turn.markdownSource}
                  {...(onLinkActivate === undefined ? {} : { onLinkActivate })}
                />
              </div>
              {turn.attachments && turn.attachments.length > 0 ? (
                <ul
                  aria-label="Submitted attachments"
                  className="conversation-turn__attachments"
                >
                  {turn.attachments.map((attachment) => (
                    <li key={attachment.id}>
                      <span aria-hidden="true">
                        {attachment.referenceNumber}
                      </span>
                      <strong>{attachment.name}</strong>
                      <small>{attachment.metadata}</small>
                    </li>
                  ))}
                </ul>
              ) : null}
              {turn.replyContext ? (
                <ArtifactReplyContextDisclosure context={turn.replyContext} />
              ) : null}
              <TurnActions
                markdownSource={turn.markdownSource}
                onCopy={onCopy}
                onReply={onReply}
                turnId={turn.id}
              />
            </article>
          ),
        )}
      </div>
      {showScrollToLatest ? (
        <button
          type="button"
          className="conversation-transcript__latest"
          aria-label="Scroll to latest message"
          onClick={scrollToLatest}
        >
          ↓
        </button>
      ) : null}
    </section>
  );
}

interface ArtifactReplyContextDisclosureProps {
  readonly context: NonNullable<
    Extract<ConversationTranscriptTurn, { author: "user" }>["replyContext"]
  >;
}

/** Exposes the exact immutable Artifact context attached to a submitted turn. */
function ArtifactReplyContextDisclosure({
  context,
}: ArtifactReplyContextDisclosureProps) {
  return (
    <details className="conversation-turn__artifact-context">
      <summary>
        Reply to {context.providerType} ·{" "}
        {formatContextBytes(context.suppliedBytes)}
        {context.truncated ? " · truncated" : ""}
        {context.unsaved ? " · includes unsaved draft" : ""}
      </summary>
      <dl>
        <div>
          <dt>Artifact</dt>
          <dd>{context.artifactId}</dd>
        </div>
        <div>
          <dt>Version</dt>
          <dd>
            Provider {context.providerVersion} · record {context.recordRevision}
          </dd>
        </div>
        <div>
          <dt>Context budget</dt>
          <dd>
            {formatContextBytes(context.suppliedBytes)} of{" "}
            {formatContextBytes(context.maximumBytes)}
            {context.omittedBytes > 0
              ? ` · ${formatContextBytes(context.omittedBytes)} omitted`
              : " · complete"}
          </dd>
        </div>
        <div>
          <dt>Stable reference</dt>
          <dd className="conversation-turn__artifact-reference">
            {context.stableReference}
          </dd>
        </div>
      </dl>
      <div className="conversation-turn__artifact-segments">
        {context.segments.map((segment, index) => (
          <section key={`${segment.source}:${index}`}>
            <h4>{segment.source}</h4>
            {segment.text.length > 0 ? <pre>{segment.text}</pre> : null}
            {segment.omittedBytes > 0 ? (
              <small>{formatContextBytes(segment.omittedBytes)} omitted</small>
            ) : null}
          </section>
        ))}
      </div>
      <ul aria-label="Artifact Reply capabilities">
        {context.capabilities.map((capability) => (
          <li key={capability.capabilityId}>
            {capability.capabilityId}: {capability.access}
            {capability.reasonCode ? ` (${capability.reasonCode})` : ""}
          </li>
        ))}
      </ul>
    </details>
  );
}

function formatContextBytes(value: number): string {
  return value < 1_024
    ? `${value} B`
    : `${new Intl.NumberFormat("en", { maximumFractionDigits: 1 }).format(value / 1_024)} KB`;
}

interface AssistantTurnProps {
  readonly onArtifactFocusRequest: ((artifactId: string) => void) | undefined;
  readonly onCancelAttempt: ((attemptId: string) => void) | undefined;
  readonly onCopy: ConversationTranscriptProps["onCopy"];
  readonly onLinkActivate: ConversationTranscriptProps["onLinkActivate"];
  readonly onProvenanceExpandedChange: ConversationTranscriptProps["onProvenanceExpandedChange"];
  readonly onReply: ConversationTranscriptProps["onReply"];
  readonly onRetryAttempt: ((attemptId: string) => void) | undefined;
  readonly onWorkExpandedChange: ConversationTranscriptProps["onWorkExpandedChange"];
  readonly placement: ConversationTranscriptPlacement;
  readonly turn: AssistantTranscriptTurn;
}

/** Preserves the accepted work, identity, content, and action order. */
function AssistantTurn({
  onArtifactFocusRequest,
  onCancelAttempt,
  onCopy,
  onLinkActivate,
  onProvenanceExpandedChange,
  onReply,
  onRetryAttempt,
  onWorkExpandedChange,
  placement,
  turn,
}: AssistantTurnProps) {
  const workExpanded = turn.status === "streaming" || turn.work.isExpanded;
  const responseVisible = turn.responseVisible ?? turn.status !== "streaming";

  return (
    <article
      className="conversation-turn conversation-turn--assistant"
      aria-busy={turn.status === "streaming"}
      aria-label="C4OS response"
      data-status={turn.status}
    >
      <section className="conversation-work" aria-label="Assistant work">
        <button
          type="button"
          className="conversation-work__trigger"
          aria-expanded={workExpanded}
          onClick={() => onWorkExpandedChange(turn.id, !workExpanded)}
        >
          <span>
            {turn.work.kind === "reasoning" ? "Reasoning" : "Activity"}
          </span>
          <strong>
            {turn.status === "streaming"
              ? turn.work.summary
              : (turn.work.durationLabel ?? turn.work.summary)}
          </strong>
        </button>
        {workExpanded ? (
          <div className="conversation-work__details">
            {turn.work.progress.map((progress, index) => (
              <p key={`${turn.id}-progress-${index}`}>{progress}</p>
            ))}
            {turn.work.details.length > 0 ? (
              <ul aria-label="Work activity">
                {turn.work.details.map((detail) => (
                  <li key={detail.id} data-state={detail.state ?? "completed"}>
                    <span>{detail.label}</span>
                    {detail.detail ? <small>{detail.detail}</small> : null}
                  </li>
                ))}
              </ul>
            ) : null}
          </div>
        ) : null}
        {turn.status === "streaming" && onCancelAttempt ? (
          <button
            className="conversation-work__cancel"
            onClick={() => onCancelAttempt(turn.id)}
            type="button"
          >
            Stop response
          </button>
        ) : null}
      </section>

      {responseVisible ? (
        <>
          <div className="conversation-turn__identity">
            <strong>C4OS</strong>
            <span>{turn.modelLabel}</span>
            <button
              type="button"
              aria-expanded={turn.provenance.isExpanded}
              onClick={() =>
                onProvenanceExpandedChange(turn.id, !turn.provenance.isExpanded)
              }
            >
              Run details
            </button>
          </div>
          {turn.provenance.isExpanded ? (
            <dl className="conversation-turn__provenance">
              <div>
                <dt>Runtime</dt>
                <dd>{turn.provenance.runtime}</dd>
              </div>
              <div>
                <dt>Adapter</dt>
                <dd>{turn.provenance.adapter}</dd>
              </div>
              <div>
                <dt>Environment</dt>
                <dd>{turn.provenance.environment}</dd>
              </div>
              <div>
                <dt>Effective capabilities</dt>
                <dd>{turn.provenance.capabilitySummary}</dd>
              </div>
            </dl>
          ) : null}
          {turn.markdownSource.length > 0 ? (
            <div className="conversation-turn__bubble">
              <SafeMarkdown
                source={turn.markdownSource}
                {...(onLinkActivate === undefined ? {} : { onLinkActivate })}
              />
            </div>
          ) : null}
          {turn.artifact ? (
            <ArtifactPresentationHook
              artifact={turn.artifact}
              onFocusRequest={onArtifactFocusRequest}
              placement={placement}
            />
          ) : null}
          <TurnActions
            markdownSource={turn.markdownSource}
            onCopy={onCopy}
            onReply={onReply}
            turnId={turn.id}
          />
          {turn.status === "failed" && onRetryAttempt ? (
            <button
              className="conversation-turn__retry"
              onClick={() => onRetryAttempt(turn.id)}
              type="button"
            >
              Retry response
            </button>
          ) : null}
        </>
      ) : null}

      <span
        className="conversation-visually-hidden"
        aria-atomic="true"
        aria-live="polite"
        role="status"
      >
        {turn.announceCompletion && turn.status === "completed"
          ? "C4OS response complete."
          : ""}
      </span>
    </article>
  );
}

interface TurnActionsProps {
  readonly markdownSource: string;
  readonly onCopy: ConversationTranscriptProps["onCopy"];
  readonly onReply: ConversationTranscriptProps["onReply"];
  readonly turnId: string;
}

/** Reserves action geometry while keeping subdued controls in the tab order. */
function TurnActions({
  markdownSource,
  onCopy,
  onReply,
  turnId,
}: TurnActionsProps) {
  return (
    <div className="conversation-turn__actions" aria-label="Message actions">
      <button type="button" onClick={() => onCopy(turnId, markdownSource)}>
        Copy
      </button>
      <button type="button" onClick={() => onReply(turnId, markdownSource)}>
        Reply
      </button>
    </div>
  );
}

interface ArtifactPresentationHookProps {
  readonly artifact: TranscriptArtifactPresentation;
  readonly onFocusRequest: ((artifactId: string) => void) | undefined;
  readonly placement: ConversationTranscriptPlacement;
}

/** Exposes metadata and focus intent without implementing a facility provider. */
function ArtifactPresentationHook({
  artifact,
  onFocusRequest,
  placement,
}: ArtifactPresentationHookProps) {
  const providerContent = artifact.renderContent?.(placement);
  return (
    <section
      className="conversation-artifact-hook"
      aria-label={`${artifact.type} response artifact`}
      data-artifact-id={artifact.id}
      data-artifact-placement={placement}
      data-focused={artifact.isFocused}
    >
      {providerContent ?? (
        <div>
          <strong>{artifact.title}</strong>
          <span>{artifact.summary}</span>
        </div>
      )}
      {providerContent === undefined &&
      artifact.focusSupported &&
      !artifact.isFocused ? (
        <button
          type="button"
          data-artifact-focus-trigger={artifact.id}
          disabled={onFocusRequest === undefined}
          onClick={() => onFocusRequest?.(artifact.id)}
        >
          Expand
        </button>
      ) : null}
    </section>
  );
}

export interface PendingConversationPromptProps {
  readonly projectName: string;
  readonly supportingContent?: ReactNode;
}

/** Renders the memory-only pending Chat prompt before session promotion. */
export function PendingConversationPrompt({
  projectName,
  supportingContent,
}: PendingConversationPromptProps) {
  return (
    <section
      className="conversation-pending"
      aria-labelledby="pending-chat-title"
    >
      <h1 id="pending-chat-title">
        What do you want to build in {projectName}?
      </h1>
      {supportingContent}
    </section>
  );
}
