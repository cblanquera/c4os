# item-032: TASK-031 Resolve V1 Worktree Lifecycle Scope

## Status

blocked

## Objective

Resolve the missing product, architecture, cleanup, and acceptance decisions
before implementing Git worktree creation or cleanup.

## Dependencies

- item-028
- item-029
- item-031

## Blocker

Worktree implementation is not safe to infer from current repository context.
The roadmap lists worktree creation and cleanup as V1, but the current
decisions and acceptance files still leave the lifecycle undefined.

## Evidence

- `plans/decisions/implementation-readiness-gaps.md` lists worktree cleanup
  policy as missing information.
- `plans/decisions/non-goals.md` says worktrees may be reconsidered after users
  need parallel sessions or safer isolated edits.
- `plans/acceptance/git-integration.md` keeps worktree management out of scope.
- Current acceptance files do not define create, select, dirty-state, cleanup,
  branch, submodule, LFS, nested repo, or failure behavior.

## Exact User Action Needed

Decide whether V1 should implement worktrees now. If yes, provide or approve
acceptance criteria for:

- Worktree creation source and branch behavior.
- Worktree selection relationship to selected project and sessions.
- Dirty worktree cleanup rules.
- Failed cleanup recovery.
- Submodule, Git LFS, nested repo, and branch reuse behavior.

## Resume Prompt

Resolve TASK-031 with the approved V1 worktree lifecycle scope, update
acceptance criteria, then implement the narrowest Sprint 9 worktree slice.
