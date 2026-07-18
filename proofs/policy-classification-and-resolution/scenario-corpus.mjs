const legacyGroups = {
  terminal: [
    'terminal.project.read', 'terminal.project.write', 'terminal.external.read',
    'terminal.external.write', 'terminal.network.read', 'terminal.network.write',
    'terminal.unknown',
  ],
  git: [
    'git.project.read', 'git.external.read', 'git.remote.read', 'git.project.write',
    'git.external.write', 'git.remote.write', 'git.unknown',
  ],
  filesystem: [
    'file.project.read', 'file.external.read', 'file.project.write',
    'file.external.write', 'file.project.delete', 'file.external.delete',
    'file.project.watch', 'file.external.watch', 'file.unknown',
  ],
  browser: [
    'browser.load', 'browser.post', 'browser.navigation', 'browser.use',
    'browser.read', 'browser.capture', 'browser.download', 'browser.upload',
    'browser.local.read', 'browser.auth.use', 'browser.unknown',
  ],
  network: [
    'network.read', 'network.write', 'network.listen', 'network.connect',
    'network.unknown',
  ],
  credentials: [
    'credential.use', 'credential.create', 'credential.update',
    'credential.delete', 'credential.reveal', 'credential.unknown',
  ],
  processes: [
    'process.read', 'process.start', 'process.control', 'process.stop',
    'process.unknown',
  ],
  desktop: [
    'clipboard.read', 'clipboard.write', 'dialog.open', 'dialog.save',
    'notification.show', 'system.read', 'system.write', 'app.open', 'app.control',
  ],
  c4os: [
    'config.read', 'config.write', 'plugin.read', 'plugin.write', 'extension.use',
    'mcp.read', 'mcp.use', 'mcp.write', 'artifact.read', 'artifact.write',
    'share.export', 'unknown',
  ],
};

const aliases = {
  project: 'workspace',
  external: 'external-local',
  remote: 'remote',
};

const effectAliases = {
  load: 'read', navigation: 'read', use: 'control', post: 'publish',
  upload: 'publish', download: 'create', watch: 'read', connect: 'control',
  start: 'execute', stop: 'control', show: 'create', open: 'control',
  export: 'publish', create: 'create', update: 'modify', write: 'modify',
};

const surfaceAliases = {
  clipboard: 'desktop', dialog: 'desktop', notification: 'desktop',
  system: 'desktop', app: 'desktop', config: 'c4os', plugin: 'c4os',
  extension: 'c4os', mcp: 'c4os', artifact: 'c4os', share: 'c4os',
};

export const legacyScenarioKeys = Object.values(legacyGroups).flat();

export function legacyScenarioToIntent(key) {
  const parts = key.split('.');
  if (key === 'unknown' || parts.includes('unknown')) {
    return {
      legacyKey: key,
      surface: surfaceAliases[parts[0]] ?? parts[0] ?? 'c4os',
      effects: ['unknown'],
      scope: aliases[parts[1]] ?? 'unknown',
      initiator: 'agent',
      sensitivity: 'ordinary',
      reversibility: 'unknown',
      confidence: 'ambiguous',
    };
  }

  let surface = surfaceAliases[parts[0]] ?? parts[0];
  let scope = parts.map((part) => aliases[part]).find(Boolean) ?? 'workspace';
  let operation = parts.at(-1);
  let effect = effectAliases[operation] ?? operation;
  let sensitivity = 'ordinary';
  let reversibility = ['delete', 'stop'].includes(operation) ? 'destructive' : 'reversible';

  if (parts.includes('network')) scope = 'remote';
  if (surface === 'browser' && !parts.includes('local')) scope = 'remote';
  if (surface === 'network' || key === 'share.export') scope = 'remote';
  if (parts.includes('auth')) sensitivity = 'authenticated';
  if (surface === 'credential') sensitivity = 'credential';
  if (key === 'credential.reveal') effect = 'reveal';
  if (key === 'network.listen') scope = 'external-local';
  if (key === 'system.write') scope = 'system';
  if (key === 'browser.capture') effect = 'capture';
  if (key === 'browser.local.read') scope = 'external-local';
  if (key === 'git.remote.write') effect = 'publish';
  if (key === 'mcp.write') effect = 'modify';

  return {
    legacyKey: key,
    surface,
    effects: [effect],
    scope,
    initiator: 'agent',
    sensitivity,
    reversibility,
    confidence: 'known',
  };
}

export const scenarioCorpus = legacyScenarioKeys.map(legacyScenarioToIntent);

export const legacyScenarioGroups = Object.freeze(
  Object.fromEntries(Object.entries(legacyGroups).map(([group, keys]) => [group, [...keys]])),
);
