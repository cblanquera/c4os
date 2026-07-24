import { StrictMode } from "react";
import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { TerminalViewport } from "./TerminalViewport";

interface MockTerminalInstance {
  readonly pendingWriteCallbacks: Array<() => void>;
  readonly writes: Uint8Array[];
  readonly resets: { count: number };
  readonly disposed: { value: boolean };
  readonly selectionListeners: Set<() => void>;
  selection: string;
  cols: number;
  rows: number;
  options: Record<string, unknown>;
  scrollToBottomCount: number;
}

interface MockFitInstance {
  readonly disposed: { value: boolean };
  readonly fits: { count: number };
}

const harness = vi.hoisted(() => ({
  autoCompleteWrites: true,
  terminals: [] as MockTerminalInstance[],
  fits: [] as MockFitInstance[],
  fitColumns: 100,
  fitRows: 36,
}));

vi.mock("@xterm/xterm", () => ({
  Terminal: class MockTerminal {
    readonly writes: Uint8Array[] = [];
    readonly resets = { count: 0 };
    readonly disposed = { value: false };
    readonly selectionListeners = new Set<() => void>();
    readonly pendingWriteCallbacks: Array<() => void> = [];
    selection = "";
    cols: number;
    rows: number;
    options: Record<string, unknown>;
    scrollToBottomCount = 0;
    readonly buffer = { active: { viewportY: 0, baseY: 0 } };

    constructor(options: Record<string, unknown>) {
      this.options = { ...options };
      this.cols = Number(options.cols);
      this.rows = Number(options.rows);
      harness.terminals.push(this);
    }

    loadAddon(addon: { activate?: (terminal: MockTerminal) => void }) {
      addon.activate?.(this);
    }

    open() {}

    onSelectionChange(listener: () => void) {
      this.selectionListeners.add(listener);
      return {
        dispose: () => this.selectionListeners.delete(listener),
      };
    }

    getSelection() {
      return this.selection;
    }

    write(bytes: Uint8Array, callback?: () => void) {
      this.writes.push(new Uint8Array(bytes));
      if (callback) {
        if (harness.autoCompleteWrites) callback();
        else this.pendingWriteCallbacks.push(callback);
      }
    }

    reset() {
      this.resets.count += 1;
    }

    scrollToBottom() {
      this.scrollToBottomCount += 1;
    }

    dispose() {
      this.disposed.value = true;
    }
  },
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class MockFitAddon {
    readonly disposed = { value: false };
    readonly fits = { count: 0 };
    terminal: MockTerminalInstance | null = null;

    constructor() {
      harness.fits.push(this);
    }

    activate(terminal: MockTerminalInstance) {
      this.terminal = terminal;
    }

    fit() {
      this.fits.count += 1;
      if (this.terminal) {
        this.terminal.cols = harness.fitColumns;
        this.terminal.rows = harness.fitRows;
      }
    }

    dispose() {
      this.disposed.value = true;
    }
  },
}));

class MockResizeObserver {
  static readonly instances: MockResizeObserver[] = [];
  readonly disconnect = vi.fn();
  readonly observe = vi.fn();

  constructor(readonly callback: ResizeObserverCallback) {
    MockResizeObserver.instances.push(this);
  }

  trigger() {
    this.callback([], this as unknown as ResizeObserver);
  }
}

let animationFrames = new Map<number, FrameRequestCallback>();
let nextAnimationFrame = 1;

beforeEach(() => {
  harness.terminals.length = 0;
  harness.fits.length = 0;
  harness.autoCompleteWrites = true;
  harness.fitColumns = 100;
  harness.fitRows = 36;
  MockResizeObserver.instances.length = 0;
  animationFrames = new Map();
  nextAnimationFrame = 1;
  vi.stubGlobal("ResizeObserver", MockResizeObserver);
  vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
    const id = nextAnimationFrame++;
    animationFrames.set(id, callback);
    return id;
  });
  vi.spyOn(window, "cancelAnimationFrame").mockImplementation((id) => {
    animationFrames.delete(id);
  });
  vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockReturnValue(640);
  vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(320);
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("TerminalViewport", () => {
  it("replays once, appends proven deltas, and resets after truncation or identity change", () => {
    const { rerender } = render(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        outputBase64={encode("alpha")}
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={5}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    const terminal = harness.terminals[0];
    expect(terminal).toBeDefined();
    expect(terminal?.resets.count).toBe(1);
    expect(decodedWrites(terminal)).toEqual(["alpha"]);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        outputBase64={encode("alpha")}
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={5}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    expect(decodedWrites(terminal)).toEqual(["alpha"]);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        outputBase64={encode("alpha beta")}
        outputSequence={2}
        reducedMotion={false}
        retainedBytes={10}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    expect(decodedWrites(terminal)).toEqual(["alpha", " beta"]);
    expect(terminal?.resets.count).toBe(1);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={6}
        outputBase64={encode("beta")}
        outputSequence={3}
        reducedMotion={false}
        retainedBytes={4}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    expect(decodedWrites(terminal)).toEqual(["alpha", " beta", "beta"]);
    expect(terminal?.resets.count).toBe(2);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        outputBase64={encode("new")}
        outputSequence={4}
        reducedMotion={false}
        retainedBytes={3}
        rows={24}
        snapshotKey="command:2"
      />,
    );
    expect(decodedWrites(terminal)).toEqual(["alpha", " beta", "beta", "new"]);
    expect(terminal?.resets.count).toBe(3);
  });

  it("coalesces resize, reports selection, and updates reduced motion in place", () => {
    const onResize = vi.fn();
    const onSelectionChange = vi.fn();
    const { rerender } = render(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        onResize={onResize}
        onSelectionChange={onSelectionChange}
        outputBase64={encode("ready")}
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={5}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    const terminal = harness.terminals[0];
    const observer = MockResizeObserver.instances[0];
    expect(terminal?.options.disableStdin).toBe(true);
    expect(terminal?.options.screenReaderMode).toBe(true);
    expect(terminal?.options.cursorBlink).toBe(true);

    observer?.trigger();
    expect(animationFrames).toHaveLength(1);
    flushAnimationFrames();
    expect(onResize).toHaveBeenCalledWith(100, 36);

    observer?.trigger();
    observer?.trigger();
    expect(animationFrames).toHaveLength(1);
    flushAnimationFrames();
    expect(onResize).toHaveBeenCalledTimes(1);

    harness.fitColumns = 112;
    harness.fitRows = 40;
    observer?.trigger();
    flushAnimationFrames();
    expect(onResize).toHaveBeenLastCalledWith(112, 40);

    if (terminal) {
      terminal.selection = "chosen output";
      for (const listener of terminal.selectionListeners) listener();
    }
    expect(onSelectionChange).toHaveBeenCalledWith("chosen output");

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={112}
        droppedBytes={0}
        onResize={onResize}
        onSelectionChange={onSelectionChange}
        outputBase64={encode("ready")}
        outputSequence={1}
        reducedMotion
        retainedBytes={5}
        rows={40}
        snapshotKey="command:1"
      />,
    );
    expect(harness.terminals).toHaveLength(1);
    expect(terminal?.options.cursorBlink).toBe(false);
  });

  it("acknowledges each changed sequence only after xterm commits its write", () => {
    harness.autoCompleteWrites = false;
    const onOutputWritten = vi.fn();
    const { rerender } = render(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        onOutputWritten={onOutputWritten}
        outputBase64={encode("alpha")}
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={5}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    const terminal = harness.terminals[0];
    expect(onOutputWritten).not.toHaveBeenCalled();
    expect(terminal?.pendingWriteCallbacks).toHaveLength(1);

    act(() => terminal?.pendingWriteCallbacks.shift()?.());
    expect(onOutputWritten).toHaveBeenCalledWith(1);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        onOutputWritten={onOutputWritten}
        outputBase64={encode("alpha")}
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={5}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    expect(terminal?.pendingWriteCallbacks).toHaveLength(0);
    expect(onOutputWritten).toHaveBeenCalledTimes(1);

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        onOutputWritten={onOutputWritten}
        outputBase64={encode("alpha beta")}
        outputSequence={2}
        reducedMotion={false}
        retainedBytes={10}
        rows={24}
        snapshotKey="command:1"
      />,
    );
    expect(onOutputWritten).toHaveBeenCalledTimes(1);
    expect(terminal?.pendingWriteCallbacks).toHaveLength(1);
    act(() => terminal?.pendingWriteCallbacks.shift()?.());
    expect(onOutputWritten).toHaveBeenNthCalledWith(2, 2);
  });

  it("does not acknowledge pristine empty output before a native cursor exists", () => {
    harness.autoCompleteWrites = false;
    const onOutputWritten = vi.fn();
    const { rerender } = render(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={0}
        onOutputWritten={onOutputWritten}
        outputBase64=""
        outputSequence={1}
        reducedMotion={false}
        retainedBytes={0}
        rows={24}
        snapshotKey="command:queued"
      />,
    );
    const terminal = harness.terminals[0];
    expect(terminal?.pendingWriteCallbacks).toHaveLength(0);
    expect(onOutputWritten).not.toHaveBeenCalled();

    rerender(
      <TerminalViewport
        accessibleLabel="Terminal output"
        columns={80}
        droppedBytes={7}
        onOutputWritten={onOutputWritten}
        outputBase64=""
        outputSequence={2}
        reducedMotion={false}
        retainedBytes={0}
        rows={24}
        snapshotKey="command:queued"
      />,
    );
    expect(terminal?.pendingWriteCallbacks).toHaveLength(1);
    act(() => terminal?.pendingWriteCallbacks.shift()?.());
    expect(onOutputWritten).toHaveBeenCalledWith(2);
  });

  it("cleans every xterm resource through React Strict Mode remounts", () => {
    const { unmount } = render(
      <StrictMode>
        <TerminalViewport
          accessibleLabel="Terminal output"
          columns={80}
          droppedBytes={0}
          outputBase64={encode("ready")}
          outputSequence={1}
          reducedMotion={false}
          retainedBytes={5}
          rows={24}
          snapshotKey="command:1"
        />
      </StrictMode>,
    );
    expect(harness.terminals.length).toBeGreaterThanOrEqual(2);
    expect(harness.terminals[0]?.disposed.value).toBe(true);
    expect(harness.fits[0]?.disposed.value).toBe(true);
    expect(MockResizeObserver.instances[0]?.disconnect).toHaveBeenCalled();

    unmount();
    expect(harness.terminals.at(-1)?.disposed.value).toBe(true);
    expect(harness.fits.at(-1)?.disposed.value).toBe(true);
    expect(MockResizeObserver.instances.at(-1)?.disconnect).toHaveBeenCalled();
    expect(animationFrames).toHaveLength(0);
  });
});

function flushAnimationFrames() {
  const callbacks = [...animationFrames.values()];
  animationFrames.clear();
  act(() => {
    for (const callback of callbacks) callback(performance.now());
  });
}

function decodedWrites(terminal: MockTerminalInstance | undefined): string[] {
  return (terminal?.writes ?? []).map((bytes) =>
    new TextDecoder().decode(bytes),
  );
}

function encode(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return window.btoa(binary);
}
