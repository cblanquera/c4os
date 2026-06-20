# C4OS Agent Routing

This repository uses `.agents/` as the agent-readable planning, workflow, and
execution-state surface. Start with `.agents/AGENTS.md` before answering
project-specific planning, POC, implementation, review, or handoff questions.

## Source Routing

- Human-facing product plans live in `plans/`.
- Current wireframe and UI handoff artifacts live in `wireframes/`.
- Agent-maintained context, specs, workflows, reviews, validation records, and
  progress state live in `.agents/`.
- Repo-level POC implementations live in `proofs/`.

## POC Implementation Rule

Use `proofs/<proof-name>/` for every POC implementation artifact: runnable
code, small harnesses, fixtures, proof-local notes, and evidence files. Keep
the matching `.agents/specs/<spec-id>/poc/` records focused on the proof
question, expected proof, verification method, result, decision, and links back
to the implementation directory.

Do not commit generated build output, dependency folders, caches, or large
runtime artifacts from proof work. Keep proof directories small enough for
review and repeatable enough to rerun.

## Operating Rules

- Do not treat proposed `.agents/specs/**/tasks.md` records as active work
  until they are converted into progress items or the user explicitly asks for
  execution.
- Do not start implementation from planning records unless the user explicitly
  asks for active execution.
- Preserve source-of-truth boundaries: implementation proof code in `proofs/`,
  durable product/spec records in `.agents/`, visual review artifacts in
  `wireframes/`, and human product plans in `plans/`.
