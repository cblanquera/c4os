# Core App Shell UX Status

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
- Non-Batch-1 wireframes and executable acceptance are still pending where
  named by this spec. The shell panel POC and approved r05 Batch 1 shell
  foundation wireframes exist, but do not treat this spec as
  implementation-ready until remaining records are created, reviewed, and
  accepted or explicitly deferred.

## Architect Resolution

- Shell UX open questions are resolved toward a thin event-driven shell with
  plugin-owned enhanced work surfaces, worker-friendly language, and a
  documented layout state machine.
- Approved r05 Batch 1 shell-foundation wireframe evidence is recorded in
  `evidence.md` and covers the named shell layout states needed for pending
  final-implementation spec review. This closes only the approved Batch 1
  wireframe gap; it does not freeze this spec or start implementation.
