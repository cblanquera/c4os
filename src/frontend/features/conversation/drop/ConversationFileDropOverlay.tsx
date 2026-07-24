import type { ConversationFileDropGrant } from "./native-file-drop";
import type {
  ConversationFileDropInvalidListener,
  ConversationFileDropSubscriber,
} from "./native-file-drop";
import { useConversationFileDrop } from "./useConversationFileDrop";
import "./conversation-file-drop.css";

export interface ConversationFileDropOverlayProps {
  readonly isChatActive: boolean;
  readonly onDrop: (grants: readonly ConversationFileDropGrant[]) => void;
  readonly onInvalidEvent?: ConversationFileDropInvalidListener;
  readonly subscribe?: ConversationFileDropSubscriber;
}

/** Covers the application with a non-interactive, accessible valid-file target. */
export function ConversationFileDropOverlay({
  isChatActive,
  onDrop,
  onInvalidEvent,
  subscribe,
}: ConversationFileDropOverlayProps) {
  const options = {
    isChatActive,
    onDrop,
    ...(onInvalidEvent ? { onInvalidEvent } : {}),
    ...(subscribe ? { subscribe } : {}),
  };
  const { grants, isDragging } = useConversationFileDrop(options);

  if (!isDragging) {
    return null;
  }

  const count = grants.length;
  return (
    <div
      aria-atomic="true"
      aria-live="polite"
      className="conversation-file-drop"
      data-file-count={count}
      role="status"
    >
      <div className="conversation-file-drop__target">
        <span aria-hidden="true" className="conversation-file-drop__icon">
          ↓
        </span>
        <strong>Drop {count === 1 ? "file" : "files"} to attach</strong>
        <span>
          {count === 1
            ? "This file will be added to your Chat draft."
            : `${count} files will be added to your Chat draft.`}
        </span>
      </div>
    </div>
  );
}
