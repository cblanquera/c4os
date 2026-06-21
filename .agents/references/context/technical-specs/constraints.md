# Product Constraints

Status: active
Created: 2026-06-18
Updated: 2026-06-21
Source Note: Imported from human planning sources and reconciled with accepted research decisions and POC results.

## Constraints

- C4OS is local-first by default.
- Project-local operations require explicit trusted-root containment.
- High-impact actions require app-owned approval policy before runtime execution.
- Shell commands, file writes/deletes, MCP mutation, credential use, networked tools, worktree mutation, share/export, and destructive actions are high-impact action categories.
- Raw API keys must be stored only in OS keychain or an equivalent secure storage abstraction.
- Workspace descriptor files must not contain raw secrets, full transcripts, artifact archives, or private operational state by default.
- Generated or untrusted HTML must render without privileged app APIs.
- External web browsing must be visually and technically separate from local artifact previews.
- Browser is a user-owned desktop surface with project-scoped profile. Agent Browser use is request-scoped, and agent file/Browser actions must be recorded.
- Browser implementation must not ignore the failed direct Tauri `WebviewWindow` proof: page content observed `window.__TAURI_INTERNALS__` in that POC. Local-file browsing, logged-in use, privileged bridge exposure, and agent authority require explicit handling.
- Terminal sessions must be backend-owned; renderer code must not spawn arbitrary shells directly.
- Terminal implementation still needs Tauri command capability scoping, renderer backpressure/audit persistence, and non-macOS PTY/ConPTY confirmation.
- File editing is user-controlled; agent-initiated mutation requires separate approval-boundary work.
- Hidden files and folders are visible in the file explorer except `.git`.
- Guarded delete requires exact-path confirmation.
- Marketplace readiness is an architecture goal, not MVP scope.
- Extensions are disabled by default before runtime execution, model context, tools, or app-owned state can be affected.
- Extensions can affect runtime execution, model context, tools, or app-owned state only after explicit per-extension enablement.
- Plugin, skill, and MCP install/connect flows are MVP scope.
- Concurrent sessions/runs must keep approvals, outputs, working directories, artifacts, runtime events, and cancellation state isolated.

## Open Validation Areas

- Browser implementation must reconcile desktop browsing, local-file browsing, project-scoped profiles, and request-scoped agent browsing with audit records.
- Terminal implementation must preserve backend ownership, deterministic command allowlist, approval policy, and command audit records.
- Credentialed OpenCode prompt execution and live permission-request capture still need validation because the POC avoided provider credentials and token spend.
