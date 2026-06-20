# Native Terminal Tauri PTY POC Evidence

Date: 2026-06-16
Status: passed-on-macos
Scope: disposable POC only

## Source

- `.agents/progress/items/item-050.md`
- `.agents/specs/ai-workspace/post-mvp-native-terminal-plugin.md`
- `.agents/poc/post-mvp-native-terminal-plugin.md`

## POC Artifact

- `poc/native-terminal-tauri/Cargo.toml`
- `poc/native-terminal-tauri/src/main.rs`

This POC is a standalone Rust CLI using native Unix PTY APIs through `libc`.
It does not add Tauri commands, renderer transport, product Terminal UI, or any
production Terminal plugin feature.

## Command Evidence

```sh
cargo build
```

Result: passed.

```sh
cargo run
```

Result: passed.

Observed output summary:

```json
{
  "poc": "native-terminal-tauri",
  "platform": "macos",
  "trustedRoot": "/Users/cblanquera/server/projects/cblanquera/c4os2",
  "passed": true,
  "checks": [
    {
      "name": "trusted_root_scoping",
      "passed": true,
      "evidence": {
        "trustedAllowed": true,
        "outOfRootBlocked": true,
        "blockedPath": "/tmp"
      }
    },
    {
      "name": "approval_gated_launch",
      "passed": true,
      "evidence": {
        "command": "rm -rf /tmp/native-terminal-tauri-poc",
        "approvalRequired": true,
        "spawnAttempted": false
      }
    },
    {
      "name": "pty_starts_in_trusted_root",
      "passed": true,
      "evidence": {
        "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os2",
        "output": "started:/Users/cblanquera/server/projects/cblanquera/c4os2\\r\\nready\\r\\n"
      }
    },
    {
      "name": "streaming_input_resize",
      "passed": true,
      "evidence": {
        "inputObserved": true,
        "resizeIoctlSucceeded": true,
        "maxChunkLength": 67,
        "chunkLimit": 128,
        "output": "hello-from-renderer\\r\\ninput:hello-from-renderer\\r\\nsize:40 120\\r\\n"
      }
    },
    {
      "name": "cancellation_exit_cleanup",
      "passed": true,
      "evidence": {
        "killRequested": true,
        "exitObserved": true,
        "status": -1,
        "cleanupClosedMaster": true
      }
    },
    {
      "name": "command_failure_observable",
      "passed": true,
      "evidence": {
        "output": "/bin/sh: definitely_missing_terminal_poc_command_123: command not found\\r\\n",
        "exitObserved": true,
        "status": 127,
        "cleanupClosedMaster": true
      }
    }
  ]
}
```

## Proof Criteria Result

Passed on macOS:

- PTY starts in the trusted repository root.
- Output streams as bounded renderer-style events with a 128-byte chunk cap.
- Input is sent through the backend-owned PTY master.
- Resize is observable by the child PTY session as `40 120`.
- Cancellation kills and reaps the child.
- Cleanup closes the PTY master descriptor.
- Command-not-found failure is observable with output and exit status.
- Out-of-root cwd is blocked before launch.
- High-impact command launch is approval-gated before spawn.

## Security Boundary Findings

- Renderer direct spawn remains unnecessary for the proven lifecycle.
- Approval gating can happen before native process launch.
- Trusted-root cwd validation can block out-of-root execution before launch.
- PTY output must remain untrusted renderer text.
- The POC does not inject provider credentials or app secrets into the child
  environment.

## Failure Or Inconclusive Criteria

Not failed on macOS.

Still inconclusive:

- Windows ConPTY behavior is not tested.
- Linux PTY behavior is not tested.
- Tauri command capability scoping is not wired or tested.
- Renderer event transport/backpressure is represented by bounded chunks but
  not tested through the production bridge.
- Production audit persistence is not implemented.

## Promotion Decision

Promote the native Terminal plugin from "blocked on Rust/Tauri PTY lifecycle"
to "eligible for a first narrow production slice after spec gating."

Do not promote this POC directly into product implementation. The first
production slice should implement only a backend-owned Terminal session model
and command lifecycle/audit contract, then a minimal Tauri command boundary
behind approval policy. Keep the MVP Terminal surface mocked until that slice
is explicitly accepted.
