import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { requiredFixtures, sourceChecks } from './ui/matrix.mjs';

const html = await readFile(new URL('./ui/index.html', import.meta.url), 'utf8');
const css = await readFile(new URL('./ui/styles.css', import.meta.url), 'utf8');
const script = await readFile(new URL('./ui/proof.js', import.meta.url), 'utf8');

test('covers the required representative surface matrix', () => {
  assert.deepEqual(requiredFixtures, ['shell', 'transcript', 'artifact', 'dialog', 'popover', 'editor', 'terminal', 'settings']);
});
for (const [name, passed] of sourceChecks({ html, css, script })) {
  test(name, () => assert.equal(passed, true));
}
