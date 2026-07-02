import test from 'node:test';
import assert from 'node:assert/strict';
import { createShellStateMachine } from './proof.mjs';

test('shell panels toggle, replace per side, restore per session, and preserve center minimum', () => {
  const shell = createShellStateMachine({
    viewportWidth: 1440,
    centerMinWidth: 640,
    plugins: {
      files: { side: 'left', defaultWidth: 320 },
      search: { side: 'left', defaultWidth: 300 },
      browser: { side: 'right', defaultWidth: 420 }
    }
  });

  assert.deepEqual(shell.visiblePanels(), { left: null, right: null });

  shell.clickPlugin('files');
  shell.clickPlugin('browser');
  assert.deepEqual(shell.visiblePanels(), { left: 'files', right: 'browser' });

  shell.clickPlugin('search');
  assert.deepEqual(shell.visiblePanels(), { left: 'search', right: 'browser' });

  shell.enterSettings();
  assert.deepEqual(shell.visiblePanels(), { left: null, right: null });
  assert.equal(shell.currentRoute(), 'settings');

  shell.leaveSettings();
  assert.deepEqual(shell.visiblePanels(), { left: 'search', right: 'browser' });

  shell.switchSession('chat-b');
  assert.deepEqual(shell.visiblePanels(), { left: null, right: null });
  shell.clickPlugin('browser');
  assert.deepEqual(shell.visiblePanels(), { left: null, right: 'browser' });

  shell.switchSession('chat-a');
  assert.deepEqual(shell.visiblePanels(), { left: 'search', right: 'browser' });

  const resizeResult = shell.resizePanel('right', 900);
  assert.equal(resizeResult.closedOppositePanel, 'left');
  assert.equal(shell.visiblePanels().left, null);
  assert.equal(shell.centerWidth() >= 640, true);
  assert.equal(shell.panelWidth('right'), 800);
});
