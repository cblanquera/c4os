import assert from 'node:assert/strict';
import test from 'node:test';

import {
  BACKEND_COMMANDS,
  createBackendCommandRegistry,
} from '../src/backend/commands.js';
import { createAppCommandClient } from '../src/backend/tauri-adapter.js';

test('backend status command exposes local-only diagnostics and no telemetry', async () => {
  const registry = createBackendCommandRegistry();

  const status = await registry.invoke(BACKEND_COMMANDS.getAppStatus);

  assert.equal(status.appName, 'C4OS');
  assert.equal(status.backendReady, true);
  assert.equal(status.telemetryEnabled, false);
  assert.deepEqual(status.diagnostics, {
    storageMode: 'local-only',
    productTelemetry: 'disabled',
    automaticCrashUpload: false,
    supportBundleUpload: false,
  });
  assert.deepEqual(status.provider, {
    id: 'openrouter',
    ready: false,
    selectedModel: null,
    metadataStatus: 'unknown',
    disclosure: 'Prompts and bounded context may be sent through OpenRouter.',
  });
  assert.deepEqual(status.providerExpansion, {
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
  });
  assert.deepEqual(status.project, {
    active: false,
    rootPath: null,
    branch: null,
    dirty: false,
    changedFileCount: 0,
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
      registeredProjectCount: 0,
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
  });
  assert.deepEqual(status.timeline, {
    toolActivityVisible: true,
    rawOutputExportAvailable: false,
    liveDrawerLabel: 'live/ephemeral',
  });
  assert.deepEqual(status.session, {
    active: false,
    id: null,
    title: null,
    runtimeState: 'stopped',
    runtimeStates: [
      'running',
      'waiting_for_approval',
      'stopped',
      'failed',
      'complete',
    ],
    transcriptAppendOnly: true,
    canStartNewRun: true,
    failureDisplay: null,
    messages: [],
  });
  assert.deepEqual(status.retention, {
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
  });
  assert.deepEqual(status.memory, {
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
  });
  assert.deepEqual(status.compatibility, {
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
  });
  assert.deepEqual(status.browserWebViewing, {
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
  });
  assert.deepEqual(status.pluginMarketplace, {
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
  });
  assert.deepEqual(status.approvals, {
    pendingCount: 0,
    choices: ['allow_once', 'allow_session', 'deny'],
    highRiskSessionAllow: false,
    policyBlocksExecute: false,
  });
  assert.deepEqual(status.recovery, {
    canStop: false,
    canRetry: false,
    retryAppendsNewAction: true,
    autoContinueAfterRestart: false,
    reattachAfterRestart: false,
    resendPendingPromptsAfterRestart: false,
  });
  assert.deepEqual(status.changes, {
    changedFileCount: 0,
    diffViewerAvailable: true,
    gitInspectionLogged: true,
    approvalRequiredForInspection: false,
    refreshAfterToolExecution: true,
  });
  assert.deepEqual(status.artifacts, {
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
    supportedPreviewTypes: ['text', 'markdown', 'log', 'diff', 'source', 'config', 'png', 'jpeg', 'webp', 'gif'],
    unsupportedPreviewTypes: ['svg', 'html', 'pdf', 'document', 'spreadsheet', 'remote_url'],
  });
  assert.deepEqual(status.skills, {
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
  });
  assert.deepEqual(status.mcp, {
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
  });
  assert.deepEqual(status.portability, {
    supportTier: 'project_json_export_only',
    exportAvailable: true,
    importAvailable: false,
    roundTripCompatibility: false,
    rawSecretsIncluded: false,
    rawShellOutputIncluded: false,
    providerKeysIncluded: false,
    absoluteLocalPathsIncluded: false,
    artifactPayloadsIncluded: false,
  });
});

test('backend registry exports selected project JSON without import support', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await client.registerProject({ rootPath: '/tmp/c4os-test-project' });
  const exported = await client.exportProjectJson();

  assert.equal(exported.format, 'c4os.project_export.v1');
  assert.equal(exported.supportTier, 'project_json_export_only');
  assert.equal(exported.importAvailable, false);
  assert.equal(exported.roundTripCompatibility, false);
  assert.equal(exported.project.rootPath, null);
  assert.equal(exported.security.rawSecretsIncluded, false);
  assert.equal(exported.security.rawShellOutputIncluded, false);
  assert.equal(exported.security.providerKeysIncluded, false);
  assert.equal(exported.security.absoluteLocalPathsIncluded, false);
  assert.equal(exported.security.artifactPayloadsIncluded, false);
});

test('backend registry exposes archived-session delete-only boundary', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await client.configureOpenRouter({
    apiKey: 'sk-or-valid-test-key',
    selectedModel: 'openai/gpt-4.1',
  });
  await client.registerProject({ rootPath: '/tmp/c4os-test-project' });
  const session = await client.submitPrompt({ prompt: 'Create a session.' });

  await assert.rejects(
    () => client.deleteArchivedSession({ sessionId: session.id }),
    /Only archived sessions can be deleted/,
  );
});

test('backend registry exposes explicit project-local skill commands', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await client.registerProject({ rootPath: '/tmp/c4os-test-project' });

  const catalog = await client.listProjectSkills();
  const invocation = await client.invokeProjectSkill({
    relativePath: 'skills/review/SKILL.md',
  });

  assert.equal(catalog.supportTier, 'explicit_discovery_and_invocation_only');
  assert.equal(catalog.autoInvocationAvailable, false);
  assert.deepEqual(catalog.skills, []);
  assert.equal(invocation.relativePath, 'skills/review/SKILL.md');
  assert.equal(invocation.contextEffect, 'explicit_user_selected_context');
  assert.equal(invocation.permissionEffect, 'none');
});

test('backend registry exposes explicit local stdio MCP launch proposals', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await client.registerProject({ rootPath: '/tmp/c4os-test-project' });
  const server = await client.registerLocalStdioMcpServer({
    name: 'filesystem',
    command: 'node',
    args: ['server.js'],
    workingDirectory: '.',
  });
  const proposal = await client.proposeMcpServerLaunch({
    serverId: server.id,
  });

  assert.equal(server.transport, 'stdio');
  assert.equal(server.autoStart, false);
  assert.equal(proposal.requiresApproval, true);
  assert.equal(proposal.startsProcess, false);
  assert.equal(proposal.approvalActionType, 'mcp_stdio_launch');
  assert.equal(proposal.rootScope, '/tmp/c4os-test-project');
});

test('backend registry rejects remote MCP and does not auto-import project MCP files', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await client.registerProject({ rootPath: '/tmp/c4os-test-project' });
  const catalog = await client.listMcpServers();

  assert.deepEqual(catalog.servers, []);
  await assert.rejects(
    () => client.registerLocalStdioMcpServer({
      name: 'remote',
      command: 'remote',
      args: [],
      workingDirectory: '.',
      remoteUrl: 'https://example.com/mcp',
    }),
    /Remote MCP is unavailable/,
  );
});

test('UI command client can invoke the backend boundary', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  const status = await client.getAppStatus();

  assert.equal(status.backendReady, true);
});

test('backend registry supports the MVP first-run workflow', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  const provider = await client.configureOpenRouter({
    apiKey: 'sk-or-valid-test-key',
    selectedModel: 'openai/gpt-4.1',
  });
  const project = await client.registerProject({
    rootPath: '/tmp/c4os-test-project',
  });
  const session = await client.submitPrompt({
    prompt: 'Summarize this repository.',
  });
  const status = await client.getAppStatus();

  assert.equal(provider.ready, true);
  assert.equal(provider.selectedModel, 'openai/gpt-4.1');
  assert.equal(project.active, true);
  assert.equal(project.rootPath, '/tmp/c4os-test-project');
  assert.equal(session.messages[0].content, 'Summarize this repository.');
  assert.equal(status.provider.ready, true);
  assert.equal(status.project.active, true);
  assert.equal(status.session.active, false);
  assert.equal(status.session.runtimeState, 'complete');
  assert.equal(status.session.failureDisplay, null);
  assert.match(status.session.messages[1].content, /desktop build runs OpenCode/);
});

test('first-run workflow fails closed without provider and project setup', async () => {
  const registry = createBackendCommandRegistry();
  const client = createAppCommandClient({ registry });

  await assert.rejects(
    () => client.configureOpenRouter({ apiKey: '', selectedModel: 'openai/gpt-4.1' }),
    /OpenRouter key is required/,
  );
  await assert.rejects(
    () => client.submitPrompt({ prompt: 'Try without setup.' }),
    /Configure OpenRouter before starting a session/,
  );
});

test('UI command client forwards commands through Tauri invoke when available', async () => {
  const calls = [];
  const client = createAppCommandClient({
    invoke(commandName, payload) {
      calls.push({ commandName, payload });

      if (commandName === BACKEND_COMMANDS.getAppStatus) {
        return Promise.resolve({ backendReady: true });
      }

      return Promise.resolve({ ok: true });
    },
  });

  await client.configureOpenRouter({
    apiKey: 'sk-or-valid-test-key',
    selectedModel: 'openai/gpt-4.1',
  });
  await client.deleteArchivedSession({ sessionId: 'session-1' });
  await client.getAppStatus();

  assert.deepEqual(calls, [
    {
      commandName: BACKEND_COMMANDS.configureOpenRouter,
      payload: {
        apiKey: 'sk-or-valid-test-key',
        selectedModel: 'openai/gpt-4.1',
      },
    },
    {
      commandName: BACKEND_COMMANDS.deleteArchivedSession,
      payload: {
        sessionId: 'session-1',
      },
    },
    {
      commandName: BACKEND_COMMANDS.getAppStatus,
      payload: {},
    },
  ]);
});

test('unknown backend commands fail closed', async () => {
  const registry = createBackendCommandRegistry();

  await assert.rejects(
    () => registry.invoke('unknown_command'),
    /Unknown backend command/,
  );
});
