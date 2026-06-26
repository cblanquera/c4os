import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-012 settings IA and extension records backend boundary", { concurrency: false }, () => {
  it("defines app-owned extension records with enablement and audit fields", async () => {
    const extensionSource = await readFile(new URL("../../backend/src/extensions.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const mockSource = await readFile(new URL("../../backend/src/mock_data.rs", import.meta.url), "utf8");

    for (const field of [
      "provenance",
      "scopes",
      "workspace_scope",
      "project_scope",
      "shared_data",
      "runtime_access",
      "tool_access",
      "enabled",
      "audit"
    ]) {
      assert.match(extensionSource, new RegExp(field));
    }

    assert.match(commandsSource, /list_extensions/);
    assert.match(commandsSource, /extension_records/);
    assert.doesNotMatch(commandsSource, /mock_workspace\(\).*plugin_catalog/s);
    assert.doesNotMatch(mockSource, /pub plugin_catalog: Vec<String>/);
    assert.doesNotMatch(mockSource, /pub skill_catalog: Vec<String>/);
    assert.doesNotMatch(mockSource, /pub mcp_servers: Vec<String>/);
  });

  it("passes Rust tests for inert extension records and workspace payload shape", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_012",
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
