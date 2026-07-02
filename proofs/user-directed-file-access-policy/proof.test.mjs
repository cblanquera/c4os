import test from 'node:test';
import assert from 'node:assert/strict';
import { evaluateFileAccessPolicy } from './proof.mjs';

test('file access policy distinguishes user-directed reads from agent-initiated outside-project reads', () => {
  const result = evaluateFileAccessPolicy();

  assert.equal(result.userDirectedOutsideRead, 'allow');
  assert.equal(result.agentInitiatedOutsideRead, 'ask');
  assert.equal(result.trustedProjectWrite, 'allow');
  assert.equal(result.destructiveTrustedProjectDelete, 'ask');
  assert.equal(result.outsideProjectWriteWithoutExplicitRequest, 'ask');
  assert.equal(result.outsideProjectWriteWithExplicitRequest, 'allow');
});
