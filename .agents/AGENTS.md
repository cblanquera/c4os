# .agents Rules

This folder is the agent-readable project knowledge base, planning surface, and execution state store for C4OS.

## Start Here

Read the workflow that matches the task:

- `workflows/context-ingestion.md` for adding files, links, pasted text, or raw resources to project context.
- `workflows/goal-manager.md` for coordinating a documented goal across planning, design, implementation, QA, and handoff.
- `workflows/document-integrity.md` for checking context, specs, indexes, derived views, and progress state.
- `workflows/import.md` for converting existing planning material into compact records.
- `workflows/mvp.md` for creating or repairing `.agents/specs/mvp/`, the customer-usable MVP contract.
- `workflows/feature-development.md` for bounded work after MVP scope is accepted.
- `workflows/poc.md` for feasibility proofs that can change architecture, security, runtime, browser, terminal, or extension scope.
- `workflows/wireframes.md` for low-fidelity screens, flows, and interface review.
- `workflows/creatives.md` for visual direction, asset notes, creative review rounds, and approved creative guidelines.
- `workflows/review.md` for readiness, risk, consistency, and traceability review.
- `workflows/validation.md` for evidence-gathering on risky claims or blockers.
- `workflows/freeze.md` for turning accepted MVP or bounded feature records into implementation contracts.
- `workflows/progress.md` for active execution packets, logs, and handoffs after a frozen spec exists.
- `workflows/batch-reconciliation.md` for validated QA, feedback, and polish batches.
- `workflows/source-retirement.md` before declaring old planning sources obsolete.
- `workflows/handoff.md` before stopping, switching sessions, or leaving substantial work for a future agent.
- `workflows/ad-hoc.md` for unplanned requests.

## Source Of Truth

- `plans/product-brief.md` and `plans/product-interface.md` are the imported human planning sources for this restart.
- `plans/pegs/*.png` are the visual peg sources for the UI direction.
- `.agents/context/` contains shared product understanding future specs should read first.
- `.agents/specs/research/` contains the current imported research, MVP-scope analysis, wireframe acceptance, and POC validation records. It is discovery input only.
- `.agents/specs/research/research-freeze.md` closes the research round and recommends the next planning path. It is not an implementation contract.
- `.agents/specs/mvp/` is the required contract for the distributable desktop MVP. Create or repair it before active MVP implementation.
- `proofs/` contains repo-level POC implementation artifacts. Put runnable proof code, harnesses, fixtures, and proof-specific evidence files there instead of burying implementation code inside `.agents/`.
- `wireframes/` contains wireframe routing notes and links back to the visual pegs.
- `creatives/` contains creative direction, asset, and guideline artifacts only when creative work is created or approved.
- `.agents/development/progress/` should exist only after implementation or active execution tracking begins.

## Implementation Locations

For the distributable desktop MVP, production work belongs only in:

- `backend/`: Rust/Tauri backend authority, native commands, state, security boundaries, runtime adapter, trusted filesystem access, terminal execution, Browser records, extension records, approvals, persistence, and restart/resume behavior.
- `frontend/`: desktop app UI loaded by the product shell. It renders state and calls backend commands, but does not own security decisions.
- `tests/server/`: backend, integration, and MVP acceptance tests for authoritative behavior.

Do not create or use `src-tauri/`.

Use `proofs/<proof-name>/` only for POCs, spikes, throwaway harnesses, and proof evidence.

## Planning Lifecycle

Use this sequence for MVP work:

1. Research records live under `.agents/specs/research/`.
2. Research closeout writes or updates `.agents/specs/research/research-freeze.md`.
3. Accepted reusable findings are promoted or reconciled into `.agents/context/`.
4. The MVP workflow creates or repairs `.agents/specs/mvp/`.
5. MVP freeze marks `.agents/specs/mvp/status.md` as `frozen-for-implementation`.
6. Progress converts accepted MVP tasks into `.agents/development/progress/` items.
7. Implementation changes are made in `backend/`, `frontend/`, and `tests/server/`.

If `.agents/specs/mvp/status.md` does not exist or is not frozen for implementation, route MVP/distribution work to `workflows/mvp.md` before creating progress items or editing product code.

## Record Rules

- Use stable IDs such as `REQ-001`, `CAP-001`, `CON-001`, `DEC-001`, `RISK-001`, `AC-001`, `EVD-001`, and `TASK-001`.
- Keep records short, source-linked, and explicit about status and confidence.
- Proposed `TASK` records are not active work until converted into progress items.
- MVP `TASK` records are not active work until they live in `.agents/specs/mvp/` and the MVP spec is frozen for implementation.
- Do not invent completed implementation, verification, runtime behavior, or user decisions.
- Raw feedback must be validated, rejected, classified, or reconciled before becoming implementation work unless evidence is already explicit.
- Treat imported plan content as product intent unless a later review, validation result, or user decision changes it.
- Promote only final accepted reusable findings into `.agents/context/`.
- Keep `.agents/specs/<spec-id>/poc/` focused on proof questions, expected proof, results, links, and promotion decisions. Link to `proofs/<proof-name>/` for the implementation artifact.

## Boundaries

- Keep generated `.agents/**/*.md` files under 500 lines.
- Put long rationale, research, source excerpts, screenshots, QA notes, and detailed evidence under `.agents/references/` when needed.
- Keep POC implementations under `proofs/<proof-name>/` with proof-local source, harnesses, fixtures, README/evidence notes, and ignored build output. Do not commit generated build directories such as `target/`, `node_modules/`, caches, or other bulky artifacts.
- Put creative assets and approved guidelines under root `creatives/` when creative work exists. Promote accepted creative rules into `.agents/context/` before frontend implementation depends on them.
- Do not retire or delete source planning docs without `workflows/source-retirement.md`.
- Do not start implementation unless the user explicitly asks for active execution and the relevant MVP or feature spec is frozen for implementation.
