/** Shared renderer geometry inherited from the accepted r013 relationships. */
export const SHELL_GEOMETRY = {
  headerHeight: 58,
  projectPanelInitialWidth: 228,
  projectPanelMinimumWidth: 180,
  centerMinimumWidth: 420,
  projectPanelOverlayBreakpoint: 992,
  contextualChatContainedBreakpoint: 860,
  settingsNavigationWidth: 224,
  settingsCompressedBreakpoint: 680,
  settingsContentMaximumWidth: 920,
  composerMaximumWidth: 760,
  launchStackBreakpoint: 620,
  dialogRegularWidth: 620,
  dialogCompactWidth: 480,
} as const;

export const {
  centerMinimumWidth: SHELL_CENTER_MINIMUM_WIDTH,
  projectPanelInitialWidth: SHELL_PROJECT_PANEL_INITIAL_WIDTH,
  projectPanelMinimumWidth: SHELL_PROJECT_PANEL_MINIMUM_WIDTH,
  projectPanelOverlayBreakpoint: SHELL_PROJECT_PANEL_OVERLAY_BREAKPOINT,
} = SHELL_GEOMETRY;
