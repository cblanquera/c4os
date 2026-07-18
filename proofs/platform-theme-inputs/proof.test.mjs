import assert from 'node:assert/strict';
import test from 'node:test';
import { fallbackTokens, tokenChecks } from './ui/tokens.mjs';

for (const scheme of ['light', 'dark']) {
  test(`${scheme} fallback text pairs meet WCAG AA`, () => {
    for (const [name, ratio, required] of tokenChecks(scheme)) {
      assert.ok(ratio >= required, `${name} is ${ratio.toFixed(2)}:1; expected ${required}:1`);
    }
  });
  test(`${scheme} exposes the complete bounded fallback set`, () => {
    assert.deepEqual(Object.keys(fallbackTokens[scheme]), ['surface', 'text', 'secondary', 'border', 'accent', 'accentText', 'danger']);
  });
}
