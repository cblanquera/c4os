# Import Workflow

Use this when existing planning material should become compact `.agents` records.

## Read First

- `.agents/AGENTS.md`
- `.agents/context/product-brief.md`, if present, for the document map
- The imported planning sources

## Process

1. Inventory source material without overloading the spec with copied prose.
2. Promote reusable accepted background into the relevant major `.agents/context/` document.
3. Extract requirements, capabilities, constraints, decisions, assumptions, risks, acceptance criteria, evidence, and proposed tasks into grouped spec records.
4. Mark imported confidence as `imported` unless validation upgrades or conflicts exist.
5. Mark ambiguous, stale, duplicated, or conflicting content explicitly.
6. Preserve visual peg routing in `wireframes/` when UI sources exist.
7. When imported material identifies MVP scope, route to `workflows/mvp.md` to create or repair `.agents/specs/mvp/`.
8. Do not create active progress from imported research records.

## Stop

Stop when durable facts are compact records, useful source value is linked, and the recommended next step is review, validation, MVP specification, POC, source retirement, or no follow-up.
