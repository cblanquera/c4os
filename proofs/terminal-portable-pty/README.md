# Terminal Portable PTY Proof

Status: proof artifact
Updated: 2026-06-24

## Purpose

Validate `portable-pty` as the Rust backend layer for interactive C4OS terminal
sessions before changing production backend code.

## Run

```sh
cargo run --quiet --manifest-path proofs/terminal-portable-pty/Cargo.toml
```

## Scope

This proof validates trusted-root scoping, shell launch, stdin writes, output
reads, resize, termination, and command failure observation through
`portable-pty`.

It does not wire Tauri commands/events or product session records.
