# item-014: Implement Approval Prompt State And Ledger

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Create pending approval state, user decisions, durable answered approval
records, and non-durable pending approval discard behavior.

## Inputs

- `plans/acceptance/security-and-permissions.md`
- `plans/acceptance/file-access.md`

## Outputs

- Pending approval model.
- Approval API.
- Durable answered approval ledger records.
- Restart discard behavior for pending approvals.

## Constraints

- Answered approvals are durable.
- Pending prompts are not durable approvable decisions.
- Approval records must not store raw command output, raw secrets, or full
  prompt replay blobs.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-013

## Notes

- Verified on 2026-06-11 with answered approval persistence, pending approval
  restart discard, missing-pending fail-closed, Node tests, and native Tauri
  build.
