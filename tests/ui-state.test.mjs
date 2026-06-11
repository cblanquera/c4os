import assert from 'node:assert/strict';
import test from 'node:test';

import {
  canSubmitPrompt,
  errorMessage,
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
