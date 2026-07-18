# Task 00005 — Native macOS Platform And Semantic UI Foundation

Status: open

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

Result: failed — production native/semantic evidence is absent.

Required evidence: native test results; source/diff inspection; production launch screenshots; live appearance switch; menu/shortcut/picker walkthrough; accessibility tree; width/overflow/contrast matrices; limitations and commands.

## Implementation Notes

Not started. Static r013 grayscale is structural only; accepted semantic Light/Dark supersedes it.

## Verification Notes

Not run.

## Agent Acceptance Notes

Side quest 00015B independently repeats the applicable rendered/native checks.
