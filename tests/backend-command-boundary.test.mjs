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

test('unknown backend commands fail closed', async () => {
  const registry = createBackendCommandRegistry();

  await assert.rejects(
    () => registry.invoke('unknown_command'),
    /Unknown backend command/,
  );
});
