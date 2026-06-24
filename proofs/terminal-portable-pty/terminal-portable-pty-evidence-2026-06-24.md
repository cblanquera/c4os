# Terminal Portable PTY POC Evidence

Date: 2026-06-24
Status: passed-on-macos-ready-for-review
Scope: proof-only backend PTY validation

## Proof Artifact

- `proofs/terminal-portable-pty/`

## Intended Verification

```sh
cargo run --quiet --manifest-path proofs/terminal-portable-pty/Cargo.toml
```

## Verification Run

Command:

```sh
cargo run --quiet --manifest-path proofs/terminal-portable-pty/Cargo.toml
```

Result: passed.

Raw result:

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
        "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os",
        "output": "prompt:/Users/cblanquera/server/projects/cblanquera/c4os\r\nc4os % "
      },
      "name": "pty_starts_in_trusted_root",
      "passed": true
    },
    {
      "evidence": {
        "inputObserved": true,
        "output": "ls\r\ncommand:ls\r\noutput:ls\r\nsize:",
        "resizeObserved": true
      },
      "name": "stdin_output_resize",
      "passed": true
    },
    {
      "evidence": {
        "killRequested": true,
        "status": 1
      },
      "name": "termination_observable",
      "passed": true
    },
    {
      "evidence": {
        "output": "/bin/sh: definitely_missing_c4os_portable_pty_command: command not found\r\n",
        "status": 127
      },
      "name": "failure_observable",
      "passed": true
    }
  ],
  "passed": true,
  "platform": "macos",
  "poc": "terminal-portable-pty",
  "trustedRoot": "/Users/cblanquera/server/projects/cblanquera/c4os"
}
```

## Decision Target

If the proof passes, `portable-pty` is a viable candidate for the production
backend PTY layer. Production code should keep PTY/session/runtime details
behind C4OS-owned records and Tauri commands/events.
