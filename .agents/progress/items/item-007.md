# item-007: Implement Project-Root Path Resolver

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Centralize project-root containment, symlink resolution, traversal handling,
and secret-deny file blocking.

## Inputs

- `.agents/archive/planning-corpus/acceptance/file-access.md`
- `.agents/archive/planning-corpus/specs/security-specification.md`

## Outputs

- Path resolver service.
- Secret-deny matcher.
- Boundary-check test fixtures.

## Constraints

- Resolve traversal and symlinks before boundary decisions.
- Reads and writes through outside-root symlinks must be blocked.
- Secret-deny files have no MVP approval override.

## Acceptance Criteria

- Outside-root paths are blocked.
- Symlinks are resolved before access decisions.
- Secret-deny files are blocked for agent reads and writes.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-006

## Notes

- Verified on 2026-06-11 with traversal, symlink, secret-deny, Node, and native
  Tauri build checks.
