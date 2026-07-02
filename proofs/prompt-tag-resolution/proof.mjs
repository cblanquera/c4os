const registry = {
  skills: new Map([
    ['summarizer', { id: 'summarizer', source: 'skills' }]
  ]),
  resources: new Map([
    ['notes', { id: 'notes', source: 'files', pluginId: 'files-plugin', inline: '@notes', state: 'enabled' }],
    ['blocked-doc', { id: 'blocked-doc', source: 'docs-plugin', pluginId: 'docs-plugin', state: 'dependency-blocked' }]
  ]),
  commands: new Map([
    ['browser.open', { id: 'browser.open', route: 'runtime/tool-gateway' }]
  ])
};

export function resolvePromptInteractions(prompt) {
  const displayTokens = [];
  const backendResolutionEvents = [];
  const resolved = { skills: [], resources: [], commands: [] };
  const hiddenResources = [];

  for (const raw of prompt.match(/[$@/][\w.-]+/g) || []) {
    const prefix = raw[0];
    const value = raw.slice(1);
    if (prefix === '$') {
      displayTokens.push({ raw, kind: 'skill' });
      backendResolutionEvents.push({ type: 'resolve_skill', value });
      const skill = registry.skills.get(value);
      if (skill) resolved.skills.push(skill);
    } else if (prefix === '@') {
      displayTokens.push({ raw, kind: 'resource' });
      backendResolutionEvents.push({ type: 'resolve_resource', value });
      const resource = registry.resources.get(value);
      if (resource?.state === 'enabled') {
        resolved.resources.push(withoutState(resource));
      } else if (resource) {
        hiddenResources.push({
          id: resource.id,
          reason: resource.state,
          repairRoute: 'Settings > Plugins'
        });
      }
    } else if (prefix === '/') {
      displayTokens.push({ raw, kind: 'command' });
      backendResolutionEvents.push({ type: 'resolve_command', value });
      const command = registry.commands.get(value);
      if (command) resolved.commands.push(command);
    }
  }

  return {
    displayTokens,
    backendResolutionEvents,
    resolved,
    hiddenResources,
    frontendCanExecuteDisabledResource: false
  };
}

function withoutState(record) {
  const { state, ...publicRecord } = record;
  return publicRecord;
}
