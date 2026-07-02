export function scanSkillsSettings() {
  const settingsRows = [
    skill('skill-creator', 'bundled', {
      valid: true,
      enabled: true,
      editable: false,
      description: 'Create Codex-compatible skills.'
    }),
    skill('skill-creator', 'user-global', {
      valid: true,
      enabled: false,
      editable: true,
      description: 'Customized skill creator copy.',
      overridesBundledByExplicitChoice: true
    }),
    skill('sales-brief', 'user-global', {
      valid: true,
      enabled: true,
      editable: true,
      description: 'Prepare reusable sales workflow briefs.'
    }),
    skill('plugin-research', 'plugin-provided', {
      valid: true,
      enabled: false,
      parentPlugin: 'research-plugin',
      repairReason: 'parent plugin disabled'
    }),
    skill('missing-skill-md', 'user-global', {
      valid: false,
      repairReason: 'missing SKILL.md'
    }),
    skill('bad-frontmatter', 'user-global', {
      valid: false,
      repairReason: 'invalid frontmatter'
    }),
    skill('missing-description', 'user-global', {
      valid: false,
      repairReason: 'missing name or description'
    }),
    skill('duplicate-name', 'user-global', {
      valid: false,
      repairReason: 'duplicate name'
    }),
    skill('unreadable-skill', 'user-global', {
      valid: false,
      repairReason: 'unreadable'
    }),
    skill('source-unavailable', 'user-global', {
      valid: false,
      repairReason: 'source unavailable'
    }),
    skill('project-local-skill', 'project-local', {
      valid: true,
      enabled: false,
      repairReason: 'requires FS plugin authority before promotion'
    })
  ];

  const eligible = settingsRows.filter((row) => {
    if (!row.valid || !row.enabled) return false;
    if (row.source === 'bundled') return false;
    if (row.source === 'plugin-provided' && row.parentPluginDisabled) return false;
    if (row.source === 'project-local') return false;
    return true;
  });

  return {
    settingsRows,
    customization: {
      from: 'bundled',
      source: 'user-global',
      editable: true,
      overridesBundledByExplicitChoice: true
    },
    dollarSuggestions: eligible.map((row) => row.name),
    runtimeContextSkills: eligible.map((row) => row.name),
    fullInstructionsLoadedDuringScan: false
  };
}

function skill(name, source, options = {}) {
  const parentPluginDisabled = options.source === 'plugin-provided' || Boolean(options.parentPlugin);
  return {
    name,
    source,
    description: options.description ?? '',
    status: options.enabled ? 'enabled' : 'disabled-or-invalid',
    valid: options.valid ?? true,
    enabled: options.enabled ?? false,
    editable: options.editable ?? false,
    parentPlugin: options.parentPlugin ?? null,
    parentPluginDisabled,
    repairReason: options.repairReason ?? null,
    visibleInSettings: true,
    visibleInDollarSuggestions: false,
    loadedIntoRuntimeContext: false,
    overridesBundledByExplicitChoice: options.overridesBundledByExplicitChoice ?? false
  };
}
