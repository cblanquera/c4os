# WO-006 Runtime Tool Gateway Refactor

Status: ready
Updated: 2026-06-30

## Source

- Work order: `.agents/context/work-orders.md` `WO-006`
- Technical specs: `.agents/context/technical-specs.md` `Runtime Boundary`
- Runtime reference:
  `.agents/references/context/technical-specs/runtime-adapter.md`
  `Future Runtime Tool Gateway`
- Accepted predecessor:
  `.agents/development/progress/items/TASK-015-pause-at-feature-complete.md`
- Deferred successor: `.agents/specs/mvp/tasks.md` `TASK-016`

## Goal

Refactor runtime-driven tool execution so prompts flow through a formal
C4OS-owned tool gateway before `TASK-016` security and approval hardening.

The app should stop treating the TASK-011A/TASK-011B explicit command parser as
the durable command-planning mechanism. Prompt planning belongs to the runtime;
tool execution, trusted-root authority, approval decisions, persistence, and
audit records belong to C4OS.

## Required Scope

- Keep the accepted r04 shell and existing Browser / Files / Terminal right
  panel contract.
- Preserve TASK-015 feature-complete behavior while changing the internal
  runtime/tool execution path.
- Define stable C4OS tool identities such as `terminal.run`, `files.read`,
  `files.write`, `browser.open`, and `artifact.preview`.
- Define lifecycle events separately from tool identities, including
  `tool_call_requested`, `tool_call_started`, `tool_output_delta`,
  `tool_call_completed`, `tool_call_rejected`, and `final_response`.
- Add or refactor backend runtime adapter code so a runtime can request tools
  by stable identity and structured args.
- Route tool requests through C4OS-owned authority boundaries for workspace
  root, Files, Browser, Terminal, artifact preview, records, and audit.
- Introduce per-session tool configuration that maps tool identities to
  enabled state, access level, and approval policy.
- Let tool implementations define default and maximum approval levels.
  Per-session config may narrow authority but must not silently widen it.
- Preserve existing action/audit record persistence and extend it where needed
  for normalized tool-call records.
- Keep the transitional explicit command prompt path only as compatibility
  input that creates a `terminal.run` request through the gateway.

## Out Of Scope

- Broad `TASK-016` security and approval hardening.
- Final keychain credential policy.
- New visible routes, panels, settings categories, or alternate layouts.
- Extension invocation, skill/plugin loading, MCP execution, or prompt-tag
  runtime impact beyond modeling them as future gateway-capable tool families.
- Browser downloads.
- Release packaging and final MVP acceptance closeout.

## Expected Implementation Paths

- `backend/`
- `frontend/` only where connector/runtime event handling must change.
- `tests/server/`
- Existing frontend tests when user-visible behavior must be protected.
- `.agents/development/progress/`

## Verification Plan

- Add failing backend tests for stable tool identity dispatch, lifecycle event
  normalization, per-session config narrowing, and max-approval clamping or
  rejection.
- Add focused regression coverage proving `run ls` still updates the bottom
  Agent command terminal through `terminal.run`, not through an ad hoc parser
  execution path.
- Add or update Browser/Files/Terminal record tests so gateway executions still
  mirror action and audit records.
- Run relevant TASK-011, TASK-011A, TASK-011B, TASK-013, TASK-014, and
  connector regression tests.
- Run `cargo test --manifest-path backend/Cargo.toml -- --test-threads=1`.
- Run `npm test` or the smallest justified frontend/server suite if full
  browser harness execution is blocked by local loopback permissions.
- Run `cargo fmt --manifest-path backend/Cargo.toml -- --check`.
- Run `git diff --check`.

## Acceptance Boundary

This item is complete when C4OS has a formal runtime tool gateway boundary that
the current Terminal command path can use without changing the accepted UI.
`TASK-016` can then harden trusted roots, approvals, Browser authority,
Terminal policy, extension gates, and audit logs against the gateway-backed
surface instead of hardening the transitional parser directly.
