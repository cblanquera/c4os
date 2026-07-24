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

interface QaProductRouteState {
  providerGeneration: number;
  configurationGeneration: number;
  onboardingCompletedAtMs: number | null;
  provider: Record<string, unknown> | null;
  workspaceGeneration: number;
  conversationGeneration: number;
  activatedWorkspaceId: string | null;
  activatedWorkspaceName: string | null;
  activeRecoveryNotice: typeof QA_WORKSPACE_RECOVERY_NOTICE | null;
  draftMode: "chat" | "files" | "browser" | "terminal";
  draftPrompt: string;
  draftProviderId: string | null;
  draftModelId: string | null;
  draftReasoningMode: "off" | "low" | "medium" | "high" | null;
}

export interface QaProductRouteFixtureSnapshot {
  readonly authority: "qa-fixture-only";
  readonly providerGeneration: number;
  readonly configurationGeneration: number;
  readonly workspaceGeneration: number;
  readonly conversationGeneration: number;
  readonly onboardingCompleted: boolean;
  readonly providerConfigured: boolean;
  readonly activatedWorkspaceId: string | null;
  readonly activeRecoveryId: string | null;
}

export interface QaProductRouteAdapter {
  invoke(command: string, args: Args): Promise<unknown>;
  reset(): QaProductRouteFixtureSnapshot;
  snapshot(): QaProductRouteFixtureSnapshot;
}

function initialQaProductRouteState(): QaProductRouteState {
  return {
    providerGeneration: 1,
    configurationGeneration: 1,
    onboardingCompletedAtMs: null,
    provider: null,
    workspaceGeneration: 51,
    conversationGeneration: 51,
    activatedWorkspaceId: "workspace-qa-0001",
    activatedWorkspaceName: "C4OS QA Workspace",
    activeRecoveryNotice: null,
    draftMode: "chat",
    draftPrompt: "Preserved QA composer draft",
    draftProviderId: "openrouter",
    draftModelId: "moonshotai/kimi-k2",
    draftReasoningMode: null,
  };
}

function productRouteFixtureSnapshot(
  state: QaProductRouteState,
): QaProductRouteFixtureSnapshot {
  return {
    authority: "qa-fixture-only",
    providerGeneration: state.providerGeneration,
    configurationGeneration: state.configurationGeneration,
    workspaceGeneration: state.workspaceGeneration,
    conversationGeneration: state.conversationGeneration,
    onboardingCompleted: state.onboardingCompletedAtMs !== null,
    providerConfigured: state.provider !== null,
    activatedWorkspaceId: state.activatedWorkspaceId,
    activeRecoveryId: state.activeRecoveryNotice?.recoveryId ?? null,
  };
}

export function createQaProductRouteAdapter(): QaProductRouteAdapter {
  let state = initialQaProductRouteState();
  return {
    invoke(command, args) {
      return invokeQaProductRouteWithState(state, command, args);
    },
    reset() {
      state = initialQaProductRouteState();
      return productRouteFixtureSnapshot(state);
    },
    snapshot() {
      return productRouteFixtureSnapshot(state);
    },
  };
}

const defaultQaProductRouteAdapter = createQaProductRouteAdapter();

export function invokeQaProductRoute(
  command: string,
  args: Args,
): Promise<unknown> {
  return defaultQaProductRouteAdapter.invoke(command, args);
}

export function resetQaProductRoute(): QaProductRouteFixtureSnapshot {
  return defaultQaProductRouteAdapter.reset();
}

function invokeQaProductRouteWithState(
  state: QaProductRouteState,
  command: string,
  args: Args,
): Promise<unknown> {
  if (command === "platform_snapshot") {
    const prefersDark =
      typeof globalThis.matchMedia === "function" &&
      globalThis.matchMedia("(prefers-color-scheme: dark)").matches;
    return Promise.resolve(
      envelope(args, 0, {
        contractVersion: 1,
        platform: "macos",
        architecture: "aarch64",
        initialTheme: {
          scheme: prefersDark ? "dark" : "light",
          source: "webviewPreferredColorScheme",
        },
        liveThemeSource: "webviewPrefersColorScheme",
        window: {
          decorations: "standard",
          titlebarTransparent: false,
          titlebarOverlay: false,
          initiallyVisible: false,
          revealFallbackTimeoutMs: 5_000,
        },
        vocabulary: {
          revealAction: "Reveal in Finder",
          primaryModifierSymbol: "⌘",
          alternateModifierSymbol: "⌥",
          shiftModifierSymbol: "⇧",
        },
        capabilities: {
          nativeApplicationMenu: true,
          nativeSettingsShortcut: true,
          nativeFilePicker: true,
          nativeFolderPicker: true,
          nativeWorkspacePicker: true,
          standardWindowDecorations: true,
        },
        settingsMenu: {
          menuItemId: "c4os.menu.settings",
          commandId: "c4os.command.openSettings",
          route: "/settings/providers",
          accelerator: "CmdOrCtrl+,",
          keyboardLabel: "⌘,",
        },
      }),
    );
  }
  if (command === "platform_reveal_main") {
    return Promise.resolve(
      envelope(args, 0, {
        revealed: true,
        fallback: false,
      }),
    );
  }
  if (command.startsWith("provider_")) {
    return Promise.resolve(providerCommand(state, command, args));
  }
  if (command === "workspace_start_snapshot") {
    return Promise.resolve(
      envelope(args, state.workspaceGeneration, workspaceStartSnapshot(state)),
    );
  }
  if (command === "workspace_start_open_recent") {
    const input = record(args.input);
    const workspaceId = text(input.workspaceId);
    const recent = QA_RECENT_WORKSPACES.find(({ id }) => id === workspaceId);
    const payload = activateWorkspace(state, workspaceId, recent);
    return Promise.resolve(envelope(args, state.workspaceGeneration, payload));
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
      state,
      QA_WORKSPACE_RECOVERY_NOTICE.workspaceId,
      recent,
    );
    return Promise.resolve(envelope(args, state.workspaceGeneration, payload));
  }
  if (command === "workspace_recovery_acknowledge") {
    const request = record(args.request);
    const input = record(args.input);
    if (
      state.activeRecoveryNotice === null ||
      integer(request.expectedGeneration) !== state.workspaceGeneration ||
      integer(input.expectedGeneration) !== state.workspaceGeneration ||
      text(input.recoveryId) !== state.activeRecoveryNotice.recoveryId ||
      text(input.workspaceId) !== state.activeRecoveryNotice.workspaceId ||
      integer(input.workingGeneration) !==
        state.activeRecoveryNotice.workingGeneration ||
      integer(input.archiveGeneration) !==
        state.activeRecoveryNotice.archiveGeneration
    ) {
      throw new Error("The QA Workspace recovery identity is stale.");
    }
    state.workspaceGeneration += 1;
    state.activeRecoveryNotice = null;
    return Promise.resolve(
      envelope(args, state.workspaceGeneration, workspaceStartSnapshot(state)),
    );
  }
  if (command === "conversation_snapshot") {
    return Promise.resolve(
      envelope(args, state.conversationGeneration, conversationSnapshot(state)),
    );
  }
  if (command === "conversation_update_draft") {
    const input = record(args.input);
    state.conversationGeneration += 1;
    state.draftPrompt = text(input.prompt);
    state.draftMode = composerMode(input.mode);
    state.draftProviderId = nullableText(input.providerId);
    state.draftModelId = nullableText(input.modelId);
    state.draftReasoningMode = reasoningMode(input.reasoningMode);
    return Promise.resolve(
      envelope(args, state.conversationGeneration, conversationSnapshot(state)),
    );
  }
  if (
    command === "conversation_activate_session" ||
    command === "conversation_activate_project"
  ) {
    return Promise.resolve(
      envelope(args, state.conversationGeneration, conversationSnapshot(state)),
    );
  }
  if (command === "artifact_snapshot") {
    return Promise.resolve(
      envelope(args, state.conversationGeneration, {
        protocolVersion: 1,
        generation: state.conversationGeneration,
        authority: "qa-fixture-only",
        workspaceId: state.activatedWorkspaceId,
        activeProjectId:
          state.activatedWorkspaceId === null ? null : "project-qa-0001",
        activeSessionId:
          state.activatedWorkspaceId === null ? null : "chat-qa-0001",
        focusedArtifactId: null,
        artifacts: [],
      }),
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
  return Promise.reject(
    new Error(`QA fixture command is not allowlisted: ${command}`),
  );
}

function activateWorkspace(
  state: QaProductRouteState,
  workspaceId: string,
  recent: (typeof QA_RECENT_WORKSPACES)[number] | undefined,
) {
  state.workspaceGeneration += 1;
  state.conversationGeneration += 1;
  state.activatedWorkspaceId = workspaceId;
  state.activatedWorkspaceName = recent?.name ?? "QA Workspace";
  state.activeRecoveryNotice =
    recent?.isMissing === true ? QA_WORKSPACE_RECOVERY_NOTICE : null;
  return {
    authority: "qa-fixture-only",
    workspaceId,
    workspaceName: state.activatedWorkspaceName,
    recovered: state.activeRecoveryNotice !== null,
    recoveryNotice: state.activeRecoveryNotice,
  };
}

function providerCommand(
  state: QaProductRouteState,
  command: string,
  args: Args,
): unknown {
  const input = record(args.input);
  switch (command) {
    case "provider_snapshot":
    case "provider_accept_session_credentials":
      break;
    case "provider_save_profile": {
      state.providerGeneration += 1;
      const authentication = record(input.authentication);
      const existingProfile =
        state.provider === null ? null : record(state.provider.profile);
      const submittedSecret =
        typeof input.secret === "string" && input.secret.length > 0;
      state.provider = {
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
        generation: state.providerGeneration,
      };
      break;
    }
    case "provider_test_connection": {
      if (state.provider === null)
        throw new Error("QA provider is unavailable");
      state.providerGeneration += 1;
      state.provider = {
        ...state.provider,
        testStatus: { state: "succeeded", checkedAtMs: 1_784_390_400_000 },
        models: {
          "openai/gpt-5": providerModel(),
        },
        selectedModelId: "openai/gpt-5",
        generation: state.providerGeneration,
      };
      break;
    }
    case "provider_select_model":
      if (state.provider === null)
        throw new Error("QA provider is unavailable");
      state.providerGeneration += 1;
      state.provider = {
        ...state.provider,
        selectedModelId: text(input.modelId),
        generation: state.providerGeneration,
      };
      break;
    case "provider_complete_onboarding":
      state.providerGeneration += 1;
      state.configurationGeneration += 1;
      state.onboardingCompletedAtMs = 1_784_390_400_000;
      if (state.provider !== null) {
        state.provider = {
          ...state.provider,
          generation: state.providerGeneration,
        };
      }
      break;
    case "provider_set_models_enabled":
      state.providerGeneration += 1;
      if (state.provider !== null) {
        state.provider = {
          ...state.provider,
          generation: state.providerGeneration,
        };
      }
      break;
    case "provider_delete_profile":
      state.providerGeneration += 1;
      state.provider = null;
      state.onboardingCompletedAtMs = null;
      break;
    case "provider_answer_approval":
      break;
    default:
      throw new Error(`Unsupported QA Provider command: ${command}`);
  }
  return envelope(args, state.providerGeneration, providerSnapshot(state));
}

function providerSnapshot(state: QaProductRouteState) {
  const activeProviderId =
    state.provider === null
      ? null
      : text(record(state.provider.profile).providerId);
  return {
    authority: "qa-fixture-only",
    coordinatorGeneration: state.providerGeneration,
    configurationGeneration: state.configurationGeneration,
    credentialProtection: "installation-key",
    credentialFallbackRequired: false,
    onboardingCompleted:
      state.onboardingCompletedAtMs !== null && state.provider !== null,
    providers: {
      generation: state.providerGeneration,
      onboardingCompletedAtMs: state.onboardingCompletedAtMs,
      providers: state.provider === null ? [] : [state.provider],
    },
    modelRoute:
      state.onboardingCompletedAtMs === null || activeProviderId === null
        ? null
        : `${activeProviderId}::openai/gpt-5`,
    defaultRuntime: state.onboardingCompletedAtMs === null ? null : "opencode",
    defaultEnvironment: state.onboardingCompletedAtMs === null ? null : "local",
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

function conversationSnapshot(state: QaProductRouteState) {
  if (state.activatedWorkspaceId === null) {
    return {
      protocolVersion: 1,
      generation: state.conversationGeneration,
      authority: "qa-fixture-only",
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
    generation: state.conversationGeneration,
    authority: "qa-fixture-only",
    workspaceId: state.activatedWorkspaceId,
    workspaceName: state.activatedWorkspaceName,
    activeProjectId: "project-qa-0001",
    activeSessionId: "chat-qa-0001",
    pending: null,
    draft: {
      prompt: state.draftPrompt,
      attachments: [
        {
          attachmentId: "attachment:qa-concept-board",
          displayName: "concept-board.png",
          mediaType: "image/png",
          byteLength: 284_672,
          stableReference: "qa-fixture:concept-board.png",
          originalReference: 1,
        },
      ],
      nextAttachmentReference: 2,
      providerId: state.draftProviderId,
      modelId: state.draftModelId,
      reasoningMode: state.draftReasoningMode,
      mode: state.draftMode,
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
      {
        projectId: "project-qa-0002",
        displayName: "quotable-ai",
        pathState: "found",
        position: 1,
        gitVersioned: true,
      },
      {
        projectId: "project-qa-0003",
        displayName: "legacy-ui",
        pathState: "missing",
        position: 2,
        gitVersioned: false,
      },
    ],
    sessions: [
      {
        sessionId: "chat-qa-0001",
        projectId: "project-qa-0001",
        title: "QA Chat",
        updatedAtMs: 1_784_390_400_000,
      },
      {
        sessionId: "chat-qa-0002",
        projectId: "project-qa-0001",
        title: "Design workspace projects",
        updatedAtMs: 1_784_390_399_000,
      },
      {
        sessionId: "chat-qa-0003",
        projectId: "project-qa-0002",
        title: "Establish project knowledge base",
        updatedAtMs: 1_784_390_398_000,
      },
    ],
    activeConversation: {
      sessionId: "chat-qa-0001",
      title: "QA Chat",
      turns: [
        {
          turnId: "turn:qa-user",
          prompt: "Build the **workspace shell**.",
          attachments: [],
          artifactContext: null,
          mcpProvenance: null,
          submittedAtMs: 1_784_390_400_000,
        },
      ],
      attempts: [],
      activeAttemptId: null,
    },
    models: [
      {
        providerId: "openrouter",
        providerName: "OpenRouter",
        modelId: "moonshotai/kimi-k2",
        selected: state.draftModelId === "moonshotai/kimi-k2",
        available: true,
        supportsVision: false,
        supportsTools: true,
        supportsReasoning: false,
        supportsAudio: false,
        contextTokens: 128_000,
      },
      {
        providerId: "openai",
        providerName: "OpenAI",
        modelId: "openai/gpt-5",
        selected: state.draftModelId === "openai/gpt-5",
        available: true,
        supportsVision: true,
        supportsTools: true,
        supportsReasoning: true,
        supportsAudio: false,
        contextTokens: 400_000,
      },
    ],
    branchControl: null,
  };
}

function workspaceStartSnapshot(state: QaProductRouteState) {
  return {
    protocolVersion: 1,
    generation: state.workspaceGeneration,
    authority: "qa-fixture-only",
    recents: QA_RECENT_WORKSPACES.map((recent, index) => ({
      workspaceId: recent.id,
      displayName: recent.name,
      lastOpenedAt: 1_784_390_400 - index * 86_400,
      isMissing: recent.isMissing ?? false,
    })),
    activeRecoveryNotice: state.activeRecoveryNotice,
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

function nullableText(value: unknown): string | null {
  if (value === null || value === undefined) return null;
  return text(value);
}

function composerMode(
  value: unknown,
): "chat" | "files" | "browser" | "terminal" {
  if (
    value === "chat" ||
    value === "files" ||
    value === "browser" ||
    value === "terminal"
  ) {
    return value;
  }
  throw new Error("Invalid QA fixture Composer mode");
}

function reasoningMode(
  value: unknown,
): "off" | "low" | "medium" | "high" | null {
  if (value === null || value === undefined) return null;
  if (
    value === "off" ||
    value === "low" ||
    value === "medium" ||
    value === "high"
  ) {
    return value;
  }
  throw new Error("Invalid QA fixture reasoning mode");
}
