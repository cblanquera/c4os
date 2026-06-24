# TASK-011 Terminal Inline UI POC

Status: planned
Updated: 2026-06-24
Phase: poc

## Proof Question

Can the accepted right-panel Terminal tab use a terminal-emulator-style UI where
commands are entered inline with output and the active cursor remains at the
bottom, instead of a separate command form above split output panes?

## Why It Matters

User review on 2026-06-24 confirmed the backend-owned user terminal execution
works, but rejected the current TASK-011 UI direction. The target behavior is
closer to a native macOS Terminal transcript:

- empty terminal shows a prompt and cursor;
- commands such as `ls`, `rm`, `git init`, and `yarn install` appear inline in
  the same terminal stream;
- command output follows immediately after the submitted command;
- the next prompt and cursor stay at the bottom.

## Current Tool Finding

C4OS is not using Tauri's CLI plugin for TASK-011. The current implementation
uses product-owned Tauri commands and session records. The Tauri v2 CLI plugin
is for parsing arguments passed to the app process; it is not an embedded
interactive terminal or PTY surface.

## Expected Proof

- A browser-reviewable proof renders the terminal empty state with prompt and
  cursor in the first row.
- Typing commands appends the command inline to the transcript.
- Simulated command output appears directly below the command.
- The next prompt and cursor render at the bottom after each command.
- The proof does not add a new product route, panel, settings abstraction,
  React/Preact/JSX, or bundler.

## Failure Signal

- The input remains a separate form outside the terminal transcript.
- User command and output are visually split into different panes.
- The cursor does not remain at the next prompt after output.
- The UI cannot fit inside the existing right-panel Terminal tab shape.

## Verification Method

- Open `proofs/terminal-inline-ui/index.html` in a browser.
- Run the included transcript simulation with `ls`, `git init`, and
  `yarn install`.
- Capture the result in `proofs/terminal-inline-ui/terminal-inline-ui-evidence-2026-06-24.md`.

## Decision Target

Use this proof to decide whether TASK-011 production UI should replace the
current command-form/split-pane terminal with a transcript-driven terminal
surface. Backend PTY lifecycle remains covered by existing terminal POCs under
`proofs/native-terminal-plugin/` and `proofs/native-terminal-tauri/`.
