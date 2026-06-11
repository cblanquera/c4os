# item-017: Implement Shell Executor And Environment Filter

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Execute approved shell commands with project-root working directories,
filtered environment, and app-supervised process handling.

## Inputs

- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`

## Outputs

- Shell executor.
- Environment filter.
- Supervised process result.

## Constraints

- Shell commands run as current OS user with normal network access.
- The app must not claim strong sandboxing.
- Secret-shaped environment variables must be stripped.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-015

## Notes

- Verified on 2026-06-11 with secret environment stripping, working-directory
  enforcement, outside-root block, Node tests, and native Tauri build.
