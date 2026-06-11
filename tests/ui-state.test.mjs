import assert from 'node:assert/strict';
import test from 'node:test';

import {
  canSubmitPrompt,
  errorMessage,
  skillCapabilityLabel,
  mcpCapabilityLabel,
  sessionActivityMessage,
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
