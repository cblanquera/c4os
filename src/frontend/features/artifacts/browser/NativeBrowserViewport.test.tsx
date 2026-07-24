import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { NativeBrowserViewport } from "./NativeBrowserViewport";

class MockResizeObserver {
  static readonly instances: MockResizeObserver[] = [];
  readonly disconnect = vi.fn();
  readonly observe = vi.fn();

  constructor(private readonly callback: ResizeObserverCallback) {
    MockResizeObserver.instances.push(this);
  }

  notify() {
    this.callback([], this as unknown as ResizeObserver);
  }
}

class MockMutationObserver {
  static readonly instances: MockMutationObserver[] = [];
  readonly disconnect = vi.fn();
  readonly observe = vi.fn();

  constructor(private readonly callback: MutationCallback) {
    MockMutationObserver.instances.push(this);
  }

  notify() {
    this.callback([], this as unknown as MutationObserver);
  }
}

const lifecycleIdentity = {
  artifactId: "artifact:browser-viewport",
  baseRecordRevision: 9,
  controllerGeneration: 8,
  mountGeneration: 3,
  presentation: "focused" as const,
};

let nextFrame = 1;
let frames = new Map<number, FrameRequestCallback>();
const originalWidth = window.innerWidth;
const originalHeight = window.innerHeight;

function flushFrames() {
  const queued = [...frames.entries()];
  frames = new Map();
  for (const [, callback] of queued) callback(performance.now());
}

beforeEach(() => {
  MockResizeObserver.instances.length = 0;
  MockMutationObserver.instances.length = 0;
  nextFrame = 1;
  frames = new Map();
  vi.stubGlobal("ResizeObserver", MockResizeObserver);
  vi.stubGlobal("MutationObserver", MockMutationObserver);
  vi.stubGlobal(
    "requestAnimationFrame",
    vi.fn((callback: FrameRequestCallback) => {
      const id = nextFrame;
      nextFrame += 1;
      frames.set(id, callback);
      return id;
    }),
  );
  vi.stubGlobal(
    "cancelAnimationFrame",
    vi.fn((id: number) => frames.delete(id)),
  );
  Object.defineProperty(window, "innerWidth", {
    configurable: true,
    value: 800,
  });
  Object.defineProperty(window, "innerHeight", {
    configurable: true,
    value: 600,
  });
});

afterEach(() => {
  Object.defineProperty(window, "innerWidth", {
    configurable: true,
    value: originalWidth,
  });
  Object.defineProperty(window, "innerHeight", {
    configurable: true,
    value: originalHeight,
  });
  vi.unstubAllGlobals();
});

describe("NativeBrowserViewport", () => {
  it("coalesces observed geometry and clamps it to finite visible CSS pixels", () => {
    const onLifecycle = vi.fn();
    render(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        onLifecycle={onLifecycle}
      />,
    );
    const viewport = screen.getByRole("group", {
      name: "Native Browser viewport: Example",
    });
    viewport.getBoundingClientRect = vi.fn(
      () =>
        ({
          bottom: 650,
          height: 670,
          left: -20,
          right: 920,
          top: -20,
          width: 940,
          x: -20,
          y: -20,
          toJSON: () => ({}),
        }) as DOMRect,
    );

    MockResizeObserver.instances[0]?.notify();
    MockResizeObserver.instances[0]?.notify();
    expect(requestAnimationFrame).toHaveBeenCalledTimes(1);
    flushFrames();

    expect(onLifecycle).toHaveBeenCalledWith({
      ...lifecycleIdentity,
      kind: "geometry",
      rect: { x: 0, y: 0, width: 800, height: 600 },
      visible: true,
    });
    expect(MockResizeObserver.instances[0]?.observe).toHaveBeenCalledWith(
      viewport,
    );
  });

  it("emits hidden and detach lifecycle events on unmount", () => {
    const onLifecycle = vi.fn();
    const rendered = render(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        onLifecycle={onLifecycle}
      />,
    );
    const viewport = screen.getByRole("group");
    viewport.getBoundingClientRect = vi.fn(
      () =>
        ({
          bottom: 320,
          height: 300,
          left: 10,
          right: 410,
          top: 20,
          width: 400,
          x: 10,
          y: 20,
          toJSON: () => ({}),
        }) as DOMRect,
    );
    flushFrames();
    rendered.unmount();

    expect(onLifecycle).toHaveBeenNthCalledWith(2, {
      ...lifecycleIdentity,
      kind: "hidden",
      rect: { x: 10, y: 20, width: 400, height: 300 },
      visible: false,
    });
    expect(onLifecycle).toHaveBeenNthCalledWith(3, {
      ...lifecycleIdentity,
      kind: "detach",
    });
    expect(MockResizeObserver.instances[0]?.disconnect).toHaveBeenCalled();
  });

  it("keeps one native surface across durable record revision updates", () => {
    const onFocusIntent = vi.fn();
    const onLifecycle = vi.fn();
    const rendered = render(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        onFocusIntent={onFocusIntent}
        onLifecycle={onLifecycle}
      />,
    );
    const viewport = screen.getByRole("group");
    viewport.getBoundingClientRect = vi.fn(
      () =>
        ({
          bottom: 320,
          height: 300,
          left: 10,
          right: 410,
          top: 20,
          width: 400,
          x: 10,
          y: 20,
          toJSON: () => ({}),
        }) as DOMRect,
    );
    flushFrames();

    rendered.rerender(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        baseRecordRevision={10}
        onFocusIntent={onFocusIntent}
        onLifecycle={onLifecycle}
      />,
    );
    expect(onLifecycle).toHaveBeenCalledTimes(1);
    fireEvent.pointerDown(viewport, { button: 0 });
    expect(onFocusIntent).toHaveBeenCalledWith({
      ...lifecycleIdentity,
      baseRecordRevision: 10,
      input: "pointer",
    });

    rendered.unmount();
    expect(onLifecycle).toHaveBeenNthCalledWith(2, {
      ...lifecycleIdentity,
      baseRecordRevision: 10,
      kind: "hidden",
      rect: { x: 10, y: 20, width: 400, height: 300 },
      visible: false,
    });
    expect(onLifecycle).toHaveBeenNthCalledWith(3, {
      ...lifecycleIdentity,
      baseRecordRevision: 10,
      kind: "detach",
    });
  });

  it("remeasures a same-size viewport moved by approval DOM changes", () => {
    const onLifecycle = vi.fn();
    render(
      <div data-artifact-scroll-region="body">
        <NativeBrowserViewport
          {...lifecycleIdentity}
          accessibleTitle="Example"
          onLifecycle={onLifecycle}
        />
      </div>,
    );
    const viewport = screen.getByRole("group");
    let top = 236;
    viewport.getBoundingClientRect = vi.fn(
      () =>
        ({
          bottom: top + 300,
          height: 300,
          left: 0,
          right: 500,
          top,
          width: 500,
          x: 0,
          y: top,
          toJSON: () => ({}),
        }) as DOMRect,
    );
    flushFrames();

    top = 168;
    MockMutationObserver.instances[0]?.notify();
    flushFrames();

    expect(onLifecycle).toHaveBeenNthCalledWith(2, {
      ...lifecycleIdentity,
      kind: "geometry",
      rect: { x: 0, y: 168, width: 500, height: 300 },
      visible: true,
    });
    expect(MockMutationObserver.instances[0]?.observe).toHaveBeenCalledWith(
      expect.any(HTMLElement),
      expect.objectContaining({ childList: true, subtree: true }),
    );
  });

  it("exposes pointer and keyboard focus intent without native service access", () => {
    const onFocusIntent = vi.fn();
    render(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        onFocusIntent={onFocusIntent}
      />,
    );
    const viewport = screen.getByRole("group");

    fireEvent.pointerDown(viewport, { button: 0 });
    fireEvent.keyDown(viewport, { key: "Enter" });
    fireEvent.keyDown(viewport, { key: " " });
    fireEvent.keyDown(viewport, { key: "Escape" });

    expect(onFocusIntent.mock.calls).toEqual([
      [{ ...lifecycleIdentity, input: "pointer" }],
      [{ ...lifecycleIdentity, input: "keyboard" }],
      [{ ...lifecycleIdentity, input: "keyboard" }],
    ]);
  });

  it("reports a zero-area host as hidden", () => {
    const onLifecycle = vi.fn();
    render(
      <NativeBrowserViewport
        {...lifecycleIdentity}
        accessibleTitle="Example"
        onLifecycle={onLifecycle}
      />,
    );
    flushFrames();

    expect(onLifecycle).toHaveBeenCalledWith({
      ...lifecycleIdentity,
      kind: "hidden",
      rect: { x: 0, y: 0, width: 0, height: 0 },
      visible: false,
    });
  });
});
