# Terminal Xterm PTY Bridge Proof

Status: proof artifact
Updated: 2026-06-24

## Purpose

Validate the integrated terminal stack before production implementation:

- `@xterm/xterm` browser renderer
- Rust `portable-pty` backend shell
- local bridge for PTY output, xterm input, and resize

## Setup

```sh
cd proofs/terminal-xterm-pty-bridge
npm install
```

## Run

From the repo root:

```sh
cargo run --quiet --manifest-path proofs/terminal-xterm-pty-bridge/Cargo.toml
```

Open the printed local URL.

## Scope

This proof does not touch production `backend/` or `frontend/`. It does not
use Tauri commands/events yet. It proves the renderer/backend terminal loop
that production Tauri commands/events would carry.
