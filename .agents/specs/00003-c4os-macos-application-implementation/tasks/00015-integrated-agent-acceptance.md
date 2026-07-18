# Task 00015 — Deterministic QA And Integrated Agent Acceptance

Status: open

Coverage: UX-002, UX-009, QA-001, QA-002; integrated closure support for all 50 Feature Coverage IDs.

## Summary

Build the deterministic production QA adapter and perform the complete Agent Acceptance matrix on a launched local macOS development build, including every r013 destination and cross-cutting native, security, persistence, accessibility, failure, recovery, console, and overflow gate.

## Implementation Steps

1. Implement a typed, build-gated fixture schema/adapter with deterministic IDs, clock, data, reset, replay, isolation, and direct-route launcher that cannot masquerade as production state.
2. Implement tiered Rust, renderer, adapter, native, Playwright, accessibility, security, persistence, failure, recovery, and fixture test commands.
3. Build and launch the real application; traverse all 16 r013 destinations plus complete coverage-specific normal and negative paths.
4. Exercise Light/Dark, reduced motion, wide/minimum/overlay widths, keyboard-only operation, focus order/restoration, accessibility tree, native labels, menus, pickers, and failure/recovery states.
5. Assert no uncaught console errors, prohibited overflow, unauthorized side effects, fabricated runtime events, secret exposure, source-only claims, or deferred-feature leakage.
6. Record reproducible evidence, close coverage only when all primary and supporting tasks pass, and rerun the full regression set after fixes.

## Verification Process

- Run every documented deterministic and environment-specific suite with results separated by tier.
- Use Computer Use or an applicable visual browser/native tool for production-rendered walkthroughs and capture evidence artifacts.
- Inspect the production bundle, logs, authoritative databases, audits, diagnostics, and evidence index for truthful boundaries.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — no production build or integrated acceptance evidence exists.

Required evidence: complete command/results index; launched-build identity; 16-route and 50-ID traceability; screenshots and accessibility records; native/keyboard/theme/responsive/failure/recovery matrix; console/overflow/security assertions; evidence paths and limitations.

## Implementation Notes

Not started. Static wireframe QA and proof suites may inform scenarios but never substitute for this production-rendered run.

## Verification Notes

Not run.

## Agent Acceptance Notes

This task passes only after side quests 00015A, 00015B, and 00015C also pass.
