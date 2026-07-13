# Browser Plugin Status

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
- Batch 5 Browser wireframe evidence is accepted for this spec. Annotation,
  clear-after-send, hydration, local-file, profile isolation, and privileged
  bridge behavior that is not visible product UI remains covered by
  spec/Markdown evidence. Do not treat this spec as implementation-ready until
  acceptance criteria are converted into executable verification and scoped
  work orders are created after freeze.

## Resolved Prior Grill Overlaps

- Q036 is superseded by Q036A for document preview ownership. Q036A confirms
  document-family plugins own previews, Browser hosts compatible rendered
  output, Browser-native PDF preview is accepted, and advanced PDF support is a
  separate scope.
- Q042 has no submitted intake answer and is superseded by Q042A. Q042A covers
  screenshot scope, annotation persistence, target metadata, marker/comment
  model, multi-annotation support, and clear-after-send behavior.
- No additional Browser-specific grill question is currently needed from the
  Q036/Q042 overlap set.

## Architect Resolution

- Browser gaps are resolved toward plugin-owned Browser views, C4OS-owned
  Browser state and attachments, typed events, Codex-style annotations,
  document-family plugin preview ownership, platform/security provisions, and
  general worker evidence-capture workflows.
