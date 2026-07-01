# Document Integrity Workflow

Use this when context, specs, wireframes, progress, releases, or derived views may have drifted.

## Check

- Links and source references are valid enough for local routing.
- Context contains only shared reusable product truth and starts routing from `.agents/context/product-brief.md`.
- Context reference routing lets agents load only the context or reference needed for the task.
- Context files link only to `.agents/context/` or `.agents/references/`; provenance for source paths, root artifacts, specs, progress, URLs, and local files belongs in `.agents/references/`.
- Spec records contain detailed requirements, risks, evidence, tasks, and acceptance criteria.
- Spec decisions live in the affected spec's `decisions.md`; multi-spec
  decisions are copied into each affected spec and cite the same source.
- Specs can be understood from `.agents/context/`, `.agents/references/`, and
  their own files without relying on sibling specs for project-wide truth.
- Imported archive material is referenced as provenance only, not live project
  truth.
- Research freeze is named `.agents/specs/research/research-freeze.md`.
- MVP implementation has a frozen `.agents/specs/mvp/status.md`.
- Progress files exist only for active execution after a frozen spec.
- Progress items link to frozen spec tasks, requirements, and acceptance.
- Distributable MVP implementation paths are `backend/`, `frontend/`, and `tests/server/`.
- `src-tauri/` is not created or referenced as an implementation target.
- Proof code is under `proofs/<proof-name>/`.
- Final accepted records have been considered for context promotion.
- Accepted grill, review, or decision-question answers are reconciled into the
  relevant spec decisions, marked superseded/rejected/deferred with reasons, or
  promoted into shared context when reusable.
- Accepted POC, wireframe, creative, review, QA, and feedback outcomes have been promoted into context or records before freeze, closeout, or release readiness.
- Raw feedback is validated, rejected, classified, or reconciled before becoming progress work.
- Verification claims name actual checks.
- Generated `.agents/**/*.md` files stay under 500 lines.

## Repair Rules

- Repair routing, statuses, and stale references compactly.
- Do not change product scope while doing integrity repair.
- Do not retire original planning sources without a source-retirement pass.
- Do not move decisions into a non-implementable spec or root `docs/adr`
  archive. Use relevant spec `decisions.md`, context, and references.

## Stop

Stop when the changed documents are coherent and any unresolved conflict is called out explicitly.
