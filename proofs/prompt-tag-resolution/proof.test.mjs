import test from 'node:test';
import assert from 'node:assert/strict';
import { resolvePromptInteractions } from './proof.mjs';

test('prompt tokens route through backend authority and hide disabled resources', () => {
  const result = resolvePromptInteractions('Use $summarizer on @notes and /browser.open @blocked-doc');

  assert.deepEqual(result.displayTokens, [
    { raw: '$summarizer', kind: 'skill' },
    { raw: '@notes', kind: 'resource' },
    { raw: '/browser.open', kind: 'command' },
    { raw: '@blocked-doc', kind: 'resource' }
  ]);
  assert.deepEqual(result.backendResolutionEvents, [
    { type: 'resolve_skill', value: 'summarizer' },
    { type: 'resolve_resource', value: 'notes' },
    { type: 'resolve_command', value: 'browser.open' },
    { type: 'resolve_resource', value: 'blocked-doc' }
  ]);
  assert.deepEqual(result.resolved.skills, [{ id: 'summarizer', source: 'skills' }]);
  assert.deepEqual(result.resolved.resources, [
    { id: 'notes', source: 'files', pluginId: 'files-plugin', inline: '@notes' }
  ]);
  assert.deepEqual(result.resolved.commands, [
    { id: 'browser.open', route: 'runtime/tool-gateway' }
  ]);
  assert.deepEqual(result.hiddenResources, [
    { id: 'blocked-doc', reason: 'dependency-blocked', repairRoute: 'Settings > Plugins' }
  ]);
  assert.equal(result.frontendCanExecuteDisabledResource, false);
});
