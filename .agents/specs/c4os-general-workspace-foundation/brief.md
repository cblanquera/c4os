# C4OS General Workspace Foundation Brief

Spec ID: c4os-general-workspace-foundation
Type: planned-brownfield
Status: proposed

## Objective

Plan the first post-V1 general workspace foundation for research, writing,
documentation, and analysis workflows.

## Audience

- Product and architecture agents defining the first non-coding workspace slice.
- Implementation agents that need a bounded feature spec before creating
  `.agents/progress/` packets.
- Review agents checking that general workspace work does not silently promote
  deferred browser, memory, plugin, provider, or concurrent-agent scope.

## Source Material

- `.agents/specs/c4os-general-workspace-roadmap/`
- `.agents/specs/c4os-mvp/`
- `.agents/progress/manifest.md`
- Current source files under `src/`, `src-tauri/src/`, and `tests/`
- User decision on 2026-06-12 selecting research, writing, documentation, and
  analysis as first audience priorities.

## Scope

- Workspace-level home or overview for the selected product direction.
- Project and session organization for research/writing/docs/analysis work.
- Better project/session navigation using existing project and session
  primitives.
- Clear labels and status surfaces for research, writing, documentation, and
  analysis sessions.
- Planning for non-coding workflow ergonomics without requiring browser,
  durable memory, plugin, provider expansion, or concurrent agents.

## Non-Goals

- Do not implement browser/web viewing in this feature spec.
- Do not add durable memory, embeddings, background indexing, or automatic
  cross-session summaries.
- Do not add plugin or marketplace workflows.
- Do not add direct/local providers or model fallback.
- Do not add worktree lifecycle or multiple concurrent agents.
- Do not relax backend-owned approval, file, shell, Git, secret, or runtime
  boundaries.

## Current State

C4OS has a coding-first MVP shell with provider setup, one active selected Git
project, one active run, project/session persistence, read-only file browsing,
explicit project-local skills, local stdio MCP approval proposals, artifacts,
exports, and review surfaces. Project selector and session catalog modules
exist but intentionally expose many management controls as unavailable.

## Intended State

C4OS should feel usable as a local-first AI workspace for research, writing,
documentation, and analysis work, with clear project/session organization and
workflow labels while remaining inside the current safety and deferred-scope
boundaries.

## Definition Of Ready

- Current-state evidence is recorded from code and existing specs.
- Proposed capabilities are traceable to acceptance criteria and tasks.
- Deferred scopes remain explicit.
- Readiness review finds no unresolved blocker before progress packets are
  created.
