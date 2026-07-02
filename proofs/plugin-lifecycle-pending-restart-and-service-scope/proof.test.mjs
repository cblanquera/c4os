import test from 'node:test';
import assert from 'node:assert/strict';
import { runLifecycleScopeProof } from './proof.mjs';

test('plugin lifecycle separates live UI changes from restart-gated backend tools and shared services', () => {
  const result = runLifecycleScopeProof();

  assert.equal(result.uiSettingAppliedLive, true);
  assert.equal(result.backendToolState, 'pending-restart');
  assert.equal(result.unavailableToolVisible, false);
  assert.equal(result.requiredDependencyState, 'dependency-blocked');
  assert.equal(result.optionalDependencyState, 'enabled-degraded');
  assert.deepEqual(result.serviceStarts, ['workspace:demo-service']);
  assert.equal(result.serviceSharedAcrossChats, true);
  assert.deepEqual(result.shutdownReasons, ['idle', 'disable', 'uninstall', 'app-exit']);
});
