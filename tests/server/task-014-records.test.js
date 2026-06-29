import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-014 local memory, action, and audit records boundary", { concurrency: false }, () => {
  it("defines app-owned records outside workspace descriptors and provider stores", async () => {
    const recordsSource = await readFile(new URL("../../backend/src/records.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");
    const workspaceSource = await readFile(new URL("../../backend/src/workspace.rs", import.meta.url), "utf8");
    const providerSource = await readFile(new URL("../../backend/src/provider_models.rs", import.meta.url), "utf8");

    for (const name of [
      "LocalMemoryRecord",
      "ActionRecord",
      "AuditRecord",
      "C4OS_RECORD_STORE",
      "record_browser_action",
      "record_terminal_action"
    ]) {
      assert.match(recordsSource, new RegExp(name));
    }

    for (const command of [
      "list_local_memory_records",
      "save_local_memory_record",
      "list_action_records",
      "list_audit_records"
    ]) {
      assert.match(commandsSource, new RegExp(command));
      assert.match(libSource, new RegExp(`commands::${command}`));
    }

    assert.doesNotMatch(workspaceSource, /LocalMemoryRecord|ActionRecord|AuditRecord|C4OS_RECORD_STORE/);
    assert.doesNotMatch(providerSource, /LocalMemoryRecord|ActionRecord|AuditRecord|C4OS_RECORD_STORE/);
    assert.doesNotMatch(recordsSource, /api_key|apiKey|sk-(or-v1|proj)-[A-Za-z0-9]/);
  });

  it("passes Rust record persistence and separation tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_014",
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
