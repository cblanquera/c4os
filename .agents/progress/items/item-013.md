# item-013: Implement Protected Action Classifier

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Classify runtime-proposed file writes, shell commands, Git state changes,
read-only Git inspection, file reads, blocked policy actions, and MVP-scope
blocks.

## Inputs

- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`
- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`
- `.agents/archive/planning-corpus/acceptance/git-integration.md`

## Outputs

- Action classifier.
- Risk category model.
- Denial category model.
- Tests for protected and policy-allowed actions.

## Constraints

- File writes, shell commands, and Git state changes require approval.
- Read-only project-root file reads and Git inspection are logged without
  approval.
- Outside-root and secret-deny actions are blocked.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-007
- item-010

## Notes

- Verified on 2026-06-11 with file read/write, secret-deny, shell high-risk,
  Git inspection/state-change, Node, and native Tauri build tests.
