# Runtime And Tool Policy

Status: proposed
Created: 2026-07-02

## Purpose

Define runtime/tool gateway, app-tool policy, config.toml, Pi proof, discovery, approval defaults, and command routing.

## Source Boundaries

This spec derives from shared context, references, original goal inputs, and the exact grill answer JSON. It must not depend on sibling specs for project-wide truth.

## Sources

- `.agents/references/research/final-implementation-import/adhoc-goals.md`
- `.agents/references/research/final-implementation-import/accepted-instructions.md`
- `.agents/references/research/final-implementation-source-inventory.md`
- `.agents/context/technical-specs.md`
- `.agents/context/work-orders.md`
- `.agents/references/research/final-implementation-import/research/plugin-shell-research-pass-2-2026-07-01.md`
- `.agents/references/research/final-implementation-import/grill-session/002-c4os-grill-question-002-plugin-backend-authority.json`
- `.agents/references/research/final-implementation-import/grill-session/002a-c4os-grill-question-002a-tauri-tool-authority-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/002b-c4os-grill-question-002b-runtime-tool-discovery-and-invocation.json`
- `.agents/references/research/final-implementation-import/grill-session/003a-c4os-grill-question-003a-tool-event-fanout.json`
- `.agents/references/research/final-implementation-import/grill-session/020-c4os-grill-question-020-app-tool-approval-defaults.json`
- `.agents/references/research/final-implementation-import/grill-session/021-c4os-grill-question-021-core-tool-approval-defaults.json`
- `.agents/references/research/final-implementation-import/grill-session/021a-c4os-grill-question-021a-read-approval-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/022-c4os-grill-question-022-mutating-tool-defaults.json`
- `.agents/references/research/final-implementation-import/grill-session/023-c4os-grill-question-023-non-file-risk-tool-defaults.json`
- `.agents/references/research/final-implementation-import/grill-session/024-c4os-grill-question-024-browser-tool-defaults.json`
- `.agents/references/research/final-implementation-import/grill-session/025-c4os-grill-question-025-prompt-tag-routing.json`
- `.agents/references/research/final-implementation-import/grill-session/040-c4os-grill-question-040-plugin-backend-registration-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/041-c4os-grill-question-041-config-toml-and-tool-policy.json`
- `.agents/references/research/final-implementation-import/grill-session/045-c4os-grill-question-045-model-attachment-compatibility.json`
- `.agents/references/research/final-implementation-import/grill-session/047-c4os-grill-question-047-terminal-plugin-tool-boundary.json`
- `.agents/references/research/final-implementation-import/grill-session/049-c4os-grill-question-049-app-tool-taxonomy-and-pi-proof.json`

## Package Files

- `status.md`
- `requirements.md`
- `acceptance.md`
- `decisions.md`
- `risks.md`
- `evidence.md`
- `tasks.md`
- `traceability.md`
- `poc/index.md`
