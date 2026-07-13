# Plugin System And Settings Management Traceability

Status: proposed

## Requirement Coverage

| Requirement | Acceptance | QIDs | Source Note |
| --- | --- | --- | --- |
| REQ-001 | AC-001, AC-006 | 017, 038, 040, 2026-07-02 grill refinement | Codex plugin compatibility, plugin.json versus agents/c4os.yaml parser boundary, and C4OS app-shell metadata |
| REQ-002 | AC-002 | 005, 006, 015A, 016, 041 | User-global config, settings schema, and config.toml ownership |
| REQ-003 | AC-002, AC-008 | 016, 017, 2026-07-02 grill refinement | Field schema, field metadata, sensitive/visibleWhen behavior, unknown-key warnings, and shell-reserved key validation |
| REQ-004 | AC-001, AC-007 | 018, 019, 2026-07-02 grill refinements | Typed dependency model, separate dependencies manifest section, required/optional behavior, manual plugin enablement, visible dependency-blocked states, and cascading disable behavior |
| REQ-005 | AC-003, AC-004 | 039, 2026-07-02 grill refinement | Marketplace sources, cache, uninstall, reinstall, pending-restart backend registration, and disabled/uninstalled tool availability |
| REQ-006 | AC-002 | 044 | SVG icon source, sanitization, rendering, and theme rules |
| REQ-007 | AC-005 | 004, 040, 2026-07-02 grill refinement | Per-chat plugin instances are lightweight view/state instances; heavy services are lazily started and shared at narrowest safe scope |
| REQ-008 | AC-009 | 041, 043, 2026-07-02 grill refinement | Sensitive plugin settings use secure secret storage/keychain, config references/redacted placeholders, governed secret use, and hard redaction |

## Source Rule

This spec derives project-wide truth from `.agents/context/` and detailed evidence from `.agents/resources/research/`, `.agents/resources/grill/`, and `.agents/resources/history/`. It must not depend on sibling specs for project-wide truth.

## Reference Routing

- `.agents/resources/history/context/work-orders.md`
  Purpose: Shared accepted sequencing, guardrails, and pending-spec routing.
  Load when: checking whether this spec is proposed, frozen, or ready for execution conversion.
  Skip when: only reading local requirement coverage.

- `.agents/resources/grill/final-implementation/answers/`
  Purpose: Exact grill Q&A JSON source records.
  Load when: verifying a QID, answer wording, note, or superseded question.
  Skip when: local decisions and context already answer the planning question.

## POC Coverage

| Proof | Requirements | Evidence |
| --- | --- | --- |
| `proofs/plugin-settings-renderer/` | REQ-002, REQ-003, REQ-008 | EVD-POC-001 |
| `proofs/codex-marketplace-install-cache/` | REQ-005, REQ-011 | EVD-POC-002 |
| `proofs/plugin-svg-sanitization/` | REQ-006 | EVD-POC-003 |
| `proofs/plugin-lifecycle-pending-restart-and-service-scope/` | REQ-004, REQ-007, REQ-010, REQ-012 | EVD-POC-004 |
