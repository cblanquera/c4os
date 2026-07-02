# Runtime And Tool Policy Status

Status: proposed
Created: 2026-07-02

## Classification

- Planning stream: final implementation replay
- Source confidence: imported from exact grill answers and archive research
- Implementation state: not started
- Freeze state: not frozen

## Rules

- Do not create progress items from this spec until it is frozen and execution is explicitly requested.
- Do not implement product code during planning.
- Decisions live in this spec when they affect this spec.
- Shared reusable truth is promoted to `.agents/context/`; long rationale stays in `.agents/references/`.

## Freeze Gaps

- Acceptance criteria are planning acceptance only. Before freeze, convert them into executable acceptance with concrete verification method, evidence target, and pass/fail boundary.
- Proposed tasks are not implementation decomposition. Before execution, convert accepted scope into scoped work orders under `.agents/development/<spec-id>/` with non-conflicting task IDs.
- Wireframes and proofs are still pending where named by this spec. Do not treat this spec as implementation-ready until those records are created, reviewed, and accepted or explicitly deferred.
- `remember` approval semantics are resolved by the 2026-07-02 grill
  follow-up: remembered rules are keyed by tool/action/target scope with
  plugin id for plugin tools, duration is user-selected as session-only or
  user-global, and Settings > Configuration owns review/edit/revoke as
  per-server-tool policy items with tool explanations.

## Architect Resolution

- Runtime/tool open questions are resolved toward a C4OS-owned,
  MCP-shaped internal tool gateway with structured descriptors, typed request
  and result envelopes, lifecycle events, approval policy, traceability,
  cancellation, output caps, redaction metadata, and standards-first alignment.
