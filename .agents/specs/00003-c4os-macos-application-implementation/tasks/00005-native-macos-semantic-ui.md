# Task 00005 — Native macOS Platform And Semantic UI Foundation

Status: verified

Coverage: UX-003, UI-001 through UI-005, SET-003; native support for pickers, Browser, File, theme, focus, and acceptance.

## Summary

Implement the target-qualified macOS PlatformService, native application menu/Settings shortcut, standard window, theme inputs, semantic tokens, controls, typography, focus, scroll, motion, responsive geometry, and native pickers.

## Implementation Steps

1. Implement native platform/theme snapshot before reveal plus independent live `prefers-color-scheme` observation without persisted manual override.
2. Implement semantic Light/Dark tokens and shared component styles; prohibit hardcoded product colors and a second design system.
3. Implement standard decorations, native application menu, `Cmd+,`, Settings action, native file/folder/workspace pickers, Reveal terminology, keyboard symbols, and platform capabilities.
4. Implement accepted density, breakpoints, internal scroll, resizers, focus-visible, popover/dialog containment, reduced motion, and no document overflow.
5. Add native target harnesses for initial/live theme, menu/window/focus/pickers and deterministic permission fixtures.

## Verification Process

- Fresh/repeated launch, wrong-theme-flash, live Light/Dark, state retention, contrast, computed-token, hardcoded-color, reduced-motion, and width matrices.
- Native menu/`Cmd+,`/traffic-light/window/focus/scroll/picker tests and accessibility-tree inspection.
- Production-rendered screenshots in Light/Dark at wide, minimum, overlay, narrow Settings, policies, dialogs, and failure states.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: passed — independent read-only review reported P0=0 and P1=0 on 2026-07-21.

Evidence reviewed: native and renderer test results; source/diff inspection; production launch screenshots; live appearance switch; menu/shortcut/picker walkthrough; accessibility tree; width/overflow/contrast matrices; limitations and commands in `.agents/resources/native/task-00005-acceptance.md`.

## Implementation Notes

Started 2026-07-20 from verified checkpoint `3e06b0b`. Static r013 grayscale is structural only; accepted semantic Light/Dark supersedes it. The first integration target is the smallest production-composed path that resolves the native platform/theme snapshot before reveal, keeps an independent live webview listener, renders a semantic-token QA surface, and reaches native Settings/menu and deterministic picker boundaries without weakening Rust authority.

## Verification Notes

Automated and native verification completed 2026-07-21; see `.agents/resources/native/task-00005-acceptance.md`. The complete renderer and Rust regressions are green, the exact debug app rebuild passed, and the native Light/Dark, menu, shortcut, state-retention, picker, focus, dialog, minimum-width, narrow-Settings, overflow, and accessibility matrices passed. Agent Acceptance is pending an independent read-only review.

## Agent Acceptance Notes

The independent reviewer confirmed fail-closed whole-value runtime publication, opaque picker grants without renderer paths, live-router Settings restoration, native-first/live-webview theme authority, standard macOS semantics, modal focus containment, reduced motion, and responsive internal overflow. Lower-severity observations were the honestly recorded 12.4s/15.8s accessibility availability rather than instrumented first-pixel timing, internally retained runtime initialization errors whose detailed presentation belongs to Task 00014, and the expected supporting integration still owned by Tasks 00006, 00007, 00008, 00013, and 00015B. Side quest 00015B independently repeats the applicable rendered/native checks.
