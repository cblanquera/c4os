import test from 'node:test';
import assert from 'node:assert/strict';
import { runBrowserHydrationScenario } from './proof.mjs';

test('Browser tool results created without a visible view hydrate compatible views without claiming source state', () => {
  const result = runBrowserHydrationScenario();

  assert.equal(result.backendCallCount, 1);
  assert.equal(result.initialVisibleBrowserViews, 0);
  assert.equal(result.appOwnedState.owner, 'c4os-app');
  assert.equal(result.appOwnedState.chatId, 'chat-1');
  assert.equal(result.appOwnedState.latestNavigation.url, 'https://example.test/research');
  assert.deepEqual(result.openedViews.map((view) => view.hydratedFromStateId), ['browser-state-1', 'browser-state-1']);
  assert.notDeepEqual(result.openedViews[0].viewLocalState, result.openedViews[1].viewLocalState);
  assert.equal(result.appOwnedState.claimedByViewId, null);
  assert.equal(result.events.every((event) => event.parsedFromProse === false), true);
});
