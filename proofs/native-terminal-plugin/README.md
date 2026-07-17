# Native Terminal Plugin Proof

This Python proof checks backend-owned PTY lifecycle semantics for a C4OS
Terminal without giving the renderer direct process-spawn authority.

## Run

From the repository root on a Unix-like system with Python 3:

```sh
python3 proofs/native-terminal-plugin/pty_lifecycle_poc.py
```

## Result

The current C4OS evidence passed on macOS. It covers trusted-root startup,
streaming output, input, resize, pre-launch approval, observable failures,
cancellation, and cleanup.

## Limits

The proof uses Python standard-library PTY primitives. It does not establish
Rust/Tauri transport, Windows or Linux behavior, or production Terminal
integration.

See `native-terminal-plugin-evidence-2026-06-20.md` for the current recorded
checks and raw result.
