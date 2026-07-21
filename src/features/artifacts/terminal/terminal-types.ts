import type { ArtifactShellStatus } from "../types";

export const TERMINAL_PHASES = [
  "queued",
  "approvalWaiting",
  "running",
  "stdinReady",
  "stopping",
  "completed",
  "interrupted",
  "failed",
  "recovery",
] as const;

export type TerminalPhase = (typeof TERMINAL_PHASES)[number];

/** The exact bounded Terminal state projected by the Rust-owned core. */
export interface TerminalArtifactState {
  readonly terminalSessionId: string;
  readonly commandId: string;
  readonly commandSequence: number;
  readonly command: string;
  readonly workingDirectoryDisplay: string;
  readonly shellPath: string;
  readonly environmentId: string;
  readonly environmentGeneration: number;
  readonly processGeneration: number;
  readonly shellProcessId: number | null;
  readonly foregroundProcessGroupId: number | null;
  readonly columns: number;
  readonly rows: number;
  readonly outputBase64: string;
  readonly outputText: string;
  readonly outputSequence: number;
  readonly retainedBytes: number;
  readonly droppedBytes: number;
  readonly phase: TerminalPhase;
  readonly exitCode: number | null;
  readonly statusMessage: string | null;
  readonly stdinReady: boolean;
  readonly stopAvailable: boolean;
  readonly promptReady: boolean;
  readonly shellReplaced: boolean;
}

/** Adds shared Artifact identity/chrome state to one native Terminal snapshot. */
export interface TerminalArtifactModel extends TerminalArtifactState {
  readonly artifactId: string;
  readonly pendingApprovalId: string | null;
  readonly status?: ArtifactShellStatus;
}

export interface TerminalReplySelection {
  readonly selectedText: string;
}

export interface TerminalResizeIntent {
  readonly artifactId: string;
  readonly terminalSessionId: string;
  readonly columns: number;
  readonly rows: number;
}

export interface TerminalStdinIntent {
  readonly artifactId: string;
  readonly commandId: string;
  readonly input: string;
}

export interface TerminalRunIntent {
  readonly artifactId: string;
  readonly terminalSessionId: string;
  readonly command: string;
}

/** Decodes one already-bounded native PTY snapshot without adding authority. */
export function decodeTerminalOutputBytes(outputBase64: string): Uint8Array {
  const binary = window.atob(outputBase64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

/** Keeps renderer selection hints bounded to the same native artifact limit. */
export function boundedTerminalSelection(
  selectedText: string,
  outputText: string,
): TerminalReplySelection | undefined {
  if (
    selectedText.length === 0 ||
    selectedText.length > 1_048_576 ||
    !outputText.includes(selectedText)
  ) {
    return undefined;
  }
  return { selectedText };
}
