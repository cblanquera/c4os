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
- Browser content must not access provider credentials, arbitrary workspace files, shell state, or privileged app APIs.
- Terminal sessions must be backend-owned; renderer code must not spawn arbitrary shells directly.
- File editing is user-controlled; agent-initiated mutation requires separate approval-boundary work.
- Extensions must be disabled by default before they can affect runtime execution, model context, tools, hooks, or app-owned state.
- Marketplace readiness is an architecture goal, not MVP scope.

## Open Validation Areas

- Runtime lifecycle stability for OpenCode Runtime or Pi Runtime behind an adapter.
- Cross-platform native browser isolation behavior.
- Backend-owned terminal lifecycle and event transport.
- Concurrent session isolation across approvals, outputs, working directories, and artifacts.
