import test from 'node:test';
import assert from 'node:assert/strict';
import { runRememberPolicyProof } from './proof.mjs';

test('remembered approval rules are narrowly keyed, expire or persist correctly, and surface per tool', () => {
  const result = runRememberPolicyProof();

  assert.deepEqual(result.ruleKey, {
    toolId: 'terminal.run',
    risk: 'terminal',
    targetScope: 'trusted-project:/repo',
    pluginId: 'terminal-plugin'
  });
  assert.equal(result.sessionRuleAfterSessionEnd, undefined);
  assert.equal(result.userGlobalRulePersisted, true);
  assert.deepEqual(result.settingsItems, ['terminal.run', 'files.write', 'browser.open']);
  assert.deepEqual(result.controls, ['review', 'edit', 'revoke']);
});
