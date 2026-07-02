import test from 'node:test';
import assert from 'node:assert/strict';
import { sanitizePluginIcon } from './proof.mjs';

test('unsafe svg content is blocked and fallback icon is selected', () => {
  const result = sanitizePluginIcon();

  assert.equal(result.safeIcon.accepted, true);
  assert.equal(result.unsafeIcon.accepted, false);
  assert.equal(result.unsafeIcon.fallback, 'default-plugin-icon');
  assert.deepEqual(result.unsafeIcon.blockedReasons, ['script element', 'event handler attribute', 'external href']);
});
