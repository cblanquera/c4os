import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const rust = await readFile(new URL('./src/main.rs', import.meta.url), 'utf8');
const config = JSON.parse(await readFile(new URL('./tauri.conf.json', import.meta.url), 'utf8'));
const html = await readFile(new URL('./ui/index.html', import.meta.url), 'utf8');

test('uses standard native decorations', () => {
  assert.equal(config.app.windows[0].decorations, true);
  assert.doesNotMatch(html, /data-tauri-drag-region/);
});
test('places Settings in the native application menu with platform shortcut', () => {
  assert.match(rust, /"settings", "Settings…"/);
  assert.match(rust, /CmdOrCtrl\+,/);
  assert.match(rust, /__C4OS_OPEN_SETTINGS__/);
});
test('keeps native Window and Help submenu identities', () => {
  assert.match(rust, /WINDOW_SUBMENU_ID/);
  assert.match(rust, /HELP_SUBMENU_ID/);
});
