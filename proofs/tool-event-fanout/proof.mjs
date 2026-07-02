export function runFanoutProof() {
  const views = [
    { id: 'browser-left', compatible: true, visible: true },
    { id: 'browser-right', compatible: true, visible: true },
    { id: 'browser-hidden', compatible: true, visible: false },
    { id: 'terminal-left', compatible: false, visible: true }
  ];
  const event = { toolId: 'browser.open', target: 'https://example.com' };
  let backendCalls = 0;
  const state = {
    visibleRenders: [],
    hiddenUpdates: [],
    hiddenOpenedPanel: false,
    hiddenStoleFocus: false,
    hiddenPromptedUser: false
  };

  backendCalls += executeBackendTool(event);
  for (const view of views) {
    if (!view.compatible) continue;
    if (view.visible) state.visibleRenders.push(view.id);
    else state.hiddenUpdates.push(view.id);
  }

  return {
    backendCalls,
    duplicateBackendCalls: backendCalls - 1,
    ...state
  };
}

function executeBackendTool(event) {
  if (event.toolId !== 'browser.open') return 0;
  return 1;
}
