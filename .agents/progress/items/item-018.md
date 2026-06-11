# item-018: Implement Shell Output Summary Policy

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Persist bounded redacted/truncated shell summaries and omit output when a safe
summary cannot be produced.

## Inputs

- `.agents/archive/planning-corpus/validation/FINDING-003-shell-security-policy.md`
- `.agents/archive/planning-corpus/acceptance/shell-execution.md`
- `.agents/archive/planning-corpus/acceptance/openrouter-integration.md`

## Outputs

- Redaction matcher set.
- Summary byte/line/line-length caps.
- Safe reason labels.
- Output-omitted marker.
- No raw stdout/stderr fallback.

## Constraints

- Do not persist raw stdout/stderr as fallback.
- Do not persist redacted substrings, raw byte counts, offsets, hashes, or
  reconstruction metadata.
- Model context may use only the persisted summary or omitted marker and safe
  labels.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-017

## Notes

- Verified on 2026-06-11 with secret redaction, large output truncation,
  binary/untrusted encoding omission, private-key redaction, Node tests, and
  native Tauri build.
