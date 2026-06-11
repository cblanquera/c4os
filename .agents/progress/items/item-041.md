# item-041: TASK-040 Scope V1 Session And Artifact Export Import

## Status

verified

## Objective

Choose and document whether V1 should support any session or artifact
export/import behavior before implementation.

## Dependencies

- item-040

## Scope

- Revisit session and artifact export/import after passive raster image
  previews.
- Define export format, absolute-path handling, secret exclusion,
  artifact-type behavior, raw-output exclusion, provider-secret exclusion, and
  compatibility claims.
- Import is deferred while export is scoped.
- Implement accepted `project_json_export_only` tier.

## Inputs

- `.agents/progress/items/item-032.md`
- `.agents/archive/planning-corpus/specs/data-model-specification.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-008-storage-audit-portability.md`
- `.agents/archive/planning-corpus/spikes/SPIKE-014-memory-session-retention.md`
- `.agents/archive/planning-corpus/adr/005-standards-first-interoperability.md`
- `.agents/archive/planning-corpus/adr/007-local-first-storage-model.md`

## Deliverables

- Accepted or deferred V1 support tier for session and artifact export/import.
- Updated acceptance criteria for export/import behavior.
- Updated deferred-decision and compatibility notes.
- Follow-on implementation item only if a narrow tier is accepted.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml export`.
- `npm test -- tests/backend-command-boundary.test.mjs`.
- `npm test`.
- `npm run build`.
- `cargo test --manifest-path src-tauri/Cargo.toml`.
- `npm run tauri -- build`.

## Handoff

Accepted tier: `project_json_export_only`.

Implement a selected-project JSON export manifest with portable project
metadata, sessions, messages, safe tool summaries, and artifact metadata.
Import, round-trip compatibility, raw secrets, provider keys, raw shell
stdout/stderr, absolute local paths, and artifact binary payload export remain
unavailable.

## Result

- Added selected-project JSON export support.
- App status exposes `project_json_export_only`, export available, import
  unavailable, and no round-trip compatibility claim.
- Export redacts secret-shaped message and tool-summary lines.
- Export omits absolute local paths, provider keys, raw shell stdout/stderr,
  and artifact payloads.
