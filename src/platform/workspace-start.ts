import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type ProtocolError,
  type RequestId,
  type SnapshotRequest,
  type StateGeneration,
  type WorkspaceId,
  type WorkspaceRecentSnapshot,
  type WorkspaceStartSnapshot,
  type WorkspaceStartSnapshotEnvelope,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const WORKSPACE_START_SNAPSHOT_COMMAND =
  "workspace_start_snapshot" as const;
const MAX_RECENTS = 3;
const MAX_DISPLAY_NAME_BYTES = 512;

export interface WorkspaceStartTransport {
  invoke(
    command: typeof WORKSPACE_START_SNAPSHOT_COMMAND,
    args: { readonly request: SnapshotRequest },
  ): Promise<unknown>;
}

export interface WorkspaceStartAdapter {
  readonly currentGeneration: StateGeneration;
  readSnapshot(): Promise<WorkspaceStartSnapshot>;
}

const nativeAdapter = createWorkspaceStartAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

/** Reads the bounded, Rust-owned Workspace Start projection. */
export async function readWorkspaceStartSnapshot(): Promise<WorkspaceStartSnapshot> {
  return nativeAdapter.readSnapshot();
}

export function createWorkspaceStartAdapter(
  transport: WorkspaceStartTransport,
  options: {
    readonly requestIdFactory?: () => RequestId;
    readonly correlationIdFactory?: () => CorrelationId;
    readonly initialGeneration?: StateGeneration;
  } = {},
): WorkspaceStartAdapter {
  let currentGeneration = options.initialGeneration ?? (0 as StateGeneration);
  const requestIdFactory =
    options.requestIdFactory ?? (() => secureUuid("request ID") as RequestId);
  const correlationIdFactory =
    options.correlationIdFactory ??
    (() => secureUuid("correlation ID") as CorrelationId);

  return {
    get currentGeneration() {
      return currentGeneration;
    },
    async readSnapshot() {
      const requestId = requestIdFactory();
      const correlationId = correlationIdFactory();
      const request: SnapshotRequest = {
        protocolVersion: PROTOCOL_VERSION,
        requestId,
        correlationId,
        expectedGeneration: currentGeneration,
      };

      let raw: unknown;
      try {
        raw = await transport.invoke(WORKSPACE_START_SNAPSHOT_COMMAND, {
          request,
        });
      } catch (error) {
        throw normalizeCoreFailure(error);
      }

      const envelope = parseEnvelope(raw);
      if (
        envelope.requestId !== requestId ||
        envelope.correlationId !== correlationId
      ) {
        throw new ProtocolBoundaryError(
          "correlationMismatch",
          "The Workspace Start response identity did not match its request.",
        );
      }
      if (envelope.generation < currentGeneration) {
        throw new ProtocolBoundaryError(
          "staleGeneration",
          "The Workspace Start response was stale.",
        );
      }
      if (envelope.payload.generation !== envelope.generation) {
        throw new ProtocolBoundaryError(
          "staleGeneration",
          "The Workspace Start payload generation did not match its envelope.",
        );
      }

      currentGeneration = envelope.generation;
      return envelope.payload;
    },
  };
}

function parseEnvelope(raw: unknown): WorkspaceStartSnapshotEnvelope {
  const envelope = requireRecord(raw, "Workspace Start envelope");
  requireProtocolVersion(envelope.protocolVersion);
  const generation = asGeneration(envelope.generation);
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: asIdentifier(envelope.requestId, "request ID") as RequestId,
    correlationId: asIdentifier(
      envelope.correlationId,
      "correlation ID",
    ) as CorrelationId,
    generation,
    payload: parseSnapshot(envelope.payload),
  };
}

function parseSnapshot(raw: unknown): WorkspaceStartSnapshot {
  const snapshot = requireRecord(raw, "Workspace Start snapshot");
  requireProtocolVersion(snapshot.protocolVersion);
  if (snapshot.authority !== "rust-core") {
    throw invalidPayload("The Workspace Start authority is invalid.");
  }
  if (
    !Array.isArray(snapshot.recents) ||
    snapshot.recents.length > MAX_RECENTS
  ) {
    throw invalidPayload("The Workspace Start recent list is invalid.");
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    generation: asGeneration(snapshot.generation),
    authority: "rust-core",
    recents: snapshot.recents.map(parseRecent),
  };
}

function parseRecent(raw: unknown): WorkspaceRecentSnapshot {
  const recent = requireRecord(raw, "recent Workspace");
  const displayName = requireString(
    recent.displayName,
    "Workspace display name",
  );
  if (
    displayName.trim().length === 0 ||
    new TextEncoder().encode(displayName).length > MAX_DISPLAY_NAME_BYTES ||
    Array.from(displayName).some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return codePoint < 32 || codePoint === 127;
    })
  ) {
    throw invalidPayload("A recent Workspace display name is invalid.");
  }
  if (!Number.isSafeInteger(recent.lastOpenedAt)) {
    throw invalidPayload("A recent Workspace timestamp is invalid.");
  }
  if (typeof recent.isMissing !== "boolean") {
    throw invalidPayload("A recent Workspace availability state is invalid.");
  }
  return {
    workspaceId: asIdentifier(
      recent.workspaceId,
      "Workspace ID",
    ) as WorkspaceId,
    displayName,
    lastOpenedAt: recent.lastOpenedAt as number,
    isMissing: recent.isMissing,
  };
}

function normalizeCoreFailure(error: unknown): Error {
  if (isProtocolError(error)) {
    return new ProtocolBoundaryError(
      error.code,
      error.message,
      error.retryable,
      error.details,
    );
  }
  return new ProtocolBoundaryError(
    "unavailable",
    "Workspace Start is unavailable.",
    true,
  );
}

function isProtocolError(value: unknown): value is ProtocolError {
  if (!isRecord(value)) return false;
  return (
    typeof value.code === "string" &&
    typeof value.message === "string" &&
    typeof value.retryable === "boolean" &&
    isRecord(value.details)
  );
}

function secureUuid(label: string): string {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw new ProtocolBoundaryError(
      "unavailable",
      `Secure ${label} generation is unavailable.`,
    );
  }
  return globalThis.crypto.randomUUID();
}

function requireProtocolVersion(value: unknown): void {
  if (value !== PROTOCOL_VERSION) {
    throw new ProtocolBoundaryError(
      "unknownProtocolVersion",
      "The Workspace Start protocol version is unsupported.",
    );
  }
}

function asGeneration(value: unknown): StateGeneration {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw invalidPayload("A Workspace Start generation is invalid.");
  }
  return value as StateGeneration;
}

function asIdentifier(value: unknown, label: string): string {
  const identifier = requireString(value, label);
  if (
    identifier.length === 0 ||
    identifier.length > MAX_IDENTIFIER_BYTES ||
    !/^[A-Za-z0-9_.:@-]+$/u.test(identifier)
  ) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return identifier;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string") {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return value;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value)) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function invalidPayload(message: string): ProtocolBoundaryError {
  return new ProtocolBoundaryError("invalidPayload", message);
}
