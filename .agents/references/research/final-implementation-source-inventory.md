# Final Implementation Source Inventory

Status: active-reference
Created: 2026-07-02

## Purpose

This records the import provenance for the final-implementation planning
replay. `.agents/references/research/final-implementation-import` is source material only; live project truth is
promoted into `.agents/context/`, `.agents/references/`, and the affected spec
packages.

## Source Classification

| Source | Type | Count | Use |
| --- | --- | ---: | --- |
| `.agents/references/research/final-implementation-import/adhoc-goals.md` | goal input | 1 | Original desired final-implementation feature/spec goals. |
| `.agents/references/research/final-implementation-import/accepted-instructions.md` | instruction | 1 | Accepted process, spec package, research, grill, and workflow repair instructions. |
| `.agents/references/research/final-implementation-import/grill-session/*.json` | exact grill answer | 59 | Accepted decision answers and notes; Q042 is superseded by Q042A. |
| `.agents/references/research/final-implementation-import/grill-schemas/grill-question-*.json` | grill schema | 59 | Question schemas/options used to interpret each grill answer. |
| `.agents/references/research/final-implementation-import/research/*.md` | research evidence | 3 | Tauri, Codex plugin/marketplace, modular shell, document preview, and AI harness research. |

## Confidence

- Grill JSON files are treated as exact recovered answer records from the
  referenced Codex thread.
- Research notes are supporting evidence, not product truth until promoted into
  context or spec evidence.
- Original goals and accepted instructions are user intent and process
  constraints for this planning replay.
- Archive files are not live source of truth after import.

## Required Reconciliation

- Every answered grill QID must be represented in affected spec decisions or
  explicitly marked superseded, rejected, or deferred.
- Q042 must be marked superseded by Q042A.
- Every represented decision must cite its source JSON path.
- Shared reusable truth must be compacted into one of the five
  `.agents/context/` files.
- Long rationale, research, and provenance must remain in `.agents/references/`.

## Research Evidence Files

- `.agents/references/research/final-implementation-import/research/plugin-shell-research-2026-06-30.md`
- `.agents/references/research/final-implementation-import/research/plugin-shell-research-pass-2-2026-07-01.md`
- `.agents/references/research/final-implementation-import/research/codex-plugin-marketplace-research-2026-07-01.md`
