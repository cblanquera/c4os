# item-006: Implement Project Registration And Git Status

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Register local Git projects and show root, branch, dirty state, and changed
files.

## Inputs

- `plans/acceptance/project-management.md`
- `plans/acceptance/git-integration.md`

## Outputs

- Folder/path registration backend contract.
- Git project validation.
- Project record persistence.
- Branch, dirty, and changed-file status.

## Constraints

- Non-Git and unreadable paths must fail clearly.
- Git inspection is visible but not approval-gated.
- State-changing Git workflows remain out of MVP scope.

## Acceptance Criteria

- Git projects can be registered and reopened.
- Non-Git or unreadable paths fail clearly.
- Git status includes root, branch, dirty state, and changed files.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-002

## Notes

- Verified on 2026-06-11 with temporary Git repository tests, Node tests, and
  native Tauri build.
