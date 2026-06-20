# Product Constraints

Status: active
Created: 2026-06-18
Updated: 2026-06-18
Source Note: Imported from `plans/product-brief.md`.

## Constraints

- C4OS is local-first by default.
- Project-local operations require explicit trusted-root containment.
- High-impact actions require app-owned approval policy before runtime execution.
- Raw API keys must be stored only in OS keychain or an equivalent secure storage abstraction.
- Workspace descriptor files must not contain raw secrets, full transcripts, artifact archives, or private operational state by default.
- Generated or untrusted HTML must render without privileged app APIs.
- Browser is a user-owned desktop surface with project-scoped profile. Agent Browser use is request-scoped, and agent file/Browser actions must be recorded.
- Terminal sessions must be backend-owned; renderer code must not spawn arbitrary shells directly.
- File editing is user-controlled; agent-initiated mutation requires separate approval-boundary work.
- Extensions can affect runtime execution, model context, tools, hooks, or app-owned state only after explicit per-extension enablement.
- Plugin, skill, and MCP install/connect flows are MVP scope.

## Open Validation Areas

- Browser implementation must reconcile desktop browsing, local-file browsing, project-scoped profiles, and request-scoped agent browsing with audit records.
- Terminal implementation must preserve backend ownership, deterministic command allowlist, approval policy, and command audit records.
- Concurrent session isolation across approvals, outputs, working directories, artifacts, runtime events, and cancellation state.
