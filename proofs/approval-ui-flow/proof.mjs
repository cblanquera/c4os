export function runApprovalUiFlow() {
  const request = {
    toolId: 'terminal.run',
    title: 'Run terminal command',
    action: 'terminal',
    targetScope: 'trusted-project:/repo/scripts',
    pluginId: 'terminal-plugin'
  };

  const decide = (requestId, decision, remember = null) => ({
    type: 'approval_decision',
    requestId,
    decision,
    remember,
    resume: decision === 'allow'
  });

  const rememberedSummary = {
    tool: request.toolId,
    action: request.action,
    targetScope: request.targetScope,
    pluginId: request.pluginId,
    duration: 'user-global'
  };

  return {
    allowEvent: decide('req-allow', 'allow'),
    denyEvent: decide('req-deny', 'deny'),
    rememberedEvent: decide('req-remember', 'allow', 'user-global'),
    rememberedSummary,
    threadContextSummaries: [rememberedSummary],
    chatDebugSummaries: [rememberedSummary],
    settingsConfigurationRoute: {
      path: 'Settings > Configuration',
      items: [
        {
          toolId: request.toolId,
          title: request.title,
          controls: ['review', 'edit', 'revoke']
        }
      ]
    }
  };
}
