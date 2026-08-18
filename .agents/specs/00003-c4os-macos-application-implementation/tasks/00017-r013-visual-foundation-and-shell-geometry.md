# Task 00017 — r013 Visual Foundation And Shell Geometry

Status: verified

Coverage: corrective support for UX-001, UI-002, UI-004, UI-005, CHAT-001, SET-002, and SET-003.

## Summary

Restore the r013 visual hierarchy by rebuilding shared production geometry around the r012 shell that r013 preserves. Keep semantic macOS Light/Dark adaptation, React Aria semantics, and production component boundaries while matching r013 shape, density, placement, and responsive relationships as closely as native metrics allow.

## Implementation Steps

1. Inventory r013 shell, launch, Settings, transcript, composer, row, card, dialog, popover, notice, and typography geometry against the current production components and CSS.
2. Consolidate shared layout tokens for the 58px header, 228px project panel, 224px Settings navigation, 920px Settings content cap, 760px composer cap, reading widths, compact headings, spacing, radii, and responsive breakpoints.
3. Replace the generic oversized route-heading treatment with concise route-specific composition. Avoid duplicated route titles, fake native chrome, and component-owned secondary headings that repeat the page purpose.
4. Restore one raised, bordered, width-capped composer container used by the real production Composer rather than only by the fallback shell.
5. Establish compact, scan-oriented Settings rows/cards and one shared Add/Edit dialog geometry without importing the wireframe DOM or creating a second visual system.
6. Verify native Light/Dark, focus, reduced motion, text scaling, internal scrolling, and no document-level overflow after the geometry changes.

## Verification Process

- Component and computed-style tests for shared dimensions, density relationships, heading hierarchy, composer containment, dialog containment, and breakpoint behavior.
- Playwright screenshot and interaction checks at r013 review widths plus the supported native minimum.
- Native macOS Light/Dark inspection with standard decorations, keyboard focus, system controls, and text scaling.
- Source inspection proving semantic tokens and shared components remain the implementation boundary.

## Acceptance Criteria

The user reviews a side-by-side production/r013 foundation board covering onboarding shell, Workspace Start, Settings shell, Chat/transcript/composer, shared dialog, and wide/narrow states in Light and Dark.

## Implementation Notes

Started 2026-07-27. Shared TypeScript geometry constants and semantic CSS geometry tokens now own the accepted shell relationships. The production Composer is contained by the same dock as the fallback, workspace-owned route content bypasses the generic oversized heading, Settings no longer renders fake native titlebar chrome, and shared Settings/modal/scan-row density is converging before route-specific composition.

## Verification Notes

Component verification was followed by the integrated Task 00022 matrix. The final renderer gate passed formatting, lint, typecheck, 80 Vitest files / 472 tests, production and QA builds, and both bundle-authority checks. The 47/47 Playwright matrix passed all accepted routes and widths from 1440px through 390px, including reduced motion, overflow, focus, Settings compression, Browser focus/overlay, and shared Composer geometry. Computer Use inspected 1100x761 QA-native Workspace Start, Chat, Settings, exact Back restoration, and a live Light-to-Dark appearance transition; it also inspected production-native Keychain recovery in both appearances and restored the system to Dark. Native text-size adjustment was not separately toggled; scaling containment remains covered by component and Playwright checks. See `.agents/resources/native/task-00022-review-package.md`.

## Acceptance Notes

Integrated verification passed. Explicit user review of the Task 00022 production/r013 comparison remains pending.
