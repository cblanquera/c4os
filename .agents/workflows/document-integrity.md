# Document Integrity Workflow

Use this when context, specs, wireframes, progress, releases, or derived views may have drifted.

## Check

- Links and source references are valid enough for local routing.
- Context contains only shared reusable product truth and starts routing from `.agents/context/product-brief.md`.
- Context reference routing lets agents load only the context or reference needed for the task.
- Context files link only to `.agents/context/` or `.agents/references/`; provenance for source paths, root artifacts, specs, progress, URLs, and local files belongs in `.agents/references/`.
- Spec records contain detailed requirements, risks, evidence, tasks, and acceptance criteria.
- Research freeze is named `.agents/specs/research/research-freeze.md`.
- MVP implementation has a frozen `.agents/specs/mvp/status.md`.
- Progress files exist only for active execution after a frozen spec.
- Progress items link to frozen spec tasks, requirements, and acceptance.
- Distributable MVP implementation paths are `backend/`, `frontend/`, and `tests/server/`.
- `src-tauri/` is not created or referenced as an implementation target.
- Proof code is under `proofs/<proof-name>/`.
- Final accepted records have been considered for context promotion.
- Accepted POC, wireframe, creative, review, QA, and feedback outcomes have been promoted into context or records before freeze, closeout, or release readiness.
- Raw feedback is validated, rejected, classified, or reconciled before becoming progress work.
- Verification claims name actual checks.
- Generated `.agents/**/*.md` files stay under 500 lines.

## Repair Rules

- Repair routing, statuses, and stale references compactly.
- Do not change product scope while doing integrity repair.
- Do not retire original planning sources without a source-retirement pass.

## Stop

Stop when the changed documents are coherent and any unresolved conflict is called out explicitly.
