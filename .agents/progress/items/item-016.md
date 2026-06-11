# item-016: Implement File Tool Policy Execution

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Execute project-root file reads and approved writes under the path resolver and
Approval Gateway.

## Inputs

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`

## Outputs

- File read executor.
- File write proposal/preview flow.
- Batch write caps.
- Denied/blocked write behavior.

## Constraints

- Writes do not execute before approval.
- Denied writes leave target files unchanged.
- Oversized or unsafe batches are blocked as a whole.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-015
- item-007

## Notes

- Verified on 2026-06-11 with allowed reads, secret-deny reads, denied write
  no-op, approved write, batch cap, Node tests, and native Tauri build.
