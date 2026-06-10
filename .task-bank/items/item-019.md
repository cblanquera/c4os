# item-019: Implement Tool Timeline And Ledger UI

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Show runtime tool activity, approvals, results, affected files, denial
categories, and bounded summaries in the session timeline.

## Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/shell-execution.md`

## Outputs

- Tool activity timeline data model.
- Tool ledger view state.
- Approval and denial result display state.
- Live/ephemeral shell drawer label state.

## Constraints

- No dedicated raw shell output export/copy control.
- Completed live terminal drawers are labeled live/ephemeral.
- Timeline uses persisted summaries or omitted markers, not raw stdout/stderr.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-010
- item-014
- item-018

## Notes

- Verified on 2026-06-11 with safe summary projection, live/ephemeral drawer
  label, no raw-output export flag, Node tests, and native Tauri build.
