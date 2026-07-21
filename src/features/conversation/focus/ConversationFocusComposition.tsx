import type { ReactElement, ReactNode } from "react";
import { cloneElement, useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

import type { ConversationTranscriptProps } from "../ui";
import "./conversation-focus.css";

// FocusedConversationArtifact carries only the renderer state needed to compose
// the authoritative provider surface into the workspace center.
export interface FocusedConversationArtifact {
  readonly content: ReactNode;
  readonly id: string;
  readonly ownsClose?: boolean;
  readonly title: string;
  readonly type: "browser" | "file" | "folder" | "terminal" | "unknown";
}

// ConversationFocusCompositionSlots let the shell place center and contextual
// content in its existing layout without giving this primitive layout authority.
export interface ConversationFocusCompositionSlots {
  readonly center: ReactNode;
  readonly contextualChat: ReactNode | null;
}

// ConversationFocusCompositionProps keep focus state and effects controlled by
// the coordinator while this component owns the one-transcript composition rule.
export interface ConversationFocusCompositionProps {
  readonly children: (slots: ConversationFocusCompositionSlots) => ReactNode;
  readonly focusedArtifact: FocusedConversationArtifact | null;
  readonly onCloseFocusedArtifact: (artifactId: string) => void;
  readonly onRestoreChat: () => void;
  readonly transcript: ReactElement<ConversationTranscriptProps>;
}

/** Composes one transcript into either the center or contextual Chat slot. */
export function ConversationFocusComposition({
  children,
  focusedArtifact,
  onCloseFocusedArtifact,
  onRestoreChat,
  transcript,
}: ConversationFocusCompositionProps) {
  const contextualChatTitleId = useId();
  const focusedArtifactTitleId = useId();
  const centerTranscriptMount = useRef<HTMLDivElement>(null);
  const contextualTranscriptMount = useRef<HTMLDivElement>(null);
  const [transcriptHost] = useState(() => {
    const host = document.createElement("div");
    host.className = "conversation-focus__transcript-host";
    return host;
  });
  const hasFocusedArtifact = focusedArtifact !== null;

  // The transcript element is cloned exactly once so its placement metadata
  // always agrees with the one slot that receives the real conversation tree.
  const placedTranscript = cloneElement(transcript, {
    focusedArtifactId: focusedArtifact?.id ?? null,
    placement: focusedArtifact === null ? "center" : "context-pane",
  });

  // Keep one stable portal container and move that container between the two
  // layout-owned mounts. React therefore preserves the original transcript
  // component tree, DOM nodes, selection, and scroll state across focus.
  useLayoutEffect(() => {
    const mount = hasFocusedArtifact
      ? contextualTranscriptMount.current
      : centerTranscriptMount.current;
    if (mount === null) return;
    mount.appendChild(transcriptHost);
    return () => {
      if (transcriptHost.parentNode === mount) {
        mount.removeChild(transcriptHost);
      }
    };
  }, [hasFocusedArtifact, transcriptHost]);

  const transcriptPortal = createPortal(
    placedTranscript,
    transcriptHost,
    "conversation-transcript",
  );

  // Ordinary Chat owns the center and does not reserve an empty contextual pane.
  if (focusedArtifact === null) {
    return (
      <>
        {children({
          center: (
            <section
              aria-label="Conversation center"
              className="conversation-focus__center"
              data-conversation-focus-state="chat"
            >
              <div
                className="conversation-focus__center-transcript"
                ref={centerTranscriptMount}
              />
            </section>
          ),
          contextualChat: null,
        })}
        {transcriptPortal}
      </>
    );
  }

  // Explicit artifact focus replaces center while the same transcript element
  // appears only in the shell's contextual Chat slot.
  return (
    <>
      {children({
        center: (
          <section
            {...(focusedArtifact.ownsClose
              ? { "aria-label": `Focused ${focusedArtifact.title}` }
              : { "aria-labelledby": focusedArtifactTitleId })}
            className="conversation-focus__artifact"
            data-artifact-id={focusedArtifact.id}
            data-artifact-type={focusedArtifact.type}
          >
            {focusedArtifact.ownsClose ? null : (
              <header className="conversation-focus__artifact-header">
                <h2 id={focusedArtifactTitleId}>{focusedArtifact.title}</h2>
                <button
                  type="button"
                  onClick={() => onCloseFocusedArtifact(focusedArtifact.id)}
                >
                  Close
                </button>
              </header>
            )}
            <div className="conversation-focus__artifact-content">
              {focusedArtifact.content}
            </div>
          </section>
        ),
        contextualChat: (
          <section
            aria-labelledby={contextualChatTitleId}
            className="conversation-focus__contextual-chat"
            data-focused-artifact-id={focusedArtifact.id}
          >
            <header className="conversation-focus__contextual-header">
              <h2 id={contextualChatTitleId}>Chat</h2>
              <span className="conversation-focus__contextual-actions">
                <button
                  type="button"
                  disabled
                  title="Detached Chat windows are not available"
                >
                  Detach Chat
                </button>
                <button type="button" onClick={onRestoreChat}>
                  Restore Chat
                </button>
              </span>
            </header>
            <div
              className="conversation-focus__contextual-transcript"
              ref={contextualTranscriptMount}
            />
          </section>
        ),
      })}
      {transcriptPortal}
    </>
  );
}
