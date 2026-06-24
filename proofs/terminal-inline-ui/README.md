# Terminal Inline UI Proof

Status: proof artifact
Updated: 2026-06-24

## Purpose

This proof isolates the TASK-011 terminal UI question raised during review:
the Terminal tab should behave visually like a terminal transcript, not like a
form above separated output panes.

## How To Review

Open `index.html` directly in a browser.

The proof is static HTML/CSS/JS and does not require a dev server, package
install, React/Preact/JSX, or a bundler.

## What It Proves

- Empty state prompt and cursor render inline.
- Commands typed by the user are appended to the same transcript.
- Output follows the command immediately.
- The next prompt and cursor stay at the bottom.

## What It Does Not Prove

- Real PTY lifecycle, streaming, resize, cancellation, or cleanup.
- Tauri command capability wiring.
- Approval policy hardening.

Those backend/runtime behaviors are covered by the existing native terminal
POCs in `proofs/native-terminal-plugin/` and `proofs/native-terminal-tauri/`.
