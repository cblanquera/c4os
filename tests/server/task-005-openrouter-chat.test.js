import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-005 OpenRouter chat session review slice", () => {
  it("adds a backend OpenRouter stream normalizer without persisted API key material", async () => {
    const openRouterSource = await readFile(new URL("../../backend/src/openrouter.rs", import.meta.url), "utf8");
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const workspaceSource = await readFile(new URL("../../backend/src/workspace.rs", import.meta.url), "utf8");
    const providerSource = await readFile(new URL("../../backend/src/provider_models.rs", import.meta.url), "utf8");

    assert.match(openRouterSource, /OPENROUTER_API_KEY/);
    assert.match(providerSource, /google\/gemini-2\.5-flash-lite/);
    assert.match(openRouterSource, /reasoning_details/);
    assert.match(openRouterSource, /chat\/completions/);
    assert.match(commandsSource, /c4os:\/\/runtime-event/);
    assert.doesNotMatch(openRouterSource, /sk-or-v1-/);
    assert.doesNotMatch(commandsSource, /sk-or-v1-/);
    assert.doesNotMatch(workspaceSource, /sk-or-v1-/);
  });

  it("passes Rust tests for OpenRouter reasoning and final-response event parsing", async () => {
    const result = await run("cargo", [
      "test",
      "--manifest-path",
      "backend/Cargo.toml",
      "task_005"
    ]);

    assert.equal(result.code, 0, result.stderr || result.stdout);
    assert.match(result.stdout, /test result: ok/);
  });

  it("keeps native prompt execution off the synchronous command path for realtime events", async () => {
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");

    assert.match(commandsSource, /pub async fn send_prompt/);
    assert.match(commandsSource, /tauri::async_runtime::spawn_blocking/);
    assert.match(commandsSource, /app\.emit\("c4os:\/\/runtime-event"/);
  });

  it("lists project chat sessions newest first for the left navigation", async () => {
    const sessionsSource = await readFile(new URL("../../backend/src/runtime_sessions.rs", import.meta.url), "utf8");

    assert.match(sessionsSource, /pub fn project_sessions/);
    assert.match(sessionsSource, /\.rev\(\)/);
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
