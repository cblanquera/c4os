import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

describe("TASK-011 terminal backend boundary", () => {
  it("defines session-scoped backend-owned terminal execution records", async () => {
    const commandsSource = await readFile(new URL("../../backend/src/commands.rs", import.meta.url), "utf8");
    const sessionsSource = await readFile(new URL("../../backend/src/runtime_sessions.rs", import.meta.url), "utf8");
    const ptySource = await readFile(new URL("../../backend/src/terminal_pty.rs", import.meta.url), "utf8");
    const cargoSource = await readFile(new URL("../../backend/Cargo.toml", import.meta.url), "utf8");

    assert.match(cargoSource, /portable-pty/);
    assert.match(commandsSource, /pub fn start_terminal_session/);
    assert.match(commandsSource, /pub fn write_terminal_input/);
    assert.match(commandsSource, /pub fn read_terminal_output/);
    assert.match(commandsSource, /pub fn resize_terminal_session/);
    assert.match(commandsSource, /session_id: Option<String>/);
    assert.match(commandsSource, /terminal_kind: Option<String>/);
    assert.match(commandsSource, /fn terminal_scope_from_request/);
    assert.match(commandsSource, /workspace:\{\}/);
    assert.match(commandsSource, /active_workspace_descriptor/);
    assert.match(commandsSource, /c4os:\/\/terminal-output/);
    assert.doesNotMatch(commandsSource, /let terminal: TerminalState = mock_workspace\(\)\.terminal/);
    assert.match(sessionsSource, /pub struct TerminalActionRecord/);
    assert.match(ptySource, /pub fn start_terminal/);
    assert.match(ptySource, /pub fn write_terminal/);
    assert.match(ptySource, /pub fn resize_terminal/);
    assert.match(ptySource, /native_pty_system\(\)/);
    assert.match(ptySource, /CommandBuilder::new/);
  });

  it("keeps user and agent terminal panes distinct in product-owned state", async () => {
    const mockDataSource = await readFile(new URL("../../backend/src/mock_data.rs", import.meta.url), "utf8");
    const dataSource = await readFile(new URL("../../frontend/data.js", import.meta.url), "utf8");

    assert.match(mockDataSource, /pub user_terminal: TerminalPaneState/);
    assert.match(mockDataSource, /pub agent_terminal: TerminalPaneState/);
    assert.match(dataSource, /userTerminal/);
    assert.match(dataSource, /agentTerminal/);
  });

  it("reruns the Tauri resource build when terminal frontend assets change", async () => {
    const buildSource = await readFile(new URL("../../backend/build.rs", import.meta.url), "utf8");

    assert.match(buildSource, /cargo:rerun-if-changed=\.\.\/frontend\/app\.js/);
    assert.match(buildSource, /cargo:rerun-if-changed=\.\.\/frontend\/vendor\/xterm\/xterm\.js/);
  });
});
