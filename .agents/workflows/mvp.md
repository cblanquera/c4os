# MVP Workflow

Use this to create or repair `.agents/specs/mvp/`, the smallest customer-usable C4OS product slice.

Run this workflow after research is frozen and before active MVP implementation.

## Read First

- `.agents/context/product-brief.md`
- `.agents/context/product-specs.md`
- `.agents/context/technical-specs.md`
- `.agents/context/creative-specs.md`
- `.agents/context/work-orders.md`
- `.agents/specs/research/research-freeze.md`
- `.agents/specs/research/index.md`, `status.md`, requirements, acceptance, decisions, risks, evidence, traceability, and viability gaps
- `wireframes/screens.md`, when UI behavior matters
- root `creatives/` artifacts or deferral decisions, when creative direction matters

## Required MVP Spec Files

Create or repair these files when the MVP scope is being made executable:

- `.agents/specs/mvp/brief.md`
- `.agents/specs/mvp/status.md`
- `.agents/specs/mvp/requirements.md`
- `.agents/specs/mvp/acceptance.md`
- `.agents/specs/mvp/risks.md`
- `.agents/specs/mvp/decisions.md`
- `.agents/specs/mvp/evidence.md`
- `.agents/specs/mvp/tasks.md`
- `.agents/specs/mvp/traceability.md`
- `.agents/specs/mvp/mvp-viability-gaps.md`

## Process

1. Identify the target evaluator and the smallest coherent workflow.
2. Convert accepted research into MVP requirements, acceptance, decisions, risks, evidence, and viability gaps.
3. Define minimum UI, backend behavior, data behavior, states, permissions, errors, empty states, restart behavior, and verification.
4. State the implementation architecture in MVP decisions:
   - `backend/` for Rust/Tauri backend authority and native behavior
   - `frontend/` for the desktop app UI
   - `tests/server/` for backend, integration, and MVP acceptance tests
   - no `src-tauri/`
5. Distinguish proof-only behavior from production MVP behavior.
6. Reconcile accepted review, validation, wireframe, creative, feedback, and POC learning into records before freeze.
7. Write proposed tasks in `.agents/specs/mvp/tasks.md`; these are not active progress yet.
8. Promote final shared product understanding into the relevant major `.agents/context/` document.

## Stop

Stop when `.agents/specs/mvp/` is complete enough for review or freeze, blockers are explicit, and the next step is review, validation, POC, freeze, or progress creation.
