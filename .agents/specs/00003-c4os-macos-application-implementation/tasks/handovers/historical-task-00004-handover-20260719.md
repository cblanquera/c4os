# Historical Task 00004 Handoff

> **Superseded historical record.** This handoff captures a paused state from
> 2026-07-19. Task 00004 is now verified with passed Agent Acceptance; use the
> [current Task status](../status.md) and
> [Task 00004 record](../00004-runtime-provider-capability-lifecycle.md) for
> current implementation status.
>
> Source provenance: preserved verbatim in
> [Raw Task 00004 handover](../../../../resources/handovers/Candace-Techops-chat-session-handover-spec-00003-task-00004-20260719.md).

## Summary

This is the paused implementation-coordinator run for Frozen Spec 00003, the C4OS macOS application implementation. The terminal objective remains a working, locally verified, agent-accepted macOS development build covering all 50 normative coverage IDs. Tasks 00001 through 00003 are verified and locally committed. Task 00004, Runtime Adapters, Providers, And Capability Lifecycle, is substantially implemented but is still `started`/`failed` because its latest merged production composition has not completed compilation, focused verification, independent P1 review, rebuilt native QA, or checkpoint commit.

The run was paused at the user's request on 2026-07-19 at approximately 16:21 PST. All three Task 00004 subagents were interrupted. Preserve the current worktree exactly until it has been reconciled and verified.

## Context

- Repository: `/Users/cblanquera/server/projects/cblanquera/c4os`
- Branch: `build/mvp-3`
- Governing package: `.agents/specs/00003-c4os-macos-application-implementation/`
- Current task: `.agents/specs/00003-c4os-macos-application-implementation/tasks/00004-runtime-provider-capability-lifecycle.md`
- Task status source: `.agents/specs/00003-c4os-macos-application-implementation/tasks/status.md`
- Original run request: `/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`
- The original request authorizes bounded parallel subagents, local checkpoint commits, Computer Use, bounded primary-source research, and persistence through the complete Frozen spec. It does not authorize pushing, opening a PR, signing, notarizing, or distributing the app.

Completed local checkpoints:

- `476fb33 Reconcile accepted r013 implementation authority`
- `145cc31 Plan Spec 00003 implementation`
- `bee259f Build verified C4OS foundation`
- `7062daa Build durable C4OS core`
- `eb3477a Build secure C4OS action gateway`

## Scope Boundary

This handoff captures the paused Task 00004 implementation and the sequence needed to resume the full Spec 00003 coordinator run. It does not declare Task 00004 verified, does not stage or commit its worktree, and does not replace the Frozen spec, accepted Knowledge Base, task package, coverage records, research ledger, or executable evidence.

## Current State

Latest known state as of 2026-07-19:

- Task 00001: verified, Agent Acceptance passed.
- Task 00002: verified, Agent Acceptance passed.
- Task 00003: verified, Agent Acceptance passed.
- Task 00004: started, Agent Acceptance still failed until final verification is recorded.
- Tasks 00005 through 00015C: open.
- The Task 00004 worktree has roughly 50 modified or untracked entries. `git diff --check` was green immediately before pausing.
- No Task 00004 files are staged or committed.
- The last complete compile reported by the production-composition subagent was green with three warnings before its latest Pi/provider/approval edits. The warnings were then edited, but a fresh compile was not run before pause.
- A previous native walkthrough used `target/debug/bundle/macos/C4OS.app` and captured provider/model/runtime/recovery/responsive states. Because production host code changed afterward, that bundle and walkthrough are not final evidence and must be rebuilt and repeated.

Major implemented surfaces currently present in the worktree:

- Provider profiles, provider connectivity, model routes, capability resolution/evidence, runtime/session/attempt lifecycle, dispatch authority, persistence, recovery, retry, and stale-event handling.
- OpenCode 1.18.3 launcher, SDK client, broker channel, private credential channel, adapter, process supervision, action routing, native assets, and tests.
- Pi 0.80.10 sidecar, SDK driver, adapter, Action Gateway bridge, credential delivery, process supervision, and tests.
- Descriptor-rooted attachment materialization using `openat` plus `O_NOFOLLOW`, exact digest/length/version checks, bounded native file parts, and workspace-root substitution resistance.
- App-owned `RuntimeProductionApplication<RuntimeProductionBootstrap>`, activation/shutdown/approval commands, background pumping, coherent coordinator/capability snapshots, runtime policy authority, atomic workspace binding, and Drop cleanup.
- Runtime provider/model/recovery renderer states plus native transport integration.

Previously reported green focused evidence, all of which still requires final merged-tree reruns:

- Attachment materializer: 4/4 focused Rust tests.
- OpenCode exact native image/PDF file-part serialization: 1/1.
- Pi SDK exact image path and broker/credential framing: 6/6.
- Runtime dispatch focused suite: 14/14.
- Capability evidence focused suite: 8/8.
- OpenCode credential Rust suite: 3/3, including two C4OS OpenAI profiles sharing native provider `openai` across distinct sessions.
- OpenCode SDK/plugin sidecar suite: 13/13.
- Pi sidecar suite was previously reported 23/23 before the final production-composition edits.
- The deterministic Pi suite previously reported 4 passed and 1 ignored.

Existing native screenshots are under `.agents/resources/native/`:

- `task-00004-providers.jpeg`
- `task-00004-model-preflight-blocked.jpeg`
- `task-00004-model-preflight-compatible.jpeg`
- `task-00004-runtime-selection.jpeg`
- `task-00004-recovery-blocked.jpeg`
- `task-00004-recovery-ready.jpeg`
- `task-00004-recovery-retried.jpeg`
- `task-00004-responsive-820x620.jpeg`

## Key Decisions

- Keep OpenCode and Pi as peer runtime adapters; neither owns C4OS policy, persistence, credentials, or Action Gateway authority.
- Pin OpenCode to 1.18.3 and all maintained Pi packages to 0.80.10 for Task 00004 evidence.
- Deliver OpenCode credentials over a separate private descriptor channel. Do not write `auth.json`, use OpenCode auth APIs, or expose secrets through environment variables, config, arguments, renderer state, logs, or the broker channel.
- Derive provider routes from registered C4OS Provider Profiles. Multiple C4OS profiles may share one native provider ID and must remain isolated by session/profile/generation binding.
- Pi credential delivery must derive the native provider route internally, serialize no vault reference, validate the request before releasing a one-use lease, and bind the complete operation identity through a bounded hash label.
- Attachments must be materialized only from a descriptor-bound workspace root with exact digest/length/version validation. Never trust a renderer-provided ambient path.
- OpenCode supports bounded image/PDF native file parts. Pi 0.80.10 supports native images only; PDF/audio/video remain unsupported unless pinned upstream evidence changes.
- Effective capabilities must be conservative: unknown or unsupported evidence cannot be upgraded by a more optimistic layer.
- Coordinator and capability evidence must be published/snapshotted coherently with one lock order. Runtime policy authority is app-owned rather than hardcoded in the Pi path.
- Zero providers may still permit runtime activation; dispatch must then fail closed. One and many-provider routes must work deterministically.
- Approval answers must resume or deny the native OpenCode/Pi continuation rather than leaving a peer blocked.
- Do not checkpoint Task 00004 until compilation, warning-free validation, native-tier tests, independent P1 review, rebuilt macOS walkthrough, documentation, and Agent Acceptance are complete.

## Conflicts or Changes

- Earlier Task 00004 native screenshots were valid for the renderer states at the time, but production composition changed afterward. They remain useful intermediate evidence, not final acceptance evidence.
- Earlier full/isolated green test reports predate the latest production-host, Pi credential, provider-routing, and approval edits. Treat them as progress evidence and rerun them on the reconciled tree.
- The task status file currently says there are no blockers and describes runtime work as incomplete. More implementation now exists than that prose reflects, but the formal `started`/`failed` status remains correct until verification finishes.

## Open Questions

- Does the latest merged production tree compile with zero warnings after all three subagent edit streams are reconciled?
- Is the generic attachment materializer protected by an aggregate byte limit, not only a per-file and count limit?
- Does the ignored OpenCode loopback native test pass against exact 1.18.3 while proving the real Authorization header and absence of persisted credentials across the process tree?
- Do real-process tests prove process-group and descendant cleanup when attach CAS fails after peer installation?
- Do Pi production tests prove automatic credential delivery, substitution/replay rejection, no vault reference on the wire, invalid-payload no-delivery, and zero/one/many provider behavior?
- Do both OpenCode and Pi approval-continuation tests cover allow and deny?
- Can the packaged Pi production test be strengthened from idle activation/pump/shutdown to one positive dispatch/poll/pump path without weakening determinism?

## Blockers or Risks

- Task 00004 must not be marked verified from source inspection or older test output. Its newest composition has not been compiled or run end to end.
- A fake-peer attach-CAS test is insufficient for descendant cleanup; an actual process-group proof is still required.
- The OpenCode private credential hook has an unavoidable short-lived immutable V8 header string after hook return. Record this precisely as residual risk; do not misstate it as complete in-process zeroization.
- Concurrent mechanical formatting may have touched files owned by multiple subagents. Review diffs by intent before staging.
- Do not use broad staging. The repository may contain unrelated user work, and the run contract requires explicit Task 00004 scope control.
- Do not push, open a PR, sign, notarize, or distribute.

## Recommended Next Actions

- Action: Re-read the root and project-local agent contracts, accepted context index, terminology, Frozen Spec 00003 task/status/coverage package, research ledger, original run request, and this handoff. Then inspect `git status`, `git diff --check`, and the Task 00004 diff without modifying it.
  Owner: primary coordinator
  Target timing: first action on resume
- Action: Reconcile the latest production application, Pi credential, OpenCode credential, and attachment changes. Inspect provider-ID mapping, bounded hashed lease operation, absence of vault refs on the Pi wire, validation-before-delivery, atomic workspace binding, coherent capability snapshots, approval routing, and aggregate attachment bounds.
  Owner: primary coordinator with bounded subagents if useful
  Target timing: immediately after read-only orientation
- Action: Run focused compilation and tests for production application, runtime integration/supervision/dispatch, capability atomicity, OpenCode credential/native/adapter/assets/SDK/broker, Pi adapter/process/gateway/production, attachments, and both sidecar suites. Fix failures in the smallest owning surface.
  Owner: implementation agents; coordinator integrates
  Target timing: before broad validation
- Action: Add or complete real process-group attach-CAS rollback evidence, OpenCode exact loopback Authorization evidence, Pi credential binding/absence/substitution evidence, zero/one/many provider matrices, and allow/deny approval continuation tests.
  Owner: focused implementation agents
  Target timing: before independent audit
- Action: Run an independent P1 audit against the four prior findings: reachable production composition, exact attachment bytes, derived credential delivery, and atomic capability publication; also audit real process cleanup.
  Owner: independent review subagent
  Target timing: after focused suites are green
- Action: Run full Rust and renderer quality gates, build the debug macOS app, run bundle/native ignored tests, relaunch the rebuilt app, and repeat native Computer Use QA for the affected Task 00004 states.
  Owner: primary coordinator
  Target timing: after zero-P1 audit
- Action: Update Task 00004 verification/evidence/residual-risk notes, research ledger, status, and coverage evidence only from current outputs. Inspect explicit scope, stage only Task 00004 paths, and create one local checkpoint commit.
  Owner: primary coordinator
  Target timing: only after Agent Acceptance passes
- Action: Continue directly to Task 00005 and then through Task 00015C until all 50 coverage IDs and final acceptance conditions pass.
  Owner: primary coordinator
  Target timing: after Task 00004 checkpoint

## Operational Impact

There is no production outage: this is an uncommitted local development build. The operational risk is loss of auditability or false acceptance if the paused worktree is reformatted, broadly staged, or marked verified before current native and merged-tree evidence is regenerated.

## Continuation Prompt

```text
Resume the C4OS Frozen Spec 00003 implementation coordinator run in `/Users/cblanquera/server/projects/cblanquera/c4os` from the paused Task 00004 worktree.

First read, in order:
1. the root `AGENTS.md` and `.agents/AGENTS.md`
2. `.agents/context/index.md` and `.agents/TERMS.md`
3. `/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`
4. `.agents/specs/00003-c4os-macos-application-implementation/` task/status/coverage package and research ledger
5. `notes/chat-session-handover/Candace-Techops-chat-session-handover-spec-00003-task-00004-20260719.md`

The branch is `build/mvp-3`. Tasks 00001-00003 are verified and committed through `eb3477a`. Task 00004 is substantially implemented but intentionally remains `started`/`failed`; its worktree is uncommitted and must be preserved. All previous subagents were interrupted when the run paused.

Begin with a read-only reconciliation: inspect `git status`, `git diff --check`, and the complete Task 00004 diff. Do not discard, broadly format, broadly stage, or commit anything. Reconcile and verify the latest production host, Pi/OpenCode credential paths, descriptor-rooted attachments, coherent capability evidence, approval continuation, and real process cleanup. Rerun all focused tests because the last production edits were not compiled after they landed. Then commission an independent zero-P1 audit, run the full Rust/renderer/native quality gates, rebuild `C4OS.app`, and repeat Computer Use QA on the rebuilt bundle.

Only after Task 00004 Agent Acceptance passes should you update its evidence/status, explicitly stage its exact paths, and create a scoped local checkpoint commit. Do not push, open a PR, sign, notarize, or distribute. Continue immediately through Tasks 00005-00015C and all 50 coverage IDs under the original terminal conditions. Keep the user updated during long work and use bounded parallel subagents where their ownership is clear.
```

## References

- `.agents/AGENTS.md`
- `.agents/context/index.md`
- `.agents/TERMS.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/status.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/00004-runtime-provider-capability-lifecycle.md`
- `.agents/specs/00003-c4os-macos-application-implementation/tasks/research-ledger.md`
- `src/backend/src/runtime/`
- `src/backend/tests/`
- `sidecars/`
- `src/features/runtime/`
- `.agents/resources/native/`
- `/Users/cblanquera/.codex/attachments/13459e82-45e4-462f-85a8-0179396a6635/pasted-text.txt`
