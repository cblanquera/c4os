import test from 'node:test';
import assert from 'node:assert/strict';
import { classifyMigrationFailures } from './proof.mjs';

test('migration failures route to auto recovery, reset, blocked, or visible disabled states', () => {
  const result = classifyMigrationFailures();

  assert.equal(result.cacheFailure.state, 'previous-state');
  assert.equal(result.cacheFailure.action, 'auto-recover-cache');
  assert.equal(result.corruptUserConfig.state, 'enabled-with-reset-config-notice');
  assert.equal(result.unsupportedSchema.state, 'disabled-with-reason');
  assert.equal(result.missingDependency.state, 'dependency-blocked');
  assert.equal(result.securityViolation.state, 'disabled-with-security-reason');
});
