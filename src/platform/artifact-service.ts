import { invokeNative } from "./native-transport";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type ArtifactId,
  type CorrelationId,
  type PickerGrantId,
  type ProjectId,
  type RequestId,
  type SessionId,
  type SnapshotRequest,
  type StateGeneration,
  type WorkspaceId,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";
import {
  parseArtifactContext,
  type ConversationArtifactContextSnapshot,
} from "./conversation-service";
import type { ArtifactBreadcrumbSnapshot as GeneratedArtifactBreadcrumbSnapshot } from "../generated/ArtifactBreadcrumbSnapshot";
import type { ArtifactApprovalInput as GeneratedArtifactApprovalInput } from "../generated/ArtifactApprovalInput";
import type { ArtifactContextExpandInput as GeneratedArtifactContextExpandInput } from "../generated/ArtifactContextExpandInput";
import type { ArtifactFileDraftInput as GeneratedArtifactFileDraftInput } from "../generated/ArtifactFileDraftInput";
import type { ArtifactFileConflictInput as GeneratedArtifactFileConflictInput } from "../generated/ArtifactFileConflictInput";
import type { ArtifactFileSnapshot as GeneratedArtifactFileSnapshot } from "../generated/ArtifactFileSnapshot";
import type { ArtifactFileStateSnapshot as GeneratedArtifactFileStateSnapshot } from "../generated/ArtifactFileStateSnapshot";
import type { ArtifactFolderSnapshot as GeneratedArtifactFolderSnapshot } from "../generated/ArtifactFolderSnapshot";
import type { ArtifactFolderNavigateInput as GeneratedArtifactFolderNavigateInput } from "../generated/ArtifactFolderNavigateInput";
import type { ArtifactFolderSelectInput as GeneratedArtifactFolderSelectInput } from "../generated/ArtifactFolderSelectInput";
import type { ArtifactMutationInput as GeneratedArtifactMutationInput } from "../generated/ArtifactMutationInput";
import type { ArtifactOpenInput as GeneratedArtifactOpenInput } from "../generated/ArtifactOpenInput";
import type { ArtifactProviderStateSnapshot as GeneratedArtifactProviderStateSnapshot } from "../generated/ArtifactProviderStateSnapshot";
import type { ArtifactReplyInput as GeneratedArtifactReplyInput } from "../generated/ArtifactReplyInput";
import type { ArtifactResourceVersionSnapshot as GeneratedArtifactResourceVersionSnapshot } from "../generated/ArtifactResourceVersionSnapshot";
import type { ArtifactShellStatusSnapshot as GeneratedArtifactShellStatusSnapshot } from "../generated/ArtifactShellStatusSnapshot";
import type { ArtifactSnapshot as GeneratedArtifactSnapshot } from "../generated/ArtifactSnapshot";
import type { ArtifactWorkspaceSnapshot as GeneratedArtifactWorkspaceSnapshot } from "../generated/ArtifactWorkspaceSnapshot";

export type ArtifactShellStatus = Omit<
  Readonly<GeneratedArtifactShellStatusSnapshot>,
  "kind" | "message"
> &
  (
    | { readonly kind: "ready"; readonly message?: string }
    | {
        readonly kind: "loading" | "error" | "degraded" | "recovery";
        readonly message: string;
      }
  );

export type ArtifactBreadcrumbSnapshot =
  Readonly<GeneratedArtifactBreadcrumbSnapshot>;

export type ArtifactResourceVersionSnapshot =
  Readonly<GeneratedArtifactResourceVersionSnapshot>;

export type ArtifactFileStateSnapshot =
  | Readonly<
      Extract<GeneratedArtifactFileStateSnapshot, { readonly phase: "read" }>
    >
  | Readonly<
      Extract<
        GeneratedArtifactFileStateSnapshot,
        { readonly phase: "edit" | "dirty" }
      >
    >
  | (Omit<
      Readonly<
        Extract<
          GeneratedArtifactFileStateSnapshot,
          { readonly phase: "proposed" }
        >
      >,
      "proposalDiff"
    > & { readonly proposalDiff?: string })
  | (Omit<
      Readonly<
        Extract<
          GeneratedArtifactFileStateSnapshot,
          { readonly phase: "approval" }
        >
      >,
      "proposalDiff"
    > & { readonly proposalDiff?: string })
  | Readonly<
      Extract<
        GeneratedArtifactFileStateSnapshot,
        { readonly phase: "conflict" }
      >
    >
  | Readonly<
      Extract<
        GeneratedArtifactFileStateSnapshot,
        { readonly phase: "recovery" }
      >
    >;

export type ArtifactFileSnapshot = Omit<
  Readonly<GeneratedArtifactFileSnapshot>,
  "breadcrumbs" | "state"
> & {
  readonly breadcrumbs: readonly ArtifactBreadcrumbSnapshot[];
  readonly state: ArtifactFileStateSnapshot;
};

export type ArtifactFolderSnapshot = Omit<
  Readonly<GeneratedArtifactFolderSnapshot>,
  "breadcrumbs" | "entries" | "listing"
> & {
  readonly breadcrumbs: readonly ArtifactBreadcrumbSnapshot[];
  readonly entries: readonly {
    readonly id: string;
    readonly kind: "file" | "folder";
    readonly metadata: string | null;
    readonly name: string;
  }[];
  readonly listing: {
    readonly phase: "ready" | "loading" | "error";
    readonly message: string | null;
  };
};

export type ArtifactProviderStateSnapshot =
  | (Omit<
      Extract<
        GeneratedArtifactProviderStateSnapshot,
        { readonly type: "file" }
      >,
      "value"
    > & { readonly value: ArtifactFileSnapshot })
  | (Omit<
      Extract<
        GeneratedArtifactProviderStateSnapshot,
        { readonly type: "folder" }
      >,
      "value"
    > & { readonly value: ArtifactFolderSnapshot })
  | Readonly<
      Extract<
        GeneratedArtifactProviderStateSnapshot,
        { readonly type: "unknown" }
      >
    >;

export type ArtifactSnapshot = Omit<
  Readonly<GeneratedArtifactSnapshot>,
  | "artifactId"
  | "history"
  | "projectId"
  | "providerState"
  | "resourceVersion"
  | "sessionId"
  | "status"
> & {
  readonly artifactId: ArtifactId;
  readonly projectId: ProjectId;
  readonly sessionId: SessionId;
  readonly status: ArtifactShellStatus;
  readonly resourceVersion: ArtifactResourceVersionSnapshot;
  readonly history: readonly {
    readonly recordRevision: number;
    readonly kind: string;
    readonly recordedAtMs: number;
    readonly resourceVersion: ArtifactResourceVersionSnapshot;
  }[];
  readonly providerState: ArtifactProviderStateSnapshot;
};

export type ArtifactWorkspaceSnapshot = Omit<
  Readonly<GeneratedArtifactWorkspaceSnapshot>,
  | "activeProjectId"
  | "activeSessionId"
  | "artifacts"
  | "authority"
  | "focusedArtifactId"
  | "generation"
  | "protocolVersion"
  | "workspaceId"
> & {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly generation: StateGeneration;
  readonly authority: "rust-core";
  readonly workspaceId: WorkspaceId | null;
  readonly activeProjectId: ProjectId | null;
  readonly activeSessionId: SessionId | null;
  readonly focusedArtifactId: ArtifactId | null;
  readonly artifacts: readonly ArtifactSnapshot[];
};

export type ArtifactMutationInput = Omit<
  Readonly<GeneratedArtifactMutationInput>,
  "artifactId"
> & { readonly artifactId: ArtifactId };

export type ArtifactReplyInput = Omit<
  Readonly<GeneratedArtifactReplyInput>,
  "artifactId" | "selectedEntryId" | "selectedText"
> & {
  readonly artifactId: ArtifactId;
  readonly selectedEntryId?: string;
  readonly selectedText?: string;
};

export type ArtifactContextExpandInput = Omit<
  Readonly<GeneratedArtifactContextExpandInput>,
  "artifactId" | "expectedResourceVersion" | "selectedEntryId" | "selectedText"
> & {
  readonly artifactId: ArtifactId;
  readonly expectedResourceVersion: ArtifactResourceVersionSnapshot;
  readonly selectedEntryId?: string;
  readonly selectedText?: string;
};

export type ArtifactFileConflictInput = Omit<
  Readonly<GeneratedArtifactFileConflictInput>,
  "artifactId"
> & { readonly artifactId: ArtifactId };

export type ArtifactFileDraftInput = Omit<
  Readonly<GeneratedArtifactFileDraftInput>,
  "artifactId"
> & { readonly artifactId: ArtifactId };

export type ArtifactFolderNavigateInput = Omit<
  Readonly<GeneratedArtifactFolderNavigateInput>,
  "artifactId"
> & { readonly artifactId: ArtifactId };

export type ArtifactFolderSelectInput = Omit<
  Readonly<GeneratedArtifactFolderSelectInput>,
  "artifactId"
> & { readonly artifactId: ArtifactId };

export type ArtifactApprovalInput = Readonly<GeneratedArtifactApprovalInput>;

export type ArtifactCommand =
  | "artifact_snapshot"
  | "artifact_open_file"
  | "artifact_open_folder"
  | "artifact_focus"
  | "artifact_close_focus"
  | "artifact_begin_file_edit"
  | "artifact_update_file_draft"
  | "artifact_discard_file_draft"
  | "artifact_reject_file_proposal"
  | "artifact_resolve_file_conflict"
  | "artifact_save_file"
  | "artifact_answer_approval"
  | "artifact_refresh_folder"
  | "artifact_navigate_folder"
  | "artifact_select_folder_entry"
  | "artifact_reply"
  | "artifact_expand_context";

export interface ArtifactTransport {
  invoke(
    command: ArtifactCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface ArtifactAdapter {
  readonly currentGeneration: StateGeneration;
  read(): Promise<ArtifactWorkspaceSnapshot>;
  openFile(pickerGrantId: PickerGrantId): Promise<ArtifactWorkspaceSnapshot>;
  openFolder(pickerGrantId: PickerGrantId): Promise<ArtifactWorkspaceSnapshot>;
  focus(input: ArtifactMutationInput): Promise<ArtifactWorkspaceSnapshot>;
  closeFocus(): Promise<ArtifactWorkspaceSnapshot>;
  beginFileEdit(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  updateFileDraft(
    input: ArtifactFileDraftInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  discardFileDraft(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  rejectFileProposal(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  resolveFileConflict(
    input: ArtifactFileConflictInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  saveFile(input: ArtifactMutationInput): Promise<ArtifactWorkspaceSnapshot>;
  answerApproval(
    input: ArtifactApprovalInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  refreshFolder(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  navigateFolder(
    input: ArtifactFolderNavigateInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  selectFolderEntry(
    input: ArtifactFolderSelectInput,
  ): Promise<ArtifactWorkspaceSnapshot>;
  reply(input: ArtifactReplyInput): Promise<ArtifactWorkspaceSnapshot>;
  expandContext(
    input: ArtifactContextExpandInput,
  ): Promise<ConversationArtifactContextSnapshot>;
}

class NativeArtifactAdapter implements ArtifactAdapter {
  currentGeneration = 0 as StateGeneration;

  constructor(private readonly transport: ArtifactTransport) {}

  read(): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_snapshot", {});
  }

  openFile(pickerGrantId: PickerGrantId): Promise<ArtifactWorkspaceSnapshot> {
    const input: Readonly<GeneratedArtifactOpenInput> = { pickerGrantId };
    return this.invoke("artifact_open_file", {
      input,
    });
  }

  openFolder(pickerGrantId: PickerGrantId): Promise<ArtifactWorkspaceSnapshot> {
    const input: Readonly<GeneratedArtifactOpenInput> = { pickerGrantId };
    return this.invoke("artifact_open_folder", {
      input,
    });
  }

  focus(input: ArtifactMutationInput): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_focus", { input });
  }

  closeFocus(): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_close_focus", {});
  }

  beginFileEdit(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_begin_file_edit", { input });
  }

  updateFileDraft(
    input: ArtifactFileDraftInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_update_file_draft", { input });
  }

  discardFileDraft(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_discard_file_draft", { input });
  }

  rejectFileProposal(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_reject_file_proposal", { input });
  }

  resolveFileConflict(
    input: ArtifactFileConflictInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_resolve_file_conflict", { input });
  }

  saveFile(input: ArtifactMutationInput): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_save_file", { input });
  }

  answerApproval(
    input: ArtifactApprovalInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_answer_approval", { input });
  }

  refreshFolder(
    input: ArtifactMutationInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_refresh_folder", { input });
  }

  navigateFolder(
    input: ArtifactFolderNavigateInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_navigate_folder", { input });
  }

  selectFolderEntry(
    input: ArtifactFolderSelectInput,
  ): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_select_folder_entry", { input });
  }

  reply(input: ArtifactReplyInput): Promise<ArtifactWorkspaceSnapshot> {
    return this.invoke("artifact_reply", { input });
  }

  async expandContext(
    input: ArtifactContextExpandInput,
  ): Promise<ConversationArtifactContextSnapshot> {
    const request = makeRequest(this.currentGeneration);
    let raw: unknown;
    try {
      raw = await this.transport.invoke("artifact_expand_context", {
        request,
        input,
      });
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = record(raw, "Artifact context envelope");
    if (
      envelope.protocolVersion !== PROTOCOL_VERSION ||
      envelope.requestId !== request.requestId ||
      envelope.correlationId !== request.correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "Artifact context response identity changed.",
      );
    }
    const generation = nonnegative(envelope.generation, "generation");
    if (generation < this.currentGeneration) {
      throw boundary("staleGeneration", "Artifact context response was stale.");
    }
    const context = parseArtifactContext(envelope.payload);
    if (
      context.artifactId !== input.artifactId ||
      context.artifactRecordRevision !== input.baseRecordRevision ||
      context.capturedResourceVersion.sequence !==
        input.expectedResourceVersion.sequence ||
      context.capturedResourceVersion.sha256 !==
        input.expectedResourceVersion.sha256 ||
      context.capturedResourceVersion.observedAtMs !==
        input.expectedResourceVersion.observedAtMs ||
      context.maximumBytes !== input.maximumBytes
    ) {
      throw boundary(
        "invalidPayload",
        "Artifact context expansion changed its requested authority.",
      );
    }
    this.currentGeneration = generation as StateGeneration;
    return context;
  }

  private async invoke(
    command: ArtifactCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<ArtifactWorkspaceSnapshot> {
    const request = makeRequest(this.currentGeneration);
    let raw: unknown;
    try {
      raw = await this.transport.invoke(command, { request, ...args });
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = record(raw, "Artifact envelope");
    if (
      envelope.protocolVersion !== PROTOCOL_VERSION ||
      envelope.requestId !== request.requestId ||
      envelope.correlationId !== request.correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "Artifact response identity changed.",
      );
    }
    const generation = nonnegative(envelope.generation, "generation");
    if (generation < this.currentGeneration) {
      throw boundary("staleGeneration", "Artifact response was stale.");
    }
    const snapshot = parseWorkspaceSnapshot(envelope.payload);
    if (snapshot.generation !== generation) {
      throw boundary(
        "invalidGeneration",
        "Artifact payload generation changed.",
      );
    }
    this.currentGeneration = generation as StateGeneration;
    return snapshot;
  }
}

export function createArtifactAdapter(
  transport: ArtifactTransport,
): ArtifactAdapter {
  return new NativeArtifactAdapter(transport);
}

const nativeAdapter = createArtifactAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readArtifactWorkspaceSnapshot = () => nativeAdapter.read();
export const openFileArtifact = (pickerGrantId: PickerGrantId) =>
  nativeAdapter.openFile(pickerGrantId);
export const openFolderArtifact = (pickerGrantId: PickerGrantId) =>
  nativeAdapter.openFolder(pickerGrantId);
export const focusArtifact = (input: ArtifactMutationInput) =>
  nativeAdapter.focus(input);
export const closeArtifactFocus = () => nativeAdapter.closeFocus();
export const beginFileArtifactEdit = (input: ArtifactMutationInput) =>
  nativeAdapter.beginFileEdit(input);
export const updateFileArtifactDraft = (input: ArtifactFileDraftInput) =>
  nativeAdapter.updateFileDraft(input);
export const discardFileArtifactDraft = (input: ArtifactMutationInput) =>
  nativeAdapter.discardFileDraft(input);
export const rejectFileArtifactProposal = (input: ArtifactMutationInput) =>
  nativeAdapter.rejectFileProposal(input);
export const resolveFileArtifactConflict = (input: ArtifactFileConflictInput) =>
  nativeAdapter.resolveFileConflict(input);
export const saveFileArtifact = (input: ArtifactMutationInput) =>
  nativeAdapter.saveFile(input);
export const answerArtifactApproval = (input: ArtifactApprovalInput) =>
  nativeAdapter.answerApproval(input);
export const refreshFolderArtifact = (input: ArtifactMutationInput) =>
  nativeAdapter.refreshFolder(input);
export const navigateFolderArtifact = (input: ArtifactFolderNavigateInput) =>
  nativeAdapter.navigateFolder(input);
export const selectFolderArtifactEntry = (input: ArtifactFolderSelectInput) =>
  nativeAdapter.selectFolderEntry(input);
export const replyToArtifact = (input: ArtifactReplyInput) =>
  nativeAdapter.reply(input);
export const expandArtifactContext = (input: ArtifactContextExpandInput) =>
  nativeAdapter.expandContext(input);

function parseWorkspaceSnapshot(raw: unknown): ArtifactWorkspaceSnapshot {
  const value = record(raw, "Artifact Workspace snapshot");
  if (
    value.protocolVersion !== PROTOCOL_VERSION ||
    value.authority !== "rust-core"
  ) {
    throw boundary("invalidPayload", "Artifact authority is invalid.");
  }
  const artifacts = array(value.artifacts, 4_096, "artifacts").map(
    parseArtifact,
  );
  const ids = new Set(artifacts.map(({ artifactId }) => artifactId));
  if (ids.size !== artifacts.length) {
    throw boundary("invalidPayload", "Artifact identities are repeated.");
  }
  const focusedArtifactId = nullableIdentifier(
    value.focusedArtifactId,
    "focused artifact ID",
  ) as ArtifactId | null;
  if (
    focusedArtifactId !== null &&
    !artifacts.some(
      (artifact) =>
        artifact.artifactId === focusedArtifactId && artifact.focusSupported,
    )
  ) {
    throw boundary("invalidPayload", "Focused artifact is unavailable.");
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    generation: nonnegative(value.generation, "generation") as StateGeneration,
    authority: "rust-core",
    workspaceId: nullableIdentifier(
      value.workspaceId,
      "Workspace ID",
    ) as WorkspaceId | null,
    activeProjectId: nullableIdentifier(
      value.activeProjectId,
      "Project ID",
    ) as ProjectId | null,
    activeSessionId: nullableIdentifier(
      value.activeSessionId,
      "session ID",
    ) as SessionId | null,
    focusedArtifactId,
    artifacts,
  };
}

function parseArtifact(raw: unknown): ArtifactSnapshot {
  const value = record(raw, "artifact");
  const providerType = identifier(value.providerType, "provider type");
  const providerState = parseProviderState(value.providerState);
  const providerVersion = positive(value.providerVersion, "provider version");
  const stateSchemaVersion = positive(
    value.stateSchemaVersion,
    "state schema version",
  );
  if (
    (providerState.type === "file" && providerType !== "file") ||
    (providerState.type === "folder" && providerType !== "folder") ||
    (providerState.type === "file" &&
      (providerVersion !== 1 || stateSchemaVersion !== 1)) ||
    (providerState.type === "folder" &&
      (providerVersion !== 1 || stateSchemaVersion !== 1))
  ) {
    throw boundary(
      "invalidPayload",
      "Artifact provider state is inconsistent.",
    );
  }
  const focusSupported = booleanValue(value.focusSupported, "focus support");
  const status = parseStatus(value.status);
  const pendingApprovalId = nullableIdentifier(
    value.pendingApprovalId,
    "Artifact approval prompt ID",
  );
  const supportedKnownVersion =
    (providerType === "file" || providerType === "folder") &&
    providerVersion === 1 &&
    stateSchemaVersion === 1;
  if (
    providerState.type === "unknown" &&
    (focusSupported || status.kind !== "degraded" || supportedKnownVersion)
  ) {
    throw boundary("invalidPayload", "Unknown artifacts must fail closed.");
  }
  const approvalState =
    providerState.type === "file" &&
    providerState.value.state.phase === "approval";
  if ((pendingApprovalId !== null) !== approvalState) {
    throw boundary(
      "invalidPayload",
      "Artifact approval state is inconsistent.",
    );
  }
  return {
    artifactId: identifier(value.artifactId, "artifact ID") as ArtifactId,
    projectId: identifier(value.projectId, "Project ID") as ProjectId,
    sessionId: identifier(value.sessionId, "session ID") as SessionId,
    providerType,
    providerVersion,
    stateSchemaVersion,
    recordRevision: positive(value.recordRevision, "record revision"),
    title: text(value.title, "artifact title", 512),
    focusSupported,
    pendingApprovalId,
    status,
    sourceLabel: text(value.sourceLabel, "artifact source", 512),
    resourceVersion: parseResourceVersion(value.resourceVersion),
    history: array(value.history, 256, "artifact history").map((rawHistory) => {
      const history = record(rawHistory, "artifact history entry");
      return {
        recordRevision: positive(history.recordRevision, "history revision"),
        kind: identifier(history.kind, "history kind"),
        recordedAtMs: positive(history.recordedAtMs, "history timestamp"),
        resourceVersion: parseResourceVersion(history.resourceVersion),
      };
    }),
    providerState,
  };
}

function parseProviderState(raw: unknown): ArtifactProviderStateSnapshot {
  const value = record(raw, "artifact provider state");
  const type = value.type;
  if (type === "unknown") return { type };
  if (type === "file") {
    const file = record(value.value, "File provider state");
    const state = record(file.state, "File state");
    const phase = state.phase;
    if (
      typeof phase !== "string" ||
      !new Set([
        "read",
        "edit",
        "dirty",
        "proposed",
        "approval",
        "conflict",
        "recovery",
      ]).has(phase)
    ) {
      throw boundary("invalidPayload", "File phase is invalid.");
    }
    const content = textAllowEmpty(state.content, "File content", 1_048_576);
    const parsedState: ArtifactFileStateSnapshot = (() => {
      switch (phase) {
        case "read":
          return { phase, content };
        case "edit":
        case "dirty":
          return {
            phase,
            content,
            draft: textAllowEmpty(state.draft, "File draft", 1_048_576),
          };
        case "proposed": {
          const proposalDiff = nullableText(
            state.proposalDiff,
            "File proposal diff",
          );
          return {
            phase,
            content,
            proposedContent: textAllowEmpty(
              state.proposedContent,
              "File proposal",
              1_048_576,
            ),
            ...(proposalDiff === null ? {} : { proposalDiff }),
            proposalSummary: text(state.proposalSummary, "proposal summary"),
          };
        }
        case "approval": {
          const proposalDiff = nullableText(
            state.proposalDiff,
            "File proposal diff",
          );
          return {
            phase,
            content,
            proposedContent: textAllowEmpty(
              state.proposedContent,
              "File proposal",
              1_048_576,
            ),
            ...(proposalDiff === null ? {} : { proposalDiff }),
            approvalSummary: text(state.approvalSummary, "approval summary"),
          };
        }
        case "conflict":
          return {
            phase,
            content,
            draft: textAllowEmpty(state.draft, "File draft", 1_048_576),
            conflictMessage: text(state.conflictMessage, "conflict message"),
            currentVersionLabel: text(
              state.currentVersionLabel,
              "current version",
            ),
          };
        case "recovery":
          return {
            phase,
            content,
            draft: textAllowEmpty(state.draft, "File draft", 1_048_576),
            recoveryMessage: text(state.recoveryMessage, "recovery message"),
          };
        default:
          throw boundary("invalidPayload", "File phase is invalid.");
      }
    })();
    return {
      type,
      value: {
        breadcrumbs: parseBreadcrumbs(file.breadcrumbs),
        languageLabel: nullableText(file.languageLabel, "File language"),
        versionLabel: nullableText(file.versionLabel, "File version"),
        state: parsedState,
      },
    };
  }
  if (type === "folder") {
    const folder = record(value.value, "Folder provider state");
    const listing = record(folder.listing, "Folder listing state");
    const phase = listing.phase;
    if (phase !== "ready" && phase !== "loading" && phase !== "error") {
      throw boundary("invalidPayload", "Folder listing phase is invalid.");
    }
    return {
      type,
      value: {
        breadcrumbs: parseBreadcrumbs(folder.breadcrumbs),
        entries: array(folder.entries, 512, "Folder entries").map(
          (rawEntry) => {
            const entry = record(rawEntry, "Folder entry");
            const kind = entry.kind;
            if (kind !== "file" && kind !== "folder") {
              throw boundary("invalidPayload", "Folder entry kind is invalid.");
            }
            return {
              id: identifier(entry.id, "Folder entry ID"),
              kind,
              metadata: nullableText(entry.metadata, "Folder entry metadata"),
              name: text(entry.name, "Folder entry name", 512),
            };
          },
        ),
        listing: {
          phase,
          message: nullableText(listing.message, "Folder listing message"),
        },
        listingLimit: positive(folder.listingLimit, "Folder listing limit"),
        selectedEntryId: nullableIdentifier(
          folder.selectedEntryId,
          "Folder selection",
        ),
      },
    };
  }
  throw boundary("invalidPayload", "Artifact provider state is unknown.");
}

function parseBreadcrumbs(raw: unknown): readonly ArtifactBreadcrumbSnapshot[] {
  const breadcrumbs = array(raw, 128, "artifact breadcrumbs").map((item) => {
    const value = record(item, "artifact breadcrumb");
    return {
      id: textAllowEmpty(value.id, "breadcrumb ID", 4_096),
      label: text(value.label, "breadcrumb label", 512),
      isCurrent: booleanValue(value.isCurrent, "breadcrumb current state"),
    };
  });
  if (
    breadcrumbs.length === 0 ||
    breadcrumbs.filter(({ isCurrent }) => isCurrent).length !== 1 ||
    !breadcrumbs.at(-1)?.isCurrent
  ) {
    throw boundary("invalidPayload", "Artifact breadcrumbs are invalid.");
  }
  return breadcrumbs;
}

function parseStatus(raw: unknown): ArtifactShellStatus {
  const value = record(raw, "artifact status");
  const kind = value.kind;
  if (kind === "ready") {
    const message = nullableText(value.message, "artifact status");
    return message === null ? { kind } : { kind, message };
  }
  if (
    kind === "loading" ||
    kind === "error" ||
    kind === "degraded" ||
    kind === "recovery"
  ) {
    return { kind, message: text(value.message, "artifact status", 8_192) };
  }
  throw boundary("invalidPayload", "Artifact status is invalid.");
}

function parseResourceVersion(raw: unknown): ArtifactResourceVersionSnapshot {
  const value = record(raw, "artifact resource version");
  const sha256 = text(value.sha256, "resource digest", 71);
  if (!/^sha256:[0-9a-fA-F]{64}$/.test(sha256)) {
    throw boundary("invalidPayload", "Artifact resource digest is invalid.");
  }
  return {
    sequence: positive(value.sequence, "resource sequence"),
    sha256,
    observedAtMs: positive(value.observedAtMs, "resource timestamp"),
  };
}

function makeRequest(expectedGeneration: StateGeneration): SnapshotRequest {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw boundary(
      "unavailable",
      "Secure Artifact request identity is unavailable.",
    );
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: globalThis.crypto.randomUUID() as RequestId,
    correlationId: globalThis.crypto.randomUUID() as CorrelationId,
    expectedGeneration,
  };
}

function record(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value as Record<string, unknown>;
}

function array(
  value: unknown,
  maximum: number,
  label: string,
): readonly unknown[] {
  if (!Array.isArray(value) || value.length > maximum) {
    throw boundary("invalidPayload", `${label} are invalid.`);
  }
  return value;
}

function text(value: unknown, label: string, maximum = 8_192): string {
  const result = textAllowEmpty(value, label, maximum);
  if (result.trim().length === 0) {
    throw boundary("invalidPayload", `${label} is empty.`);
  }
  return result;
}

function textAllowEmpty(
  value: unknown,
  label: string,
  maximum: number,
): string {
  if (
    typeof value !== "string" ||
    value.length > maximum ||
    value.includes("\0")
  ) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value;
}

function nullableText(value: unknown, label: string): string | null {
  return value === null || value === undefined ? null : text(value, label);
}

function identifier(value: unknown, label: string): string {
  const result = text(value, label, MAX_IDENTIFIER_BYTES);
  if (![...result].every((character) => /[A-Za-z0-9_.:@-]/.test(character))) {
    throw boundary("invalidIdentifier", `${label} is invalid.`);
  }
  return result;
}

function nullableIdentifier(value: unknown, label: string): string | null {
  return value === null || value === undefined
    ? null
    : identifier(value, label);
}

function positive(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 1) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value as number;
}

function nonnegative(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value as number;
}

function booleanValue(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value;
}

function normalizeFailure(error: unknown): Error {
  const value =
    typeof error === "object" && error !== null
      ? (error as Record<string, unknown>)
      : null;
  if (
    value &&
    typeof value.code === "string" &&
    typeof value.message === "string"
  ) {
    return boundary(value.code, value.message, value.retryable === true);
  }
  return boundary("unavailable", "Artifact service is unavailable.", true);
}

function boundary(
  code: string,
  message: string,
  retryable = false,
): ProtocolBoundaryError {
  return new ProtocolBoundaryError(code as never, message, retryable);
}
