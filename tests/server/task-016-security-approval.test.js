import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-016 security and approval policies", { concurrency: false }, () => {
  it("defines backend-owned security policy storage and routes gateway enforcement through it", async () => {
    const securitySource = await readFile(new URL("../../backend/src/security.rs", import.meta.url), "utf8");
    const gatewaySource = await readFile(new URL("../../backend/src/tool_gateway.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");

    for (const concept of [
      "ToolApprovalPolicyStore",
      "WorkspaceToolPolicy",
      "SessionToolPolicyOverride",
      "EffectiveToolPolicySnapshot",
      "C4OS_TOOL_POLICY_STORE",
      "load_tool_approval_policy",
      "save_tool_approval_policy",
      "effective_tool_policy_snapshot",
      "record_security_audit",
      "secure_key_storage_policy"
    ]) {
      assert.match(securitySource, new RegExp(concept));
    }

    for (const identity of [
      "terminal.run",
      "files.read",
      "files.write",
      "browser.open",
      "artifact.preview"
    ]) {
      assert.match(securitySource, new RegExp(identity.replace(".", "\\.")));
    }

    assert.match(gatewaySource, /effective_tool_policy_snapshot/);
    assert.match(gatewaySource, /enforce_tool_request/);
    assert.match(commandsSource, /load_tool_approval_policy/);
    assert.match(commandsSource, /save_tool_approval_policy/);
    assert.match(libSource, /pub mod security;/);
    assert.match(libSource, /commands::load_tool_approval_policy/);
    assert.match(libSource, /commands::save_tool_approval_policy/);
  });

  it("passes Rust policy storage, gateway enforcement, and audit tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_016",
      "--",
      "--test-threads=1"
    ]);

    assert.equal(result.code, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /test result: ok/);
  });
});

function run(command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      cwd: new URL("../..", import.meta.url),
      env: process.env
    });
    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => stdout += chunk);
    child.stderr.on("data", (chunk) => stderr += chunk);
    child.on("close", (code) => resolve({ code, stderr, stdout }));
  });
}
