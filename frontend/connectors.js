import { validateWorkspacePayload } from "./connector-contract.js";

export function createConnector(options = {}) {
  const search = options.params || new URLSearchParams(window.location.search);
  const fetchImpl = options.fetch || globalThis.fetch.bind(globalThis);
  const tauriInvoke = options.tauriInvoke || resolveTauriInvoke();
  const tauriListen = options.tauriListen || resolveTauriListen();
  const kind = search.get("connector") || (tauriInvoke ? "tauri" : "none");

  if (kind === "server") {
    return serverConnector({
      api: search.get("api"),
      delay: search.get("delay") || "0",
      fetch: fetchImpl,
      scenario: search.get("scenario") || "success"
    });
  }

  if (kind === "tauri") {
    return tauriConnector({
      invoke: tauriInvoke,
      listen: tauriListen
    });
  }

  return disabledConnector("none", "No connector configured");
}

function serverConnector(config) {
  if (!config.api) return disabledConnector("server", "Server connector requires an API origin");

  const withScenario = (path) => {
    const url = new URL(path, config.api);
    url.searchParams.set("scenario", config.scenario);
    url.searchParams.set("delay", config.delay);
    return url.toString();
  };

  return {
    available: true,
    kind: "server",
    async loadWorkspace() {
      const response = await config.fetch(withScenario("/api/workspace"));
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || "Workspace request failed");
      const missing = validateWorkspacePayload(payload);
      if (missing.length > 0) throw new Error(`Workspace payload missing: ${missing.join(", ")}`);
      return payload;
    },
    async sendPrompt(prompt) {
      const response = await config.fetch(withScenario("/api/runs"), {
        body: JSON.stringify({ prompt }),
        headers: { "content-type": "application/json" },
        method: "POST"
      });
      const payload = await response.json();
      if (!response.ok) throw new Error(payload.error || "Agent run failed");
      return payload;
    }
  };
}

function tauriConnector(config) {
  if (!config.invoke) return disabledConnector("tauri", "Tauri invoke bridge is not available");

  const invoke = config.invoke;

  return {
    available: true,
    kind: "tauri",
    async loadWorkspace() {
      const payload = await invoke("load_workspace");
      const missing = validateWorkspacePayload(payload);
      if (missing.length > 0) throw new Error(`Workspace payload missing: ${missing.join(", ")}`);
      return payload;
    },
    async sendPrompt(prompt, options = {}) {
      let unlisten;
      if (config.listen && options.onEvent) {
        try {
          unlisten = await config.listen("c4os://runtime-event", (event) => {
            options.onEvent(event.payload || event);
          });
        } catch {
          unlisten = null;
        }
      }
      try {
        return await invoke("send_prompt", { prompt });
      } finally {
        if (typeof unlisten === "function") unlisten();
      }
    },
    async openWorkspace(path) {
      return invoke("open_workspace", { request: { path } });
    },
    async createSession(project) {
      return invoke("create_session", { project });
    },
    async readFile(path) {
      return invoke("read_file", { request: { path } });
    },
    async saveFile(path) {
      return invoke("save_file", { request: { path } });
    },
    async runTerminalCommand(command) {
      return invoke("run_terminal_command", { request: { command } });
    },
    async openBrowserPreview() {
      return invoke("open_browser_preview");
    },
    async listExtensions() {
      return invoke("list_extensions");
    }
  };
}

function disabledConnector(kind, message) {
  const unavailable = async () => {
    throw new Error(message);
  };

  return {
    available: false,
    kind,
    loadWorkspace: unavailable,
    sendPrompt: unavailable
  };
}

function resolveTauriInvoke() {
  return globalThis.__TAURI__?.core?.invoke || globalThis.__TAURI__?.invoke;
}

function resolveTauriListen() {
  return globalThis.__TAURI__?.event?.listen;
}
