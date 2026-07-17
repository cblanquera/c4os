# Native Terminal Rust PTY Proof

This standalone Rust proof checks the native PTY lifecycle needed behind a
C4OS Terminal boundary.

## Run

From the repository root on macOS or another compatible Unix-like system:

```sh
cargo run --quiet --manifest-path proofs/native-terminal-tauri/Cargo.toml
```

## Result

The current C4OS evidence passed on macOS. It covers trusted-root validation,
approval before launch, PTY startup, bounded output, input, resize,
cancellation, cleanup, and command-failure observation.

## Limits

Despite the directory's historical name, this is a standalone Rust CLI and
does not wire Tauri commands or renderer events. Windows, Linux, production
capability scoping, and transport backpressure remain unproven.

See `native-terminal-tauri-evidence-2026-06-20.md` for the current recorded
checks and raw result.
