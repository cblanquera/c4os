# TASK-011 Xterm Renderer POC

Status: passed-static-smoke-ready-for-review
Updated: 2026-06-24
Phase: poc

## Proof Question

Can `@xterm/xterm` provide the accepted Terminal-tab interaction model for
C4OS: a single terminal transcript where input, submitted commands, output,
and the next cursor remain inline?

## Why It Matters

User review confirmed backend command execution works but rejected the current
command-form/split-pane UI. A production fix should avoid hand-rolling a
terminal emulator if a proven browser terminal renderer can provide native-like
prompt, cursor, wrapping, transcript, and input behavior inside the existing
right-panel Terminal tab.

## Expected Proof

- `@xterm/xterm` mounts inside a browser-reviewable static proof.
- The empty state shows a prompt and cursor.
- Typed commands appear inline.
- Command output follows the submitted command.
- The next prompt/cursor appears at the bottom.
- The proof can be reviewed without React/Preact/JSX or a bundler.

## Failure Signal

- `@xterm/xterm` cannot run without adding a bundler or framework.
- Input cannot be captured cleanly for backend command forwarding.
- Transcript/cursor behavior does not match the native Terminal reference.
- Styling cannot fit inside the existing right-panel Terminal tab constraints.

## Verification Method

- Install proof-local dependencies in `proofs/terminal-xterm-renderer/`.
- Open `proofs/terminal-xterm-renderer/index.html` in a browser.
- Use Tab to cycle sample commands and Enter to submit them.
- Run the Playwright smoke check recorded in the proof evidence.

## Result

The proof-local `@xterm/xterm` renderer mounted successfully and passed the
browser smoke check on 2026-06-24. It rendered inline prompts, submitted
commands, command output, and the next prompt inside one xterm transcript.

## Decision Target

If accepted, promote `@xterm/xterm` as the production renderer for the
right-panel Terminal tab while keeping execution and policy behind C4OS-owned
Tauri commands/events.
