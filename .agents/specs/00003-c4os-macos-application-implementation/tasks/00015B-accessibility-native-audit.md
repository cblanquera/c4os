# Side Quest 00015B — Accessibility And Native macOS Audit

Status: open

Coverage: independent cross-cutting audit of UX-001 through UX-009, UX-011, UI-001 through UI-005, CHAT-001 through CHAT-008, ART-001 through ART-006, SET-001 through SET-011, QA-001, QA-002.

## Summary

Independently audit the launched application for accessible semantics, keyboard operation, focus, native macOS conventions, Light/Dark behavior, reduced motion, responsive geometry, readable state, console health, and overflow.

## Implementation Steps

1. Exercise all 16 accepted destinations and material dialog/popover/menu/artifact/failure states in Light and Dark.
2. Traverse keyboard-only paths, focus order/traps/restoration, skip/navigation semantics, live regions, editor and terminal interactions, and screen-reader labels.
3. Verify native menu and `Cmd+,`, standard window behavior, native pickers, platform vocabulary, system theme changes, initial reveal, and WKWebView geometry/focus.
4. Exercise wide, minimum-center, and overlay widths plus zoom/text scaling, reduced motion, loading/empty/error/degraded states, internal scroll, and long content.
5. Use Computer Use or the applicable native visual tool, capture evidence, file defects, and rerun after fixes.

## Verification Process

- Automated accessibility, keyboard, theme, responsive, screenshot, console, and overflow tests.
- Production-rendered native walkthrough with accessibility-tree and focus records.
- Comparison against accepted r013 behavior/content hierarchy and Context authority, not pixel imitation.

Acceptance criteria: none — implementation acceptance is delegated to the coordinator’s Agent Acceptance process for Spec 00003.

## Agent Acceptance

Result: failed — a launched production build does not yet exist.

Required evidence: route/state matrix; screenshots; accessibility and focus records; native menu/picker/theme/window observations; width/zoom/reduced-motion results; console and overflow results; defect resolutions and limitations.

## Implementation Notes

Not started. Source-only accessibility claims and static wireframe screenshots do not count.

## Verification Notes

Not run.

## Agent Acceptance Notes

Every in-scope blocking accessibility or native-convention defect must be fixed before passing.
