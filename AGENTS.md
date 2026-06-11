# Agent Guide

This repository uses `.agents/` as the source of truth for product scope,
architecture, implementation sequencing, active progress, and verification.

## Start Here

Before implementation work:

1. Read `CONTEXT.md` for product vocabulary, MVP boundaries, and durable constraints.
2. Read `.agents/AGENTS.md` for the current agent workspace layout.
3. Read `.agents/specs/AGENTS.md`,
   `.agents/specs/c4os-mvp/index.md`,
   `.agents/specs/c4os-mvp/status.md`, and relevant spec records.
4. Read `.agents/progress/brief.md`, `.agents/progress/manifest.md`, and the
   relevant item or batch packet before changing implementation state.
5. Use `.agents/archive/planning-corpus/` for detailed historical planning
   context, acceptance criteria, ADRs, validation evidence, reviews, research,
   and implementation sequencing when compact spec records are not enough.
6. Do not recreate or reference removed root planning or task-bank folders for
   new work.
7. Do not silently promote future-scope features into MVP.
8. Use `.agents/specs/` for durable AI-readable planning records and
   `.agents/progress/` for active execution state, item packets, logs, outputs,
   blockers, and verification evidence.

## Sprint Implementation Loops

For sprint implementation loops, follow `.agents/progress/brief.md`,
`.agents/progress/manifest.md`, and the relevant item or batch packet. Use
`.agents/specs/c4os-mvp/` for scope, acceptance, risks, decisions, and evidence.

Pause when blocked by a missing user decision, denied approval, unavailable credential, contradictory documentation, failing external dependency, or an issue that cannot be safely resolved from repository context.

When blocked, explain exactly what user action is needed to resume.
