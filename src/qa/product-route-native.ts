import {
  QA_RECENT_WORKSPACES,
  QA_WORKSPACE_RECOVERY_NOTICE,
} from "./workspace-fixture";

type Request = {
  readonly protocolVersion: number;
  readonly requestId: string;
  readonly correlationId: string;
  readonly expectedGeneration: number;
};

type Args = Readonly<Record<string, unknown>>;

let providerGeneration = 1;
let configurationGeneration = 1;
let onboardingCompletedAtMs: number | null = null;
let provider: Record<string, unknown> | null = null;
let workspaceGeneration = 51;
let conversationGeneration = 51;
let activatedWorkspaceId: string | null = null;
let activatedWorkspaceName: string | null = null;
let activeRecoveryNotice: typeof QA_WORKSPACE_RECOVERY_NOTICE | null = null;

export function invokeQaProductRoute(
  command: string,
  args: Args,
): Promise<unknown> | null {
  if (command.startsWith("provider_")) {
    return Promise.resolve(providerCommand(command, args));
  }
  if (command === "workspace_start_snapshot") {
    return Promise.resolve(
      envelope(args, workspaceGeneration, workspaceStartSnapshot()),
    );
  }
  if (command === "workspace_start_open_recent") {
    const input = record(args.input);
    const workspaceId = text(input.workspaceId);
    const recent = QA_RECENT_WORKSPACES.find(({ id }) => id === workspaceId);
    const payload = activateWorkspace(workspaceId, recent);
    return Promise.resolve(envelope(args, workspaceGeneration, payload));
  }
  if (command === "workspace_start_open_archive") {
    const input = record(args.input);
    if (text(input.pickerGrantId) !== "picker-grant:legacy-workspace") {
      throw new Error("The QA Workspace archive grant is invalid.");
    }
    const recent = QA_RECENT_WORKSPACES.find(
      ({ id }) => id === QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
    );
    const payload = activateWorkspace(
      QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
      recent,
    );
    return Promise.resolve(envelope(args, workspaceGeneration, payload));
  }
  if (command === "workspace_recovery_acknowledge") {
    const request = record(args.request);
    const input = record(args.input);
    if (
      activeRecoveryNotice === null ||
      integer(request.expectedGeneration) !== workspaceGeneration ||
      integer(input.expectedGeneration) !== workspaceGeneration ||
      text(input.recoveryId) !== activeRecoveryNotice.recoveryId ||
      text(input.workspaceId) !== activeRecoveryNotice.workspaceId ||
      integer(input.workingGeneration) !==
        activeRecoveryNotice.workingGeneration ||
      integer(input.archiveGeneration) !==
        activeRecoveryNotice.archiveGeneration
    ) {
      throw new Error("The QA Workspace recovery identity is stale.");
    }
    workspaceGeneration += 1;
    activeRecoveryNotice = null;
    return Promise.resolve(
      envelope(args, workspaceGeneration, workspaceStartSnapshot()),
    );
  }
  if (command === "conversation_snapshot") {
    return Promise.resolve(
      envelope(args, conversationGeneration, conversationSnapshot()),
    );
  }
  if (command === "platform_pick") {
    const picker = record(args.picker);
    if (picker.purpose === "openWorkspaceArchive") {
      return Promise.resolve(
        envelope(args, 0, {
          type: "selected",
          contractVersion: 1,
          requestId: text(picker.requestId),
          grants: [
            {
              grantId: "picker-grant:legacy-workspace",
              objectKind: "file",
              displayName: "Legacy UI.c4workspace",
            },
          ],
        }),
      );
    }
    return Promise.resolve(
      envelope(args, 0, {
        type: "cancelled",
        contractVersion: 1,
        requestId: text(picker.requestId),
      }),
    );
  }
  return null;
}

function activateWorkspace(
  workspaceId: string,
  recent: (typeof QA_RECENT_WORKSPACES)[number] | undefined,
) {
  workspaceGeneration += 1;
  conversationGeneration += 1;
  activatedWorkspaceId = workspaceId;
  activatedWorkspaceName = recent?.name ?? "QA Workspace";
  activeRecoveryNotice =
    recent?.isMissing === true ? QA_WORKSPACE_RECOVERY_NOTICE : null;
  return {
    authority: "rust-workspace-service",
    workspaceId,
    workspaceName: activatedWorkspaceName,
    recovered: activeRecoveryNotice !== null,
    recoveryNotice: activeRecoveryNotice,
  };
}

function providerCommand(command: string, args: Args): unknown {
  const input = record(args.input);
  switch (command) {
    case "provider_snapshot":
    case "provider_accept_session_credentials":
      break;
    case "provider_save_profile": {
      providerGeneration += 1;
      const authentication = record(input.authentication);
      const existingProfile =
        provider === null ? null : record(provider.profile);
      const submittedSecret =
        typeof input.secret === "string" && input.secret.length > 0;
      provider = {
        profile: {
          schemaVersion: 1,
          providerId: text(input.providerId),
          kind: text(input.kind),
          displayName: text(input.displayName),
          endpoint: input.endpoint,
          authentication,
          credentialReference:
            authentication.type === "none"
              ? null
              : submittedSecret
                ? "credential-qa-provider"
                : (existingProfile?.credentialReference ?? null),
          headers: input.headers,
          enabled: input.enabled,
        },
        testStatus: { state: "untested" },
        connectionEvidence: null,
        models: {},
        disabledModelIds: [],
        selectedModelId: null,
        generation: providerGeneration,
      };
      break;
    }
    case "provider_test_connection": {
      if (provider === null) throw new Error("QA provider is unavailable");
      providerGeneration += 1;
      provider = {
        ...provider,
        testStatus: { state: "succeeded", checkedAtMs: 1_784_390_400_000 },
        models: {
          "openai/gpt-5": providerModel(),
        },
        selectedModelId: "openai/gpt-5",
        generation: providerGeneration,
      };
      break;
    }
    case "provider_select_model":
      if (provider === null) throw new Error("QA provider is unavailable");
      providerGeneration += 1;
      provider = {
        ...provider,
        selectedModelId: text(input.modelId),
        generation: providerGeneration,
      };
      break;
    case "provider_complete_onboarding":
      providerGeneration += 1;
      configurationGeneration += 1;
      onboardingCompletedAtMs = 1_784_390_400_000;
      if (provider !== null) {
        provider = { ...provider, generation: providerGeneration };
      }
      break;
    case "provider_set_models_enabled":
      providerGeneration += 1;
      if (provider !== null) {
        provider = { ...provider, generation: providerGeneration };
      }
      break;
    case "provider_delete_profile":
      providerGeneration += 1;
      provider = null;
      onboardingCompletedAtMs = null;
      break;
    case "provider_answer_approval":
      break;
    default:
      throw new Error(`Unsupported QA Provider command: ${command}`);
  }
  return envelope(args, providerGeneration, providerSnapshot());
}

function providerSnapshot() {
  const activeProviderId =
    provider === null ? null : text(record(provider.profile).providerId);
  return {
    authority: "rust-provider-service",
    coordinatorGeneration: providerGeneration,
    configurationGeneration,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    onboardingCompleted: onboardingCompletedAtMs !== null && provider !== null,
    providers: {
      generation: providerGeneration,
      onboardingCompletedAtMs,
      providers: provider === null ? [] : [provider],
    },
    modelRoute:
      onboardingCompletedAtMs === null || activeProviderId === null
        ? null
        : `${activeProviderId}::openai/gpt-5`,
    defaultRuntime: onboardingCompletedAtMs === null ? null : "opencode",
    defaultEnvironment: onboardingCompletedAtMs === null ? null : "local",
    pendingApproval: null,
  };
}

function providerModel() {
  return {
    modelId: "openai/gpt-5",
    displayName: "GPT-5",
    recommendationRank: 0,
    availability: "available",
    checkedAtMs: 1_784_390_400_000,
    capabilities: {
      lifecycle: "active",
      features: {
        tools: { state: "supported" },
        vision: { state: "supported" },
      },
      numericLimits: {
        "context-tokens": { maximum: 128_000 },
        "output-tokens": { maximum: 16_384 },
      },
    },
    providerDeclaration: {
      schemaVersion: 1,
      providerModelId: "openai/gpt-5",
      modelRevision: "qa-2026-07-23",
      lifecycle: "active",
      features: {},
      numericLimits: {},
      rawCatalogSha256: `sha256:${"1".repeat(64)}`,
      declaredAtMs: 1_784_390_400_000,
      expiresAtMs: 1_784_476_800_000,
    },
  };
}

function conversationSnapshot() {
  if (activatedWorkspaceId === null) {
    return {
      protocolVersion: 1,
      generation: conversationGeneration,
      authority: "rust-core",
      workspaceId: null,
      workspaceName: null,
      activeProjectId: null,
      activeSessionId: null,
      pending: null,
      draft: {
        prompt: "",
        attachments: [],
        nextAttachmentReference: 1,
        providerId: null,
        modelId: null,
        reasoningMode: null,
        mode: "chat",
        replyTargetId: null,
      },
      projects: [],
      sessions: [],
      activeConversation: null,
      models: [],
      branchControl: null,
    };
  }
  return {
    protocolVersion: 1,
    generation: conversationGeneration,
    authority: "rust-core",
    workspaceId: activatedWorkspaceId,
    workspaceName: activatedWorkspaceName,
    activeProjectId: "project-qa-0001",
    activeSessionId: "chat-qa-0001",
    pending: null,
    draft: {
      prompt: "",
      attachments: [],
      nextAttachmentReference: 1,
      providerId: null,
      modelId: null,
      reasoningMode: null,
      mode: "chat",
      replyTargetId: null,
    },
    projects: [
      {
        projectId: "project-qa-0001",
        displayName: "C4OS QA",
        pathState: "found",
        position: 0,
        gitVersioned: true,
      },
    ],
    sessions: [
      {
        sessionId: "chat-qa-0001",
        projectId: "project-qa-0001",
        title: "QA Chat",
        updatedAtMs: 1_784_390_400_000,
      },
    ],
    activeConversation: {
      sessionId: "chat-qa-0001",
      title: "QA Chat",
      turns: [],
      attempts: [],
      activeAttemptId: null,
    },
    models: [],
    branchControl: null,
  };
}

function workspaceStartSnapshot() {
  return {
    protocolVersion: 1,
    generation: workspaceGeneration,
    authority: "rust-core",
    recents: QA_RECENT_WORKSPACES.map((recent, index) => ({
      workspaceId: recent.id,
      displayName: recent.name,
      lastOpenedAt: 1_784_390_400 - index * 86_400,
      isMissing: recent.isMissing ?? false,
    })),
    activeRecoveryNotice,
  };
}

function envelope(args: Args, generation: number, payload: unknown) {
  const request = record(args.request) as unknown as Request;
  return {
    protocolVersion: 1,
    requestId: request.requestId,
    correlationId: request.correlationId,
    generation,
    payload,
  };
}

function record(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function text(value: unknown): string {
  if (typeof value !== "string") throw new Error("Invalid QA fixture input");
  return value;
}

function integer(value: unknown): number {
  if (!Number.isSafeInteger(value)) {
    throw new Error("Invalid QA fixture generation");
  }
  return value as number;
}
