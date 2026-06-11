# Planning Directory Guide

This directory is a fuller version of spec-driven development. Treat the documents here as the product and architecture planning system that comes before implementation. The goal is to make requirements, evidence, decisions, risks, and acceptance gates explicit enough that implementation work can be traced back to intent.

The process is based on a staged planning lifecycle: intake, discovery, specifications, architecture review, ADRs, readiness gaps, spikes, validation evidence, decision consolidation, MVP definition, acceptance criteria, non-goals, final grilling, readiness resolution, MVP freeze, epics, tasks, and implementation planning.

Do not start implementation from this directory alone. Use these documents to understand what should be built, why it should be built, what is intentionally out of scope, and which decisions or risks still block the work. As of the current freeze, planning is ready for implementation sequencing, but release readiness still depends on app-build validation, code-level tests, and product QA gates.

## How The Folders Fit Together

The current planning flow is:

1. Capture the pre-implementation brief and research-first intent.
2. Define the product and system requirements in `specs/`.
3. Gather evidence and standards context in `research/`.
4. Pressure-test the plan in `reviews/`.
5. Record architectural choices in `adr/`.
6. Identify readiness gaps and unresolved assumptions.
7. Investigate unknowns in `spikes/`.
8. Convert risky assumptions into evidence in `validation/`.
9. Track cross-cutting decision state in `decisions/`.
10. Narrow the first shippable slice in `mvp/`.
11. Use `acceptance/` as the verification contract for implementation.
12. Run the final grill/readiness review before freezing scope.
13. Freeze the MVP in `plans/mvp/mvp-freeze.md`.
14. Convert the frozen MVP into epics, tasks, dependencies, build order, and sprint sequencing in `implementation/`.

## Folder Purposes

### Root Planning Files

Root-level files under `plans/` are for lifecycle-wide planning documents that do not belong to one specialized folder. For example, a pre-plan brief belongs at `plans/preplan-brief.md` because it frames the entire planning effort before specifications are generated.

Use root planning files sparingly. Most new documents should live in one of the typed folders below.

### `specs/`

Canonical product, architecture, security, data, UX, marketplace, plugin, functional, and requirements specifications. These files describe the desired system at a requirements level.

Use this folder when you need the source of truth for what the product is supposed to do, what constraints matter, and how major product areas are expected to behave.

### `mvp/`

MVP scope, user journeys, validation plan, success metrics, and the frozen MVP implementation contract. These files separate the smallest useful validation product from later ambitions.

Use this folder to decide whether a feature belongs in the first private alpha, V1, V2, or future work. If an implementation task is not clearly justified by MVP scope, treat it as suspect unless another document explicitly promotes it.

`plans/mvp/mvp-freeze.md` is now the implementation contract. Features not listed there are out of scope for the first implementation pass.

### `research/`

Background research, standards analysis, and open research gaps. These documents provide external context and evidence behind the specs.

Use this folder when a recommendation depends on an ecosystem standard, third-party tool, protocol, or market convention.

### `spikes/`

Time-boxed technical investigations for unresolved assumptions. Each spike should state the objective, context, questions to answer, hypothesis, investigation plan, success criteria, decisions unlocked, and estimated effort.

Use this folder when the plan depends on something uncertain enough that research or prototyping must happen before an ADR can be finalized.

### `validation/`

Evidence gathered after review findings, spikes, or unresolved ADRs identify pre-implementation risk. This folder records the validation loop, evidence matrix, decision log, finding-specific proof, failure reproductions, fallback decisions, and acceptance-gate separation.

Use this folder when a claim needs proof before it can unlock freeze-and-plan. Accepted evidence includes official documentation, source references, local probes, reproducible command transcripts, observed structured payloads, failure-mode reproduction, and explicit fallback or scope decisions. Do not use assumptions, UI-only observation for backend control, terminal scraping as proof of structured control, or post-execution audit as proof of pre-execution control.

### `adr/`

Architecture Decision Records. These capture individual architectural decisions, alternatives considered, tradeoffs, consequences, and follow-up questions.

Use this folder when a choice needs durable architectural memory. ADRs may begin unresolved, but each should eventually become accepted, rejected, deferred, or superseded.

### `decisions/`

Decision rollups and decision-state management. These files summarize finalized, deferred, unresolved, and readiness-impacting decisions across multiple ADRs or planning areas.

Use this folder when you need a project-level view of what is settled, what is deferred, and what still blocks implementation readiness.

### `reviews/`

Critical reviews and risk analyses. These documents challenge the specs and ADRs from architecture, security, product, implementation, and operational perspectives.

Use this folder to find objections, weak assumptions, missing tests, and pre-implementation risks. Reviews are not implementation plans; they are pressure tests that should feed back into specs, spikes, ADRs, and acceptance criteria.

The final grill session belongs here as `plans/reviews/final-implementation-readiness-review.md`. It should challenge the plan against the existing documents and classify findings strongly enough that blocker and high-risk items can be resolved before implementation planning.

### `roadmap/`

Implementation sequencing at a specification level. The roadmap turns the validated plan into phases, exit criteria, and post-MVP sequencing.

Use this folder to understand order of operations and phase gates. The roadmap should not override MVP scope or unresolved decision blockers.

### `acceptance/`

Acceptance criteria organized by product or system area. These files define what must be true for implementation work to be considered complete.

Use this folder when writing tests, QA checklists, implementation tickets, or completion criteria. Acceptance files should be concrete enough to verify behavior, security, performance, UX, failure conditions, and out-of-scope boundaries.

### `implementation/`

Implementation epics, task breakdowns, dependencies, build order, and sprint plans. This folder should be created only after the final readiness review has been resolved and `plans/mvp/mvp-freeze.md` exists.

Use this folder to translate the frozen MVP into implementation work. Do not use it for speculative features, unresolved architecture, or pre-freeze task generation.

The current implementation folder is valid because blocker/high readiness findings were resolved or explicitly converted into accepted fallback/scope decisions before MVP freeze.

## Working Rules For Agents

- Preserve the distinction between requirements, decisions, risks, and acceptance criteria.
- Do not silently promote future-scope features into MVP.
- When changing a spec, check whether related MVP, ADR, decision, roadmap, review, spike, or acceptance documents also need updates.
- When resolving an uncertainty, update the relevant spike and any ADR or decision rollup it unlocks.
- When validation produces evidence, update `validation/evidence-matrix.md`, `validation/decision-log.md`, affected ADRs, and decision rollups.
- When adding implementation detail, keep it at planning level unless the user explicitly asks for code.
- Prefer adding traceable links between documents over duplicating large sections.
- Keep unresolved items explicit. Do not rewrite ambiguity into false certainty.
- Block implementation planning until blocker and high-risk findings from the final readiness review are accepted, rejected, or deferred.
- Do not create `implementation/` epics or tasks before `plans/mvp/mvp-freeze.md` exists.
- Do not treat implementation readiness as release readiness. App-build validation, code-level tests, product QA, and public-release platform gates must still be tracked after implementation begins.

## Sprint Implementation Loop

When the user asks to run a sprint implementation loop, continue working through the current sprint plan until all documented app goals and acceptance gates for that sprint are met, or until progress is blocked by an issue that requires user action.

For each loop:

1. Read `CONTEXT.md`, this file, `plans/mvp/mvp-freeze.md`, the current sprint or task plan, and the relevant acceptance criteria.
2. Pick the highest-priority unblocked task.
3. Implement the narrowest change that satisfies the task.
4. Run relevant verification for the changed behavior.
5. Update the current sprint or task document with status, verification evidence, remaining work, and blockers.
6. Continue to the next unblocked task.

Pause only when blocked by a missing user decision, denied approval, unavailable credential, contradictory documentation, failing external dependency, or an issue that cannot be safely resolved from repository context.

When pausing for a blocker, report:

- Blocker.
- What was tried.
- Why progress cannot continue safely.
- Exact user action needed.
- Resume prompt.

## Unclear Areas To Clarify

- The boundary between `adr/` and `decisions/` should stay disciplined: `adr/` holds individual decision records, while `decisions/` holds rollups and cross-ADR status. Some current files overlap, so future edits should avoid duplicating full ADR content in rollups.
- The boundary between `research/` and `spikes/` depends on whether the work is evidence gathering or targeted validation. Use `research/` for broad context and `spikes/` for scoped investigations that unlock decisions.
- The boundary between `reviews/` and `acceptance/` should remain explicit: reviews identify risks and critiques; acceptance documents define pass/fail criteria for implementation.
- `plans/preplan-brief.md` exists and is historical context. Later MVP and freeze documents supersede it where they conflict.
- `plans/mvp/mvp-freeze.md` and `plans/implementation/` now exist because final readiness findings were resolved through the validation loop.
- The original lifecycle mentions "Open Questions and Research Gaps" under `plans/reviews`, while the current tree has `plans/research/open-questions-and-research-gaps.md`. That is reasonable if the file is treated as research inventory, but keep the convention consistent going forward.
