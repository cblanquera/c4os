import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import test from 'node:test';

import { resolveRequestPath } from '../scripts/dev-server.mjs';

test('dev server root route resolves to index.html', () => {
  const filePath = resolveRequestPath('/');

  assert.equal(filePath.endsWith('/index.html'), true);
  assert.equal(existsSync(filePath), true);
});

test('dev server blocks path traversal outside the project root', () => {
  assert.equal(resolveRequestPath('/../package.json'), null);
  assert.equal(resolveRequestPath('/%2e%2e/package.json'), null);
});
