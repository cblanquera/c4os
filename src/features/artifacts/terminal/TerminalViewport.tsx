import { FitAddon } from "@xterm/addon-fit";
import { Terminal as XtermTerminal } from "@xterm/xterm";
import { useEffect, useRef } from "react";

import { decodeTerminalOutputBytes } from "./terminal-types";
import "@xterm/xterm/css/xterm.css";

export interface TerminalViewportProps {
  readonly accessibleLabel: string;
  readonly columns: number;
  readonly droppedBytes: number;
  readonly onOutputWritten?: ((outputSequence: number) => void) | undefined;
  readonly onResize?: ((columns: number, rows: number) => void) | undefined;
  readonly onSelectionChange?: ((selectedText: string) => void) | undefined;
  readonly outputBase64: string;
  readonly outputSequence: number;
  readonly reducedMotion: boolean;
  readonly retainedBytes: number;
  readonly rows: number;
  readonly snapshotKey: string;
}

interface WrittenOutput {
  readonly bytes: Uint8Array;
  readonly droppedBytes: number;
  readonly outputSequence: number;
  readonly retainedBytes: number;
  readonly snapshotKey: string;
}

/** Owns the focused-only xterm rendering lifecycle for a native PTY snapshot. */
export function TerminalViewport({
  accessibleLabel,
  columns,
  droppedBytes,
  onOutputWritten,
  onResize,
  onSelectionChange,
  outputBase64,
  outputSequence,
  reducedMotion,
  retainedBytes,
  rows,
  snapshotKey,
}: TerminalViewportProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<XtermTerminal | null>(null);
  const writtenOutputRef = useRef<WrittenOutput | null>(null);
  const acknowledgedOutputRef = useRef<{
    readonly outputSequence: number;
    readonly snapshotKey: string;
  } | null>(null);
  const outputWrittenCallbackRef = useRef(onOutputWritten);
  const resizeCallbackRef = useRef(onResize);
  const selectionCallbackRef = useRef(onSelectionChange);
  const initialConfigurationRef = useRef({
    columns: positiveDimension(columns),
    reducedMotion,
    rows: positiveDimension(rows),
  });

  useEffect(() => {
    outputWrittenCallbackRef.current = onOutputWritten;
  }, [onOutputWritten]);

  useEffect(() => {
    resizeCallbackRef.current = onResize;
  }, [onResize]);

  useEffect(() => {
    selectionCallbackRef.current = onSelectionChange;
  }, [onSelectionChange]);

  useEffect(() => {
    const host = hostRef.current;
    if (host === null) return;
    const initialConfiguration = initialConfigurationRef.current;

    const terminal = new XtermTerminal({
      allowTransparency: false,
      cols: initialConfiguration.columns,
      convertEol: false,
      cursorBlink: !initialConfiguration.reducedMotion,
      disableStdin: true,
      drawBoldTextInBrightColors: false,
      rows: initialConfiguration.rows,
      screenReaderMode: true,
      scrollback: 5_000,
      theme: terminalTheme(host),
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(host);
    terminalRef.current = terminal;

    const selectionDisposable = terminal.onSelectionChange(() => {
      selectionCallbackRef.current?.(terminal.getSelection());
    });
    let animationFrame: number | null = null;
    let lastReportedDimensions = {
      columns: initialConfiguration.columns,
      rows: initialConfiguration.rows,
    };
    const fitAndReport = () => {
      animationFrame = null;
      if (host.clientWidth <= 0 || host.clientHeight <= 0) return;
      try {
        fitAddon.fit();
      } catch {
        return;
      }
      if (
        terminal.cols > 0 &&
        terminal.rows > 0 &&
        (terminal.cols !== lastReportedDimensions.columns ||
          terminal.rows !== lastReportedDimensions.rows)
      ) {
        lastReportedDimensions = {
          columns: terminal.cols,
          rows: terminal.rows,
        };
        resizeCallbackRef.current?.(terminal.cols, terminal.rows);
      }
    };
    const scheduleFit = () => {
      if (animationFrame !== null) return;
      animationFrame = window.requestAnimationFrame(fitAndReport);
    };
    const resizeObserver = new ResizeObserver(scheduleFit);
    resizeObserver.observe(host);
    const themeObserver = new MutationObserver(() => {
      terminal.options.theme = terminalTheme(host);
    });
    themeObserver.observe(document.documentElement, {
      attributeFilter: ["class", "data-color-scheme", "style"],
      attributes: true,
    });
    scheduleFit();

    return () => {
      resizeObserver.disconnect();
      themeObserver.disconnect();
      if (animationFrame !== null) {
        window.cancelAnimationFrame(animationFrame);
      }
      selectionDisposable.dispose();
      fitAddon.dispose();
      terminal.dispose();
      terminalRef.current = null;
      writtenOutputRef.current = null;
    };
  }, []);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (terminal === null) return;
    const nextBytes = decodeTerminalOutputBytes(outputBase64);
    const prior = writtenOutputRef.current;
    const next: WrittenOutput = {
      bytes: nextBytes,
      droppedBytes,
      outputSequence,
      retainedBytes,
      snapshotKey,
    };
    if (sameOutput(prior, next)) return;

    const wasAtBottom =
      terminal.buffer.active.viewportY >= terminal.buffer.active.baseY;
    const acknowledgeWrittenOutput = () => {
      // Ignore a callback from a Strict Mode-disposed xterm instance. The new
      // instance will replay and acknowledge the same bounded snapshot.
      if (terminalRef.current !== terminal) return;
      const acknowledged = acknowledgedOutputRef.current;
      if (
        acknowledged?.snapshotKey === snapshotKey &&
        acknowledged.outputSequence === outputSequence
      ) {
        return;
      }
      acknowledgedOutputRef.current = { outputSequence, snapshotKey };
      outputWrittenCallbackRef.current?.(outputSequence);
    };
    if (isAppendOnly(prior, next)) {
      terminal.write(nextBytes.slice(prior.bytes.length), () => {
        if (wasAtBottom) terminal.scrollToBottom();
        acknowledgeWrittenOutput();
      });
    } else {
      terminal.reset();
      terminal.write(nextBytes, () => {
        if (wasAtBottom) terminal.scrollToBottom();
        acknowledgeWrittenOutput();
      });
    }
    writtenOutputRef.current = next;
  }, [droppedBytes, outputBase64, outputSequence, retainedBytes, snapshotKey]);

  useEffect(() => {
    const terminal = terminalRef.current;
    const host = hostRef.current;
    if (terminal === null || host === null) return;
    terminal.options.cursorBlink = !reducedMotion;
    terminal.options.theme = terminalTheme(host);
  }, [reducedMotion]);

  return (
    <div
      aria-label={accessibleLabel}
      className="artifact-terminal__viewport"
      data-terminal-output-sequence={outputSequence}
      ref={hostRef}
      role="region"
    />
  );
}

function positiveDimension(value: number): number {
  return Math.max(1, Math.floor(value));
}

function sameOutput(prior: WrittenOutput | null, next: WrittenOutput): boolean {
  return (
    prior !== null &&
    prior.outputSequence === next.outputSequence &&
    prior.droppedBytes === next.droppedBytes &&
    prior.retainedBytes === next.retainedBytes &&
    prior.snapshotKey === next.snapshotKey &&
    bytesEqual(prior.bytes, next.bytes)
  );
}

function isAppendOnly(
  prior: WrittenOutput | null,
  next: WrittenOutput,
): prior is WrittenOutput {
  if (
    prior === null ||
    next.outputSequence <= prior.outputSequence ||
    next.droppedBytes !== prior.droppedBytes ||
    next.retainedBytes < prior.retainedBytes ||
    next.bytes.length < prior.bytes.length ||
    next.snapshotKey !== prior.snapshotKey
  ) {
    return false;
  }
  for (let index = 0; index < prior.bytes.length; index += 1) {
    if (prior.bytes[index] !== next.bytes[index]) return false;
  }
  return true;
}

function bytesEqual(left: Uint8Array, right: Uint8Array): boolean {
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) return false;
  }
  return true;
}

function terminalTheme(host: HTMLElement) {
  const style = window.getComputedStyle(host);
  return {
    background: style.getPropertyValue("--surface-sunken").trim() || "#111111",
    cursor: style.getPropertyValue("--focus-ring").trim() || "#7aa2ff",
    foreground: style.getPropertyValue("--text-primary").trim() || "#f2f2f2",
    selectionBackground:
      style.getPropertyValue("--selection-background").trim() || "#34558a",
  };
}
