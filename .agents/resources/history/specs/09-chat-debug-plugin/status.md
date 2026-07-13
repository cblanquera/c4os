# Chat Debug Plugin Status

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
- Batch 5 Chat Debug wireframe evidence is accepted for this spec. Do not
  treat this spec as implementation-ready until acceptance criteria are
  converted into executable verification and scoped work orders are created
  after freeze.

## Architect Resolution

- Chat Debug gaps are resolved toward typed observability events, redaction
  before persistence/display, bounded per-chat history, no export, and
  support-friendly diagnostic language.
