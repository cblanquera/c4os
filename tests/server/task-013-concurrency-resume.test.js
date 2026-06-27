import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-013 concurrency and restart/resume boundary", { concurrency: false }, () => {
  it("persists trusted-workspace identity on session records", async () => {
    const sessionsSource = await readFile(new URL("../../backend/src/runtime_sessions.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");

    assert.match(sessionsSource, /workspace_root/);
    assert.match(sessionsSource, /current_workspace_record/);
    assert.match(sessionsSource, /session_workspace_root/);
    assert.match(commandsSource, /active_workspace_sessions/);
    assert.match(commandsSource, /file_command_root/);
    assert.match(commandsSource, /browser_state_for_target\(\s*&session,\s*&root/s);
  });

  it("passes Rust concurrency isolation and restart/resume tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_013",
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
