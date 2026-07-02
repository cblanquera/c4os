export function classifyMigrationFailures() {
  return {
    cacheFailure: classify('cache-failure'),
    corruptUserConfig: classify('corrupt-user-config'),
    unsupportedSchema: classify('unsupported-schema'),
    missingDependency: classify('missing-dependency'),
    securityViolation: classify('security-violation')
  };
}

function classify(kind) {
  const table = {
    'cache-failure': { action: 'auto-recover-cache', state: 'previous-state' },
    'corrupt-user-config': {
      action: 'reset-plugin-owned-config',
      state: 'enabled-with-reset-config-notice'
    },
    'unsupported-schema': {
      action: 'disable',
      state: 'disabled-with-reason'
    },
    'missing-dependency': {
      action: 'block-enable',
      state: 'dependency-blocked'
    },
    'security-violation': {
      action: 'disable',
      state: 'disabled-with-security-reason'
    }
  };
  return table[kind];
}
