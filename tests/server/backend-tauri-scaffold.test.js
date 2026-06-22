import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { spawn } from "node:child_process";

describe("TASK-003 Rust/Tauri backend scaffold", () => {
  it("exposes package scripts to build and run the backend", async () => {
    const packageJson = JSON.parse(await readFile(new URL("../../package.json", import.meta.url), "utf8"));

    assert.equal(packageJson.scripts["backend:build"], "cargo build --manifest-path backend/Cargo.toml");
    assert.equal(packageJson.scripts["backend:run"], "cargo run --manifest-path backend/Cargo.toml");
    assert.equal(packageJson.scripts["backend:test"], "cargo test --manifest-path backend/Cargo.toml");
  });

  it("keeps backend implementation as Rust/Tauri code only", async () => {
    const backendEntries = await readdir(new URL("../../backend/", import.meta.url));

    assert.equal(backendEntries.includes("Cargo.toml"), true);
    assert.equal(backendEntries.includes("tauri.conf.json"), true);
    assert.equal(backendEntries.some((entry) => entry.endsWith(".js")), false);
    assert.equal(backendEntries.includes("src"), true);
  });

  it("documents the native desktop menu contract in the Rust backend surface", async () => {
    const readme = await readFile(new URL("../../backend/README.md", import.meta.url), "utf8");
    const libSource = await readFile(new URL("../../backend/src/lib.rs", import.meta.url), "utf8");
    const menuSource = await readFile(new URL("../../backend/src/menu.rs", import.meta.url), "utf8");
    const tauriConfig = JSON.parse(await readFile(new URL("../../backend/tauri.conf.json", import.meta.url), "utf8"));

    for (const item of [
      "File > Open Workspace",
      "File > Save Workspace",
      "File > Save File",
      "Edit > Undo",
      "Edit > Redo",
      "Edit > Select All",
      "Edit > Cut",
      "Edit > Copy",
      "Edit > Paste"
    ]) {
      assert.match(readme, new RegExp(item.replace(/[>]/g, "\\$&")));
    }

    assert.match(readme, /not an in-app toolbar/);
    assert.match(libSource, /\.menu\(menu::build_app_menu\)/);
    assert.match(libSource, /\.on_menu_event\(menu::handle_menu_event\)/);
    assert.match(menuSource, /MenuContract/);
    assert.match(menuSource, /pub fn build_app_menu/);
    assert.match(menuSource, /pub fn apply_native_menu_state/);
    assert.match(menuSource, /MenuBuilder::new/);
    assert.match(menuSource, /SubmenuBuilder::with_id/);
    assert.match(menuSource, /set_enabled\(state.commands\["file.saveFile"\].enabled\)/);
    assert.match(menuSource, /file_editor_open && focus_state.file_can_save/);
    assert.match(menuSource, /focus_state.editable/);
    assert.equal(tauriConfig.app.withGlobalTauri, true);
  });

  it("passes Rust tests for TASK-002 mock parity and native menu state", async () => {
    const result = await run("cargo", ["test", "--manifest-path", "backend/Cargo.toml"]);

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
