export function runLifecycleProof() {
  const states = [];
  const plugin = {
    id: 'builtin-browser',
    source: 'bundled/default',
    state: 'installed-disabled'
  };

  states.push(plugin.state);
  plugin.state = 'enabled';
  states.push(plugin.state);
  states.push('uninstall-prompt-delete-data');
  plugin.state = 'uninstalled';
  states.push(plugin.state);
  plugin.state = 'reinstalled-from-bundled-default';
  states.push(plugin.state);
  plugin.state = 'installed-disabled';
  states.push(plugin.state);

  return {
    states,
    dataDeletePromptShown: true,
    reinstallSource: plugin.source
  };
}
