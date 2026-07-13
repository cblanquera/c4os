# Runtime Adapter Context

Status: active
Created: 2026-06-21
Updated: 2026-06-26
Source Note: Promoted from accepted research POC records.

## Runtime Direction

- The first implementation targets OpenCode behind a thin C4OS-owned runtime adapter.
- Pi remains a later adapter target, not an MVP implementation target.
- C4OS-owned product records, UI, approvals, artifacts, provider settings, extension enablement, persistence, and audit state remain outside runtime ownership.
- OpenCode and Pi concepts must not become the primary user-facing product model.

## OpenCode Findings

- OpenCode proved local server startup, project/path inspection, session creation, session resume, server/session event streaming, permission configuration, and session abort.
- Credentialed model-backed prompt execution and live permission request capture were not proven in the POC because the proof avoided provider credentials and token spend.
- OpenCode defaults to user-level state and config paths unless C4OS supplies isolated runtime environment and config paths.

## Pi Findings

- Pi proved session creation/resume, prompt execution through `Agent.prompt`, lifecycle/tool event streaming, pre-execution tool interception, approval denial, and run lifecycle control.
- Artifact identity remains a C4OS concern layered on top of transcript and tool-result state.
- Pi does not provide a built-in filesystem, process, network, or credential permission sandbox; C4OS must own those boundaries.

## Implementation Implications

- Treat OpenCode as the first backend for server-mode coding workflow, existing CLI ecosystem compatibility, sessions, and event APIs.
- Preserve the adapter boundary so a future Pi adapter can be added without changing the product model.
- Implement runtime state paths, config paths, permission decisions, artifact mapping, and audit records as C4OS-owned concerns.

## Future Runtime Tool Gateway

The long-term command and tool path should trust the runtime to decide what
tool to request, but not trust the runtime to execute tools directly. C4OS
should formalize an MCP-shaped tool gateway:

1. User prompt enters the runtime adapter.
2. Runtime streams status, assistant deltas, and tool-call lifecycle events.
3. Runtime requests a tool by stable tool identity and structured args.
4. C4OS evaluates per-session tool config, approval policy, trusted-root
   containment, and tool-specific authority.
5. C4OS executes or rejects the tool through product-owned implementations.
6. C4OS streams tool output/error events back to the runtime and UI.
7. Runtime produces the final assistant response.
8. C4OS persists normalized session, run, message, tool action, approval, and
   audit records.

Tool identities should be stable capability names, not event names:

```json
{
  "tools": {
    "terminal.run": {
      "enabled": true,
      "approval": "ask_every_time",
      "access": "workspace_shell"
    },
    "files.read": {
      "enabled": true,
      "approval": "auto_readonly",
      "access": "workspace_read"
    },
    "files.write": {
      "enabled": true,
      "approval": "ask_every_time",
      "access": "workspace_write"
    },
    "browser.open": {
      "enabled": true,
      "approval": "auto_public_urls",
      "access": "public_or_trusted_local"
    }
  }
}
```

Runtime events should describe lifecycle, for example:

```json
{
  "kind": "tool_call_requested",
  "tool": "terminal.run",
  "args": {
    "command": "git status"
  },
  "sessionId": "c4os-session-run-git-status"
}
```

The tool implementation may define default and maximum approval levels. Session
configuration may narrow authority, but it must not silently widen beyond the
tool maximum. If a session requests wider authority than the tool permits, C4OS
should clamp or reject the config.

The current explicit prompt command bridge from TASK-011A/TASK-011B is a
transitional UX bridge. It should not keep expanding into natural-language
command planning. Future work should remove frontend command parsing in favor
of runtime tool-call requests flowing through this gateway.
