# Native Terminal Plugin POC Evidence: 2026-06-15

Status: passed
Started: 2026-06-15T08:48:30Z
Finished: 2026-06-15T08:48:30Z

## Scope

Disposable stdlib PTY lifecycle POC for backend-owned terminal
execution semantics. This is not product Terminal implementation.

## Checks

- PASS: project-scoped PTY starts and streams output (6ms)
- PASS: PTY accepts input and associates output with session/run (63ms)
- PASS: PTY resize event is observable (7ms)
- PASS: trusted-root policy blocks out-of-root cwd (0ms)
- PASS: approval gate blocks high-impact command before launch (0ms)
- PASS: failure states are observable (6ms)
- PASS: cancellation terminates and cleanup closes PTY (28ms)

## Security Findings

- Renderer direct spawn: blocked by design; renderer submits requests only
- Approval before launch: required for high-impact commands
- Trusted root: /Users/cblanquera/server/projects/cblanquera/c4os2
- Environment: minimal PATH/TERM only; provider credentials omitted
- Output handling: terminal output is untrusted event text

## Limits

- This POC uses Python stdlib PTY primitives, not Rust/Tauri PTY crates.
- It proves backend-owned PTY semantics are viable on this macOS host.
- Cross-platform Windows/Linux behavior and Tauri capability wiring
  still require a follow-up native Rust/Tauri POC.

## Raw Result

```json
{
  "startedAt": "2026-06-15T08:48:30Z",
  "trustedRoot": "/Users/cblanquera/server/projects/cblanquera/c4os2",
  "disallowedRoot": "/private/tmp",
  "checks": [
    {
      "name": "project-scoped PTY starts and streams output",
      "status": "pass",
      "duration_ms": 6,
      "evidence": {
        "output": "stream-one\r\nstream-two\r\n",
        "events": [
          {
            "kind": "started",
            "pid": 77869,
            "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os2",
            "sessionId": "poc-session",
            "runId": "poc-run"
          },
          {
            "kind": "output",
            "text": "stream-one\r\nstream-two\r\n"
          },
          {
            "kind": "exit",
            "status": 0
          },
          {
            "kind": "cleanup",
            "fdClosed": true
          }
        ],
        "exitStatus": null
      }
    },
    {
      "name": "PTY accepts input and associates output with session/run",
      "status": "pass",
      "duration_ms": 63,
      "evidence": {
        "output": "hello-from-renderer-request\r\necho:hello-from-renderer-request\r\n",
        "events": [
          {
            "kind": "started",
            "pid": 77870,
            "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os2",
            "sessionId": "poc-session",
            "runId": "poc-run"
          },
          {
            "kind": "input",
            "bytes": 28
          },
          {
            "kind": "output",
            "text": "hello-from-renderer-request\r\n"
          },
          {
            "kind": "output",
            "text": "echo:hello-from-renderer-request\r\n"
          },
          {
            "kind": "cleanup-signal",
            "signal": "SIGTERM",
            "target": "process"
          },
          {
            "kind": "exit",
            "status": 0
          },
          {
            "kind": "cleanup",
            "fdClosed": true
          }
        ],
        "exitStatus": null
      }
    },
    {
      "name": "PTY resize event is observable",
      "status": "pass",
      "duration_ms": 7,
      "evidence": {
        "output": "resize-ready\r\n",
        "resizeEvents": [
          {
            "kind": "resize",
            "rows": 33,
            "cols": 101
          }
        ]
      }
    },
    {
      "name": "trusted-root policy blocks out-of-root cwd",
      "status": "pass",
      "duration_ms": 0,
      "evidence": {
        "blocked": true,
        "reason": "cwd outside trusted root: /private/tmp"
      }
    },
    {
      "name": "approval gate blocks high-impact command before launch",
      "status": "pass",
      "duration_ms": 0,
      "evidence": {
        "action": "blocked",
        "reason": "high-impact command requires explicit approval",
        "command": "rm -rf ./target",
        "source": "agent",
        "launched": false
      }
    },
    {
      "name": "failure states are observable",
      "status": "pass",
      "duration_ms": 6,
      "evidence": {
        "output": "bash: definitely-missing-command-c4os-poc: command not found\r\n",
        "events": [
          {
            "kind": "started",
            "pid": 77872,
            "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os2",
            "sessionId": "poc-session",
            "runId": "poc-run"
          },
          {
            "kind": "output",
            "text": "bash: definitely-missing-command-c4os-poc: command not found\r\n"
          },
          {
            "kind": "exit",
            "status": 127
          },
          {
            "kind": "cleanup",
            "fdClosed": true
          }
        ],
        "exitStatus": 127
      }
    },
    {
      "name": "cancellation terminates and cleanup closes PTY",
      "status": "pass",
      "duration_ms": 28,
      "evidence": {
        "output": "sleeping\r\n",
        "events": [
          {
            "kind": "started",
            "pid": 77873,
            "cwd": "/Users/cblanquera/server/projects/cblanquera/c4os2",
            "sessionId": "poc-session",
            "runId": "poc-run"
          },
          {
            "kind": "output",
            "text": "sleeping\r\n"
          },
          {
            "kind": "cancel",
            "signal": "SIGTERM",
            "target": "process"
          },
          {
            "kind": "exit",
            "status": "signal:15"
          },
          {
            "kind": "cleanup",
            "fdClosed": true
          }
        ],
        "exitStatus": "signal:15",
        "cancelled": true
      }
    }
  ],
  "policy": {
    "rendererDirectSpawn": "blocked by design; renderer submits requests only",
    "approvalBeforeLaunch": "required for high-impact commands",
    "trustedRoot": "/Users/cblanquera/server/projects/cblanquera/c4os2",
    "environment": "minimal PATH/TERM only; provider credentials omitted",
    "outputHandling": "terminal output is untrusted event text"
  },
  "finishedAt": "2026-06-15T08:48:30Z",
  "summary": {
    "status": "passed",
    "passed": 7,
    "failed": 0
  }
}
```
