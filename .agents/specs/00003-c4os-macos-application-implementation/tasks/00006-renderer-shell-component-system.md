# Task 00006 — Stateful Renderer Shell And Accessible Component System

Status: open

Coverage: UX-001, UX-004, UX-011; major support for UX-002 through UX-009, UI-002 through UI-005, QA-001.

## Summary

Implement the client-only React hash data router, Redux domain projections/drafts, typed Tauri adapter, accessible unstyled primitives, stateful desktop shell, direct QA routes, Settings round trips, responsive panels, and prohibited-feature gates.

## Implementation Steps

1. Define domain slices/selectors for platform, launch, workspace, sessions, conversation, composer, artifacts, Settings, approvals, runtime, extensions, notices, and QA; authoritative Rust generations win.
2. Implement all top-level direct routes without server fallback and preserve workspace state across Settings and focus transitions.
3. Build reusable React Aria controls, menus, dialogs, popovers, disclosures, tabs, rows, notices, resizers, live regions, loading/error/empty/degraded states, and icon/branding primitives.
4. Build one stateful shell with onboarding/start/workspace/Settings/policy layouts, fixed composer region, no right panel, overlay behavior, and stable focus restoration.
5. Implement visible unavailable `Detach Chat` and negative gates for other deferred behavior.
6. Add component/state/router/accessibility/responsive/overflow tests and QA-only fixture injection separated from production state.

## Verification Process

- Vitest/Testing Library route, reducer, generation, focus, dialog, keyboard, live-region, state matrix, and prohibited-feature tests.
- Playwright direct-route, Settings round-trip, resize, theme, focus, accessibility, reduced-motion, overflow, and console assertions.
- Production build launch and source inspection proving no localStorage authority or arbitrary IPC.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — production renderer evidence is absent.

Required evidence: automated results; state/typed-boundary diff inspection; direct-route screenshots; accessibility tree; keyboard walkthrough; wide/minimum/overlay matrices; no console/overflow; deferred affordance checks; evidence paths/commands and limitations.

## Implementation Notes

Not started. r013 may guide behavior/content hierarchy but its monolithic fixture architecture must not be copied.

## Verification Notes

Not run.

## Agent Acceptance Notes

No coverage closes until real service integration removes simulated behavior.
