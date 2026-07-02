export function renderAndPersistSettings() {
  const schema = {
    username: { field: 'string', default: 'ada' },
    prompt: { field: 'boolean', default: false },
    panel: { field: ['left', 'right'], default: 'left' },
    apiKey: { field: 'string', sensitive: true },
    advancedNote: { field: 'text', visibleWhen: { key: 'prompt', equals: true } },
    unknownWidget: { field: 'color-wheel' },
    enabled: { field: 'string' }
  };
  const input = {
    username: 'ada',
    prompt: false,
    panel: 'right',
    apiKey: 'sk-live-demo',
    advancedNote: 'hidden until prompt=true'
  };
  const secretStore = {};
  const config = { plugin: {} };
  const validationWarnings = [];
  const reservedKeyErrors = [];

  for (const [key, field] of Object.entries(schema)) {
    if (key === 'unknownWidget') {
      validationWarnings.push('ignored unknown key: unknownWidget');
      continue;
    }
    if (key === 'enabled' && field.field !== 'boolean') {
      reservedKeyErrors.push('enabled must be boolean');
      continue;
    }
    if (field.sensitive) {
      secretStore[`demo-plugin:${key}`] = input[key];
      config.plugin[key] = `secret://demo-plugin/${key}`;
      continue;
    }
    config.plugin[key] = input[key] ?? field.default;
  }

  return {
    renderedFields: ['username', 'prompt', 'panel', 'apiKey', 'advancedNote'],
    visibleFields: ['username', 'prompt', 'panel', 'apiKey'],
    config,
    secretStore,
    displayValues: { apiKey: '********' },
    validationWarnings,
    reservedKeyErrors
  };
}
