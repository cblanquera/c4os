# item-015: Implement Session Allow And Structured Denial Results

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Apply narrow shell session allow and return structured denials for user-denied
or policy-blocked actions.

## Inputs

- `.agents/archive/planning-corpus/validation/FINDING-001-pre-execution-interception.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/security-and-permissions.md`

## Outputs

- Session allow matcher.
- Denial result mapper.
- Tool ledger denial display fields.

## Constraints

- Session allow only covers matching non-destructive shell commands inside the
  selected project root or approved subpath.
- Destructive commands, outside-root targets, secret-deny files, and Git state
  changes always require fresh approval or block.
- Runtime receives structured denial, not success or silence.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-014

## Notes

- Verified on 2026-06-11 with exact-match session allow, destructive command
  fresh approval, outside-root denial, user-denial result, Node tests, and
  native Tauri build.
