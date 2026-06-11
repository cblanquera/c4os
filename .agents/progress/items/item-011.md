# item-011: Implement Instruction Preflight And Disclosure

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Inventory runtime-native instruction sources, redact config, disclose effective
sources before session start, and block undisclosable sources.

## Inputs

- `.agents/archive/planning-corpus/validation/FINDING-001-instruction-loading-observability.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-runtime-config-isolation.md`
- `.agents/archive/planning-corpus/validation/FINDING-014-runtime-fallback-decision.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/skills.md`

## Outputs

- Instruction-source scanner.
- Effective config capture with secret redaction.
- Session disclosure record.
- Block-on-undisclosable-source behavior.

## Constraints

- Root `AGENTS.md` display remains passive unless explicitly read.
- Sensitive config values must not be persisted raw.
- Session start must block when required source inventory cannot be produced.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-009
- item-008

## Notes

- Verified on 2026-06-11 with root/nested `AGENTS.md` inventory, config
  redaction, disclosure persistence, Node tests, and native Tauri build.
