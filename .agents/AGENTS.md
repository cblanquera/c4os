# .agents Rules

This folder is the agent-readable project knowledge base, planning surface, and execution state store for C4OS.

## Start Here

Read the workflow that matches the task:

- `workflows/context-ingestion.md` for adding files, links, pasted text, or raw resources to project context.
- `workflows/goal-manager.md` for coordinating a documented goal across planning, design, implementation, QA, and handoff.
- `workflows/document-integrity.md` for checking context routing, specs, derived views, and progress state.
- `workflows/import.md` for converting existing planning material into compact records.
- `workflows/mvp.md` for creating or repairing `.agents/specs/mvp/`, the customer-usable MVP contract.
- `workflows/feature-development.md` for bounded work after MVP scope is accepted.
- `workflows/poc.md` for feasibility proofs that can change architecture, security, runtime, browser, terminal, or extension scope.
- `workflows/wireframes.md` for low-fidelity screens, flows, and interface review.
- `workflows/creatives.md` for visual direction, asset notes, creative review rounds, and approved creative guidelines.
- `workflows/review.md` for readiness, risk, consistency, and traceability review.
- `workflows/validation.md` for evidence-gathering on risky claims or blockers.
- `workflows/manual-qa.md` for built desktop app acceptance passes, visible navigation checks, and runtime log capture.
- `workflows/freeze.md` for turning accepted MVP or bounded feature records into implementation contracts.
- `workflows/progress.md` for active execution packets, logs, and handoffs after a frozen spec exists.
- `workflows/batch-reconciliation.md` for validated QA, feedback, and polish batches.
- `workflows/source-retirement.md` before declaring old planning sources obsolete.
- `workflows/handoff.md` before stopping, switching sessions, or leaving substantial work for a future agent.
- `workflows/ad-hoc.md` for unplanned requests.

## Source Of Truth

- `plans/product-brief.md` and `plans/product-interface.md` are the imported human planning sources for this restart.
- `plans/pegs/*.png` are the visual peg sources for the UI direction.
- `.agents/context/` contains five shared prework documents. Start with `.agents/context/product-brief.md` for the document map, then load only the context document or reference needed for the task: product brief, product specs, technical specs, creative specs, or work orders.
- `.agents/references/` is the non-entry detail, evidence, provenance, and historical-support layer. Do not load references by default; load them only through a context document's `Reference Routing` table or a workflow-specific need.
- `.agents/specs/research/` contains the current imported research, MVP-scope analysis, wireframe acceptance, and POC validation records. It is discovery input only.
- `.agents/specs/research/research-freeze.md` closes the research round and recommends the next planning path. It is not an implementation contract.
- `.agents/specs/mvp/` is the required contract for the distributable desktop MVP. Create or repair it before active MVP implementation.
- `proofs/` contains repo-level POC implementation artifacts. Put runnable proof code, harnesses, fixtures, and proof-specific evidence files there instead of burying implementation code inside `.agents/`.
- `wireframes/` contains wireframe routing notes and links back to the visual pegs.
- `creatives/` contains creative direction, asset, and guideline artifacts only when creative work is created or approved.
- `.agents/development/progress/` should exist only after implementation or active execution tracking begins.

## QA And Acceptance Boundary

Agents must not ask the user to perform incremental QA for implementation
behavior that the agent can reasonably verify in the local app. Before marking
an implementation item ready for stakeholder acceptance, run the relevant
automated checks and, when the behavior is visible in the desktop product,
perform a manual built-app QA pass through `workflows/manual-qa.md`.

Use the user as a stakeholder acceptance reviewer after agent verification is
complete, not as the first tester for routine navigation, persistence,
workspace switching, Browser, Files, Terminal, provider/model, or settings
behavior. If manual QA cannot be run, record the blocker explicitly in the
progress item and do not present the behavior as accepted or ready.

## Context And References

Use `.agents/context/` for compact, accepted, reusable product truth that future agents should read first. Keep it limited to the five major documents:

- `product-brief.md` for product identity, users, goals, principles, vocabulary, and the document map.
- `product-specs.md` for customer workflow, product behavior, feature surfaces, and MVP user-facing scope.
- `technical-specs.md` for product model, runtime boundary, persistence, security, Browser, Terminal, and implementation constraints.
- `creative-specs.md` for experience direction, interface contract, UI behavior, visual tone, and accessibility.
- `work-orders.md` for accepted decisions, sequencing, validation needs, deferred work, and implementation guardrails.

Use `.agents/references/` for material that supports context or specs but should not be loaded by default:

- `.agents/references/context/product/` for expanded product background, vocabulary detail, and feature-goal inputs.
- `.agents/references/context/product-specs/` for detailed product experience, feature surfaces, and MVP inventory.
- `.agents/references/context/technical-specs/` for detailed product model, runtime adapter, constraints, and validation caveats.
- `.agents/references/context/creative-specs/` and `.agents/references/context/ui-handoff/` for detailed interface and handoff material.
- `.agents/references/context/work-orders/` for decision history and work-order support.
- `.agents/references/research/` for research support, grill-session material, schemas, validation evidence, and long-form research notes.

When contributing context, choose one major context document as the owner, add only compact accepted reusable truth there, route long detail to the matching references folder, and update that context document's `Reference Routing` table when the new reference should be discoverable. Do not add a sixth `.agents/context/` document unless the folder contract is intentionally changed.

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
3. Accepted reusable findings are promoted or reconciled into the relevant major `.agents/context/` document.
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
- Promote only final accepted reusable findings into the relevant major `.agents/context/` document and route long detail through `.agents/references/`.
- Keep `.agents/specs/<spec-id>/poc/` focused on proof questions, expected proof, results, links, and promotion decisions. Link to `proofs/<proof-name>/` for the implementation artifact.

## Boundaries

- Keep generated `.agents/**/*.md` files under 500 lines.
- Put long rationale, research, source excerpts, screenshots, QA notes, and detailed evidence under `.agents/references/` when needed.
- Keep POC implementations under `proofs/<proof-name>/` with proof-local source, harnesses, fixtures, README/evidence notes, and ignored build output. Do not commit generated build directories such as `target/`, `node_modules/`, caches, or other bulky artifacts.
- Put creative assets and approved guidelines under root `creatives/` when creative work exists. Promote accepted creative rules into `.agents/context/creative-specs.md` before frontend implementation depends on them.
- Do not retire or delete source planning docs without `workflows/source-retirement.md`.
- Do not start implementation unless the user explicitly asks for active execution and the relevant MVP or feature spec is frozen for implementation.
