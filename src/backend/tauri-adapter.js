//client
import {
  BACKEND_COMMANDS,
  createBackendCommandRegistry,
} from './commands.js';

/**
 * Creates UI-facing command invokers for Tauri or browser smoke mode.
 */
export function createAppCommandClient(options = {}) {
  if (options.invoke) {
    return {
      getAppStatus() {
        return options.invoke(BACKEND_COMMANDS.getAppStatus);
      },
    };
  }

  const registry = options.registry ?? createBackendCommandRegistry();

  return {
    getAppStatus() {
      return registry.invoke(BACKEND_COMMANDS.getAppStatus);
    },
  };
}
