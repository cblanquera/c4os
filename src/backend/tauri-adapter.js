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
