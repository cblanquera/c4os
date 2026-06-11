//client
import {
  BACKEND_COMMANDS,
  createBackendCommandRegistry,
} from './commands.js';

/**
 * Creates UI-facing command invokers for Tauri or browser smoke mode.
 */
export function createAppCommandClient(options = {}) {
  const invoke = options.invoke ?? globalTauriInvoke();

  if (invoke) {
    return {
      getAppStatus() {
        return invoke(BACKEND_COMMANDS.getAppStatus, {});
      },
      configureOpenRouter(payload) {
        return invoke(BACKEND_COMMANDS.configureOpenRouter, payload);
      },
      registerProject(payload) {
        return invoke(BACKEND_COMMANDS.registerProject, payload);
      },
      listProjectSkills() {
        return invoke(BACKEND_COMMANDS.listProjectSkills, {});
      },
      invokeProjectSkill(payload) {
        return invoke(BACKEND_COMMANDS.invokeProjectSkill, payload);
      },
      listMcpServers() {
        return invoke(BACKEND_COMMANDS.listMcpServers, {});
      },
      registerLocalStdioMcpServer(payload) {
        return invoke(BACKEND_COMMANDS.registerLocalStdioMcpServer, payload);
      },
      proposeMcpServerLaunch(payload) {
        return invoke(BACKEND_COMMANDS.proposeMcpServerLaunch, payload);
      },
      exportProjectJson() {
        return invoke(BACKEND_COMMANDS.exportProjectJson, {});
      },
      deleteArchivedSession(payload) {
        return invoke(BACKEND_COMMANDS.deleteArchivedSession, payload);
      },
      submitPrompt(payload) {
        return invoke(BACKEND_COMMANDS.submitPrompt, payload);
      },
    };
  }

  const registry = options.registry ?? createBackendCommandRegistry();

  return {
    getAppStatus() {
      return registry.invoke(BACKEND_COMMANDS.getAppStatus);
    },
    configureOpenRouter(payload) {
      return registry.invoke(BACKEND_COMMANDS.configureOpenRouter, payload);
    },
    registerProject(payload) {
      return registry.invoke(BACKEND_COMMANDS.registerProject, payload);
    },
    listProjectSkills() {
      return registry.invoke(BACKEND_COMMANDS.listProjectSkills);
    },
    invokeProjectSkill(payload) {
      return registry.invoke(BACKEND_COMMANDS.invokeProjectSkill, payload);
    },
    listMcpServers() {
      return registry.invoke(BACKEND_COMMANDS.listMcpServers);
    },
    registerLocalStdioMcpServer(payload) {
      return registry.invoke(BACKEND_COMMANDS.registerLocalStdioMcpServer, payload);
    },
    proposeMcpServerLaunch(payload) {
      return registry.invoke(BACKEND_COMMANDS.proposeMcpServerLaunch, payload);
    },
    exportProjectJson() {
      return registry.invoke(BACKEND_COMMANDS.exportProjectJson);
    },
    deleteArchivedSession(payload) {
      return registry.invoke(BACKEND_COMMANDS.deleteArchivedSession, payload);
    },
    submitPrompt(payload) {
      return registry.invoke(BACKEND_COMMANDS.submitPrompt, payload);
    },
  };
}

/**
 * Finds Tauri's global invoke hook when the app runs in the desktop shell.
 */
function globalTauriInvoke() {
  const tauri = globalThis.window?.__TAURI__;

  return tauri?.core?.invoke ?? tauri?.invoke ?? null;
}
