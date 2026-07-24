import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type CorrelationId,
  type PickerGrantId,
  type ProtocolError,
  type RequestId,
  type SnapshotRequest,
  type StateGeneration,
  type WorkspaceId,
  type WorkspaceRecentSnapshot,
  type WorkspaceRecoveryNoticeSnapshot as ProtocolWorkspaceRecoveryNoticeSnapshot,
  type WorkspaceStartSnapshot as ProtocolWorkspaceStartSnapshot,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";

export const WORKSPACE_START_SNAPSHOT_COMMAND =
  "workspace_start_snapshot" as const;
export const WORKSPACE_START_OPEN_FOLDER_COMMAND =
  "workspace_start_open_folder" as const;
export const WORKSPACE_START_OPEN_ARCHIVE_COMMAND =
  "workspace_start_open_archive" as const;
export const WORKSPACE_START_OPEN_RECENT_COMMAND =
  "workspace_start_open_recent" as const;
export const WORKSPACE_START_CLONE_REPOSITORY_COMMAND =
  "workspace_start_clone_repository" as const;
export const WORKSPACE_START_ANSWER_CLONE_APPROVAL_COMMAND =
  "workspace_start_answer_clone_approval" as const;
export const WORKSPACE_RECOVERY_ACKNOWLEDGE_COMMAND =
  "workspace_recovery_acknowledge" as const;
type WorkspaceStartCommand =
  | typeof WORKSPACE_START_SNAPSHOT_COMMAND
  | typeof WORKSPACE_START_OPEN_FOLDER_COMMAND
  | typeof WORKSPACE_START_OPEN_ARCHIVE_COMMAND
  | typeof WORKSPACE_START_OPEN_RECENT_COMMAND
  | typeof WORKSPACE_START_CLONE_REPOSITORY_COMMAND
  | typeof WORKSPACE_START_ANSWER_CLONE_APPROVAL_COMMAND
  | typeof WORKSPACE_RECOVERY_ACKNOWLEDGE_COMMAND;
const MAX_RECENTS = 3;
const MAX_DISPLAY_NAME_BYTES = 512;

export interface WorkspaceStartTransport {
  invoke(
    command: WorkspaceStartCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface WorkspaceStartOpenResult {
  readonly authority: "rust-workspace-service";
  readonly workspaceId: WorkspaceId;
  readonly workspaceName: string;
  readonly recovered: boolean;
  readonly recoveryNotice: WorkspaceRecoveryNotice | null;
}

export type WorkspaceRecoveryNotice = ProtocolWorkspaceRecoveryNoticeSnapshot;

export type WorkspaceStartSnapshot = ProtocolWorkspaceStartSnapshot;

export type WorkspaceStartCloneResult =
  | ({ readonly state: "opened" } & WorkspaceStartOpenResult)
  | {
      readonly state: "pendingApproval";
      readonly promptId: string;
      readonly summary: string;
    }
  | { readonly state: "denied" };

export interface WorkspaceStartAdapter {
  readonly currentGeneration: StateGeneration;
  readSnapshot(): Promise<WorkspaceStartSnapshot>;
  openFolder(pickerGrantId: PickerGrantId): Promise<WorkspaceStartOpenResult>;
  openArchive(pickerGrantId: PickerGrantId): Promise<WorkspaceStartOpenResult>;
  openRecent(workspaceId: WorkspaceId): Promise<WorkspaceStartOpenResult>;
  cloneRepository(
    pickerGrantId: PickerGrantId,
    repositoryUrl: string,
  ): Promise<WorkspaceStartCloneResult>;
  answerCloneApproval(
    promptId: string,
    answer: "allow" | "deny",
  ): Promise<WorkspaceStartCloneResult>;
  acknowledgeRecovery(
    notice: WorkspaceRecoveryNotice,
  ): Promise<WorkspaceStartSnapshot>;
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

export const openWorkspaceFolder = (pickerGrantId: PickerGrantId) =>
  nativeAdapter.openFolder(pickerGrantId);
export const openWorkspaceArchive = (pickerGrantId: PickerGrantId) =>
  nativeAdapter.openArchive(pickerGrantId);
export const openRecentWorkspace = (workspaceId: WorkspaceId) =>
  nativeAdapter.openRecent(workspaceId);
export const cloneWorkspaceRepository = (
  pickerGrantId: PickerGrantId,
  repositoryUrl: string,
) => nativeAdapter.cloneRepository(pickerGrantId, repositoryUrl);
export const answerWorkspaceCloneApproval = (
  promptId: string,
  answer: "allow" | "deny",
) => nativeAdapter.answerCloneApproval(promptId, answer);
export const acknowledgeWorkspaceRecovery = (notice: WorkspaceRecoveryNotice) =>
  nativeAdapter.acknowledgeRecovery(notice);

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
    openFolder(pickerGrantId) {
      return invokeOpen(WORKSPACE_START_OPEN_FOLDER_COMMAND, {
        pickerGrantId: asIdentifier(pickerGrantId, "picker grant ID"),
      });
    },
    openArchive(pickerGrantId) {
      return invokeOpen(WORKSPACE_START_OPEN_ARCHIVE_COMMAND, {
        pickerGrantId: asIdentifier(pickerGrantId, "picker grant ID"),
      });
    },
    openRecent(workspaceId) {
      return invokeOpen(WORKSPACE_START_OPEN_RECENT_COMMAND, {
        workspaceId: asIdentifier(workspaceId, "Workspace ID"),
      });
    },
    cloneRepository(pickerGrantId, repositoryUrl) {
      if (
        repositoryUrl.length === 0 ||
        repositoryUrl.length > 2_048 ||
        hasControlCharacter(repositoryUrl)
      ) {
        throw invalidPayload("The repository URL is invalid.");
      }
      return invokeClone(WORKSPACE_START_CLONE_REPOSITORY_COMMAND, {
        pickerGrantId: asIdentifier(pickerGrantId, "picker grant ID"),
        repositoryUrl,
      });
    },
    answerCloneApproval(promptId, answer) {
      return invokeClone(WORKSPACE_START_ANSWER_CLONE_APPROVAL_COMMAND, {
        promptId: asIdentifier(promptId, "clone approval prompt ID"),
        answer,
      });
    },
    acknowledgeRecovery(notice) {
      return invokeRecoveryAcknowledge(notice);
    },
  };

  async function invokeOpen(
    command: Exclude<
      WorkspaceStartCommand,
      typeof WORKSPACE_START_SNAPSHOT_COMMAND
    >,
    input: Readonly<Record<string, unknown>>,
  ): Promise<WorkspaceStartOpenResult> {
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
      raw = await transport.invoke(command, { request, input });
    } catch (error) {
      throw normalizeCoreFailure(error);
    }
    const envelope = parseRawEnvelope(raw);
    if (
      envelope.requestId !== requestId ||
      envelope.correlationId !== correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The Workspace open response identity did not match its request.",
      );
    }
    if (envelope.generation < currentGeneration) {
      throw new ProtocolBoundaryError(
        "staleGeneration",
        "The Workspace open response was stale.",
      );
    }
    const payload = parseOpenResult(envelope.payload);
    currentGeneration = envelope.generation;
    return payload;
  }

  async function invokeClone(
    command:
      | typeof WORKSPACE_START_CLONE_REPOSITORY_COMMAND
      | typeof WORKSPACE_START_ANSWER_CLONE_APPROVAL_COMMAND,
    input: Readonly<Record<string, unknown>>,
  ): Promise<WorkspaceStartCloneResult> {
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
      raw = await transport.invoke(command, { request, input });
    } catch (error) {
      throw normalizeCoreFailure(error);
    }
    const envelope = parseRawEnvelope(raw);
    if (
      envelope.requestId !== requestId ||
      envelope.correlationId !== correlationId
    ) {
      throw new ProtocolBoundaryError(
        "correlationMismatch",
        "The clone response identity did not match its request.",
      );
    }
    if (envelope.generation < currentGeneration) {
      throw new ProtocolBoundaryError(
        "staleGeneration",
        "The clone response was stale.",
      );
    }
    const payload = parseCloneResult(envelope.payload);
    currentGeneration = envelope.generation;
    return payload;
  }

  async function invokeRecoveryAcknowledge(
    notice: WorkspaceRecoveryNotice,
  ): Promise<WorkspaceStartSnapshot> {
    const requestId = requestIdFactory();
    const correlationId = correlationIdFactory();
    const expectedGeneration = currentGeneration;
    const request: SnapshotRequest = {
      protocolVersion: PROTOCOL_VERSION,
      requestId,
      correlationId,
      expectedGeneration,
    };
    let raw: unknown;
    try {
      raw = await transport.invoke(WORKSPACE_RECOVERY_ACKNOWLEDGE_COMMAND, {
        request,
        input: {
          expectedGeneration,
          recoveryId: asIdentifier(notice.recoveryId, "Workspace recovery ID"),
          workspaceId: asIdentifier(
            notice.workspaceId,
            "recovered Workspace ID",
          ),
          workingGeneration: asGeneration(notice.workingGeneration),
          archiveGeneration: asGeneration(notice.archiveGeneration),
        },
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
        "The Workspace recovery acknowledgement identity did not match its request.",
      );
    }
    if (
      envelope.generation < currentGeneration ||
      envelope.payload.generation !== envelope.generation
    ) {
      throw new ProtocolBoundaryError(
        "staleGeneration",
        "The Workspace recovery acknowledgement was stale.",
      );
    }
    if (envelope.payload.activeRecoveryNotice !== null) {
      throw invalidPayload(
        "The Workspace recovery acknowledgement was not reflected in the refreshed snapshot.",
      );
    }

    currentGeneration = envelope.generation;
    return envelope.payload;
  }
}

function parseCloneResult(raw: unknown): WorkspaceStartCloneResult {
  const value = requireRecord(raw, "Workspace clone result");
  if (value.state === "opened") {
    return { state: "opened", ...parseOpenResult(value) };
  }
  if (value.state === "pendingApproval") {
    return {
      state: "pendingApproval",
      promptId: asIdentifier(value.promptId, "clone approval prompt ID"),
      summary: boundedText(value.summary, "clone approval summary", 1_024),
    };
  }
  if (value.state === "denied") return { state: "denied" };
  throw invalidPayload("The Workspace clone result state is invalid.");
}

function parseEnvelope(raw: unknown): {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly payload: WorkspaceStartSnapshot;
} {
  const envelope = parseRawEnvelope(raw);
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: envelope.requestId,
    correlationId: envelope.correlationId,
    generation: envelope.generation,
    payload: parseSnapshot(envelope.payload),
  };
}

function parseRawEnvelope(raw: unknown): {
  readonly requestId: RequestId;
  readonly correlationId: CorrelationId;
  readonly generation: StateGeneration;
  readonly payload: unknown;
} {
  const envelope = requireRecord(raw, "Workspace Start envelope");
  requireProtocolVersion(envelope.protocolVersion);
  return {
    requestId: asIdentifier(envelope.requestId, "request ID") as RequestId,
    correlationId: asIdentifier(
      envelope.correlationId,
      "correlation ID",
    ) as CorrelationId,
    generation: asGeneration(envelope.generation),
    payload: envelope.payload,
  };
}

function parseOpenResult(raw: unknown): WorkspaceStartOpenResult {
  const result = requireRecord(raw, "Workspace open result");
  if (result.authority !== "rust-workspace-service") {
    throw invalidPayload("The Workspace open authority is invalid.");
  }
  const workspaceName = requireString(result.workspaceName, "Workspace name");
  if (
    workspaceName.trim().length === 0 ||
    new TextEncoder().encode(workspaceName).length > MAX_DISPLAY_NAME_BYTES ||
    hasControlCharacter(workspaceName)
  ) {
    throw invalidPayload("The Workspace name is invalid.");
  }
  if (typeof result.recovered !== "boolean") {
    throw invalidPayload("The Workspace recovery state is invalid.");
  }
  const recoveryNotice =
    result.recoveryNotice === null || result.recoveryNotice === undefined
      ? null
      : parseRecoveryNotice(result.recoveryNotice);
  if (result.recovered !== (recoveryNotice !== null)) {
    throw invalidPayload(
      "The Workspace recovery notice does not match the recovery state.",
    );
  }
  const workspaceId = asIdentifier(
    result.workspaceId,
    "Workspace ID",
  ) as WorkspaceId;
  if (
    recoveryNotice !== null &&
    (recoveryNotice.workspaceId !== workspaceId ||
      recoveryNotice.workspaceName !== workspaceName)
  ) {
    throw invalidPayload(
      "The Workspace recovery notice does not match the opened Workspace.",
    );
  }
  return {
    authority: "rust-workspace-service",
    workspaceId,
    workspaceName,
    recovered: result.recovered,
    recoveryNotice,
  };
}

function parseRecoveryNotice(raw: unknown): WorkspaceRecoveryNotice {
  const notice = requireRecord(raw, "Workspace recovery notice");
  if (
    notice.action !== "review_recovered_workspace_before_save" ||
    notice.mustNotifyBeforeNextSave !== true
  ) {
    throw invalidPayload("The Workspace recovery action is invalid.");
  }
  return {
    recoveryId: asIdentifier(notice.recoveryId, "Workspace recovery ID"),
    correlationId: asIdentifier(
      notice.correlationId,
      "Workspace recovery correlation ID",
    ) as CorrelationId,
    workspaceId: asIdentifier(
      notice.workspaceId,
      "recovered Workspace ID",
    ) as WorkspaceId,
    workspaceName: boundedText(
      notice.workspaceName,
      "recovered Workspace name",
      MAX_DISPLAY_NAME_BYTES,
    ),
    summary: boundedText(notice.summary, "Workspace recovery summary", 1_024),
    action: "review_recovered_workspace_before_save",
    workingGeneration: asGeneration(notice.workingGeneration),
    archiveGeneration: asGeneration(notice.archiveGeneration),
    mustNotifyBeforeNextSave: asBoolean(
      notice.mustNotifyBeforeNextSave,
      "Workspace recovery notification state",
    ),
  };
}

function hasControlCharacter(value: string): boolean {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return codePoint < 32 || codePoint === 127;
  });
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
    activeRecoveryNotice:
      snapshot.activeRecoveryNotice === null ||
      snapshot.activeRecoveryNotice === undefined
        ? null
        : parseRecoveryNotice(snapshot.activeRecoveryNotice),
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

function asBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return value;
}

function boundedText(value: unknown, label: string, maximum: number): string {
  const text = requireString(value, label);
  if (
    text.trim().length === 0 ||
    text.length > maximum ||
    hasControlCharacter(text)
  ) {
    throw invalidPayload(`The ${label} is invalid.`);
  }
  return text;
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
