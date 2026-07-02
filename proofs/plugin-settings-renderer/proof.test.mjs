import test from 'node:test';
import assert from 'node:assert/strict';
import { renderAndPersistSettings } from './proof.mjs';

test('plugin settings render, persist, redact sensitive values, and validate shell keys', () => {
  const result = renderAndPersistSettings();

  assert.deepEqual(result.renderedFields, ['username', 'prompt', 'panel', 'apiKey', 'advancedNote']);
  assert.equal(result.visibleFields.includes('advancedNote'), false);
  assert.equal(result.config.plugin.apiKey, 'secret://demo-plugin/apiKey');
  assert.equal(result.secretStore['demo-plugin:apiKey'], 'sk-live-demo');
  assert.equal(result.displayValues.apiKey, '********');
  assert.deepEqual(result.validationWarnings, ['ignored unknown key: unknownWidget']);
  assert.deepEqual(result.reservedKeyErrors, ['enabled must be boolean']);
});
