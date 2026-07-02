import test from 'node:test';
import assert from 'node:assert/strict';
import { scanSkillsSettings } from './proof.mjs';

test('Skills Settings lists sources and invalid states while filtering runtime and dollar suggestions', () => {
  const result = scanSkillsSettings();

  assert.deepEqual(result.settingsRows.map((row) => row.name), [
    'skill-creator',
    'skill-creator',
    'sales-brief',
    'plugin-research',
    'missing-skill-md',
    'bad-frontmatter',
    'missing-description',
    'duplicate-name',
    'unreadable-skill',
    'source-unavailable',
    'project-local-skill'
  ]);
  assert.equal(result.settingsRows.find((row) => row.name === 'skill-creator' && row.source === 'bundled').editable, false);
  assert.equal(result.customization.source, 'user-global');
  assert.equal(result.customization.overridesBundledByExplicitChoice, true);
  assert.deepEqual(result.dollarSuggestions, ['sales-brief']);
  assert.deepEqual(result.runtimeContextSkills, ['sales-brief']);
  assert.equal(result.settingsRows.filter((row) => row.visibleInSettings && row.valid === false).length, 6);
  assert.equal(result.settingsRows.find((row) => row.name === 'project-local-skill').repairReason, 'requires FS plugin authority before promotion');
  assert.equal(result.fullInstructionsLoadedDuringScan, false);
});
