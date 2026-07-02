import test from 'node:test';
import assert from 'node:assert/strict';
import { runFanoutProof } from './proof.mjs';

test('one backend call fans out to visible and hidden compatible plugin views', () => {
  const result = runFanoutProof();

  assert.equal(result.backendCalls, 1);
  assert.deepEqual(result.visibleRenders, ['browser-left', 'browser-right']);
  assert.deepEqual(result.hiddenUpdates, ['browser-hidden']);
  assert.equal(result.hiddenOpenedPanel, false);
  assert.equal(result.hiddenStoleFocus, false);
  assert.equal(result.hiddenPromptedUser, false);
  assert.equal(result.duplicateBackendCalls, 0);
});
