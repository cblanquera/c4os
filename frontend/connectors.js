import { validateWorkspacePayload } from "./connector-contract.js";

export function createConnector(options = {}) {
  const search = options.params || new URLSearchParams(window.location.search);
  const fetchImpl = options.fetch || globalThis.fetch.bind(globalThis);
  const kind = search.get("connector") || "none";

  if (kind === "server") {
    return serverConnector({
      api: search.get("api"),
      delay: search.get("delay") || "0",
      fetch: fetchImpl,
      scenario: search.get("scenario") || "success"
    });
  }

  if (kind === "tauri") {
    return tauriConnector();
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

function tauriConnector() {
  return disabledConnector("tauri", "Tauri connector is not available until native commands are registered");
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
