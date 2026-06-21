# Runtime Adapter Context

Status: active
Created: 2026-06-21
Updated: 2026-06-21
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
