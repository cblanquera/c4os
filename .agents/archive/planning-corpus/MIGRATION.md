# Planning Corpus Archive

Date: 2026-06-11

## Purpose

This archive preserves the repository-root human planning corpus inside the
`.agents` workspace so future agents can use `.agents` as the complete planning
and progress source.

## Coverage

- File count verified at migration: 105 Markdown files.
- Directory categories preserved: acceptance, ADRs, decisions,
  implementation, MVP, research, reviews, roadmap, specs, spikes, validation,
  and root planning guidance.
- Internal references that previously pointed at the repository-root planning
  corpus were rewritten to point at `.agents/archive/planning-corpus/`.

## Retirement Meaning

The archived copy preserves detailed rationale, acceptance criteria, ADR
history, validation evidence, reviews, research, and implementation sequencing
that are intentionally too detailed for compact records.

Compact routing records remain under `.agents/specs/c4os-mvp/`. Active
execution state remains under `.agents/progress/`.

