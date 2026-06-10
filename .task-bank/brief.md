# Task Bank Brief

Goal: implement the C4OS MVP from the frozen planning corpus, starting with the
first vertical slice in the recommended build order.

Non-Goals:

- Do not implement post-MVP plugin, marketplace, MCP, browser automation, or
  rich artifact preview scope.
- Do not claim release readiness before app-build validation and product QA
  gates pass.

Constraints:

- Follow `plans/mvp/mvp-freeze.md` as the MVP scope authority.
- Follow `plans/implementation/recommended-build-order.md` for dependency
  order.
- Keep backend authority over filesystem, shell, Git state-changing policy,
  process supervision, secrets, approvals, persistence, and runtime adapter
  control.
- Product telemetry must remain absent from MVP.

Definition Of Done:

- Sprint 1 architecture spine is implemented and verified.
- Sprint 2 protected local actions are policy-controlled by the backend.
- Sprint 3 user workflow and release gates are complete.
- The progress sheet and task bank accurately reflect current status.

Source Materials:

- `plans/mvp/mvp-freeze.md`
- `plans/implementation/tasks.md`
- `plans/implementation/recommended-build-order.md`
- `plans/acceptance/`
- `plans/adr/`

Verification Expectations:

- Run focused automated tests for each item before status advances to review.
- Mark items verified only after their stated verification command or manual
  gate has actually passed.
