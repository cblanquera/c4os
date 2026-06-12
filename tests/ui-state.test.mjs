import assert from 'node:assert/strict';
import test from 'node:test';

import {
  canSubmitPrompt,
  errorMessage,
  skillCapabilityLabel,
  mcpCapabilityLabel,
  resumableSessionLabel,
  sessionActivityMessage,
  workflowContextLabel,
  workflowPurposeOptions,
  workspaceNavigationLabel,
} from '../src/ui/state.js';

test('errorMessage renders Tauri string rejections', () => {
  assert.equal(
    errorMessage('A session is already running.'),
    'A session is already running.',
  );
});

test('errorMessage renders structured unknown failures without a blank notice', () => {
  assert.equal(
    errorMessage({ code: 'OPENROUTER_FAILED', message: '' }),
    '{"code":"OPENROUTER_FAILED","message":""}',
  );
});

test('active backend sessions block new prompt submission', () => {
  assert.equal(
    canSubmitPrompt({
      provider: { ready: true },
      project: { active: true },
      session: { active: true },
    }, null),
    false,
  );
});

test('sessionActivityMessage explains active backend runs after rerender', () => {
  assert.equal(
    sessionActivityMessage({
      session: { active: true, runtimeState: 'running' },
    }, null),
    'OpenCode is running through OpenRouter. Waiting for a response...',
  );
});

test('skillCapabilityLabel reports explicit-only Agent Skills support', () => {
  assert.equal(
    skillCapabilityLabel({
      skills: {
        supportTier: 'explicit_discovery_and_invocation_only',
        autoInvocationAvailable: false,
        scriptExecutionAvailable: false,
      },
    }),
    'Explicit skills only',
  );
});

test('mcpCapabilityLabel reports local stdio explicit approval support', () => {
  assert.equal(
    mcpCapabilityLabel({
      mcp: {
        supportTier: 'local_stdio_explicit_approval_only',
        localStdioAvailable: true,
        remoteAvailable: false,
        autoStartFromProjectFiles: false,
      },
    }),
    'Local MCP approval only',
  );
});

test('workflowPurposeOptions exposes only bounded foundation workflows', () => {
  assert.deepEqual(
    workflowPurposeOptions({
      workflowOrganization: {
        allowedPurposes: ['research', 'writing', 'coding', 'documentation', 'analysis'],
      },
    }),
    ['research', 'writing', 'documentation', 'analysis'],
  );
});

test('workspaceNavigationLabel reports promoted project and session filters', () => {
  assert.equal(
    workspaceNavigationLabel({
      project: {
        selector: {
          searchAvailable: true,
          workflowPurposeFilterAvailable: true,
        },
      },
      session: {
        catalog: {
          searchAvailable: true,
          workflowPurposeFilterAvailable: true,
        },
      },
    }),
    'Project and session filters',
  );
});

test('workflowContextLabel preserves no-hidden-context semantics', () => {
  assert.equal(
    workflowContextLabel({
      workflowOrganization: {
        modelContextEffect: 'none',
        autoContextInjection: false,
      },
    }),
    'Labels only',
  );
});

test('resumableSessionLabel reports latest local session state', () => {
  assert.equal(
    resumableSessionLabel({
      session: {
        id: 'session-1',
        active: false,
        runtimeState: 'complete',
      },
    }),
    'complete',
  );
});
