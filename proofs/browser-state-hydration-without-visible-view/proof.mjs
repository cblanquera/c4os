export function runBrowserHydrationScenario() {
  const initialVisibleBrowserViews = 0;
  const appOwnedState = {
    id: 'browser-state-1',
    owner: 'c4os-app',
    chatId: 'chat-1',
    claimedByViewId: null,
    latestNavigation: {
      url: 'https://example.test/research',
      title: 'Research Result',
      status: 200
    },
    actions: [
      {
        id: 'action-1',
        toolId: 'browser.open',
        requestedBy: 'runtime-tool-gateway',
        result: 'stored-without-visible-view'
      }
    ]
  };

  const openedViews = [
    hydrateView('browser-view-left', appOwnedState, { scrollY: 120, selectedMarker: null }),
    hydrateView('browser-view-right', appOwnedState, { scrollY: 0, selectedMarker: 2 })
  ];

  return {
    backendCallCount: 1,
    initialVisibleBrowserViews,
    appOwnedState,
    openedViews,
    events: [
      {
        kind: 'browser.navigation.completed',
        stateId: appOwnedState.id,
        visibleViewId: null,
        parsedFromProse: false
      },
      ...openedViews.map((view) => ({
        kind: 'browser.view.hydrated',
        viewId: view.id,
        stateId: view.hydratedFromStateId,
        parsedFromProse: false
      }))
    ]
  };
}

function hydrateView(id, state, viewLocalState) {
  return {
    id,
    hydratedFromStateId: state.id,
    sourceStateOwner: state.owner,
    latestNavigation: state.latestNavigation,
    viewLocalState
  };
}
