import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-007 runtime adapter and persistent sessions", { concurrency: false }, () => {
  it("adds C4OS-owned session/run/message records behind a runtime adapter boundary", async () => {
    const sessionsSource = await readFile(new URL("../../backend/src/runtime_sessions.rs", import.meta.url), "utf8");
    const adapterSource = await readFile(new URL("../../backend/src/runtime_adapter.rs", import.meta.url), "utf8");
    const frontendSource = await readFile(new URL("../../frontend/data.js", import.meta.url), "utf8");

    assert.match(sessionsSource, /C4osWorkspaceRecord/);
    assert.match(sessionsSource, /C4osSessionRecord/);
    assert.match(sessionsSource, /C4osRunRecord/);
    assert.match(sessionsSource, /C4osMessageRecord/);
    assert.match(sessionsSource, /C4osRuntimeReference/);
    assert.match(adapterSource, /C4osRuntimeAdapter/);
    assert.doesNotMatch(frontendSource, /OpenCode|opencode|session_id|thread_id|message_id/);
  });

  it("passes Rust session isolation, append targeting, and restart resume tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_007",
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
