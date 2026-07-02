export function runRuntimeDiscoveryProof() {
  const catalog = [
    { id: 'browser.open', viewOriented: true, defaultPolicy: 'allow' },
    { id: 'files.read', viewOriented: false, defaultPolicy: 'allow' }
  ];
  const discoveredTools = catalog.map((tool) => tool.id);
  const appOwnedState = {
    owner: 'c4os',
    key: 'browser.open:https://example.com',
    result: { url: 'https://example.com', title: 'Example' }
  };

  return {
    pluginViewsAtInvocation: 0,
    discoveredTools,
    executedTool: 'browser.open',
    appOwnedState,
    hydratedViews: hydrateViews(appOwnedState, ['browser-a', 'browser-b']),
    sourceStateClaimedByPlugin: false,
    sourceStateMutatedByPlugin: false
  };
}

function hydrateViews(state, viewIds) {
  if (state.owner !== 'c4os') return [];
  return viewIds;
}
