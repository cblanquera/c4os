# item-032: TASK-031 Resolve V1 Worktree Lifecycle Scope

## Status

verified

## Objective

Resolve the missing product, architecture, cleanup, and acceptance decisions
before implementing Git worktree creation or cleanup.

## Dependencies

- item-028
- item-029
- item-031

## Blocker

Resolved by deferring worktree creation and cleanup beyond current V1.
Implementation remains unsafe to infer from repository context, so current V1
will not implement worktree lifecycle behavior.

## Evidence

- `.agents/archive/planning-corpus/roadmap/implementation-roadmap.md` moves worktree creation and cleanup
  beyond current V1.
- `.agents/archive/planning-corpus/decisions/non-goals.md` records the Sprint 9 defer decision.
- `.agents/archive/planning-corpus/decisions/implementation-readiness-gaps.md` resolves the cleanup
  policy gap by deferral.
- `.agents/archive/planning-corpus/acceptance/git-integration.md` keeps worktree management out of scope,
  including lifecycle and Git edge-case behavior.

## Remaining V1 Audit

Checked on 2026-06-11 after Sprint 8 verification:

- Worktree creation and cleanup is deferred beyond current V1 by this item.
- Nested `AGENTS.md` resolution remains deferred until standards conformance
  and effective-instruction UI decisions are made.
- Explicit skill discovery and invocation remains deferred until skill
  conformance and trust model decisions are made.
- Local stdio MCP remains deferred until V1 MCP scope, roots, resources,
  prompts, sampling, and policy-routing decisions are made.
- Richer artifact previews remain deferred until preview sandboxing, active
  HTML policy, browser-content ingestion policy, and Chromium requirements are
  validated.
- Export/import for sessions and artifacts remains deferred until export format,
  absolute-path handling, secret exclusion, artifact-type behavior, and
  round-trip compatibility are specified.

Later V1 roadmap items still need their own accepted scope and acceptance
criteria before implementation.

## Decision

Current V1 will not implement worktree creation or cleanup. Worktrees may be
reconsidered after recurring parallel-session or isolated-edit demand is
validated and a fresh product, architecture, and acceptance decision defines:

- Worktree creation source and branch behavior.
- Worktree selection relationship to selected project and sessions.
- Dirty worktree cleanup rules.
- Failed cleanup recovery.
- Submodule, Git LFS, nested repo, and branch reuse behavior.

## Resume Prompt

Continue the sprint loop after Sprint 9. Treat worktree creation and cleanup as
deferred beyond current V1 and select the next unblocked documented roadmap
item with accepted scope and acceptance criteria.
