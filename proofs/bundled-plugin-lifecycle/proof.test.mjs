import test from 'node:test';
import assert from 'node:assert/strict';
import { runLifecycleProof } from './proof.mjs';

test('bundled plugin installs disabled, uninstalls, and reinstalls from bundled source', () => {
  const result = runLifecycleProof();

  assert.deepEqual(result.states, [
    'installed-disabled',
    'enabled',
    'uninstall-prompt-delete-data',
    'uninstalled',
    'reinstalled-from-bundled-default',
    'installed-disabled'
  ]);
  assert.equal(result.dataDeletePromptShown, true);
  assert.equal(result.reinstallSource, 'bundled/default');
});
