import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createConnector } from "../frontend/connectors.js";
import { connectorMethods, validateWorkspacePayload } from "../frontend/connector-contract.js";
import { createMockWorkspace } from "./server/mock-data.js";

describe("frontend connector boundary", () => {
  it("normalizes server transport methods behind the connector interface", async () => {
    const calls = [];
    const connector = createConnector({
      fetch: async (url, options = {}) => {
        calls.push({ method: options.method || "GET", url });
        return {
          ok: true,
          json: async () => options.method === "POST"
            ? { agent: "done", run: "done" }
            : createMockWorkspace()
        };
      },
      params: new URLSearchParams("connector=server&api=http%3A%2F%2F127.0.0.1%3A4310&scenario=success&delay=7")
    });

    assert.equal(connector.kind, "server");
    assert.equal(connector.available, true);
    assert.equal((await connector.loadWorkspace()).workspace.project, "Mock Workspace Alpha");
    assert.deepEqual(await connector.sendPrompt("hello"), { agent: "done", run: "done" });
    assert.deepEqual(calls, [
      { method: "GET", url: "http://127.0.0.1:4310/api/workspace?scenario=success&delay=7" },
      { method: "POST", url: "http://127.0.0.1:4310/api/runs?scenario=success&delay=7" }
    ]);
  });

  it("normalizes tauri native commands behind the connector interface", async () => {
    const calls = [];
    const connector = createConnector({
      params: new URLSearchParams("connector=tauri"),
      tauriInvoke: async (command, payload) => {
        calls.push({ command, payload });
        if (command === "load_workspace") return createMockWorkspace();
        if (command === "send_prompt") {
          return {
            agent: "Mock agent completed the requested transition.",
            prompt: payload.prompt,
            run: "Mock agent completed the requested transition."
          };
        }
        return { ok: true, command, payload };
      }
    });

    assert.equal(connector.kind, "tauri");
    assert.equal(connector.available, true);
    assert.equal((await connector.loadWorkspace()).workspace.project, "Mock Workspace Alpha");
    assert.deepEqual(await connector.sendPrompt("hello tauri"), {
      agent: "Mock agent completed the requested transition.",
      prompt: "hello tauri",
      run: "Mock agent completed the requested transition."
    });
    assert.deepEqual(calls, [
      { command: "load_workspace", payload: undefined },
      { command: "send_prompt", payload: { prompt: "hello tauri" } }
    ]);
  });

  it("auto-selects tauri transport when native invoke is available and no connector query is set", async () => {
    const connector = createConnector({
      params: new URLSearchParams(),
      tauriInvoke: async (command) => {
        if (command === "load_workspace") return createMockWorkspace();
        return { ok: true };
      }
    });

    assert.equal(connector.kind, "tauri");
    assert.equal(connector.available, true);
    assert.equal((await connector.loadWorkspace()).workspace.project, "Mock Workspace Alpha");
  });

  it("keeps app orchestration free of mock-server-specific function names", async () => {
    const source = await readFile(new URL("../frontend/app.js", import.meta.url), "utf8");

    assert.equal(source.includes("beginMockStateLoad"), false);
    assert.equal(source.includes("sendMockPrompt"), false);
    assert.equal(source.includes("mockRuntime"), false);
  });

  it("documents the normalized connector method inventory for future transports", () => {
    assert.deepEqual(connectorMethods, [
      "loadWorkspace",
      "sendPrompt",
      "openWorkspace",
      "createSession",
      "readFile",
      "saveFile",
      "runTerminalCommand",
      "openBrowserPreview",
      "listExtensions"
    ]);
  });

  it("validates complete workspace payloads and reports missing top-level fields", () => {
    assert.deepEqual(validateWorkspacePayload(createMockWorkspace()), []);
    assert.deepEqual(validateWorkspacePayload({ workspace: {} }), [
      "projects",
      "providers",
      "models",
      "pluginCatalog",
      "pluginMarketplaces",
      "skillCatalog",
      "mcpServers",
      "browser",
      "files",
      "terminal",
      "thread"
    ]);
  });

  it("documents mocked behavior by connector method", async () => {
    const readme = await readFile(new URL("./server/README.md", import.meta.url), "utf8");

    for (const method of ["loadWorkspace()", "sendPrompt(prompt)"]) {
      assert.match(readme, new RegExp(`### ${method.replace(/[()]/g, "\\$&")}`));
    }
  });
});
