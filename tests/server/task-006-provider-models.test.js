import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-006 provider and model management slice", { concurrency: false }, () => {
  it("adds backend-owned provider/model records without persisted API key material", async () => {
    const providerSource = await readFile(new URL("../../backend/src/provider_models.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const workspaceSource = await readFile(new URL("../../backend/src/workspace.rs", import.meta.url), "utf8");

    assert.match(providerSource, /ProviderProfile/);
    assert.match(providerSource, /OpenAI-compatible/);
    assert.match(providerSource, /ApiKeyStatus/);
    assert.match(providerSource, /manual/);
    assert.match(commandsSource, /list_provider_profiles/);
    assert.match(commandsSource, /save_provider_profile/);
    assert.match(commandsSource, /delete_provider_profile/);
    assert.match(commandsSource, /set_provider_enabled/);
    assert.match(commandsSource, /set_model_enabled/);
    assert.match(commandsSource, /select_session_model/);
    assert.doesNotMatch(providerSource, /sk-(or-v1|proj)-[A-Za-z0-9]/);
    assert.doesNotMatch(commandsSource, /sk-(or-v1|proj)-[A-Za-z0-9]/);
    assert.doesNotMatch(workspaceSource, /sk-(or-v1|proj)-[A-Za-z0-9]/);
  });

  it("passes Rust tests for provider update, deletion, and discovered model defaults", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_006_provider_profile_lifecycle",
      "--",
      "--test-threads=1"
    ]);

    assert.equal(result.code, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /test result: ok/);
  });

  it("passes Rust tests for empty initial state and saved-key chat authority", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_006_empty_initial_state_and_saved_key_authority",
      "--",
      "--test-threads=1"
    ]);

    assert.equal(result.code, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /test result: ok/);
  });

  it("passes Rust tests for secure key status, manual models, and session selection", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_006",
      "--",
      "--test-threads=1"
    ]);

    assert.equal(result.code, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /test result: ok/);
  });

  it("backend provider save path does not serialize raw API key material", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_006_save",
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
