import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

describe("WO-006 runtime tool gateway boundary", () => {
  it("defines a C4OS-owned tool gateway contract with stable identities and lifecycle events", async () => {
    const gatewaySource = await readFile(new URL("../../backend/src/tool_gateway.rs", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");

    assert.match(libSource, /pub mod tool_gateway;/);

    for (const identity of [
      "terminal.run",
      "files.list",
      "files.read",
      "files.write",
      "browser.open",
      "artifact.preview"
    ]) {
      assert.match(gatewaySource, new RegExp(identity.replace(".", "\\.")));
    }

    for (const lifecycle of [
      "tool_call_requested",
      "tool_call_started",
      "tool_output_delta",
      "tool_call_completed",
      "tool_call_rejected"
    ]) {
      assert.match(gatewaySource, new RegExp(lifecycle));
    }

    for (const concept of [
      "ToolCallRequest",
      "ToolCallResponse",
      "ToolApproval",
      "ToolAccess",
      "SessionToolConfig",
      "default_approval",
      "max_approval",
      "dispatch_tool_call"
    ]) {
      assert.match(gatewaySource, new RegExp(concept));
    }
  });

  it("routes existing app interaction commands through the gateway", async () => {
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");

    for (const command of [
      "read_file",
      "save_file",
      "create_artifact_preview",
      "run_terminal_command",
      "open_browser"
    ]) {
      const start = commandsSource.indexOf(`pub fn ${command}`);
      assert.notEqual(start, -1, `${command} command exists`);
      const body = commandsSource.slice(start, commandsSource.indexOf("\n#[tauri::command]", start + 1));
      assert.match(body, /dispatch_tool_call/, `${command} should delegate through gateway`);
    }
  });
});
