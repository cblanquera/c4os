import { invokeNative } from "./native-transport";
import type { ConversationArtifactCapabilitySnapshot as GeneratedConversationArtifactCapabilitySnapshot } from "../generated/ConversationArtifactCapabilitySnapshot";
import type { ConversationArtifactContextSegmentSnapshot as GeneratedConversationArtifactContextSegmentSnapshot } from "../generated/ConversationArtifactContextSegmentSnapshot";
import type { ConversationArtifactContextSnapshot as GeneratedConversationArtifactContextSnapshot } from "../generated/ConversationArtifactContextSnapshot";
import {
  MAX_IDENTIFIER_BYTES,
  PROTOCOL_VERSION,
  type ArtifactId,
  type AttachmentId,
  type AttemptId,
  type CorrelationId,
  type EnvironmentId,
  type PickerGrantId,
  type ProjectId,
  type RequestId,
  type RuntimeId,
  type SessionId,
  type SnapshotRequest,
  type StateGeneration,
  type TurnId,
  type WorkspaceId,
} from "./protocol";
import { ProtocolBoundaryError } from "./tauri-adapter";
import {
  isAcceptedQaAwareAuthority,
  type QaAwareAuthority,
} from "../qa/authority";

export interface ConversationAttachmentSnapshot {
  readonly attachmentId: AttachmentId;
  readonly displayName: string;
  readonly mediaType: string;
  readonly byteLength: number;
  readonly stableReference: string;
  readonly originalReference: number;
}

export interface ConversationAttachmentPreviewInput {
  readonly attachmentId: AttachmentId;
  readonly stableReference: string;
}

export interface ConversationAttachmentPreviewSnapshot {
  readonly attachmentId: AttachmentId;
  readonly mediaType: string;
  readonly dataUrl: string;
}

export interface ConversationProjectSnapshot {
  readonly projectId: ProjectId;
  readonly displayName: string;
  readonly pathState: "found" | "missing" | "relocated";
  readonly position: number;
  readonly gitVersioned: boolean;
}

export interface ConversationBranchControlSnapshot {
  readonly currentBranch: string | null;
  readonly branches: readonly {
    readonly name: string;
    readonly targetOid: string;
    readonly selected: boolean;
  }[];
  readonly pendingApprovalId: string | null;
  readonly operationStatus:
    "pending" | "switched" | "created" | "blocked" | "denied" | null;
  readonly operationMessage: string | null;
}

export interface ConversationSessionSummarySnapshot {
  readonly sessionId: SessionId;
  readonly projectId: ProjectId;
  readonly title: string;
  readonly updatedAtMs: number;
}

type ConversationArtifactContextSegmentSnapshot = Omit<
  Readonly<GeneratedConversationArtifactContextSegmentSnapshot>,
  "priority"
> & {
  readonly priority: "selection" | "visibleOrCurrent" | "recent" | "metadata";
};

type ConversationArtifactCapabilitySnapshot = Omit<
  Readonly<GeneratedConversationArtifactCapabilitySnapshot>,
  "access"
> & {
  readonly access: "readable" | "approvalRequired" | "denied" | "unknown";
};

export type ConversationArtifactContextSnapshot = Omit<
  Readonly<GeneratedConversationArtifactContextSnapshot>,
  | "artifactId"
  | "capabilities"
  | "payloadKind"
  | "projectId"
  | "providerType"
  | "segments"
  | "sessionId"
> & {
  readonly artifactId: ArtifactId;
  readonly projectId: ProjectId;
  readonly sessionId: SessionId;
  readonly providerType: "file" | "folder" | "browser" | "terminal";
  readonly payloadKind: "file" | "folder" | "browser" | "terminal";
  readonly segments: readonly ConversationArtifactContextSegmentSnapshot[];
  readonly capabilities: readonly ConversationArtifactCapabilitySnapshot[];
};

export interface ConversationTurnSnapshot {
  readonly turnId: TurnId;
  readonly prompt: string | null;
  readonly attachments: readonly ConversationAttachmentSnapshot[];
  readonly artifactContext: ConversationArtifactContextSnapshot | null;
  readonly mcpProvenance: ConversationMcpProvenanceSnapshot | null;
  readonly submittedAtMs: number;
}

export interface ConversationMcpProvenanceSnapshot {
  readonly snapshotId: string;
  readonly serverCount: number;
  readonly toolCount: number;
  readonly omittedToolCount: number;
  readonly truncated: boolean;
  readonly tools: readonly {
    readonly serverId: string;
    readonly sourceKind: "user" | "plugin";
    readonly sourceId: string | null;
    readonly toolName: string;
  }[];
}

export interface ConversationAttemptSnapshot {
  readonly attemptId: AttemptId;
  readonly turnId: TurnId;
  readonly status:
    | "starting"
    | "working"
    | "cancelling"
    | "completed"
    | "failed"
    | "interrupted"
    | "cancelled";
  readonly assistantMarkdown: string;
  readonly activities: readonly {
    readonly sequence: number;
    readonly kind: string;
    readonly label: string;
    readonly detail: string | null;
  }[];
  readonly runtimeId: RuntimeId;
  readonly runtimeKind: "open-code" | "pi";
  readonly environmentId: EnvironmentId;
  readonly providerId: string;
  readonly modelId: string;
  readonly adapterId: string;
  readonly inputTokens: number;
  readonly outputTokens: number;
  readonly durationMs: number | null;
}

export interface ConversationSessionSnapshot {
  readonly sessionId: SessionId;
  readonly title: string | null;
  readonly turns: readonly ConversationTurnSnapshot[];
  readonly attempts: readonly ConversationAttemptSnapshot[];
  readonly activeAttemptId: AttemptId | null;
}

export interface ConversationModelSnapshot {
  readonly providerId: string;
  readonly providerName: string;
  readonly modelId: string;
  readonly selected: boolean;
  readonly available: boolean;
  readonly supportsVision: boolean;
  readonly supportsTools: boolean;
  readonly supportsReasoning: boolean;
  readonly supportsAudio: boolean;
  readonly contextTokens: number;
}

export interface ConversationSnapshot {
  readonly protocolVersion: typeof PROTOCOL_VERSION;
  readonly generation: StateGeneration;
  readonly authority: QaAwareAuthority<"rust-core">;
  readonly workspaceId: WorkspaceId | null;
  readonly workspaceName: string | null;
  readonly activeProjectId: ProjectId | null;
  readonly activeSessionId: SessionId | null;
  readonly pending: {
    readonly sessionId: SessionId;
    readonly projectId: ProjectId;
    readonly title: string;
    readonly attachments: readonly ConversationAttachmentSnapshot[];
  } | null;
  readonly draft: {
    readonly prompt: string;
    readonly attachments: readonly ConversationAttachmentSnapshot[];
    readonly nextAttachmentReference: number;
    readonly providerId: string | null;
    readonly modelId: string | null;
    readonly reasoningMode: "off" | "low" | "medium" | "high" | null;
    readonly mode: "chat" | "files" | "browser" | "terminal";
    readonly replyTargetId: string | null;
  };
  readonly projects: readonly ConversationProjectSnapshot[];
  readonly sessions: readonly ConversationSessionSummarySnapshot[];
  readonly activeConversation: ConversationSessionSnapshot | null;
  readonly models: readonly ConversationModelSnapshot[];
  readonly branchControl: ConversationBranchControlSnapshot | null;
}

export interface ConversationSubmitInput {
  readonly prompt: string | null;
  readonly pickerGrantIds: readonly PickerGrantId[];
  readonly retainedAttachmentIds: readonly AttachmentId[];
  readonly providerId: string | null;
  readonly modelId: string | null;
  readonly reasoningMode: string | null;
  readonly resumeMode: "chat" | "files" | "browser" | "terminal";
}

export interface ConversationDraftInput {
  readonly prompt: string;
  readonly pickerGrantIds: readonly PickerGrantId[];
  readonly retainedAttachmentIds: readonly AttachmentId[];
  readonly providerId: string | null;
  readonly modelId: string | null;
  readonly reasoningMode: string | null;
  readonly mode: "chat" | "files" | "browser" | "terminal";
  readonly replyTargetId: string | null;
}

export interface ConversationRetryInput {
  readonly parentAttemptId: AttemptId;
  readonly providerId: string | null;
  readonly modelId: string | null;
  readonly reasoningMode: string | null;
}

export interface ConversationBranchInput {
  readonly operation: "switch" | "create";
  readonly branch: string;
}

export interface ConversationBranchApprovalInput {
  readonly promptId: string;
  readonly answer: "allow" | "deny";
}

export type ConversationCommand =
  | "conversation_snapshot"
  | "conversation_attachment_preview"
  | "conversation_request_branch"
  | "conversation_answer_branch_approval"
  | "conversation_begin_pending"
  | "conversation_cancel_pending"
  | "conversation_update_draft"
  | "conversation_submit"
  | "conversation_cancel_attempt"
  | "conversation_retry_attempt"
  | "conversation_activate_session"
  | "conversation_activate_project"
  | "conversation_add_project"
  | "conversation_relocate_project"
  | "conversation_rename_project"
  | "conversation_copy_project_path"
  | "conversation_reveal_project"
  | "conversation_reorder_projects"
  | "conversation_inactivate_project"
  | "conversation_inactivate_session";

export interface ConversationTransport {
  invoke(
    command: ConversationCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown>;
}

export interface ConversationAdapter {
  readonly currentGeneration: StateGeneration;
  read(): Promise<ConversationSnapshot>;
  previewAttachment(
    input: ConversationAttachmentPreviewInput,
  ): Promise<ConversationAttachmentPreviewSnapshot>;
  requestBranch(input: ConversationBranchInput): Promise<ConversationSnapshot>;
  answerBranchApproval(
    input: ConversationBranchApprovalInput,
  ): Promise<ConversationSnapshot>;
  begin(projectId: ProjectId): Promise<ConversationSnapshot>;
  cancel(): Promise<ConversationSnapshot>;
  updateDraft(input: ConversationDraftInput): Promise<ConversationSnapshot>;
  submit(input: ConversationSubmitInput): Promise<ConversationSnapshot>;
  cancelAttempt(attemptId: AttemptId): Promise<ConversationSnapshot>;
  retryAttempt(input: ConversationRetryInput): Promise<ConversationSnapshot>;
  activate(sessionId: SessionId): Promise<ConversationSnapshot>;
  activateProject(projectId: ProjectId): Promise<ConversationSnapshot>;
  addProject(pickerGrantId: PickerGrantId): Promise<ConversationSnapshot>;
  relocateProject(
    projectId: ProjectId,
    pickerGrantId: PickerGrantId,
  ): Promise<ConversationSnapshot>;
  renameProject(
    projectId: ProjectId,
    displayName: string,
  ): Promise<ConversationSnapshot>;
  copyProjectPath(projectId: ProjectId): Promise<ConversationSnapshot>;
  revealProject(projectId: ProjectId): Promise<ConversationSnapshot>;
  reorderProjects(
    orderedProjectIds: readonly ProjectId[],
  ): Promise<ConversationSnapshot>;
  inactivateProject(projectId: ProjectId): Promise<ConversationSnapshot>;
  inactivateSession(sessionId: SessionId): Promise<ConversationSnapshot>;
}

class NativeConversationAdapter implements ConversationAdapter {
  currentGeneration = 0 as StateGeneration;

  constructor(private readonly transport: ConversationTransport) {}

  read(): Promise<ConversationSnapshot> {
    return this.invoke("conversation_snapshot", {});
  }

  async previewAttachment(
    input: ConversationAttachmentPreviewInput,
  ): Promise<ConversationAttachmentPreviewSnapshot> {
    const request = makeRequest(this.currentGeneration);
    let raw: unknown;
    try {
      raw = await this.transport.invoke("conversation_attachment_preview", {
        request,
        input,
      });
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = record(raw, "Conversation attachment preview envelope");
    if (
      envelope.protocolVersion !== PROTOCOL_VERSION ||
      envelope.requestId !== request.requestId ||
      envelope.correlationId !== request.correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "Conversation attachment preview identity changed.",
      );
    }
    const generation = generationValue(envelope.generation);
    if (generation < this.currentGeneration) {
      throw boundary(
        "staleGeneration",
        "Conversation attachment preview was stale.",
      );
    }
    return parseAttachmentPreview(envelope.payload, input);
  }

  requestBranch(input: ConversationBranchInput): Promise<ConversationSnapshot> {
    return this.invoke("conversation_request_branch", { input });
  }

  answerBranchApproval(
    input: ConversationBranchApprovalInput,
  ): Promise<ConversationSnapshot> {
    return this.invoke("conversation_answer_branch_approval", { input });
  }

  begin(projectId: ProjectId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_begin_pending", { projectId });
  }

  cancel(): Promise<ConversationSnapshot> {
    return this.invoke("conversation_cancel_pending", {});
  }

  updateDraft(input: ConversationDraftInput): Promise<ConversationSnapshot> {
    return this.invoke("conversation_update_draft", { input });
  }

  submit(input: ConversationSubmitInput): Promise<ConversationSnapshot> {
    return this.invoke("conversation_submit", { input });
  }

  cancelAttempt(attemptId: AttemptId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_cancel_attempt", { attemptId });
  }

  retryAttempt(input: ConversationRetryInput): Promise<ConversationSnapshot> {
    return this.invoke("conversation_retry_attempt", { input });
  }

  activate(sessionId: SessionId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_activate_session", { sessionId });
  }

  activateProject(projectId: ProjectId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_activate_project", { projectId });
  }

  addProject(pickerGrantId: PickerGrantId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_add_project", { pickerGrantId });
  }

  relocateProject(
    projectId: ProjectId,
    pickerGrantId: PickerGrantId,
  ): Promise<ConversationSnapshot> {
    return this.invoke("conversation_relocate_project", {
      projectId,
      pickerGrantId,
    });
  }

  renameProject(
    projectId: ProjectId,
    displayName: string,
  ): Promise<ConversationSnapshot> {
    return this.invoke("conversation_rename_project", {
      projectId,
      displayName,
    });
  }

  copyProjectPath(projectId: ProjectId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_copy_project_path", { projectId });
  }

  revealProject(projectId: ProjectId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_reveal_project", { projectId });
  }

  reorderProjects(
    orderedProjectIds: readonly ProjectId[],
  ): Promise<ConversationSnapshot> {
    return this.invoke("conversation_reorder_projects", {
      orderedProjectIds,
    });
  }

  inactivateProject(projectId: ProjectId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_inactivate_project", { projectId });
  }

  inactivateSession(sessionId: SessionId): Promise<ConversationSnapshot> {
    return this.invoke("conversation_inactivate_session", { sessionId });
  }

  private async invoke(
    command: ConversationCommand,
    args: Readonly<Record<string, unknown>>,
  ): Promise<ConversationSnapshot> {
    const request = makeRequest(this.currentGeneration);
    let raw: unknown;
    try {
      raw = await this.transport.invoke(command, { request, ...args });
    } catch (error) {
      throw normalizeFailure(error);
    }
    const envelope = record(raw, "Conversation envelope");
    if (
      envelope.protocolVersion !== PROTOCOL_VERSION ||
      envelope.requestId !== request.requestId ||
      envelope.correlationId !== request.correlationId
    ) {
      throw boundary(
        "correlationMismatch",
        "Conversation response identity changed.",
      );
    }
    const generation = generationValue(envelope.generation);
    if (generation < this.currentGeneration) {
      throw boundary("staleGeneration", "Conversation response was stale.");
    }
    const snapshot = parseSnapshot(envelope.payload);
    if (snapshot.generation !== generation) {
      throw boundary("staleGeneration", "Conversation generations disagreed.");
    }
    this.currentGeneration = generation;
    return snapshot;
  }
}

export function createConversationAdapter(
  transport: ConversationTransport,
): ConversationAdapter {
  return new NativeConversationAdapter(transport);
}

const nativeConversation = createConversationAdapter({
  invoke(command, args) {
    return invokeNative(command, args);
  },
});

export const readConversationSnapshot = () => nativeConversation.read();
export const previewConversationAttachment = (
  input: ConversationAttachmentPreviewInput,
) => nativeConversation.previewAttachment(input);
export const requestConversationBranch = (input: ConversationBranchInput) =>
  nativeConversation.requestBranch(input);
export const answerConversationBranchApproval = (
  input: ConversationBranchApprovalInput,
) => nativeConversation.answerBranchApproval(input);
export const beginPendingConversation = (projectId: ProjectId) =>
  nativeConversation.begin(projectId);
export const cancelPendingConversation = () => nativeConversation.cancel();
export const updateConversationDraft = (input: ConversationDraftInput) =>
  nativeConversation.updateDraft(input);
export const submitConversation = (input: ConversationSubmitInput) =>
  nativeConversation.submit(input);
export const cancelConversationAttempt = (attemptId: AttemptId) =>
  nativeConversation.cancelAttempt(attemptId);
export const retryConversationAttempt = (input: ConversationRetryInput) =>
  nativeConversation.retryAttempt(input);
export const activateConversationSession = (sessionId: SessionId) =>
  nativeConversation.activate(sessionId);
export const activateConversationProject = (projectId: ProjectId) =>
  nativeConversation.activateProject(projectId);
export const addConversationProject = (pickerGrantId: PickerGrantId) =>
  nativeConversation.addProject(pickerGrantId);
export const relocateConversationProject = (
  projectId: ProjectId,
  pickerGrantId: PickerGrantId,
) => nativeConversation.relocateProject(projectId, pickerGrantId);
export const renameConversationProject = (
  projectId: ProjectId,
  displayName: string,
) => nativeConversation.renameProject(projectId, displayName);
export const copyConversationProjectPath = (projectId: ProjectId) =>
  nativeConversation.copyProjectPath(projectId);
export const revealConversationProject = (projectId: ProjectId) =>
  nativeConversation.revealProject(projectId);
export const reorderConversationProjects = (
  orderedProjectIds: readonly ProjectId[],
) => nativeConversation.reorderProjects(orderedProjectIds);
export const inactivateConversationProject = (projectId: ProjectId) =>
  nativeConversation.inactivateProject(projectId);
export const inactivateConversationSession = (sessionId: SessionId) =>
  nativeConversation.inactivateSession(sessionId);

function parseSnapshot(raw: unknown): ConversationSnapshot {
  const value = record(raw, "Conversation snapshot");
  if (
    value.protocolVersion !== PROTOCOL_VERSION ||
    !isAcceptedQaAwareAuthority(value.authority, "rust-core")
  ) {
    throw boundary("invalidPayload", "Conversation authority is invalid.");
  }
  const projects = array(value.projects, 250, "projects").map((entry) => {
    const project = record(entry, "Project");
    const pathState = text(project.pathState, "path state");
    if (
      pathState !== "found" &&
      pathState !== "missing" &&
      pathState !== "relocated"
    ) {
      throw boundary("invalidPayload", "Project path state is invalid.");
    }
    return {
      projectId: identifier(project.projectId, "project ID") as ProjectId,
      displayName: text(project.displayName, "project name"),
      pathState: pathState as ConversationProjectSnapshot["pathState"],
      position: integer(project.position, "project position"),
      gitVersioned: booleanValue(project.gitVersioned, "Git repository state"),
    };
  });
  const sessions = array(value.sessions, 250, "sessions").map(
    parseSessionSummary,
  );
  return {
    protocolVersion: PROTOCOL_VERSION,
    generation: generationValue(value.generation),
    authority: value.authority,
    workspaceId: nullableIdentifier(
      value.workspaceId,
      "workspace ID",
    ) as WorkspaceId | null,
    workspaceName: nullableText(value.workspaceName, "workspace name"),
    activeProjectId: nullableIdentifier(
      value.activeProjectId,
      "project ID",
    ) as ProjectId | null,
    activeSessionId: nullableIdentifier(
      value.activeSessionId,
      "session ID",
    ) as SessionId | null,
    pending: value.pending === null ? null : parsePending(value.pending),
    draft: parseDraft(value.draft),
    projects,
    sessions,
    activeConversation:
      value.activeConversation === null
        ? null
        : parseActiveConversation(value.activeConversation),
    models: array(value.models, 250, "models").map(parseModel),
    branchControl:
      value.branchControl === null
        ? null
        : parseBranchControl(value.branchControl),
  };
}

function parseBranchControl(raw: unknown): ConversationBranchControlSnapshot {
  const value = record(raw, "Git Branch control");
  const currentBranch = nullableText(value.currentBranch, "current Git branch");
  const branches = array(value.branches, 4_096, "Git branches").map(
    (rawBranch) => {
      const branch = record(rawBranch, "Git branch");
      const targetOid = text(branch.targetOid, "Git branch object identity");
      if (
        ![40, 64].includes(targetOid.length) ||
        !/^[a-fA-F0-9]+$/.test(targetOid)
      ) {
        throw boundary(
          "invalidPayload",
          "Git branch object identity is invalid.",
        );
      }
      return {
        name: boundedText(branch.name, "Git branch", 255),
        targetOid,
        selected: booleanValue(branch.selected, "Git branch selection"),
      };
    },
  );
  const selected = branches.filter((branch) => branch.selected);
  if (
    selected.length > 1 ||
    (currentBranch === null) !== (selected.length === 0) ||
    (currentBranch !== null && selected[0]?.name !== currentBranch)
  ) {
    throw boundary("invalidPayload", "Git branch selection is inconsistent.");
  }
  const operationStatus = nullableText(
    value.operationStatus,
    "Git branch operation status",
  );
  if (
    operationStatus !== null &&
    !new Set(["pending", "switched", "created", "blocked", "denied"]).has(
      operationStatus,
    )
  ) {
    throw boundary("invalidPayload", "Git branch operation status is invalid.");
  }
  return {
    currentBranch,
    branches,
    pendingApprovalId: nullableIdentifier(
      value.pendingApprovalId,
      "Git branch approval ID",
    ),
    operationStatus:
      operationStatus as ConversationBranchControlSnapshot["operationStatus"],
    operationMessage:
      value.operationMessage === null
        ? null
        : boundedText(
            value.operationMessage,
            "Git branch operation message",
            4_096,
          ),
  };
}

function parseSessionSummary(raw: unknown): ConversationSessionSummarySnapshot {
  const value = record(raw, "Chat summary");
  return {
    sessionId: identifier(value.sessionId, "session ID") as SessionId,
    projectId: identifier(value.projectId, "project ID") as ProjectId,
    title: text(value.title, "Chat title"),
    updatedAtMs: nonnegative(value.updatedAtMs, "Chat timestamp"),
  };
}

function parsePending(
  raw: unknown,
): NonNullable<ConversationSnapshot["pending"]> {
  const value = record(raw, "pending Chat");
  return {
    sessionId: identifier(value.sessionId, "session ID") as SessionId,
    projectId: identifier(value.projectId, "project ID") as ProjectId,
    title: text(value.title, "pending title"),
    attachments: array(value.attachments, 64, "attachments").map(
      parseAttachment,
    ),
  };
}

function parseDraft(raw: unknown): ConversationSnapshot["draft"] {
  const value = record(raw, "composer draft");
  const mode = text(value.mode, "composer mode");
  if (!new Set(["chat", "files", "browser", "terminal"]).has(mode)) {
    throw boundary("invalidPayload", "Composer mode is invalid.");
  }
  const reasoningMode = nullableText(value.reasoningMode, "reasoning mode");
  if (
    reasoningMode !== null &&
    !new Set(["off", "low", "medium", "high"]).has(reasoningMode)
  ) {
    throw boundary("invalidPayload", "Reasoning mode is invalid.");
  }
  const attachments = array(value.attachments, 64, "draft attachments").map(
    parseAttachment,
  );
  const nextAttachmentReference = positiveInteger(
    value.nextAttachmentReference,
    "next attachment reference",
  );
  const originalReferences = new Set<number>();
  for (const { originalReference } of attachments) {
    if (
      originalReference >= nextAttachmentReference ||
      originalReferences.has(originalReference)
    ) {
      throw boundary(
        "invalidPayload",
        "Draft attachment references are invalid.",
      );
    }
    originalReferences.add(originalReference);
  }
  return {
    prompt: textAllowEmpty(value.prompt, "draft prompt"),
    attachments,
    nextAttachmentReference,
    providerId: nullableIdentifier(value.providerId, "provider ID"),
    modelId:
      value.modelId === null
        ? null
        : identifier(value.modelId, "model ID", true),
    reasoningMode:
      reasoningMode as ConversationSnapshot["draft"]["reasoningMode"],
    mode: mode as ConversationSnapshot["draft"]["mode"],
    replyTargetId: nullableIdentifier(value.replyTargetId, "Reply target ID"),
  };
}

function parseActiveConversation(raw: unknown): ConversationSessionSnapshot {
  const value = record(raw, "active Conversation");
  return {
    sessionId: identifier(value.sessionId, "session ID") as SessionId,
    title: nullableText(value.title, "Chat title"),
    turns: array(value.turns, 4_096, "turns").map((rawTurn) => {
      const turn = record(rawTurn, "turn");
      return {
        turnId: identifier(turn.turnId, "turn ID") as TurnId,
        prompt: nullableText(turn.prompt, "turn prompt"),
        attachments: array(turn.attachments, 64, "attachments").map(
          parseAttachment,
        ),
        artifactContext:
          turn.artifactContext === null
            ? null
            : parseArtifactContext(turn.artifactContext),
        mcpProvenance:
          turn.mcpProvenance === null
            ? null
            : parseMcpProvenance(turn.mcpProvenance),
        submittedAtMs: nonnegative(turn.submittedAtMs, "turn timestamp"),
      };
    }),
    attempts: array(value.attempts, 4_096, "attempts").map(parseAttempt),
    activeAttemptId: nullableIdentifier(
      value.activeAttemptId,
      "attempt ID",
    ) as AttemptId | null,
  };
}

function parseMcpProvenance(raw: unknown): ConversationMcpProvenanceSnapshot {
  const value = record(raw, "MCP provenance");
  const snapshotId = text(value.snapshotId, "MCP snapshot ID");
  if (!/^mcp-turn:[a-f0-9]{64}$/u.test(snapshotId)) {
    throw boundary("invalidPayload", "MCP snapshot ID is invalid.");
  }
  const tools = array(value.tools, 128, "MCP provenance tools").map(
    (rawTool) => {
      const tool = record(rawTool, "MCP tool provenance");
      const sourceKind = oneOf(
        tool.sourceKind,
        ["user", "plugin"] as const,
        "MCP source kind",
      );
      const sourceId = nullableIdentifier(tool.sourceId, "MCP source ID");
      if (
        (sourceKind === "user" && sourceId !== null) ||
        (sourceKind === "plugin" && sourceId === null)
      ) {
        throw boundary("invalidPayload", "MCP source identity is invalid.");
      }
      return {
        serverId: identifier(tool.serverId, "MCP server ID"),
        sourceKind,
        sourceId,
        toolName: text(tool.toolName, "MCP tool name"),
      };
    },
  );
  const serverCount = nonnegative(value.serverCount, "MCP server count");
  const toolCount = nonnegative(value.toolCount, "MCP tool count");
  const omittedToolCount = nonnegative(
    value.omittedToolCount,
    "MCP omitted tool count",
  );
  const truncated = booleanValue(value.truncated, "MCP provenance truncation");
  if (
    toolCount !== tools.length ||
    serverCount !== new Set(tools.map((tool) => tool.serverId)).size ||
    truncated !== omittedToolCount > 0
  ) {
    throw boundary("invalidPayload", "MCP provenance counts are invalid.");
  }
  return {
    snapshotId,
    serverCount,
    toolCount,
    omittedToolCount,
    truncated,
    tools,
  };
}

export function parseArtifactContext(
  raw: unknown,
): ConversationArtifactContextSnapshot {
  const value = record(raw, "Artifact Reply context");
  const providerType = oneOf(
    value.providerType,
    ["file", "folder", "browser", "terminal"] as const,
    "Artifact context provider",
  );
  const payloadKind = oneOf(
    value.payloadKind,
    ["file", "folder", "browser", "terminal"] as const,
    "Artifact context payload",
  );
  if (providerType !== payloadKind) {
    throw boundary(
      "invalidPayload",
      "Artifact context provider and payload changed.",
    );
  }
  const resource = record(
    value.capturedResourceVersion,
    "Artifact resource version",
  );
  const sha256 = text(resource.sha256, "Artifact resource digest");
  if (!/^sha256:[a-f0-9]{64}$/.test(sha256)) {
    throw boundary("invalidPayload", "Artifact resource digest is invalid.");
  }
  const segments = array(value.segments, 32, "Artifact context segments").map(
    (rawSegment) => {
      const segment = record(rawSegment, "Artifact context segment");
      const textValue = textAllowEmpty(segment.text, "Artifact context text");
      const originalBytes = nonnegative(
        segment.originalBytes,
        "Artifact context original bytes",
      );
      const omittedBytes = nonnegative(
        segment.omittedBytes,
        "Artifact context omitted bytes",
      );
      if (
        new TextEncoder().encode(textValue).length + omittedBytes !==
        originalBytes
      ) {
        throw boundary(
          "invalidPayload",
          "Artifact context segment budget changed.",
        );
      }
      return {
        priority: oneOf(
          segment.priority,
          ["selection", "visibleOrCurrent", "recent", "metadata"] as const,
          "Artifact context priority",
        ),
        source: identifier(segment.source, "Artifact context source"),
        text: textValue,
        originalBytes,
        omittedBytes,
      };
    },
  );
  const maximumBytes = positiveInteger(
    value.maximumBytes,
    "Artifact context maximum bytes",
  );
  const usedBytes = nonnegative(value.usedBytes, "Artifact context used bytes");
  const omittedBytes = nonnegative(
    value.omittedBytes,
    "Artifact context omitted bytes",
  );
  const omittedSegments = nonnegative(
    value.omittedSegments,
    "Artifact context omitted segments",
  );
  const computedUsed = segments.reduce(
    (total, segment) => total + new TextEncoder().encode(segment.text).length,
    0,
  );
  const computedOmitted = segments.reduce(
    (total, segment) => total + segment.omittedBytes,
    0,
  );
  const computedOmittedSegments = segments.filter(
    (segment) => segment.text.length === 0 && segment.originalBytes > 0,
  ).length;
  const truncated = booleanValue(value.truncated, "Artifact truncation");
  if (
    maximumBytes > 4 * 1_024 * 1_024 ||
    usedBytes > maximumBytes ||
    usedBytes !== computedUsed ||
    omittedBytes !== computedOmitted ||
    omittedSegments !== computedOmittedSegments ||
    truncated !== omittedBytes > 0
  ) {
    throw boundary("invalidPayload", "Artifact context budget is invalid.");
  }
  const stableReference = largeBoundedText(
    value.stableReference,
    "Artifact stable reference",
    512,
  );
  if (!/^[A-Za-z0-9_.:@-]+$/.test(stableReference)) {
    throw boundary(
      "invalidIdentifier",
      "Artifact stable reference is invalid.",
    );
  }
  return {
    snapshotId: identifier(value.snapshotId, "Artifact context snapshot ID"),
    stableReference,
    artifactId: identifier(value.artifactId, "Artifact ID") as ArtifactId,
    projectId: identifier(value.projectId, "Project ID") as ProjectId,
    sessionId: identifier(value.sessionId, "session ID") as SessionId,
    providerType,
    providerVersion: positiveInteger(
      value.providerVersion,
      "Artifact provider version",
    ),
    artifactRecordRevision: positiveInteger(
      value.artifactRecordRevision,
      "Artifact record revision",
    ),
    capturedResourceVersion: {
      sequence: positiveInteger(
        resource.sequence,
        "Artifact resource sequence",
      ),
      sha256,
      observedAtMs: positiveInteger(
        resource.observedAtMs,
        "Artifact resource timestamp",
      ),
    },
    payloadKind,
    segments,
    maximumBytes,
    usedBytes,
    omittedBytes,
    omittedSegments,
    truncated,
    unsaved: booleanValue(value.unsaved, "Artifact unsaved state"),
    redactions: array(value.redactions, 64, "Artifact redactions").map(
      (redaction) => identifier(redaction, "Artifact redaction"),
    ),
    capabilities: array(value.capabilities, 128, "Artifact capabilities").map(
      (rawCapability) => {
        const capability = record(rawCapability, "Artifact capability");
        return {
          capabilityId: identifier(
            capability.capabilityId,
            "Artifact capability ID",
          ),
          access: oneOf(
            capability.access,
            ["readable", "approvalRequired", "denied", "unknown"] as const,
            "Artifact capability access",
          ),
          reasonCode: nullableIdentifier(
            capability.reasonCode,
            "Artifact capability reason",
          ),
        };
      },
    ),
    capturedAtMs: positiveInteger(
      value.capturedAtMs,
      "Artifact capture timestamp",
    ),
  };
}

function parseAttachment(raw: unknown): ConversationAttachmentSnapshot {
  const value = record(raw, "attachment");
  return {
    attachmentId: identifier(
      value.attachmentId,
      "attachment ID",
    ) as AttachmentId,
    displayName: text(value.displayName, "attachment name"),
    mediaType: text(value.mediaType, "attachment media type"),
    byteLength: nonnegative(value.byteLength, "attachment length"),
    stableReference: identifier(value.stableReference, "attachment reference"),
    originalReference: positiveInteger(
      value.originalReference,
      "original attachment reference",
    ),
  };
}

function parseAttachmentPreview(
  raw: unknown,
  input: ConversationAttachmentPreviewInput,
): ConversationAttachmentPreviewSnapshot {
  const value = record(raw, "attachment preview");
  const attachmentId = identifier(
    value.attachmentId,
    "attachment preview ID",
  ) as AttachmentId;
  const mediaType = text(value.mediaType, "attachment preview media type");
  const dataUrl = largeBoundedText(
    value.dataUrl,
    "attachment preview data URL",
    2_800_000,
  );
  if (
    attachmentId !== input.attachmentId ||
    !new Set(["image/png", "image/jpeg", "image/gif", "image/webp"]).has(
      mediaType,
    ) ||
    !dataUrl.startsWith(`data:${mediaType};base64,`) ||
    !/^[A-Za-z0-9+/]*={0,2}$/.test(dataUrl.slice(dataUrl.indexOf(",") + 1))
  ) {
    throw boundary("invalidPayload", "Attachment preview is invalid.");
  }
  return { attachmentId, mediaType, dataUrl };
}

function parseAttempt(raw: unknown): ConversationAttemptSnapshot {
  const value = record(raw, "attempt");
  const status = text(value.status, "attempt status");
  if (
    !new Set([
      "starting",
      "working",
      "cancelling",
      "completed",
      "failed",
      "interrupted",
      "cancelled",
    ]).has(status)
  ) {
    throw boundary("invalidPayload", "Attempt status is invalid.");
  }
  const runtimeKind = text(value.runtimeKind, "runtime kind");
  if (runtimeKind !== "open-code" && runtimeKind !== "pi") {
    throw boundary("invalidPayload", "Runtime kind is invalid.");
  }
  return {
    attemptId: identifier(value.attemptId, "attempt ID") as AttemptId,
    turnId: identifier(value.turnId, "turn ID") as TurnId,
    status: status as ConversationAttemptSnapshot["status"],
    assistantMarkdown: textAllowEmpty(
      value.assistantMarkdown,
      "assistant response",
    ),
    activities: array(value.activities, 4_096, "activities").map(
      (rawActivity) => {
        const activity = record(rawActivity, "activity");
        return {
          sequence: nonnegative(activity.sequence, "activity sequence"),
          kind: text(activity.kind, "activity kind"),
          label: text(activity.label, "activity label"),
          detail: nullableText(activity.detail, "activity detail"),
        };
      },
    ),
    runtimeId: identifier(value.runtimeId, "runtime ID") as RuntimeId,
    runtimeKind,
    environmentId: identifier(
      value.environmentId,
      "environment ID",
    ) as EnvironmentId,
    providerId: identifier(value.providerId, "provider ID"),
    modelId: identifier(value.modelId, "model ID", true),
    adapterId: identifier(value.adapterId, "adapter ID"),
    inputTokens: nonnegative(value.inputTokens, "input tokens"),
    outputTokens: nonnegative(value.outputTokens, "output tokens"),
    durationMs:
      value.durationMs === null
        ? null
        : nonnegative(value.durationMs, "attempt duration"),
  };
}

function parseModel(raw: unknown): ConversationModelSnapshot {
  const value = record(raw, "model");
  return {
    providerId: identifier(value.providerId, "provider ID"),
    providerName: text(value.providerName, "provider name"),
    modelId: identifier(value.modelId, "model ID", true),
    selected: booleanValue(value.selected, "selected state"),
    available: booleanValue(value.available, "model availability"),
    supportsVision: booleanValue(value.supportsVision, "Vision support"),
    supportsTools: booleanValue(value.supportsTools, "Tools support"),
    supportsReasoning: booleanValue(
      value.supportsReasoning,
      "Reasoning support",
    ),
    supportsAudio: booleanValue(value.supportsAudio, "Audio support"),
    contextTokens: nonnegative(value.contextTokens, "context tokens"),
  };
}

function makeRequest(expectedGeneration: StateGeneration): SnapshotRequest {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw boundary(
      "unavailable",
      "Secure Conversation request identity is unavailable.",
    );
  }
  return {
    protocolVersion: PROTOCOL_VERSION,
    requestId: globalThis.crypto.randomUUID() as RequestId,
    correlationId: globalThis.crypto.randomUUID() as CorrelationId,
    expectedGeneration,
  };
}

function identifier(value: unknown, label: string, slash = false): string {
  const candidate = text(value, label);
  const valid = [...candidate].every(
    (character) =>
      /[A-Za-z0-9_.:@-]/.test(character) || (slash && character === "/"),
  );
  if (
    !valid ||
    new TextEncoder().encode(candidate).length > MAX_IDENTIFIER_BYTES
  ) {
    throw boundary("invalidIdentifier", `${label} is invalid.`);
  }
  return candidate;
}

function nullableIdentifier(value: unknown, label: string): string | null {
  return value === null ? null : identifier(value, label);
}

function generationValue(value: unknown): StateGeneration {
  return nonnegative(value, "generation") as StateGeneration;
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

function record(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value as Record<string, unknown>;
}

function oneOf<const Values extends readonly string[]>(
  value: unknown,
  values: Values,
  label: string,
): Values[number] {
  if (typeof value !== "string" || !values.includes(value)) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value as Values[number];
}

function text(value: unknown, label: string): string {
  const candidate = textAllowEmpty(value, label);
  if (candidate.trim().length === 0)
    throw boundary("invalidPayload", `${label} is empty.`);
  return candidate;
}

function boundedText(value: unknown, label: string, maximum: number): string {
  const candidate = text(value, label);
  if (
    candidate.length > maximum ||
    [...candidate].some((character) => {
      const code = character.charCodeAt(0);
      return code <= 31 || code === 127;
    })
  ) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return candidate;
}

function largeBoundedText(
  value: unknown,
  label: string,
  maximum: number,
): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > maximum ||
    value.includes("\0")
  ) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value;
}

function textAllowEmpty(value: unknown, label: string): string {
  if (
    typeof value !== "string" ||
    value.includes("\0") ||
    value.length > 1_048_576
  ) {
    throw boundary("invalidPayload", `${label} is invalid.`);
  }
  return value;
}

function nullableText(value: unknown, label: string): string | null {
  return value === null ? null : textAllowEmpty(value, label);
}

function integer(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value))
    throw boundary("invalidPayload", `${label} is invalid.`);
  return value as number;
}

function nonnegative(value: unknown, label: string): number {
  const parsed = integer(value, label);
  if (parsed < 0) throw boundary("invalidPayload", `${label} is invalid.`);
  return parsed;
}

function positiveInteger(value: unknown, label: string): number {
  const parsed = integer(value, label);
  if (parsed < 1) throw boundary("invalidPayload", `${label} is invalid.`);
  return parsed;
}

function booleanValue(value: unknown, label: string): boolean {
  if (typeof value !== "boolean")
    throw boundary("invalidPayload", `${label} is invalid.`);
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
  return boundary("unavailable", "Conversation service is unavailable.", true);
}

function boundary(
  code: string,
  message: string,
  retryable = false,
): ProtocolBoundaryError {
  return new ProtocolBoundaryError(code as never, message, retryable);
}
