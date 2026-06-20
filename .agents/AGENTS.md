# .agents Rules

This folder is the agent-readable project knowledge base, planning surface, and execution state store for C4OS.

## Start Here

Read the workflow that matches the task:

- `workflows/context-ingestion.md` for adding files, links, pasted text, or raw resources to project context.
- `workflows/goal-manager.md` for coordinating a documented goal across planning, design, implementation, QA, and handoff.
- `workflows/document-integrity.md` for checking context, specs, indexes, derived views, and progress state.
- `workflows/import.md` for converting existing planning material into compact records.
- `workflows/mvp.md` for defining, narrowing, validating, or freezing the smallest usable product slice.
- `workflows/feature-development.md` for bounded work after MVP scope is accepted.
- `workflows/poc.md` for feasibility proofs that can change architecture, security, runtime, browser, terminal, or extension scope.
- `workflows/wireframes.md` for low-fidelity screens, flows, and interface review.
- `workflows/review.md` for readiness, risk, consistency, and traceability review.
- `workflows/validation.md` for evidence-gathering on risky claims or blockers.
- `workflows/freeze.md` for turning accepted records into implementation contracts.
- `workflows/progress.md` for active execution packets, logs, and handoffs.
- `workflows/source-retirement.md` before declaring old planning sources obsolete.
- `workflows/handoff.md` before stopping, switching sessions, or leaving substantial work for a future agent.
- `workflows/ad-hoc.md` for unplanned requests.

## Source Of Truth

- `plans/product-brief.md` and `plans/product-interface.md` are the imported human planning sources for this restart.
- `plans/pegs/*.png` are the visual peg sources for the UI direction.
- `.agents/context/` contains shared product understanding future specs should read first.
- `.agents/specs/research/` contains the current imported research, MVP-scope analysis, wireframe acceptance, and POC validation records. It is planning input until a freeze record creates an implementation contract.
- `proofs/` contains repo-level POC implementation artifacts. Put runnable proof code, harnesses, fixtures, and proof-specific evidence files there instead of burying implementation code inside `.agents/`.
- `wireframes/` contains wireframe routing notes and links back to the visual pegs.
- `.agents/development/progress/` should exist only after implementation or active execution tracking begins.

## Record Rules

- Use stable IDs such as `REQ-001`, `CAP-001`, `CON-001`, `DEC-001`, `RISK-001`, `AC-001`, `EVD-001`, and `TASK-001`.
- Keep records short, source-linked, and explicit about status and confidence.
- Proposed `TASK` records are not active work until converted into progress items.
- Do not invent completed implementation, verification, runtime behavior, or user decisions.
- Treat imported plan content as product intent unless a later review, validation result, or user decision changes it.
- Promote only final accepted reusable findings into `.agents/context/`.
- Keep `.agents/specs/<spec-id>/poc/` focused on proof questions, expected proof, results, links, and promotion decisions. Link to `proofs/<proof-name>/` for the implementation artifact.

## Boundaries

- Keep generated `.agents/**/*.md` files under 500 lines.
- Put long rationale, research, source excerpts, screenshots, QA notes, and detailed evidence under `.agents/references/` when needed.
- Keep POC implementations under `proofs/<proof-name>/` with proof-local source, harnesses, fixtures, README/evidence notes, and ignored build output. Do not commit generated build directories such as `target/`, `node_modules/`, caches, or other bulky artifacts.
- Do not retire or delete source planning docs without `workflows/source-retirement.md`.
- Do not start implementation unless the user explicitly asks for active execution.
