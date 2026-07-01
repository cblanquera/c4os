# Import Workflow

Use this when existing planning material should become compact `.agents` records.

## Read First

- `.agents/AGENTS.md`
- `.agents/context/product-brief.md`, if present, for the document map
- The imported planning sources

## Process

1. Inventory source material without overloading the spec with copied prose.
2. Treat imported folders such as `archives/` as provenance only. They are not
   live project truth after import.
3. Promote reusable accepted background into the relevant major
   `.agents/context/` document.
4. Put long source detail, evidence, research notes, and provenance under
   `.agents/references/`.
5. Extract requirements, capabilities, constraints, assumptions, risks,
   acceptance criteria, evidence, and proposed tasks into grouped spec records.
6. Put decisions in the affected spec's `decisions.md`. If a decision affects
   multiple specs, copy the relevant decision into each affected spec and cite
   the same source.
7. Keep specs understandable from `.agents/context/`, `.agents/references/`,
   and their own files. Do not make one spec the source of truth for a sibling
   spec.
8. Mark imported confidence as `imported` unless validation upgrades or
   conflicts exist.
9. Mark ambiguous, stale, duplicated, superseded, rejected, deferred, or
   conflicting content explicitly.
10. Preserve visual peg routing in `wireframes/` when UI sources exist.
11. When imported material identifies MVP scope, route to `workflows/mvp.md` to
   create or repair `.agents/specs/mvp/`.
12. Do not create active progress from imported research records.

## Stop

Stop when durable facts are compact records, useful source value is linked, and the recommended next step is review, validation, MVP specification, POC, source retirement, or no follow-up.
