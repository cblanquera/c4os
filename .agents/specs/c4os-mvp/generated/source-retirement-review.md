# Source Retirement Review

Date: 2026-06-11

## Summary

The human planning corpus has been archived into
`.agents/archive/planning-corpus/`, and compact durable facts have been
extracted into `.agents/specs/c4os-mvp/`. `.task-bank/` has been migrated into
`.agents/progress/` and should remain in place as a deprecated historical
source until the user explicitly confirms deletion.

| Source | Role | Extracted To | Remaining Unique Value | Recommendation |
| --- | --- | --- | --- | --- |
| `CONTEXT.md` | Product vocabulary, MVP boundaries, durable constraints | `.agents/specs/c4os-mvp/records/requirements.md`, `constraints.md` | Full glossary and precise avoid-language remain authoritative. | keep |
| Repository-root human planning corpus | Original human planning corpus, acceptance, ADRs, validation history, implementation sequencing | `.agents/archive/planning-corpus/`; `.agents/specs/c4os-mvp/records/`; `indexes/`; `imports/source-documents.md` | None after archive verification; `.agents/archive/planning-corpus/` is the retained detailed copy. | delete-after-confirmation |
| `.agents/archive/planning-corpus/` | Archived human planning corpus, acceptance, ADRs, validation history, implementation sequencing | current retained archive | Detailed rationale, validation history, acceptance documents, and stakeholder context remain authoritative here. | keep |
| `.task-bank/` | Legacy progress bank | `.agents/progress/`; `.agents/specs/c4os-mvp/records/evidence.md`; `.agents/specs/c4os-mvp/records/decisions.md` | Historical source and unreviewed local changes may remain useful until the user confirms removal. | archive |
| `.agents/progress/` | Active progress bank | current active source | None; this is the replacement source. | keep |
| `.agents/specs/c4os-mvp/` | Durable AI-readable spec layer | current active source | None; this is the replacement source. | keep |

## Retirement Decision

The repository-root human planning corpus can be removed after user
confirmation because a verified archive now exists under
`.agents/archive/planning-corpus/` and compact routing records exist under
`.agents/specs/c4os-mvp/`.

`.task-bank/` remains deprecated, not deleted. Use `.agents/progress/` for new
progress updates and `.agents/specs/c4os-mvp/` for durable planning records.
