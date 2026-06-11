# Validation Loop

This document defines the evidence loop required before MVP freeze-and-plan or
implementation.

Status as of 2026-06-10: complete for implementation planning. Keep this file
as the validation loop record; use `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`
for the current remaining implementation and release gates.

## Objective

Resolve the remaining BLOCKER and HIGH findings with evidence rather than
assumptions.

The loop ends only when the decision rollups can truthfully remove all BLOCKER and HIGH findings or explicitly reject the current MVP/runtime path.

## Inputs

 - `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`
 - `.agents/archive/planning-corpus/decisions/finalized-decisions.md`
 - `.agents/archive/planning-corpus/reviews/final-implementation-readiness-review.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-001-opencode-runtime-control.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-002-openrouter-verification.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-003-shell-security-policy.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-004-instruction-loading.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-005-runtime-fallback-strategy.md`
 - `.agents/archive/planning-corpus/spikes/SPIKE-006-tauri-platform-validation.md`

## Current Readiness

READY FOR FREEZE-AND-PLAN.

Remaining BLOCKER:

 - None.

Remaining HIGH:

 - None as pre-implementation gates.

Remaining implementation or release validation:

 - FINDING-009 Tauri UI validation after an app build exists.
 - Code-level adapter, shell-policy, credential, and disclosure tests.
 - Product QA journeys after implementation.

## Loop Stages

### Stage 1: Evidence Collection

Run the mandatory research spikes and capture evidence in `.agents/archive/planning-corpus/validation/evidence-matrix.md`.

Required evidence:

 - External PoC review of Odysseus as a research input only, not an app architecture or readiness proof.
 - Runtime-control matrix for OpenCode.
 - Pre-execution interception proof for file writes, shell commands, and runtime-proposed Git state changes.
 - Structured denial-result proof.
 - Stop and child-process supervision proof.
 - Effective OpenRouter provider/model verification proof.
 - OpenCode config precedence and override proof.
 - Runtime-native instruction-loading proof.
 - Shell redaction, truncation, environment filtering, destructive classification, and unclassifiable command policy.
 - Tauri platform matrix and validation result.

### Stage 2: Finding Resolution

For each finding, decide one of:

 - Resolved by evidence.
 - Resolved by scope reduction.
 - Resolved by explicit ADR change.
 - Not resolved, fallback required.
 - Not resolved, MVP path rejected.

Record decisions in `.agents/archive/planning-corpus/validation/decision-log.md`.

### Stage 3: ADR And Decision Rollup Update

Update affected ADRs or decision rollups only after evidence is recorded.

Required updates:

 - `.agents/archive/planning-corpus/decisions/finalized-decisions.md`
 - `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`
 - Any affected standalone ADR files if they remain canonical.
 - Acceptance documents only if Phase 0 gates need to be separated from product QA.

### Stage 4: Readiness Check

Run a readiness pass over:

 - Remaining BLOCKER findings.
 - Remaining HIGH findings.
 - MEDIUM findings that affect build order.
 - Any fallback decision triggered by failed direct OpenCode validation.

If no BLOCKER or HIGH findings remain, the rollup must explicitly state:

READY FOR FREEZE-AND-PLAN

Then freeze-and-plan may generate implementation epics.

## Required Validation Order

1. OpenCode runtime control.
2. OpenRouter runtime verification.
3. Runtime-native instruction loading.
4. Runtime fallback strategy, only if direct OpenCode fails any mandatory gate.
5. Shell security policy.
6. Tauri platform validation.
7. Acceptance gate separation.

## Exit Criteria

The validation loop exits successfully when:

 - FINDING-001 is resolved with evidence or an accepted fallback path.
 - FINDING-002 is resolved through runtime-control evidence.
 - FINDING-003 has concrete shell policy limits.
 - FINDING-004 has runtime-level OpenRouter verification.
 - FINDING-005 has instruction-loading behavior disabled, observed, or disclosed.
 - No BLOCKER or HIGH findings remain in `.agents/archive/planning-corpus/decisions/unresolved-decisions.md`.
 - Any remaining MEDIUM findings are explicitly accepted, deferred, or assigned
   to implementation/release validation.

Current result: exited successfully for implementation planning on 2026-06-10.

The validation loop exits unsuccessfully when:

 - Direct OpenCode fails a mandatory gate and no acceptable fallback preserves the MVP thesis.
 - OpenRouter-only routing cannot be verified or constrained.
 - Runtime-native instruction loading contradicts display-only `AGENTS.md` scope and cannot be controlled.
 - Shell safety cannot be made concrete enough to test.
 - Tauri fails mandatory platform validation and no shell decision is accepted.

## Non-Goals

 - Do not generate implementation tasks.
 - Do not start product epics.
 - Do not build production runtime adapters.
 - Do not weaken safety gates to reach implementation faster.
 - Do not claim readiness without evidence.
