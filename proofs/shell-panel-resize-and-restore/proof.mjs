export function createShellStateMachine({ viewportWidth, centerMinWidth, plugins }) {
  const sessions = new Map();
  let activeSession = 'chat-a';
  let route = 'chat';
  let settingsRestore = null;
  const widths = {};

  for (const [id, plugin] of Object.entries(plugins)) {
    widths[id] = plugin.defaultWidth;
  }

  const stateFor = (sessionId) => {
    if (!sessions.has(sessionId)) {
      sessions.set(sessionId, { left: null, right: null });
    }
    return sessions.get(sessionId);
  };

  const activeState = () => stateFor(activeSession);

  const closeForCenterMinimum = (resizedSide) => {
    const state = activeState();
    const opposite = resizedSide === 'left' ? 'right' : 'left';
    const resizedPlugin = state[resizedSide];
    const oppositePlugin = state[opposite];
    if (!resizedPlugin || !oppositePlugin) return null;

    const centerAfterResize = () => {
      const leftWidth = state.left ? widths[state.left] : 0;
      const rightWidth = state.right ? widths[state.right] : 0;
      return viewportWidth - leftWidth - rightWidth;
    };

    if (centerAfterResize() >= centerMinWidth) return null;
    state[opposite] = null;
    return opposite;
  };

  return {
    clickPlugin(pluginId) {
      const plugin = plugins[pluginId];
      const state = activeState();
      const side = plugin.side;
      state[side] = state[side] === pluginId ? null : pluginId;
      return this.visiblePanels();
    },
    enterSettings() {
      settingsRestore = { session: activeSession, panels: { ...activeState() } };
      const state = activeState();
      state.left = null;
      state.right = null;
      route = 'settings';
    },
    leaveSettings() {
      if (settingsRestore?.session === activeSession) {
        sessions.set(activeSession, { ...settingsRestore.panels });
      }
      settingsRestore = null;
      route = 'chat';
    },
    switchSession(sessionId) {
      activeSession = sessionId;
      stateFor(activeSession);
      route = 'chat';
    },
    resizePanel(side, requestedWidth) {
      const state = activeState();
      const pluginId = state[side];
      if (!pluginId) return { width: 0, closedOppositePanel: null };

      widths[pluginId] = requestedWidth;
      const opposite = side === 'left' ? 'right' : 'left';
      const closedSide = closeForCenterMinimum(side);
      const otherWidth = state[opposite] ? widths[state[opposite]] : 0;
      const max = viewportWidth - otherWidth - centerMinWidth;
      widths[pluginId] = Math.max(0, Math.min(requestedWidth, max));

      return {
        width: widths[pluginId],
        closedOppositePanel: closedSide
      };
    },
    visiblePanels() {
      return { ...activeState() };
    },
    panelWidth(side) {
      const pluginId = activeState()[side];
      return pluginId ? widths[pluginId] : 0;
    },
    centerWidth() {
      return viewportWidth - this.panelWidth('left') - this.panelWidth('right');
    },
    currentRoute() {
      return route;
    }
  };
}
