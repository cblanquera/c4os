# Pre-Implementation Prework

Date: 2026-06-10

## Purpose

This is the exact work that had to happen before Phase 1 implementation starts.
It converts the validation loop into an executable checklist and records the
current completion state.

Current state: ready for freeze-and-plan.

The Phase 0 harness exists and passes. OpenCode runtime evidence, OpenRouter
routing evidence, shell policy, fallback decision, platform matrix, and
acceptance gate separation are now recorded in `plans/validation/`.

## Definition Of Ready

Phase 1 implementation may start only when all of these are true:

- FINDING-001 is resolved by evidence or by an accepted fallback runtime path.
- FINDING-002 is resolved through the same runtime-control evidence or fallback.
- FINDING-003 has concrete shell safety limits.
- FINDING-004 has runtime-level OpenRouter/provider/model evidence.
- FINDING-005 has an accepted instruction-loading behavior.
- FINDING-006 is reconciled so the decision rollups no longer conflict.
- FINDING-009 has a named mandatory platform set and initial Tauri validation result, or an explicit deferral.
- FINDING-011 separates Phase 0 gates from ordinary product QA.
- `plans/decisions/unresolved-decisions.md` no longer lists any BLOCKER or HIGH item without a resolution, accepted risk, or explicit deferral.

Status: satisfied for freeze-and-plan on 2026-06-10. App-build validation and
code-level tests remain implementation or release gates.

## Workstream 1: OpenCode Runtime-Control Evidence

Related findings:

- FINDING-001
- FINDING-002
- FINDING-004
- FINDING-005
- FINDING-014

Related files:

- `plans/spikes/SPIKE-001-opencode-runtime-control.md`
- `plans/validation/evidence-matrix.md`
- `tools/phase0/runtime-control-harness.mjs`

Tasks:

1. Identify the supported OpenCode integration surface.
   - Evidence needed: official documentation, source reference, or minimal probe transcript.
   - Done when: the evidence matrix records whether direct runtime control is supported, unsupported, unstable, or undocumented.

2. Prove structured runtime events.
   - Evidence needed: observed structured payloads for session start, assistant output, model call, tool proposal, tool result, denial, error, and stop/interruption.
   - Done when: event payload examples are recorded without relying on terminal scraping.

3. Prove pre-execution interception.
   - Evidence needed: file-write, shell-command, and Git-state-change probes showing the backend can approve or deny before execution.
   - Done when: protected actions either wait for approval or direct OpenCode is rejected.

4. Prove structured denial behavior.
   - Evidence needed: denied action returns to the runtime as denial, not success or silent failure.
   - Done when: the denial result shape is recorded.

5. Prove stop behavior.
   - Evidence needed: active model/runtime work cancels, app-supervised child processes terminate, and partial records survive.
   - Done when: stop/recovery transcript is recorded.

6. Prove app-owned record mapping.
   - Evidence needed: mapping between OpenCode IDs/logs/persistence and app canonical session/tool/message records.
   - Done when: OpenCode state is confirmed as adapter metadata, not source of truth.

7. Prove runtime config isolation.
   - Evidence needed: config precedence matrix showing ambient OpenCode config cannot silently override provider, model, tools, permissions, sessions, or instructions.
   - Done when: override paths are blocked, observable, or treated as fallback-triggering failures.

8. Run the Phase 0 harness with the collected evidence.
   - Evidence needed: evaluator output and command transcript.
   - Done when: `ready` is true, or failed gates point to the fallback decision tree.

Exit decision:

- If every mandatory gate passes: direct OpenCode may continue to freeze-and-plan.
- If any mandatory gate fails: stop direct OpenCode planning and run Workstream 4.

## Workstream 2: OpenRouter Provider And Model Verification

Related findings:

- FINDING-004

Related files:

- `plans/spikes/SPIKE-002-openrouter-verification.md`
- `plans/acceptance/openrouter-integration.md`
- `plans/acceptance/model-providers.md`

Tasks:

1. Prove runtime model calls use OpenRouter.
2. Prove the runtime effective model matches the app-selected model at session start and during active runs.
3. Prove credential-reference stability for active sessions.
4. Prove context boundaries exclude hidden repo ingestion, implicit artifacts, secret-deny contents, raw shell output, and automatic instruction loading unless explicitly accepted.

Exit decision:

- Passed if OpenRouter route and model are observable and constrained.
- Failed if OpenCode can silently route elsewhere or hot-swap credential/model state.

## Workstream 3: Instruction Loading

Related findings:

- FINDING-005

Related files:

- `plans/spikes/SPIKE-004-instruction-loading.md`
- `plans/acceptance/file-access.md`

Tasks:

1. Inventory OpenCode instruction sources and trigger points.
2. Test root and nested `AGENTS.md` behavior.
3. Determine whether instruction loading can be disabled.
4. If it cannot be disabled, determine whether it can be observed and disclosed accurately.
5. Update the MVP rule for root `AGENTS.md` display-only scope.

Exit decision:

- Passed if instruction loading is disabled, observable, or disclosed without contradicting MVP scope.
- Failed if hidden instruction loading affects runtime behavior without control or disclosure.

## Workstream 4: Runtime Fallback Decision Tree

Related findings:

- FINDING-001
- FINDING-014

Related file:

- `plans/spikes/SPIKE-005-runtime-fallback-strategy.md`

Tasks:

1. Define fallback order: wrapper, proxy, fork, runtime replacement, or MVP scope reduction.
2. For each fallback, name the minimum change to preserve the MVP thesis.
3. Record the decision threshold for abandoning direct OpenCode.
4. Update ADR-003 and ADR-004 if fallback is selected.

Exit decision:

- Passed if a failed direct OpenCode gate has an accepted runtime path.
- Failed if no fallback preserves the MVP safety model.

## Workstream 5: Shell Safety Policy

Related findings:

- FINDING-003

Related files:

- `plans/spikes/SPIKE-003-shell-security-policy.md`
- `plans/acceptance/shell-execution.md`
- `plans/specs/security-specification.md`

Tasks:

1. Define exact environment variable keep/strip/conditional policy.
2. Define exact redaction matcher categories and known limitations.
3. Define stdout, stderr, persisted summary, line, and omission limits.
4. Define destructive command categories, examples, non-examples, and one-time approval behavior.
5. Define conservative handling for unclassifiable shell constructs.

Exit decision:

- Passed if the shell policy is concrete enough to implement and test.
- Failed if the policy cannot honestly bound secret exposure or destructive execution.

## Workstream 6: Tauri Platform Validation

Related findings:

- FINDING-009
- FINDING-015

Related file:

- `plans/spikes/SPIKE-006-tauri-platform-validation.md`

Tasks:

1. Name mandatory launch platforms.
2. Name optional and deferred platforms.
3. Validate text-heavy UI and artifact preview feasibility on mandatory platforms.
4. Record any shell/browser/artifact preview limitations.

Exit decision:

- Passed if mandatory platforms are named and viable.
- Deferred only if the deferral is explicit and does not affect Phase 1 architecture.

## Workstream 7: Planning Corpus Cleanup

Related findings:

- FINDING-006
- FINDING-011
- FINDING-012
- FINDING-013

Related files:

- `plans/decisions/implementation-readiness-gaps.md`
- `plans/decisions/finalized-decisions.md`
- `plans/decisions/unresolved-decisions.md`
- `plans/preplan-brief.md`
- `plans/acceptance/*.md`

Tasks:

1. Reconcile old readiness-gap rollup with finalized and deferred decisions.
2. Move Phase 0 gates out of normal product QA acceptance language.
3. Mark plugin and marketplace specs as post-MVP reference only.
4. Record local indefinite retention as an explicit MVP tradeoff or add cleanup scope.
5. Clarify ADR numbering and rollup references enough for implementation planning.

Exit decision:

- Passed if the corpus has one current blocker/high-risk truth source.
- Failed if implementation planning would still read contradictory readiness state.

## Exact Execution Order

1. Run Workstream 1 through the integration-surface and structured-events checks.
2. If direct OpenCode has no usable control surface, stop and run Workstream 4.
3. If direct OpenCode has a usable surface, finish Workstream 1.
4. Run Workstream 2.
5. Run Workstream 3.
6. If any mandatory runtime/provider/instruction gate fails, run Workstream 4.
7. Run Workstream 5.
8. Run Workstream 6.
9. Run Workstream 7.
10. Update `plans/validation/evidence-matrix.md`.
11. Record decisions in `plans/validation/decision-log.md`.
12. Update ADRs and decision rollups.
13. Run the final readiness check.
14. Only if no BLOCKER or unresolved HIGH remains, start freeze-and-plan for implementation epics.

## Can This Be Done Now?

Yes, the validation cycle can start now.

The immediate next task is Workstream 1, task 1: identify the supported OpenCode
integration surface. That will likely require current OpenCode documentation,
source inspection, or a local probe. The Phase 0 harness is ready to classify
the evidence once those results exist.

The pre-implementation checklist is complete enough to start freeze-and-plan.
Do not claim release readiness until the deferred app-build validation,
code-level tests, and product QA journeys pass.
