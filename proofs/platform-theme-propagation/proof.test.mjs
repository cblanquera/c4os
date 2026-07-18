import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const html = await readFile(new URL('./ui/index.html', import.meta.url), 'utf8');
const css = await readFile(new URL('./ui/styles.css', import.meta.url), 'utf8');
const js = await readFile(new URL('./ui/proof.js', import.meta.url), 'utf8');

test('declares color scheme before loading the stylesheet', () => {
  assert.ok(html.indexOf('name="color-scheme"') < html.indexOf('rel="stylesheet"'));
  assert.ok(html.indexOf('dataset.colorScheme') < html.indexOf('rel="stylesheet"'));
});

test('keeps native and webview signals independent', () => {
  assert.match(js, /theme_snapshot/);
  assert.match(js, /prefers-color-scheme/);
  assert.match(js, /native-theme-changed/);
});

test('contains explicit semantic and reduced-motion fallbacks', () => {
  assert.match(css, /CanvasText/);
  assert.match(css, /AccentColor/);
  assert.match(css, /prefers-reduced-motion: reduce/);
});
