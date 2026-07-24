import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import {
  MAX_IDENTIFIER_BYTES,
  type PickerGrantId,
} from "../../../platform/protocol";

export const CONVERSATION_FILE_DROP_EVENT =
  "c4os://conversation/file-drop" as const;

export type ConversationFileDropPhase = "enter" | "leave" | "drop";

export interface ConversationFileDropGrant {
  readonly grantId: PickerGrantId;
  readonly objectKind: "file";
  readonly displayName: string;
}

export interface ConversationFileDropEvent {
  readonly phase: ConversationFileDropPhase;
  readonly grants: readonly ConversationFileDropGrant[];
}

export type ConversationFileDropListener = (
  event: ConversationFileDropEvent,
) => void;

export type ConversationFileDropInvalidListener = (error: Error) => void;

export type ConversationFileDropSubscriber = (
  onEvent: ConversationFileDropListener,
  onInvalidEvent?: ConversationFileDropInvalidListener,
) => Promise<UnlistenFn>;

interface NativeEventSource {
  listen(
    eventName: typeof CONVERSATION_FILE_DROP_EVENT,
    handler: (event: { readonly payload: unknown }) => void,
  ): Promise<UnlistenFn>;
}

/** Identifies rejected native input without retaining its potentially sensitive value. */
export class ConversationFileDropBoundaryError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ConversationFileDropBoundaryError";
  }
}

const tauriEventSource: NativeEventSource = {
  listen(eventName, handler) {
    if (import.meta.env.VITE_C4OS_QA_FIXTURES === "1") {
      void eventName;
      void handler;
      return Promise.resolve(() => undefined);
    }
    return listen<unknown>(eventName, handler);
  },
};

/** Builds an injectable, payload-validating subscription over the native event source. */
export function createConversationFileDropSubscriber(
  source: NativeEventSource,
): ConversationFileDropSubscriber {
  return (onEvent, onInvalidEvent) =>
    source.listen(CONVERSATION_FILE_DROP_EVENT, ({ payload }) => {
      try {
        onEvent(parseConversationFileDropEvent(payload));
      } catch (error) {
        onInvalidEvent?.(
          error instanceof Error
            ? error
            : new ConversationFileDropBoundaryError(
                "The native file-drop event was invalid.",
              ),
        );
      }
    });
}

/** Subscribes to the C4OS-owned opaque-grant event, never Tauri path events. */
export const subscribeToConversationFileDrop =
  createConversationFileDropSubscriber(tauriEventSource);

/** Parses the exact renderer-safe wire shape and rejects filesystem-bearing data. */
export function parseConversationFileDropEvent(
  raw: unknown,
): ConversationFileDropEvent {
  const event = exactRecord(raw, ["phase", "grants"], "file-drop event");
  if (
    event.phase !== "enter" &&
    event.phase !== "leave" &&
    event.phase !== "drop"
  ) {
    throw invalidPayload("The native file-drop phase was invalid.");
  }
  if (!Array.isArray(event.grants) || event.grants.length > 64) {
    throw invalidPayload("The native file-drop grant count was invalid.");
  }
  if (
    (event.phase === "leave" && event.grants.length !== 0) ||
    (event.phase !== "leave" && event.grants.length === 0)
  ) {
    throw invalidPayload(
      "The native file-drop grants did not match their phase.",
    );
  }

  const grantIds = new Set<string>();
  const grants = event.grants.map((rawGrant) => {
    const grant = exactRecord(
      rawGrant,
      ["grantId", "objectKind", "displayName"],
      "file-drop grant",
    );
    const grantId = asIdentifier(grant.grantId, "file-drop grant ID");
    if (grant.objectKind !== "file" || grantIds.has(grantId)) {
      throw invalidPayload("The native file-drop grant was invalid.");
    }
    const displayName = asDisplayName(grant.displayName);
    grantIds.add(grantId);
    return {
      grantId: grantId as PickerGrantId,
      objectKind: "file",
      displayName,
    } satisfies ConversationFileDropGrant;
  });

  return {
    phase: event.phase,
    grants,
  } as ConversationFileDropEvent;
}

function exactRecord(
  value: unknown,
  expectedKeys: readonly string[],
  label: string,
): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw invalidPayload(`The native ${label} was invalid.`);
  }
  const record = value as Record<string, unknown>;
  const keys = Object.keys(record).sort();
  const expected = [...expectedKeys].sort();
  if (
    keys.length !== expected.length ||
    keys.some((key, index) => key !== expected[index])
  ) {
    throw invalidPayload(`The native ${label} fields were invalid.`);
  }
  return record;
}

function asIdentifier(value: unknown, label: string): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > MAX_IDENTIFIER_BYTES ||
    !/^[A-Za-z0-9_.:@-]+$/u.test(value)
  ) {
    throw invalidPayload(`The native ${label} was invalid.`);
  }
  return value;
}

function asDisplayName(value: unknown): string {
  if (
    typeof value !== "string" ||
    value.trim().length === 0 ||
    value.length > 512 ||
    Array.from(value).some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return (
        character === "/" ||
        character === "\\" ||
        codePoint < 32 ||
        codePoint === 127
      );
    })
  ) {
    throw invalidPayload("The native file-drop display name was invalid.");
  }
  return value;
}

function invalidPayload(message: string): ConversationFileDropBoundaryError {
  return new ConversationFileDropBoundaryError(message);
}
