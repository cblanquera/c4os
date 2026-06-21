# Goal Manager Workflow

Use this when a documented goal should be carried across planning, design, implementation, QA, documentation, and handoff loops.

The goal manager routes work. It does not replace the MVP, POC, freeze, or
progress workflows.

## Read First

- `.agents/AGENTS.md`
- `.agents/context/index.md`
- `.agents/context/feature-goals.md`
- `.agents/specs/manifest.md`
- `.agents/specs/research/research-freeze.md`, when the goal depends on research output
- `.agents/specs/mvp/status.md`, when the goal mentions MVP, viability, distribution, dogfood, desktop, or release
- `.agents/development/progress/manifest.md`, if it exists
- `.agents/workflows/batch-reconciliation.md`, when the goal is validated feedback, QA mismatch, or polish batching
- The workflow for the current phase

## MVP Preflight

Before creating progress or editing product code for MVP/distribution work:

1. Check whether `.agents/specs/mvp/status.md` exists.
2. Check whether its status is `frozen-for-implementation`.
3. If it is missing or not frozen, route to `workflows/mvp.md`.
4. If architecture, implementation folders, acceptance, or viability gaps are unclear, route to `workflows/mvp.md` or `workflows/freeze.md`.
5. After MVP freeze, route to `workflows/progress.md` to create implementation items.

For this repository, implementation paths for distributable MVP work are
`backend/`, `frontend/`, and `tests/server/`. Do not create or use
`src-tauri/`.

## Process

1. Restate the active goal, scope, non-goals, source inputs, done definition, and verification expectation.
2. Check context, feature goals, specs manifest, and current progress before selecting work.
3. Determine the current phase: research, MVP specification, creative review, freeze, progress, implementation, QA, batch reconciliation, or closeout.
4. Route to the smallest useful specialist workflow.
5. Create progress state only after an accepted/frozen spec exists and active execution is requested.
6. Before closeout, run document-integrity checks against changed context, specs, progress, and derived views.
7. Report the recommended next step.

## Stop And Ask

Stop when a product decision is required, multiple valid paths materially change scope, required information is not in project artifacts, or the change would alter MVP boundaries.
