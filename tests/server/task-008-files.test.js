import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-008 trusted-root Files slice", { concurrency: false }, () => {
  it("adds backend-owned trusted file authority behind the Files surface", async () => {
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const filesSource = await readFile(new URL("../../backend/src/files.rs", import.meta.url), "utf8");

    assert.match(commandsSource, /read_file\(request: Option<FileRequest>\) -> Result<FileReadResponse, String>/);
    assert.match(commandsSource, /save_file\(request: Option<FileRequest>\) -> Result<FileSaveResponse, String>/);
    assert.match(filesSource, /trusted_path/);
    assert.match(filesSource, /outside trusted root/);
    assert.match(filesSource, /Casual \.git mutation is rejected/);
  });

  it("passes Rust trusted browsing, save, revert, and guard tests", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_008",
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
