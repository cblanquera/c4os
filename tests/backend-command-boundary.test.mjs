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
    visibleRecords: true,
    searchAvailable: false,
    activeHtmlPreview: false,
    richPreview: false,
    exportAvailable: false,
    includeInModelContextByDefault: false,
  });
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
