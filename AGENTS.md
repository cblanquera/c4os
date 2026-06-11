# Agent Guide

This repository uses explicit planning documents as the source of truth for product scope, architecture, implementation sequencing, and verification.

## Start Here

Before implementation work:

1. Read `CONTEXT.md` for product vocabulary, MVP boundaries, and durable constraints.
2. Read `plans/AGENTS.md` before editing anything under `plans/` or using planning documents to drive implementation.
3. Read `plans/mvp/mvp-freeze.md`, the current implementation plan, the current sprint or task document, and relevant acceptance criteria.
4. Do not silently promote future-scope features into MVP.
5. Keep progress, blockers, and verification evidence in the relevant implementation, sprint, or task document.

## Sprint Implementation Loops

For sprint implementation loops, follow the process in `plans/AGENTS.md`.

Pause when blocked by a missing user decision, denied approval, unavailable credential, contradictory documentation, failing external dependency, or an issue that cannot be safely resolved from repository context.

When blocked, explain exactly what user action is needed to resume.
