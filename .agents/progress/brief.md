# Agent Progress Brief

Goal: implement and track the C4OS MVP and current V1 follow-on work from the
frozen planning corpus, using `.agents/progress/` as the active execution bank.

Non-Goals:

- Do not implement post-MVP plugin, marketplace, MCP, browser automation, rich
  artifact preview, concurrent-agent, direct-provider, or worktree lifecycle
  scope unless a later spec decision explicitly promotes it.
- Do not add new progress updates to `.task-bank/`.
- Do not claim release readiness before app-build validation and product QA
  gates pass.

Constraints:

- Follow `.agents/archive/planning-corpus/mvp/mvp-freeze.md` as the MVP scope authority.
- Follow `.agents/archive/planning-corpus/implementation/recommended-build-order.md` for dependency
  order.
- Keep backend authority over filesystem, shell, Git state-changing policy,
  process supervision, secrets, approvals, persistence, and runtime adapter
  control.
- Product telemetry must remain absent from MVP.

Definition Of Done:

- Sprint 1 architecture spine is implemented and verified.
- Sprint 2 protected local actions are policy-controlled by the backend.
- Sprint 3 user workflow and release gates are complete.
- Current V1 follow-on items in the manifest are verified.
- The progress sheet and agent progress bank accurately reflect current status.

Source Materials:

- `.agents/specs/c4os-mvp/`
- `.agents/archive/planning-corpus/mvp/mvp-freeze.md`
- `.agents/archive/planning-corpus/implementation/tasks.md`
- `.agents/archive/planning-corpus/implementation/recommended-build-order.md`
- `.agents/archive/planning-corpus/acceptance/`
- `.agents/archive/planning-corpus/adr/`
- `.task-bank/` historical source

Verification Expectations:

- Run focused automated tests for each item before status advances to review.
- Mark items verified only after the stated verification command or manual gate
  has actually passed.

