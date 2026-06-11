# item-010: Implement Runtime Event Stream Normalizer

Status: verified
Type: implementation
Priority: high
Owner: codex

## Goal

Map OpenCode structured events into app-owned session, message, model, tool,
approval, denial, error, stop, and idle records.

## Inputs

- `.agents/archive/planning-corpus/validation/FINDING-001-structured-events.md`
- `.agents/archive/planning-corpus/validation/FINDING-001-app-owned-record-mapping.md`
- `.agents/archive/planning-corpus/acceptance/agent-execution.md`
- `.agents/archive/planning-corpus/acceptance/sessions.md`

## Outputs

- Event normalization layer.
- App-owned record writers for initial message/tool/status events.
- Regression fixtures for observed event payload shapes.

## Constraints

- Runtime-native IDs are adapter metadata only.
- App-owned records remain canonical.
- Tool activity must be normalizable without terminal scraping.

## Acceptance Criteria

- Runtime records are never the source of truth.
- Message and tool activity events map into app-owned records.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run tauri -- build`

## Dependencies

- item-009
- item-002

## Notes

- Verified on 2026-06-11 with fixture-based session status, message, and tool
  event mapping tests, Node tests, and native Tauri build.
