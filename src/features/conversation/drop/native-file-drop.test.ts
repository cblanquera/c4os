import { describe, expect, it, vi } from "vitest";

import {
  CONVERSATION_FILE_DROP_EVENT,
  ConversationFileDropBoundaryError,
  createConversationFileDropSubscriber,
  parseConversationFileDropEvent,
} from "./native-file-drop";

const grant = {
  grantId: "picker-grant-1",
  objectKind: "file",
  displayName: "reference.png",
} as const;

describe("native conversation file-drop boundary", () => {
  it("accepts the exact phase and opaque file-grant shape", () => {
    expect(
      parseConversationFileDropEvent({ phase: "enter", grants: [grant] }),
    ).toEqual({ phase: "enter", grants: [grant] });
    expect(
      parseConversationFileDropEvent({ phase: "leave", grants: [] }),
    ).toEqual({ phase: "leave", grants: [] });
    expect(
      parseConversationFileDropEvent({ phase: "drop", grants: [grant] }),
    ).toEqual({ phase: "drop", grants: [grant] });
  });

  it.each([
    { phase: "over", grants: [grant] },
    { phase: "enter", grants: [] },
    { phase: "leave", grants: [grant] },
    { phase: "drop", grants: [{ ...grant, objectKind: "folder" }] },
    { phase: "drop", grants: [{ ...grant, path: "/Users/private/file" }] },
    {
      phase: "drop",
      grants: [{ ...grant, displayName: "folder/reference.png" }],
    },
    { phase: "drop", grants: [grant, grant] },
    { phase: "drop", grants: [grant], paths: ["/Users/private/file"] },
  ])("rejects unsafe or malformed native payload %#", (payload) => {
    expect(() => parseConversationFileDropEvent(payload)).toThrow(
      ConversationFileDropBoundaryError,
    );
  });

  it("subscribes only to the C4OS event and quarantines invalid payloads", async () => {
    let nativeHandler:
      ((event: { readonly payload: unknown }) => void) | undefined;
    const unlisten = vi.fn();
    const source = {
      listen: vi.fn(async (_eventName, handler) => {
        nativeHandler = handler;
        return unlisten;
      }),
    };
    const onEvent = vi.fn();
    const onInvalidEvent = vi.fn();
    const subscribe = createConversationFileDropSubscriber(source);

    const dispose = await subscribe(onEvent, onInvalidEvent);
    expect(source.listen).toHaveBeenCalledWith(
      CONVERSATION_FILE_DROP_EVENT,
      expect.any(Function),
    );

    nativeHandler?.({ payload: { phase: "drop", grants: [grant] } });
    expect(onEvent).toHaveBeenCalledWith({ phase: "drop", grants: [grant] });

    nativeHandler?.({
      payload: {
        phase: "drop",
        grants: [{ ...grant, path: "/Users/private/file" }],
      },
    });
    expect(onEvent).toHaveBeenCalledTimes(1);
    expect(onInvalidEvent).toHaveBeenCalledWith(
      expect.any(ConversationFileDropBoundaryError),
    );

    dispose();
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
