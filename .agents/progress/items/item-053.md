# item-053: Foundation Workflow Label Data Model And Status

## Status

verified

## Type

implementation

## Objective

Add the foundation data-model and backend status support for lightweight
workflow-purpose labels on projects and sessions.

## Spec Links

- `.agents/specs/c4os-general-workspace-foundation/records/requirements.md`
- `.agents/specs/c4os-general-workspace-foundation/records/decisions.md`
- `.agents/specs/c4os-general-workspace-foundation/records/acceptance.md`
- `.agents/specs/c4os-general-workspace-foundation/indexes/traceability.md`

## Scope

- Add nullable bounded workflow-purpose metadata for projects and sessions.
- Keep allowed purposes limited to research, writing, documentation, analysis,
  or unset.
- Expose backend status/capability flags for workflow labels.
- Preserve selected-project, one-active-run, approval, file, shell, Git,
  credential, and model-context behavior.
- Update project JSON export to include workflow labels as safe metadata while
  preserving `project_json_export_only`.

## Non-Goals

- Do not support non-Git folders.
- Do not add browser/web, durable memory, import, provider expansion, plugins,
  worktrees, or concurrent agents.
- Do not auto-inject workflow labels or related files into model context.

## Acceptance Criteria

- Existing databases migrate without data loss.
- Project and session records can store unset or allowed workflow-purpose
  values.
- Invalid workflow-purpose values are rejected or normalized according to a
  documented backend rule.
- Export includes labels as safe metadata and preserves all existing security
  exclusions and no-import/no-round-trip claims.
- Backend status exposes workflow-label support without broad compatibility or
  hidden context claims.

## Verification

- Focused Rust migration/storage/export tests passed.
- Targeted JS backend status tests passed.
- Full `cargo test`, full `npm test`, and `npm run build` passed.
