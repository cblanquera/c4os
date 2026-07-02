export function runRememberPolicyProof() {
  const ruleKey = {
    toolId: 'terminal.run',
    risk: 'terminal',
    targetScope: 'trusted-project:/repo',
    pluginId: 'terminal-plugin'
  };
  const sessionRules = new Map([[JSON.stringify(ruleKey), { decision: 'allow' }]]);
  const globalRules = new Map();
  const persistedKey = JSON.stringify({ ...ruleKey, targetScope: 'trusted-project:/repo/scripts' });
  globalRules.set(persistedKey, { decision: 'allow', duration: 'user-global' });
  sessionRules.clear();

  return {
    ruleKey,
    sessionRuleAfterSessionEnd: sessionRules.get(JSON.stringify(ruleKey)),
    userGlobalRulePersisted: globalRules.has(persistedKey),
    settingsItems: ['terminal.run', 'files.write', 'browser.open'],
    controls: ['review', 'edit', 'revoke']
  };
}
