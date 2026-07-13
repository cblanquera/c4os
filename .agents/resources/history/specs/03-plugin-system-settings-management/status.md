# Plugin System And Settings Management Status

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
- Batch 2 Settings wireframes are accepted for this spec. Do not treat this spec
  as implementation-ready until acceptance criteria are converted into
  executable verification and scoped work orders are created after freeze.

## Prior Grill Overlaps And Remaining Question

- Codex plugin compatibility is accepted as the target and the core matrix is
  now covered by prior grill answers plus follow-up refinements: `plugin.json`
  remains the Codex compatibility surface; `agents/c4os.yaml` owns C4OS
  app-shell metadata; marketplace/cache/uninstall/reinstall behavior is
  sourced from Q039; native backend limits are sourced from Q040; schemaVersion
  incompatibility is sourced from Q038/Q050; typed dependencies and settings
  field metadata are recorded from the 2026-07-02 grill follow-up.
- Plugin settings marked `sensitive` are now resolved by the 2026-07-02 grill
  follow-up: secure secret storage/keychain owns raw values, config stores only
  references or redacted placeholders, and plugins use secrets through
  C4OS-governed calls rather than raw reads by default.

## Architect Resolution

- Plugin system open questions are resolved toward standards-first,
  metadata-first, inert-by-default discovery; C4OS-specific app-shell metadata
  in `agents/c4os.yaml`; Codex-compatible metadata in the Codex-standard
  manifest; memory-aware service lifecycles; Windows-ready platform adapters;
  and explicit module boundaries for maintainable implementation.
