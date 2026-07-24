import { useEffect, useMemo, useRef, useState } from "react";

import type {
  ConversationFileDropEvent,
  ConversationFileDropGrant,
  ConversationFileDropInvalidListener,
  ConversationFileDropSubscriber,
} from "./native-file-drop";
import { subscribeToConversationFileDrop } from "./native-file-drop";

export interface ConversationFileDropState {
  readonly grants: readonly ConversationFileDropGrant[];
  readonly isDragging: boolean;
}

export interface UseConversationFileDropOptions {
  readonly isChatActive: boolean;
  readonly onDrop: (grants: readonly ConversationFileDropGrant[]) => void;
  readonly onInvalidEvent?: ConversationFileDropInvalidListener;
  readonly subscribe?: ConversationFileDropSubscriber;
}

const IDLE_STATE: ConversationFileDropState = {
  grants: [],
  isDragging: false,
};

interface SubscriptionFileDropState extends ConversationFileDropState {
  readonly subscription: symbol | null;
}

const INITIAL_STATE: SubscriptionFileDropState = {
  ...IDLE_STATE,
  subscription: null,
};

/** Tracks renderer-safe native drag phases only while ordinary Chat is active. */
export function useConversationFileDrop({
  isChatActive,
  onDrop,
  onInvalidEvent,
  subscribe = subscribeToConversationFileDrop,
}: UseConversationFileDropOptions): ConversationFileDropState {
  const [state, setState] = useState<SubscriptionFileDropState>(INITIAL_STATE);
  const activation = useMemo(
    () =>
      Symbol(`conversation-file-drop-${isChatActive ? "active" : "inactive"}`),
    [isChatActive],
  );
  const onDropRef = useRef(onDrop);
  const onInvalidEventRef = useRef(onInvalidEvent);

  useEffect(() => {
    onDropRef.current = onDrop;
  }, [onDrop]);

  useEffect(() => {
    onInvalidEventRef.current = onInvalidEvent;
  }, [onInvalidEvent]);

  useEffect(() => {
    if (!isChatActive) {
      return;
    }

    const subscription = activation;
    let isDisposed = false;
    let unlisten: (() => void) | undefined;

    const handleEvent = (event: ConversationFileDropEvent) => {
      if (isDisposed) {
        return;
      }
      if (event.phase === "enter") {
        setState({ grants: event.grants, isDragging: true, subscription });
        return;
      }

      setState({ ...IDLE_STATE, subscription });
      if (event.phase === "drop") {
        onDropRef.current(event.grants);
      }
    };

    void subscribe(handleEvent, (error) => {
      if (!isDisposed) {
        setState({ ...IDLE_STATE, subscription });
        onInvalidEventRef.current?.(error);
      }
    })
      .then((disposeSubscription) => {
        if (isDisposed) {
          disposeSubscription();
        } else {
          unlisten = disposeSubscription;
        }
      })
      .catch((error: unknown) => {
        if (!isDisposed) {
          setState({ ...IDLE_STATE, subscription });
          onInvalidEventRef.current?.(
            error instanceof Error
              ? error
              : new Error("The native file-drop listener was unavailable."),
          );
        }
      });

    return () => {
      isDisposed = true;
      unlisten?.();
    };
  }, [activation, isChatActive, subscribe]);

  return isChatActive && state.subscription === activation
    ? { grants: state.grants, isDragging: state.isDragging }
    : IDLE_STATE;
}
