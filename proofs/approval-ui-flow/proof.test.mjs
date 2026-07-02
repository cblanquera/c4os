import test from 'node:test';
import assert from 'node:assert/strict';
import { runApprovalUiFlow } from './proof.mjs';

test('approval UI emits typed decisions, remember summaries, and Settings policy routing', () => {
  const result = runApprovalUiFlow();

  assert.deepEqual(result.allowEvent, {
    type: 'approval_decision',
    requestId: 'req-allow',
    decision: 'allow',
    remember: null,
    resume: true
  });
  assert.deepEqual(result.denyEvent, {
    type: 'approval_decision',
    requestId: 'req-deny',
    decision: 'deny',
    remember: null,
    resume: false
  });
  assert.deepEqual(result.rememberedEvent, {
    type: 'approval_decision',
    requestId: 'req-remember',
    decision: 'allow',
    remember: 'user-global',
    resume: true
  });
  assert.deepEqual(result.rememberedSummary, {
    tool: 'terminal.run',
    action: 'terminal',
    targetScope: 'trusted-project:/repo/scripts',
    pluginId: 'terminal-plugin',
    duration: 'user-global'
  });
  assert.deepEqual(result.threadContextSummaries, [result.rememberedSummary]);
  assert.deepEqual(result.chatDebugSummaries, [result.rememberedSummary]);
  assert.deepEqual(result.settingsConfigurationRoute, {
    path: 'Settings > Configuration',
    items: [
      {
        toolId: 'terminal.run',
        title: 'Run terminal command',
        controls: ['review', 'edit', 'revoke']
      }
    ]
  });
});
