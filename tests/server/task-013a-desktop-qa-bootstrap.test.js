import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-013A desktop QA bootstrap and persistence boundary", { concurrency: false }, () => {
  it("exposes workspace file commands and explicit launch-flag bootstrap through the backend boundary", async () => {
    const packageSource = await readFile(new URL("../../package.json", import.meta.url), "utf8");
    const workspaceSource = await readFile(new URL("../../backend/src/workspace.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");
    const providerSource = await readFile(new URL("../../backend/src/provider_models.rs", import.meta.url), "utf8");

    assert.match(libSource, /--workspace/);
    assert.match(libSource, /--workspace-file/);
    assert.match(packageSource, /backend:run/);
    assert.match(packageSource, /node scripts\/run-backend\.js/);
    assert.match(workspaceSource, /bootstrap_workspace_from_launch_flag/);
    assert.doesNotMatch(workspaceSource, /bootstrap_workspace_from_environment/);
    assert.match(commandsSource, /open_workspace_file/);
    assert.match(commandsSource, /save_workspace_file/);
    assert.match(commandsSource, /Option<WorkspaceFileRequest>/);
    assert.match(workspaceSource, /choose_workspace_file/);
    assert.match(workspaceSource, /choose_save_workspace_file/);
    assert.match(libSource, /commands::open_workspace_file/);
    assert.match(libSource, /commands::save_workspace_file/);
    assert.match(providerSource, /C4OS_PROVIDER_STORE/);
    assert.match(providerSource, /C4OS_PROVIDER_KEY_STORE/);
    assert.doesNotMatch(providerSource, /sk-(or-v1|proj)-[A-Za-z0-9]/);
  });

  it("passes Rust TASK-013A bootstrap and provider persistence tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_013a",
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
