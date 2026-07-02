import test from 'node:test';
import assert from 'node:assert/strict';
import { buildChatDebugHistory } from './proof.mjs';

test('Chat Debug stores typed redacted active and historical events without export', () => {
  const result = buildChatDebugHistory();

  assert.deepEqual(result.visibleRuns.map((run) => run.id), ['run-003', 'run-002']);
  assert.equal(result.prunedRunIds.includes('run-001'), true);
  assert.equal(result.exportAvailable, false);
  assert.deepEqual(result.approvalSurfaces, ['thread-context', 'chat-debug']);
  assert.deepEqual(result.events.map((event) => event.kind), [
    'runtime.tool.requested',
    'approval.decision',
    'plugin.setting.changed',
    'terminal.tool.completed',
    'attachment.adapted',
    'structured.error'
  ]);
  assert.equal(result.persistedJson.includes('sk-live-provider-key'), false);
  assert.equal(result.persistedJson.includes('Bearer secret-token'), false);
  assert.equal(result.persistedJson.includes('raw-cookie-value'), false);
  assert.equal(result.persistedJson.includes('database-password'), false);
  assert.equal(result.displayRows.every((row) => row.summary.length > 0), true);
});
