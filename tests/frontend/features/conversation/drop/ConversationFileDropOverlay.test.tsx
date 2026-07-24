import { act, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConversationFileDropOverlay } from "../../../../../src/frontend/features/conversation/drop/ConversationFileDropOverlay";
import type {
  ConversationFileDropEvent,
  ConversationFileDropGrant,
  ConversationFileDropListener,
  ConversationFileDropSubscriber,
} from "../../../../../src/frontend/features/conversation/drop/native-file-drop";

const grants = [
  {
    grantId: "picker-grant-1",
    objectKind: "file",
    displayName: "brief.md",
  },
  {
    grantId: "picker-grant-2",
    objectKind: "file",
    displayName: "screen.png",
  },
] as unknown as readonly ConversationFileDropGrant[];

function createSubscriptionHarness(): {
  readonly emit: (event: ConversationFileDropEvent) => void;
  readonly subscribe: ConversationFileDropSubscriber;
  readonly unlisten: ReturnType<typeof vi.fn>;
} {
  let listener: ConversationFileDropListener | undefined;
  const unlisten = vi.fn();
  const subscribe: ConversationFileDropSubscriber = vi.fn(
    async (nextListener) => {
      listener = nextListener;
      return unlisten;
    },
  );
  return {
    emit(event) {
      if (!listener) {
        throw new Error("The file-drop listener is not ready.");
      }
      listener(event);
    },
    subscribe,
    unlisten,
  };
}

describe("ConversationFileDropOverlay", () => {
  it("shows a full-window live status for valid drag and delivers opaque grants", async () => {
    const harness = createSubscriptionHarness();
    const onDrop = vi.fn();
    render(
      <ConversationFileDropOverlay
        isChatActive
        onDrop={onDrop}
        subscribe={harness.subscribe}
      />,
    );
    await waitFor(() => expect(harness.subscribe).toHaveBeenCalledOnce());

    act(() => harness.emit({ phase: "enter", grants }));
    const status = screen.getByRole("status");
    expect(status).toHaveAttribute("aria-live", "polite");
    expect(status).toHaveAttribute("aria-atomic", "true");
    expect(status).toHaveAttribute("data-file-count", "2");
    expect(status).toHaveTextContent("Drop files to attach");
    expect(status).toHaveTextContent("2 files will be added");

    act(() => harness.emit({ phase: "drop", grants }));
    expect(screen.queryByRole("status")).toBeNull();
    expect(onDrop).toHaveBeenCalledWith(grants);
  });

  it("hides on leave without submitting and cleans up the native listener", async () => {
    const harness = createSubscriptionHarness();
    const onDrop = vi.fn();
    const { unmount } = render(
      <ConversationFileDropOverlay
        isChatActive
        onDrop={onDrop}
        subscribe={harness.subscribe}
      />,
    );
    await waitFor(() => expect(harness.subscribe).toHaveBeenCalledOnce());

    act(() => harness.emit({ phase: "enter", grants: [grants[0]!] }));
    expect(screen.getByRole("status")).toHaveTextContent("Drop file to attach");
    act(() => harness.emit({ phase: "leave", grants: [] }));
    expect(screen.queryByRole("status")).toBeNull();
    expect(onDrop).not.toHaveBeenCalled();

    unmount();
    expect(harness.unlisten).toHaveBeenCalledOnce();
  });

  it("does not subscribe or render outside ordinary Chat", () => {
    const harness = createSubscriptionHarness();
    render(
      <ConversationFileDropOverlay
        isChatActive={false}
        onDrop={vi.fn()}
        subscribe={harness.subscribe}
      />,
    );

    expect(harness.subscribe).not.toHaveBeenCalled();
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("reports a native subscription failure without showing a stale target", async () => {
    const unavailable = new Error("listener unavailable");
    const onInvalidEvent = vi.fn();
    const subscribe: ConversationFileDropSubscriber = vi.fn(async () => {
      throw unavailable;
    });
    render(
      <ConversationFileDropOverlay
        isChatActive
        onDrop={vi.fn()}
        onInvalidEvent={onInvalidEvent}
        subscribe={subscribe}
      />,
    );

    await waitFor(() =>
      expect(onInvalidEvent).toHaveBeenCalledWith(unavailable),
    );
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("disposes a subscription that resolves after Chat becomes inactive", async () => {
    let resolveSubscription: ((unlisten: () => void) => void) | undefined;
    const unlisten = vi.fn();
    const subscribe: ConversationFileDropSubscriber = vi.fn(
      () =>
        new Promise<() => void>((resolve) => {
          resolveSubscription = resolve;
        }),
    );
    const { rerender } = render(
      <ConversationFileDropOverlay
        isChatActive
        onDrop={vi.fn()}
        subscribe={subscribe}
      />,
    );
    await waitFor(() => expect(subscribe).toHaveBeenCalledOnce());

    rerender(
      <ConversationFileDropOverlay
        isChatActive={false}
        onDrop={vi.fn()}
        subscribe={subscribe}
      />,
    );
    await act(async () => resolveSubscription?.(unlisten));
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
