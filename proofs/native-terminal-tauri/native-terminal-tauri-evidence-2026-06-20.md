# Native Terminal Tauri PTY POC Evidence

Date: 2026-06-20
Status: passed-on-macos
Scope: disposable POC only

## Command

```sh
cargo run --quiet --manifest-path proofs/native-terminal-tauri/Cargo.toml
```

## Summary

The Rust/macOS PTY proof passed in this checkout. It proves trusted-root
scoping, approval-gated launch decisions, PTY startup in the trusted root,
streaming input/resize behavior, cancellation/cleanup, and command failure
observation using native Unix PTY APIs.

## Raw Result

```json
{
  "checks": [
    {
      "evidence": {
        "blockedPath": "/tmp",
        "outOfRootBlocked": true,
        "trustedAllowed": true
      },
      "name": "trusted_root_scoping",
      "passed": true
    },
    {
      "evidence": {
        "approvalRequired": true,
        "command": "rm -rf /tmp/native-terminal-tauri-poc",
        "spawnAttempted": false
      },
      "name": "approval_gated_launch",
      "passed": true
    },
    {
      "evidence": {
        "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os",
        "output": "started:/Users/cblanquera/server/projects/cblanquera/c4os\\r\\nready\\r\\n"
      },
      "name": "pty_starts_in_trusted_root",
      "passed": true
    },
    {
      "evidence": {
        "chunkLimit": 128,
        "inputObserved": true,
        "maxChunkLength": 66,
        "output": "hello-from-renderer\\r\\ninput:hello-from-renderer\\r\\nsize:40 120\\r\\n",
        "resizeIoctlSucceeded": true
      },
      "name": "streaming_input_resize",
      "passed": true
    },
    {
      "evidence": {
        "cleanupClosedMaster": true,
        "exitObserved": true,
        "killRequested": true,
        "status": -1
      },
      "name": "cancellation_exit_cleanup",
      "passed": true
    },
    {
      "evidence": {
        "cleanupClosedMaster": true,
        "exitObserved": true,
        "output": "/bin/sh: definitely_missing_terminal_poc_command_123: command not found\\r\\n",
        "status": 127
      },
      "name": "command_failure_observable",
      "passed": true
    }
  ],
  "passed": true,
  "platform": "macos",
  "poc": "native-terminal-tauri",
  "trustedRoot": "/Users/cblanquera/server/projects/cblanquera/c4os"
}
```

## Limits

- Windows ConPTY behavior is not tested.
- Linux PTY behavior is not tested in this run.
- Production Tauri command capability scoping and renderer event backpressure
  still need implementation-level verification.
