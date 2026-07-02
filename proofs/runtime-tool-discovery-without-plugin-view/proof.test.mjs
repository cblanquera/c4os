import test from 'node:test';
import assert from 'node:assert/strict';
import { runRuntimeDiscoveryProof } from './proof.mjs';

test('runtime invokes view-oriented registered tools without plugin views and stores app-owned state', () => {
  const result = runRuntimeDiscoveryProof();

  assert.equal(result.pluginViewsAtInvocation, 0);
  assert.deepEqual(result.discoveredTools, ['browser.open', 'files.read']);
  assert.equal(result.executedTool, 'browser.open');
  assert.equal(result.appOwnedState.owner, 'c4os');
  assert.deepEqual(result.hydratedViews, ['browser-a', 'browser-b']);
  assert.equal(result.sourceStateClaimedByPlugin, false);
  assert.equal(result.sourceStateMutatedByPlugin, false);
});
