# Task 00006 — Stateful Renderer Shell And Accessible Component System

Status: verified

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

Result: passed — independent read-only review reported P0=0 and P1=0 on 2026-07-21.

Required evidence: automated results; state/typed-boundary diff inspection; direct-route screenshots; accessibility tree; keyboard walkthrough; wide/minimum/overlay matrices; no console/overflow; deferred affordance checks; evidence paths/commands and limitations.

Evidence reviewed: production and QA renderer source; typed native snapshot ingestion; Redux authority/draft transitions; all direct routes; Settings round trip; responsive panel and ARIA reconciliation; focused-artifact gating; automated results; exact rebuilt-app accessibility walkthrough; screenshots, timing, limitations, and commands in `output/native/task-00006-acceptance.md`.

## Implementation Notes

Started 2026-07-21 from verified checkpoint `c35b702`. r013 guides behavior and content hierarchy, but its monolithic fixture architecture must not be copied. The smallest production-composed golden path is one client hash-router document and one Redux projection store rendering `/chat` as the stateful desktop shell; native Settings entry must visit a route inside that same router and Back must restore the exact workspace, panel, composer draft, and focus state. Every other accepted top-level destination must then become directly addressable inside the same shell or launch/Settings layout, with QA fixtures build-gated away from production authority.

## Verification Notes

Completed 2026-07-21. Final stabilized gates passed: format, lint, typecheck, 106/106 unit tests, production and QA builds, 26/26 Playwright tests, exact debug `C4OS.app` rebuild, three exact bundled-sidecar checks, and native `/start` to Settings to Back acceptance with clean exit. The complete unchanged Rust regression remains the serialized 239.11s pass from this task epoch; the two final renderer-only repairs reran their focused/full frontend matrices and Tauri's two Cargo sidecar gates rather than blindly repeating unchanged Rust coverage. No new bounded online research was required, so the research ledger remains unchanged.

## Agent Acceptance Notes

The first independent review found P0=0/P1=4: absent production native projection ingestion, a collapsed-to-overlay dead end, inconsistent resizer bounds, and unconditional contextual Chat. The second review accepted three repairs but found retained panel widths were not reconciled after viewport changes. The final frozen diff publishes only validated allowlisted native snapshots, reopens mobile overlays, preserves exact appearance provenance, gates contextual Chat on artifact focus, and reconciles rendered/ARIA/Redux panel widths across 704→340→280 transitions. Final verdict: PASS, P0=0/P1=0. Fractional odd-width maxima may differ from the rounded panel width by less than one pixel; this is non-blocking and does not permit an out-of-range value. Primary coverage remains open only where the ledger names later supporting integration/audit tasks.
