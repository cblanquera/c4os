# Shell Plugin Architecture Refactor Status

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

## Resolved Planning Gaps

- Architecture model, glossary, named authority boundaries, lifecycle state
  machine, and migration failure classification are now recorded in
  `decisions.md`.
- Backend architect profile alignment is now recorded in `decisions.md`,
  including standards precedence, MCP-shaped tool structure, event-driven
  communication, memory/RAM posture, Windows platform adapters, worker-friendly
  shell/plugin scope, and maintainable authority boundaries.
- `TASK-001` and `TASK-002` are resolved as planning records only. They are not
  active progress items and do not make the spec frozen for implementation.
