//client
import { createLocalDiagnostics } from './diagnostics.js';

// Backend command names exposed through the app boundary.
export const BACKEND_COMMANDS = Object.freeze({
  getAppStatus: 'get_app_status',
  configureOpenRouter: 'configure_openrouter',
  registerProject: 'register_project',
  listProjectSkills: 'list_project_skills',
  invokeProjectSkill: 'invoke_project_skill',
  listMcpServers: 'list_mcp_servers',
  registerLocalStdioMcpServer: 'register_local_stdio_mcp_server',
  proposeMcpServerLaunch: 'propose_mcp_server_launch',
  exportProjectJson: 'export_project_json',
  deleteArchivedSession: 'delete_archived_session',
  submitPrompt: 'submit_prompt',
});

/**
 * Creates the backend command registry used by UI and Tauri adapters.
 */
export function createBackendCommandRegistry(options = {}) {
  const diagnostics = options.diagnostics ?? createLocalDiagnostics();
  const state = {
    provider: {
      id: 'openrouter',
      ready: false,
      selectedModel: null,
      metadataStatus: 'unknown',
      disclosure:
        'Prompts and bounded context may be sent through OpenRouter.',
    },
    project: {
      active: false,
      id: null,
      rootPath: null,
      branch: null,
      dirty: false,
      changedFileCount: 0,
    },
    session: {
      active: false,
      id: null,
      title: null,
      runtimeState: 'stopped',
      messages: [],
    },
    mcpServers: [],
  };

  // Keep the command map explicit so future protected commands have one
  // obvious registration point.
  const handlers = new Map([
    [
      BACKEND_COMMANDS.getAppStatus,
      async () => ({
        appName: 'C4OS',
        backendReady: true,
        telemetryEnabled: false,
        diagnostics,
        provider: { ...state.provider },
        providerExpansion: providerExpansionStatus(),
        project: {
          active: state.project.active,
          rootPath: state.project.rootPath,
          branch: state.project.branch,
          dirty: state.project.dirty,
          changedFileCount: state.project.changedFileCount,
          browserMode: 'read-only',
          rootAgentsDisplay: 'passive',
          instructionResolution: {
            nestedAgentsSupported: true,
            supportTier: 'display_guidance_order_only',
            permissionEffect: 'none',
            modelContextEffect: 'none_without_explicit_runtime_read',
            editReloadBehavior: 'next_preflight_only',
            conflictDiagnostics: 'ordered_sources_no_semantic_merge',
          },
          selector: {
            available: true,
            listRegisteredProjects: true,
            selectExactlyOneActive: true,
            registeredProjectCount: state.project.active ? 1 : 0,
            multipleActiveProjectsAllowed: false,
            searchAvailable: false,
            groupingAvailable: false,
            archiveAvailable: false,
            deleteAvailable: false,
            favoritesAvailable: false,
            metadataEditingAvailable: false,
            crossProjectViewsAvailable: false,
            nonGitProjectsAllowed: false,
            worktreeManagementAvailable: false,
          },
        },
        timeline: {
          toolActivityVisible: true,
          rawOutputExportAvailable: false,
          liveDrawerLabel: 'live/ephemeral',
        },
        session: {
          active: state.session.active,
          id: state.session.id,
          title: state.session.title,
          runtimeState: state.session.runtimeState,
          runtimeStates: [
            'running',
            'waiting_for_approval',
            'stopped',
            'failed',
            'complete',
          ],
          transcriptAppendOnly: true,
          canStartNewRun: !state.session.active,
          failureDisplay: state.session.failureDisplay ?? null,
          messages: [...state.session.messages],
        },
        retention: retentionStatus(),
        memory: memoryStatus(),
        compatibility: compatibilityStatus(),
        browserWebViewing: browserWebViewingStatus(),
        pluginMarketplace: pluginMarketplaceStatus(),
        approvals: {
          pendingCount: 0,
          choices: ['allow_once', 'allow_session', 'deny'],
          highRiskSessionAllow: false,
          policyBlocksExecute: false,
        },
        recovery: {
          canStop: false,
          canRetry: false,
          retryAppendsNewAction: true,
          autoContinueAfterRestart: false,
          reattachAfterRestart: false,
          resendPendingPromptsAfterRestart: false,
        },
        changes: {
          changedFileCount: 0,
          diffViewerAvailable: true,
          gitInspectionLogged: true,
          approvalRequiredForInspection: false,
          refreshAfterToolExecution: true,
        },
        artifacts: {
          supportTier: 'raster_image_preview_only',
          visibleRecords: true,
          searchAvailable: false,
          activeHtmlPreview: false,
          rasterImagePreview: true,
          richPreview: true,
          exportAvailable: true,
          exportSupportTier: 'project_json_export_only',
          importAvailable: false,
          roundTripCompatibility: false,
          includeInModelContextByDefault: false,
          supportedPreviewTypes: [
            'text',
            'markdown',
            'log',
            'diff',
            'source',
            'config',
            'png',
            'jpeg',
            'webp',
            'gif',
          ],
          unsupportedPreviewTypes: [
            'svg',
            'html',
            'pdf',
            'document',
            'spreadsheet',
            'remote_url',
          ],
        },
        portability: portabilityStatus(),
        skills: skillCapabilityStatus(),
        mcp: mcpCapabilityStatus(),
      }),
    ],
    [
      BACKEND_COMMANDS.configureOpenRouter,
      async (payload) => {
        const apiKey = `${payload.apiKey ?? ''}`.trim();
        const selectedModel = `${payload.selectedModel ?? ''}`.trim();

        if (!apiKey) {
          throw new Error('OpenRouter key is required.');
        }

        if (!selectedModel) {
          throw new Error('Select a model before saving provider setup.');
        }

        state.provider.ready = true;
        state.provider.selectedModel = selectedModel;

        return { ...state.provider };
      },
    ],
    [
      BACKEND_COMMANDS.registerProject,
      async (payload) => {
        const rootPath = `${payload.rootPath ?? ''}`.trim();

        if (!rootPath) {
          throw new Error('Project path is required.');
        }

        state.project = {
          active: true,
          id: `project-${stableHash(rootPath)}`,
          rootPath,
          branch: 'unknown',
          dirty: false,
          changedFileCount: 0,
        };

        return { ...state.project };
      },
    ],
    [
      BACKEND_COMMANDS.listProjectSkills,
      async () => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before listing skills.');
        }

        return {
          ...skillCapabilityStatus(),
          skills: [],
        };
      },
    ],
    [
      BACKEND_COMMANDS.invokeProjectSkill,
      async (payload) => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before invoking skills.');
        }

        const relativePath = `${payload.relativePath ?? ''}`.trim();

        if (!relativePath) {
          throw new Error('Skill path is required.');
        }

        return {
          name: relativePath.split('/').at(-2) ?? 'skill',
          description: '',
          relativePath,
          content: '',
          contextEffect: 'explicit_user_selected_context',
          permissionEffect: 'none',
          unsupportedCapabilities: unsupportedSkillCapabilities(),
        };
      },
    ],
    [
      BACKEND_COMMANDS.listMcpServers,
      async () => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before using MCP servers.');
        }

        return {
          supportTier: 'local_stdio_explicit_approval_only',
          servers: state.mcpServers.map((server) => ({ ...server })),
        };
      },
    ],
    [
      BACKEND_COMMANDS.registerLocalStdioMcpServer,
      async (payload) => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before using MCP servers.');
        }

        if (payload.remoteUrl) {
          throw new Error('Remote MCP is unavailable in V1. Register a local stdio server instead.');
        }

        const name = `${payload.name ?? ''}`.trim();
        const command = `${payload.command ?? ''}`.trim();

        if (!name || !command) {
          throw new Error('MCP server name and command are required.');
        }

        const server = {
          id: `mcp-${stableSlug(name)}`,
          name,
          transport: 'stdio',
          command,
          args: Array.isArray(payload.args) ? [...payload.args] : [],
          workingDirectory: resolveProjectPath(
            state.project.rootPath,
            `${payload.workingDirectory ?? '.'}`,
          ),
          rootScope: state.project.rootPath,
          autoStart: false,
          remoteUrl: null,
          resourcesAutoContext: false,
          promptsAutoContext: false,
          samplingAutoApproved: false,
          elicitationAutoDisclosure: false,
        };

        state.mcpServers = [
          ...state.mcpServers.filter((existing) => existing.id !== server.id),
          server,
        ];

        return { ...server };
      },
    ],
    [
      BACKEND_COMMANDS.proposeMcpServerLaunch,
      async (payload) => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before using MCP servers.');
        }

        const server = state.mcpServers.find(
          (candidate) => candidate.id === payload.serverId,
        );

        if (!server) {
          throw new Error('Registered MCP server was not found.');
        }

        return {
          serverId: server.id,
          command: [server.command, ...server.args].join(' '),
          workingDirectory: server.workingDirectory,
          rootScope: state.project.rootPath,
          requiresApproval: true,
          riskLevel: 'medium',
          approvalActionType: 'mcp_stdio_launch',
          filteredEnvironment: true,
          startsProcess: false,
          unsupportedCapabilities: unsupportedMcpCapabilities(),
        };
      },
    ],
    [
      BACKEND_COMMANDS.exportProjectJson,
      async () => {
        if (!state.project.active) {
          throw new Error('Register and select a Git project before exporting.');
        }

        return {
          format: 'c4os.project_export.v1',
          supportTier: 'project_json_export_only',
          importAvailable: false,
          roundTripCompatibility: false,
          project: {
            id: state.project.id,
            name: state.project.rootPath.split('/').filter(Boolean).at(-1) ?? 'Project',
            rootPath: null,
            rootPathPolicy: 'excluded_absolute_local_path',
            defaultModel: state.provider.selectedModel,
            defaultAgentRef: null,
          },
          sessions: state.session.id
            ? [
                {
                  id: state.session.id,
                  title: state.session.title,
                  status: state.session.runtimeState,
                  mode: 'agent',
                  modelId: state.provider.selectedModel,
                  runtime: 'opencode',
                  agentRef: null,
                  runtimeSessionRef: null,
                  messages: state.session.messages.map((message) => ({
                    id: message.id,
                    role: message.role,
                    content: redactExportText(message.content),
                    status: message.status,
                    metadata: {},
                  })),
                  toolCalls: [],
                },
              ]
            : [],
          artifacts: [],
          security: exportSecurityStatus(),
        };
      },
    ],
    [
      BACKEND_COMMANDS.deleteArchivedSession,
      async (payload) => {
        const sessionId = `${payload.sessionId ?? ''}`.trim();

        if (!sessionId) {
          throw new Error('Session id is required.');
        }

        if (sessionId !== state.session.id) {
          throw new Error('Session was not found.');
        }

        if (!state.session.archived) {
          throw new Error('Only archived sessions can be deleted.');
        }

        if (state.session.pinned) {
          throw new Error('Pinned sessions must be unpinned before deletion.');
        }

        throw new Error('The latest session cannot be deleted in this V1 tier.');
      },
    ],
    [
      BACKEND_COMMANDS.submitPrompt,
      async (payload) => {
        const prompt = `${payload.prompt ?? ''}`.trim();

        if (!state.provider.ready) {
          throw new Error('Configure OpenRouter before starting a session.');
        }

        if (!state.project.active) {
          throw new Error('Register and select a Git project before starting a session.');
        }

        if (!prompt) {
          throw new Error('Enter a prompt before starting a session.');
        }

        state.session = {
          active: false,
          id: `session-${Date.now()}`,
          title: firstLine(prompt),
          runtimeState: 'complete',
          failureDisplay: null,
          archived: false,
          pinned: false,
          messages: [
            {
              id: `message-${Date.now()}`,
              role: 'user',
              content: prompt,
              status: 'submitted',
            },
            {
              id: `message-${Date.now()}-assistant`,
              role: 'assistant',
              content:
                'Browser smoke mode recorded this prompt. The desktop build runs OpenCode through the backend.',
              status: 'complete',
            },
          ],
        };

        return { ...state.session, messages: [...state.session.messages] };
      },
    ],
  ]);

  return {
    /**
     * Invokes a registered backend command by its stable command name.
     */
    async invoke(commandName, payload = {}) {
      const handler = handlers.get(commandName);

      if (!handler) {
        throw new Error(`Unknown backend command: ${commandName}`);
      }

      return handler(payload);
    },
  };
}

/**
 * Describes the accepted explicit-only Agent Skills support tier.
 */
function skillCapabilityStatus() {
  return {
    supportTier: 'explicit_discovery_and_invocation_only',
    discoveryAvailable: true,
    explicitInvocationRequired: true,
    autoInvocationAvailable: false,
    scriptExecutionAvailable: false,
    referencesAutoLoaded: false,
    trustedAssetRendering: false,
    globalCatalogAvailable: false,
    pluginProvidedSkillsAvailable: false,
    permissionEffect: 'none',
    modelContextEffect: 'explicit_user_selected_context_only',
  };
}

/**
 * Describes the accepted local stdio MCP support tier.
 */
function mcpCapabilityStatus() {
  return {
    supportTier: 'local_stdio_explicit_approval_only',
    localStdioAvailable: true,
    remoteAvailable: false,
    explicitRegistrationRequired: true,
    autoStartFromProjectFiles: false,
    approvalRequiredForLaunch: true,
    rootsLimitedToSelectedProject: true,
    toolsRequireApproval: true,
    resourcesAutoContext: false,
    promptsAutoContext: false,
    samplingAutoApproved: false,
    elicitationAutoDisclosure: false,
    fullCompatibilityClaim: false,
  };
}

/**
 * Describes the accepted V1 export-only portability tier.
 */
function portabilityStatus() {
  return {
    supportTier: 'project_json_export_only',
    exportAvailable: true,
    importAvailable: false,
    roundTripCompatibility: false,
    ...exportSecurityStatus(),
  };
}

/**
 * Describes the accepted V1 retention/delete tier.
 */
function retentionStatus() {
  return {
    supportTier: 'archived_session_delete_only',
    deleteAvailable: true,
    explicitConfirmationRequired: true,
    archivedUnpinnedSessionDelete: true,
    activeSessionDelete: false,
    latestSessionDelete: false,
    runningSessionDelete: false,
    pendingApprovalSessionDelete: false,
    pinnedSessionDelete: false,
    automaticCleanup: false,
    storageQuotaCleanup: false,
    messageLevelDelete: false,
    messageRedaction: false,
    memoryControls: false,
    importAvailable: false,
    roundTripCompatibility: false,
  };
}

/**
 * Describes the accepted V1 no-durable-memory tier.
 */
function memoryStatus() {
  return {
    supportTier: 'no_durable_memory_v1',
    durableMemoryAvailable: false,
    crossSessionMemory: false,
    learnedPreferences: false,
    automaticSummaries: false,
    embeddings: false,
    backgroundIndexing: false,
    memoryWritePrompts: false,
    memoryInspectEditDeleteUi: false,
    providerSideMemoryClaims: false,
    memoryImportExport: false,
    modelContextAutoInjection: false,
    deletedSessionMemoryRecords: false,
  };
}

/**
 * Describes the accepted V1 compatibility-claims tier.
 */
function compatibilityStatus() {
  return {
    supportTier: 'no_broader_compatibility_claims_v1',
    claimsAreFeatureLevel: true,
    openRouterBackedModelAccess: true,
    localGitProjectSupport: true,
    rootAgentsDisplay: true,
    nestedAgentsDisplayGuidance: true,
    explicitProjectLocalSkills: true,
    localStdioMcpExplicitApproval: true,
    appOwnedTextAndRasterArtifacts: true,
    projectJsonExport: true,
    archivedSessionDelete: true,
    durableMemory: false,
    fullAgentsMdCompatibility: false,
    fullAgentSkillsCompatibility: false,
    fullMcpCompatibility: false,
    codexPluginCompatibility: false,
    openCodeConfigCompatibility: false,
    importCompatibility: false,
    roundTripCompatibility: false,
    browserCompatibility: false,
    standardsCompatibleMarketingClaim: false,
  };
}

/**
 * Describes the accepted V1 provider expansion boundary.
 */
function providerExpansionStatus() {
  return {
    supportTier: 'openrouter_only_v1_no_direct_or_local_provider_expansion',
    openRouterOnly: true,
    openRouterConfigurable: true,
    directProviderSetup: false,
    localModelProviderSetup: false,
    offlineModelFallback: false,
    providerFallbackRouting: false,
    automaticModelSwitching: false,
    byokProviderSubconfiguration: false,
    providerSpecificSettings: false,
    providerComparisonWorkflow: false,
    multiProviderConfiguration: false,
    credentialStorageRemainsKeychainOnly: true,
    runningSessionCredentialMutation: false,
    oneModelPerSession: true,
    midSessionModelSwitching: false,
  };
}

/**
 * Describes the accepted V1 browser and web-viewing boundary.
 */
function browserWebViewingStatus() {
  return {
    supportTier: 'no_browser_or_web_viewing_v1',
    inAppBrowserPanel: false,
    remoteUrlViewing: false,
    domExtraction: false,
    screenshots: false,
    screenshotUnderstanding: false,
    browserTesting: false,
    browserAutomation: false,
    webContentIngestion: false,
    chromiumBackedRendering: false,
    generatedHtmlExecution: false,
    browserContentModelContextIngestion: false,
    rendererIsolationSpecified: false,
    promptInjectionMitigationSpecified: false,
  };
}

/**
 * Describes the accepted V1 plugin and marketplace boundary.
 */
function pluginMarketplaceStatus() {
  return {
    supportTier: 'no_plugin_or_marketplace_v1',
    pluginInstallAvailable: false,
    pluginEnableAvailable: false,
    pluginExecutionAvailable: false,
    pluginScriptsHooks: false,
    pluginPermissionGrants: false,
    trustedPluginAssets: false,
    pluginProvidedMcpServers: false,
    marketplaceBrowsing: false,
    remoteMarketplaceManifests: false,
    pluginSearchInstallUpdate: false,
    ratingsReviewsAdvisories: false,
    curationWorkflows: false,
    projectLocalPluginMetadataTrusted: false,
    projectLocalPluginMetadataExecuted: false,
  };
}

/**
 * Lists the skill capabilities intentionally kept out of V1.
 */
function unsupportedSkillCapabilities() {
  return [
    'auto_invocation_unavailable',
    'script_execution_unavailable',
    'references_auto_load_unavailable',
    'trusted_asset_rendering_unavailable',
    'permission_effect_unavailable',
    'global_catalog_unavailable',
    'plugin_provided_skills_unavailable',
    'version_conflict_resolution_unavailable',
    'marketplace_unavailable',
    'round_trip_compatibility_unavailable',
  ];
}

/**
 * Lists MCP capabilities intentionally kept out of V1.
 */
function unsupportedMcpCapabilities() {
  return [
    'remote_mcp_unavailable',
    'authorization_flows_unavailable',
    'automatic_project_file_startup_unavailable',
    'automatic_resource_context_unavailable',
    'automatic_prompt_use_unavailable',
    'unapproved_tool_invocation_unavailable',
    'automatic_sampling_unavailable',
    'automatic_elicitation_disclosure_unavailable',
    'full_mcp_compatibility_unavailable',
  ];
}

/**
 * Security posture for project JSON exports.
 */
function exportSecurityStatus() {
  return {
    rawSecretsIncluded: false,
    rawShellOutputIncluded: false,
    providerKeysIncluded: false,
    absoluteLocalPathsIncluded: false,
    artifactPayloadsIncluded: false,
  };
}

/**
 * Creates a deterministic browser-smoke identifier without storing raw paths.
 */
function stableHash(value) {
  let hash = 0;

  for (const character of value) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }

  return hash.toString(16);
}

/**
 * Creates a stable lower-case identifier from a visible server name.
 */
function stableSlug(value) {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, '')
    .slice(0, 48);

  return slug || stableHash(value);
}

/**
 * Resolves the browser-smoke working directory within the selected project.
 */
function resolveProjectPath(rootPath, requestedPath) {
  if (!requestedPath || requestedPath === '.') {
    return rootPath;
  }

  if (requestedPath.startsWith('/')) {
    if (!requestedPath.startsWith(rootPath)) {
      throw new Error('MCP server working directories must stay inside the selected project root.');
    }

    return requestedPath;
  }

  return `${rootPath.replace(/\/$/, '')}/${requestedPath}`;
}

/**
 * Redacts secret-shaped lines before they enter export payloads.
 */
function redactExportText(value) {
  return `${value ?? ''}`
    .split('\n')
    .map((line) => (containsSecretMaterial(line) ? '[REDACTED_SECRET_LINE]' : line))
    .join('\n');
}

/**
 * Detects obvious secret material in exportable text.
 */
function containsSecretMaterial(value) {
  const lower = `${value}`.toLowerCase();

  return lower.includes('sk-or-')
    || lower.includes('sk-')
    || lower.includes('token=')
    || lower.includes('api_key')
    || lower.includes('apikey')
    || lower.includes('password')
    || lower.includes('secret')
    || lower.includes('bearer ')
    || lower.includes('-----begin');
}

/**
 * Uses the first prompt line as the local session title.
 */
function firstLine(value) {
  return value.split('\n')[0].slice(0, 80);
}
