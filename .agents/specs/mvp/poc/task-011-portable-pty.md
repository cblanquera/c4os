# TASK-011 Portable PTY POC

Status: passed-on-macos-ready-for-review
Updated: 2026-06-24
Phase: poc

## Proof Question

Can the Rust `portable-pty` crate provide the backend PTY layer C4OS needs for
interactive terminal sessions, including trusted-root launch, stdin writes,
output reads, resize, termination, and observable failure?

## Why It Matters

The accepted Terminal behavior needs more than one-shot command execution. It
needs a backend-owned interactive process session that can feed a terminal
renderer such as `@xterm/xterm`. C4OS should validate this before replacing the
current TASK-011 command execution path.

## Expected Proof

- A proof-local Rust binary uses `portable-pty`.
- PTY launches a shell in the trusted project root.
- Renderer-style input writes are observed by the child process.
- Output can be read from the PTY master.
- Resize can be sent to the PTY.
- Cancellation/termination and failure states are observable.

## Failure Signal

- `portable-pty` cannot build in the current Tauri/Rust environment.
- PTY cannot be constrained to trusted root.
- Input/output cannot be streamed reliably.
- Resize or cleanup is not observable enough for product behavior.

## Verification Method

- Run `cargo run --quiet --manifest-path proofs/terminal-portable-pty/Cargo.toml`.
- Record JSON evidence in
  `proofs/terminal-portable-pty/terminal-portable-pty-evidence-2026-06-24.md`.

## Result

The proof-local Rust binary passed on macOS on 2026-06-24. It verified trusted
root scoping, shell launch, stdin write, output read, resize observation,
termination, and failure observation through `portable-pty`.

## Decision Target

If accepted, promote `portable-pty` as the production backend PTY layer behind
C4OS-owned Tauri commands/events. Do not expose raw PTY or crate concepts as
the product model.
