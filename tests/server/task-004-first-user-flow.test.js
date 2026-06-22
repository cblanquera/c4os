import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-004 first real user flow", () => {
  it("moves workspace activation out of the TASK-003 mock fixture", async () => {
    const workspaceSource = await readFile(new URL("../../backend/src/workspace.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");

    assert.match(workspaceSource, /WorkspaceDescriptor/);
    assert.match(workspaceSource, /\.c4os/);
    assert.match(workspaceSource, /workspace\.json/);
    assert.match(commandsSource, /activate_workspace/);
    assert.match(workspaceSource, /choose_workspace_folder/);
    assert.match(workspaceSource, /Open C4OS Workspace Folder/);
    assert.doesNotMatch(commandsSource, /unwrap_or_else\(\|\| "\/mock\/workspace"\.into\(\)\)/);
  });

  it("passes backend tests for descriptor-backed workspace activation", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_004"
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
