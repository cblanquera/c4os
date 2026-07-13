# Final Implementation Import Evidence

Status: imported-reference
Created: 2026-07-02

## Purpose

This folder preserves the exact import evidence used to rebuild the
final-implementation planning docs. It is the retained `.agents` copy of the
goal inputs, accepted process instructions, exact grill answers, grill schemas,
and research notes.

## Contents

| Path | Count | Meaning |
| --- | ---: | --- |
| `adhoc-goals.md` | 1 | Original feature/spec goals. |
| `accepted-instructions.md` | 1 | Accepted process and workflow instructions. |
| `grill-session/*.json` | 59 | Exact recovered grill answer records, including raw intake text where present. |
| `grill-schemas/grill-question-*.json` | 59 | Question schemas/options used for the grill. |
| `research/*.md` | 3 | Research notes for Tauri, Codex plugins/marketplace, modular shell, document preview, and AI harnesses. |

## Rules

- This folder is evidence/provenance, not live product truth.
- Promote compact accepted reusable facts into `.agents/context/`.
- Put spec-specific decisions in the affected spec's `decisions.md`.
- Keep this folder available until a source-retirement pass proves it is no
  longer needed.
