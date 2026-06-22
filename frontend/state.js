import {
  activateSessionState,
  connectorState,
  threadTurns,
  toolState,
  workspace
} from "./data.js";

/**
 * Tiny app-owned store facade for the no-build frontend. The imported records
 * stay mutable for now, but route, session, tool, run, turn, and composer
 * ownership is explicit before later slices add more real data.
 */
export const appStore = {
  composer: {
    bySurface: {}
  },
  connectorRun: connectorState,
  route: "app-start",
  shell: {
    leftCollapsed: false,
    rightCollapsed: false,
    leftWidth: null,
    rightWidth: null,
    terminalBottom: null
  },
  tools: toolState,
  turns: threadTurns,
  workspace,
  activeSessionKey() {
    return sessionSurfaceKey(workspace.project, workspace.session || "untitled");
  },
  setRoute(route) {
    this.route = route;
    return this.route;
  },
  setTool(surface, tool) {
    this.tools.bySurface[surface] = tool;
    return tool;
  },
  setFileView(surface, view) {
    this.tools.fileViewBySurface ||= {};
    this.tools.fileViewBySurface[surface] = view;
    return view;
  },
  setActiveSession(project, session) {
    return activateSessionState(project, session);
  },
  composerFor(surface) {
    this.composer.bySurface[surface] ||= {
      approval: "Approve for me",
      attachments: [],
      branch: workspace.branch,
      openPicker: null,
      prompt: ""
    };
    return this.composer.bySurface[surface];
  },
  setComposerValue(surface, key, value) {
    const composer = this.composerFor(surface);
    composer[key] = value;
    return composer;
  },
  setShellValue(key, value) {
    this.shell[key] = value;
    return value;
  },
  fileViewFor(surface, fallback) {
    return this.tools.fileViewBySurface?.[surface] || fallback;
  },
  toolFor(surface, fallback) {
    return this.tools.bySurface[surface] || fallback;
  },
  turnById(turnId) {
    return this.turns.find((turn) => turn.id === turnId);
  }
};

export function sessionSurfaceKey(project, session) {
  return `chat:${project}:${session}`;
}
