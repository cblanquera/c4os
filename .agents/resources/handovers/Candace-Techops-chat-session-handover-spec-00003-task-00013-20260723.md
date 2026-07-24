Candace-Techops-chat-session-handover-spec-00003-task-00013-20260723.md

# Handoff Summary

## Summary

This is the paused implementation-coordinator run for Frozen Spec 00003, the C4OS macOS application implementation. Tasks 00001 through 00012 are verified and locally committed. Task 00013, Onboarding And Settings Integration, is substantially implemented and repaired through its focused renderer failures, but it intentionally remains `started` with failed Agent Acceptance until the full regression, native acceptance matrix, restart/security evidence, and a fresh independent P0/P1 audit are complete.

The run was paused at the user's request on 2026-07-23. Do not commit Task 00013 from this handoff state.

## Context

- Repository: `/Users/cblanquera/server/projects/cblanquera/c4os`
- Branch: `build/mvp-3`
- Current HEAD and latest verified checkpoint: `eedc3e8 feat(mcp): complete Task 00012 lifecycle`
- Governing package: `.agents/specs/00003-c4os-macos-application-implementation/`
- Current task: `.agents/specs/00003-c4os-macos-application-implementation/tasks/00013-onboarding-settings-integration.md`
- Original run request: `/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`
- Accepted task contract: `.agents/references/00007-launch-and-settings-contract.md`
- Previous handoff checked: `notes/chat-session-handover/Candace-Techops-chat-session-handover-spec-00003-task-00004-20260719.md`

The original run authorizes bounded, non-overlapping subagents and scoped local checkpoint commits. The coordinator exclusively owns shared contracts, integration, Cargo scheduling, evidence, staging, and commits. Only one Cargo command may run at a time. The run does not authorize pushing, opening a pull request, signing, notarizing, distributing, or expanding into deferred scope.

## Scope Boundary

This handoff captures the uncommitted Task 00013 implementation and the exact verification checkpoint. It does not declare Task 00013 verified, does not replace the Frozen spec or accepted Context Files, and does not authorize staging unrelated worktree paths.

## Current State

Latest known state as of 2026-07-23:

- Tasks 00001 through 00012: verified with passed Agent Acceptance.
- Task 00013: `started`; formal Agent Acceptance remains failed.
- Tasks 00014 through 00015C: open.
- No Task 00013 checkpoint commit exists. No Task 00013 files are staged.
- All three Task 00013 test subagents completed their non-overlapping test ownership and are inactive.
- The isolated native fixture formerly listening on `127.0.0.1:56312` was stopped for the pause.

Task 00013 production work currently present includes:

- First-launch provider gate and provider onboarding backed by Rust-owned provider, credential, configuration, capability, and runtime services.
- Exact provider credential binding, cleanup, re-entry when security-relevant bindings change, and direct Action Gateway connection testing.
- Provider/model/runtime/configuration/policy Settings integration, durable defaults, real approval exceptions, and configuration observer rollback.
- Workspace clone validation and pending approval, picker identity revalidation, route authority, startup/runtime activation, model routing, and Settings return identity reconciliation.
- Structural mapping for all 28 advanced policy settings, collision coverage, and atomic policy/security persistence.
- Complete native Conversation draft reconciliation plus serialized composer persistence before Terminal/Browser artifact mutation.
- Production launch/provider gating with deterministic QA-only direct-route composition.
- Native project-picker Settings content preserved alongside provider settings.

Focused validation already green on the current workstream:

- `cargo check` and `cargo test --no-run` passed before the latest renderer-only repairs.
- Focused Rust suites passed for policy mapping, policy atomicity, configuration watcher rollback, picker identity, provider lifecycle/connectivity, runtime app/coordinator/dispatch/persistence/production, OpenCode adapter/credential/native tiers, production MCP sampling, and Action Gateway.
- Full frontend unit suite passed: 66 files, 340 tests.
- Production web build and QA build passed before the last router/composer repairs.
- Latest TypeScript check passed after the router/composer repairs.
- Latest focused renderer unit checks passed: 14/14 after the final route composition repair; an earlier focused set passed 18/18.
- The prior full Playwright run passed 37/42. Its five failures were inspected and repaired rather than blindly rerun.
- The repaired Terminal generation/draft race passed its exact focused Playwright scenario: 1/1.
- The four repaired QA route and Settings composition scenarios passed their exact focused Playwright rerun: 4/4.

Still required before Task 00013 can be verified:

- Rerun the complete formatter/lint/type/unit/build matrix on the latest renderer tree.
- Rerun all 42 Playwright scenarios on the latest tree.
- Run the full Rust workspace regression only after renderer stabilization, keeping the single-Cargo rule.
- Run protocol generation/checks and the native build/ignored acceptance tiers one Cargo-bearing command at a time.
- Perform rebuilt native macOS acceptance, restart/recovery, secret-boundary scans, log review, and degraded-state checks.
- Commission a fresh independent Agent Acceptance review and require P0=0 and P1=0.
- Update Task 00013 evidence, research/coverage/status ledgers, stage only exact Task 00013 paths, and create one scoped local checkpoint commit.

## Key Decisions

- Preserve production provider gating. QA fixture bypasses are build-gated and must not weaken production launch or protected-route behavior.
- Provider connection tests must traverse the real Action Gateway and exact provider binding; simulated success is not acceptance evidence.
- Runtime/provider/configuration/policy authority remains Rust-owned. The renderer projects validated snapshots and never becomes credential or policy authority.
- Remembered approval exceptions compare every security-relevant action fact, have explicit once/session/persistent lifetimes, and support promotion, restart persistence, and revocation.
- Configuration watcher activation must reconcile immediately; observer failure compensates the configuration write back to the prior snapshot.
- Every advanced-policy UI setting maps structurally to an exact Rust Action Gateway subject/action/resource tuple, with positive and nearest-negative evidence.
- Native Conversation snapshots reconcile the complete composer draft. Terminal and Browser submission persist the current composer draft inside the serialized Workspace operation before artifact mutation so authoritative rebase cannot erase the selected mode.
- Native project access from Task 00005 remains composed in the single Providers Settings route alongside Task 00013 provider controls.
- Do not checkpoint Task 00013 until current full regression, native evidence, and independent Agent Acceptance are complete.

## Conflicts or Changes

- An earlier full Playwright result was 37/42. All five named failures are now individually green after source repair, but the complete 42-test matrix has not yet been rerun on the final composition.
- Earlier independent audits reported nonzero P0/P1 findings. Their owning surfaces have been repaired and focused-tested, but only a new independent audit can change formal Agent Acceptance to passed.
- The Task 00006 `output/native/task-00006-settings-compressed.png` file was rewritten by Playwright during Task 00013 and restored to HEAD during pause cleanup.
- The preexisting unrelated `output/native/task-00006-chat-settings-roundtrip.png` modification was preserved byte-for-byte; its SHA-256 remains `18ba2436c855272e25c55a0de809b1eed54fbddff292c9e9089915eff2b7c1fa`.

## Open Questions

- Does the final 42-test Playwright matrix remain fully green when all repaired surfaces run together?
- Does the full Rust workspace regression reveal any interaction not covered by the focused suites?
- Does the rebuilt macOS bundle pass provider onboarding, Settings round trip, policy exception persistence/revocation, restart recovery, and secret-boundary inspection?
- Will fresh independent reviewers report P0=0 and P1=0 across Rust/security, renderer/integration, and policy/configuration boundaries?

## Blockers or Risks

- There is no user-owned decision blocker. The run is paused only because the user is moving to a new chat.
- Do not treat focused green tests as full Task 00013 acceptance.
- Do not run Cargo commands concurrently or allow relevant writers to edit during compilation/verification.
- Do not broadly stage. Preserve unrelated paths, especially `src/generated/MarketplaceSnapshot.ts`, `notes/`, `output/branding/`, Task 00007 images, superseded Task 00010 images, `wireframes/sites/`, and the preexisting Task 00006 round-trip screenshot modification.
- `git diff --check` currently reports only the preexisting trailing whitespace in `src/generated/MarketplaceSnapshot.ts`; do not repair it as part of Task 00013.
- Do not push, open a pull request, sign, notarize, or distribute.

## Recommended Next Actions

- Action: Re-read the local contracts, original request, Spec 00003 status/coverage/research ledgers, accepted launch/settings contract, current Task 00013 file, and this handoff; then inspect status and the complete uncommitted diff read-only.
  Owner: primary coordinator
  Target timing: first action on resume
- Action: Confirm the latest router and serialized composer changes remain narrow, then run formatter check, lint, typecheck, full frontend unit tests, production/QA builds, and all 42 Playwright tests. Back up and restore the unrelated Task 00006 round-trip screenshot around Playwright.
  Owner: primary coordinator
  Target timing: before Rust full regression
- Action: Freeze writers and run the full Rust workspace regression, protocol checks, native build, and ignored/native acceptance tiers one Cargo-bearing command at a time.
  Owner: primary coordinator
  Target timing: after renderer stabilization
- Action: Perform rebuilt native macOS acceptance, restart/recovery, secret and log scans, and degraded-state inspection.
  Owner: primary coordinator
  Target timing: after the native build is current
- Action: Assign independent reviewers non-overlapping Rust/security, renderer/integration, and policy/configuration acceptance ownership; repair any finding and require P0=0 and P1=0.
  Owner: primary coordinator and independent reviewers
  Target timing: after the full matrix is green
- Action: Update Task 00013 verification/evidence/status and coverage/research ledgers, inspect exact scope, explicitly stage only Task 00013 paths, and create one local checkpoint commit.
  Owner: primary coordinator
  Target timing: only after Agent Acceptance passes
- Action: Continue directly through Tasks 00014 through 00015C and all remaining coverage IDs under the original terminal conditions.
  Owner: primary coordinator
  Target timing: after the Task 00013 checkpoint

## Operational Impact

There is no production outage. This is an uncommitted local development build. The operational risk is loss of auditability or false acceptance if the worktree is broadly staged, generated evidence overwrites unrelated files, Cargo commands overlap, or Task 00013 is marked verified from focused tests alone.

## Continuation Prompt

```text
Resume the governing C4OS Frozen Spec 00003 implementation coordinator run in `/Users/cblanquera/server/projects/cblanquera/c4os` from the paused Task 00013 worktree.

The original request remains authoritative:
`/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`

First read, in order:
1. the root `AGENTS.md` and `.agents/AGENTS.md`
2. `.agents/context/index.md` and `.agents/TERMS.md`
3. the original request above
4. the current Spec 00003 status, task status, coverage ledger, research ledger, and Task 00013 file
5. `.agents/references/00007-launch-and-settings-contract.md`
6. `notes/chat-session-handover/Candace-Techops-chat-session-handover-spec-00003-task-00013-20260723.md`

The branch is `build/mvp-3`. HEAD is the verified Task 00012 checkpoint `eedc3e8`. Tasks 00001-00012 are verified. Task 00013 is substantially implemented but intentionally remains `started` with failed Agent Acceptance; its worktree is uncommitted and must be preserved. No Task 00013 files are staged. The isolated native fixture was stopped for the pause.

Begin with a read-only reconciliation: inspect `git status`, `git diff --check`, and the complete Task 00013 diff. Preserve unrelated changes. In particular, do not touch the preexisting `src/generated/MarketplaceSnapshot.ts` whitespace diff, `notes/`, `output/branding/`, Task 00007 images, superseded Task 00010 images, `wireframes/sites/`, or the preexisting Task 00006 round-trip screenshot modification.

The last full Playwright run was 37/42. All five failures were inspected and repaired: the Terminal generation/draft race now passes its exact scenario 1/1, and the four QA route/Settings composition failures pass 4/4. Do not repeat those focused runs unchanged. Run the complete formatter/lint/type/unit/build matrix on the latest renderer tree, then rerun all 42 Playwright tests. Back up and restore `output/native/task-00006-chat-settings-roundtrip.png` around Playwright; its preserved SHA-256 is `18ba2436c855272e25c55a0de809b1eed54fbddff292c9e9089915eff2b7c1fa`.

After renderer stabilization, freeze relevant writers and run the full Rust regression, protocol checks, native build, and native/ignored acceptance tiers with only one Cargo-bearing command at a time. Then perform rebuilt macOS acceptance, restart/recovery, secret/log scans, and degraded-state checks. Assign fresh independent reviewers explicit non-overlapping Rust/security, renderer/integration, and policy/configuration ownership. Task 00013 may be verified only when Agent Acceptance reports P0=0 and P1=0.

Only then update Task 00013 evidence and the coverage/research/status ledgers, explicitly stage the exact Task 00013-owned paths, and create one scoped local checkpoint commit. Do not push, open a pull request, sign, notarize, distribute, or expand deferred scope. Report the checkpoint and timing breakdown, then continue through Tasks 00014-00015C and all 50 coverage IDs unless the user pauses or redirects the run.
```

## References

- `AGENTS.md`
- `.agents/AGENTS.md`
- `.agents/context/index.md`
- `.agents/TERMS.md`
- `.agents/specs/00003-c4os-macos-application-implementation/status.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/status.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/coverage-ledger.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/research-ledger.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/00013-onboarding-settings-integration.md`
- `.agents/references/00007-launch-and-settings-contract.md`
- `src/app/router.tsx`
- `src/features/shell/ShellRouteController.tsx`
- `src/features/shell/native-bootstrap.ts`
- `src/features/settings/`
- `src/platform/`
- `src-tauri/src/`
- `src-tauri/tests/`
- `/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`
